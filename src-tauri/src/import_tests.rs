use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, process};

use rusqlite::Connection;

use super::{sql_dump, sqlite_file};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn path(extension: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "litestation-import-test-{}-{}.{}",
        process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        extension
    ))
}

#[test]
fn conventional_dumps_defer_foreign_keys_and_reject_unsafe_statements() -> Result<(), Box<dyn Error>> {
    let source = path("sql");
    fs::write(&source, "PRAGMA foreign_keys=OFF;\nBEGIN TRANSACTION;\nCREATE TABLE parent (id INTEGER PRIMARY KEY);\nCREATE TABLE child (id INTEGER REFERENCES parent(id));\nINSERT INTO child VALUES (1);\nINSERT INTO parent VALUES (1);\nCOMMIT;\n")?;
    let connection = Connection::open_in_memory()?;
    sql_dump(&connection, source.to_string_lossy().as_ref())?;
    assert_eq!(connection.query_row("SELECT count(*) FROM child", [], |row| row.get::<_, i64>(0))?, 1);
    for sql in
        ["ATTACH DATABASE ':memory:' AS elsewhere;", "DETACH DATABASE elsewhere;", "PRAGMA cache_size=10;", "VACUUM;"]
    {
        fs::write(&source, sql)?;
        assert_eq!(
            sql_dump(&connection, source.to_string_lossy().as_ref()).expect_err("unsafe statement").code,
            "unsupported_dump_statement"
        );
    }
    fs::remove_file(&source)?;
    Ok(())
}

#[test]
fn failed_import_cleans_up_connection_for_reuse() -> Result<(), Box<dyn Error>> {
    let source = path("sql");
    fs::write(&source, "CREATE TABLE created (value); INSERT INTO missing VALUES (1);")?;
    let connection = Connection::open_in_memory()?;
    let error = sql_dump(&connection, source.to_string_lossy().as_ref()).expect_err("invalid import");
    assert_eq!(error.code, "sql_dump_import_failed");
    assert!(connection.is_autocommit());
    connection.execute("CREATE TABLE usable (value)", [])?;
    fs::remove_file(source)?;
    Ok(())
}

#[test]
fn sqlite_backup_validates_and_never_replaces_destination() -> Result<(), Box<dyn Error>> {
    let source = path("db");
    let destination = path("db");
    let connection = Connection::open(&source)?;
    connection
        .execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE items (value); INSERT INTO items VALUES ('visible');")?;
    drop(connection);
    sqlite_file(source.to_string_lossy().as_ref(), destination.to_string_lossy().as_ref())?;
    let copy = Connection::open(&destination)?;
    assert_eq!(copy.query_row("SELECT value FROM items", [], |row| row.get::<_, String>(0))?, "visible");
    drop(copy);
    assert_eq!(
        sqlite_file(source.to_string_lossy().as_ref(), destination.to_string_lossy().as_ref())
            .expect_err("existing destination")
            .code,
        "destination_exists"
    );
    let invalid = path("db");
    fs::write(&invalid, "not sqlite")?;
    let invalid_destination = path("db");
    assert!(sqlite_file(invalid.to_string_lossy().as_ref(), invalid_destination.to_string_lossy().as_ref()).is_err());
    assert!(!invalid_destination.exists());
    fs::remove_file(&source)?;
    let _ = fs::remove_file(source.with_extension("db-wal"));
    fs::remove_file(destination)?;
    fs::remove_file(invalid)?;
    Ok(())
}
