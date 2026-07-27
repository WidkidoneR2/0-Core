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
                    CommandResult::Error(e) => e.clone(),
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
        return CommandResult::Error(block_reason);
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
struct SpineCommandRunner<'a> {
    db: &'a ForestDb,
    core_root: &'a str,
}

impl crate::spine::plan::CommandRunner for SpineCommandRunner<'_> {
    fn run_capture(&self, plan: &crate::spine::plan::ExecutionPlan) -> Result<String, String> {
        // Matched exhaustively, and each arm for a stated reason -- no catch-all, because a
        // future variant should be a compile error here rather than a generic message.
        match commands::execute_plan_dispatch(plan, "", self.db, self.core_root) {
            CommandResult::Output(s) => Ok(s),
            // Produced nothing, so substituted nothing. Correct, not an error.
            CommandResult::Empty => Ok(String::new()),
            CommandResult::Error(e) => Err(e),
            // Stringifying a structured Value here would make this adapter invent display
            // semantics for a layer it does not own. What `$(tt)` should mean is a Lane 5
            // question about structured pipelines, not something to settle by accident.
            CommandResult::Value(_) => {
                Err("nested command produced a structured value, not text".to_string())
            }
            // A substitution that tries to terminate the shell is a FAILED capture, not an empty
            // one -- swallowing it as `Ok("")` would make `$(exit)` silently expand to nothing.
            CommandResult::Exit => Err("nested command attempted to exit the shell".to_string()),
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
pub fn execute_spine_source(
    source: &str,
    shell: &ShellContext,
    db: &ForestDb,
    core_root: &str,
    rules: &[BeforeRunRule],
) -> CommandResult {
    let node = match crate::spine::parser::parse(source) {
        Ok(n) => n,
        Err(e) => return CommandResult::Error(format!("spine: parse error: {e:?}")),
    };
    // INT-169 blocker 4: this door DOES get a runner, and the earlier `None` here was a
    // mis-application of the scorecard's ruling. That ruling says `spine exec` is correct to sit
    // ABOVE alias expansion and refuse aliases -- a statement about the INPUT PHASE, not about
    // which constructs the spine may execute. Command substitution is now one the spine supports,
    // and refusing it here would leave the capability with no consumer at all.
    let runner = SpineCommandRunner { db, core_root };
    let ctx = crate::spine::plan::LowerContext {
        vars: Some(shell),
        runner: Some(&runner),
    };
    let plan = match crate::spine::plan::lower(&node, &ctx) {
        Ok(p) => p,
        Err(e) => return CommandResult::Error(format!("spine: cannot lower yet: {e:?}")),
    };
    execute_spine(&plan, source, db, core_root, rules)
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
    let expanded = ctx.expanded.as_str();
    let lower = expanded.to_lowercase();
    if !lower.contains("-rf") && !lower.contains("-fr") {
        return None;
    }
    // Absolute block -- these targets are never safe
    let blocked_targets = ["/", "/home", "/etc", "/usr", "/var", "/boot"];
    for target in &blocked_targets {
        // Match exact path -- must be followed by space, end of string, or be standalone
        let matches = expanded
            .split_whitespace()
            .any(|token| token == *target || token == &format!("{}/", target));
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
    fn aliased(raw: &str, expanded: &str, cmd: &str) -> ExecContext {
        ExecContext {
            raw: raw.to_string(),
            expanded: expanded.to_string(),
            cmd: cmd.to_string(),
            args: Vec::new(),
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
        let ctx = aliased("nuke", "rm -rf /home", "rm");
        assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_some());
    }

    /// Forest-source protection. Still inline in preexec and therefore untested until now: a
    /// future edit could repoint it at `ctx.raw` and nothing would object.
    #[test]
    fn blocks_aliased_rm_on_forest_source() {
        let intents = faelight_core::paths::intents_dir()
            .to_string_lossy()
            .to_string();
        let ctx = aliased("cleanup", &format!("rm -rf {}", intents), "rm");
        assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_some());
    }

    /// Self-overwrite protection for cp/mv, same exposure.
    #[test]
    fn blocks_aliased_copy_over_core_binary() {
        let ctx = aliased(
            "install",
            "cp /tmp/thing /home/christian/.cargo/bin/core",
            "cp",
        );
        assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_some());
    }

    /// The negative direction matters as much: a harmless execution must not be blocked just
    /// because the typed form looked alarming.
    #[test]
    fn allows_harmless_execution_with_alarming_typed_form() {
        let ctx = aliased("rm -rf /home", "echo pretend", "echo");
        assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_none());
    }
}
#[cfg(test)]
mod catastrophic_rm_tests {
    use super::{blocks_catastrophic_rm, ExecContext};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn ctx(raw: &str, expanded: &str, cmd: &str) -> ExecContext {
        ExecContext {
            raw: raw.to_string(),
            expanded: expanded.to_string(),
            cmd: cmd.to_string(),
            args: Vec::new(),
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
        assert!(blocks_catastrophic_rm(&ctx("nuke", "rm -rf /home", "rm")).is_some());
    }

    #[test]
    fn blocks_direct_form() {
        assert!(blocks_catastrophic_rm(&ctx("rm -rf /home", "rm -rf /home", "rm")).is_some());
    }

    #[test]
    fn blocks_reversed_flag_order() {
        assert!(blocks_catastrophic_rm(&ctx("rm -fr /etc", "rm -fr /etc", "rm")).is_some());
    }

    /// The command word gates the policy: text that merely CONTAINS a dangerous string is not a
    /// dangerous command.
    #[test]
    fn allows_unrelated_command_mentioning_rm() {
        let c = ctx("echo rm -rf /home", "echo rm -rf /home", "echo");
        assert!(blocks_catastrophic_rm(&c).is_none());
    }

    /// THE TRUE INVERSE OF THE LOCK. cmd is `rm` and the EXECUTED target is harmless, while
    /// the TYPED form names a blocked path. A version reading `raw` finds /home and blocks; the
    /// correct version reads `expanded`, finds only /tmp/scratch, and allows. This fails if the
    /// fields are swapped the other way -- which a cmd-gated negative case cannot detect, because
    /// it never reaches any path scan.
    #[test]
    fn allows_safe_execution_when_typed_form_names_a_blocked_path() {
        let c = ctx("rm -rf /home", "rm -rf /tmp/scratch", "rm");
        assert!(blocks_catastrophic_rm(&c).is_none());
    }

    #[test]
    fn allows_rm_without_recursive_force() {
        assert!(blocks_catastrophic_rm(&ctx("rm file.txt", "rm file.txt", "rm")).is_none());
    }

    #[test]
    fn allows_recursive_force_on_unprotected_path() {
        let c = ctx("rm -rf /tmp/scratch", "rm -rf /tmp/scratch", "rm");
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
        CommandResult::Exit => crate::db::EXEC_EXIT,
        CommandResult::Error(_) => crate::db::EXEC_ERROR,
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
            result: CommandResult::Error(block_reason),
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
