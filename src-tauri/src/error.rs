use std::error::Error;
use std::fmt;

use serde::Serialize;

pub const INVALID_IDENTITY: &str = "invalid_identity";
pub const INVALID_PAGINATION: &str = "invalid_pagination";
pub const INVALID_SQL: &str = "invalid_sql";
pub const QUERY_FAILED: &str = "query_failed";
pub const INVALID_TEXT_ENCODING: &str = "invalid_text_encoding";
pub const QUERY_TIMEOUT: &str = "query_timeout";
pub const RESULT_CELL_TOO_LARGE: &str = "result_cell_too_large";
pub const RESULT_COLUMN_LIMIT_EXCEEDED: &str = "result_column_limit_exceeded";
pub const RESULT_PAYLOAD_TOO_LARGE: &str = "result_payload_too_large";
pub const SCHEMA_LIMIT_EXCEEDED: &str = "schema_limit_exceeded";
pub const SQL_TOO_LARGE: &str = "sql_too_large";
pub const INVALID_PATH: &str = "invalid_path";
pub const DESTINATION_EXISTS: &str = "destination_exists";
pub const DATABASE_OPEN_FAILED: &str = "database_open_failed";
pub const SQL_BLOCK_COMMENT_START: &[u8] = b"/*";
pub const SQL_BLOCK_COMMENT_END: &[u8] = b"*/";
pub const SQL_BEGIN: &str = "BEGIN";
pub const SQL_COMMIT: &str = "COMMIT";
pub const SQL_ROLLBACK: &str = "ROLLBACK";
pub const SQL_SAVEPOINT: &str = "SAVEPOINT";
pub const SQL_RELEASE: &str = "RELEASE";
pub const SQL_END: &str = "END";
pub const SQL_ATTACH: &str = "ATTACH";
pub const SQL_DETACH: &str = "DETACH";
pub const SQL_VACUUM: &str = "VACUUM";
pub const SQL_PRAGMA: &str = "PRAGMA";

#[derive(Debug, Serialize)]
pub struct BackendError {
    pub code: String,
    pub message: String,
}

impl fmt::Display for BackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for BackendError {}

impl BackendError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code: code.into(), message: message.into() }
    }

    pub fn database(code: &'static str, error: &rusqlite::Error) -> Self {
        Self::new(code, format!("SQLite operation failed: {error}"))
    }
}
