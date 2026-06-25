// src/utils/logger.rs — Timestamped operation log to file

use std::fs::OpenOptions;
use std::io::Write;

/// Log an action to defender_remover.log in the executable's directory
pub fn log(message: &str) {
    let timestamp = get_timestamp();
    let log_line = format!("[{}] {}\n", timestamp, message);

    // Try to write to log file (non-fatal if it fails)
    let log_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("defender_remover.log")));

    if let Some(path) = log_path {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
}

/// Log a success event
pub fn log_ok(action: &str) {
    log(&format!("[OK] {}", action));
}

/// Log a warning event
pub fn log_warn(action: &str) {
    log(&format!("[WARN] {}", action));
}

/// Log an error event
pub fn log_err(action: &str, err: &str) {
    log(&format!("[ERR] {} — {}", action, err));
}

/// Simple timestamp without external crates (uses Windows SYSTEMTIME)
#[cfg(target_os = "windows")]
fn get_timestamp() -> String {
    use windows::Win32::System::SystemInformation::GetLocalTime;
    unsafe {
        let st = GetLocalTime();
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
        )
    }
}

#[cfg(not(target_os = "windows"))]
fn get_timestamp() -> String {
    "0000-00-00 00:00:00".to_string()
}
