//! Points playwright-rs at a Playwright driver that ships *inside* the app.
//!
//! playwright-rs normally uses the driver that its build script assembled on
//! the build machine, at that machine's path. In a released app (built on a
//! CI runner, installed elsewhere) that path does not exist, so the driver has
//! to be bundled as a Tauri resource and `PLAYWRIGHT_DRIVER_PATH` set to it
//! before the first Playwright call.
//!
//! Call [`use_bundled_driver`] once, at the very start of the app (before
//! `install_browsers` / `Playwright::launch`).

use std::path::{Path, PathBuf};

/// Folder name used in the resources (see `tauri.conf.json`).
pub const DRIVER_DIR_NAME: &str = "playwright-driver";

fn node_exe() -> &'static str {
    if cfg!(windows) {
        "node.exe"
    } else {
        "node"
    }
}

/// A driver directory holds the node binary and `package/cli.js`.
fn is_driver_dir(dir: &Path) -> bool {
    dir.join(node_exe()).is_file() && dir.join("package").join("cli.js").is_file()
}

/// Windows paths from Tauri can come with the `\\?\` prefix; Node and the
/// playwright-rs check are happier without it.
fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC\\") => PathBuf::from(rest),
        _ => path.to_path_buf(),
    }
}

/// Looks for the bundled driver below `base` (Tauri's resource directory).
/// Depending on how `bundle.resources` is written, the files end up either
/// directly below `base` or below `base/resources`; both are checked.
pub fn find_driver_dir(base: &Path) -> Option<PathBuf> {
    [
        base.join(DRIVER_DIR_NAME),
        base.join("resources").join(DRIVER_DIR_NAME),
    ]
    .into_iter()
    .find(|dir| is_driver_dir(dir))
    .map(|dir| strip_verbatim_prefix(&dir))
}

/// Sets `PLAYWRIGHT_DRIVER_PATH` to the bundled driver found below `base`.
/// Returns the driver directory, or `None` if the app ships none (e.g. in
/// `tauri dev`), in which case playwright-rs keeps its normal lookup.
/// A `PLAYWRIGHT_DRIVER_PATH` the user has set themselves is respected.
pub fn use_bundled_driver(base: &Path) -> Option<PathBuf> {
    if std::env::var_os("PLAYWRIGHT_DRIVER_PATH").is_some_and(|v| !v.is_empty()) {
        return None;
    }
    let dir = find_driver_dir(base)?;
    // Only safe if no other thread reads the environment right now, so call
    // this first thing in `main` / `run`. (`unsafe` is required on edition 2024.)
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var("PLAYWRIGHT_DRIVER_PATH", &dir);
    }
    Some(dir)
}

/// Variant for use before Tauri's `Builder` exists: on Windows and Linux the
/// resources are installed next to the executable.
pub fn use_bundled_driver_next_to_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    use_bundled_driver(exe.parent()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_driver(dir: &Path) {
        std::fs::create_dir_all(dir.join("package")).unwrap();
        std::fs::write(dir.join(node_exe()), b"").unwrap();
        std::fs::write(dir.join("package").join("cli.js"), b"").unwrap();
    }

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("drv-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn finds_driver_directly_below_base() {
        let base = temp("direct");
        make_driver(&base.join(DRIVER_DIR_NAME));
        assert_eq!(find_driver_dir(&base), Some(base.join(DRIVER_DIR_NAME)));
        std::fs::remove_dir_all(base).ok();
    }

    #[test]
    fn finds_driver_below_resources_subfolder() {
        let base = temp("nested");
        make_driver(&base.join("resources").join(DRIVER_DIR_NAME));
        assert_eq!(
            find_driver_dir(&base),
            Some(base.join("resources").join(DRIVER_DIR_NAME))
        );
        std::fs::remove_dir_all(base).ok();
    }

    #[test]
    fn incomplete_or_missing_driver_is_not_found() {
        let base = temp("incomplete");
        std::fs::create_dir_all(base.join(DRIVER_DIR_NAME).join("package")).unwrap();
        std::fs::write(base.join(DRIVER_DIR_NAME).join(node_exe()), b"").unwrap(); // no cli.js
        assert_eq!(find_driver_dir(&base), None);
        assert_eq!(find_driver_dir(&base.join("nope")), None);
        std::fs::remove_dir_all(base).ok();
    }

    #[test]
    fn verbatim_prefix_is_removed_but_unc_is_kept() {
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"\\?\C:\Program Files\app")),
            PathBuf::from(r"C:\Program Files\app")
        );
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"\\?\UNC\server\share")),
            PathBuf::from(r"\\?\UNC\server\share")
        );
        assert_eq!(
            strip_verbatim_prefix(Path::new("/usr/lib")),
            PathBuf::from("/usr/lib")
        );
    }
}
