use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str;
use std::time::Duration;

use rusqlite::{backup::Backup, Connection, OpenFlags, OptionalExtension};

use crate::database::introspect;
use crate::error::{
    BackendError, DATABASE_OPEN_FAILED, INVALID_PATH, SQL_ATTACH, SQL_BEGIN, SQL_BLOCK_COMMENT_END,
    SQL_BLOCK_COMMENT_START, SQL_COMMIT, SQL_DETACH, SQL_END, SQL_PRAGMA, SQL_RELEASE, SQL_ROLLBACK, SQL_SAVEPOINT,
    SQL_VACUUM,
};
use crate::file_output::StagedOutput;
use crate::model::{ImportSqlDumpResponse, ImportSqliteFileResponse};

const MAX_DUMP_BYTES: u64 = 64 * 1024 * 1024;

pub fn sql_dump(connection: &Connection, source_path: &str) -> Result<ImportSqlDumpResponse, BackendError> {
    let source = regular_file(source_path, "SQL dump")?;
    if source.metadata().map_err(file_error)?.len() > MAX_DUMP_BYTES {
        return Err(BackendError::new("sql_dump_too_large", "SQL dump exceeds the 64 MiB import limit"));
    }
    let dump = fs::read_to_string(&source).map_err(|error| {
        if error.kind() == io::ErrorKind::InvalidData {
            BackendError::new("invalid_utf8_sql_dump", "SQL dump must be UTF-8 encoded")
        } else {
            file_error(error)
        }
    })?;
    let dump = normalize_dump(&dump)?;

    connection.execute_batch("SAVEPOINT litestation_import").map_err(sql_error)?;
    let result = (|| {
        connection.execute_batch("PRAGMA defer_foreign_keys = ON").map_err(sql_error)?;
        connection.execute_batch(&dump).map_err(sql_error)?;
        let violation =
            connection.query_row("PRAGMA foreign_key_check", [], |_| Ok(())).optional().map_err(sql_error)?;
        match violation {
            Some(()) => Err(BackendError::new("foreign_key_check_failed", "SQL dump leaves foreign-key violations")),
            None => {
                connection
                    .execute_batch("PRAGMA defer_foreign_keys = OFF; RELEASE litestation_import")
                    .map_err(sql_error)?;
                Ok(())
            }
        }
    })();
    if let Err(error) = result {
        if let Err(cleanup_error) = cleanup_import(connection) {
            return Err(BackendError::new(
                "import_cleanup_failed",
                format!("SQL dump import failed: {error}; cleanup failed: {cleanup_error}"),
            ));
        }
        return Err(error);
    }
    Ok(ImportSqlDumpResponse { schema: introspect(connection)? })
}

fn cleanup_import(connection: &Connection) -> Result<(), BackendError> {
    let mut cleanup_errors = Vec::new();
    for statement in ["ROLLBACK TO litestation_import", "RELEASE litestation_import", "PRAGMA defer_foreign_keys = OFF"]
    {
        if let Err(error) = connection.execute_batch(statement) {
            cleanup_errors.push(format!("{statement}: {error}"));
        }
    }
    if cleanup_errors.is_empty() && connection.is_autocommit() {
        return Ok(());
    }

    let mut reset_errors = Vec::new();
    if let Err(error) = connection.execute_batch(SQL_ROLLBACK) {
        reset_errors.push(format!("ROLLBACK: {error}"));
    }
    if let Err(error) = connection.execute_batch("PRAGMA defer_foreign_keys = OFF") {
        reset_errors.push(format!("PRAGMA defer_foreign_keys = OFF: {error}"));
    }
    if reset_errors.is_empty() && connection.is_autocommit() {
        return Ok(());
    }

    Err(BackendError::new(
        "import_cleanup_failed",
        format!(
            "Could not restore SQLite connection state (cleanup: {}; reset: {}; autocommit: {})",
            if cleanup_errors.is_empty() {
                "connection remained in a transaction".to_string()
            } else {
                cleanup_errors.join(", ")
            },
            if reset_errors.is_empty() {
                "connection state could not be confirmed".to_string()
            } else {
                reset_errors.join(", ")
            },
            connection.is_autocommit()
        ),
    ))
}

pub fn sqlite_file(source_path: &str, destination: &str) -> Result<ImportSqliteFileResponse, BackendError> {
    let source = fs::canonicalize(regular_file(source_path, "SQLite source")?).map_err(file_error)?;
    let mut staged = StagedOutput::create(destination)?;
    if source == staged_destination(&staged) {
        return Err(BackendError::new(INVALID_PATH, "Source and destination must be different files"));
    }
    let source_connection = Connection::open_with_flags(&source, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| BackendError::database(DATABASE_OPEN_FAILED, &error))?;
    let mut check_statement = source_connection
        .prepare("PRAGMA quick_check")
        .map_err(|error| BackendError::database(DATABASE_OPEN_FAILED, &error))?;
    let checks = check_statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| BackendError::database(DATABASE_OPEN_FAILED, &error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| BackendError::database(DATABASE_OPEN_FAILED, &error))?;
    if checks.as_slice() != ["ok"] {
        return Err(BackendError::new("invalid_sqlite_source", "SQLite source failed PRAGMA quick_check"));
    }
    staged.close_for_external_writer()?;
    let staged_path = staged_path(&staged);
    let mut destination_connection =
        Connection::open(&staged_path).map_err(|error| BackendError::database("database_copy_failed", &error))?;
    let backup = Backup::new(&source_connection, &mut destination_connection)
        .map_err(|error| BackendError::database("database_copy_failed", &error))?;
    backup
        .run_to_completion(128, Duration::ZERO, None)
        .map_err(|error| BackendError::database("database_copy_failed", &error))?;
    drop(backup);
    destination_connection.close().map_err(|(_, error)| BackendError::database("database_copy_failed", &error))?;
    fs::File::open(&staged_path).and_then(|file| file.sync_all()).map_err(file_error)?;
    let path = staged.install()?;
    Ok(ImportSqliteFileResponse {
        path: path.to_string_lossy().into_owned(),
        display_name: path
            .file_name()
            .map_or_else(|| path.to_string_lossy().into_owned(), |name| name.to_string_lossy().into_owned()),
    })
}

// These accessors keep staging paths private to the output primitive while allowing SQLite,
// which opens files itself, to populate the already-exclusive staging entry.
fn staged_path(staged: &StagedOutput) -> PathBuf {
    staged.path_for_sqlite()
}

fn staged_destination(staged: &StagedOutput) -> PathBuf {
    staged.destination_for_comparison()
}

fn regular_file(path: &str, label: &str) -> Result<PathBuf, BackendError> {
    let path = Path::new(path);
    let metadata = fs::metadata(path)
        .map_err(|error| BackendError::new("source_not_found", format!("{label} cannot be read: {error}")))?;
    if !metadata.is_file() {
        return Err(BackendError::new("source_not_file", format!("{label} must be a regular file")));
    }
    Ok(path.to_path_buf())
}

fn normalize_dump(dump: &str) -> Result<String, BackendError> {
    let mut lines = dump.lines().collect::<Vec<_>>();
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    if lines.first().is_some_and(|line| line.trim() == "PRAGMA foreign_keys=OFF;") {
        lines.remove(0);
    }
    let starts = lines.first().is_some_and(|line| exact_wrapper(line, &format!("{SQL_BEGIN} TRANSACTION")));
    let ends = lines.last().is_some_and(|line| exact_wrapper(line, SQL_COMMIT));
    if starts != ends {
        return Err(unsafe_dump_error());
    }
    if starts {
        lines.remove(0);
        lines.pop();
    }
    let normalized = lines.join("\n");
    reject_unsafe_statements(&normalized)?;
    Ok(normalized)
}

fn exact_wrapper(line: &str, statement: &str) -> bool {
    line.trim() == format!("{statement};")
}

fn reject_unsafe_statements(sql: &str) -> Result<(), BackendError> {
    let mut scanner = StatementScanner::new(sql);
    while let Some(keyword) = scanner.next_statement_keyword() {
        if [
            SQL_ATTACH,
            SQL_DETACH,
            SQL_VACUUM,
            SQL_PRAGMA,
            SQL_BEGIN,
            SQL_COMMIT,
            SQL_END,
            SQL_ROLLBACK,
            SQL_SAVEPOINT,
            SQL_RELEASE,
        ]
        .contains(&keyword.as_str())
        {
            return Err(unsafe_dump_error());
        }
    }
    Ok(())
}

fn unsafe_dump_error() -> BackendError {
    BackendError::new(
        "unsupported_dump_statement",
        "SQL dumps may not contain ATTACH, DETACH, VACUUM, PRAGMA, or transaction-control statements",
    )
}

/// Finds only top-level statement keywords; trigger-body semicolons stay within CREATE TRIGGER.
struct StatementScanner<'a> {
    sql: &'a [u8],
    position: usize,
}

impl<'a> StatementScanner<'a> {
    fn new(sql: &'a str) -> Self {
        Self { sql: sql.as_bytes(), position: 0 }
    }

    fn next_statement_keyword(&mut self) -> Option<String> {
        self.skip_space_comments();
        if self.position >= self.sql.len() {
            return None;
        }
        let start = self.position;
        while self.position < self.sql.len() && self.sql[self.position].is_ascii_alphabetic() {
            self.position += 1;
        }
        let keyword = str::from_utf8(&self.sql[start..self.position]).ok()?.to_ascii_uppercase();
        let trigger = keyword == "CREATE" && self.remaining_is_trigger();
        self.skip_statement(trigger);
        Some(keyword)
    }

    fn remaining_is_trigger(&self) -> bool {
        let rest = String::from_utf8_lossy(&self.sql[self.position..]).to_ascii_uppercase();
        rest.split_whitespace().take(4).any(|word| word == "TRIGGER")
    }

    fn skip_statement(&mut self, trigger: bool) {
        let mut quote = None;
        let mut trigger_end = false;
        while self.position < self.sql.len() {
            let byte = self.sql[self.position];
            if let Some(delimiter) = quote {
                if byte == delimiter {
                    if delimiter == b'\'' && self.sql.get(self.position + 1) == Some(&b'\'') {
                        self.position += 2;
                        continue;
                    }
                    quote = None;
                }
            } else if matches!(byte, b'\'' | b'"' | b'`') {
                quote = Some(byte);
            } else if self.sql.get(self.position..self.position + 2) == Some(b"--") {
                while self.position < self.sql.len() && self.sql[self.position] != b'\n' {
                    self.position += 1;
                }
                continue;
            } else if self.sql.get(self.position..self.position + 2) == Some(SQL_BLOCK_COMMENT_START) {
                self.position += 2;
                while self.sql.get(self.position..self.position + 2) != Some(SQL_BLOCK_COMMENT_END)
                    && self.position < self.sql.len()
                {
                    self.position += 1;
                }
                self.position += 2;
                continue;
            } else if byte == b';' && (!trigger || trigger_end) {
                self.position += 1;
                return;
            } else if trigger
                && self
                    .sql
                    .get(self.position..self.position + SQL_END.len())
                    .is_some_and(|word| word.eq_ignore_ascii_case(SQL_END.as_bytes()))
            {
                let before = self
                    .position
                    .checked_sub(1)
                    .and_then(|index| self.sql.get(index))
                    .is_none_or(|byte| !byte.is_ascii_alphanumeric());
                let after =
                    self.sql.get(self.position + SQL_END.len()).is_none_or(|byte| !byte.is_ascii_alphanumeric());
                trigger_end = before && after;
            }
            self.position += 1;
        }
    }

    fn skip_space_comments(&mut self) {
        loop {
            while self.position < self.sql.len()
                && (self.sql[self.position].is_ascii_whitespace() || self.sql[self.position] == b';')
            {
                self.position += 1;
            }
            if self.sql.get(self.position..self.position + 2) == Some(b"--") {
                while self.position < self.sql.len() && self.sql[self.position] != b'\n' {
                    self.position += 1;
                }
            } else if self.sql.get(self.position..self.position + 2) == Some(SQL_BLOCK_COMMENT_START) {
                self.position += 2;
                while self.sql.get(self.position..self.position + 2) != Some(SQL_BLOCK_COMMENT_END)
                    && self.position < self.sql.len()
                {
                    self.position += 1;
                }
                self.position += 2;
            } else {
                return;
            }
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
fn sql_error(error: rusqlite::Error) -> BackendError {
    BackendError::database("sql_dump_import_failed", &error)
}
#[expect(clippy::needless_pass_by_value)]
fn file_error(error: io::Error) -> BackendError {
    BackendError::new("file_read_failed", format!("Could not read source file: {error}"))
}

#[cfg(test)]
#[path = "import_tests.rs"]
mod tests;
