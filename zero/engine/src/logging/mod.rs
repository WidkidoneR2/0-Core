#![allow(dead_code)]
use chrono::Local;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct LogEntry {
    timestamp: String,
    domain: String,
    level: String,
    message: String,
}

pub fn log_event(runtime_dir: &Path, domain: &str, level: &str, message: &str) {
    let log_dir = runtime_dir.join("logs");
    let _ = fs::create_dir_all(&log_dir);
    let entry = LogEntry {
        timestamp: Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        domain: domain.to_string(),
        level: level.to_string(),
        message: message.to_string(),
    };
    let line = serde_json::to_string(&entry).unwrap_or_default();
    let log_file = log_dir.join(format!("{}.jsonl", domain));
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .map(|mut f| {
            use std::io::Write;
            let _ = writeln!(f, "{}", line);
        });
}
