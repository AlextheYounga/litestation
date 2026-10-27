use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, process};

use rusqlite::{types::ValueRef, Connection};

use super::{database_dump_to, rows, sql_literal};
use crate::model::{ExportRowsRequest, SqlValue};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn path() -> String {
    env::temp_dir()
        .join(format!("litestation-export-test-{}-{}.out", process::id(), NEXT.fetch_add(1, Ordering::Relaxed)))
        .to_string_lossy()
        .into_owned()
}

#[test]
fn row_exports_reject_non_finite_and_encoded_limits() {
    let destination = path();
    let non_finite = ExportRowsRequest {
        columns: vec!["real".into()],
        rows: vec![vec![SqlValue::Real { value: f64::INFINITY }]],
        format: "json".into(),
        destination_path: destination.clone(),
    };
    assert_eq!(rows(&non_finite).expect_err("non-finite").code, "non_finite_real");
    let large = ExportRowsRequest {
        columns: vec!["value".into()],
        rows: vec![vec![SqlValue::Text { value: "x".repeat(16 * 1024 * 1024) }]],
        format: "csv".into(),
        destination_path: destination.clone(),
    };
    assert_eq!(rows(&large).expect_err("encoded limit").code, "export_payload_too_large");
    assert!(!Path::new(&destination).exists());
}

#[test]
fn sql_literal_rejects_nan_instead_of_serializing_null() {
    let error = sql_literal(ValueRef::Real(f64::NAN)).expect_err("NaN must be rejected");
    assert_eq!(error.code, "non_finite_real");
}

#[test]
fn dump_preserves_foreign_key_order_sequence_and_generated_columns() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    connection.execute_batch("CREATE TABLE z_parent (id INTEGER PRIMARY KEY AUTOINCREMENT); CREATE TABLE empty_sequence (id INTEGER PRIMARY KEY AUTOINCREMENT); CREATE TABLE a_child (parent INTEGER REFERENCES z_parent(id)); CREATE TABLE generated (base INTEGER, doubled INTEGER GENERATED ALWAYS AS (base * 2)); INSERT INTO z_parent DEFAULT VALUES; INSERT INTO empty_sequence DEFAULT VALUES; DELETE FROM empty_sequence; INSERT INTO a_child VALUES (1); INSERT INTO generated(base) VALUES (3);")?;
    let mut dump = Vec::new();
    let mut count = 0;
    database_dump_to(&mut dump, &connection, &mut count)?;
    let copy = Connection::open_in_memory()?;
    copy.execute_batch(&String::from_utf8(dump)?)?;
    assert_eq!(copy.query_row("SELECT doubled FROM generated", [], |row| row.get::<_, i64>(0))?, 6);
    copy.execute("INSERT INTO empty_sequence DEFAULT VALUES", [])?;
    assert_eq!(copy.last_insert_rowid(), 2);
    assert_eq!(count, 3);
    let _ = fs::remove_file(path());
    Ok(())
}
