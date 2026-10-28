use std::mem;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde_json::Value;
use tauri_plugin_cli::Matches;

use crate::error::BackendError;

pub const PATHS_AVAILABLE_EVENT: &str = "database-paths-available";

#[derive(Default)]
pub struct LaunchPathState {
    pending: Mutex<Vec<String>>,
}

impl LaunchPathState {
    pub fn enqueue(&self, paths: Vec<String>) -> Result<bool, BackendError> {
        if paths.is_empty() {
            return Ok(false);
        }

        self.pending.lock().map_err(lock_error)?.extend(paths);
        Ok(true)
    }

    pub fn take_all(&self) -> Result<Vec<String>, BackendError> {
        let mut pending = self.pending.lock().map_err(lock_error)?;
        Ok(mem::take(&mut *pending))
    }
}

pub fn database_paths(matches: &Matches, working_directory: &Path) -> Result<Vec<String>, BackendError> {
    let Some(argument) = matches.args.get("database") else {
        return Ok(Vec::new());
    };

    paths_from_value(&argument.value, working_directory)
}

fn paths_from_value(value: &Value, working_directory: &Path) -> Result<Vec<String>, BackendError> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::String(path) => Ok(vec![resolve_path(path, working_directory)]),
        Value::Array(paths) => paths
            .iter()
            .map(|path| path.as_str().map(|value| resolve_path(value, working_directory)).ok_or_else(invalid_argument))
            .collect(),
        _ => Err(invalid_argument()),
    }
}

fn resolve_path(path: &str, working_directory: &Path) -> String {
    let path = PathBuf::from(path);
    let resolved = if path.is_absolute() { path } else { working_directory.join(path) };
    resolved.to_string_lossy().into_owned()
}

fn invalid_argument() -> BackendError {
    BackendError::new("invalid_cli_argument", "Database paths must be text values")
}

fn lock_error<T>(_: PoisonError<T>) -> BackendError {
    BackendError::new("launch_state_unavailable", "Pending database paths are unavailable")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::{paths_from_value, LaunchPathState};

    #[test]
    fn resolves_a_single_database_path() {
        let result = paths_from_value(&json!("database.sqlite"), Path::new("/home/user/project"));

        assert_eq!(result.ok(), Some(vec!["/home/user/project/database.sqlite".to_string()]));
    }

    #[test]
    fn resolves_relative_database_paths_against_the_calling_directory() {
        let result =
            paths_from_value(&json!(["data/one.sqlite", "/var/data/two.sqlite"]), Path::new("/home/user/project"));

        assert_eq!(
            result.ok(),
            Some(vec!["/home/user/project/data/one.sqlite".to_string(), "/var/data/two.sqlite".to_string()])
        );
    }

    #[test]
    fn launch_path_state_drains_each_path_once() {
        let state = LaunchPathState::default();
        assert_eq!(state.enqueue(vec!["one.sqlite".to_string(), "two.sqlite".to_string()]).ok(), Some(true));
        assert_eq!(state.take_all().ok(), Some(vec!["one.sqlite".to_string(), "two.sqlite".to_string()]));
        assert_eq!(state.take_all().ok(), Some(Vec::new()));
    }
}
