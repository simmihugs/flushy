//! Drives ses-360.com with Playwright and downloads every result page as xlsx.

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use playwright_rs::{
    AriaRole, BrowserContextOptions, GetByRoleOptions, LaunchOptions, Page, Playwright,
    WaitForOptions, WaitForState,
};

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnd {
    /// Every page was downloaded (no next button on the last page).
    ReachedEnd,
    /// The callback asked to stop.
    StoppedEarly,
}

fn role_name(name: &str) -> Option<GetByRoleOptions> {
    let mut o = GetByRoleOptions::default();
    o.name = Some(name.into());
    Some(o)
}

/// Downloads every result page as an xlsx into `download_dir`.
/// `on_page(n, path)` is called after each page; return `Ok(true)` to stop.
///
/// The browser is always closed, also when a step fails halfway.
pub async fn run_ses_scraper<F>(
    url: &str,
    email: &str,
    password: &str,
    logger: &Option<crate::LogCallback>,
    download_dir: &Path,
    headless: bool,
    on_page: F,
) -> Result<RunEnd>
where
    F: FnMut(usize, &Path) -> Result<bool>,
{
    crate::emit_log(logger, "INFO", "scraper", &format!("Öffne {url}"));

    std::fs::create_dir_all(download_dir)?;

    let pw = Playwright::launch().await?;
    let mut launch_opts = LaunchOptions::default();
    launch_opts.headless = Some(headless);
    let browser = pw.chromium().launch_with_options(launch_opts).await?;

    let mut ctx_opts = BrowserContextOptions::default();
    ctx_opts.accept_downloads = Some(true.into());
    let context = browser.new_context_with_options(ctx_opts).await?;
    context.set_default_timeout(60_000.0).await;
    let page = context.new_page().await?;

    let result = scrape_pages(&page, logger, url, email, password, download_dir, on_page).await;

    let _ = context.close().await;
    let _ = browser.close().await;
    result
}

async fn scrape_pages<F>(
    page: &Page,
    logger: &Option<crate::LogCallback>,
    url: &str,
    email: &str,
    password: &str,
    download_dir: &Path,
    mut on_page: F,
) -> Result<RunEnd>
where
    F: FnMut(usize, &Path) -> Result<bool>,
{
    page.goto(url, None).await?;

    page.get_by_role(AriaRole::Textbox, role_name("email"))
        .fill(email, None)
        .await?;
    page.get_by_role(AriaRole::Button, role_name("Next"))
        .click(None)
        .await?;
    page.get_by_role(AriaRole::Textbox, role_name("password"))
        .fill(password, None)
        .await?;
    page.get_by_role(AriaRole::Button, role_name("Next"))
        .click(None)
        .await?;
    crate::emit_log(logger, "INFO", "scraper", "Login into SES successful.");

    let frame = page.frame_locator("#extension-module");
    frame
        .get_by_title("Advanced Search Settings", false)
        .click(None)
        .await?;
    frame
        .get_by_role(AriaRole::Slider, None)
        .fill("250", None)
        .await?;
    frame.locator("#SearchButton").click(None).await?;

    let hidden_120s = || {
        WaitForOptions::builder()
            .state(WaitForState::Hidden)
            .timeout(120_000.0)
            .build()
    };

    let mut n: usize = 0;
    loop {
        frame
            .locator("#ExportButton")
            .get_by_text("Export", false)
            .click(None)
            .await?;
        frame
            .locator("#customPrompt")
            .fill(&format!("360-library-results{n}"), None)
            .await?;

        let waiter = page.expect_download(Some(120_000.0)).await?;
        frame
            .get_by_role(AriaRole::Button, role_name("Ok"))
            .click(None)
            .await?;
        let download = waiter.wait().await?;

        let target = download_dir.join(format!("360-library-results{n}.xlsx"));
        download
            .save_as(target.to_str().unwrap_or_default())
            .await?;
        crate::emit_log(
            logger,
            "INFO",
            "scraper",
            format!("Successfully downloaded page {}", n + 1).as_str(),
        );

        if on_page(n, &target)? {
            crate::emit_log(
                logger,
                "INFO",
                "scraper",
                format!("Stop requested after page {}", n + 1).as_str(),
            );
            return Ok(RunEnd::StoppedEarly);
        }

        let next_btn = frame.locator("nav.paging a:has(i.icon-control-next)");
        if next_btn.count().await? == 0 {
            crate::emit_log(logger, "INFO", "scraper", "Reached end of data storage");
            return Ok(RunEnd::ReachedEnd);
        }
        next_btn.click(None).await?;
        n += 1;

        frame
            .locator(".waiting.library-loading")
            .wait_for(Some(hidden_120s()))
            .await?;

        let expected = (n + 1).to_string();
        let start = Instant::now();
        loop {
            let text = frame
                .locator("nav.paging a.current-page")
                .inner_text()
                .await
                .unwrap_or_default();
            if text.trim() == expected {
                break;
            }
            if start.elapsed() > Duration::from_secs(120) {
                bail!("Expected page {expected}, still showing {:?}", text.trim());
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
}
