// faelight-shell — Execution Context
// INT-162 Phase 0 — ExecContext: From String-Driven to Context-Driven
//
// This is the foundation layer. Every command execution passes through
// ExecContext. All hooks, logging, and intelligence attach here.
//
// Architecture:
//   line → build_context() → preexec() → dispatch() → postexec() → result

use colored::Colorize;
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

        // Parse cmd and args from raw line -- quote-aware tokenizer
        fn tokenize(s: &str) -> Vec<String> {
            let mut tokens: Vec<String> = Vec::new();
            let mut current = String::new();
            let mut in_quote = false;
            let mut quote_char = ' ';
            for ch in s.chars() {
                match ch {
                    '\"' | '\'' if !in_quote => { in_quote = true; quote_char = ch; }
                    c if in_quote && c == quote_char => { in_quote = false; }
                    ' ' if !in_quote => {
                        if !current.is_empty() { tokens.push(current.clone()); current.clear(); }
                    }
                    c => current.push(c),
                }
            }
            if !current.is_empty() { tokens.push(current); }
            tokens
        }
        let mut parts = raw.splitn(2, ' ');
        let cmd = parts.next().unwrap_or("").to_lowercase();
        let args: Vec<String> = parts.next()
            .map(|s| tokenize(s))
            .unwrap_or_default();

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
                // Match exact path — must be followed by space, end of string, or be standalone
                let matches = raw.split_whitespace()
                    .any(|token| token == *target || token == &format!("{}/", target));
                if matches {
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


    // ── Safety Rule 4: Smarter DELETE confirmation for rm -rf (INT-194) ──────
    if cmd == "rm" {
        let raw_lower = raw.to_lowercase();
        if raw_lower.contains("-rf") || raw_lower.contains("-fr") {
            let target = ctx.args.iter()
                .find(|a| !a.starts_with('-'))
                .map(|s| s.as_str())
                .unwrap_or(".");
            let expanded = if target.starts_with("~/") {
                let home = std::env::var("HOME").unwrap_or_default();
                target.replacen("~/", &format!("{}/", home), 1)
            } else {
                target.to_string()
            };
            let path = std::path::Path::new(&expanded);
            if path.exists() {
                let mut file_count: u64 = 0;
                let mut total_bytes: u64 = 0;
                let mut newest_file = String::new();
                let mut newest_time: u64 = 0;
                if let Ok(walker) = std::fs::read_dir(path) {
                    for entry in walker.flatten() {
                        if let Ok(meta) = entry.metadata() {
                            file_count += 1;
                            total_bytes += meta.len();
                            if let Ok(modified) = meta.modified() {
                                let secs = modified.duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_secs()).unwrap_or(0);
                                if secs > newest_time {
                                    newest_time = secs;
                                    newest_file = entry.file_name().to_string_lossy().to_string();
                                }
                            }
                        }
                    }
                }
                let size_str = if total_bytes > 1_073_741_824 {
                    format!("{:.1} GB", total_bytes as f64 / 1_073_741_824.0)
                } else if total_bytes > 1_048_576 {
                    format!("{:.1} MB", total_bytes as f64 / 1_048_576.0)
                } else {
                    format!("{:.1} KB", total_bytes as f64 / 1024.0)
                };
                println!();
                println!("  {} {}", "⚠️ ".normal(), raw.bright_red());
                println!("  {} {} files, {}", "→".bright_yellow(), file_count, size_str.bright_yellow());
                if !newest_file.is_empty() {
                    println!("  {} Most recent: {}", "→".bright_yellow(), newest_file.bright_white());
                }
                println!("  {} Type {} to confirm, or Ctrl+C to cancel",
                    "→".bright_yellow(), "DELETE".bright_red().bold());
                println!();
                use std::io::BufRead;
                let stdin = std::io::stdin();
                let input = stdin.lock().lines().next()
                    .and_then(|l| l.ok())
                    .unwrap_or_default();
                if input.trim() != "DELETE" {
                    return Some("🛡  Cancelled — type DELETE exactly to confirm".to_string());
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

    // ── INT-233: Knowledge engine query on failure ────────────────────────────
    // After every failed command, search knowledge base for known fixes
    if status == "error" {
        let error_msg = match result {
            CommandResult::Error(e) => e.clone(),
            _ => String::new(),
        };
        let cmd_lower = ctx.cmd.to_lowercase();
        let error_lower = error_msg.to_lowercase();
        // Tokenize error + command into meaningful keywords
        // Filter noise words, try each token against knowledge base
        let noise: &[&str] = &["the","a","an","is","in","of","to","with","and","or","not","for","at","by","on","as","it","be","this","that","was","are"];
        let full_text = format!("{} {}", cmd_lower, error_lower);
        let tokens: Vec<String> = full_text
            .split(|c: char| !c.is_alphanumeric() && c != '0')
            .filter(|t| t.len() >= 3 && !noise.contains(t))
            .map(|t| t.to_string())
            .collect();
        // Use command name as fallback token
        let search_tokens: Vec<String> = if tokens.is_empty() {
            vec![cmd_lower.clone()]
        } else {
            tokens.into_iter().take(5).collect()
        };
        // Try each token against knowledge base (search_tokens built above)
        // Try each token against knowledge_entries table (search id + description + resolution)
        let mut lesson: Option<(String, String, f64)> = None;
        for token in &search_tokens {
            let result = db.conn.query_row(
                "SELECT id, resolution, confidence FROM knowledge_entries
                 WHERE (LOWER(COALESCE(error_signature,'')) LIKE ?1
                     OR LOWER(COALESCE(description,'')) LIKE ?1
                     OR LOWER(COALESCE(resolution,'')) LIKE ?1
                     OR LOWER(id) LIKE ?1)
                 AND confidence >= 0.85
                 ORDER BY confidence DESC, success_count DESC
                 LIMIT 1",
                rusqlite::params![format!("%{}%", token)],
                |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?))
            ).ok();
            if result.is_some() {
                lesson = result;
                break;
            }
        }
        if let Some((id, resolution, confidence)) = lesson {
            println!();
            println!("  {} Friday knows this ({:.0}% confidence):", "🌲".normal(), confidence * 100.0);
            println!("  {} {}", "->".bright_cyan(), resolution.chars().take(120).collect::<String>());
            println!("  {} core knowledge show {}", "·".dimmed(), id.dimmed());
            println!();
        }
    }
    // ── Suggest system -- INT-171 Phase 4 ─────────────────────────────────────
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

        // ── Phase 28: Predictive Suggestions ──────────────────────────────────
        // Read shell_history to find what commands usually follow this one
        if suggestion.is_none() {
            let cmd_prefix = ctx.cmd.clone();
            let full_raw = ctx.raw.clone();

            // Find the most common next command after this one
            let next_cmd: Option<String> = db.conn.query_row(
                "SELECT next_cmd, COUNT(*) as freq FROM (
                    SELECT h2.command as next_cmd
                    FROM shell_history h1
                    JOIN shell_history h2 ON h2.id = h1.id + 1
                    WHERE h1.command = ?1 OR h1.command LIKE ?2
                    AND h2.command != h1.command
                    AND h2.command NOT IN ('q', 'exit', 'clear', 'c', 'pwd')
                ) GROUP BY next_cmd ORDER BY freq DESC LIMIT 1",
                rusqlite::params![cmd_prefix, format!("{}%", full_raw)],
                |r| r.get(0)
            ).ok();

            if let Some(next) = next_cmd {
                let freq: i64 = db.conn.query_row(
                    "SELECT COUNT(*) FROM (
                        SELECT h2.command as next_cmd
                        FROM shell_history h1
                        JOIN shell_history h2 ON h2.id = h1.id + 1
                        WHERE (h1.command = ?1 OR h1.command LIKE ?2)
                        AND h2.command = ?3
                    )",
                    rusqlite::params![cmd_prefix, format!("{}%", ctx.raw), next],
                    |r| r.get(0)
                ).unwrap_or(0);

                let total: i64 = db.conn.query_row(
                    "SELECT COUNT(*) FROM shell_history WHERE command = ?1 OR command LIKE ?2",
                    rusqlite::params![cmd_prefix, format!("{}%", ctx.raw)],
                    |r| r.get(0)
                ).unwrap_or(1);

                let pct = (freq * 100) / total.max(1);
                // Phase 28 gate — INT-186: must meet ALL thresholds before firing
                // Firing on weak patterns trains the user to ignore suggestions
                let confidence = pct as f64 / 100.0;
                let occurrences = freq;
                let accuracy_ok  = pct >= 80;       // >= 80% accuracy
                let volume_ok    = occurrences >= 30; // >= 30 occurrences
                let conf_ok      = confidence >= 0.7; // >= 0.7 confidence
                // Cooldown: no suggestion in last 3 minutes
                let last_suggest: i64 = db.conn.query_row(
                    "SELECT MAX(timestamp) FROM shell_history WHERE command LIKE 'SUGGEST:%'",
                    [], |r| r.get(0)
                ).ok().flatten().unwrap_or(0);
                let now_ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64).unwrap_or(0);
                let cooldown_ok = (now_ts - last_suggest) > 180; // 3 min cooldown
                if accuracy_ok && volume_ok && conf_ok && cooldown_ok {
                    // Full judgment credibility output — INT-186
                    println!("  {} Suggestion: {}",
                        "💡".normal(), next.bright_white());
                    println!("     {} Confidence: {:.2}  ·  {} occurrences  ·  {}% accuracy",
                        "·".dimmed(), confidence, occurrences, pct);
                    println!("     {} Causality: after '{}' this follows {}% of the time ({} sessions)",
                        "·".dimmed(), ctx.cmd.dimmed(), pct, occurrences);
                    // Counterfactual — what would make this wrong
                    let counterfactual = if ctx.cmd == "d" {
                        "already ran d recently"
                    } else if ctx.cmd.starts_with("deploy") {
                        "build failed or different tool deployed"
                    } else if ctx.cmd.starts_with("fg") {
                        "already pushed or no changes staged"
                    } else {
                        "pattern recently changed or different context"
                    };
                    println!("     {} Might be wrong if: {}", "·".dimmed(), counterfactual.dimmed());
                    // Log suggestion for cooldown tracking
                    let _ = db.conn.execute(
                        "INSERT INTO shell_history (command, timestamp) VALUES (?1, ?2)",
                        rusqlite::params![format!("SUGGEST:{}", next), now_ts],
                    );
                }
            }
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
