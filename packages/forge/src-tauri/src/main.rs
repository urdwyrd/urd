// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod compiler;
mod shell;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            compiler::bridge::compile_project,
            shell::reveal_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Urd Forge");
}
