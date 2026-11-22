use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};

use ses_downloader_pw::store::{Store, StoredEntry};
use ses_downloader_pw::sync::{sync_library, Progress, SyncOptions, SyncSummary};

#[derive(Default)]
pub struct SyncState {
    running: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
}

#[derive(Debug, Serialize)]
pub struct DbListResponse {
    pub total: usize,
    pub items: Vec<StoredEntry>,
}

/// Resets the "running" flag on every exit path, including early returns.
struct RunGuard(Arc<AtomicBool>);
impl Drop for RunGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// Downloads Chromium for Playwright. Call once before the first sync
/// (e.g. on first start, behind a "Setting up..." screen).
#[tauri::command]
pub async fn setup_browser() -> Result<(), String> {
    playwright_rs::install_browsers(Some(&["chromium"]))
        .await
        .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub async fn sync_library_cmd(
    app: AppHandle,
    state: State<'_, SyncState>,
    db: String,
    url: String,
    email: String,
    password: String,
) -> Result<SyncSummary, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    if state.running.swap(true, Ordering::SeqCst) {
        return Err("A sync is already running".into());
    }
    let _guard = RunGuard(state.running.clone());
    state.cancel.store(false, Ordering::SeqCst);

    sync_library(
        SyncOptions {
            db,
            url,
            email,
            password,
            data_dir,
            headless: true,
            keep_files: false,
        },
        state.cancel.clone(),
        |progress: Progress| {
            let _ = app.emit("sync-progress", progress);
        },
    )
    .await
    .map_err(|e| format!("{e:#}"))
}

/// Takes effect after the page that is currently being downloaded.
#[tauri::command]
pub fn cancel_sync(state: State<'_, SyncState>) {
    state.cancel.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub fn check_db_connection(db: String, app: tauri::AppHandle) -> Result<usize, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    let store =
        ses_downloader_pw::store::Store::open(data_dir.join(db)).map_err(|e| format!("{e:#}"))?;
    store.count().map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub fn start_sync(
    app: tauri::AppHandle,
    state: tauri::State<'_, SyncState>,
    email: String,
    password: String,
    db: String,
    url: String,
    headless: bool,
) -> Result<(), String> {
    if state.running.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    state.cancel.store(false, Ordering::SeqCst);

    let running = state.running.clone();
    let cancel = state.cancel.clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let _guard = RunGuard(running);

        if email.is_empty() || password.is_empty() {
            let _ = app_handle.emit(
                "sync-finished",
                serde_json::json!({
                    "ok": false,
                    "message": "EMAIL oder PASSWD fehlt in .env"
                }),
            );
            return;
        }

        let data_dir = match app_handle.path().app_data_dir() {
            Ok(d) => d,
            Err(e) => {
                let _ = app_handle.emit(
                    "sync-finished",
                    serde_json::json!({
                        "ok": false,
                        "message": e.to_string()
                    }),
                );
                return;
            }
        };

        match ses_downloader_pw::sync::sync_library(
            ses_downloader_pw::sync::SyncOptions {
                db,
                url,
                email,
                password,
                data_dir,
                headless,
                keep_files: false,
            },
            cancel,
            |progress| {
                let _ = app_handle.emit("sync-progress", progress);
            },
        )
        .await
        {
            Ok(summary) => {
                let _ = app_handle.emit(
                    "sync-finished",
                    serde_json::json!({
                        "ok": true,
                        "message": format!("Synced ({} Einträge)", summary.total_in_db),
                        "total_in_db": summary.total_in_db
                    }),
                );
            }
            Err(e) => {
                let _ = app_handle.emit(
                    "sync-finished",
                    serde_json::json!({
                        "ok": false,
                        "message": format!("{e:#}")
                    }),
                );
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn list_db_entries(
    app: AppHandle,
    db: String,
    search: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<DbListResponse, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_name = if db.trim().is_empty() {
        "ses2.db"
    } else {
        db.trim()
    };
    let store = Store::open(data_dir.join(db_name)).map_err(|e| format!("{e:#}"))?;
    let total = store.count().map_err(|e| format!("{e:#}"))?;
    let items = store
        .list_entries(search.as_deref(), limit.unwrap_or(300), offset.unwrap_or(0))
        .map_err(|e| format!("{e:#}"))?;

    Ok(DbListResponse { total, items })
}
