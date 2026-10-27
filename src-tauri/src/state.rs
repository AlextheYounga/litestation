use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use rusqlite::Connection;

use crate::database::OpenedDatabase;
use crate::error::BackendError;
use crate::model::OpenDatabaseResponse;

struct StoredDatabase {
    connection: Connection,
}

#[derive(Default)]
pub struct DatabaseState {
    connections: Mutex<BTreeMap<u64, StoredDatabase>>,
    next_id: Mutex<u64>,
}

impl DatabaseState {
    pub fn insert(&self, opened: OpenedDatabase) -> Result<OpenDatabaseResponse, BackendError> {
        let mut next_id = self.next_id.lock().map_err(lock_error)?;
        let id = *next_id;
        *next_id = next_id
            .checked_add(1)
            .ok_or_else(|| BackendError::new("connection_id_exhausted", "No connection IDs remain"))?;
        drop(next_id);

        let response = OpenDatabaseResponse {
            id,
            path: opened.path.to_string_lossy().into_owned(),
            display_name: opened
                .path
                .file_name()
                .filter(|name| !name.is_empty())
                .map_or_else(|| opened.path.to_string_lossy().into_owned(), |name| name.to_string_lossy().into_owned()),
            schema: opened.schema,
        };
        let stored = StoredDatabase { connection: opened.connection };
        self.connections.lock().map_err(lock_error)?.insert(id, stored);
        Ok(response)
    }

    pub fn close(&self, id: u64) -> Result<(), BackendError> {
        let removed = self.connections.lock().map_err(lock_error)?.remove(&id);
        if let Some(database) = removed {
            drop(database.connection);
            Ok(())
        } else {
            Err(BackendError::new("connection_not_found", "Database connection was not found"))
        }
    }

    pub fn with_database<T>(
        &self,
        id: u64,
        operation: impl FnOnce(&Connection) -> Result<T, BackendError>,
    ) -> Result<T, BackendError> {
        let connections = self.connections.lock().map_err(lock_error)?;
        let database = connections
            .get(&id)
            .ok_or_else(|| BackendError::new("connection_not_found", "Database connection was not found"))?;
        operation(&database.connection)
    }

    #[cfg(test)]
    fn execute(&self, id: u64, sql: &str) -> Result<(), BackendError> {
        let connections = self.connections.lock().map_err(lock_error)?;
        let database = connections
            .get(&id)
            .ok_or_else(|| BackendError::new("connection_not_found", "Database connection was not found"))?;
        database.connection.execute_batch(sql).map_err(|error| BackendError::database("database_query_failed", &error))
    }
}

fn lock_error<T>(_: PoisonError<T>) -> BackendError {
    BackendError::new("state_unavailable", "Database connection state is unavailable")
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::{env, process};

    use rusqlite::Connection;

    use super::DatabaseState;
    use crate::database::open;

    static NEXT_TEST_FILE: AtomicUsize = AtomicUsize::new(1000);

    #[test]
    fn close_removes_connection_from_state() -> Result<(), Box<dyn Error>> {
        let path = env::temp_dir().join(format!(
            "litestation-state-{}-{}.db",
            process::id(),
            NEXT_TEST_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        let connection = Connection::open(&path)?;
        connection.execute_batch("CREATE TABLE items (id INTEGER)")?;
        drop(connection);
        let state = DatabaseState::default();
        let response = state.insert(open(path.to_string_lossy().as_ref())?)?;
        assert!(!response.display_name.is_empty());
        state.close(response.id)?;
        assert_eq!(state.close(response.id).err().map(|error| error.code), Some("connection_not_found".to_string()));
        assert_eq!(
            state.execute(response.id, "INSERT INTO items VALUES (1)").err().map(|error| error.code),
            Some("connection_not_found".to_string())
        );
        let _ = fs::remove_file(path);
        Ok(())
    }
}
