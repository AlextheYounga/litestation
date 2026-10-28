mod commands;
mod database;
mod error;
mod export;
mod file_output;
mod import;
mod launch;
mod limits;
mod model;
mod mutation;
mod query;
mod state;
mod table;
mod value;

use std::env;
use std::path::Path;
use std::process;

use tauri::{Emitter, Manager};
use tauri_plugin_cli::CliExt;

pub use commands::{
    browse_table, close_database, delete_row, execute_sql, export_rows, export_sql_dump, get_table_metadata,
    import_sql_dump, import_sqlite_file, insert_row, open_local_database, refresh_schema, take_pending_database_paths,
    update_row,
};
use launch::{database_paths, LaunchPathState, PATHS_AVAILABLE_EVENT};
pub use state::DatabaseState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, arguments, working_directory| {
        match app.cli().matches_from(arguments) {
            Ok(matches) => match database_paths(&matches, Path::new(&working_directory))
                .and_then(|paths| app.state::<LaunchPathState>().enqueue(paths))
            {
                Ok(true) => {
                    let _ = app.emit(PATHS_AVAILABLE_EVENT, ());
                }
                Ok(false) => {}
                Err(error) => eprintln!("Could not queue database paths: {error}"),
            },
            Err(error) => eprintln!("Could not parse LiteStation arguments: {error}"),
        }

        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }));

    builder
        .manage(LaunchPathState::default())
        .manage(DatabaseState::default())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let matches = app.cli().matches()?;
            let working_directory = env::current_dir()?;
            let paths = database_paths(&matches, &working_directory)?;
            app.state::<LaunchPathState>().enqueue(paths)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_local_database,
            close_database,
            take_pending_database_paths,
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
