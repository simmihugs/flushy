//! Library part of the SES downloader. Used by the CLI in `main.rs` and
//! copied/linked into the Tauri app (`src-tauri`).

pub mod scraper;
pub mod store;
pub mod sync;
pub mod xlsx;

use std::sync::Arc;

pub type LogCallback = Arc<dyn Fn(&str, &str, &str) + Send + Sync>;

pub fn emit_log(cb: &Option<LogCallback>, level: &str, scope: &str, msg: &str) {
    if let Some(f) = cb {
        f(level, scope, msg);
    } else {
        println!("[{level}] [{scope}] {msg}");
    }
}
