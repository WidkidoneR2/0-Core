//! Error logging for faelight-bar v4.0.0

use std::fs::OpenOptions;
use std::io::Write;

const LOG_FILE: &str = "/tmp/faelight-bar.log";

pub fn init() {
    if let Err(e) = clear_log() {
        eprintln!("⚠️  Failed to clear log: {}", e);
    }
    log_info("Faelight-Bar v4.0.0 starting...");
}

fn clear_log() -> std::io::Result<()> {
    std::fs::write(LOG_FILE, "")?;
    Ok(())
}

pub fn log_info(msg: &str) {
    write_log("INFO", msg);
}

pub fn log_warn(msg: &str) {
    write_log("WARN", msg);
    eprintln!("⚠️  {}", msg);
}

pub fn log_error(msg: &str) {
    write_log("ERROR", msg);
    eprintln!("❌ {}", msg);
}

fn write_log(level: &str, msg: &str) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("[{}] {}: {}\n", timestamp, level, msg);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LOG_FILE) {
        let _ = file.write_all(line.as_bytes());
    }
}
