use std::error::Error;
use std::fmt;

use serde::Serialize;

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
