//! Library part of the SES downloader. Used by the CLI in `main.rs` and
//! copied/linked into the Tauri app (`src-tauri`).

pub mod scraper;
pub mod store;
pub mod sync;
pub mod xlsx;
