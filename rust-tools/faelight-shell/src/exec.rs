// faelight-shell — Execution Context
// INT-162 Phase 0 — ExecContext: From String-Driven to Context-Driven
//
// This is the foundation layer. Every command execution passes through
// ExecContext. All hooks, logging, and intelligence attach here.
//
// Architecture:
//   line → build_context() → preexec() → dispatch() → postexec() → result

use crate::config::{BeforeRunRule, RuleAction};
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
/// Returns None to allow execution, Some(message) to block
fn preexec(ctx: &ExecContext, _db: &ForestDb, core_root: &str, rules: &[BeforeRunRule]) -> Option<String> {
    let cmd = ctx.cmd.as_str();
    let raw = ctx.raw.as_str();

    // ── Safety Rule 1: Catastrophic rm -rf protection ─────────────────────────
    // Block any rm -rf targeting root, home, or core source directories
    if cmd == "rm" {
        let raw_lower = raw.to_lowercase();
        if raw_lower.contains("-rf") || raw_lower.contains("-fr") {
            // Absolute block — these targets are never safe
            let blocked_targets = ["/", "/home", "/etc", "/usr", "/var", "/boot"];
            for target in &blocked_targets {
                if raw.contains(target) {
                    return Some(format!(
                        "🛡  Blocked: rm -rf on protected path '{}' — this cannot be undone",
                        target
                    ));
                }
            }
            // Block rm -rf on core source directories
            let core_src = format!("{}/rust-tools", core_root);
            let core_engine = format!("{}/engine", core_root);
            let core_intents = format!("{}/intents", core_root);
            for protected in &[core_src.as_str(), core_engine.as_str(), core_intents.as_str()] {
                if raw.contains(protected) {
                    return Some(format!(
                        "🛡  Blocked: rm -rf on forest source '{}' — use git to manage removals",
                        protected
                    ));
                }
            }
        }
    }

    // ── Safety Rule 2: Core lock enforcement ──────────────────────────────────
    // Block git and fg operations when core is locked
    let in_core = ctx.cwd.starts_with(core_root);
    if in_core && is_core_locked(core_root) {
        let blocked_git = cmd == "git" && matches!(
            ctx.args.first().map(|s| s.as_str()).unwrap_or(""),
            "commit" | "push" | "add" | "rm" | "reset" | "rebase" | "merge"
        );
        let blocked_fg = cmd == "fg" && matches!(
            ctx.args.first().map(|s| s.as_str()).unwrap_or(""),
            "commit" | "push" | "sync"
        );
        if blocked_git || blocked_fg {
            return Some(
                "🔒 Core is LOCKED — run unlock-core first, then make your changes".to_string()
            );
        }
    }

    // ── Safety Rule 3: Protect against self-overwriting core binary ───────────
    if cmd == "cp" || cmd == "mv" {
        let core_bin = format!("{}/scripts/core", core_root);
        if raw.contains(&core_bin) && !raw.contains("deploy") {
            return Some(
                "🛡  Blocked: direct copy to core binary — use deploy script instead".to_string()
            );
        }
    }

    // ── Config Rules: evaluate before_run rules from config.fsh ────────────
    for rule in rules {
        if rule.matches(&ctx.raw) {
            match &rule.action {
                RuleAction::Block => {
                    return Some(format!("🛡  Blocked: {}", rule.message));
                }
                RuleAction::Warn => {
                    println!("  ⚠️  {}", rule.message);
                }
                RuleAction::Suggest => {
                    println!("  💡 {}", rule.message);
                }
            }
        }
    }

    None
}

/// Check if core_root has the immutable flag set
fn is_core_locked(core_root: &str) -> bool {
    std::process::Command::new("lsattr")
        .args(["-d", core_root])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("----i"))
        .unwrap_or(false)
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

    // Log to shell_history
    if status != "exit" {
        db.save_history_entry(&ctx.raw);
    }

    // ── Failure Memory — INT-176 ──────────────────────────────────────────────
    // Store last failed command so last_command retry/explain/fix can use it
    if status == "error" {
        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_failed_command', ?1)",
            rusqlite::params![ctx.raw],
        );
        // Append to session failure log
        let ts = ctx.timestamp as i64;
        let error_msg = match result {
            crate::commands::CommandResult::Error(e) => e.clone(),
            _ => "unknown error".to_string(),
        };
        let log_key = format!("failure_log_{}", ts);
        let log_val = format!("{}|{}", ctx.raw, error_msg);
        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES (?1, ?2)",
            rusqlite::params![log_key, log_val],
        );
    }

    // ── Suggest system — INT-171 Phase 4 ─────────────────────────────────────
    if status == "ok" || status == "empty" {
        let suggestion = match ctx.cmd.as_str() {
            "fg" if ctx.args.first().map(|s| s.as_str()) == Some("commit") => {
                Some("💡 Suggestion: run d — verify health after committing")
            }
            "deploy" => Some("💡 Suggestion: run d — verify health after deploy"),
            "unlock-core" => Some("💡 Reminder: run lock-core before shutdown"),
            "cicomplete" => Some("💡 Next: fg commit — record the completion"),
            "cistart" => Some("💡 Next: read the intent carefully before writing any code"),
            "paru" | "pacman" => Some("💡 Suggestion: run d — verify system health after update"),
            _ => None,
        };
        if let Some(msg) = suggestion {
            println!("  {}", msg);
        }
    }
}

/// Main execution pipeline — the single entry point for all command execution
///
/// parse → preexec → dispatch → postexec → result
pub fn execute_with_context(line: &str, db: &ForestDb, core_root: &str, rules: &[BeforeRunRule]) -> CommandResult {
    // Build execution context
    let ctx = ExecContext::from_line(line, db);

    // Preexec — can block execution
    if let Some(block_reason) = preexec(&ctx, db, core_root, rules) {
        return CommandResult::Error(block_reason);
    }

    // Dispatch — call existing execute() (unchanged)
    let result = commands::execute(line, db, core_root);

    // Postexec — observe the result
    postexec(&ctx, &result, db);

    result
}
