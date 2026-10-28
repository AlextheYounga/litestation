// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

fn main() {
    if handle_terminal_request() {
        return;
    }

    #[cfg(target_os = "linux")]
    // SAFETY: these variables are set before Tauri or any other application threads start.
    unsafe {
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    litestation_lib::run();
}

fn handle_terminal_request() -> bool {
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--" {
            return false;
        }
        if argument == "--help" || argument == "-h" {
            println!(
                "LiteStation {version}\n\nOpen one or more SQLite databases in LiteStation\n\nUsage: litestation [DATABASE]...\n\nArguments:\n  [DATABASE]...  SQLite database paths\n\nOptions:\n  -h, --help     Print help\n  -V, --version  Print version",
                version = env!("CARGO_PKG_VERSION")
            );
            return true;
        }
        if argument == "--version" || argument == "-V" {
            println!("LiteStation {}", env!("CARGO_PKG_VERSION"));
            return true;
        }
    }
    false
}
