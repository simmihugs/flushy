//! The one function the UI calls: download, parse, store, stop when up to date.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;

use crate::scraper::{run_ses_scraper, RunEnd};
use crate::store::Store;
use crate::xlsx::read_xlsx;

pub struct SyncOptions {
    pub db: String,
    pub url: String,
    pub email: String,
    pub password: String,
    /// Where `ses.db` and the temporary `downloads/` folder live
    /// (in Tauri: `app.path().app_data_dir()`).
    pub data_dir: PathBuf,
    pub headless: bool,
    /// Keep the downloaded xlsx files after they were stored in the DB.
    pub keep_files: bool,
    pub logger: Option<crate::LogCallback>,
}

/// Sent to the UI after every page.
#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub page: usize,
    pub rows: usize,
    pub new_rows: usize,
    pub known_rows: usize,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Went through the whole list.
    ReachedEnd,
    /// Stopped because the DB already had the following entries.
    UpToDate,
    /// The user cancelled.
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncSummary {
    pub outcome: Outcome,
    pub pages: usize,
    pub new_rows: usize,
    pub total_in_db: usize,
}

/// `cancel` is checked after each page (a page takes a few seconds).
pub async fn sync_library<P>(
    opts: SyncOptions,
    cancel: Arc<AtomicBool>,
    mut on_progress: P,
) -> Result<SyncSummary>
where
    P: FnMut(Progress) + Send,
{
    std::fs::create_dir_all(&opts.data_dir)?;
    let mut store = Store::open(opts.data_dir.join(&opts.db))?;
    let apply_early_stop = store.initial_sync_complete()?;
    store.begin_run();

    let download_dir = opts.data_dir.join("downloads");
    let mut pages = 0usize;
    let mut new_total = 0usize;
    let mut cancelled = false;

    let end = run_ses_scraper(
        &opts.url,
        &opts.email,
        &opts.password,
        &opts.logger,
        &download_dir,
        opts.headless,
        |n, path| {
            let rows = read_xlsx(path)?;
            let result = store.ingest(&rows)?;
            if !opts.keep_files {
                let _ = std::fs::remove_file(path);
            }
            pages = n + 1;
            new_total += result.new_rows;
            on_progress(Progress {
                page: n + 1,
                rows: rows.len(),
                new_rows: result.new_rows,
                known_rows: result.known_rows,
            });
            if cancel.load(Ordering::Relaxed) {
                cancelled = true;
                return Ok(true);
            }
            Ok(apply_early_stop && result.known_rows > 0)
        },
    )
    .await?;

    if end == RunEnd::ReachedEnd {
        store.mark_initial_sync_complete()?;
    }
    let outcome = match (cancelled, end) {
        (true, _) => Outcome::Cancelled,
        (false, RunEnd::ReachedEnd) => Outcome::ReachedEnd,
        (false, RunEnd::StoppedEarly) => Outcome::UpToDate,
    };
    Ok(SyncSummary {
        outcome,
        pages,
        new_rows: new_total,
        total_in_db: store.count()?,
    })
}

#[allow(dead_code)]
fn sync_future_is_send() {
    fn assert_send<T: Send>(_: T) {}
    let opts = SyncOptions {
        url: String::new(),
        db: String::new(),
        email: String::new(),
        password: String::new(),
        logger: None,
        data_dir: PathBuf::new(),
        headless: true,
        keep_files: false,
    };
    assert_send(sync_library(
        opts,
        Arc::new(AtomicBool::new(false)),
        |_p| {},
    ));
}
