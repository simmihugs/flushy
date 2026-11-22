pub mod app_log;
pub mod config;
pub mod driver_path;
pub mod events;
pub mod ses_portal;

use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let bundled_driver = driver_path::use_bundled_driver_next_to_exe();
    tauri::Builder::default()
        .setup(move |app| {
            if let Ok(data_dir) = app.path().app_data_dir() {
                app_log::init_logging(data_dir.clone());
                app_log::info(
                    "app",
                    &format!("Logging initialisiert in {}/logs", data_dir.display()),
                );

                match &bundled_driver {
                    Some(dir) => {
                        app_log::info("app", &format!("Playwright-Treiber: {}", dir.display()))
                    }
                    None => app_log::info(
                        "app",
                        "Kein mitgelieferter Playwright-Treiber, nutze Standardsuche",
                    ),
                }
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = handle.emit(
                    "sync-progress",
                    serde_json::json!({
                        "message": "Prüfe Playwright Chromium..."
                    }),
                );
                if let Err(e) = playwright_rs::install_browsers(Some(&["chromium"])).await {
                    app_log::error("setup", &format!("Browser-Setup fehlgeschlagen: {e:#}"));
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
