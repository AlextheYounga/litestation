use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use rusqlite::{Connection, Error};

use crate::database::introspect;
use crate::error::{
    BackendError, INVALID_SQL, QUERY_FAILED, QUERY_TIMEOUT, RESULT_COLUMN_LIMIT_EXCEEDED, SQL_ATTACH, SQL_BEGIN,
    SQL_BLOCK_COMMENT_END, SQL_BLOCK_COMMENT_START, SQL_COMMIT, SQL_DETACH, SQL_END, SQL_RELEASE, SQL_ROLLBACK,
    SQL_SAVEPOINT, SQL_TOO_LARGE,
};
use crate::limits::{PayloadBudget, MAX_RESULT_COLUMNS, MAX_SQL_BYTES};
use crate::model::{ExecuteSqlRequest, ExecuteSqlResponse, SqlValue};
use crate::value::from_ref;

pub fn execute(connection: &Connection, request: &ExecuteSqlRequest) -> Result<ExecuteSqlResponse, BackendError> {
    execute_with_timeout(connection, request, Duration::from_secs(10))
}

fn execute_with_timeout(
    connection: &Connection,
    request: &ExecuteSqlRequest,
    timeout_duration: Duration,
) -> Result<ExecuteSqlResponse, BackendError> {
    if request.sql.len() > MAX_SQL_BYTES {
        return Err(BackendError::new(SQL_TOO_LARGE, "SQL request exceeds 1 MiB"));
    }
    if request.sql.trim().is_empty() {
        return Err(BackendError::new(INVALID_SQL, "SQL must contain one statement"));
    }
    if transaction_control(&request.sql) {
        return Err(BackendError::new(
            "transactions_not_supported",
            "Transaction control statements are not supported",
        ));
    }
    if attached_databases(&request.sql) {
        return Err(BackendError::new("attached_databases_not_supported", "Attached databases are not supported"));
    }
    let started = Instant::now();
    let schema_before = schema_version(connection)?;
    let timeout = QueryTimeout::start(connection, timeout_duration);
    let execution = execute_statement(connection, request, &timeout);
    drop(timeout);
    let (columns, rows, affected_rows, truncated) = execution?;
    let schema_changed = schema_before != schema_version(connection)?;
    if schema_changed {
        introspect(connection)?;
    }
    Ok(ExecuteSqlResponse {
        columns,
        rows,
        affected_rows,
        truncated,
        duration_ms: started.elapsed().as_millis(),
        schema_changed,
    })
}

type StatementResult = (Vec<String>, Vec<Vec<SqlValue>>, Option<usize>, bool);

fn execute_statement(
    connection: &Connection,
    request: &ExecuteSqlRequest,
    timeout: &QueryTimeout,
) -> Result<StatementResult, BackendError> {
    let mut statement = connection.prepare(&request.sql).map_err(|error| prepare_error(&error))?;
    if statement.column_count() > MAX_RESULT_COLUMNS {
        return Err(BackendError::new(RESULT_COLUMN_LIMIT_EXCEEDED, "Result exceeds 512 columns"));
    }
    let columns = statement.column_names().iter().map(ToString::to_string).collect::<Vec<_>>();
    let mut budget = PayloadBudget::default();
    for column in &columns {
        budget.add_column(column)?;
    }
    if columns.is_empty() {
        let affected = statement.execute([]).map_err(|error| timeout.database_error(&error))?;
        return Ok((columns, Vec::new(), Some(affected), false));
    }

    let is_read_only_statement = statement.readonly();
    let max_rows = usize::try_from(request.max_rows.unwrap_or(200).clamp(1, 1000)).unwrap_or(1000);
    let mut cursor = statement.query([]).map_err(|error| timeout.database_error(&error))?;
    let mut rows = Vec::new();
    let mut truncated = false;
    let mut conversion_error = None;
    while let Some(row) = cursor.next().map_err(|error| timeout.database_error(&error))? {
        if rows.len() == max_rows || conversion_error.is_some() {
            truncated = true;
            if is_read_only_statement {
                break;
            }
            continue;
        }
        match row_values(row, columns.len(), &mut budget) {
            Ok(values) => rows.push(values),
            Err(error) => {
                conversion_error = Some(error);
                if is_read_only_statement {
                    break;
                }
            }
        }
    }
    if let Some(error) = conversion_error {
        return Err(error);
    }
    Ok((columns, rows, None, truncated))
}

fn row_values(
    row: &rusqlite::Row<'_>,
    column_count: usize,
    budget: &mut PayloadBudget,
) -> Result<Vec<SqlValue>, BackendError> {
    (0..column_count)
        .map(|index| {
            let value = row.get_ref(index).map_err(|error| database_error(&error)).and_then(from_ref)?;
            budget.add_value(&value)?;
            Ok(value)
        })
        .collect()
}

struct QueryTimeout<'connection> {
    connection: &'connection Connection,
    timed_out: Arc<AtomicBool>,
}

impl<'connection> QueryTimeout<'connection> {
    fn start(connection: &'connection Connection, duration: Duration) -> Self {
        let timed_out = Arc::new(AtomicBool::new(false));
        let callback_timed_out = Arc::clone(&timed_out);
        let started = Instant::now();
        connection.progress_handler(
            1_000,
            Some(move || {
                let expired = started.elapsed() >= duration;
                if expired {
                    callback_timed_out.store(true, Ordering::Relaxed);
                }
                expired
            }),
        );
        Self { connection, timed_out }
    }

    fn database_error(&self, error: &Error) -> BackendError {
        if self.timed_out.load(Ordering::Relaxed) {
            BackendError::new(QUERY_TIMEOUT, "SQL execution exceeded 10 seconds")
        } else {
            database_error(error)
        }
    }
}

impl Drop for QueryTimeout<'_> {
    fn drop(&mut self) {
        self.connection.progress_handler(0, None::<fn() -> bool>);
    }
}

fn transaction_control(sql: &str) -> bool {
    let Some(token) = first_token(sql) else { return false };
    is_transaction_keyword(token)
}

pub(crate) fn is_transaction_keyword(token: &str) -> bool {
    matches!(
        token.to_ascii_uppercase().as_str(),
        SQL_BEGIN | SQL_COMMIT | SQL_ROLLBACK | SQL_SAVEPOINT | SQL_RELEASE | SQL_END
    )
}

fn attached_databases(sql: &str) -> bool {
    let Some(token) = first_token(sql) else { return false };
    matches!(token.to_ascii_uppercase().as_str(), SQL_ATTACH | SQL_DETACH)
}

fn first_token(sql: &str) -> Option<&str> {
    let bytes = sql.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"--") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes[index..].starts_with(SQL_BLOCK_COMMENT_START) {
            index += 2;
            while index + 1 < bytes.len() && !bytes[index..].starts_with(SQL_BLOCK_COMMENT_END) {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }
        let remainder = &sql[index..];
        let end = remainder
            .char_indices()
            .find(|(_, character)| character.is_whitespace() || *character == ';')
            .map_or(remainder.len(), |(position, _)| position);
        return (end > 0).then(|| &remainder[..end]);
    }
    None
}

fn schema_version(connection: &Connection) -> Result<i64, BackendError> {
    connection.query_row("PRAGMA schema_version", [], |row| row.get(0)).map_err(|error| database_error(&error))
}

fn prepare_error(error: &Error) -> BackendError {
    if matches!(error, Error::MultipleStatement) {
        BackendError::new("multiple_statements_not_allowed", "Only one SQL statement is allowed")
    } else {
        BackendError::database(INVALID_SQL, &error)
    }
}

pub(crate) fn database_error(error: &Error) -> BackendError {
    use rusqlite::ErrorCode;
    if let Error::SqliteFailure(code, _) = &error {
        return match code.code {
            ErrorCode::ConstraintViolation => BackendError::database("constraint_failed", &error),
            ErrorCode::DatabaseBusy => BackendError::database("database_busy", &error),
            ErrorCode::DatabaseLocked => BackendError::database("database_locked", &error),
            _ => BackendError::database(QUERY_FAILED, &error),
        };
    }
    BackendError::database(QUERY_FAILED, &error)
}

#[cfg(test)]
#[path = "query_tests.rs"]
mod tests;
