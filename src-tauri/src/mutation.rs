use std::iter::repeat_n;

use rusqlite::types::Value;
use rusqlite::{params_from_iter, Connection};

use crate::error::{BackendError, INVALID_IDENTITY, INVALID_SQL};
use crate::model::{
    ColumnValue, DeleteRowRequest, InsertRowRequest, MutationResult, RowIdentity, SqlValue, TableColumn, TableMetadata,
    UpdateRowRequest,
};
use crate::query::database_error;
use crate::table::{metadata, quote, rowid_alias};
use crate::value::to_value;

pub fn insert(connection: &Connection, request: &InsertRowRequest) -> Result<MutationResult, BackendError> {
    let table = editable_table(connection, &request.table_name)?;
    validate_values(&table.columns, &request.values)?;
    let sql = if request.values.is_empty() {
        format!("INSERT INTO {} DEFAULT VALUES", quote(&request.table_name))
    } else {
        let names = request.values.iter().map(|value| quote(&value.column)).collect::<Vec<_>>().join(", ");
        let marks = repeat_n("?", request.values.len()).collect::<Vec<_>>().join(", ");
        format!("INSERT INTO {} ({names}) VALUES ({marks})", quote(&request.table_name))
    };
    let values = values(&request.values)?;
    let affected_rows = connection.execute(&sql, params_from_iter(values)).map_err(|error| mutation_error(&error))?;
    let identity = if table.identity_kind == "rowid" {
        Some(RowIdentity::Rowid { value: connection.last_insert_rowid().to_string() })
    } else if table.identity_kind == "primaryKey" {
        let identity_values = if table.primary_key.len() == 1 && integer_primary_key(&table) {
            let key = &table.primary_key[0];
            Some(vec![request.values.iter().find(|value| value.column == *key).map_or_else(
                || SqlValue::Integer { value: connection.last_insert_rowid().to_string() },
                |value| value.value.clone(),
            )])
        } else {
            table
                .primary_key
                .iter()
                .map(|name| request.values.iter().find(|value| value.column == *name).map(|value| value.value.clone()))
                .collect::<Option<Vec<_>>>()
        };
        identity_values.map(|values| RowIdentity::PrimaryKey { values })
    } else {
        None
    };
    Ok(MutationResult { affected_rows, identity })
}

fn integer_primary_key(table: &TableMetadata) -> bool {
    table.primary_key.len() == 1
        && table.columns.iter().any(|column| {
            column.name == table.primary_key[0]
                && column
                    .declared_type
                    .as_deref()
                    .is_some_and(|declared| declared.trim().eq_ignore_ascii_case("INTEGER"))
        })
}

pub fn update(connection: &Connection, request: &UpdateRowRequest) -> Result<MutationResult, BackendError> {
    let table = editable_table(connection, &request.table_name)?;
    validate_expected(&table.columns, &request.expected)?;
    validate_values(&table.columns, &request.changes)?;
    if request.changes.iter().any(|change| table.primary_key.contains(&change.column)) {
        return Err(BackendError::new("primary_key_change_not_supported", "Primary key changes are not supported"));
    }
    validate_identity(&table, &request.identity)?;
    if request.changes.is_empty() {
        return Err(BackendError::new(INVALID_SQL, "Update changes must not be empty"));
    }
    let assignments =
        request.changes.iter().map(|value| format!("{} = ?", quote(&value.column))).collect::<Vec<_>>().join(", ");
    let (predicate, mut bindings) = predicate(&table, &request.identity, &request.expected)?;
    let mut change_values = values(&request.changes)?;
    change_values.append(&mut bindings);
    let sql = format!("UPDATE {} SET {assignments} WHERE {predicate}", quote(&request.table_name));
    exact_one(connection.execute(&sql, params_from_iter(change_values)).map_err(|error| mutation_error(&error))?)
}

pub fn delete(connection: &Connection, request: &DeleteRowRequest) -> Result<MutationResult, BackendError> {
    let table = editable_table(connection, &request.table_name)?;
    validate_expected(&table.columns, &request.expected)?;
    validate_identity(&table, &request.identity)?;
    let (predicate, bindings) = predicate(&table, &request.identity, &request.expected)?;
    let sql = format!("DELETE FROM {} WHERE {predicate}", quote(&request.table_name));
    exact_one(connection.execute(&sql, params_from_iter(bindings)).map_err(|error| mutation_error(&error))?)
}

fn editable_table(connection: &Connection, table_name: &str) -> Result<TableMetadata, BackendError> {
    let table = metadata(connection, table_name)?;
    if !table.editable {
        return Err(BackendError::new(
            "table_not_editable",
            table.non_editable_reason.unwrap_or_else(|| "Table is not editable".to_string()),
        ));
    }
    Ok(table)
}

fn validate_values(columns: &[TableColumn], values: &[ColumnValue]) -> Result<(), BackendError> {
    for value in values {
        if !columns.iter().any(|column| column.name == value.column) {
            return Err(BackendError::new("unknown_column", format!("Column '{}' does not exist", value.column)));
        }
    }
    if values.iter().enumerate().any(|(index, value)| values[..index].iter().any(|prior| prior.column == value.column))
    {
        return Err(BackendError::new("duplicate_column", "Columns may only be supplied once"));
    }
    Ok(())
}

fn validate_expected(columns: &[TableColumn], expected: &[ColumnValue]) -> Result<(), BackendError> {
    validate_values(columns, expected)?;
    if expected.len() != columns.len()
        || columns.iter().any(|column| !expected.iter().any(|value| value.column == column.name))
    {
        return Err(BackendError::new(INVALID_IDENTITY, "Expected values must include every table column"));
    }
    Ok(())
}

fn validate_identity(table: &TableMetadata, identity: &RowIdentity) -> Result<(), BackendError> {
    match identity {
        RowIdentity::PrimaryKey { values }
            if table.identity_kind == "primaryKey" && values.len() == table.primary_key.len() =>
        {
            Ok(())
        }
        RowIdentity::Rowid { value } if table.identity_kind == "rowid" && value.parse::<i64>().is_ok() => Ok(()),
        _ => Err(BackendError::new(INVALID_IDENTITY, "The row identity does not match this table")),
    }
}

fn predicate(
    table: &TableMetadata,
    identity: &RowIdentity,
    expected: &[ColumnValue],
) -> Result<(String, Vec<Value>), BackendError> {
    let (mut identity_sql, mut bindings) = match identity {
        RowIdentity::PrimaryKey { values } => (
            table.primary_key.iter().map(|column| format!("{} IS ?", quote(column))).collect::<Vec<_>>().join(" AND "),
            values.iter().map(to_value).collect::<Result<Vec<_>, _>>()?,
        ),
        RowIdentity::Rowid { value } => (
            format!(
                "{} IS ?",
                quote(
                    &rowid_alias(table)
                        .ok_or_else(|| BackendError::new(INVALID_IDENTITY, "Table has no usable rowid"))?
                )
            ),
            vec![Value::Integer(
                value.parse().map_err(|_| BackendError::new(INVALID_IDENTITY, "Rowid must be an i64"))?,
            )],
        ),
    };
    for expected_value in expected {
        identity_sql.push_str(&format!(" AND {} IS ?", quote(&expected_value.column)));
        bindings.push(to_value(&expected_value.value)?);
    }
    Ok((identity_sql, bindings))
}

fn values(values: &[ColumnValue]) -> Result<Vec<Value>, BackendError> {
    values.iter().map(|value| to_value(&value.value)).collect()
}
fn exact_one(affected_rows: usize) -> Result<MutationResult, BackendError> {
    match affected_rows {
        1 => Ok(MutationResult { affected_rows, identity: None }),
        0 => Err(BackendError::new("row_not_found_or_conflict", "The row was changed or removed")),
        _ => Err(BackendError::new("ambiguous_row_identity", "The row identity matched multiple rows")),
    }
}

fn mutation_error(error: &rusqlite::Error) -> BackendError {
    database_error(&error)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rusqlite::Connection;

    use super::{delete, insert, update};
    use crate::model::{ColumnValue, DeleteRowRequest, InsertRowRequest, RowIdentity, SqlValue, UpdateRowRequest};

    fn value(column: &str, value: SqlValue) -> ColumnValue {
        ColumnValue { column: column.to_string(), value }
    }

    #[test]
    fn inserts_defaults_and_safely_updates_and_deletes_rowids() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch("CREATE TABLE items (name TEXT DEFAULT 'default', note TEXT);")?;
        let inserted = insert(
            &connection,
            &InsertRowRequest { connection_id: 0, table_name: "items".to_string(), values: Vec::new() },
        )?;
        let identity = inserted.identity.ok_or("missing rowid")?;
        let expected =
            vec![value("name", SqlValue::Text { value: "default".to_string() }), value("note", SqlValue::Null)];
        update(
            &connection,
            &UpdateRowRequest {
                connection_id: 0,
                table_name: "items".to_string(),
                identity: identity.clone(),
                expected: expected.clone(),
                changes: vec![value("note", SqlValue::Text { value: "changed".to_string() })],
            },
        )?;
        let stale = delete(
            &connection,
            &DeleteRowRequest {
                connection_id: 0,
                table_name: "items".to_string(),
                identity: identity.clone(),
                expected,
            },
        );
        assert_eq!(stale.err().map(|error| error.code), Some("row_not_found_or_conflict".to_string()));
        let current = vec![
            value("name", SqlValue::Text { value: "default".to_string() }),
            value("note", SqlValue::Text { value: "changed".to_string() }),
        ];
        delete(
            &connection,
            &DeleteRowRequest { connection_id: 0, table_name: "items".to_string(), identity, expected: current },
        )?;
        Ok(())
    }

    #[test]
    fn rejects_primary_key_changes() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch(
            "CREATE TABLE keyed (id INTEGER PRIMARY KEY, value TEXT); INSERT INTO keyed VALUES (1, 'one');",
        )?;
        let result = update(
            &connection,
            &UpdateRowRequest {
                connection_id: 0,
                table_name: "keyed".to_string(),
                identity: RowIdentity::PrimaryKey { values: vec![SqlValue::Integer { value: "1".to_string() }] },
                expected: vec![
                    value("id", SqlValue::Integer { value: "1".to_string() }),
                    value("value", SqlValue::Text { value: "one".to_string() }),
                ],
                changes: vec![value("id", SqlValue::Integer { value: "2".to_string() })],
            },
        );
        assert_eq!(result.err().map(|error| error.code), Some("primary_key_change_not_supported".to_string()));
        Ok(())
    }

    #[test]
    fn returns_generated_and_explicit_integer_primary_key_identities() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch("CREATE TABLE keyed (id INTEGER PRIMARY KEY, value TEXT)")?;
        let generated = insert(
            &connection,
            &InsertRowRequest {
                connection_id: 0,
                table_name: "keyed".to_string(),
                values: vec![value("value", SqlValue::Text { value: "generated".to_string() })],
            },
        )?;
        assert_eq!(
            generated.identity,
            Some(RowIdentity::PrimaryKey { values: vec![SqlValue::Integer { value: "1".to_string() }] })
        );
        let explicit = insert(
            &connection,
            &InsertRowRequest {
                connection_id: 0,
                table_name: "keyed".to_string(),
                values: vec![
                    value("id", SqlValue::Integer { value: "9007199254740993".to_string() }),
                    value("value", SqlValue::Text { value: "explicit".to_string() }),
                ],
            },
        )?;
        assert_eq!(
            explicit.identity,
            Some(RowIdentity::PrimaryKey { values: vec![SqlValue::Integer { value: "9007199254740993".to_string() }] })
        );
        Ok(())
    }
}
