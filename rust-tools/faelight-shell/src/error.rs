#![allow(dead_code)]
// faelight-shell — Structured Error System
// INT-174 — The Shell Explains Its Failures
//
// Every error becomes a structured value.
// "An error that disappears is an error that repeats."

use std::time::{SystemTime, UNIX_EPOCH};

/// A structured shell error — code, message, suggestion, context
#[derive(Debug, Clone)]
pub struct ShellError {
    pub code:       &'static str,
    pub message:    String,
    pub suggestion: String,
    pub command:    String,
    pub directory:  String,
    pub timestamp:  u64,
}

impl ShellError {
    pub fn new(code: &'static str, message: &str, suggestion: &str, command: &str, directory: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            code,
            message:    message.to_string(),
            suggestion: suggestion.to_string(),
            command:    command.to_string(),
            directory:  directory.to_string(),
            timestamp,
        }
    }

    /// Format for display in the shell
    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("\n  ❌ {}: {}\n", self.code, self.message));
        if !self.suggestion.is_empty() {
            out.push_str(&format!("     💡 {}\n", self.suggestion));
        }
        out
    }

    /// Format as a single-line string for storage in db
    pub fn to_storage(&self) -> String {
        format!("{}|{}|{}|{}|{}|{}",
            self.code, self.message, self.suggestion,
            self.command, self.directory, self.timestamp)
    }

    /// Parse from storage string
    pub fn from_storage(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(6, '|').collect();
        if parts.len() < 6 { return None; }
        Some(Self {
            code:       Box::leak(parts[0].to_string().into_boxed_str()),
            message:    parts[1].to_string(),
            suggestion: parts[2].to_string(),
            command:    parts[3].to_string(),
            directory:  parts[4].to_string(),
            timestamp:  parts[5].parse().unwrap_or(0),
        })
    }
}

/// Known error codes — structured, documented, consistent
#[allow(dead_code)]
pub mod codes {
    pub const CMD_NOT_FOUND:  &str = "E_CMD_NOT_FOUND";
    pub const PERMISSION:     &str = "E_PERMISSION";
    pub const NOT_GIT_REPO:   &str = "E_NOT_GIT_REPO";
    pub const PIPE_EMPTY:     &str = "E_PIPE_EMPTY";
    pub const PARSE_FAILED:   &str = "E_PARSE_FAILED";
    pub const CORE_LOCKED:    &str = "E_CORE_LOCKED";
    pub const EXIT_NONZERO:   &str = "E_EXIT_NONZERO";
    pub const NOT_FOUND:      &str = "E_NOT_FOUND";
    pub const IO_ERROR:       &str = "E_IO_ERROR";
}

/// Build a structured error string for use in CommandResult::Error
/// Stores the error in db and returns formatted display string
pub fn make_error(
    code: &'static str,
    message: &str,
    suggestion: &str,
    command: &str,
    directory: &str,
    db: &crate::db::ForestDb,
) -> String {
    let err = ShellError::new(code, message, suggestion, command, directory);
    // Store in shell_state as last_error
    let _ = db.conn.execute(
        "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_error', ?1)",
        rusqlite::params![err.to_storage()],
    );
    // Append to session error log
    let ts = err.timestamp;
    let log_key = format!("error_log_{}", ts);
    let _ = db.conn.execute(
        "INSERT OR REPLACE INTO shell_state (key, value) VALUES (?1, ?2)",
        rusqlite::params![log_key, err.to_storage()],
    );
    err.display()
}
