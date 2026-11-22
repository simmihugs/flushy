pub mod config;
pub mod events;
pub mod ses_portal;

use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = handle.emit(
                    "sync-progress",
                    serde_json::json!({
                        "message": "Prüfe Playwright Chromium..."
                    }),
                );
                if let Err(e) = playwright_rs::install_browsers(Some(&["chromium"])).await {
                    let _ = handle.emit(
                        "sync-finished",
                        serde_json::json!({
                            "ok": false,
                            "message": format!("Browser-Setup fehlgeschlagen: {e:#}")
                        }),
                    );
                }
            });
            Ok(())
        })
        .manage(ses_portal::SyncState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            events::parse_file,
            config::load_or_create_config,
            config::save_config,
            ses_portal::start_sync,
            ses_portal::check_db_connection,
            ses_portal::list_db_entries,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
