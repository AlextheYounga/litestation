use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::error::BackendError;
use crate::model::{DatabaseMode, DatabaseSchema, SchemaItem};

pub struct OpenedDatabase {
    pub path: PathBuf,
    pub mode: DatabaseMode,
    pub schema: DatabaseSchema,
    pub connection: Connection,
}

pub fn open(path: &str, mode: DatabaseMode) -> Result<OpenedDatabase, BackendError> {
    let input_path = Path::new(path);
    let metadata = fs::metadata(input_path)
        .map_err(|error| BackendError::new("path_not_found", format!("Database path cannot be read: {error}")))?;
    if !metadata.is_file() {
        return Err(BackendError::new("path_not_file", "Database path is not a regular file"));
    }

    let canonical_path = fs::canonicalize(input_path).map_err(|error| {
        BackendError::new("path_canonicalize_failed", format!("Database path cannot be resolved: {error}"))
    })?;
    let flags = match mode {
        DatabaseMode::ReadOnly => OpenFlags::SQLITE_OPEN_READ_ONLY,
        DatabaseMode::ReadWrite => OpenFlags::SQLITE_OPEN_READ_WRITE,
    };
    let connection = Connection::open_with_flags(&canonical_path, flags)
        .map_err(|error| BackendError::database("database_open_failed", &error))?;
    let schema = introspect(&connection)?;

    Ok(OpenedDatabase { path: canonical_path, mode, schema, connection })
}

fn introspect(connection: &Connection) -> Result<DatabaseSchema, BackendError> {
    let mut schema = DatabaseSchema::default();
    schema.tables = objects_of_type(connection, "table")?;
    schema.views = objects_of_type(connection, "view")?;
    schema.indexes = objects_of_type(connection, "index")?;
    schema.triggers = objects_of_type(connection, "trigger")?;
    Ok(schema)
}

fn objects_of_type(connection: &Connection, object_type: &str) -> Result<Vec<SchemaItem>, BackendError> {
    let mut statement = connection
        .prepare("SELECT name, tbl_name, sql FROM sqlite_schema WHERE type = ?1 AND name NOT LIKE 'sqlite_%' ORDER BY name COLLATE BINARY")
        .map_err(|error| BackendError::database("schema_query_failed", &error))?;
    let rows = statement
        .query_map([object_type], |row| Ok(SchemaItem { name: row.get(0)?, table_name: row.get(1)?, sql: row.get(2)? }))
        .map_err(|error| BackendError::database("schema_query_failed", &error))?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|error| BackendError::database("schema_query_failed", &error))
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::{env, process};

    use rusqlite::Connection;

    use super::open;
    use crate::model::DatabaseMode;

    static NEXT_TEST_FILE: AtomicUsize = AtomicUsize::new(0);

    fn test_path(label: &str) -> PathBuf {
        let number = NEXT_TEST_FILE.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!("litestation-{label}-{}-{number}.db", process::id()))
    }

    fn remove(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn opens_and_groups_schema_in_deterministic_order() -> Result<(), Box<dyn Error>> {
        let path = test_path("schema");
        remove(&path);
        let connection = Connection::open(&path)?;
        connection.execute_batch(
            "CREATE TABLE zeta (id INTEGER); CREATE TABLE alpha (id INTEGER); CREATE VIEW view_one AS SELECT id FROM alpha; CREATE INDEX index_zeta ON zeta(id); CREATE TRIGGER trigger_zeta AFTER INSERT ON zeta BEGIN SELECT 1; END;",
        )?;
        drop(connection);

        let opened = open(path.to_string_lossy().as_ref(), DatabaseMode::ReadOnly)?;
        assert_eq!(opened.schema.tables.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(), ["alpha", "zeta"]);
        assert_eq!(opened.schema.views.len(), 1);
        assert_eq!(opened.schema.indexes.len(), 1);
        assert_eq!(opened.schema.triggers.len(), 1);
        assert_eq!(opened.path, fs::canonicalize(&path)?);
        drop(opened);
        remove(&path);
        Ok(())
    }

    #[test]
    fn rejects_malformed_database_during_open() {
        let path = test_path("malformed");
        remove(&path);
        assert!(fs::write(&path, b"not sqlite").is_ok());
        let result = open(path.to_string_lossy().as_ref(), DatabaseMode::ReadOnly);
        assert!(result.is_err());
        assert_eq!(result.err().map(|error| error.code), Some("schema_query_failed".to_string()));
        remove(&path);
    }

    #[test]
    fn read_only_connection_rejects_writes() -> Result<(), Box<dyn Error>> {
        let path = test_path("readonly");
        remove(&path);
        let connection = Connection::open(&path)?;
        connection.execute_batch("CREATE TABLE items (id INTEGER)")?;
        drop(connection);
        let opened = open(path.to_string_lossy().as_ref(), DatabaseMode::ReadOnly)?;
        assert!(opened.connection.execute("INSERT INTO items VALUES (1)", []).is_err());
        drop(opened);
        remove(&path);
        Ok(())
    }

    #[test]
    fn writable_connection_does_not_create_missing_file() {
        let path = test_path("missing");
        remove(&path);
        let result = open(path.to_string_lossy().as_ref(), DatabaseMode::ReadWrite);
        assert!(result.is_err());
        assert!(!path.exists());
    }

    #[test]
    fn writable_connection_permits_writes() -> Result<(), Box<dyn Error>> {
        let path = test_path("readwrite");
        remove(&path);
        let connection = Connection::open(&path)?;
        connection.execute_batch("CREATE TABLE items (id INTEGER)")?;
        drop(connection);

        let opened = open(path.to_string_lossy().as_ref(), DatabaseMode::ReadWrite)?;
        opened.connection.execute("INSERT INTO items VALUES (1)", [])?;
        let count: i64 = opened.connection.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))?;
        assert_eq!(count, 1);
        drop(opened);
        remove(&path);
        Ok(())
    }
}
