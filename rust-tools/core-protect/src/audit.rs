use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub fn log_event(event: &str) {
    let log_path = audit_log_path();

    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let timestamp = Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown-time".to_string());

    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

    let line = format!("[{}] {} | user={}\n", timestamp, event, user);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
        file.write_all(line.as_bytes()).ok();
    }
}

pub fn show_log() {
    let log_path = audit_log_path();

    if !log_path.exists() {
        println!("  No audit log found — no events recorded yet.");
        return;
    }

    match fs::read_to_string(&log_path) {
        Ok(contents) => {
            if contents.is_empty() {
                println!("  Audit log is empty.");
            } else {
                print!("{}", contents);
            }
        }
        Err(e) => println!("  Failed to read audit log: {}", e),
    }
}

pub fn audit_log_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("faelight")
        .join("core-protect-audit.log")
}
