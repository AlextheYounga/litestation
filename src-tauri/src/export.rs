use std::io::{self, Write};

use rusqlite::{types::ValueRef, Connection, OptionalExtension};
use serde_json::{json, Value};

use crate::error::{BackendError, INVALID_TEXT_ENCODING, SQL_BEGIN, SQL_ROLLBACK};
use crate::file_output::write_new;
use crate::model::{ExportRowsRequest, ExportRowsResponse, ExportSqlDumpResponse, SqlValue};
use crate::table::quote;

const MAX_COLUMNS: usize = 256;
const MAX_ROWS: usize = 1000;
const MAX_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
const MAX_COLUMN_NAME_BYTES: usize = 64 * 1024;

pub fn rows(request: &ExportRowsRequest) -> Result<ExportRowsResponse, BackendError> {
    validate_rows(request)?;
    let encoded = match request.format.as_str() {
        "json" => encode_json(request)?,
        "csv" => encode_csv(request)?,
        _ => return Err(BackendError::new("unsupported_export_format", "Export format must be json or csv")),
    };
    let path = write_new(&request.destination_path, |file| file.write_all(&encoded).map_err(write_error))?;
    Ok(ExportRowsResponse { path: path.to_string_lossy().into_owned(), row_count: request.rows.len() })
}

pub fn sql_dump(connection: &Connection, destination: &str) -> Result<ExportSqlDumpResponse, BackendError> {
    let mut row_count = 0;
    let path = write_new(destination, |file| {
        with_read_snapshot(connection, || database_dump_to(file, connection, &mut row_count))
    })?;
    Ok(ExportSqlDumpResponse { path: path.to_string_lossy().into_owned(), row_count })
}

fn validate_rows(request: &ExportRowsRequest) -> Result<(), BackendError> {
    if request.columns.len() > MAX_COLUMNS || request.rows.len() > MAX_ROWS {
        return Err(BackendError::new("export_limit_exceeded", "Exports are limited to 256 columns and 1000 rows"));
    }
    if request.columns.iter().any(|column| column.len() > MAX_COLUMN_NAME_BYTES) {
        return Err(BackendError::new("export_limit_exceeded", "Export column names are limited to 64 KiB"));
    }
    if request.rows.iter().any(|row| row.len() != request.columns.len()) {
        return Err(BackendError::new("invalid_export_shape", "Every exported row must match the column count"));
    }
    if request.rows.iter().flatten().any(|value| matches!(value, SqlValue::Real { value } if !value.is_finite())) {
        return Err(BackendError::new("non_finite_real", "JSON and CSV exports do not support non-finite REAL values"));
    }
    Ok(())
}

fn encode_json(request: &ExportRowsRequest) -> Result<Vec<u8>, BackendError> {
    let data = json!({ "columns": request.columns, "rows": json_rows(&request.rows) });
    let mut output = LimitedBuffer::default();
    serde_json::to_writer(&mut output, &data).map_err(json_error)?;
    Ok(output.into_inner())
}

fn encode_csv(request: &ExportRowsRequest) -> Result<Vec<u8>, BackendError> {
    let mut output = LimitedBuffer::default();
    write_csv(&mut output, &request.columns, &request.rows)?;
    Ok(output.into_inner())
}

#[derive(Default)]
struct LimitedBuffer(Vec<u8>);

impl LimitedBuffer {
    fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl Write for LimitedBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > MAX_PAYLOAD_BYTES {
            return Err(io::Error::new(io::ErrorKind::FileTooLarge, "encoded export exceeds 16 MiB"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn json_rows(rows: &[Vec<SqlValue>]) -> Vec<Vec<Value>> {
    rows.iter().map(|row| row.iter().map(json_value).collect()).collect()
}

fn json_value(value: &SqlValue) -> Value {
    match value {
        SqlValue::Null => Value::Null,
        SqlValue::Integer { value } | SqlValue::Text { value } => Value::String(value.clone()),
        SqlValue::Real { value } => serde_json::Number::from_f64(*value).map_or(Value::Null, Value::Number),
        SqlValue::Blob { value } => json!({ "type": "blob", "base64": value }),
    }
}

fn write_csv(file: &mut impl Write, columns: &[String], rows: &[Vec<SqlValue>]) -> Result<(), BackendError> {
    write_csv_record(file, columns.iter().map(String::as_str))?;
    for row in rows {
        let fields = row.iter().map(csv_value).collect::<Vec<_>>();
        write_csv_record(file, fields.iter().map(String::as_str))?;
    }
    Ok(())
}

fn write_csv_record<'a>(file: &mut impl Write, fields: impl Iterator<Item = &'a str>) -> Result<(), BackendError> {
    for (index, field) in fields.enumerate() {
        if index > 0 {
            file.write_all(b",").map_err(encode_error)?;
        }
        let quoted = field.contains([',', '"', '\r', '\n']);
        if quoted {
            file.write_all(b"\"").map_err(encode_error)?;
        }
        file.write_all(field.replace('"', "\"\"").as_bytes()).map_err(encode_error)?;
        if quoted {
            file.write_all(b"\"").map_err(encode_error)?;
        }
    }
    file.write_all(b"\r\n").map_err(encode_error)
}

fn csv_value(value: &SqlValue) -> String {
    match value {
        SqlValue::Null => String::new(),
        SqlValue::Integer { value } | SqlValue::Text { value } => value.clone(),
        SqlValue::Real { value } => value.to_string(),
        SqlValue::Blob { value } => format!("base64:{value}"),
    }
}

fn with_read_snapshot<T>(
    connection: &Connection,
    action: impl FnOnce() -> Result<T, BackendError>,
) -> Result<T, BackendError> {
    connection.execute_batch(SQL_BEGIN).map_err(query_error)?;
    let result = action();
    let rollback = connection.execute_batch(SQL_ROLLBACK).map_err(query_error);
    match (result, rollback) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
}

fn database_dump_to(
    output: &mut impl Write,
    connection: &Connection,
    row_count: &mut usize,
) -> Result<(), BackendError> {
    reject_virtual_tables(connection)?;
    output.write_all(b"PRAGMA foreign_keys=OFF;\nBEGIN TRANSACTION;\n").map_err(write_error)?;
    let tables = objects(connection, "table")?;
    for (name, sql) in &tables {
        write_statement(output, sql)?;
        let columns = columns(connection, name)?;
        if columns.is_empty() {
            continue;
        }
        let names = columns.iter().map(|column| quote(column)).collect::<Vec<_>>().join(", ");
        let select = format!("SELECT {names} FROM {} ORDER BY {names}", quote(name));
        let mut statement = connection.prepare(&select).map_err(query_error)?;
        let mut rows = statement.query([]).map_err(query_error)?;
        while let Some(row) = rows.next().map_err(query_error)? {
            let values = (0..columns.len())
                .map(|index| row.get_ref(index).map_err(query_error).and_then(sql_literal))
                .collect::<Result<Vec<_>, _>>()?;
            writeln!(output, "INSERT INTO {} ({names}) VALUES ({});", quote(name), values.join(", "))
                .map_err(write_error)?;
            *row_count += 1;
        }
    }
    write_sequences(output, connection)?;
    for object_type in ["view", "index", "trigger"] {
        for (_, sql) in objects(connection, object_type)? {
            write_statement(output, &sql)?;
        }
    }
    output.write_all(b"COMMIT;\n").map_err(write_error)
}

fn reject_virtual_tables(connection: &Connection) -> Result<(), BackendError> {
    let virtual_table: Option<String> = connection
        .query_row("SELECT name FROM pragma_table_list WHERE schema = 'main' AND type = 'virtual' LIMIT 1", [], |row| {
            row.get(0)
        })
        .optional()
        .map_err(query_error)?;
    if virtual_table.is_some() {
        return Err(BackendError::new(
            "unsupported_virtual_table_dump",
            "Virtual tables cannot be exported as SQL dumps",
        ));
    }
    Ok(())
}

fn write_sequences(output: &mut impl Write, connection: &Connection) -> Result<(), BackendError> {
    let exists: Option<()> = connection
        .query_row("SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'sqlite_sequence'", [], |_| Ok(()))
        .optional()
        .map_err(query_error)?;
    if exists.is_none() {
        return Ok(());
    }
    let mut statement = connection
        .prepare("SELECT name, seq FROM sqlite_sequence ORDER BY name COLLATE BINARY")
        .map_err(query_error)?;
    let sequences =
        statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))).map_err(query_error)?;
    for sequence in sequences {
        let (name, value) = sequence.map_err(query_error)?;
        writeln!(
            output,
            "INSERT OR REPLACE INTO sqlite_sequence(name, seq) VALUES ({}, {value});",
            string_literal(&name)
        )
        .map_err(write_error)?;
    }
    Ok(())
}

fn write_statement(output: &mut impl Write, sql: &str) -> Result<(), BackendError> {
    output.write_all(sql.as_bytes()).map_err(write_error)?;
    output.write_all(b";\n").map_err(write_error)
}

fn objects(connection: &Connection, object_type: &str) -> Result<Vec<(String, String)>, BackendError> {
    let mut statement = connection.prepare("SELECT name, sql FROM sqlite_schema WHERE type = ?1 AND name NOT LIKE 'sqlite_%' ORDER BY name COLLATE BINARY").map_err(query_error)?;
    let objects = statement
        .query_map([object_type], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)?;
    Ok(objects)
}

fn columns(connection: &Connection, table: &str) -> Result<Vec<String>, BackendError> {
    let mut statement = connection
        .prepare("SELECT name FROM pragma_table_xinfo(?1) WHERE hidden = 0 ORDER BY cid")
        .map_err(query_error)?;
    let columns = statement
        .query_map([table], |row| row.get(0))
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)?;
    Ok(columns)
}

fn sql_literal(value: ValueRef<'_>) -> Result<String, BackendError> {
    match value {
        ValueRef::Null => Ok("NULL".to_string()),
        ValueRef::Integer(value) => Ok(value.to_string()),
        ValueRef::Real(value) if value.is_nan() => {
            Err(BackendError::new("non_finite_real", "SQL dump exports do not support NaN REAL values"))
        }
        ValueRef::Real(value) if value.is_infinite() => {
            Ok(if value.is_sign_positive() { "1e999" } else { "-1e999" }.to_string())
        }
        ValueRef::Real(value) => Ok(format!("{value:?}")),
        ValueRef::Text(value) => String::from_utf8(value.to_vec())
            .map(|text| string_literal(&text))
            .map_err(|_| BackendError::new(INVALID_TEXT_ENCODING, "SQLite TEXT contains invalid UTF-8")),
        ValueRef::Blob(value) => {
            Ok(format!("X'{}'", value.iter().map(|byte| format!("{byte:02X}")).collect::<String>()))
        }
    }
}

fn string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn json_error(error: serde_json::Error) -> BackendError {
    encode_error(error.into())
}
#[expect(clippy::needless_pass_by_value)]
fn encode_error(error: io::Error) -> BackendError {
    if error.kind() == io::ErrorKind::FileTooLarge {
        BackendError::new("export_payload_too_large", "Encoded export exceeds the 16 MiB limit")
    } else {
        BackendError::new("export_write_failed", format!("Could not encode export: {error}"))
    }
}
#[expect(clippy::needless_pass_by_value)]
fn write_error(error: io::Error) -> BackendError {
    BackendError::new("export_write_failed", format!("Could not write export: {error}"))
}
#[expect(clippy::needless_pass_by_value)]
fn query_error(error: rusqlite::Error) -> BackendError {
    BackendError::database("sql_dump_export_failed", &error)
}

#[cfg(test)]
#[path = "export_tests.rs"]
mod tests;
