// faelight-shell — Execution Context
// INT-162 Phase 0 — ExecContext: From String-Driven to Context-Driven
//
// This is the foundation layer. Every command execution passes through
// ExecContext. All hooks, logging, and intelligence attach here.
//
// Architecture:
//   line → build_context() → preexec() → dispatch() → postexec() → result

use crate::db::ForestDb;
use crate::commands::{self, CommandResult};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ── ExecContext ───────────────────────────────────────────────────────────────
/// A typed description of every command execution.
/// Replaces raw string passing throughout the shell.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExecContext {
    /// Exactly what the user typed
    pub raw: String,
    /// After alias expansion (may differ from raw)
    pub expanded: String,
    /// Resolved command name (first token)
    pub cmd: String,
    /// Resolved arguments
    pub args: Vec<String>,
    /// Current working directory at time of execution
    pub cwd: PathBuf,
    /// Active intent (INT-NNN) if any — from shell focus state
    pub intent: Option<String>,
    /// Unix timestamp of execution
    pub timestamp: u64,
    /// Whether this command was executed via pipeline
    pub in_pipeline: bool,
}

impl ExecContext {
    /// Build an ExecContext from a raw input line
    pub fn from_line(line: &str, db: &ForestDb) -> Self {
        let raw = line.trim().to_string();
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Parse cmd and args from raw line
        let mut parts = raw.splitn(2, ' ');
        let cmd = parts.next().unwrap_or("").to_lowercase();
        let args: Vec<String> = parts.next()
            .unwrap_or("")
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Read active intent from db if available
        let intent = db.get_focus_intent();

        ExecContext {
            raw: raw.clone(),
            expanded: raw,   // will be updated after alias resolution
            cmd,
            args,
            cwd,
            intent,
            timestamp,
            in_pipeline: false,
        }
    }

    /// Mark this context as part of a pipeline
    #[allow(dead_code)]
    pub fn with_pipeline(mut self, in_pipeline: bool) -> Self {
        self.in_pipeline = in_pipeline;
        self
    }
}

// ── Execution Pipeline ────────────────────────────────────────────────────────

/// Preexec hook — runs before every command
/// Returns None to allow execution, Some(error) to block
fn preexec(ctx: &ExecContext, _db: &ForestDb) -> Option<String> {
    // Phase 0: minimal — just log that we have context
    // INT-171 will expand this into the full before_run system
    let _ = ctx; // context available for future hooks
    None
}

/// Postexec hook — runs after every command
fn postexec(ctx: &ExecContext, result: &CommandResult, db: &ForestDb) {
    // Phase 0: record to shell_history with context
    // Future: INT-177 observability, INT-176 failure memory
    let status = match result {
        CommandResult::Error(_) => "error",
        CommandResult::Exit => "exit",
        CommandResult::Empty => "empty",
        _ => "ok",
    };

    // Log to shell_history using existing db API
    // Future: INT-177 will expand this with full structured context
    if status != "exit" {
        db.save_history_entry(&ctx.raw);
    }
}

/// Main execution pipeline — the single entry point for all command execution
///
/// parse → preexec → dispatch → postexec → result
pub fn execute_with_context(line: &str, db: &ForestDb, core_root: &str) -> CommandResult {
    // Build execution context
    let ctx = ExecContext::from_line(line, db);

    // Preexec — can block execution
    if let Some(block_reason) = preexec(&ctx, db) {
        return CommandResult::Error(block_reason);
    }

    // Dispatch — call existing execute() (unchanged)
    let result = commands::execute(line, db, core_root);

    // Postexec — observe the result
    postexec(&ctx, &result, db);

    result
}
