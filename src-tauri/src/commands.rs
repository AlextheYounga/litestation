use tauri::State;

use crate::error::BackendError;
use crate::launch::LaunchPathState;
use crate::model::{
    BrowseTableRequest, BrowseTableResponse, DatabaseSchema, DeleteRowRequest, ExecuteSqlRequest, ExecuteSqlResponse,
    ExportRowsRequest, ExportRowsResponse, ExportSqlDumpRequest, ExportSqlDumpResponse, ImportSqlDumpRequest,
    ImportSqlDumpResponse, ImportSqliteFileRequest, ImportSqliteFileResponse, InsertRowRequest, MutationResult,
    OpenDatabaseResponse, OpenLocalDatabaseRequest, TableMetadata, TableRequest, UpdateRowRequest,
};
use crate::state::DatabaseState;
use crate::{database, export, import, mutation, query, table};

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn open_local_database(
    state: State<'_, DatabaseState>,
    request: OpenLocalDatabaseRequest,
) -> Result<OpenDatabaseResponse, BackendError> {
    let OpenLocalDatabaseRequest { path } = request;
    let opened = database::open(&path)?;
    state.insert(opened)
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn close_database(state: State<'_, DatabaseState>, connection_id: u64) -> Result<(), BackendError> {
    state.close(connection_id)
}

#[tauri::command]
#[expect(clippy::needless_pass_by_value)]
pub fn take_pending_database_paths(state: State<'_, LaunchPathState>) -> Result<Vec<String>, BackendError> {
    state.take_all()
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn execute_sql(
    state: State<'_, DatabaseState>,
    request: ExecuteSqlRequest,
) -> Result<ExecuteSqlResponse, BackendError> {
    state.with_database(request.connection_id, |connection| query::execute(connection, &request))
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn get_table_metadata(
    state: State<'_, DatabaseState>,
    request: TableRequest,
) -> Result<TableMetadata, BackendError> {
    state.with_database(request.connection_id, |connection| table::metadata(connection, &request.table_name))
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn browse_table(
    state: State<'_, DatabaseState>,
    request: BrowseTableRequest,
) -> Result<BrowseTableResponse, BackendError> {
    state.with_database(request.connection_id, |connection| {
        table::browse(connection, &request.table_name, request.offset, request.limit)
    })
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn refresh_schema(state: State<'_, DatabaseState>, connection_id: u64) -> Result<DatabaseSchema, BackendError> {
    state.with_database(connection_id, database::introspect)
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn insert_row(state: State<'_, DatabaseState>, request: InsertRowRequest) -> Result<MutationResult, BackendError> {
    state.with_database(request.connection_id, |connection| mutation::insert(connection, &request))
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn update_row(state: State<'_, DatabaseState>, request: UpdateRowRequest) -> Result<MutationResult, BackendError> {
    state.with_database(request.connection_id, |connection| mutation::update(connection, &request))
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn delete_row(state: State<'_, DatabaseState>, request: DeleteRowRequest) -> Result<MutationResult, BackendError> {
    state.with_database(request.connection_id, |connection| mutation::delete(connection, &request))
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn import_sql_dump(
    state: State<'_, DatabaseState>,
    request: ImportSqlDumpRequest,
) -> Result<ImportSqlDumpResponse, BackendError> {
    state.with_database(request.connection_id, |connection| import::sql_dump(connection, &request.source_path))
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn import_sqlite_file(request: ImportSqliteFileRequest) -> Result<ImportSqliteFileResponse, BackendError> {
    import::sqlite_file(&request.source_path, &request.destination_path)
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn export_rows(request: ExportRowsRequest) -> Result<ExportRowsResponse, BackendError> {
    export::rows(&request)
}

#[tauri::command(rename_all = "camelCase")]
#[expect(clippy::needless_pass_by_value)]
pub fn export_sql_dump(
    state: State<'_, DatabaseState>,
    request: ExportSqlDumpRequest,
) -> Result<ExportSqlDumpResponse, BackendError> {
    state.with_database(request.connection_id, |connection| export::sql_dump(connection, &request.destination_path))
}
