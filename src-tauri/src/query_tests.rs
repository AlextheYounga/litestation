use std::error::Error;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

use rusqlite::{functions::FunctionFlags, Connection};

use super::{execute, execute_with_timeout};
use crate::limits::{MAX_RESULT_COLUMNS, MAX_SQL_BYTES};
use crate::model::{ExecuteSqlRequest, SqlValue};

fn request(sql: &str, max_rows: Option<u32>) -> ExecuteSqlRequest {
    ExecuteSqlRequest { connection_id: 0, sql: sql.to_string(), max_rows }
}

#[test]
fn preserves_all_sqlite_value_classes_and_bounds_results() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    connection.execute_batch("CREATE TABLE values_test (i INTEGER, r REAL, t TEXT, b BLOB, n); INSERT INTO values_test VALUES (9007199254740993, 1.5, 'text', x'00ff', NULL); INSERT INTO values_test VALUES (2, 2.5, 'next', x'01', NULL);")?;
    let result = execute(&connection, &request("SELECT * FROM values_test ORDER BY i DESC", Some(1)))?;
    assert!(result.truncated);
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], SqlValue::Integer { value: "9007199254740993".to_string() });
    assert_eq!(result.rows[0][3], SqlValue::Blob { value: "AP8=".to_string() });
    assert_eq!(result.rows[0][4], SqlValue::Null);
    Ok(())
}

#[test]
fn rejects_transaction_and_multiple_statements() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    for sql in ["", ";", "SELECT 1; SELECT 2", "BEGIN", "-- ignored\n COMMIT", "/* ignored */ ROLLBACK"] {
        let result = execute(&connection, &request(sql, None));
        assert!(result.is_err(), "{sql}");
    }
    Ok(())
}

#[test]
fn rejects_attached_databases() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    for sql in [
        "ATTACH ':memory:' AS other",
        "-- ignored\n ATTACH ':memory:' AS other",
        "/* ignored */ ATTACH ':memory:' AS other",
        "DETACH other",
        "-- ignored\n DETACH other",
        "/* ignored */ DETACH other",
    ] {
        let error = execute(&connection, &request(sql, None)).expect_err(sql);
        assert_eq!(error.code, "attached_databases_not_supported", "{sql}");
        assert_eq!(error.message, "Attached databases are not supported");
    }
    Ok(())
}

#[test]
fn drains_returning_rows_and_reports_schema_changes_without_lexical_heuristics() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    connection.execute_batch("CREATE TABLE items (id INTEGER PRIMARY KEY, value INTEGER)")?;
    let result =
        execute(&connection, &request("INSERT INTO items (value) VALUES (1), (2), (3) RETURNING id", Some(1)))?;
    assert!(result.truncated);
    assert_eq!(connection.query_row("SELECT count(*) FROM items", [], |row| row.get::<_, i64>(0))?, 3);
    let result = execute(&connection, &request("UPDATE items SET value = value + 10 RETURNING id", Some(1)))?;
    assert!(result.truncated);
    assert_eq!(
        connection.query_row("SELECT min(value), max(value) FROM items", [], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?,
        (11, 13)
    );
    let result = execute(&connection, &request("/* DDL */ CREATE TABLE second (id INTEGER)", None))?;
    assert!(result.schema_changed);
    assert!(!execute(&connection, &request("UPDATE items SET value = value + 1", None))?.schema_changed);
    Ok(())
}

#[test]
fn rejects_invalid_text_without_lossy_conversion() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    let error =
        execute(&connection, &request("SELECT CAST(x'80' AS TEXT)", None)).expect_err("invalid UTF-8 must fail");
    assert_eq!(error.code, "invalid_text_encoding");
    Ok(())
}

#[test]
fn rejects_oversized_sql_before_preparing() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    let error = execute(&connection, &request(&"x".repeat(MAX_SQL_BYTES + 1), None))
        .expect_err("oversized SQL must be rejected before preparation");
    assert_eq!(error.code, "sql_too_large");
    Ok(())
}

#[test]
fn rejects_too_many_result_columns() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    let sql = format!(
        "SELECT {}",
        (0..=MAX_RESULT_COLUMNS).map(|index| format!("{index} AS c{index}")).collect::<Vec<_>>().join(", ")
    );
    let error = execute(&connection, &request(&sql, None)).expect_err("columns must be capped");
    assert_eq!(error.code, "result_column_limit_exceeded");
    Ok(())
}

#[test]
fn rejects_aggregate_payloads_before_retaining_all_rows() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    let error = execute(
        &connection,
        &request("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x + 1 FROM n WHERE x < 1000) SELECT zeroblob(40000) FROM n", Some(1000)),
    )
    .expect_err("payload must be capped");
    assert_eq!(error.code, "result_payload_too_large");
    Ok(())
}

#[test]
fn select_queries_stop_after_the_truncation_probe() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    connection.create_scalar_function("tick", 1, FunctionFlags::SQLITE_UTF8, move |context| {
        counter.fetch_add(1, Ordering::Relaxed);
        context.get::<i64>(0)
    })?;
    let result = execute(
        &connection,
        &request(
            "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x + 1 FROM n WHERE x < 10000) SELECT tick(x) FROM n",
            Some(1),
        ),
    )?;
    assert!(result.truncated);
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    Ok(())
}

#[test]
fn timeout_interrupts_execution_and_clears_the_progress_handler() -> Result<(), Box<dyn Error>> {
    let connection = Connection::open_in_memory()?;
    let error = execute_with_timeout(
        &connection,
        &request(
            "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x + 1 FROM n WHERE x < 1000000) SELECT x FROM n",
            None,
        ),
        Duration::ZERO,
    )
    .expect_err("short timeout must interrupt query");
    assert_eq!(error.code, "query_timeout");
    assert_eq!(connection.query_row("SELECT 1", [], |row| row.get::<_, i64>(0))?, 1);
    Ok(())
}
