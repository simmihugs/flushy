use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();
static LOG_LOCK: Mutex<()> = Mutex::new(());

pub fn init_logging(app_data_dir: PathBuf) {
    let logs_dir = app_data_dir.join("logs");
    let _ = fs::create_dir_all(&logs_dir);
    let _ = LOG_DIR.set(logs_dir);
}

pub fn write_log(level: &str, scope: &str, message: &str) {
    let now = Local::now();
    let ts = now.format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("[{ts}] [{level}] [{scope}] {message}");

    println!("{line}");

    let Some(logs_dir) = LOG_DIR.get() else {
        return;
    };

    let filename = format!("flushy-{}.log", now.format("%Y-%m-%d"));
    let path = logs_dir.join(filename);

    if let Ok(_guard) = LOG_LOCK.lock() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{line}");
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($scope:expr, $($arg:tt)*) => {
        $crate::app_log::write_log("INFO", $scope, &format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($scope:expr, $($arg:tt)*) => {
        $crate::app_log::write_log("WARN", $scope, &format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_error {
    ($scope:expr, $($arg:tt)*) => {
        $crate::app_log::write_log("ERROR", $scope, &format!($($arg)*));
    };
}

pub fn info(scope: &str, message: &str) {
    write_log("INFO", scope, message);
}

pub fn warn(scope: &str, message: &str) {
    write_log("WARN", scope, message);
}

pub fn error(scope: &str, message: &str) {
    write_log("ERROR", scope, message);
}
