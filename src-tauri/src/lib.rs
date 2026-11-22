pub mod config;
pub mod events;

// #[tauri::command]
// fn parse_file(path: String) -> Result<events::ParsedEvents, String> {
//     events::parse_file(path)
// }

// #[tauri::command]
// fn create_or_load_config(path: String) -> Result<Vec<config::Param>, String> {
//     config::load_config(path)
// }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            events::parse_file,
            config::load_or_create_config,
            config::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
