use tauri::State;

use crate::database;
use crate::error::BackendError;
use crate::model::{OpenDatabaseResponse, OpenLocalDatabaseRequest};
use crate::state::DatabaseState;

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn open_local_database(
    state: State<'_, DatabaseState>,
    request: OpenLocalDatabaseRequest,
) -> Result<OpenDatabaseResponse, BackendError> {
    let OpenLocalDatabaseRequest { path, mode } = request;
    let opened = database::open(&path, mode)?;
    state.insert(opened)
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn close_database(state: State<'_, DatabaseState>, connection_id: u64) -> Result<(), BackendError> {
    state.close(connection_id)
}
