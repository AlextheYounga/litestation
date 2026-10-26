mod commands;
mod database;
mod error;
mod model;
mod state;

use std::process;

pub use commands::{close_database, open_local_database};
pub use state::DatabaseState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DatabaseState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![open_local_database, close_database])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("error while running Tauri application: {error}");
            process::exit(1);
        });
}
