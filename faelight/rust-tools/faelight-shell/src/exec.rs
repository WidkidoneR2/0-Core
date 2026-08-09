#![allow(clippy::all)]
// faelight-shell — Execution Context
// INT-162 Phase 0 — ExecContext: From String-Driven to Context-Driven
//
// This is the foundation layer. Every command execution passes through
// ExecContext. All hooks, logging, and intelligence attach here.
//
// Architecture:
//   line → build_context() → preexec() → dispatch() → postexec() → result

use crate::commands::{self, CommandResult};
use crate::config::{BeforeRunRule, RuleAction};
use crate::db::ForestDb;
use colored::Colorize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ── ExecContext ───────────────────────────────────────────────────────────────
/// A typed description of every command execution.
/// Replaces raw string passing throughout the shell.
/// INT-169 blocker 8: THE ONLY SOURCE OF EXECUTION IDS.
///
/// The invariant is not the counter, it is the single point: an execution receives exactly
/// one id when its context is created, BEFORE preexec, dispatch, postexec or telemetry
/// observe it. Generating ids at the consumers would yield three observations rather than
/// one execution -- the same split-authority problem this intent exists to remove.
///
/// Process-local and deliberately NOT unique across restarts: the counter begins at 1 in every
/// shell process. That was the whole contract while the only consumer was in-memory correlation.
///
/// INT-191 claimed the extension this comment anticipated. `session_id()` below supplies the
/// PROCESS BOUNDARY, and together they form the persistent lifecycle identity:
///
///     (session_id, execution_id)
///
/// ⚠️ NEITHER HALF IS AN IDENTITY ALONE. Persisting `execution_id` by itself would let two shells
/// both claim 1, 2, 3 -- a key that looks unique and is not, which is the exact class of defect
/// this intent exists to remove. A cross-session identity with stronger guarantees (locking, crash
/// recovery, distributed coordination) remains a different contract and is still out of scope.
/// Not derived from the timestamp: that field already means something else.
static NEXT_EXECUTION_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub fn next_execution_id() -> u64 {
    NEXT_EXECUTION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// INT-191: WHO IS THIS SHELL INSTANCE?
///
/// Nothing owned that question before. `FSH_SESSION_ID` is read in three places and set in NONE --
/// no .rs, .nix, .sh or .fsh file in the tree writes it -- which is why `term_commands` holds 42,376
/// rows under the fallback string "unknown". That fallback turns "the variable is missing" into
/// "there is a shared session called unknown"; absence should TRIGGER CREATION, not become a value.
///
/// Born once per process, from what is already on hand. A shell session needs collision resistance
/// across concurrently running shells, not cryptographic uniqueness, so pid plus start time is
/// sufficient and adds no dependency. Deliberately NOT placed in `session.rs`: that module owns
/// `SessionMemory` and `Momentum` -- application state, a genuinely different concept -- and putting
/// process identity there would be a name that sounds close becoming a second owner.
static SESSION_ID: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn session_id() -> &'static str {
    SESSION_ID.get_or_init(|| {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("{}-{}", std::process::id(), nanos)
    })
}

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
    /// Wall-clock time of execution.
    /// INT-169 blocker 8: SystemTime, not a u64 of seconds -- the type now carries the
    /// meaning instead of every consumer inferring it from this comment. Serialization to
    /// Unix seconds happens at the database boundary, not here.
    /// This is wall-clock IDENTITY ("when did this happen"), NOT elapsed time. A future
    /// duration measurement needs its own monotonic field, because wall-clock can jump
    /// backwards across NTP correction or suspend. They answer different questions and
    /// must not share a name.
    pub timestamp: SystemTime,
    /// Monotonic process-local execution identity. Unique within the lifetime of this
    /// shell process. Every event, hook, telemetry record and debug trace referring to
    /// this execution carries this id.
    pub execution_id: u64,
    /// INT-191: which shell PROCESS produced this execution.
    ///
    /// ⚠️ Required because `execution_id` restarts at 1 in every shell. Persisted alone it would let
    /// two sessions both claim 1, 2, 3 -- a key that looks unique and is not. The lifecycle identity
    /// is the PAIR, `(session_id, execution_id)`, and storage keys on both.
    pub session_id: &'static str,
    /// Whether this command was executed via pipeline
    pub in_pipeline: bool,
}

impl ExecContext {
    /// Build an ExecContext from a raw input line
    /// Build a context from a PLAN plus the source line that produced it (INT-169).
    ///
    /// ★ THE INVARIANT MADE EXPLICIT:  old: text -> context.  new: plan + source -> context.
    /// Structurally identical to `from_line` so preexec/postexec cannot tell which path built
    /// it -- except in the two places where they SHOULD differ:
    ///
    ///   `cmd`  -- argv[0] NOT lowercased. from_line lowercases because its `cmd` doubles as a
    ///             dispatch LOOKUP key; here `cmd` is the EXECUTION IDENTITY. Those were
    ///             accidentally coupled. Consequence to expect, not a regression: postexec sites
    ///             that compare `ctx.cmd` directly against "fg" / "d" / "deploy" will not match a
    ///             capitalised invocation on this path. Where that breaks is exactly where the
    ///             old lookup identity and execution identity were conflated.
    ///   `cwd`   -- honours `plan.cwd` when the plan specifies one; otherwise the current dir,
    ///             same as from_line.
    ///
    /// `raw` stays the SOURCE line on both paths: it is provenance -- history entries, the
    /// `LIKE '{raw}%'` frequency lookups, event payloads, and before-run rule matching. It is a
    /// human-readable label, never re-parsed.
    pub fn from_plan(
        plan: &crate::spine::plan::ExecutionPlan,
        source: &str,
        db: &ForestDb,
    ) -> Self {
        let raw = source.trim().to_string();
        let cwd = plan
            .cwd
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let timestamp = SystemTime::now();
        let mut argv = plan.argv.iter().map(|a| a.to_string_lossy().to_string());
        let cmd = argv.next().unwrap_or_default();
        let args: Vec<String> = argv.collect();
        let intent = db.get_focus_intent();
        ExecContext {
            // INT-191: the spine receives `source` BEFORE any text transformation -- both spine
            // entry points sit above alias expansion and the spine performs no expansions of its
            // own. So raw and expanded are intentionally IDENTICAL here: that is a true fact about
            // today's spine, not a distinction that was lost. Revisit when the flip moves routing
            // below interpretation.
            raw: raw.clone(),
            expanded: raw,
            cmd,
            args,
            cwd,
            intent,
            timestamp,
            execution_id: next_execution_id(),
            session_id: session_id(),
            in_pipeline: false,
        }
    }

    pub fn from_line(raw_line: &str, expanded_line: &str, db: &ForestDb) -> Self {
        // INT-191: TWO ENDPOINTS, not one string doing double duty. `raw` is what crossed the
        // USER boundary; `expanded` is what crossed the EXECUTION boundary. The caller owns that
        // distinction because only the caller knows which stage it is standing at -- this
        // constructor must never try to recover one from the other.
        let raw = raw_line.trim().to_string();
        let expanded = expanded_line.trim().to_string();
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let timestamp = SystemTime::now();

        // INT-169 blocker 8: COMMAND IDENTITY IS STORED AS INVOKED. Case normalization is a
        // CONSUMER POLICY, not a property of the execution context. `cmd` used to be lowercased
        // here because it doubled as a dispatch lookup key -- see from_plan's note on the two
        // identities being accidentally coupled. It no longer does.
        // The invariant, stated as a property rather than a function name so it survives renames:
        // COMMAND IDENTITY AND ARGUMENT VECTOR ARE DERIVED FROM THE SAME TOKENIZATION RESULT.
        // The previous code split the line with `splitn(2, ' ')` for the command word and
        // tokenized only the remainder, so the two could disagree on quoted input.
        // INT-191: tokenize the EXPANDED form. `cmd` and `args` are EXECUTION identity -- what
        // actually runs -- and preexec's protected-path predicate reads `cmd`. Deriving them from
        // `raw` would describe the typed line while the shell ran something else, a worse version
        // of the bug this split fixes. The invariant above is unchanged; only the boundary that
        // tokenization consumes is now explicit rather than incidental.
        let mut tokens = commands::tokenize(expanded.trim()).into_iter();
        let cmd = tokens.next().unwrap_or_default();
        let args: Vec<String> = tokens.collect();

        // Read active intent from db if available
        let intent = db.get_focus_intent();

        ExecContext {
            raw,
            expanded,
            cmd,
            args,
            cwd,
            intent,
            timestamp,
            execution_id: next_execution_id(),
            session_id: session_id(),
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
fn preexec(ctx: &ExecContext, core_root: &str, rules: &[BeforeRunRule]) -> Option<String> {
    // INT-169 blocker 8: `ctx.cmd` preserves the CASE it was invoked with, so protection
    // predicates normalize HERE, where the policy is chosen, rather than relying on stored
    // normalization. (That was a statement about case, not about lifecycle stage.)
    //
    // INT-191: EXECUTION POLICY INSPECTS THE EXECUTION BOUNDARY. `cmd`, `args` and `expanded` all
    // describe what will actually run; `raw` is provenance only -- exactly what crossed the user
    // boundary. Mixing them is unsafe: before the endpoint split both fields held the same string,
    // so scanning `raw` happened to be scanning the executed line. Once they differ, an alias like
    // `nuke = rm -rf /home` gives cmd = "rm" from the executed form while `raw` is just `nuke`,
    // and a scan of `raw` finds no -rf and blocks nothing.
    let cmd = ctx.cmd.to_lowercase();
    let cmd = cmd.as_str();
    let expanded = ctx.expanded.as_str();

    if let Some(reason) = blocks_catastrophic_rm(ctx) {
        return Some(reason);
    }
    // ── Safety Rule 1b: forest source protection ──────────────────────────────
    // The precondition is REPEATED rather than shared, deliberately. This is a separate policy
    // that happens to apply under the same condition, and it depends on core_root and the paths::
    // helpers -- folding it into the predicate above would make that predicate impure and
    // environment-dependent, which is the wrong trade for the thing it protects.
    if cmd == "rm" {
        let expanded_lower = expanded.to_lowercase();
        if expanded_lower.contains("-rf") || expanded_lower.contains("-fr") {
            // Block rm -rf on core source directories
            let core_src = faelight_core::paths::rust_tools_dir()
                .to_string_lossy()
                .to_string();
            let core_engine = format!("{}/engine", core_root);
            let core_intents = faelight_core::paths::intents_dir()
                .to_string_lossy()
                .to_string();
            for protected in &[
                core_src.as_str(),
                core_engine.as_str(),
                core_intents.as_str(),
            ] {
                if expanded.contains(protected) {
                    return Some(format!(
                        "🛡  Blocked: rm -rf on forest source '{}' — use git to manage removals",
                        protected
                    ));
                }
            }
        }
    }
    // ── Safety Rule 3: Protect against self-overwriting core binary ───────────
    if cmd == "cp" || cmd == "mv" {
        // INT-097: was raw.contains("core") -- matched every path under ~/0-core,
        // blocking legit copies. Now block only when the DESTINATION is a core binary.
        let dest = expanded.split_whitespace().last().unwrap_or("");
        let protected = ["scripts/core", ".cargo/bin/core", "/bin/core"];
        let hits_core_binary = dest.ends_with("/core")
            || dest == "core"
            || protected.iter().any(|p| dest.ends_with(p));
        if hits_core_binary && !expanded.contains("deploy") {
            return Some(
                "🛡  Blocked: direct copy to core binary — use deploy script instead".to_string(),
            );
        }
    }

    // ── Config Rules: evaluate before_run rules from config.fsh ────────────
    for rule in rules {
        // INT-191: the EXECUTED form, which is what this saw before the endpoint split -- `raw`
        // held the expanded line then. Matching the typed form instead would make a Block rule
        // bypassable by any alias. ⚠️ Whether config rules SHOULD match typed, executed, or both is
        // a genuine open question with its own contract (Block is a safety predicate; Warn and
        // Suggest are advisory), and it is deliberately NOT decided inside this repair.
        if rule.matches(&ctx.expanded) {
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
        let expanded_lower = expanded.to_lowercase();
        if expanded_lower.contains("-rf") || expanded_lower.contains("-fr") {
            let target = ctx
                .args
                .iter()
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
                                let secs = modified
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_secs())
                                    .unwrap_or(0);
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
                println!("  {} {}", "⚠️ ".normal(), expanded.bright_red());
                println!(
                    "  {} {} files, {}",
                    "→".bright_yellow(),
                    file_count,
                    size_str.bright_yellow()
                );
                if !newest_file.is_empty() {
                    println!(
                        "  {} Most recent: {}",
                        "→".bright_yellow(),
                        newest_file.bright_white()
                    );
                }
                println!(
                    "  {} Type {} to confirm, or Ctrl+C to cancel",
                    "→".bright_yellow(),
                    "DELETE".bright_red().bold()
                );
                println!();
                use std::io::BufRead;
                let stdin = std::io::stdin();
                let input = stdin
                    .lock()
                    .lines()
                    .next()
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

/// Postexec hook — runs after every command
fn postexec(ctx: &ExecContext, result: &CommandResult, db: &ForestDb) {
    // Phase 0: record to shell_history with context
    // Future: INT-177 observability, INT-176 failure memory
    let status = execution_state(result);

    // Log to shell_history
    if status != crate::db::EXEC_EXIT {
        // INT-191: `ctx.expanded`, not `ctx.raw`. This write has ALWAYS recorded the executed
        // form -- before the endpoint split it simply arrived under the name `raw`, because one
        // argument filled both fields. Now that `raw` truthfully means "exactly what the user
        // typed", leaving this as `raw` would make BOTH history writers record the typed line and
        // silently destroy the executed-form record. The two entries are intentionally different:
        // the input boundary writes raw, completed execution writes expanded.
        if let Err(e) = db.save_history_entry(&ctx.expanded) {
            eprintln!("warning: failed to save history: {}", e);
        }
    }

    // ── Failure Memory — INT-176 ──────────────────────────────────────────────
    // Store last failed command so last_command retry/explain/fix can use it
    if status == crate::db::EXEC_ERROR {
        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_failed_command', ?1)",
            rusqlite::params![ctx.raw],
        );
        // Append to session failure log
        // Serialize wall-clock time as Unix seconds for SQLite. unwrap_or_default rather
        // than unwrap: this is the one place a clock anomaly should be handled explicitly,
        // and persistence should not panic because the wall clock misbehaved.
        let ts = ctx
            .timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let error_msg = match result {
            crate::commands::CommandResult::Error(e, _) => e.clone(),
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
    if status == crate::db::EXEC_ERROR {
        // INT-185: prefer the REAL captured stderr (from run_external's tee, stashed in
        // shell_state.last_stderr) over fsh's own "exited N" status string. This is what lets
        // Branch 1 match real fingerprints (error[E0716] etc) from actual command output.
        // Falls back to the CommandResult::Error payload for builtins (no external stderr).
        let error_msg = {
            let stashed: Option<String> = db
                .conn
                .query_row(
                    "SELECT value FROM shell_state WHERE key = 'last_stderr'",
                    [],
                    |r| r.get(0),
                )
                .ok();
            match stashed {
                Some(s) if !s.trim().is_empty() => s,
                _ => match result {
                    CommandResult::Error(e, _) => e.clone(),
                    _ => String::new(),
                },
            }
        };
        let cmd_lower = ctx.cmd.to_lowercase();
        // INT-097: search/filter tools exit non-zero to mean "no match / no result",
        // which is NORMAL, not an error worth a Friday suggestion. Skip the knowledge
        // lookup for them unless they produced real stderr (a genuine error message).
        // INT-195: canonical command derivation. Lowercasing is intentionally preserved
        // until flip blocker 8 revisits telemetry key normalization policy.
        let first_word_owned = crate::commands::command_word(&ctx.cmd).to_lowercase();
        let first_word = first_word_owned.as_str();
        let no_match_tools = [
            "grep", "rg", "egrep", "fgrep", "ripgrep", "find", "fd", "diff", "test", "ag", "ack",
            "fsearch",
        ];
        let is_no_match_exit = no_match_tools.contains(&first_word)
            && (error_msg.trim().is_empty()
                || error_msg.contains("exited")
                || error_msg.contains("os error"));
        if is_no_match_exit {
            return;
        }
        let error_lower = error_msg.to_lowercase();
        // INT-183: Friday's post-failure lesson lookup must fire on RELEVANCE, not on a single
        // loose token. Old logic matched any one token via LIKE %token% against error_signature
        // OR description -- so the generic word "error" hit error_signature "error[E0716]", and
        // "subcommand" (in any clap usage text) hit the clap lesson's description. Both fired a
        // 99% hint at unrelated commands. Two branches now, both requiring the lesson to actually
        // apply:
        //   Branch 1 (entries WITH an error_signature): fire only if the ACTUAL error output
        //     CONTAINS that signature fingerprint (e.g. "error[e0716]"). A sqlite "unable to open
        //     database" error does not contain "error[e0716]" -> silent.
        //   Branch 2 (entries WITHOUT a signature, matched by description prose): require 2+
        //     DISTINCT meaningful token hits in the description -- one word like "subcommand" is
        //     not enough. Generic error-words are noise-filtered so they never count.
        let noise: &[&str] = &[
            "the", "a", "an", "is", "in", "of", "to", "with", "and", "or", "not", "for", "at",
            "by", "on", "as", "it", "be", "this", "that", "was", "are",
            // INT-183: generic error-words -- appear in almost every failure, match almost every
            // lesson; must never count as a meaningful, relevance-bearing token.
            "error", "failed", "failure", "cannot", "unable", "warning", "panicked", "panic",
            "exited", "code", "err", "errors",
        ];
        let full_text = format!("{} {}", cmd_lower, error_lower);
        let tokens: Vec<String> = full_text
            .split(|c: char| !c.is_alphanumeric() && c != '0')
            .filter(|t| t.len() >= 3 && !noise.contains(t))
            .map(|t| t.to_string())
            .collect();
        // Deduplicate while preserving order (distinct meaningful tokens).
        let mut search_tokens: Vec<String> = Vec::new();
        for t in tokens {
            if !search_tokens.contains(&t) {
                search_tokens.push(t);
            }
        }
        search_tokens.truncate(8);

        // Branch 1: an entry whose error_signature fingerprint is actually PRESENT in the error.
        // INSTR(error_lower, sig) > 0 means the real output contains the signature.
        let mut lesson: Option<(String, String, f64)> = db
            .conn
            .query_row(
                "SELECT id, resolution, confidence FROM knowledge_entries
                 WHERE COALESCE(error_signature,'') != ''
                   AND confidence >= 0.85
                   AND INSTR(?1, LOWER(error_signature)) > 0
                 ORDER BY confidence DESC, success_count DESC
                 LIMIT 1",
                rusqlite::params![error_lower],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, f64>(2)?,
                    ))
                },
            )
            .ok();

        // Branch 2: no signature match -- fall back to description prose, but require 2+ distinct
        // meaningful token hits so a single loose word cannot fire a hint.
        if lesson.is_none() && !search_tokens.is_empty() {
            let mut best: Option<(String, String, f64, usize)> = None;
            let mut stmt = db.conn.prepare(
                "SELECT id, resolution, confidence, LOWER(COALESCE(description,'')) as descr
                 FROM knowledge_entries
                 WHERE COALESCE(error_signature,'') = ''
                   AND confidence >= 0.85",
            );
            if let Ok(ref mut stmt) = stmt {
                let rows = stmt.query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, f64>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                });
                if let Ok(rows) = rows {
                    for row in rows.flatten() {
                        let (id, resolution, confidence, descr) = row;
                        // INT-185: word-boundary match, NOT substring. The token "current"
                        // (from a resolved path like /run/current-system/) must NOT match
                        // "concurrent" inside a description. Split descr into whole words with
                        // the SAME rule the token builder uses, then require exact word membership.
                        let descr_words: std::collections::HashSet<&str> = descr
                            .split(|c: char| !c.is_alphanumeric() && c != '0')
                            .filter(|w| !w.is_empty())
                            .collect();
                        let hits = search_tokens
                            .iter()
                            .filter(|t| descr_words.contains(t.as_str()))
                            .count();
                        if hits >= 2 {
                            let better = match &best {
                                None => true,
                                Some((_, _, bc, bh)) => {
                                    hits > *bh || (hits == *bh && confidence > *bc)
                                }
                            };
                            if better {
                                best = Some((id, resolution, confidence, hits));
                            }
                        }
                    }
                }
            }
            if let Some((id, resolution, confidence, _hits)) = best {
                lesson = Some((id, resolution, confidence));
            }
        }
        if let Some((id, resolution, confidence)) = lesson {
            println!();
            println!(
                "  {} Friday knows this ({:.0}% confidence):",
                "🌲".normal(),
                confidence * 100.0
            );
            println!(
                "  {} {}",
                "->".bright_cyan(),
                resolution.chars().take(120).collect::<String>()
            );
            println!("  {} core knowledge show {}", "·".dimmed(), id.dimmed());
            println!();
        }
    }
    // ── Suggest system -- INT-171 Phase 4 ─────────────────────────────────────
    if status == crate::db::EXEC_OK || status == crate::db::EXEC_EMPTY {
        // INT-169 blocker 8: local comparison key. ctx.cmd stays what the user invoked.
        let cmd_key = ctx.cmd.to_lowercase();
        let suggestion = match cmd_key.as_str() {
            "fg" if ctx.args.first().map(|s| s.as_str()) == Some("commit") => {
                Some("💡 Suggestion: run d — verify health after committing")
            }
            "deploy" => Some("💡 Suggestion: run d — verify health after deploy"),
            "cicomplete" => Some("💡 Next: fg commit — record the completion"),
            "cistart" => Some("💡 Next: read the intent carefully before writing any code"),
            "paru" | "pacman" => Some("💡 That isn't a NixOS command — apply changes with deploy (it rebuilds + health-checks)"),
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
            let next_cmd: Option<String> = db
                .conn
                .query_row(
                    "SELECT next_cmd, COUNT(*) as freq FROM (
                    SELECT h2.command as next_cmd
                    FROM shell_history h1
                    JOIN shell_history h2 ON h2.id = h1.id + 1
                    WHERE h1.command = ?1 OR h1.command LIKE ?2
                    AND h2.command != h1.command
                    AND h2.command NOT IN ('q', 'exit', 'clear', 'c', 'pwd')
                ) GROUP BY next_cmd ORDER BY freq DESC LIMIT 1",
                    rusqlite::params![cmd_prefix, format!("{}%", full_raw)],
                    |r| r.get(0),
                )
                .ok();

            if let Some(next) = next_cmd {
                let freq: i64 = db
                    .conn
                    .query_row(
                        "SELECT COUNT(*) FROM (
                        SELECT h2.command as next_cmd
                        FROM shell_history h1
                        JOIN shell_history h2 ON h2.id = h1.id + 1
                        WHERE (h1.command = ?1 OR h1.command LIKE ?2)
                        AND h2.command = ?3
                    )",
                        rusqlite::params![cmd_prefix, format!("{}%", ctx.raw), next],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);

                let total: i64 = db
                    .conn
                    .query_row(
                        "SELECT COUNT(*) FROM shell_history WHERE command = ?1 OR command LIKE ?2",
                        rusqlite::params![cmd_prefix, format!("{}%", ctx.raw)],
                        |r| r.get(0),
                    )
                    .unwrap_or(1);

                let pct = (freq * 100) / total.max(1);
                // Phase 28 gate — INT-186: must meet ALL thresholds before firing
                // Firing on weak patterns trains the user to ignore suggestions
                let confidence = pct as f64 / 100.0;
                let occurrences = freq;
                let accuracy_ok = pct >= 80; // >= 80% accuracy
                let volume_ok = occurrences >= 30; // >= 30 occurrences
                let conf_ok = confidence >= 0.7; // >= 0.7 confidence
                                                 // Cooldown: no suggestion in last 3 minutes
                let last_suggest: i64 = db
                    .conn
                    .query_row(
                        "SELECT MAX(timestamp) FROM shell_history WHERE command LIKE 'SUGGEST:%'",
                        [],
                        |r| r.get(0),
                    )
                    .ok()
                    .flatten()
                    .unwrap_or(0);
                let now_ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let cooldown_ok = (now_ts - last_suggest) > 180; // 3 min cooldown
                if accuracy_ok && volume_ok && conf_ok && cooldown_ok {
                    // Full judgment credibility output — INT-186
                    println!("  {} Suggestion: {}", "💡".normal(), next.bright_white());
                    println!(
                        "     {} Confidence: {:.2}  ·  {} occurrences  ·  {}% accuracy",
                        "·".dimmed(),
                        confidence,
                        occurrences,
                        pct
                    );
                    println!(
                        "     {} Causality: after '{}' this follows {}% of the time ({} sessions)",
                        "·".dimmed(),
                        ctx.cmd.dimmed(),
                        pct,
                        occurrences
                    );
                    // Counterfactual — what would make this wrong
                    let cmd_cf = ctx.cmd.to_lowercase();
                    let counterfactual = if cmd_cf == "d" {
                        "already ran d recently"
                    } else if cmd_cf.starts_with("deploy") {
                        "build failed or different tool deployed"
                    } else if cmd_cf.starts_with("fg") {
                        "already pushed or no changes staged"
                    } else {
                        "pattern recently changed or different context"
                    };
                    println!(
                        "     {} Might be wrong if: {}",
                        "·".dimmed(),
                        counterfactual.dimmed()
                    );
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
/// What the spine needs from the live shell session, supplied by the REPL that owns it.
///
/// Session variables and the last exit code are PROCESS state, not persistent forest knowledge,
/// so they are passed in rather than pushed into ForestDb. `commands/mod.rs` never sees them:
/// builtins are not the owner of shell session state.
pub struct ShellContext<'a> {
    pub shell_vars: &'a std::collections::HashMap<String, String>,
    pub last_exit_code: Option<i32>,
}

impl crate::spine::plan::VarResolver for ShellContext<'_> {
    /// Matches legacy `expand_vars` exactly: session vars first, then process env. `None` here
    /// means UNSET, which the caller renders as the empty string.
    fn resolve(&self, name: &str) -> Option<String> {
        self.shell_vars
            .get(name)
            .cloned()
            .or_else(|| std::env::var(name).ok())
    }
    fn last_exit(&self) -> Option<i32> {
        self.last_exit_code
    }
    fn pid(&self) -> u32 {
        std::process::id()
    }
}

/// INT-169: execute an already-decided plan, with the same hooks the text path gets.
///
/// ★ THE PLAN ARRIVES DECIDED. This does NOT parse -- the executor must not secretly own
/// parsing, or the eventual flip would be dishonest about where decisions are made. `source` is
/// carried alongside purely as provenance (history, telemetry, before-run rule matching).
///
/// Mirrors `execute_with_context` deliberately: same preexec (which can BLOCK), same postexec
/// (INT-185/233 knowledge engine), so the spine path cannot silently lose the safety gate or the
/// telemetry that the text path has.
///
/// ⚠️ KNOWN, not blocking: preexec matches before-run rules against RAW TEXT, not against the
/// plan. That keeps working here because `source` is the same line, but it means the safety layer
/// is text-matching rather than plan-inspecting. Making policy plan-aware is a later hardening
/// step and deliberately not forced into this migration.
pub fn execute_spine(
    plan: &crate::spine::plan::ExecutionPlan,
    source: &str,
    db: &ForestDb,
    core_root: &str,
    rules: &[BeforeRunRule],
) -> CommandResult {
    let ctx = ExecContext::from_plan(plan, source, db);
    if let Some(block_reason) = preexec(&ctx, core_root, rules) {
        return CommandResult::Error(block_reason, 1);
    }
    let result = commands::execute_plan_dispatch(plan, source, db, core_root);
    postexec(&ctx, &result, db);
    result
}

/// INT-169 blocker 4: the shell-side half of `spine::plan::CommandRunner`.
///
/// Everything the spine refuses to know lives here: a database, a core root, and a process. The
/// trait passes an `ExecutionPlan` and gets back a string, so the spine never learns any of it.
///
/// ⚠️ THREE KNOWN GAPS, recorded rather than discovered later:
///
/// 1. BUILTINS DO NOT HONOUR `IoPlan::Capture`. Dispatch tries builtins first, and one that
///    returns `Output(s)` captures correctly -- but a builtin that `println!`s directly and
///    returns `Empty` leaks to the terminal and captures nothing. The missing invariant belongs to
///    the execution layer: if a plan requests capture, no branch may write to the terminal.
///
/// 2. THE SAFETY GUARD DOES NOT RUN ON A NESTED COMMAND. `execute_plan_dispatch` performs no
///    preexec. Routing through `execute_spine` instead would fix that and break INT-191's ruling,
///    because its postexec writes a `command_execution` row and a substitution is not a user
///    command. Closing this properly needs the ExecContext/plan unification that IS blocker 2.
///    Bounded meanwhile: only the opt-in `spine exec` door reaches here, and the OUTER command is
///    still guarded.
///
/// 3. PROVENANCE IS EMPTY, and the fix is NOT to hand the trait a string. Dispatch carries
///    `source` for the few builtins wanting original text, and a plan does not contain one.
///    The trait refusing text is correct -- source text is not execution identity. This
///    belongs to the blocker 2 family: once ExecutionPlan carries execution identity, the
///    missing field arrives without reopening the string-based seam this closes.
/// INT-169 blocker 5: the shell-side half of `spine::plan::GlobResolver`.
///
/// ⚠️ cwd comes from the PROCESS. `ShellContext` carries only shell_vars and last_exit_code, and
/// exec.rs already falls back to `current_dir()` elsewhere, so this matches the existing execution
/// model rather than inventing a second source of truth. Giving ShellContext a cwd later becomes an
/// intentional migration point instead of a silent inconsistency.
///
/// ⚠️ NO TILDE EXPANSION. Legacy expands a leading `~/` before globbing; the spine has no tilde
/// phase yet, so `~/x*` will not match here. Stated rather than smuggled in, because tilde is its
/// own expansion and belongs with the others.
struct SpineGlobResolver {
    cwd: std::path::PathBuf,
}

/// The legacy matcher's algorithm over a STRUCTURED pattern.
///
/// ★ Identical two-pointer walk with backtracking, so behaviour matches `expand::glob_match` -- but
/// it can do what that one cannot: tell a wildcard from a literal asterisk. `GlobPart::Many` is a
/// wildcard; `GlobPart::Literal('*')` matches an actual `*` in a filename, which is what makes
/// `'*'` and `"*"` inert.
fn glob_parts_match(pattern: &[crate::spine::plan::GlobPart], name: &str) -> bool {
    use crate::spine::plan::GlobPart;
    let n: Vec<char> = name.chars().collect();
    let (mut pi, mut ni) = (0usize, 0usize);
    let mut star_pi = usize::MAX;
    let mut star_ni = 0usize;
    while ni < n.len() {
        let here = pattern.get(pi);
        let consumes = match here {
            Some(GlobPart::Literal(c)) => *c == n[ni],
            Some(GlobPart::Any) => true,
            _ => false,
        };
        if consumes {
            pi += 1;
            ni += 1;
        } else if matches!(here, Some(GlobPart::Many)) {
            star_pi = pi;
            star_ni = ni;
            pi += 1;
        } else if star_pi != usize::MAX {
            pi = star_pi + 1;
            star_ni += 1;
            ni = star_ni;
        } else {
            return false;
        }
    }
    while matches!(pattern.get(pi), Some(GlobPart::Many)) {
        pi += 1;
    }
    pi == pattern.len()
}

impl crate::spine::plan::GlobResolver for SpineGlobResolver {
    fn expand(&self, pattern: &[crate::spine::plan::GlobPart]) -> Vec<std::ffi::OsString> {
        use crate::spine::plan::GlobPart;
        // A pattern may name a directory: `src/*.rs`. Split at the LAST literal separator -- the
        // prefix must be literal text (a wildcard spanning directories is `**`, which neither this
        // matcher nor the legacy one implements), and the remainder is the filename pattern.
        let sep = pattern
            .iter()
            .rposition(|p| matches!(p, GlobPart::Literal('/')));
        let (prefix, file_pattern) = match sep {
            Some(i) => {
                let dir: String = pattern[..i]
                    .iter()
                    .map(|p| match p {
                        GlobPart::Literal(c) => *c,
                        GlobPart::Any => '?',
                        GlobPart::Many => '*',
                    })
                    .collect();
                (dir, &pattern[i + 1..])
            }
            None => (String::new(), pattern),
        };
        let dir = if prefix.is_empty() {
            self.cwd.clone()
        } else if prefix.starts_with('/') {
            std::path::PathBuf::from(&prefix)
        } else {
            self.cwd.join(&prefix)
        };

        let mut matches: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy().to_string();
                if glob_parts_match(file_pattern, &name_str) {
                    matches.push(if prefix.is_empty() {
                        name_str
                    } else {
                        format!("{prefix}/{name_str}")
                    });
                }
            }
        }
        // Sorted, matching legacy. An empty result is reported as empty: what the shell does about
        // a pattern that matched nothing is the caller's decision, not this adapter's.
        matches.sort();
        matches.into_iter().map(std::ffi::OsString::from).collect()
    }
}

#[cfg(test)]
mod glob_matcher_tests {
    use super::glob_parts_match;
    use crate::spine::plan::GlobPart;

    /// Build a pattern the way lowering would for fully UNQUOTED text, so the two matchers are
    /// being asked the same question.
    fn unquoted(pattern: &str) -> Vec<GlobPart> {
        pattern
            .chars()
            .map(|c| match c {
                '*' => GlobPart::Many,
                '?' => GlobPart::Any,
                _ => GlobPart::Literal(c),
            })
            .collect()
    }

    /// ★ THE EXTRACTION'S WHOLE PURPOSE: the structured matcher must answer identically to the
    /// legacy one on every pattern that contains no quoted metacharacter. If these ever disagree,
    /// the spine and the legacy path would glob differently and the migration audit would report a
    /// divergence caused by this rewrite rather than by the spine.
    #[test]
    fn agrees_with_the_legacy_matcher_on_unquoted_patterns() {
        let patterns = [
            "*", "*.rs", "a*", "*a", "a*b", "?", "a?c", "??", "*?", "?*", "a**b", "", "abc", "*.*",
            "a*b*c",
        ];
        // ⚠️ Names containing `*` or `?` are EXCLUDED, and that is a finding rather than a
        // convenience: see `diverges_from_legacy_where_legacy_is_wrong` below. On every name that
        // does not contain a metacharacter, the two matchers must agree exactly.
        let names = [
            "", "a", "abc", "a.rs", "main.rs", "ab", "aab", "abcb", "a.b.c", "aXb", "a.b",
        ];
        for p in patterns {
            for n in names {
                assert_eq!(
                    glob_parts_match(&unquoted(p), n),
                    crate::expand::glob_match(p, n),
                    "pattern {p:?} against {n:?}"
                );
            }
        }
    }

    /// ★ A DELIBERATE DIVERGENCE, recorded so it is a decision rather than a surprise.
    ///
    /// `expand::glob_match` tests `p[pi] == n[ni]` BEFORE testing whether the pattern character is
    /// a wildcard. So when a filename genuinely contains `*` or `?`, the pattern's metacharacter
    /// facing the same character degrades into a literal one-for-one match. `*?` against `*` then
    /// consumes the star literally, leaves `?` unmatched, and answers false.
    ///
    /// The correct answer is true: `*` matches zero characters and `?` matches the `*` in the name.
    /// Bash agrees. The structured matcher answers correctly because a wildcard is a VARIANT here
    /// and can never be compared as text.
    ///
    /// This follows the precedent set for `${VAR:-default}` in the variable milestone: where legacy
    /// is wrong, the spine is correct and the difference stays VISIBLE in the migration audit.
    /// Copying the bug to keep the audit quiet would make it measure agreement instead of
    /// correctness.
    #[test]
    fn diverges_from_legacy_where_legacy_is_wrong() {
        assert!(
            glob_parts_match(&unquoted("*?"), "*"),
            "a wildcard then one character should match a single-star filename"
        );
        assert!(
            !crate::expand::glob_match("*?", "*"),
            "legacy gets this wrong, which is why the equivalence test excludes such names"
        );
    }

    /// What the legacy matcher CANNOT express, and the reason the structured form exists: a
    /// literal asterisk matches only an actual asterisk, never everything.
    #[test]
    fn a_literal_star_is_not_a_wildcard() {
        let pattern = vec![GlobPart::Literal('*')];
        assert!(glob_parts_match(&pattern, "*"));
        assert!(!glob_parts_match(&pattern, "anything"));
        assert!(!glob_parts_match(&pattern, ""));
        // And the legacy matcher disagrees, which is exactly the bug this replaces.
        assert!(crate::expand::glob_match("*", "anything"));
    }
}

struct SpineCommandRunner<'a> {
    db: &'a ForestDb,
    core_root: &'a str,
    /// INT-169 blocker 2: needed because a nested command must face the SAME policy gate as a
    /// typed one. Held rather than rediscovered, so "which rules are active" cannot drift between
    /// the two paths.
    rules: &'a [BeforeRunRule],
}

impl crate::spine::plan::CommandRunner for SpineCommandRunner<'_> {
    fn run_capture(&self, plan: &crate::spine::plan::ExecutionPlan) -> Result<String, String> {
        // INT-169 blocker 2: THE POLICY GATE, which a substitution previously walked straight
        // past. `execute_plan_dispatch` performs no preexec, so `$(rm -rf /somewhere)` reached a
        // process without ever meeting the guard that a typed `rm -rf /somewhere` cannot avoid.
        // A protection that applies everywhere except inside `$()` is worse than none, because it
        // is trusted.
        //
        // ⚠️ preexec ONLY. No postexec, and that asymmetry is deliberate: postexec records the
        // COMPLETION OF AN EXECUTION, and INT-191 ruled that a substitution is an expansion rather
        // than a user command, so it must not create a `command_execution` row. Blocking is a
        // decision about whether to evaluate an expression; it is not a shell lifecycle event.
        let ctx = crate::exec::ExecContext::from_plan(plan, "", self.db);
        if let Some(reason) = preexec(&ctx, self.core_root, self.rules) {
            return Err(reason);
        }
        // Matched exhaustively, and each arm for a stated reason -- no catch-all, because a
        // future variant should be a compile error here rather than a generic message.
        match commands::execute_plan_dispatch(plan, "", self.db, self.core_root) {
            CommandResult::Output(s) => Ok(s),
            // Produced nothing, so substituted nothing. Correct, not an error.
            CommandResult::Empty => Ok(String::new()),
            CommandResult::Error(e, _) => Err(e),
            // Stringifying a structured Value here would make this adapter invent display
            // semantics for a layer it does not own. What `$(tt)` should mean is a Lane 5
            // question about structured pipelines, not something to settle by accident.
            CommandResult::Value(_) => {
                Err("nested command produced a structured value, not text".to_string())
            }
            // A substitution that tries to terminate the shell is a FAILED capture, not an empty
            // one -- swallowing it as `Ok("")` would make `$(exit)` silently expand to nothing.
            CommandResult::Exit(_) => Err("nested command attempted to exit the shell".to_string()),
            // A contract violation rather than a normal outcome: dispatch already falls back to
            // execute_plan, so it should never hand this back.
            CommandResult::NotBuiltin => {
                Err("nested dispatch returned NotBuiltin, which it should never do".to_string())
            }
        }
    }
}

/// Debug/test convenience: parse and lower a line, then execute the resulting plan.
///
/// Kept SEPARATE from `execute_spine` so the real seam stays visible -- the flip will supply a
/// plan, not a string. Only this wrapper knows how a plan comes into existence.
/// Why the spine did not produce a plan. Two PHASES, kept apart deliberately: `LowerError` has
/// the right boundary today and must not learn about parsing.
#[derive(Debug)]
pub enum SpineAttemptError {
    /// ⚠️ Read only through `Debug` today, which dead-code analysis does not count -- hence the
    /// allow. The error is KEPT rather than discarded because the audit already renders decline
    /// reasons from it, and a router that wanted to distinguish a lex error from an unsupported
    /// operator would need exactly this. Dropping it to silence a warning would repeat the
    /// mistake the whole decline-reason work existed to fix.
    #[allow(dead_code)]
    Parse(crate::spine::parser::ParseError),
    /// INT-169 G2: the input is not finished. NOT a parse failure and NOT a refusal -- more input
    /// may make it spine-owned.
    #[allow(dead_code)]
    Incomplete(crate::spine::lexer::LexIncomplete),
    /// INT-169 G2: valid shell the spine intentionally does not own. THIS is what legacy fallback
    /// means; `Parse` is not. The doc above predicted this variant -- "a router that wanted to
    /// distinguish a lex error from an unsupported operator would need exactly this."
    #[allow(dead_code)]
    Refused(crate::spine::parser::Refusal),
    Lower(crate::spine::plan::LowerError),
}

/// Parse and lower one source line with the FULL capability set. ★ ONE construction path, shared by
/// the explicit `spine-exec` door and the router below -- a second copy of the runner/globber setup
/// is how the two would silently drift apart.
fn lower_spine_source(
    source: &str,
    shell: &ShellContext,
    db: &ForestDb,
    core_root: &str,
    rules: &[BeforeRunRule],
) -> Result<Vec<crate::spine::plan::ExecutionPlan>, SpineAttemptError> {
    // ⭐ INT-169 G2: THE OWNERSHIP DECISION TRAVELS; IT IS NOT FLATTENED AND RE-DERIVED. This read
    // `.map_err(SpineAttemptError::Parse)?`, which collapsed four meanings into one variant -- and the
    // router then pattern-matched them back out, with `Parse(_) => Declined` as a catch-all that
    // silently routed incompleteness, emptiness, real refusals and any future parse error to legacy
    // alike. Each arm now keeps its meaning all the way to the router.
    let node = match crate::spine::parser::parse(source) {
        crate::spine::parser::ParseResult::Complete(n) => n,
        crate::spine::parser::ParseResult::Incomplete(i) => {
            return Err(SpineAttemptError::Incomplete(i))
        }
        crate::spine::parser::ParseResult::Refused(r) => return Err(SpineAttemptError::Refused(r)),
        crate::spine::parser::ParseResult::Invalid(e) => return Err(SpineAttemptError::Parse(e)),
    };
    // INT-169 blocker 4: substitutions need a runner, and refusing one here would leave the
    // capability with no consumer. Blocker 5: a door with a runner but no globber would be
    // incoherent -- substitutions could run while pathname expansion silently could not.
    let runner = SpineCommandRunner {
        db,
        core_root,
        rules,
    };
    let glob = SpineGlobResolver {
        cwd: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    };
    let ctx = crate::spine::plan::LowerContext {
        vars: Some(shell),
        runner: Some(&runner),
        glob: Some(&glob),
    };
    // INT-200: ONE SHAPE FOR THE CALLER. `lower_pipeline` returns a plan per stage and wraps a
    // single command in a one-element vector, so the router branches on LENGTH rather than on
    // which AST variant it happened to get.
    crate::spine::plan::lower_pipeline(&node, &ctx).map_err(SpineAttemptError::Lower)
}

/// How a backgrounded line should be started. A configured Command cannot express a pipeline,
/// whose earlier stages must already be running before the last one can be handed over.
///
/// ★ exec.rs SPAWNS BUT STILL NEVER LEARNS WHAT A JobTable IS, and main.rs still never learns how
/// to lower. The boundary is unchanged; only what crosses it grew a second shape.
pub enum BackgroundAttempt {
    /// One command, not yet spawned -- the job table starts it.
    Single(std::process::Command, String),
    /// A pipeline, already running, in stage order. The LAST child carries the status.
    Chain(Vec<std::process::Child>, String),
}

/// INT-200: the BACKGROUND door. Returns how a `cmd &` line should be STARTED, or `None` when the
/// line is not a background job at all.
///
/// ★ WHY A SIBLING RATHER THAN AN UNWRAP IN main.rs: `lower_spine_source` parses its own source and
/// builds the runner, globber and capabilities internally, so main.rs could only reach them by
/// re-rendering the AST back to text -- the string-reinspection the spine exists to end -- or by
/// keeping a second copy of the capability setup, which is the drift INT-169 extracted that
/// function to prevent. Instead this returns a BackgroundAttempt and main.rs hands it to the job
/// table. exec.rs never learns what a JobTable is; main.rs never learns how to lower.
///
/// ⚠️ The OPERAND is lowered, not the wrapper. `AstNode::Background` refuses at both lowering
/// entries on purpose -- an ExecutionPlan describes one FOREGROUND process, and "do not wait" is a
/// scheduling decision that has no business inside a description of what to run.
pub fn try_spine_background_command(
    source: &str,
    shell: &ShellContext,
    db: &ForestDb,
    core_root: &str,
    rules: &[BeforeRunRule],
) -> Option<Result<BackgroundAttempt, SpineAttemptError>> {
    // INT-169 G2: only a Complete parse can be a background command. The other three arms are not
    // this function's decision to make -- it answers "is this a background command?", and anything
    // else returns None so the caller's own routing runs.
    let crate::spine::parser::ParseResult::Complete(node) = crate::spine::parser::parse(source)
    else {
        return None;
    };
    let crate::spine::ast::AstNode::Background(inner) = node.node else {
        return None;
    };
    let runner = SpineCommandRunner {
        db,
        core_root,
        rules,
    };
    let glob = SpineGlobResolver {
        cwd: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    };
    let ctx = crate::spine::plan::LowerContext {
        vars: Some(shell),
        runner: Some(&runner),
        glob: Some(&glob),
    };
    let plans = match crate::spine::plan::lower_pipeline(&inner, &ctx) {
        Ok(p) if p.is_empty() => return None,
        Ok(p) => p,
        Err(e) => return Some(Err(SpineAttemptError::Lower(e))),
    };
    let label = plans
        .first()
        .and_then(|p| p.argv.first())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| source.to_string());
    // A PIPELINE IS SPAWNED HERE RATHER THAN HANDED OVER AS A COMMAND, because every stage must
    // already be running before the last one can be registered. So the chain crosses the boundary
    // ALIVE, and its earlier stages travel with it -- the job table reaps them. Registering only
    // the tail would leave every upstream stage a zombie.
    if plans.len() > 1 {
        return Some(
            crate::commands::background_pipeline(&plans)
                .map(|children| BackgroundAttempt::Chain(children, label))
                .map_err(|e| {
                    SpineAttemptError::Lower(crate::spine::plan::LowerError::InvalidPlan {
                        message: e,
                        span: node.span,
                    })
                }),
        );
    }
    // A REDIRECT DOES NOT DECLINE HERE. `background_command` wires `plan.io` through
    // `configure_file_io`, the same function the foreground path uses -- its doc owns the
    // io rules and the deliberate absence of a tee, and restating them here would create
    // the second owner that arrangement exists to prevent. The one io it refuses is
    // `IoPlan::Capture`, which arrives below as an Err rather than as a decline.
    // ⚠️ AN IO FAILURE SURFACES, it does not fall back. `None` here would send the line to
    // legacy, which would then fail the same way with a worse message -- and a redirect
    // target that cannot be opened is the user's problem to see, not a routing decision.
    // Refusals fall back; defects surface. Same rule as InvalidPlan at the router.
    Some(
        crate::commands::background_command(&plans[0])
            .map(|(c, l)| BackgroundAttempt::Single(c, l))
            .map_err(|e| {
                SpineAttemptError::Lower(crate::spine::plan::LowerError::InvalidPlan {
                    message: e,
                    span: node.span,
                })
            }),
    )
}

/// The EXPLICIT door (`spine-exec <cmd>`). You asked for the spine, so every failure is reported
/// rather than hidden -- that is the difference from the router, and the reason both exist.
pub fn execute_spine_source(
    source: &str,
    shell: &ShellContext,
    db: &ForestDb,
    core_root: &str,
    rules: &[BeforeRunRule],
) -> CommandResult {
    match lower_spine_source(source, shell, db, core_root, rules) {
        // Same branch as the router: `spine-exec ls | grep x` should run the pipeline, not
        // report a shape it cannot handle.
        Ok(plans) => match plans.as_slice() {
            [one] => execute_spine(one, source, db, core_root, rules),
            many => crate::commands::execute_pipeline_plans(many, db),
        },
        Err(e) => CommandResult::Error(format!("spine: {e:?}"), 1),
    }
}

/// INT-169 blocker 6: THE ROUTER. `None` means NOT MINE -- the caller hands the ORIGINAL source to
/// the legacy path, exactly as if routing did not exist.
///
/// ★ REFUSALS FALL BACK; DEFECTS SURFACE. A parse error means this is not spine syntax (a pipe, a
/// redirect, a heredoc) and legacy owns it. `MissingCapability` and `UnsupportedConstruct` are the
/// same answer from the next phase down. But `InvalidPlan` means the spine ACCEPTED ownership and
/// built something internally inconsistent -- falling back there would make legacy an accidental
/// recovery mechanism and erase the only evidence of a spine bug.
///
/// ⚠️ This must never be `execute_spine_source(...).into()`. That function turns a refusal into an
/// error string, so a router built on it would swallow every piped command instead of declining it.
/// INT-169: ROUTING COUNTERS. A green test suite proves COMPATIBILITY -- a command behaves the same
/// whether the spine ran it or the router declined and legacy did. It does not prove MIGRATION.
/// These answer the second question, and they are split by REASON because a single total hides the
/// work queue: a parse decline is a grammar gap, a capability decline is a lowering feature, and an
/// unsupported construct is an intentional boundary.
static SPINE_CLAIMED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static SPINE_DECLINED_PARSE: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
static SPINE_DECLINED_CAPABILITY: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
static SPINE_DECLINED_CONSTRUCT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

fn bump(c: &std::sync::atomic::AtomicUsize) {
    c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// One line per session, not per command. `None` when the spine never saw anything.
pub fn spine_routing_report() -> Option<String> {
    let get = |c: &std::sync::atomic::AtomicUsize| c.load(std::sync::atomic::Ordering::Relaxed);
    let (claimed, parse, cap, cons) = (
        get(&SPINE_CLAIMED),
        get(&SPINE_DECLINED_PARSE),
        get(&SPINE_DECLINED_CAPABILITY),
        get(&SPINE_DECLINED_CONSTRUCT),
    );
    if claimed + parse + cap + cons == 0 {
        return None;
    }
    Some(format!(
        "  spine routing: claimed {claimed} · declined {} (parse {parse}, capability {cap}, construct {cons})",
        parse + cap + cons
    ))
}

/// What the router did with a line. `Option<CommandResult>` carried THREE meanings in a two-state
/// type: ran it, declined it, and -- once a diagnostic could be reported without executing -- owned
/// it without producing a result.
///
/// ★ NAMED BY OWNERSHIP, NOT BY TODAY'S CASE. `Handled` means the spine claimed this input and no
/// further execution should occur; a parser diagnostic is merely the first thing that fits.
pub enum SpineOutcome {
    /// A command ran and produced a result.
    Executed(CommandResult),
    /// The spine consumed the input and established the exit status. Output, if any, is done.
    Handled { exit_code: i32 },
    /// Not the spine's. Legacy may try.
    Declined,
}

pub fn try_execute_spine_source(
    source: &str,
    shell: &ShellContext,
    db: &ForestDb,
    core_root: &str,
    rules: &[BeforeRunRule],
) -> SpineOutcome {
    match lower_spine_source(source, shell, db, core_root, rules) {
        Ok(plans) => {
            bump(&SPINE_CLAIMED);
            // A single command keeps its EXISTING path -- builtins, the redirect rules, the
            // prefer-the-real-binary check. Only a genuine pipeline takes the chaining executor,
            // so nothing about ordinary commands changes.
            SpineOutcome::Executed(match plans.as_slice() {
                [one] => execute_spine(one, source, db, core_root, rules),
                many => crate::commands::execute_pipeline_plans(many, db),
            })
        }
        // ⚠️ A DEFECT, NOT A REFUSAL. `echo a >` is not shell the spine declines to own -- it is a
        // mistake the parser already located. Declining hands it to legacy, which re-derives the
        // same position by scanning the text. The spine reports it and keeps ownership.
        Err(SpineAttemptError::Parse(
            crate::spine::parser::ParseError::MissingRedirectTarget { span, .. },
        )) => {
            bump(&SPINE_CLAIMED);
            eprint!(
                "{}",
                crate::error::render_redirect_error_at(source, span.start)
            );
            SpineOutcome::Handled { exit_code: 2 }
        }
        Err(SpineAttemptError::Parse(_)) => {
            bump(&SPINE_DECLINED_PARSE);
            SpineOutcome::Declined
        }
        Err(SpineAttemptError::Lower(crate::spine::plan::LowerError::MissingCapability {
            ..
        })) => {
            bump(&SPINE_DECLINED_CAPABILITY);
            SpineOutcome::Declined
        }
        Err(SpineAttemptError::Lower(crate::spine::plan::LowerError::UnsupportedConstruct {
            ..
        })) => {
            bump(&SPINE_DECLINED_CONSTRUCT);
            SpineOutcome::Declined
        }
        // ⚠️ NOT a decline. The spine ACCEPTED ownership and produced a spine-owned error, so it
        // counts as claimed -- a defect to investigate, not a fallback to celebrate.
        Err(e) => {
            bump(&SPINE_CLAIMED);
            SpineOutcome::Executed(CommandResult::Error(format!("spine: {e:?}"), 1))
        }
    }
}

/// INT-191: catastrophic `rm -rf` protection, evaluated over the EXECUTION CONTEXT.
///
/// ⚠️ THE SIGNATURE IS THE POINT. A `(cmd: &str, expanded: &str)` helper would be purer and would
/// still be wrong, because the bug this guards against was never inside the policy -- it was a
/// CALLER handing over the wrong lifecycle fact. Taking `&ExecContext` puts the boundary choice
/// here, stated once, where no call site can pass provenance in place of execution text.
///
///   ctx.cmd      -- execution identity, derived from the expanded form
///   ctx.expanded -- execution text, what will actually run
///   ctx.raw      -- provenance, deliberately IGNORED here
///
/// The failure mode: an alias `nuke = rm -rf /home` yields cmd `rm` from the expanded form while
/// `raw` is only `nuke`, so a scan of `raw` finds no -rf and blocks nothing. That state existed
/// briefly during the endpoint split, compiled cleanly, and was caught by audit rather than by a
/// test -- which is why it has one now.
fn blocks_catastrophic_rm(ctx: &ExecContext) -> Option<String> {
    if ctx.cmd.to_lowercase() != "rm" {
        return None;
    }
    // INT-169 blocker 2 / INT-196: STRUCTURE, NOT TEXT. This predicate used to read
    // `ctx.expanded`, lowercase it, substring-search for the flags and then `split_whitespace()`
    // the result -- taking an argument vector, flattening it, and re-parsing it. Two consequences,
    // both real:
    //
    //   1. It did not fire AT ALL where `expanded` was unavailable. A command substitution builds
    //      its context from a plan with no source line, so `"".contains("-rf")` was false and the
    //      most dangerous policy in the shell silently passed everything inside `$()`.
    //   2. A SUBSTRING SEARCH ONLY EVER CAUGHT ADJACENT FLAGS. `rm -r -f /home` was never blocked,
    //      because the text contains neither "-rf" nor "-fr". So was `rm -rR -f /`.
    //
    // Reading `args` fixes both and makes the rule independent of whitespace, quoting, ordering,
    // and any future textual form of `expanded`.
    let mut recursive = false;
    let mut force = false;
    for arg in &ctx.args {
        // A flag-shaped argument only. `--` ends option parsing, and a bare `-` is stdin.
        if !arg.starts_with('-') || arg == "--" || arg == "-" {
            continue;
        }
        // ⚠️ SHORT options are a BUNDLE OF LETTERS; LONG options are a NAME. Scanning letters in
        // a long option makes its spelling matter: `--verbose` contains an `r`, so
        // `rm --verbose --force /home` would block a command that requested nothing recursive.
        // A false positive in a guard is its own damage -- it teaches people to distrust the block.
        if let Some(name) = arg.strip_prefix("--") {
            let name = name.split('=').next().unwrap_or(name);
            recursive |= name == "recursive";
            force |= name == "force";
        } else {
            let letters = arg.trim_start_matches('-');
            recursive |= letters.contains('r') || letters.contains('R');
            force |= letters.contains('f');
        }
    }
    // Both, gathered ACROSS arguments -- `-rf`, `-fr`, `-rRfF` and `-r -f` are the same command.
    if !recursive || !force {
        return None;
    }
    // Absolute block -- these targets are never safe
    let blocked_targets = ["/", "/home", "/etc", "/usr", "/var", "/boot"];
    for target in &blocked_targets {
        // An ARGUMENT is the target, not a token in a rebuilt string: a path containing a space
        // survives here where the old split could not represent it at all.
        let matches = ctx
            .args
            .iter()
            .any(|a| a == *target || a.strip_suffix('/') == Some(*target));
        if matches {
            return Some(format!(
                "🛡  Blocked: rm -rf on protected path '{}' — this cannot be undone",
                target
            ));
        }
    }
    None
}

#[cfg(test)]
mod preexec_boundary_tests {
    //! ⚠️ ONLY ASSERT BLOCKING HERE. `preexec` is NOT a pure predicate: Safety Rule 4 walks the
    //! target directory, prints a summary and PROMPTS for confirmation. A blocking case returns
    //! before reaching it; an allow case falls through and HANGS waiting for a keystroke. Negative
    //! cases belong on the pure predicates instead -- see catastrophic_rm_tests.
    use super::{preexec, ExecContext};
    use std::path::PathBuf;
    use std::time::SystemTime;

    /// A context where the TYPED form is harmless and the EXECUTED form is not -- the shape an
    /// alias produces. Every policy inside preexec must judge on `expanded`; any that reads `raw`
    /// sees only the harmless name and waves the command through.
    /// ⚠️ Takes ARGV, like `catastrophic_rm_tests::ctx` and for the reason that module records.
    /// This helper previously set `args: Vec::new()` while putting arguments in `expanded`, and
    /// when the catastrophic-rm policy became structural THAT MADE THIS MODULE HANG rather than
    /// fail: the command stopped being blocked, fell through to the confirmation prompt Safety
    /// Rule 4 raises, and waited forever. The module doc above predicted exactly that.
    fn aliased(raw: &str, argv: &[&str]) -> ExecContext {
        ExecContext {
            raw: raw.to_string(),
            expanded: argv.join(" "),
            cmd: argv.first().copied().unwrap_or_default().to_string(),
            args: argv.iter().skip(1).map(|s| s.to_string()).collect(),
            cwd: PathBuf::from("."),
            intent: None,
            timestamp: SystemTime::now(),
            execution_id: 1,
            session_id: "test",
            in_pipeline: false,
        }
    }

    /// Catastrophic target policy, reached through preexec rather than the predicate directly --
    /// this proves the WIRING, not just the rule.
    #[test]
    fn blocks_aliased_catastrophic_rm() {
        let ctx = aliased("nuke", &["rm", "-rf", "/home"]);
        assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_some());
    }

    /// Forest-source protection. Still inline in preexec and therefore untested until now: a
    /// future edit could repoint it at `ctx.raw` and nothing would object.
    #[test]
    fn blocks_aliased_rm_on_forest_source() {
        let intents = faelight_core::paths::intents_dir()
            .to_string_lossy()
            .to_string();
        let ctx = aliased("cleanup", &["rm", "-rf", intents.as_str()]);
        assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_some());
    }

    /// Self-overwrite protection for cp/mv, same exposure.
    #[test]
    fn blocks_aliased_copy_over_core_binary() {
        let ctx = aliased(
            "install",
            &["cp", "/tmp/thing", "/home/christian/.cargo/bin/core"],
        );
        assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_some());
    }

    /// The negative direction matters as much: a harmless execution must not be blocked just
    /// because the typed form looked alarming.
    #[test]
    fn allows_harmless_execution_with_alarming_typed_form() {
        let ctx = aliased("rm -rf /home", &["echo", "pretend"]);
        assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_none());
    }
}
#[cfg(test)]
mod catastrophic_rm_tests {
    use super::{blocks_catastrophic_rm, ExecContext};
    use std::path::PathBuf;
    use std::time::SystemTime;

    /// Builds a context the way PRODUCTION does: `cmd`, `args` and `expanded` all derive from one
    /// argv, exactly as `from_plan` derives them from `plan.argv`.
    ///
    /// ★ THE HELPER IS ITSELF A REGRESSION LOCK. The previous version took `expanded` and `cmd`
    /// separately and left `args` EMPTY, which is a state no execution path can reach -- a command
    /// word with arguments in its display text but none in its argument vector. That was harmless
    /// while the predicate only read text, and became dangerous the moment it read structure.
    /// Impossible states in tests are where false confidence comes from.
    ///
    /// `raw` stays separate on purpose: it is PROVENANCE, and the whole point of several of these
    /// tests is that it may legitimately disagree with execution identity.
    fn ctx(raw: &str, argv: &[&str]) -> ExecContext {
        ExecContext {
            raw: raw.to_string(),
            expanded: argv.join(" "),
            cmd: argv.first().copied().unwrap_or_default().to_string(),
            args: argv.iter().skip(1).map(|s| s.to_string()).collect(),
            cwd: PathBuf::from("."),
            intent: None,
            timestamp: SystemTime::now(),
            execution_id: 1,
            session_id: "test",
            in_pipeline: false,
        }
    }

    /// THE REGRESSION LOCK. An alias whose expansion is catastrophic must block even though the
    /// typed form is harmless. If a future edit points this policy back at `ctx.raw`, this fails --
    /// without ever launching rm.
    #[test]
    fn uses_execution_boundary_not_typed_boundary() {
        assert!(blocks_catastrophic_rm(&ctx("nuke", &["rm", "-rf", "/home"])).is_some());
    }

    #[test]
    fn blocks_direct_form() {
        assert!(blocks_catastrophic_rm(&ctx("rm -rf /home", &["rm", "-rf", "/home"])).is_some());
    }

    #[test]
    fn blocks_reversed_flag_order() {
        assert!(blocks_catastrophic_rm(&ctx("rm -fr /etc", &["rm", "-fr", "/etc"])).is_some());
    }

    /// The command word gates the policy: text that merely CONTAINS a dangerous string is not a
    /// dangerous command.
    #[test]
    fn allows_unrelated_command_mentioning_rm() {
        let c = ctx("echo rm -rf /home", &["echo", "rm", "-rf", "/home"]);
        assert!(blocks_catastrophic_rm(&c).is_none());
    }

    /// THE TRUE INVERSE OF THE LOCK. cmd is `rm` and the EXECUTED target is harmless, while
    /// the TYPED form names a blocked path. A version reading `raw` finds /home and blocks; the
    /// correct version reads `expanded`, finds only /tmp/scratch, and allows. This fails if the
    /// fields are swapped the other way -- which a cmd-gated negative case cannot detect, because
    /// it never reaches any path scan.
    #[test]
    fn allows_safe_execution_when_typed_form_names_a_blocked_path() {
        let c = ctx("rm -rf /home", &["rm", "-rf", "/tmp/scratch"]);
        assert!(blocks_catastrophic_rm(&c).is_none());
    }

    /// ★ NEWLY BLOCKED, and stated as a decision rather than left as a side effect. The old
    /// predicate substring-searched the flattened text for "-rf" or "-fr", so SEPARATED flags were
    /// never caught -- `rm -r -f /home` ran. Reading the argument vector gathers the letters across
    /// arguments, so the same command is the same command however it was written.
    /// ★ A FALSE POSITIVE IS DAMAGE TOO. A long option is a NAME, not a bundle of letters:
    /// `--verbose` contains an `r` and would otherwise pair with `--force` to block a command that
    /// asked for nothing recursive. A guard that fires on safe commands teaches people to ignore it.
    #[test]
    fn long_option_spelling_does_not_imply_recursion() {
        let c = ctx(
            "rm --verbose --force /home",
            &["rm", "--verbose", "--force", "/home"],
        );
        assert!(blocks_catastrophic_rm(&c).is_none());
        // But the real long forms still block.
        let c = ctx(
            "rm --recursive --force /home",
            &["rm", "--recursive", "--force", "/home"],
        );
        assert!(blocks_catastrophic_rm(&c).is_some());
    }

    #[test]
    fn blocks_flags_written_separately() {
        assert!(
            blocks_catastrophic_rm(&ctx("rm -r -f /home", &["rm", "-r", "-f", "/home"])).is_some()
        );
        assert!(
            blocks_catastrophic_rm(&ctx("rm -f -r /etc", &["rm", "-f", "-r", "/etc"])).is_some()
        );
        assert!(blocks_catastrophic_rm(&ctx("rm -rR -f /", &["rm", "-rR", "-f", "/"])).is_some());
    }

    /// Short bundles and long names contribute to the SAME decision, so a command written half
    /// each way is still the same command.
    #[test]
    fn mixed_short_and_long_flags_are_recognised() {
        let c = ctx("rm -r --force /home", &["rm", "-r", "--force", "/home"]);
        assert!(blocks_catastrophic_rm(&c).is_some());
        let c = ctx(
            "rm --recursive -f /etc",
            &["rm", "--recursive", "-f", "/etc"],
        );
        assert!(blocks_catastrophic_rm(&c).is_some());
    }

    /// A target containing a space is ONE argument. The old predicate rebuilt a string and split it
    /// on whitespace, so such a path could not be represented at all -- structure can.
    #[test]
    fn a_target_with_a_space_is_still_one_argument() {
        let c = ctx("rm -rf x", &["rm", "-rf", "/home/my files"]);
        assert!(
            blocks_catastrophic_rm(&c).is_none(),
            "not a protected target"
        );
        let c = ctx("rm -rf x", &["rm", "-rf", "/home"]);
        assert!(blocks_catastrophic_rm(&c).is_some());
    }

    #[test]
    fn allows_rm_without_recursive_force() {
        assert!(blocks_catastrophic_rm(&ctx("rm file.txt", &["rm", "file.txt"])).is_none());
    }

    #[test]
    fn allows_recursive_force_on_unprotected_path() {
        let c = ctx("rm -rf /tmp/scratch", &["rm", "-rf", "/tmp/scratch"]);
        assert!(blocks_catastrophic_rm(&c).is_none());
    }
}

/// INT-191: the SQLite boundary for a wall-clock instant. `unwrap_or_default` rather than a panic,
/// for the same reason as elsewhere in this file: persistence must not die because the clock
/// misbehaved.
fn unix_seconds(t: SystemTime) -> i64 {
    t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

/// INT-191: what an execution produced, plus WHICH execution produced it.
///
/// The identity has to leave this function because the exit code does not exist yet when it
/// returns -- main.rs decides the final code in the pipeline arms afterwards. Returning it beats
/// hoisting generation to the caller, because `from_line` is also used by the `spine migrate`
/// audit, and that path would then have to supply an execution id for something that never
/// executes.
pub struct ExecutionOutcome {
    pub execution_id: u64,
    pub result: CommandResult,
}

/// INT-191: the ONE mapping from a `CommandResult` to a lifecycle state.
///
/// ⚠️ Four copies of this existed -- postexec's own match plus one at each caller -- which is the
/// shape this intent exists to remove: several owners of one meaning, free to drift. Lifecycle
/// state answers "what KIND of outcome occurred". The exit code answers "what status should history
/// show", a different question that stays with the caller that knows it.
pub fn execution_state(result: &CommandResult) -> &'static str {
    match result {
        // INT-201: the code is deliberately ignored here -- see this function's doc. Lifecycle
        // state answers what KIND of outcome occurred; the status is a different question.
        CommandResult::Exit(_) => crate::db::EXEC_EXIT,
        CommandResult::Error(_, _) => crate::db::EXEC_ERROR,
        CommandResult::Empty => crate::db::EXEC_EMPTY,
        _ => crate::db::EXEC_OK,
    }
}

pub fn execute_with_context(
    raw: &str,
    expanded: &str,
    db: &ForestDb,
    core_root: &str,
    rules: &[BeforeRunRule],
) -> ExecutionOutcome {
    // INT-191: this signature is where the lifecycle used to collapse. One `line` fed both the
    // context and the dispatcher, so `ExecContext.raw` -- documented as "exactly what the user
    // typed" -- received a value that had already crossed the execution boundary. postexec was
    // never wrong; it was faithfully recording what it was handed.
    let ctx = ExecContext::from_line(raw, expanded, db);
    // INT-191: the lifecycle record opens HERE, before anything can return. postexec cannot own it:
    // it never runs for a blocked command, and it deliberately skips `exit`. Those are precisely
    // the events worth having, so recording only after an outcome would lose them.
    let started_at = unix_seconds(ctx.timestamp);
    if let Err(e) = db.begin_command_execution(&crate::db::ExecutionStart {
        session_id: ctx.session_id,
        execution_id: ctx.execution_id,
        typed_text: &ctx.raw,
        cwd: &ctx.cwd.to_string_lossy(),
        intent_id: ctx.intent.as_deref(),
        started_at,
    }) {
        eprintln!("warning: failed to open command_execution record: {e}");
    }

    // Preexec — can block execution
    if let Some(block_reason) = preexec(&ctx, core_root, rules) {
        // A block is a lifecycle OUTCOME, not an absence: no executed text because the command
        // never reached expansion, no exit code because no process ran.
        if let Err(e) = db.complete_command_execution(&crate::db::ExecutionCompletion {
            session_id: ctx.session_id,
            execution_id: ctx.execution_id,
            executed_text: None,
            state: crate::db::EXEC_BLOCKED,
            exit_code: None,
            duration_ms: None,
            finished_at: unix_seconds(SystemTime::now()),
        }) {
            eprintln!("warning: failed to close blocked command_execution record: {e}");
        }
        return ExecutionOutcome {
            execution_id: ctx.execution_id,
            result: CommandResult::Error(block_reason, 1),
        };
    }

    // Dispatch — call existing execute() (unchanged)
    let result = commands::execute(expanded, db, core_root);

    // Postexec — observe the result
    postexec(&ctx, &result, db);

    ExecutionOutcome {
        execution_id: ctx.execution_id,
        result,
    }
}

#[cfg(test)]
mod execution_id_tests {
    use super::{next_execution_id, session_id};

    /// INT-169 blocker 8: one execution gets one id. If two collide, every consumer
    /// downstream is observing a different execution than it believes it is.
    #[test]
    fn execution_ids_are_unique_and_increasing() {
        let a = next_execution_id();
        let b = next_execution_id();
        assert_ne!(a, b, "two executions shared an id");
        assert!(b > a, "execution ids must increase: {a} then {b}");
    }

    /// INT-191: the primary key of `command_execution` is (session_id, execution_id), so the
    /// session half must be STABLE for the life of the process. If it were regenerated per call,
    /// two executions in one shell would land in different namespaces and the pair would stop
    /// identifying anything.
    #[test]
    fn session_id_is_stable_within_the_process() {
        assert_eq!(session_id(), session_id());
        assert!(!session_id().is_empty());
        assert!(session_id().contains('-'), "expected pid-nanos shape");
    }
}
