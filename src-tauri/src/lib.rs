mod commands;
mod database;
mod error;
mod export;
mod file_output;
mod import;
mod limits;
mod model;
mod mutation;
mod query;
mod state;
mod table;
mod value;

use std::process;

pub use commands::{
    browse_table, close_database, delete_row, execute_sql, export_rows, export_sql_dump, get_table_metadata,
    import_sql_dump, import_sqlite_file, insert_row, open_local_database, refresh_schema, update_row,
};
pub use state::DatabaseState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DatabaseState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_local_database,
            close_database,
            execute_sql,
            get_table_metadata,
            browse_table,
            refresh_schema,
            insert_row,
            update_row,
            delete_row,
            import_sql_dump,
            import_sqlite_file,
            export_rows,
            export_sql_dump
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("error while running Tauri application: {error}");
            process::exit(1);
        });
}
