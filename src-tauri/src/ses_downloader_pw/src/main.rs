//! Command line wrapper, handy for debugging outside Tauri:
//!   HEADFUL=1 cargo run
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::{bail, Result};
use ses_downloader_pw::sync::{sync_library, SyncOptions};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let email = std::env::var("EMAIL").unwrap_or_default();
    let password = std::env::var("PASSWD").unwrap_or_default();
    let db = std::env::var("DATABASE_URL").unwrap_or_default();
    let headless = std::env::var("HEADLESS")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(true);
    if email.is_empty() || password.is_empty() {
        bail!("Email or Password missing (set EMAIL and PASSWD, or use a .env file)");
    }

    playwright_rs::install_browsers(Some(&["chromium"])).await?;

    let summary = sync_library(
        SyncOptions {
            db,
            email,
            password,
            data_dir: PathBuf::from("data"),
            headless,
            keep_files: true,
        },
        Arc::new(AtomicBool::new(false)),
        |p| {
            eprintln!(
                "page {}: {} rows, {} new, {} already in DB",
                p.page, p.rows, p.new_rows, p.known_rows
            )
        },
    )
    .await?;
    eprintln!("{summary:?}");
    Ok(())
}
