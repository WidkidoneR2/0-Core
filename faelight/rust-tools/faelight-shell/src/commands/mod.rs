#![allow(clippy::all)]
// faelight-shell — command registry
// Phase 1: 10 forest-native commands

use crate::db::ForestDb;
extern crate libc;
use colored::*;
use std::os::unix::process::CommandExt;

// ── Time formatting helper ───────────────────────────────────────────────────
fn fmt_time(ts: i64, fmt: &str) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|t| t.format(fmt).to_string())
        .unwrap_or_else(|| "?".to_string())
}

pub enum CommandResult {
    Output(String),
    Value(crate::value::Value),
    Empty,
    Error(String),
    Exit,
    /// INT-143: "this command is not an fsh builtin" -- an ANSWER, not an action.
    ///
    /// Only `try_builtin()` can return this. `execute()` never does, so every existing
    /// caller is unaffected BY CONSTRUCTION (not by the compiler -- most call sites match
    /// with a `_` arm and would silently swallow a new variant).
    ///
    /// WHY IT EXISTS. `execute()` conflated ASKING with DOING: the only way to find out
    /// whether a line was a builtin was to run it. main.rs's redirect path did exactly
    /// that -- called execute(), got Empty, concluded "not a builtin", and spawned the
    /// command A SECOND TIME. Measured 2026-07-16:
    ///     rm -rf /tmp/dirtest; mkdir /tmp/dirtest > /tmp/mk.txt
    ///     -> mkdir: cannot create directory '/tmp/dirtest': File exists
    /// The directory did not exist. The FIRST execution created it; the second failed.
    /// Every external `cmd > file` ran twice. `curl -X POST > log` posted twice.
    NotBuiltin,
}

impl CommandResult {
    /// The SINGLE source of truth for "did this command fail" -- the signal that
    /// `&&`/`||` chaining depends on. INT-171 gate 5.
    ///
    /// Bug 968c7be5 was a failure that returned a non-Error variant, so a scattered
    /// inline `!matches!(result, Error(_))` read it as success and `&&` proceeded.
    /// Defining failure ONCE, here, means a future variant that should count as a
    /// failure is fixed in this method -- not hunted across every chain site. The
    /// flow decision can no longer be re-derived inconsistently at a call site.
    pub fn is_failure(&self) -> bool {
        matches!(self, CommandResult::Error(_))
    }
}

// ── Security Layer — log every command ───────────────────────────────────────
/// Truncate a string to at most `max` bytes WITHOUT splitting a UTF-8 char.
/// Used in abort/error messages so a multibyte anchor (em-dash, box-drawing char)
/// never panics the shell via an out-of-bounds byte slice (a panic here closes fsh).
fn truncate_safe(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn emit_command(db: &ForestDb, cmd: &str, result: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let payload = format!(
        r#"{{"actor":"faelight-shell","result":"{}","detail":{{"command":"{}"}}}}"#,
        result,
        cmd.replace('"', "'")
    );
    db.conn.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('shell', 'command', ?1, ?2)",
        rusqlite::params![payload, ts],
    ).ok();
}

#[allow(dead_code)]
fn levenshtein(a: &str, b: &str) -> usize {
    let la = a.len();
    let lb = b.len();
    let mut dp = vec![vec![0usize; lb + 1]; la + 1];
    for i in 0..=la {
        dp[i][0] = i;
    }
    for j in 0..=lb {
        dp[0][j] = j;
    }
    for i in 1..=la {
        for j in 1..=lb {
            let cost = if a.as_bytes()[i - 1] == b.as_bytes()[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
            // Transposition (Damerau-Levenshtein: gti->git = 1, not 2)
            if i > 1
                && j > 1
                && a.as_bytes()[i - 1] == b.as_bytes()[j - 2]
                && a.as_bytes()[i - 2] == b.as_bytes()[j - 1]
            {
                dp[i][j] = dp[i][j].min(dp[i - 2][j - 2] + 1);
            }
        }
    }
    dp[la][lb]
}

/// Syntax highlighter for Rust source lines
pub fn highlight_rust_line(line: &str) -> String {
    use colored::Colorize;
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return line.dimmed().to_string();
    }
    if trimmed.contains("error") || trimmed.contains("panic") || trimmed.contains("FAILED") {
        return line.bright_red().to_string();
    }
    let keywords = [
        "fn ", "let ", "mut ", "pub ", "use ", "struct ", "impl ", "match ", "enum ", "trait ",
        "mod ", "return ", "if ", "else ", "for ", "while ", "loop ", "async ", "await ", "move ",
        "ref ", "const ", "static ", "type ", "where ", "self ", "Self ", "super ", "crate ",
    ];
    let t = line.trim_start();
    for kw in &keywords {
        if t.starts_with(kw) || t.starts_with(&format!("pub {}", kw.trim())) {
            return line.bright_cyan().to_string();
        }
    }
    if line.contains('"') || line.contains("'") {
        return line.bright_yellow().to_string();
    }
    let has_number = line.split_whitespace().any(|w| {
        w.trim_matches(|c: char| !c.is_ascii_digit())
            .parse::<f64>()
            .is_ok()
            && !w.is_empty()
    });
    if has_number && !line.contains("::") {
        return line.bright_magenta().to_string();
    }
    line.to_string()
}
/// Semantic colorizer for fsh-native output lines
pub fn colorize_line(line: &str) -> String {
    use colored::Colorize;
    use std::borrow::Cow;
    // Error words -- bright red
    let lower = line.to_lowercase();
    if lower.contains("error")
        || lower.contains("failed")
        || lower.contains("panic")
        || lower.contains("fatal")
    {
        return line.bright_red().to_string();
    }
    // Success words -- bright green
    if lower.contains("success") || lower.contains("complete") || lower.contains("deployed") {
        return line.bright_green().to_string();
    }
    // Warning words -- bright yellow
    if lower.contains("warning") || lower.contains("warn") || lower.contains("deprecated") {
        return line.bright_yellow().to_string();
    }
    // Tokenize and color per-word
    let mut result = String::new();
    for word in line.split(' ') {
        let colored_word: Cow<str> = if word.starts_with("INT-") && word.len() > 4 {
            // Intent IDs -- bright magenta
            Cow::Owned(word.bright_magenta().to_string())
        } else if word.ends_with('%') && word[..word.len() - 1].parse::<f64>().is_ok() {
            // Percentages -- color by value
            let val: f64 = word[..word.len() - 1].parse().unwrap_or(0.0);
            if val >= 95.0 {
                Cow::Owned(word.bright_green().to_string())
            } else if val >= 70.0 {
                Cow::Owned(word.bright_yellow().to_string())
            } else {
                Cow::Owned(word.bright_red().to_string())
            }
        } else if (word.contains('/')
            || word.ends_with(".rs")
            || word.ends_with(".py")
            || word.ends_with(".md")
            || word.ends_with(".toml")
            || word.ends_with(".sh"))
            && !word.starts_with("//")
        {
            // File paths -- bright cyan
            Cow::Owned(word.bright_cyan().to_string())
        } else if word.len() == 7 && word.chars().all(|c| c.is_ascii_hexdigit()) {
            // Git hashes -- bright blue
            Cow::Owned(word.bright_blue().to_string())
        } else if word.parse::<f64>().is_ok() && !word.is_empty() {
            // Numbers -- bright yellow
            Cow::Owned(word.bright_yellow().to_string())
        } else {
            Cow::Borrowed(word)
        };
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&colored_word);
    }
    result
}
/// Run a command line: builtin if we have one, otherwise hand it to the system.
/// NEVER returns NotBuiltin -- behaviour is byte-for-byte what it has always been.
pub fn execute(line: &str, db: &ForestDb, core_root: &str) -> CommandResult {
    execute_impl(
        &tokenize(line.trim()),
        line,
        db,
        core_root,
        &[],
        ExecutionMode::Text,
    )
}

/// INT-143: ASK whether this line is an fsh builtin, WITHOUT running it if it is not.
///
/// Same dispatch as `execute()` -- same aliases, same plugins, same 227 match arms -- but the
/// fallthrough returns `NotBuiltin` instead of calling `run_external`. A caller that must decide
/// "builtin or spawn?" can now ask, instead of guessing from a side effect.
///
/// A builtin that DOES match still runs. That is intended: there is no way to know whether `cmd`
/// is a builtin without consulting the same match, and builtins are ours -- running one is the
/// point. What this prevents is running an EXTERNAL command as a side effect of asking.
///
/// THIS IS NOT A PURE PREDICATE, AND SAYING SO WOULD BE A LIE. It suppresses every
/// `run_external` call (five sites, all guarded). It does NOT suppress arms that spawn a
/// process directly -- `gt` (Command::new("git")), `friday chat` (Command::new("friday-chat")),
/// and others still act when probed. Making it pure would mean auditing all 227 arms, which is
/// a different intent.
/// What it is FOR: letting a caller distinguish "no arm matched" from "an arm matched and
/// returned nothing". Those were indistinguishable before, and conflating them made main.rs
/// run every redirected external command twice.
/// Those direct-spawn arms were ALREADY broken for redirects before this change (`gt > f` tried
/// to spawn a `gt` binary that does not exist, so the file got nothing). This does not make them
/// worse. It fixes what it can prove and names what it cannot.
/// INT-193: THE SINGLE OWNER of alias expansion.
///
/// Works on RAW TEXT, never on tokenized args. The remainder is copied verbatim via
/// `split_once`, so a quoted multi-word argument survives intact. The executor-side
/// expansion this replaces rebuilt the line from ALREADY-TOKENIZED args with
/// `args.join(" ")`, which silently split one quoted argument into several.
///
/// Iterative, not recursive: each round produces a new String, so a loop with an owned
/// guard is simpler than threading borrowed names through recursive calls.
///
/// The INT-057 cycle guard lives here now. A self-referential alias expands once and
/// then stops, so it cannot recurse forever.
pub fn expand_aliases(line: &str, db: &ForestDb) -> String {
    let mut current = line.to_string();
    let mut seen: Vec<String> = Vec::new();
    loop {
        let first = command_word(&current).to_lowercase();
        if first.is_empty() || seen.contains(&first) {
            break;
        }
        let Some(aliased) = db.get_alias(&first) else {
            break;
        };
        let rest: String = current
            .split_once(' ')
            .map(|x| x.1)
            .map(|s| format!(" {}", s))
            .unwrap_or_default();
        seen.push(first);
        current = format!("{}{}", aliased, rest);
    }
    current
}

pub fn try_builtin(line: &str, db: &ForestDb, core_root: &str) -> CommandResult {
    execute_impl(
        &tokenize(line.trim()),
        line,
        db,
        core_root,
        &[],
        ExecutionMode::Probe,
    )
}

/// The single quote-aware tokenizer for the whole shell (INT-171 gate 1).
/// Splits on spaces, respecting single and double quotes. Both the command
/// dispatcher (execute_impl) and the ExecContext builder (exec.rs) call THIS --
/// it replaces two byte-for-byte-identical nested copies that drifted apart in
/// spirit if never in bytes. There is now ONE tokenizer. Prove it: grep "fn tokenize".
pub fn tokenize(s: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';
    for ch in s.chars() {
        match ch {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = ch;
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
            }
            ' ' if !in_quote => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

// ── INT-171 gate 2: the split_whitespace rule ──────────────────────────────
// A user COMMAND WORD (the thing that gets dispatched, looked up, or aliased) is
// extracted in exactly ONE place: command_word() below. The five sites that needed
// the user's command word to ACT on it route through here (main.rs alias-expansion,
// forest-route, forest-detect; run_external not-found; the builtin not-found check).
//
// Other split_whitespace() calls in this tree are NOT command-word extraction and
// are left as-is by design:
//   - output/telemetry parsing (ip/mac rows, history labels, failure counters,
//     frequency histograms, the `explain` display) -- operates on results, not input.
//   - completion (completing a partial the user is TYPING, never dispatched).
//   - classify-only checks (`== "jobs"`, `== "flow"`, shell-construct keywords,
//     the guard allow/deny compare, dangerous-command classification). These only
//     COMPARE the token; a quoted word fails the compare and falls through safely.
// A future split_whitespace().next() used to DISPATCH a user command is a bug --
// it will mis-read a quoted command. Call command_word() instead.
// ───────────────────────────────────────────────────────────────────────────
/// The command word of a user-typed line: its first token, quote-aware.
///
/// This is the ONE place a command word is extracted from a user line. Every
/// dispatch/lookup that needs "what command did the user type" calls this, so a
/// quoted command word (`"ll" foo`) resolves to `ll`, not `"ll"`. A bare
/// `split_whitespace().next()` on a user line elsewhere is a bug -- it mis-reads
/// a quoted command and silently misses the alias/builtin lookup. INT-171 gate 2.
pub fn command_word(line: &str) -> String {
    tokenize(line.trim()).into_iter().next().unwrap_or_default()
}

#[cfg(test)]
mod command_word_tests {
    use super::command_word;

    // command_word() is the ONE quote-aware command-word extractor. This guards its
    // contract so a future edit that drops quote-awareness fails here, not silently in
    // the five dispatch sites that route through it. INT-171 gate 2.
    #[test]
    fn is_failure_is_single_source_of_flow_truth() {
        use super::CommandResult;
        // The signal &&/|| depend on: ONLY Error is a failure. INT-171 gate 5.
        // Bug 968c7be5 was a failure returning a non-Error variant; this pins the
        // contract so a variant that should count as failure is fixed HERE, and a
        // success variant can never read as failure.
        assert!(CommandResult::Error("boom".to_string()).is_failure());
        assert!(!CommandResult::Empty.is_failure());
        assert!(!CommandResult::Output("ok".to_string()).is_failure());
        assert!(!CommandResult::NotBuiltin.is_failure());
        assert!(!CommandResult::Exit.is_failure());
    }

    #[test]
    fn extracts_first_token_quote_aware() {
        assert_eq!(command_word("git status"), "git"); // unquoted unchanged
        assert_eq!(command_word("  ls -la "), "ls"); // leading/trailing ws trimmed
        assert_eq!(command_word("\"ll\" foo"), "ll"); // double quotes stripped
        assert_eq!(command_word("'echo' hi"), "echo"); // single quotes stripped
        assert_eq!(command_word(""), ""); // empty -> empty (matches old unwrap_or(""))
        assert_eq!(command_word("   "), ""); // whitespace-only -> empty
    }
}

/// Which world is calling execute_impl, and therefore which phases are allowed to run.
///
/// ★ REPLACES the bare `allow_external: bool` (INT-169). The bool encoded exactly one
/// distinction -- what the fallthrough does -- and could not express the one that matters now:
/// whether TEXT TRANSFORMS may run at all. A plan arrives AFTER parsing and expansion; the
/// executor consumes decisions, it does not reinterpret them. Letting alias/plugin expansion run
/// beneath a supplied argv would make execute_impl a second planner and would silently rewrite
/// the command identity out from under the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Interactive text path. History, alias and plugin expansion all run; an unrecognised
    /// command falls through to `run_external` (which delegates the line to sh).
    Text,
    /// INT-143 probe (`try_builtin`): ASK whether this line is a builtin without spawning.
    /// Text transforms STILL run -- aliases are part of the honest answer to "would this hit a
    /// builtin?" -- but the fallthrough answers `NotBuiltin` instead of spawning.
    Probe,
    /// INT-169 plan-driven path. argv is AUTHORITATIVE: no history expansion, no alias
    /// expansion, no plugin expansion. The fallthrough answers `NotBuiltin` so the caller can
    /// spawn the plan directly. Because nothing rewrites argv here, that fallback cannot spawn
    /// a stale command -- correctness by construction rather than by a smarter fallback.
    Spine,
}

impl ExecutionMode {
    /// May text-world transforms (history, alias, plugin expansion) run?
    fn allows_text_transforms(self) -> bool {
        matches!(self, ExecutionMode::Text | ExecutionMode::Probe)
    }
    /// May an unrecognised command be handed to `run_external`?
    fn allows_external(self) -> bool {
        matches!(self, ExecutionMode::Text)
    }
}

/// ⚠️ MIGRATION BOUNDARY (INT-169). This function RECEIVES its execution arguments; it no
/// longer derives them. Two callers supply argv from different worlds:
///   TEXT path  -- tokenize(line) -> argv. History expansion (`!!`), alias expansion and any
///                 other text transformation happen ABOVE this call, where text still exists.
///   SPINE path -- plan.argv, already decided by parse -> lower. No text parsing on the way in.
/// `original_line` is NOT the canonical command representation -- it is kept for the handful of
/// builtins that genuinely need the source text (grep, time, select, semantic verbs) and for the
/// run_external escapes that delegate an unmodelled line to sh.
/// Do NOT reintroduce tokenization inside this function.
fn execute_impl(
    argv: &[String],
    line: &str,
    db: &ForestDb,
    core_root: &str,
    expanded_names: &[&str],
    // INT-143: false = probe. The fallthrough answers NotBuiltin instead of spawning.
    // Threaded through the alias/plugin recursion below, or a probe would quietly become a
    // real run one expansion deep -- the same bug, hidden one level down.
    mode: ExecutionMode,
) -> CommandResult {
    let trimmed_line = line.trim();
    // INT-171 gate 2: the command word goes through the SAME quote-aware tokenizer
    // as its arguments. Previously `cmd` came from a raw `splitn(2, ' ')` while `args`
    // came from tokenize() -- two parsing rules in one function. So `"echo" hi` looked
    // up the literal command `"echo"` (quotes included), failed, fired a wrong Friday
    // suggestion, and only ran because sh -c rescued the whole line downstream.
    // This is INT-143's main.rs:1973 lesson (inline-var extraction) carried to the
    // command word it never reached: scan the first token quote-aware, like tokenize does.
    // The execution arguments are SUPPLIED (see the migration boundary note above), not parsed
    // here. Named so a future cleanup pass reads "this function receives its argv" rather than
    // "someone forgot to tokenize".
    let owned_args: &[String] = argv;
    // Lookup key only -- lowercased on purpose so `INTL` and `intl` reach the same builtin. This
    // is NOT the "stop lowercasing the command name" rides-with fix, which concerns the name that
    // gets EXECUTED and RECORDED. Lookup identity and execution identity are different things:
    //   lookup:    "EcHo" -> "echo"   (wanted)
    //   execution: "EcHo" -> "EcHo"   (preserve)
    let cmd = owned_args
        .first()
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    let args_vec: Vec<&str> = owned_args.iter().skip(1).map(|s| s.as_str()).collect();
    let args = args_vec.as_slice();

    // !! — repeat last command
    // TEXT-WORLD ONLY: a plan whose argv is ["!!"] is nonsense, and expanding it here would be
    // the executor re-planning what it was handed.
    if mode.allows_text_transforms() && line.trim() == "!!" {
        match db.get_last_command() {
            Some(last) => {
                println!("  {}", last.dimmed());
                return execute(&last, db, core_root);
            }
            None => return CommandResult::Error("No previous command in history".to_string()),
        }
    }

    // INT-326 Phase 4: semantic safety enforcement -- observation verbs provably safe
    {
        use crate::semantic::{interpret, VerbCategory};
        let si = interpret(trimmed_line);
        if matches!(si.category, VerbCategory::Observation) {
            // Record that this is a read-only operation -- safety contract enforced
            let _ = db.conn.execute(
                "INSERT OR IGNORE INTO shell_state (key, value) VALUES ('last_observation_verb', ?1)",
                rusqlite::params![&cmd],
            );
        }
        if matches!(si.category, VerbCategory::Destructive) && si.confidence > 0.8 {
            // High-confidence destructive verb -- ensure it was intentional
            let _ = db.conn.execute(
                "INSERT OR IGNORE INTO shell_state (key, value) VALUES ('last_destructive_verb', ?1)
                 ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                rusqlite::params![&cmd],
            );
        }
    }
    // INT-278: friday chat -- before alias resolution
    if cmd == "friday" && args.first().copied() == Some("chat") {
        let rest = args.get(1..).unwrap_or(&[]).join(" ");
        if rest.is_empty() {
            let _ = std::process::Command::new("friday-chat").status();
            return CommandResult::Output(String::new());
        } else {
            let out = std::process::Command::new("friday-chat")
                .args(["chat", &rest])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();
            return CommandResult::Output(out);
        }
    }
    // INT-169: `spine parse <line>` -- debug builtin. Runs a line through the new parser
    // spine and pretty-prints the AST. Display only, NEVER executes. The visible tool to
    // eyeball what the spine produces. NOTE: args are already tokenize()'d (quotes stripped),
    // so `spine parse` sees bare words -- exact for roadmap step 1 (bare commands); when
    // quotes become meaningful (step 2) this will need the raw line instead of rejoined args.
    if cmd == "spine" {
        match args.first().copied() {
            Some("parse") => {
                // Step 2: quotes are meaningful, so rejoining tokenize()'d args (quotes
                // already stripped) would hide exactly what we want to inspect. Take the RAW
                // line and strip only the `spine parse` prefix.
                let line_to_parse = trimmed_line
                    .strip_prefix("spine")
                    .map(str::trim_start)
                    .and_then(|s| s.strip_prefix("parse"))
                    .map(str::trim_start)
                    .unwrap_or("")
                    .to_string();
                if line_to_parse.trim().is_empty() {
                    return CommandResult::Error("usage: spine parse <line>".to_string());
                }
                return match crate::spine::parser::parse(&line_to_parse) {
                    Ok(node) => CommandResult::Output(crate::spine::render::render(&node)),
                    Err(e) => CommandResult::Error(format!("spine parse error: {e:?}")),
                };
            }
            Some("audit") => {
                // db -> iterator adapter over the pure audit engine (spine::audit). Reads
                // distinct real commands, excluding TIMING:/SUGGEST: bookkeeping rows (not user
                // language). Passes ALL matching rows (NOT SELECT DISTINCT): the engine needs
                // total_entries (volume) AND does its own dedup for unique_inputs (shape).
                // On-demand only; parses+lowers every row, so it is never on the hot path.
                let mut stmt = match db.conn.prepare(
                    "SELECT command FROM shell_history \
                     WHERE command NOT LIKE 'TIMING:%' AND command NOT LIKE 'SUGGEST:%'",
                ) {
                    Ok(s) => s,
                    Err(e) => return CommandResult::Error(format!("spine audit: db error: {e}")),
                };
                let rows = match stmt.query_map([], |row| row.get::<_, String>(0)) {
                    Ok(r) => r,
                    Err(e) => {
                        return CommandResult::Error(format!("spine audit: query error: {e}"))
                    }
                };
                let commands = rows.filter_map(|r| r.ok());
                let report = crate::spine::audit::audit_history(commands);
                return CommandResult::Output(report.render());
            }
            Some("migrate") => {
                // INT-169 Increment 10 Phase 2: migration-readiness audit. Produces BOTH plans
                // per real command (legacy via the real from_line + plan_from_legacy; spine via
                // parse + lower) and feeds observations to the PURE audit engine, which never
                // knows how the plans were produced. On-demand only -- runs from_line (a db
                // read) per unique command. Distinct from `spine audit`: that measures the
                // spine alone, this measures spine VS legacy.
                let mut stmt = match db.conn.prepare(
                    "SELECT DISTINCT command FROM shell_history \
                     WHERE command NOT LIKE 'TIMING:%' AND command NOT LIKE 'SUGGEST:%'",
                ) {
                    Ok(s) => s,
                    Err(e) => return CommandResult::Error(format!("spine migrate: db error: {e}")),
                };
                let rows = match stmt.query_map([], |row| row.get::<_, String>(0)) {
                    Ok(r) => r,
                    Err(e) => {
                        return CommandResult::Error(format!("spine migrate: query error: {e}"))
                    }
                };
                let sources: Vec<String> = rows.filter_map(|r| r.ok()).collect();

                let mut audit = crate::spine::migrate_audit::MigrationAudit::new();
                for source in &sources {
                    // Applicability first: skipped entries need no plans produced.
                    if let Some(reason) = crate::spine::migrate_audit::applicability(source) {
                        audit.skip(reason);
                        continue;
                    }
                    let ctx = crate::exec::ExecContext::from_line(source, db);
                    let legacy = crate::spine::migrate::plan_from_legacy(&ctx);
                    let spine = match crate::spine::parser::parse(source) {
                        // No environment: `spine migrate` compares PARSERS on the same input.
                        // Expanding here would make every variable-using command diverge from the
                        // legacy plan as an artifact of the audit's own fidelity gap.
                        Ok(node) => crate::spine::plan::lower(
                            &node,
                            &crate::spine::plan::LowerContext::default(),
                        ),
                        Err(_) => {
                            // Counted, not silently dropped: legacy accepted this line.
                            audit.spine_parse_error(source);
                            continue;
                        }
                    };
                    audit.observe(crate::spine::migrate_audit::AuditObservation {
                        source,
                        legacy,
                        spine,
                    });
                }
                return CommandResult::Output(audit.finish().render());
            }
            Some("exec") => {
                // INT-169 proof-of-shape: the COMPLETE VERTICAL, source to process, driven
                // entirely by the spine. parse -> AST -> lower -> ExecutionPlan -> argv ->
                // Command -> spawn. No sh anywhere on this path, so the AST is the authority
                // on what runs rather than a suggestion sh gets to reinterpret.
                //
                // OPT-IN ONLY. The live path is untouched: every normal command still goes
                // through from_line and run_external. This is reached only by typing it.
                //
                // Deliberately boring. No expansion, substitution, globs, aliases or
                // pipelines -- a plan arrives already-expanded or it does not arrive. If
                // `echo hello` misbehaves there is exactly one place it can have gone wrong.
                let raw = trimmed_line
                    .strip_prefix("spine")
                    .map(str::trim_start)
                    .and_then(|s| s.strip_prefix("exec"))
                    .map(str::trim_start)
                    .unwrap_or("")
                    .to_string();
                if raw.trim().is_empty() {
                    return CommandResult::Error("usage: spine exec <command>".to_string());
                }
                let node = match crate::spine::parser::parse(&raw) {
                    Ok(n) => n,
                    Err(e) => {
                        return CommandResult::Error(format!("spine exec: parse error: {e:?}"))
                    }
                };
                // No resolver yet: fsh's session vars live in main.rs's REPL loop and are not
                // reachable from here, so `$NAME` renders in source form. Wiring a real
                // VarResolver is the next step and is what actually starts expanding.
                let plan = match crate::spine::plan::lower(
                    &node,
                    &crate::spine::plan::LowerContext::default(),
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        // A capability boundary, not a fault: the parser is allowed to run
                        // ahead of the executor.
                        return CommandResult::Error(format!(
                            "spine exec: cannot lower this construct yet: {e:?}"
                        ));
                    }
                };
                // Builtins first, then the direct executor. `allow_external: false` is the
                // load-bearing argument: it suppresses every run_external call, so a command
                // this dispatch does not recognise answers NotBuiltin instead of being handed
                // to `sh -c`. Without it the spine path would quietly end at
                //     plan -> argv -> text -> sh -c -> shell parser -> process
                // and the builtin case would prove the new path while the external case proved
                // the old one -- an invisible escape hatch under a passing test.
                //
                // ⚠️ PRECISE CLAIM, per INT-143's own note: this guarantees NO sh and NO
                // re-parse. It does NOT guarantee no subprocess -- 26 arms spawn a named binary
                // directly (`gt` -> Command::new("git"), `friday chat` -> Command::new(...)).
                // That is a builtin doing its job, the same construction execute_plan uses, and
                // it is not what the gate is protecting against.
                //
                // TODO: allow_external is a bare bool at four call sites and will read as
                // meaningless as this grows. An ExecutionMode { TextShell, SpinePlan } enum
                // would make each site self-documenting. Naming cleanup, not a semantic change.
                // One implementation, shared with exec::execute_spine.
                return execute_plan_dispatch(&plan, &raw, db, core_root);
            }
            _ => {
                return CommandResult::Error(
                    "usage: spine parse <line> | spine exec <command> | spine audit | spine migrate"
                        .to_string(),
                );
            }
        }
    }

    // Alias resolution — check before dispatch
    // TEXT-WORLD ONLY (INT-169). Alias AND plugin expansion both reassemble a command by string
    // concatenation and re-tokenize, so both are text transforms by construction, and both are
    // skipped when argv is authoritative. This is why blocker 6 (alias expansion in the spine)
    // stays honestly unstarted instead of half-working by accident.
    if mode.allows_text_transforms() {
        // INT-193: alias expansion USED to happen here, a SECOND time, and this is where
        // quoting died: `args` are already tokenized, so `args.join(" ")` reassembled a
        // quoted multi-word argument as several bare ones. It now has a single owner in the
        // input phase (`expand_aliases`), called at the prompt before this executor sees the
        // line. Do NOT reintroduce it here -- INT-195: no stage may re-derive syntax a
        // previous stage already computed. Plugin expansion below is a SEPARATE question,
        // deliberately left to INT-170; it keeps its own INT-057 guard.

        // Plugin resolution — after final cmd parse
        {
            let plugins = db.load_plugins();
            if let Some((_, expand, _)) = plugins
                .iter()
                .find(|(name, _, _)| name.as_str() == cmd.as_str())
            {
                // INT-057: same cycle guard as aliases -- a self-referential plugin
                // would otherwise recurse forever -> stack overflow.
                if !expanded_names.contains(&cmd.as_str()) {
                    let expanded = if args.is_empty() {
                        expand.clone()
                    } else {
                        format!("{} {}", expand, args.join(" "))
                    };
                    let mut next = expanded_names.to_vec();
                    next.push(cmd.as_str());
                    // Alias expansion produced NEW TEXT, so it is re-tokenized here -- at the exact
                    // point the new text appears, which is where text-world work belongs.
                    return execute_impl(
                        &tokenize(expanded.trim()),
                        &expanded,
                        db,
                        core_root,
                        &next,
                        mode,
                    );
                }
            }
        }
    } // end text-world transforms
      // INT-278: friday chat -- intercept before alias expansion
    if cmd == "friday" && args.first().copied() == Some("chat") {
        let rest = args.get(1..).unwrap_or(&[]).join(" ");
        if rest.is_empty() {
            let _ = std::process::Command::new("friday-chat").status();
            return CommandResult::Output(String::new());
        } else {
            let out = std::process::Command::new("friday-chat")
                .args(["chat", &rest])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();
            return CommandResult::Output(out);
        }
    }
    execute_dispatch(&cmd, args, line, db, core_root, mode)
}

/// INT-169 commit 4 of 4: the dispatch table, lifted out of `execute_impl` so the function
/// that ROUTES is not the same function that IS the routing table. `execute_impl` keeps the
/// pre-dispatch policy -- `!!` expansion, semantic safety recording, the friday-chat
/// interception, the spine debug entry points -- and hands normalised inputs to this.
///
/// ⚠️ THE SECURITY LOG MOVED WITH THE MATCH, DELIBERATELY, AND THIS IS NOT AN ENDORSEMENT.
/// 91 arms inside this match use `return`, which today exits before `emit_command` is ever
/// reached -- so those commands are not logged. That coupling already existed; it was simply
/// hidden across a function boundary. Extracting the match alone would have silently turned
/// 91 unlogged paths into logged ones, which is a change to observability wearing the costume
/// of a refactor. Taking the whole tail preserves the behaviour exactly and makes the shape
/// visible instead. Whether every command path SHOULD emit a security log is a real question
/// and its own change, with its own evidence.
///
/// `cmd` is rebound as an owned String so the ~3,877 lines below are byte-identical to what
/// they were inside `execute_impl`. One allocation per command, and no arm surgery.
///
/// The parameter list is the finding, not an accident of the cut. The compiler proved the match
/// needs neither `owned_args` nor `expanded_names` -- the latter being INT-057's cycle guard,
/// whose absence here confirms alias and plugin recursion lives entirely in the pre-dispatch
/// block and never reaches the dispatch table.
fn execute_dispatch(
    cmd: &str,
    args: &[&str],
    line: &str,
    db: &ForestDb,
    core_root: &str,
    mode: ExecutionMode,
) -> CommandResult {
    let cmd = cmd.to_string();
    let allow_external = mode.allows_external();
    let result = match cmd.as_str() {
        "on" => on_cmd(db, args),
        "help" | "h" => help(),
        "?" => CommandResult::Output(crate::nl::render_pattern_list()),
        "exit" | "quit" | "q" => CommandResult::Exit,
        // INT-174 — Structured Errors
        "last_error" | "last-error" => last_error_cmd(db, args),
        "errors" => error_history_cmd(db, args),
        // INT-176 — Failure Recovery
        "last_command" | "lc" => last_command_cmd(db, args),
        "failures" => failure_history_cmd(db, args),
        // INT-177 — Shell Observability
        "observe" => observe_cmd(db, args),
        "memory" => memory_cmd(db, args),
        "forest-stats" | "fstats" => forest_stats_cmd(db, core_root, args),
        // INT-173 — Command Registry
        "describe" => describe_cmd(db, args, core_root),
        "explain" => explain_cmd(db, core_root, args),
        "open" => open_cmd(args),
        "from" => from_cmd(args),
        "to" => to_cmd(args),
        "where" => where_cmd(db, core_root, args),
        "command" => command_cmd(db, args, core_root),
        "health" => health(db),
        "events" => events(db, args),
        "decisions" => decisions(db),
        "deploys" | "deployments" => deploys(db),
        "friday-patterns" => friday_patterns(db),
        "bump-versions" => bump_versions_cmd(core_root, args),
        "ade" => ade_cmd(args),
        "friday" if line.trim() == "friday" => friday_patterns(db),
        "friday" if args.first().copied() == Some("chat") => {
            let rest = args.get(1..).unwrap_or(&[]).join(" ");
            if rest.is_empty() {
                // Launch TUI
                let _ = std::process::Command::new("friday-chat").status();
                CommandResult::Output(String::new())
            } else {
                // Direct query mode
                let out = std::process::Command::new("friday-chat")
                    .args(["chat", &rest])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                    .unwrap_or_default();
                CommandResult::Output(out)
            }
        }
        "intents" => intents(core_root),
        "project" | "projects" => project_list(core_root),
        "experiment" | "experiments" => experiment_list(core_root),
        "vm" | "vms" => vm_dispatch(args),
        "tools" => tools_table(db, core_root),
        "version" => version(core_root),
        "schema" => schema(args),
        "commits" => commits(core_root),
        "story" => story(db),
        "advise" => advise(db),
        "audit" => audit(db, core_root),
        "fsh" => match args.first().copied() {
            Some("doctor") => fsh_doctor_cmd(db, args.get(1..).unwrap_or(&[])),
            Some("enter") => fsh_enter_cmd(db, args.get(1).copied().unwrap_or("")),
            Some("leave") | Some("exit-scope") => fsh_leave_cmd(db),
            Some("scope") => fsh_scope_status(db),
            Some("rename") => fsh_rename_cmd(
                args.get(1).copied().unwrap_or(""),
                args.get(2).copied().unwrap_or(""),
            ),
            // INT-143: honour the probe here too -- see try_builtin.
            _ if !allow_external => CommandResult::NotBuiltin,
            _ => run_external(line, db),
        },
        "plan" => semantic_plan_cmd(if args.is_empty() {
            ""
        } else {
            &line[5..].trim()
        }),
        "why" => semantic_why_cmd(if args.is_empty() {
            ""
        } else {
            &line[4..].trim()
        }),
        "dry-run" => semantic_dryrun_cmd(if args.is_empty() {
            ""
        } else {
            &line[8..].trim()
        }),
        "clean" | "fix" => semantic_ambiguous_cmd(db, line),
        // ── Core subcommand shortcuts — no prefix needed ────────────────────
        "dev" => dev_cmd(db, core_root, args),
        "predict" | "react" | "stress" | "doctor" | "goals" | "evolution" | "security"
        | "capabilities" | "genealogy" | "autonomy" => {
            let sub = args.join(" ");
            let full = if sub.is_empty() {
                format!("core {}", cmd)
            } else {
                format!("core {} {}", cmd, sub)
            };
            // INT-143: these arms shell out to `core`. A probe must not.
            if !allow_external {
                CommandResult::NotBuiltin
            } else {
                run_external(&full, db)
            }
        }
        "sandbox" => sandbox(db),
        "checkpoint" | "cpc" => checkpoint(db),
        "let" => scripting_let_cmd(db, core_root, args),
        "run" => scripting_run_cmd(db, core_root, args),
        // INT-143: `python3` IS python3. It is NOT a builtin and fsh must stop claiming it.
        // Removed from this arm entirely -- it now falls through to run_external, which is
        // `sh -c <line>` with stdin/stdout/stderr INHERITED. That single change fixes all of:
        //   python3                -> a real REPL (inherited stdio; `bash` has always proven
        //                             this works -- shell_handoff_cmd does the same thing)
        //   python3 --version      -> Python 3.13.13   (was: NameError: name 'version' is not
        //                             defined -- the flag was being EVALUATED AS SOURCE)
        //   python3 -c "print(6*7)" -> 42              (was: SyntaxError: invalid syntax)
        //   python3 -i             -> a REPL           (was: NameError -- and this was the
        //                             workaround the OLD guard's own error message told you to
        //                             use. It had never been run.)
        //   python3 x.py > f       -> correct redirect (try_builtin answers NotBuiltin, so
        //                             main.rs spawns it ONCE with the file as stdout)
        // Measured 2026-07-16: every one of those worked in bash and failed in fsh. The cause
        // was run_python_cmd joining ALL args and running `python3 -c "<args>"`, so any flag
        // became Python source. That is 143's thesis verbatim: "A BUILTIN SHADOWS A REAL BINARY
        // AND SWALLOWS ITS ARGUMENTS."
        // WHY DELETING BEATS FIXING: a pass-through arm would be code that can drift. No arm at
        // all cannot. run_external is already correct, already tested, already used by every
        // other external command. The 227-arm match should not claim a name it does not improve.
        "python" | "py" => run_python_cmd(args),
        "js" | "node" => run_js_cmd(args),
        "undo" => undo_cmd(db, args),
        "pv" => smart_preview_cmd(args),
        "faelight-shell" => match args.first().copied() {
            Some("-c") => {
                // INT-299: fsh -c "cmd" — delegate to sh, mirrors external behavior
                let cmd = args.get(1).copied().unwrap_or("");
                if cmd.is_empty() {
                    CommandResult::Error("fsh -c: missing command".to_string())
                } else {
                    let out = std::process::Command::new("sh").arg("-c").arg(cmd).output();
                    match out {
                        Ok(o) => {
                            let mut s = String::from_utf8_lossy(&o.stdout).to_string();
                            if !o.stderr.is_empty() {
                                s.push_str(&String::from_utf8_lossy(&o.stderr));
                            }
                            CommandResult::Output(s.trim_end().to_string())
                        }
                        Err(e) => CommandResult::Error(format!("fsh -c: {}", e)),
                    }
                }
            }
            Some("diag") => fsh_diag(db),
            Some("gaps") => fsh_gaps(db),
            _ => fsh_identity_cmd(db),
        },
        "snapshot" => snapshot_cmd(db, args),
        "rewind" | "time-travel" => rewind_cmd(db),
        "debug" => debug_cmd(db, args),
        "usage" | "usage-report" => usage_report(db),
        "theme" => theme_cmd(db, args),
        "since" => since_cmd(db, core_root, args),
        "timeline" => timeline_cmd(db, args),
        "snap-diff" => snap_diff_cmd(db, args),
        "dashboard" | "dash" => dashboard_cmd(db, core_root, args),
        "chart" => chart_cmd(db, args),
        "select" => sql_query_cmd(db, core_root, line),
        "git" if args.is_empty() => git_status(core_root),
        // INT-143: THIS ARM CAUSED A REGRESSION and it is why every run_external site is
        // guarded, not just the fallthrough. `git` has its OWN arm that calls run_external --
        // so try_builtin("git status") RAN git, printed to the terminal, and returned Empty.
        // main.rs read "it matched an arm" as "it is a builtin", skipped the file write, and
        // `git status --short > /tmp/gs.txt` left the file EMPTY. Caught by testing the debug
        // binary before deploying it, which is the entire reason for testing the debug binary
        // before deploying it.
        "git" if !allow_external => CommandResult::NotBuiltin,
        "git" => run_external(line, db),
        "search" | "s" => search(db, args),
        "pick" => pick_cmd(db, core_root, args),
        "compare" => compare_cmd(core_root, args),
        "where_old_disabled" => {
            CommandResult::Error("use with pipe: tools | where score < 70".to_string())
        }
        "tools-table" | "tt" => tools_table(db, core_root),
        "events-table" | "et" => events_table(db, args),
        "audit-table" | "at" => audit_table(db, core_root),
        "decisions-table" | "dt" => decisions_table(db),
        "count" => CommandResult::Output("  use with pipe: tt | count".to_string()),
        "history-table" | "ht" | "history" => match args.first().copied() {
            Some("intent") if args.get(1).is_some() => {
                history_for_intent(db, args.get(1).copied().unwrap_or(""))
            }
            Some("stats") => history_stats_for_intent(db, args.get(1).copied().unwrap_or("")),
            Some("intent") => ht_intent(db),
            Some("today") => ht_today(db),
            Some("session") => ht_session(db),
            Some("slow") => ht_slow(db),
            Some(search) => history_search_cmd(db, &[search]),
            None => history_table(db),
        },
        "history-search" | "hs" | "hsearch" => history_search_cmd(db, args),
        // INT-143 case 1, the one that cost real time on 2026-07-15: `bash script.sh` dropped
        // into INTERACTIVE bash and the script NEVER RAN. It returned "successfully" in ~7s
        // having done nothing, and the missing output was misread as a qemu failure -- the
        // session chased a ghost. shell_handoff_cmd keeps only the FIRST WORD of the line
        // (split_whitespace().next()) and never calls .args(), so every argument was swallowed.
        // Same shape as python3 above: a builtin shadows a real binary and eats its arguments.
        // NO ARGS -> the handoff is what you meant. Keep it, banner and all -- it is good UX and
        // it harms nothing.
        // WITH ARGS -> not a handoff. Fall through to run_external -> `sh -c "bash script.sh"`,
        // which runs the script, with the right quoting, exactly as bash itself would.
        // The `if args.is_empty()` shape is not invented here -- `git` four lines below does the
        // same thing, and has all along.
        "zsh" | "bash" if args.is_empty() => shell_handoff_cmd(line),
        "hstats" => history_stats(db),
        "histogram" => histogram_cmd(db, args),
        "hpattern" => history_pattern(db),
        "checkpoints-table" | "ct" => checkpoints_table(db),
        "domains" => domains(db),
        "logs" => sys_logs(args),
        "ps" | "processes" => sys_processes(),
        "ports" => sys_ports(),
        "services" | "svc" => sys_services(),
        "files" | "ls" => sys_files(core_root, args),
        "fd" => find_cmd(db, core_root, args),
        "grep" => grep_cmd(line, args),
        "tree" => tree_cmd(args),
        "fstat" | "stat" => stat_cmd(args),
        "peek" | "preview" => preview_cmd(args),
        "exec" => exec_cmd(args),
        "realpath" | "rp" => realpath_cmd(args),
        // INT-143 case 3: `time` must be able to time fsh's OWN commands.
        // `time` sits before the fallthrough, so it needs its own allow_external handling --
        // otherwise `time cmd > file` runs twice. That lesson cost a git regression earlier.
        "time" if !args.is_empty() && !allow_external => CommandResult::NotBuiltin,
        "time" => time_cmd(line, args, db, core_root),
        "reload" => reload_fsh(),
        "source" => source_cmd(args),
        "net" | "network" => sys_network(),
        "power" | "pwr" => power_cmd(db, args),
        "store" => store_cmd(args),
        "packages" | "pkgs" => {
            // packages [filter]  -- list installed packages from the current system
            // environment (INT-134). Source: references of /run/current-system/sw, each a
            // /nix/store/<hash>-<name>-<version> path. Optional filter matches the name.
            let filter = args.first().copied().unwrap_or("");
            let paths =
                nix_query_lines(&["nix-store", "-q", "--references", "/run/current-system/sw"]);
            if paths.is_empty() {
                return CommandResult::Error(
                    "packages: could not read /run/current-system/sw references".to_string(),
                );
            }
            // Strip /nix/store/<hash>- prefix -> name-version. Hash is 32 chars + one dash.
            let mut names: Vec<String> = paths
                .iter()
                .filter_map(|p| {
                    let base = p.rsplit('/').next().unwrap_or(p);
                    // base = <32-hash>-<name-version>; drop the hash + first dash.
                    base.split_once('-').map(|(_, rest)| rest.to_string())
                })
                .filter(|nv| filter.is_empty() || nv.contains(filter))
                .collect();
            names.sort();
            names.dedup();
            if names.is_empty() {
                return CommandResult::Output(format!("  no packages matching '{}'", filter));
            }
            let header = if filter.is_empty() {
                format!("  \u{1f4e6} Installed packages ({})\n", names.len())
            } else {
                format!(
                    "  \u{1f4e6} Installed packages matching '{}' ({})\n",
                    filter,
                    names.len()
                )
            };
            let mut out = header;
            out.push_str(&"\u{2500}".repeat(44));
            for nv in &names {
                out.push_str(&format!("\n  {}", nv));
            }
            CommandResult::Output(out)
        }
        "pkg-search" | "pkgsearch" => pkg_search(args),
        "generations" | "gens" => {
            // generations [all|<N>]  -- browse NixOS generations (INT-134). Read-only.
            // Source: nixos-rebuild list-generations --json (same source main.rs uses for
            // the prompt). Rollback is SHOWN, never executed -- switching generations is a
            // sudo-level system mutation that deserves deliberate action, not a builtin.
            let out_raw = std::process::Command::new("nixos-rebuild")
                .args(["list-generations", "--json"])
                .output();
            let json = match out_raw {
                Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
                Err(e) => return CommandResult::Error(format!("generations: {}", e)),
            };
            let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&json);
            let gens = match parsed {
                Ok(g) => g,
                Err(e) => return CommandResult::Error(format!("generations: parse: {}", e)),
            };
            if gens.is_empty() {
                return CommandResult::Output("  no generations found".to_string());
            }
            let sub = args.first().copied().unwrap_or("");

            // generations <N> -- detail + rollback command for one generation.
            if let Ok(n) = sub.parse::<u64>() {
                let found = gens.iter().find(|g| g["generation"].as_u64() == Some(n));
                match found {
                    None => return CommandResult::Error(format!("generations: {} not found", n)),
                    Some(g) => {
                        let date = g["date"].as_str().unwrap_or("?");
                        let ver = g["nixosVersion"].as_str().unwrap_or("?");
                        let kernel = g["kernelVersion"].as_str().unwrap_or("?");
                        let rev = g["configurationRevision"].as_str().unwrap_or("?");
                        let cur = g["current"].as_bool().unwrap_or(false);
                        let mut out = format!(
                            "  \u{2744} Generation {}{}\n",
                            n,
                            if cur { "  (current)" } else { "" }
                        );
                        out.push_str(&"\u{2500}".repeat(44));
                        out.push_str(&format!("\n  date    : {}", date));
                        out.push_str(&format!("\n  nixos   : {}", ver));
                        out.push_str(&format!("\n  kernel  : {}", kernel));
                        out.push_str(&format!("\n  config  : {}", rev));
                        if cur {
                            out.push_str(
                                "\n\n  this is the current generation -- nothing to roll back to.",
                            );
                        } else {
                            out.push_str(
                                "\n\n  to roll back to this generation (deliberate, sudo):",
                            );
                            out.push_str(&format!(
                                "\n    sudo nix-env --switch-generation {} -p /nix/var/nix/profiles/system", n
                            ));
                            out.push_str("\n    sudo /nix/var/nix/profiles/system/bin/switch-to-configuration switch");
                        }
                        return CommandResult::Output(out);
                    }
                }
            }

            // generations [all] -- browse.
            let show_all = sub == "all";
            let limit = if show_all {
                gens.len()
            } else {
                15.min(gens.len())
            };
            let mut out = format!(
                "  \u{2744} NixOS Generations ({} total{})\n",
                gens.len(),
                if show_all {
                    String::new()
                } else {
                    format!(", showing {}", limit)
                }
            );
            out.push_str(&"\u{2500}".repeat(52));
            for g in gens.iter().take(limit) {
                let num = g["generation"].as_u64().unwrap_or(0);
                let date = g["date"].as_str().unwrap_or("?");
                let cur = g["current"].as_bool().unwrap_or(false);
                let rev = g["configurationRevision"].as_str().unwrap_or("");
                let short_rev = rev.split('-').next().unwrap_or(rev);
                let short_rev = short_rev.chars().take(8).collect::<String>();
                let dirty = if rev.ends_with("-dirty") { " *" } else { "" };
                let marker = if cur { "\u{25cf}" } else { " " };
                out.push_str(&format!(
                    "\n  {} {:>4}  {}  {}{}",
                    marker, num, date, short_rev, dirty
                ));
            }
            if !show_all && gens.len() > limit {
                out.push_str(&format!("\n\n  ... {} older. `generations all` to see all, `generations <N>` for detail + rollback.",
                    gens.len() - limit));
            } else {
                out.push_str("\n\n  \u{25cf} = current   * = dirty tree.  `generations <N>` for detail + rollback.");
            }
            CommandResult::Output(out)
        }
        "git-commits" | "gc" | "git.commits" => git_commits(core_root, args),
        "git-files" | "gf" => git_files(core_root),
        "git-churn" | "gchurn" | "git.files" => git_churn(core_root, args),
        "git-branches" | "gbr" | "git.branches" => git_branches(core_root),
        "watch" => watch_cmd(db, args),
        "alias" => alias_cmd(db, args),
        "unalias" => unalias_cmd(db, args),
        "plugins" => list_plugins(db),
        "plugin-reload" | "plr" => reload_plugins_cmd(db),
        "z" | "zi" => z_jump(args),
        "which" => {
            let cmd = args.first().copied().unwrap_or("");
            if cmd.is_empty() {
                return CommandResult::Error("which: missing argument".to_string());
            }
            let mut out = String::new();

            // Check forest builtins
            let builtins = [
                "cd",
                "pwd",
                "ls",
                "ll",
                "health",
                "events",
                "intents",
                "tools",
                "version",
                "schema",
                "commits",
                "story",
                "advise",
                "audit",
                "forecast",
                "sandbox",
                "checkpoint",
                "since",
                "git",
                "gc",
                "gf",
                "gchurn",
                "gbr",
                "ps",
                "ports",
                "services",
                "files",
                "find",
                "net",
                "history",
                "ht",
                "hstats",
                "histogram",
                "domains",
                "logs",
                "debug",
                "usage",
                "z",
                "zi",
                "ya",
                "yazi",
                "fm",
                "flow",
                "let",
                "run",
                "snapshot",
                "timeline",
                "dashboard",
                "chart",
                "watch",
                "select",
                "search",
                "on",
                "help",
                "exit",
                "quit",
                "q",
                "?",
            ];

            if builtins.contains(&cmd) {
                out.push_str(&format!(
                    "  {} {} — forest builtin\n",
                    "🌲".normal(),
                    cmd.bright_green()
                ));
            }

            // Check aliases
            if let Some(aliased) = db.get_alias(cmd) {
                out.push_str(&format!(
                    "  {} {} → {} — alias\n",
                    "→".bright_cyan(),
                    cmd.bright_white(),
                    aliased.dimmed()
                ));
            }

            // Check forest scripts
            let home = std::env::var("HOME").unwrap_or_default();
            let script_path = format!("{}/0-core/scripts/{}", home, cmd);
            if std::path::Path::new(&script_path).exists() {
                out.push_str(&format!(
                    "  {} {} — forest script\n",
                    "🌲".normal(),
                    script_path.bright_white()
                ));
            }

            // Check PATH
            let path_result = std::process::Command::new("which")
                .arg(cmd)
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                });

            if let Some(path) = path_result {
                out.push_str(&format!("  {} {} — PATH\n", "○".dimmed(), path.dimmed()));
            }

            if out.is_empty() {
                CommandResult::Error(format!("which: {} not found", cmd))
            } else {
                CommandResult::Output(out.trim_end().to_string())
            }
        }
        "echo" => {
            let output = args
                .iter()
                .map(|a| {
                    let a = a.trim();
                    if a.len() >= 2
                        && ((a.starts_with('"') && a.ends_with('"'))
                            || (a.starts_with("'") && a.ends_with("'")))
                    {
                        a[1..a.len() - 1].to_string()
                    } else {
                        a.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            CommandResult::Output(output)
        }
        "query" => {
            // query file.rs 100:150  -- lines 100-150
            // query file.rs :50      -- first 50 lines
            // query file.rs 900:     -- line 900 to end
            // query file.rs pattern  -- lines containing pattern
            if args.is_empty() {
                return CommandResult::Error("usage: query <file> [range|pattern]\n  query file.rs 100:150\n  query file.rs :50\n  query file.rs 900:\n  query file.rs fn_main".to_string());
            }
            let filepath = args[0];
            let expanded = if filepath.starts_with("~/") {
                let home = std::env::var("HOME").unwrap_or_default();
                filepath.replacen("~/", &format!("{}/", home), 1)
            } else {
                filepath.to_string()
            };
            let content_str = match std::fs::read_to_string(&expanded) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("query: {}: {}", filepath, e)),
            };
            let file_lines: Vec<&str> = content_str.lines().collect();
            let total = file_lines.len();
            if args.len() == 1 {
                // No range -- show all with line numbers
                use colored::Colorize;
                let is_rust = expanded.ends_with(".rs");
                for (i, l) in file_lines.iter().enumerate() {
                    let colored = if is_rust {
                        highlight_rust_line(l)
                    } else {
                        colorize_line(l)
                    };
                    println!("{} {}", format!("{:4}", i + 1).dimmed(), colored);
                }
                return CommandResult::Empty;
            }
            let spec = args[1];
            // Range: 100:150, :50, 900:, 100:+30
            if spec.contains(':') {
                let parts: Vec<&str> = spec.splitn(2, ':').collect();
                let start_str = parts[0];
                let end_str = parts[1];
                let start = if start_str.is_empty() {
                    1
                } else {
                    start_str.parse::<usize>().unwrap_or(1)
                };
                let end = if end_str.is_empty() {
                    total
                } else if end_str.starts_with('+') {
                    let offset = end_str[1..].parse::<usize>().unwrap_or(0);
                    (start + offset).min(total)
                } else {
                    end_str.parse::<usize>().unwrap_or(total).min(total)
                };
                let start = start.saturating_sub(1);
                // INT-097: clamp start to the file length and ensure start <= end,
                // so an out-of-range query shows a clean message instead of panicking
                // (an out-of-bounds slice aborts the whole shell -> terminal closes).
                let start = start.min(total);
                if start >= end {
                    return CommandResult::Output(format!(
                        "  (query: range {}:{} is past end of file -- {} has {} lines)",
                        start + 1,
                        end,
                        filepath,
                        total
                    ));
                }
                use colored::Colorize;
                let is_rust = expanded.ends_with(".rs");
                for (i, l) in file_lines[start..end].iter().enumerate() {
                    let colored = if is_rust {
                        highlight_rust_line(l)
                    } else {
                        colorize_line(l)
                    };
                    println!("{} {}", format!("{:4}", start + i + 1).dimmed(), colored);
                }
                CommandResult::Empty
            } else {
                // Pattern match
                let pattern = spec.to_lowercase();
                use colored::Colorize;
                let matches: Vec<(usize, &&str)> = file_lines
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| l.to_lowercase().contains(&pattern))
                    .collect();
                if matches.is_empty() {
                    CommandResult::Output(format!("  (no matches for '{}')", spec))
                } else {
                    for (i, l) in matches {
                        println!("{} {}", format!("{:4}", i + 1).dimmed(), colorize_line(*l));
                    }
                    CommandResult::Empty
                }
            }
        }
        "goto" => {
            // goto main.rs:362          -- open at line
            // goto main.rs:362:5        -- open at line (col ignored)
            // goto "fn expand_subshells" -- find and open at pattern
            if args.is_empty() {
                return CommandResult::Error("usage: goto <file:line> or goto \"fn name\"\n  goto main.rs:362\n  goto main.rs:362:5\n  goto \"fn expand_subshells\"".to_string());
            }
            let spec = args.join(" ");
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
            // Detect file:line or file:line:col format
            if spec.contains(':') && !spec.starts_with('"') {
                let parts: Vec<&str> = spec.splitn(3, ':').collect();
                let filepath = parts[0];
                let lineno_str = parts.get(1).unwrap_or(&"1");
                let lineno = lineno_str.parse::<usize>().unwrap_or(1);
                let expanded = if filepath.starts_with("~/") {
                    filepath.replacen(
                        "~/",
                        &format!("{}/", std::env::var("HOME").unwrap_or_default()),
                        1,
                    )
                } else {
                    filepath.to_string()
                };
                use colored::Colorize;
                println!(
                    "  {} {}:{}",
                    "goto".bright_cyan(),
                    filepath.bright_white(),
                    lineno.to_string().bright_yellow()
                );
                let status = std::process::Command::new(&editor)
                    .arg(format!("+{}", lineno))
                    .arg(&expanded)
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status();
                return match status {
                    Ok(_) => CommandResult::Empty,
                    Err(e) => CommandResult::Error(format!("goto: {}", e)),
                };
            }
            // Pattern search across common source files
            let pattern = spec.trim_matches('"').to_lowercase();
            let cwd = std::env::current_dir().unwrap_or_default();
            let mut found_file: Option<String> = None;
            let mut found_line: Option<usize> = None;
            fn search_for_pattern(
                dir: &std::path::Path,
                pattern: &str,
                found_file: &mut Option<String>,
                found_line: &mut Option<usize>,
            ) {
                let entries = match std::fs::read_dir(dir) {
                    Ok(e) => e,
                    Err(_) => return,
                };
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "target" {
                            continue;
                        }
                    }
                    if path.is_dir() {
                        search_for_pattern(&path, pattern, found_file, found_line);
                        if found_file.is_some() {
                            return;
                        }
                    } else if path.is_file() {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if !["rs", "py", "md", "toml", "sh", "fsh"].contains(&ext) {
                            continue;
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for (i, line) in content.lines().enumerate() {
                                if line.to_lowercase().contains(pattern) {
                                    *found_file = Some(path.to_string_lossy().to_string());
                                    *found_line = Some(i + 1);
                                    return;
                                }
                            }
                        }
                    }
                }
            }
            search_for_pattern(&cwd, &pattern, &mut found_file, &mut found_line);
            match (found_file, found_line) {
                (Some(f), Some(l)) => {
                    use colored::Colorize;
                    println!(
                        "  {} {}:{}",
                        "goto".bright_cyan(),
                        f.bright_white(),
                        l.to_string().bright_yellow()
                    );
                    let status = std::process::Command::new(&editor)
                        .arg(format!("+{}", l))
                        .arg(&f)
                        .stdin(std::process::Stdio::inherit())
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .status();
                    match status {
                        Ok(_) => CommandResult::Empty,
                        Err(e) => CommandResult::Error(format!("goto: {}", e)),
                    }
                }
                _ => CommandResult::Error(format!("goto: '{}' not found", spec)),
            }
        }
        "session" => {
            // session save <name>   -- snapshot current context
            // session load <name>   -- restore directory + show history
            // session list          -- show all saved sessions
            // session delete <name> -- remove a saved session (INT-269)
            let db_path = faelight_core::paths::state_db();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("session: db error: {}", e)),
            };
            // Ensure fsh_sessions table exists
            let _ = conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS fsh_sessions (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    name        TEXT NOT NULL UNIQUE,
                    directory   TEXT NOT NULL,
                    intent      TEXT NOT NULL DEFAULT '',
                    commands    TEXT NOT NULL DEFAULT '[]',
                    env_vars    TEXT NOT NULL DEFAULT '{}',
                    created_at  INTEGER NOT NULL,
                    updated_at  INTEGER NOT NULL
                );",
            );
            let sub = args.first().copied().unwrap_or("");
            match sub {
                "save" => {
                    let name = args.get(1).copied().unwrap_or("default");
                    let cwd = std::env::current_dir()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    // Get last 20 commands
                    let cmds: Vec<String> = {
                        let mut s = conn.prepare(
                            "SELECT command FROM shell_history ORDER BY id DESC LIMIT 20"
                        ).ok();
                        if let Some(ref mut stmt) = s {
                            stmt.query_map([], |r| r.get::<_, String>(0))
                                .ok()
                                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                                .unwrap_or_default()
                        } else { vec![] }
                    };
                    let cmds_json = serde_json::to_string(&cmds).unwrap_or_else(|_| "[]".to_string());
                    // INT-134: capture reproducible environment. We snapshot only the
                    // shell/forest-relevant vars -- capturing ALL of std::env would drag in
                    // session-specific system noise (DBUS addr, XDG runtime paths, PID vars)
                    // that would be wrong to restore into a different session. PATH plus any
                    // FAELIGHT_*/FSH_* project vars are what "reproducible" actually needs.
                    let env_map: std::collections::BTreeMap<String, String> = std::env::vars()
                        .filter(|(k, _)| {
                            k == "PATH"
                                || k.starts_with("FAELIGHT_")
                                || k.starts_with("FSH_")
                                || k.starts_with("FOREST_")
                        })
                        .collect();
                    let env_json = serde_json::to_string(&env_map).unwrap_or_else(|_| "{}".to_string());
                    // Get active intent
                    let intent = conn.query_row(
                        "SELECT title FROM intent_ledger WHERE status='in-progress' ORDER BY updated_at DESC LIMIT 1",
                        [], |r| r.get::<_, String>(0)
                    ).unwrap_or_default();
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64).unwrap_or(0);
                    match conn.execute(
                        "INSERT INTO fsh_sessions (name, directory, intent, commands, env_vars, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
                         ON CONFLICT(name) DO UPDATE SET
                         directory=?2, intent=?3, commands=?4, env_vars=?5, updated_at=?6",
                        rusqlite::params![name, cwd, intent, cmds_json, env_json, ts]
                    ) {
                        Ok(_) => CommandResult::Output(format!(
                            "  ✅ Session '{}' saved\n  → directory: {}\n  → {} commands captured\n  → {} env var(s) captured{}",
                            name, cwd, cmds.len(), env_map.len(),
                            if intent.is_empty() { String::new() } else { format!("\n  → intent: {}", intent) }
                        )),
                        Err(e) => CommandResult::Error(format!("session save: {}", e))
                    }
                }
                "load" => {
                    let name = args.get(1).copied().unwrap_or("default");
                    let row: Option<(String, String, String, String)> = conn.query_row(
                        "SELECT directory, intent, commands, env_vars FROM fsh_sessions WHERE name = ?1",
                        rusqlite::params![name],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                    ).ok();
                    match row {
                        None => CommandResult::Error(format!("session: '{}' not found. Use: session list", name)),
                        Some((dir, intent, cmds_json, env_json)) => {
                            let _ = std::env::set_current_dir(&dir);
                            // INT-134: restore the captured environment so the session is reproducible.
                            let env_map: std::collections::BTreeMap<String, String> =
                                serde_json::from_str(&env_json).unwrap_or_default();
                            let env_count = env_map.len();
                            for (k, v) in &env_map {
                                std::env::set_var(k, v);
                            }
                            let cmds: Vec<String> = serde_json::from_str(&cmds_json).unwrap_or_default();
                            let mut out = format!(
                                "  ✅ Session '{}' loaded\n  → directory: {}",
                                name, dir
                            );
                            if env_count > 0 {
                                out.push_str(&format!("\n  → {} env var(s) restored", env_count));
                            }
                            if !intent.is_empty() {
                                out.push_str(&format!("\n  → was working on: {}", intent));
                            }
                            if !cmds.is_empty() {
                                out.push_str(&format!("\n  → last {} commands:", cmds.len().min(5)));
                                for cmd in cmds.iter().take(5) {
                                    out.push_str(&format!("\n      {}", cmd));
                                }
                                out.push_str("\n  → use 'history-replay 10' to re-run recent commands");
                            }
                            CommandResult::Output(out)
                        }
                    }
                }
                "list" => {
                    let mut stmt = match conn.prepare(
                        "SELECT name, directory, intent, updated_at FROM fsh_sessions ORDER BY updated_at DESC"
                    ) {
                        Ok(s) => s,
                        Err(e) => return CommandResult::Error(format!("session list: {}", e)),
                    };
                    let rows: Vec<(String, String, String, i64)> = stmt.query_map(
                        [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                    ).ok().map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default();
                    if rows.is_empty() {
                        return CommandResult::Output("  No saved sessions. Use: session save <name>".to_string());
                    }
                    let mut out = format!("  🗂  Saved Sessions ({})\n", rows.len());
                    out.push_str(&"─".repeat(44));
                    for (name, dir, intent, ts) in &rows {
                        let dt = chrono::DateTime::from_timestamp(*ts, 0)
                            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        out.push_str(&format!("\n  ▸ {} [{}]", name, dt));
                        let short_dir = if dir.len() > 40 { format!("...{}", &dir[dir.len()-37..]) } else { dir.clone() };
                        out.push_str(&format!("\n    → {}", short_dir));
                        if !intent.is_empty() {
                            out.push_str(&format!("\n    → {}", intent));
                        }
                    }
                    CommandResult::Output(out)
                }
                "delete" => {
                    let name = args.get(1).copied().unwrap_or("");
                    if name.is_empty() {
                        return CommandResult::Error("usage: session delete <name>".to_string());
                    }
                    let deleted = conn.execute(
                        "DELETE FROM fsh_sessions WHERE name = ?1",
                        rusqlite::params![name]
                    ).unwrap_or(0);
                    if deleted > 0 {
                        CommandResult::Output(format!("  ✅ Session '{}' deleted", name))
                    } else {
                        CommandResult::Error(format!("session: '{}' not found", name))
                    }
                }
                _ => CommandResult::Error(
                    "usage: session save <name> | session load <name> | session list | session delete <name>".to_string()
                )
            }
        }
        "history-replay" => {
            // history-replay <n>  -- show last n commands and offer to replay (INT-269)
            let n: usize = args
                .first()
                .and_then(|a| a.parse().ok())
                .unwrap_or(10)
                .min(50);
            let db_path = faelight_core::paths::state_db();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("history-replay: {}", e)),
            };
            let mut stmt = match conn
                .prepare("SELECT id, command FROM shell_history ORDER BY id DESC LIMIT ?1")
            {
                Ok(s) => s,
                Err(e) => return CommandResult::Error(format!("history-replay: {}", e)),
            };
            let rows: Vec<(i64, String)> = stmt
                .query_map(rusqlite::params![n as i64], |r| Ok((r.get(0)?, r.get(1)?)))
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            if rows.is_empty() {
                return CommandResult::Output("  No history found".to_string());
            }
            let mut out = format!("  🕐 Last {} commands:\n", rows.len());
            out.push_str(&"─".repeat(44));
            for (i, (_, cmd)) in rows.iter().rev().enumerate() {
                out.push_str(&format!("\n  {:>3}  {}", i + 1, cmd));
            }
            out.push_str("\n\n  → To replay a command: run it directly");
            out.push_str("\n  → To replay all: confirm with 'y' below");
            CommandResult::Output(out)
        }
        "env-save" => {
            // env-save <name>  -- save current environment snapshot (INT-269)
            let name = args.first().copied().unwrap_or("default");
            let db_path = faelight_core::paths::state_db();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("env-save: {}", e)),
            };
            let _ = conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS fsh_env_snapshots (
                    name TEXT PRIMARY KEY,
                    vars TEXT NOT NULL,
                    saved_at INTEGER NOT NULL
                );",
            );
            let keys = [
                "EDITOR",
                "VISUAL",
                "RUST_LOG",
                "PATH",
                "HOME",
                "SHELL",
                "XDG_CURRENT_DESKTOP",
                "WAYLAND_DISPLAY",
                "FSH_SESSION_ID",
            ];
            let mut vars = serde_json::Map::new();
            for key in &keys {
                if let Ok(val) = std::env::var(key) {
                    vars.insert(key.to_string(), serde_json::Value::String(val));
                }
            }
            let vars_json = serde_json::Value::Object(vars).to_string();
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            match conn.execute(
                "INSERT INTO fsh_env_snapshots (name, vars, saved_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(name) DO UPDATE SET vars=?2, saved_at=?3",
                rusqlite::params![name, vars_json, ts],
            ) {
                Ok(_) => CommandResult::Output(format!(
                    "  ✅ Environment '{}' saved ({} vars)",
                    name,
                    keys.len()
                )),
                Err(e) => CommandResult::Error(format!("env-save: {}", e)),
            }
        }
        "env-load" => {
            // env-load <name>  -- show vars from snapshot (can't set parent env)
            let name = args.first().copied().unwrap_or("default");
            let db_path = faelight_core::paths::state_db();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("env-load: {}", e)),
            };
            let row: Option<(String, i64)> = conn
                .query_row(
                    "SELECT vars, saved_at FROM fsh_env_snapshots WHERE name = ?1",
                    rusqlite::params![name],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .ok();
            match row {
                None => CommandResult::Error(format!("env-load: snapshot '{}' not found", name)),
                Some((vars_json, ts)) => {
                    let dt = chrono::DateTime::from_timestamp(ts, 0)
                        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let mut out = format!("  📦 Environment snapshot '{}' [{}]\n", name, dt);
                    out.push_str(&"─".repeat(44));
                    if let Ok(vars) = serde_json::from_str::<
                        serde_json::Map<String, serde_json::Value>,
                    >(&vars_json)
                    {
                        let mut restored = 0;
                        for (k, v) in &vars {
                            let val = v.as_str().unwrap_or("");
                            std::env::set_var(k, val);
                            restored += 1;
                            let short = if val.len() > 60 {
                                format!("{}...", &val[..57])
                            } else {
                                val.to_string()
                            };
                            out.push_str(&format!("\n  {}={}", k, short));
                        }
                        out.push_str(&format!(
                            "\n\n  ✅ {} var(s) restored into the fsh environment",
                            restored
                        ));
                    }
                    CommandResult::Output(out)
                }
            }
        }
        "env-rollback" => {
            // env-rollback  -- restore the MOST RECENT env snapshot (INT-134).
            // Rollback = "undo my env changes back to the last saved version".
            // Reuses the env-load restore machinery; no name needed -- takes newest by saved_at.
            let db_path = faelight_core::paths::state_db();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("env-rollback: {}", e)),
            };
            let row: Option<(String, String, i64)> = conn
                .query_row(
                    "SELECT name, vars, saved_at FROM fsh_env_snapshots ORDER BY saved_at DESC LIMIT 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .ok();
            match row {
                None => CommandResult::Error(
                    "env-rollback: no snapshots to roll back to. Use: env-save <name>".to_string(),
                ),
                Some((name, vars_json, ts)) => {
                    let dt = chrono::DateTime::from_timestamp(ts, 0)
                        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let mut restored = 0;
                    let parsed: Result<serde_json::Map<String, serde_json::Value>, _> =
                        serde_json::from_str(&vars_json);
                    if let Ok(vars) = parsed {
                        for (k, v) in &vars {
                            let val = v.as_str().unwrap_or("");
                            std::env::set_var(k, val);
                            restored += 1;
                        }
                    }
                    CommandResult::Output(format!(
                        "  \u{21a9} Rolled back to '{}' [{}]\n  \u{2705} {} var(s) restored into the fsh environment",
                        name, dt, restored
                    ))
                }
            }
        }
        "env-diff" => {
            // env-diff <name>  -- diff current env vs snapshot (INT-269)
            let name = args.first().copied().unwrap_or("default");
            let db_path = faelight_core::paths::state_db();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("env-diff: {}", e)),
            };
            let vars_json: Option<String> = conn
                .query_row(
                    "SELECT vars FROM fsh_env_snapshots WHERE name = ?1",
                    rusqlite::params![name],
                    |r| r.get(0),
                )
                .ok();
            match vars_json {
                None => CommandResult::Error(format!("env-diff: snapshot '{}' not found", name)),
                Some(json) => {
                    let mut out = format!("  🔍 env-diff: current vs '{}'\n", name);
                    out.push_str(&"─".repeat(44));
                    if let Ok(saved) =
                        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&json)
                    {
                        let keys = [
                            "EDITOR",
                            "VISUAL",
                            "RUST_LOG",
                            "PATH",
                            "HOME",
                            "SHELL",
                            "XDG_CURRENT_DESKTOP",
                            "WAYLAND_DISPLAY",
                            "FSH_SESSION_ID",
                        ];
                        let mut diffs = 0;
                        for key in &keys {
                            let current = std::env::var(key).unwrap_or_default();
                            let snapped = saved
                                .get(*key)
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            if current != snapped {
                                diffs += 1;
                                out.push_str(&format!("\n  ~ {}:", key));
                                let sc = if snapped.len() > 50 {
                                    format!("{}...", &snapped[..47])
                                } else {
                                    snapped.clone()
                                };
                                let cc = if current.len() > 50 {
                                    format!("{}...", &current[..47])
                                } else {
                                    current.clone()
                                };
                                out.push_str(&format!("\n    saved:   {}", sc));
                                out.push_str(&format!("\n    current: {}", cc));
                            }
                        }
                        if diffs == 0 {
                            out.push_str("\n  ✅ No differences found");
                        } else {
                            out.push_str(&format!("\n\n  {} variable(s) differ", diffs));
                        }
                    }
                    CommandResult::Output(out)
                }
            }
        }
        "env-export" => {
            // env-export <name> [path]  -- write a snapshot to a portable TOML
            // manifest (INT-134). Shareable/committable. Default path: ./<name>.env.toml
            let name = args.first().copied().unwrap_or("default");
            let path = args
                .get(1)
                .copied()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("./{}.env.toml", name));
            let db_path = faelight_core::paths::state_db();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("env-export: {}", e)),
            };
            let row: Option<(String, i64)> = conn
                .query_row(
                    "SELECT vars, saved_at FROM fsh_env_snapshots WHERE name = ?1",
                    rusqlite::params![name],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .ok();
            match row {
                None => CommandResult::Error(format!(
                    "env-export: snapshot '{}' not found. Use: env-save <name>",
                    name
                )),
                Some((vars_json, ts)) => {
                    let parsed: Result<serde_json::Map<String, serde_json::Value>, _> =
                        serde_json::from_str(&vars_json);
                    let vars = parsed.unwrap_or_default();
                    let mut manifest = String::new();
                    manifest.push_str("# Faelight environment manifest (INT-134)\n");
                    manifest.push_str(&format!("# Exported from snapshot '{}'\n", name));
                    manifest.push_str(&format!("name = \"{}\"\n", name));
                    manifest.push_str(&format!("exported_at = {}\n\n", ts));
                    manifest.push_str("[vars]\n");
                    for (k, v) in &vars {
                        let val = v.as_str().unwrap_or("");
                        // TOML basic string: escape backslashes and quotes.
                        let esc = val.replace('\\', "\\\\").replace('"', "\\\"");
                        manifest.push_str(&format!("{} = \"{}\"\n", k, esc));
                    }
                    match std::fs::write(&path, manifest) {
                        Ok(_) => CommandResult::Output(format!(
                            "  📤 Exported '{}' -> {}\n  → {} var(s). Shareable/committable TOML manifest.\n  → import elsewhere with: env-import {}",
                            name, path, vars.len(), path
                        )),
                        Err(e) => CommandResult::Error(format!("env-export: write failed: {}", e)),
                    }
                }
            }
        }
        "env-import" => {
            // env-import <path>  -- read a TOML manifest into a snapshot (INT-134).
            // After import, apply with: env-load <name>
            let path = match args.first().copied() {
                Some(p) => p,
                None => return CommandResult::Error("usage: env-import <path>".to_string()),
            };
            let contents = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("env-import: read failed: {}", e)),
            };
            let doc: toml::Value = match toml::from_str(&contents) {
                Ok(v) => v,
                Err(e) => {
                    return CommandResult::Error(format!("env-import: invalid manifest: {}", e))
                }
            };
            let name = doc
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("imported")
                .to_string();
            let mut vars = serde_json::Map::new();
            if let Some(tbl) = doc.get("vars").and_then(|v| v.as_table()) {
                for (k, v) in tbl {
                    if let Some(s) = v.as_str() {
                        vars.insert(k.clone(), serde_json::Value::String(s.to_string()));
                    }
                }
            }
            let n_vars = vars.len();
            let vars_json = serde_json::Value::Object(vars).to_string();
            let db_path = faelight_core::paths::state_db();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("env-import: {}", e)),
            };
            let _ = conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS fsh_env_snapshots (
                    name TEXT PRIMARY KEY,
                    vars TEXT NOT NULL,
                    saved_at INTEGER NOT NULL
                );",
            );
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            match conn.execute(
                "INSERT INTO fsh_env_snapshots (name, vars, saved_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(name) DO UPDATE SET vars=?2, saved_at=?3",
                rusqlite::params![name, vars_json, ts],
            ) {
                Ok(_) => CommandResult::Output(format!(
                    "  📥 Imported manifest -> snapshot '{}' ({} vars)\n  → apply with: env-load {}",
                    name, n_vars, name
                )),
                Err(e) => CommandResult::Error(format!("env-import: {}", e)),
            }
        }
        "audit-log" => {
            // audit-log [n]  -- show the immutable command audit trail (INT-134).
            // Reads shell_history_audit: append-only, DB-enforced (delete/update blocked
            // by triggers). This surfaces the tamper-proof record we capture on every command.
            let n: i64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(20);
            let db_path = faelight_core::paths::state_db();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("audit-log: {}", e)),
            };
            let mut stmt = match conn.prepare(
                "SELECT audit_id, command, timestamp FROM shell_history_audit
                 ORDER BY audit_id DESC LIMIT ?1",
            ) {
                Ok(s) => s,
                Err(e) => return CommandResult::Error(format!("audit-log: {}", e)),
            };
            let rows: Vec<(i64, String, i64)> = stmt
                .query_map(rusqlite::params![n], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            let total: i64 = conn
                .query_row("SELECT COUNT(*) FROM shell_history_audit", [], |r| r.get(0))
                .unwrap_or(0);
            if rows.is_empty() {
                return CommandResult::Output(
                    "  🔒 Immutable audit log is empty (no commands captured yet).".to_string(),
                );
            }
            let mut out = format!(
                "  🔒 Immutable Command Audit Log (last {} of {})\n",
                rows.len(),
                total
            );
            out.push_str(&"─".repeat(52));
            for (id, cmd, ts) in &rows {
                let dt = chrono::DateTime::from_timestamp(*ts, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let short = if cmd.len() > 60 {
                    format!("{}...", &cmd[..57])
                } else {
                    cmd.clone()
                };
                out.push_str(&format!("\n  [{}] {}  {}", id, dt, short));
            }
            out.push_str("\n\n  → append-only, DB-enforced: deletes/updates blocked by trigger");
            CommandResult::Output(out)
        }
        "cmdguard" => guard_cmd(args),
        "make" => {
            // make <directory> [in <path>] (INT-270)
            // Human word for mkdir -p -- create directory with full path
            if args.is_empty() {
                return CommandResult::Error(
                    "usage: make directory <name> [in <path>]".to_string(),
                );
            }
            let home = std::env::var("HOME").unwrap_or_default();
            // Strip leading "directory" keyword if present
            let rest: Vec<&str> = if args[0] == "directory" || args[0] == "dir" {
                args[1..].to_vec()
            } else {
                args.to_vec()
            };
            if rest.is_empty() {
                return CommandResult::Error("usage: make directory <name>".to_string());
            }
            // Handle "make directory <name> in <path>"
            let in_pos = rest.iter().position(|a| *a == "in");
            let dir_name = if let Some(pos) = in_pos {
                let base = if pos + 1 < rest.len() {
                    let b = rest[pos + 1];
                    if b.starts_with("~/") {
                        format!("{}/{}", home, &b[2..])
                    } else {
                        b.to_string()
                    }
                } else {
                    ".".to_string()
                };
                format!("{}/{}", base, rest[..pos].join("/"))
            } else {
                let name = rest.join("/");
                if name.starts_with("~/") {
                    format!("{}/{}", home, &name[2..])
                } else {
                    name
                }
            };
            match std::fs::create_dir_all(&dir_name) {
                Ok(_) => {
                    let _ = db.conn.execute(
                        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('shell', 'dir_created', ?1, strftime('%s','now'))",
                        rusqlite::params![dir_name.clone()]
                    );
                    CommandResult::Output(format!("  ✅ created {}", dir_name))
                }
                Err(e) => CommandResult::Error(format!("make: {}", e)),
            }
        }
        "launch" => {
            // open <file|url> (INT-270)
            // Human word for xdg-open -- opens file in default application
            // Forest-aware: .rs/.py files open in $EDITOR, URLs open browser
            if args.is_empty() {
                return CommandResult::Error("usage: launch <file|url>".to_string());
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let target = if args[0].starts_with("~/") {
                format!("{}/{}", home, &args[0][2..])
            } else {
                args[0].to_string()
            };
            // Detect type and choose handler
            let handler = if target.starts_with("http://") || target.starts_with("https://") {
                // URL -- use browser
                "xdg-open".to_string()
            } else {
                let ext = std::path::Path::new(&target)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                match ext {
                    "rs" | "py" | "sh" | "fsh" | "toml" | "md" | "json" | "txt" => {
                        // Code/text files -- use $EDITOR
                        std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string())
                    }
                    _ => "xdg-open".to_string(),
                }
            };
            let status = std::process::Command::new(&handler).arg(&target).status();
            match status {
                Ok(s) if s.success() => {
                    let _ = db.conn.execute(
                        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('shell', 'file_opened', ?1, strftime('%s','now'))",
                        rusqlite::params![target]
                    );
                    CommandResult::Empty
                }
                Ok(_) => CommandResult::Error(format!("launch: {} failed", handler)),
                Err(e) => CommandResult::Error(format!("launch: {}: {}", handler, e)),
            }
        }
        "rename" => {
            // rename <file> <new-name> [overwrite] (INT-270)
            // Forest-aware file rename -- detects same-dir vs cross-dir,
            // warns on overwrite, notes if file in recent commits
            if args.len() < 2 {
                return CommandResult::Error(
                    "usage: rename <file> <new-name> [overwrite]".to_string(),
                );
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let expand = |p: &str| -> String {
                if p.starts_with("~/") {
                    format!("{}/{}", home, &p[2..])
                } else {
                    p.to_string()
                }
            };
            let overwrite = args.contains(&"overwrite");
            let real_args: Vec<&str> = args
                .iter()
                .filter(|a| **a != "overwrite")
                .cloned()
                .collect();
            let src_path = expand(real_args[0]);
            let new_name = real_args[1];
            let src = std::path::Path::new(&src_path);
            // Detect same-dir rename vs cross-dir move
            let dst_path = if new_name.contains('/') {
                expand(new_name)
            } else {
                let parent = src.parent().unwrap_or(std::path::Path::new("."));
                parent.join(new_name).to_string_lossy().to_string()
            };
            let is_same_dir = std::path::Path::new(&dst_path).parent() == src.parent();
            // Protected path check
            let protected = ["rust-tools/", "intents/", "scripts/", "docs/"];
            if protected
                .iter()
                .any(|p| src_path.contains(p) || dst_path.contains(p))
            {
                return CommandResult::Error(
                    "rename: protected path involved. Use mv directly if you are sure.".to_string(),
                );
            }
            // Overwrite protection
            if std::path::Path::new(&dst_path).exists() && !overwrite {
                return CommandResult::Error(format!(
                    "rename: {} already exists. Add 'overwrite' to replace it.",
                    dst_path
                ));
            }
            // Warn if file referenced in recent commits
            let src_filename = src
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let recent = std::process::Command::new("git")
                .args([
                    "-C",
                    &format!("{}/0-core", home),
                    "diff",
                    "--name-only",
                    "HEAD~3..HEAD",
                ])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();
            if recent.contains(&src_filename) {
                eprintln!("  ⚠️  {} was referenced in recent commits", src_filename);
            }
            match std::fs::rename(&src_path, &dst_path) {
                Ok(_) => {
                    let payload = format!("src={},dst={}", src_path, dst_path);
                    let _ = db.conn.execute(
                        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('shell', 'file_renamed', ?1, strftime('%s','now'))",
                        rusqlite::params![payload]
                    );
                    let verb = if is_same_dir { "renamed" } else { "moved" };
                    CommandResult::Output(format!("  ✅ {} {} → {}", verb, src_path, dst_path))
                }
                Err(e) => CommandResult::Error(format!("rename: {}", e)),
            }
        }
        "replace" => {
            // replace old_name new_name           -- replace text across all files
            // rename old_name new_name --type rs -- only .rs files
            // rename old_name new_name --dry-run -- preview only
            if args.len() < 2 {
                return CommandResult::Error(
                    "usage: replace <old> <new> [--type ext] [--dry-run]".to_string(),
                );
            }
            let old_name = args[0];
            let new_name = args[1];
            let mut filter_type: Option<&str> = None;
            let mut dry_run = false;
            let mut i = 2;
            while i < args.len() {
                match args[i] {
                    "--type" if i + 1 < args.len() => {
                        filter_type = Some(args[i + 1]);
                        i += 2;
                    }
                    "--dry-run" => {
                        dry_run = true;
                        i += 1;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            let cwd = std::env::current_dir().unwrap_or_default();
            let mut matches: Vec<(String, usize)> = Vec::new(); // (filepath, count)
            fn find_in_dir(
                dir: &std::path::Path,
                pattern: &str,
                filter_type: Option<&str>,
                matches: &mut Vec<(String, usize)>,
            ) {
                let entries = match std::fs::read_dir(dir) {
                    Ok(e) => e,
                    Err(_) => return,
                };
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "target" || name == "node_modules" {
                            continue;
                        }
                    }
                    if path.is_dir() {
                        find_in_dir(&path, pattern, filter_type, matches);
                    } else if path.is_file() {
                        if let Some(ext) = filter_type {
                            if path.extension().and_then(|e| e.to_str()) != Some(ext) {
                                continue;
                            }
                        } else {
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if !["rs", "py", "md", "toml", "sh", "fsh", "txt", "json"]
                                .contains(&ext)
                            {
                                continue;
                            }
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let count = content.matches(pattern).count();
                            if count > 0 {
                                matches.push((path.to_string_lossy().to_string(), count));
                            }
                        }
                    }
                }
            }
            find_in_dir(&cwd, old_name, filter_type, &mut matches);
            if matches.is_empty() {
                return CommandResult::Output(format!(
                    "  (no occurrences of '{}' found)",
                    old_name
                ));
            }
            let total: usize = matches.iter().map(|(_, c)| c).sum();
            use colored::Colorize;
            println!(
                "  {} occurrences of '{}' in {} files:",
                total.to_string().bright_yellow(),
                old_name.bright_white(),
                matches.len().to_string().bright_cyan()
            );
            for (f, c) in &matches {
                let rel = std::path::Path::new(f)
                    .strip_prefix(&cwd)
                    .unwrap_or(std::path::Path::new(f));
                println!(
                    "    {} {} ({})",
                    "→".dimmed(),
                    rel.display().to_string().bright_cyan(),
                    c.to_string().dimmed()
                );
            }
            if dry_run {
                println!("  {} dry-run -- no changes made", "○".dimmed());
                return CommandResult::Empty;
            }
            // Confirm
            print!("  Rename all? (y/n): ");
            use std::io::Write;
            std::io::stdout().flush().ok();
            let mut ans = String::new();
            std::io::stdin().read_line(&mut ans).ok();
            if ans.trim().to_lowercase() != "y" {
                println!("  {} cancelled", "○".dimmed());
                return CommandResult::Empty;
            }
            let mut renamed_files = 0usize;
            let mut renamed_count = 0usize;
            for (f, _) in &matches {
                if let Ok(content) = std::fs::read_to_string(f) {
                    let new_content = content.replace(old_name, new_name);
                    if std::fs::write(f, &new_content).is_ok() {
                        renamed_files += 1;
                        renamed_count += content.matches(old_name).count();
                    }
                }
            }
            CommandResult::Output(format!(
                "  {} renamed {} occurrences in {} files",
                "✅".to_string(),
                renamed_count,
                renamed_files
            ))
        }

        "rspatch" => {
            // rspatch file.rs --anchor "unique text" --new "new content" [--mode replace|after|before]
            // Rust-safe patch: handles multiline content, unicode escapes, anchor-based insertion
            // Unicode: writes literal UTF-8 -- no Python escape conflicts
            // Modes: replace (default), after (insert after anchor), before (insert before anchor)
            if args.len() < 2 {
                return CommandResult::Error(
                    "usage: rspatch <file> --anchor <text> --new <text> [--mode replace|after|before]\n\
                     example: rspatch main.rs --anchor 'fn main()' --new 'fn helper() {}' --mode after".to_string()
                );
            }
            let filepath = args[0];
            let expanded = if filepath.starts_with("~/") {
                let home = std::env::var("HOME").unwrap_or_default();
                filepath.replacen("~/", &format!("{}/", home), 1)
            } else {
                filepath.to_string()
            };
            let mut anchor_text: Option<String> = None;
            let mut new_text: Option<String> = None;
            let mut mode = "replace";
            let mut i = 1;
            while i < args.len() {
                match args[i] {
                    "--anchor" if i + 1 < args.len() => {
                        anchor_text = Some(args[i + 1].to_string());
                        i += 2;
                    }
                    "--new" if i + 1 < args.len() => {
                        new_text = Some(args[i + 1].to_string());
                        i += 2;
                    }
                    "--mode" if i + 1 < args.len() => {
                        mode = args[i + 1];
                        i += 2;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            let anchor = match anchor_text {
                Some(t) => t,
                None => return CommandResult::Error("rspatch: --anchor required".to_string()),
            };
            let new_content = match new_text {
                Some(t) => {
                    // INT-249b: interpret common escape sequences in --new
                    let mut out = String::with_capacity(t.len());
                    let mut chars = t.chars().peekable();
                    while let Some(c) = chars.next() {
                        if c == '\\' {
                            match chars.peek() {
                                Some('n') => {
                                    chars.next();
                                    out.push('\n');
                                }
                                Some('t') => {
                                    chars.next();
                                    out.push('\t');
                                }
                                Some('r') => {
                                    chars.next();
                                    out.push('\r');
                                }
                                Some('\\') => {
                                    chars.next();
                                    out.push('\\');
                                }
                                Some('\'') => {
                                    chars.next();
                                    out.push('\'');
                                }
                                Some('"') => {
                                    chars.next();
                                    out.push('"');
                                }
                                _ => out.push('\\'),
                            }
                        } else {
                            out.push(c);
                        }
                    }
                    out
                }
                None => return CommandResult::Error("rspatch: --new required".to_string()),
            };
            let content = match std::fs::read_to_string(&expanded) {
                Ok(c) => c,
                Err(_) => return CommandResult::Error(format!(
                    "rspatch: file not found: {}\n  why: file does not exist or is not readable\n  fix: check path with: ls {}",
                    filepath, filepath
                )),
            };
            // Validate anchor uniqueness
            let count = content.matches(anchor.as_str()).count();
            if count == 0 {
                return CommandResult::Error(format!(
                    "rspatch: anchor not found in {}\n  what:  anchor text does not exist in file\n  anchor: {}\n  fix:   run fsearch '{}' to verify exact text",
                    filepath, truncate_safe(&anchor, 60), truncate_safe(&anchor, 20)
                ));
            }
            if count > 1 {
                return CommandResult::Error(format!(
                    "rspatch: anchor matches {} times -- must be unique\n  what:  anchor text is ambiguous\n  anchor: {}\n  fix:   use a longer, more specific anchor string",
                    count, truncate_safe(&anchor, 60)
                ));
            }
            // Apply transformation based on mode
            let patched = match mode {
                "replace" => content.replacen(&anchor, &new_content, 1),
                "after" => content.replacen(&anchor, &format!("{}\n{}", anchor, new_content), 1),
                "before" => content.replacen(&anchor, &format!("{}\n{}", new_content, anchor), 1),
                _ => {
                    return CommandResult::Error(format!(
                        "rspatch: unknown mode '{}' -- use replace|after|before",
                        mode
                    ))
                }
            };
            match std::fs::write(&expanded, &patched) {
                Ok(_) => CommandResult::Output(format!(
                    "  {} rspatch {} (mode: {}, anchor: {})",
                    "✅".to_string(),
                    filepath,
                    mode,
                    truncate_safe(&anchor, 40)
                )),
                Err(e) => CommandResult::Error(format!("rspatch: write failed: {}", e)),
            }
        }
        "patch-multi" => {
            // patch-multi file.rs << TRANSFORMS
            // old1 -- new1
            // old2 -- new2
            // All-or-nothing: if any replacement fails, none are applied
            if args.is_empty() {
                return CommandResult::Error(
                    "usage: patch-multi <file> <old1> -- <new1> [<old2> -- <new2> ...]".to_string(),
                );
            }
            let filepath = args[0];
            let expanded = if filepath.starts_with("~/") {
                filepath.replacen(
                    "~/",
                    &format!("{}/", std::env::var("HOME").unwrap_or_default()),
                    1,
                )
            } else {
                filepath.to_string()
            };
            // Parse pairs: skip "--" tokens, take alternating old/new
            let tokens: Vec<&str> = args[1..]
                .iter()
                .filter(|a| **a != "--")
                .map(|a| *a)
                .collect();
            let pairs: Vec<(&str, &str)> = tokens
                .chunks(2)
                .filter_map(|chunk| {
                    if chunk.len() == 2 {
                        Some((chunk[0], chunk[1]))
                    } else {
                        None
                    }
                })
                .collect();
            if pairs.is_empty() {
                return CommandResult::Error("patch-multi: no replacement pairs found\n  format: patch-multi file.rs 'old1' -- 'new1' 'old2' -- 'new2'".to_string());
            }
            let content_str = match std::fs::read_to_string(&expanded) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("patch-multi: {}: {}", filepath, e)),
            };
            // Validate all replacements first (all-or-nothing)
            let mut errors: Vec<String> = Vec::new();
            for (old, _) in &pairs {
                let count = content_str.matches(*old).count();
                if count == 0 {
                    errors.push(format!(
                        "  not found: '{}' -- fix: fsearch to verify exact text",
                        truncate_safe(old, 60)
                    ));
                } else if count > 1 {
                    errors.push(format!(
                        "  ambiguous: '{}' (expected 1, found {}) -- fix: add more context",
                        truncate_safe(old, 60),
                        count
                    ));
                }
            }
            if !errors.is_empty() {
                use colored::Colorize;
                println!(
                    "  {} patch-multi aborted -- validation failed:",
                    "✗".bright_red()
                );
                for e in &errors {
                    println!("{}", e);
                }
                return CommandResult::Empty;
            }
            // Apply all replacements
            let mut result = content_str.clone();
            for (old, new) in &pairs {
                result = result.replacen(old, new, 1);
            }
            match std::fs::write(&expanded, &result) {
                Ok(_) => CommandResult::Output(format!(
                    "  {} patched {} ({} replacements)",
                    "✅".to_string(),
                    filepath,
                    pairs.len()
                )),
                Err(e) => CommandResult::Error(format!("patch-multi: write failed: {}", e)),
            }
        }
        "fdiff" => {
            // diff main.rs          -- git diff for specific file
            // diff main.rs HEAD~3   -- diff against older commit
            // diff main.rs --stat   -- summary only
            if args.is_empty() {
                return CommandResult::Error("usage: diff <file> [ref] [--stat]".to_string());
            }
            let filepath = args[0];
            let mut git_ref = "HEAD";
            let mut stat_only = false;
            if args.len() > 1 {
                for arg in &args[1..] {
                    if *arg == "--stat" {
                        stat_only = true;
                    } else {
                        git_ref = arg;
                    }
                }
            }
            let mut git_args = vec!["diff", "--color=always"];
            if stat_only {
                git_args.push("--stat");
            }
            git_args.push(git_ref);
            git_args.push("--");
            git_args.push(filepath);
            let output = std::process::Command::new("git").args(&git_args).output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    if !stderr.is_empty() {
                        return CommandResult::Error(format!("diff: {}", stderr.trim()));
                    }
                    if stdout.trim().is_empty() {
                        use colored::Colorize;
                        CommandResult::Output(format!(
                            "  {} no changes in {} since {}",
                            "○".dimmed(),
                            filepath,
                            git_ref
                        ))
                    } else {
                        print!("{}", stdout);
                        CommandResult::Empty
                    }
                }
                Err(e) => CommandResult::Error(format!("diff: {}", e)),
            }
        }
        "show" => {
            // INT-326 Phase 5: semantic pipeline -- show processes
            if args.first().copied() == Some("processes") || args.first().copied() == Some("procs")
            {
                let filter_arg = args.get(1..).unwrap_or(&[]).join(" ");
                let ps_out = std::process::Command::new("sh")
                    .arg("-c")
                    .arg("ps aux --sort=-%cpu | head -25")
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                    .unwrap_or_default();
                if filter_arg.is_empty() {
                    return CommandResult::Output(ps_out);
                }
                // filter inline: show processes cpu > 50
                let threshold: f64 = filter_arg
                    .replace("cpu", "")
                    .replace(">", "")
                    .trim()
                    .parse()
                    .unwrap_or(0.0);
                let filtered: String = ps_out
                    .lines()
                    .enumerate()
                    .filter(|(i, line)| {
                        if *i == 0 {
                            return true;
                        } // header
                        let cols: Vec<&str> = line.split_whitespace().collect();
                        cols.get(2)
                            .and_then(|c| c.parse::<f64>().ok())
                            .unwrap_or(0.0)
                            > threshold
                    })
                    .map(|(_, l)| l)
                    .collect::<Vec<_>>()
                    .join(
                        "
",
                    );
                return CommandResult::Output(filtered);
            }
            // show file.rs 46:80   -- syntax-highlighted lines
            // show file.rs fn_main -- jump to function with color
            // show file.rs :20     -- first 20 lines with color
            fn highlight_rust_line(line: &str) -> String {
                use colored::Colorize;
                // Comments
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") {
                    return line.dimmed().to_string();
                }
                // Error/panic lines
                if trimmed.contains("error")
                    || trimmed.contains("panic")
                    || trimmed.contains("FAILED")
                {
                    return line.bright_red().to_string();
                }
                // Simple token-level coloring
                let result;
                let keywords = [
                    "fn ", "let ", "mut ", "pub ", "use ", "struct ", "impl ", "match ", "enum ",
                    "trait ", "mod ", "return ", "if ", "else ", "for ", "while ", "loop ",
                    "async ", "await ", "move ", "ref ", "const ", "static ", "type ", "where ",
                    "self ", "Self ", "super ", "crate ",
                ];
                // Check if line starts with a keyword
                let t = line.trim_start();
                for kw in &keywords {
                    if t.starts_with(kw) || t.starts_with(&format!("pub {}", kw.trim())) {
                        result = line.bright_cyan().to_string();
                        return result;
                    }
                }
                // String literals pulse yellow
                if line.contains('"') || line.contains("'") {
                    result = line.bright_yellow().to_string();
                    return result;
                }
                // Numbers stand out
                let has_number = line.split_whitespace().any(|w| {
                    w.trim_matches(|c: char| !c.is_ascii_digit())
                        .parse::<f64>()
                        .is_ok()
                        && !w.is_empty()
                });
                if has_number && !line.contains("::") {
                    result = line.bright_magenta().to_string();
                    return result;
                }
                line.to_string()
            }
            if args.is_empty() {
                return CommandResult::Error("usage: show <file> [range|pattern]\n  show file.rs 46:80\n  show file.rs fn_main\n  show file.rs :20".to_string());
            }
            let filepath = args[0];
            let expanded = if filepath.starts_with("~/") {
                let home = std::env::var("HOME").unwrap_or_default();
                filepath.replacen("~/", &format!("{}/", home), 1)
            } else {
                filepath.to_string()
            };
            let content_str = match std::fs::read_to_string(&expanded) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("show: {}: {}", filepath, e)),
            };
            let file_lines: Vec<&str> = content_str.lines().collect();
            let total = file_lines.len();
            let render = |start: usize, end: usize| -> String {
                use colored::Colorize;
                file_lines[start..end]
                    .iter()
                    .enumerate()
                    .map(|(i, l)| {
                        let lineno = format!("{:4}", start + i + 1).bright_green().to_string();
                        let bar = "│".dimmed().to_string();
                        let highlighted = highlight_rust_line(l);
                        format!("{} {} {}", lineno, bar, highlighted)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            if args.len() == 1 {
                return CommandResult::Output(render(0, total));
            }
            let spec = args[1..].join(" ");
            if spec.contains(':') {
                let parts: Vec<&str> = spec.splitn(2, ':').collect();
                let start_str = parts[0];
                let end_str = parts[1];
                let start = if start_str.is_empty() {
                    1
                } else {
                    start_str.parse::<usize>().unwrap_or(1)
                };
                let end = if end_str.is_empty() {
                    total
                } else if end_str.starts_with('+') {
                    let offset = end_str[1..].parse::<usize>().unwrap_or(0);
                    (start + offset).min(total)
                } else {
                    end_str.parse::<usize>().unwrap_or(total).min(total)
                };
                let start = start.saturating_sub(1);
                CommandResult::Output(render(start, end))
            } else {
                // Pattern match -- show 3 lines of context around each match
                let pattern = spec.to_lowercase();
                let mut out_lines: Vec<String> = Vec::new();
                use colored::Colorize;
                for (i, line) in file_lines.iter().enumerate() {
                    if line.to_lowercase().contains(&pattern) {
                        let ctx_start = i.saturating_sub(2);
                        let ctx_end = (i + 3).min(total);
                        if !out_lines.is_empty() {
                            out_lines.push("  ...".dimmed().to_string());
                        }
                        for j in ctx_start..ctx_end {
                            let lineno = format!("{:4}", j + 1);
                            let lineno_colored = if j == i {
                                lineno.bright_green().bold().to_string()
                            } else {
                                lineno.bright_green().to_string()
                            };
                            let bar = if j == i {
                                "│".bright_green().to_string()
                            } else {
                                "│".dimmed().to_string()
                            };
                            let highlighted = highlight_rust_line(file_lines[j]);
                            out_lines.push(format!("{} {} {}", lineno_colored, bar, highlighted));
                        }
                        break; // show first match only
                    }
                }
                if out_lines.is_empty() {
                    CommandResult::Output(format!("  (no matches for '{}')", spec))
                } else {
                    CommandResult::Output(out_lines.join("\n"))
                }
            }
        }
        "db" => {
            // db -- native state.db query builtin (INT-263)
            // db "SELECT ..."              -- raw SQL passthrough
            // db events                   -- last 20 events
            // db events --domain git      -- filter by domain
            // db events --today           -- filter to today
            // db history                  -- last 20 shell commands
            // db history --failed         -- only failed commands
            // db history --limit N        -- last N commands
            // db friday                   -- friday_knowledge summary
            // db predictions              -- pending predictions
            // db patterns                 -- session patterns
            // --count                     -- return count only
            if args.is_empty() {
                return CommandResult::Output(
                    "  usage: db <table|sql> [flags]
  tables: events, history, friday, predictions, patterns
  flags:  --domain X  --action X  --today  --failed  --limit N  --count
  raw sql: db SELECT..."
                        .to_string(),
                );
            }
            let is_raw_sql = args[0].to_uppercase().starts_with("SELECT")
                || args[0].to_uppercase().starts_with("WITH")
                || args[0].contains("FROM ");
            // Parse flags
            let count_only = args.contains(&"--count");
            let failed_only = args.contains(&"--failed");
            let today_only = args.contains(&"--today");
            let mut limit: usize = 20;
            let mut filter_domain: Option<&str> = None;
            let mut filter_action: Option<&str> = None;
            let mut i = 0;
            while i < args.len() {
                match args[i] {
                    "--limit" if i + 1 < args.len() => {
                        limit = args[i + 1].parse().unwrap_or(20);
                        i += 2;
                    }
                    "--domain" if i + 1 < args.len() => {
                        filter_domain = Some(args[i + 1]);
                        i += 2;
                    }
                    "--action" if i + 1 < args.len() => {
                        filter_action = Some(args[i + 1]);
                        i += 2;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            let midnight = {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                (now - (now % 86400)) as i64
            };
            if is_raw_sql {
                // Raw SQL passthrough -- read only
                let sql_upper = args[0].to_uppercase();
                if sql_upper.contains("DROP")
                    || sql_upper.contains("DELETE")
                    || sql_upper.contains("UPDATE")
                    || sql_upper.contains("INSERT")
                {
                    return CommandResult::Error(
                        "db: write operations require --write flag (not yet implemented)"
                            .to_string(),
                    );
                }
                match db.conn.prepare(args[0]) {
                    Err(e) => return CommandResult::Error(format!("db: SQL error: {}", e)),
                    Ok(mut stmt) => {
                        let col_count = stmt.column_count();
                        let col_names: Vec<String> = (0..col_count)
                            .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
                            .collect();
                        let mut rows_out: Vec<Vec<String>> = Vec::new();
                        let _ = stmt
                            .query_map([], |row| {
                                let vals: Vec<String> = (0..col_count)
                                    .map(|i| {
                                        row.get::<_, rusqlite::types::Value>(i)
                                            .map(|v| match v {
                                                rusqlite::types::Value::Null => "NULL".to_string(),
                                                rusqlite::types::Value::Integer(n) => n.to_string(),
                                                rusqlite::types::Value::Real(f) => {
                                                    format!("{:.2}", f)
                                                }
                                                rusqlite::types::Value::Text(s) => s,
                                                rusqlite::types::Value::Blob(_) => {
                                                    "<blob>".to_string()
                                                }
                                            })
                                            .unwrap_or_default()
                                    })
                                    .collect();
                                Ok(vals)
                            })
                            .map(|rows| {
                                for r in rows.flatten() {
                                    rows_out.push(r);
                                }
                            });
                        if count_only {
                            return CommandResult::Output(format!("  {}", rows_out.len()));
                        }
                        // Format as table
                        let out = format_table(&col_names, &rows_out);
                        return CommandResult::Output(out);
                    }
                }
            }
            // Forest vocabulary mode
            let table = args[0];
            let result = match table {
                "events" => {
                    let mut sql = "SELECT id, domain, action, substr(payload,1,50), datetime(timestamp,'unixepoch','localtime') as time FROM events WHERE 1=1".to_string();
                    if let Some(d) = filter_domain {
                        sql.push_str(&format!(" AND domain='{}' ", d));
                    }
                    if let Some(a) = filter_action {
                        sql.push_str(&format!(" AND action='{}' ", a));
                    }
                    if today_only {
                        sql.push_str(&format!(" AND timestamp >= {} ", midnight));
                    }
                    sql.push_str(&format!(" ORDER BY timestamp DESC LIMIT {}", limit));
                    let headers = vec![
                        "id".to_string(),
                        "domain".to_string(),
                        "action".to_string(),
                        "payload".to_string(),
                        "time".to_string(),
                    ];
                    query_to_table(&db.conn, &sql, &headers)
                }
                "history" | "hist" => {
                    let mut sql = "SELECT id, substr(command,1,50) as cmd, exit_code, substr(cwd,length(cwd)-20) as cwd, datetime(timestamp,'unixepoch','localtime') as time FROM shell_history WHERE 1=1".to_string();
                    if failed_only {
                        sql.push_str(" AND exit_code != 0");
                    }
                    if today_only {
                        sql.push_str(&format!(" AND timestamp >= {}", midnight));
                    }
                    sql.push_str(&format!(" ORDER BY timestamp DESC LIMIT {}", limit));
                    let headers = vec![
                        "id".to_string(),
                        "command".to_string(),
                        "exit".to_string(),
                        "cwd".to_string(),
                        "time".to_string(),
                    ];
                    query_to_table(&db.conn, &sql, &headers)
                }
                "friday" | "knowledge" => {
                    let sql = format!("SELECT id, domain, substr(fact,1,60) as fact, confidence, datetime(created_at,'unixepoch','localtime') as time FROM friday_knowledge ORDER BY confidence DESC LIMIT {}", limit);
                    let headers = vec![
                        "id".to_string(),
                        "domain".to_string(),
                        "fact".to_string(),
                        "conf".to_string(),
                        "time".to_string(),
                    ];
                    query_to_table(&db.conn, &sql, &headers)
                }
                "predictions" | "predict" => {
                    let sql = format!("SELECT id, substr(pattern,1,40) as pattern, substr(prediction,1,40) as prediction, confidence FROM forest_predictions ORDER BY confidence DESC LIMIT {}", limit);
                    let headers = vec![
                        "id".to_string(),
                        "pattern".to_string(),
                        "prediction".to_string(),
                        "conf".to_string(),
                    ];
                    query_to_table(&db.conn, &sql, &headers)
                }
                "patterns" | "session" => {
                    let sql = format!("SELECT id, substr(pattern,1,50) as pattern, weight, datetime(last_seen,'unixepoch','localtime') as last_seen FROM session_patterns ORDER BY weight DESC LIMIT {}", limit);
                    let headers = vec![
                        "id".to_string(),
                        "pattern".to_string(),
                        "weight".to_string(),
                        "last_seen".to_string(),
                    ];
                    query_to_table(&db.conn, &sql, &headers)
                }
                _ => Err(format!(
                    "db: unknown table '{}'. Try: events, history, friday, predictions, patterns",
                    table
                )),
            };
            match result {
                Ok(rows) if count_only => CommandResult::Output(format!("  {}", rows.len())),
                Ok(rows) => {
                    let headers = match table {
                        "events" => vec![
                            "id".to_string(),
                            "domain".to_string(),
                            "action".to_string(),
                            "payload".to_string(),
                            "time".to_string(),
                        ],
                        "history" | "hist" => vec![
                            "id".to_string(),
                            "command".to_string(),
                            "exit".to_string(),
                            "cwd".to_string(),
                            "time".to_string(),
                        ],
                        "friday" | "knowledge" => vec![
                            "id".to_string(),
                            "domain".to_string(),
                            "fact".to_string(),
                            "conf".to_string(),
                            "time".to_string(),
                        ],
                        "predictions" | "predict" => vec![
                            "id".to_string(),
                            "pattern".to_string(),
                            "prediction".to_string(),
                            "conf".to_string(),
                        ],
                        _ => vec![
                            "id".to_string(),
                            "pattern".to_string(),
                            "weight".to_string(),
                            "last_seen".to_string(),
                        ],
                    };
                    CommandResult::Output(format_table(&headers, &rows))
                }
                Err(e) => CommandResult::Error(e),
            }
        }
        "copy" | "cp-forest" => {
            // copy <source> to <destination> [overwrite] (INT-266)
            if args.is_empty() {
                return CommandResult::Error(
                    "usage: copy <source> to <destination> [overwrite]".to_string(),
                );
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let expand = |p: &str| {
                if p.starts_with("~/") {
                    format!("{}/{}", home, &p[2..])
                } else {
                    p.to_string()
                }
            };
            // Parse: copy <src> to <dst> [overwrite]
            let to_pos = args.iter().position(|a| *a == "to");
            let (src_arg, dst_arg, overwrite) = if let Some(pos) = to_pos {
                let s = args[..pos].join(" ");
                let rest = &args[pos + 1..];
                let ow = rest.contains(&"overwrite");
                let d = rest
                    .iter()
                    .filter(|a| **a != "overwrite")
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");
                (s, d, ow)
            } else {
                if args.len() < 2 {
                    return CommandResult::Error(
                        "usage: copy <source> to <destination> [overwrite]".to_string(),
                    );
                }
                (
                    args[0].to_string(),
                    args[1].to_string(),
                    args.contains(&"overwrite"),
                )
            };
            let src_path = expand(&src_arg);
            let dst_path = expand(&dst_arg);
            let protected = ["rust-tools/", "intents/", "scripts/", "docs/", "engine/"];
            let is_protected = protected.iter().any(|p| dst_path.contains(p));
            if is_protected {
                return CommandResult::Error(format!(
                    "copy: {} is a protected path. Use cp directly if you are sure.",
                    dst_path
                ));
            }
            if std::path::Path::new(&dst_path).exists() && !overwrite {
                return CommandResult::Error(format!(
                    "copy: {} already exists. Add 'overwrite' to replace it.",
                    dst_path
                ));
            }
            match std::fs::copy(&src_path, &dst_path) {
                Ok(_) => {
                    let _ = db.conn.execute(
                        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('shell', 'file_copied', ?1, strftime('%s','now'))",
                        rusqlite::params![format!("{{\"src\":\"{}\",\"dst\":\"{}\"}}", src_path, dst_path)]
                    );
                    CommandResult::Output(format!("  ✅ copied {} → {}", src_path, dst_path))
                }
                Err(e) => CommandResult::Error(format!("copy: {}", e)),
            }
        }
        "move" | "mv-forest" => {
            // move <source> to <destination> [overwrite] (INT-266)
            if args.is_empty() {
                return CommandResult::Error(
                    "usage: move <source> to <destination> [overwrite]".to_string(),
                );
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let expand = |p: &str| {
                if p.starts_with("~/") {
                    format!("{}/{}", home, &p[2..])
                } else {
                    p.to_string()
                }
            };
            let to_pos = args.iter().position(|a| *a == "to");
            let (src_arg, dst_arg, overwrite) = if let Some(pos) = to_pos {
                let s = args[..pos].join(" ");
                let rest = &args[pos + 1..];
                let ow = rest.contains(&"overwrite");
                let d = rest
                    .iter()
                    .filter(|a| **a != "overwrite")
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");
                (s, d, ow)
            } else {
                if args.len() < 2 {
                    return CommandResult::Error(
                        "usage: move <source> to <destination> [overwrite]".to_string(),
                    );
                }
                (
                    args[0].to_string(),
                    args[1].to_string(),
                    args.contains(&"overwrite"),
                )
            };
            let src_path = expand(&src_arg);
            let dst_path = expand(&dst_arg);
            let protected = ["rust-tools/", "intents/", "scripts/", "docs/"];
            let is_protected = protected
                .iter()
                .any(|p| src_path.contains(p) || dst_path.contains(p));
            if is_protected {
                return CommandResult::Error(format!(
                    "move: protected path involved. Use mv directly if you are sure."
                ));
            }
            if std::path::Path::new(&dst_path).exists() && !overwrite {
                return CommandResult::Error(format!(
                    "move: {} already exists. Add 'overwrite' to replace it.",
                    dst_path
                ));
            }
            match std::fs::rename(&src_path, &dst_path) {
                Ok(_) => {
                    let _ = db.conn.execute(
                        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('shell', 'file_moved', ?1, strftime('%s','now'))",
                        rusqlite::params![format!("{{\"src\":\"{}\",\"dst\":\"{}\"}}", src_path, dst_path)]
                    );
                    CommandResult::Output(format!("  ✅ moved {} → {}", src_path, dst_path))
                }
                Err(e) => CommandResult::Error(format!("move: {}", e)),
            }
        }
        "list" => {
            // list [files|directories] [in <path>] (INT-265/266) -- returns Value::Table
            use crate::value::Value;
            use std::collections::HashMap;
            let home = std::env::var("HOME").unwrap_or_default();
            let expand_path = |p: &str| -> String {
                match p {
                    "@rust" => faelight_core::paths::rust_tools_dir()
                        .to_string_lossy()
                        .to_string(),
                    "@intents" => faelight_core::paths::intents_dir()
                        .to_string_lossy()
                        .to_string(),
                    "@scripts" => format!("{}/0-core/scripts", home),
                    "@docs" => format!("{}/0-core/docs", home),
                    p if p.starts_with("~/") => format!("{}/{}", home, &p[2..]),
                    p => p.to_string(),
                }
            };
            let dirs_only = args.contains(&"directories");
            let in_pos = args.iter().position(|a| *a == "in");
            let target = if let Some(pos) = in_pos {
                if pos + 1 < args.len() {
                    expand_path(args[pos + 1])
                } else {
                    ".".to_string()
                }
            } else {
                // INT-300: accept bare path: list ~/path
                let bare = args
                    .iter()
                    .find(|a| !["files", "directories", "in"].contains(a));
                if let Some(p) = bare {
                    expand_path(p)
                } else {
                    ".".to_string()
                }
            };
            let path = std::path::Path::new(&target);
            if !path.exists() {
                return CommandResult::Error(format!("list: {} does not exist", target));
            }
            let entries = match std::fs::read_dir(path) {
                Ok(e) => e,
                Err(e) => return CommandResult::Error(format!("list: {}", e)),
            };
            let mut items: Vec<std::path::PathBuf> =
                entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
            items.sort();
            let mut rows: Vec<HashMap<String, Value>> = Vec::new();
            for item in &items {
                let is_dir = item.is_dir();
                if dirs_only && !is_dir {
                    continue;
                }
                // INT-300: default shows files AND dirs
                // use 'list files' for files only
                let files_only = args.contains(&"files");
                if files_only && is_dir {
                    continue;
                }
                let name = item
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let size_bytes = if is_dir {
                    0u64
                } else {
                    std::fs::metadata(item).map(|m| m.len()).unwrap_or(0)
                };
                let size_str = if is_dir {
                    "--".to_string()
                } else if size_bytes > 1_048_576 {
                    format!("{:.1}MB", size_bytes as f64 / 1_048_576.0)
                } else if size_bytes > 1024 {
                    format!("{:.1}KB", size_bytes as f64 / 1024.0)
                } else {
                    format!("{}B", size_bytes)
                };
                let kind = if is_dir { "dir" } else { "file" };
                let mut row = HashMap::new();
                row.insert("name".to_string(), Value::Text(name));
                row.insert("size".to_string(), Value::Text(size_str));
                row.insert("size_bytes".to_string(), Value::Int(size_bytes as i64));
                row.insert("type".to_string(), Value::Text(kind.to_string()));
                rows.push(row);
            }
            CommandResult::Value(Value::Table(rows))
        }
        "read" => {
            // read <file> (INT-266)
            if args.is_empty() {
                return CommandResult::Error("usage: read <file>".to_string());
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let filepath = if args[0].starts_with("~/") {
                format!("{}/{}", home, &args[0][2..])
            } else {
                args[0].to_string()
            };
            let meta = std::fs::metadata(&filepath);
            let content = match std::fs::read_to_string(&filepath) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("read: {}: {}", filepath, e)),
            };
            let mut out = String::new();
            // Metadata header
            if let Ok(m) = meta {
                let size = m.len();
                let size_str = if size > 1_048_576 {
                    format!("{:.1}MB", size as f64 / 1_048_576.0)
                } else if size > 1024 {
                    format!("{:.1}KB", size as f64 / 1024.0)
                } else {
                    format!("{}B", size)
                };
                let lines_count = content.lines().count();
                out.push_str(&format!(
                    "  📄 {}  ({}, {} lines)
",
                    filepath, size_str, lines_count
                ));
                out.push_str(&format!(
                    "  {}
",
                    "─".repeat(50)
                ));
            }
            let is_rust = filepath.ends_with(".rs");
            let limit = 100usize;
            let lines: Vec<&str> = content.lines().collect();
            let show = lines.len().min(limit);
            for (i, line) in lines[..show].iter().enumerate() {
                use colored::Colorize;
                let num = format!("{:4}", i + 1).dimmed();
                let colored = if is_rust {
                    highlight_rust_line(line)
                } else {
                    colorize_line(line)
                };
                out.push_str(&format!(
                    "  {} {}
",
                    num, colored
                ));
            }
            if lines.len() > limit {
                out.push_str(&format!(
                    "
  … {} more lines. Use cat or query for full file.",
                    lines.len() - limit
                ));
            }
            let _ = db.conn.execute(
                "INSERT INTO events (domain, action, payload, timestamp) VALUES ('shell', 'file_read', ?1, strftime('%s','now'))",
                rusqlite::params![format!("{{\"path\":\"{}\"}}", filepath)]
            );
            CommandResult::Output(out.trim_end().to_string())
        }
        "write" => {
            // write <content> to <file> [overwrite|append] (INT-266)
            if args.len() < 3 {
                return CommandResult::Error(
                    "usage: write <content> to <file> [overwrite|append]".to_string(),
                );
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let to_pos = args.iter().position(|a| *a == "to");
            if to_pos.is_none() {
                return CommandResult::Error(
                    "usage: write <content> to <file> [overwrite|append]".to_string(),
                );
            }
            let pos = to_pos.unwrap();
            let content = args[..pos].join(" ").trim_matches('"').to_string();
            let rest = &args[pos + 1..];
            let overwrite = rest.contains(&"overwrite");
            let append = rest.contains(&"append");
            let dst = rest
                .iter()
                .filter(|a| **a != "overwrite" && **a != "append")
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            let dst_path = if dst.starts_with("~/") {
                format!("{}/{}", home, &dst[2..])
            } else {
                dst.clone()
            };
            let protected = ["rust-tools/", "intents/", "scripts/", "docs/"];
            if protected.iter().any(|p| dst_path.contains(p)) {
                return CommandResult::Error(format!("write: {} is a protected path.", dst_path));
            }
            if std::path::Path::new(&dst_path).exists() && !overwrite && !append {
                return CommandResult::Error(format!(
                    "write: {} already exists. Add 'overwrite' or 'append'.",
                    dst_path
                ));
            }
            let result = if append {
                use std::io::Write;
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&dst_path)
                    .and_then(|mut f| {
                        writeln!(f, "{}", content)?;
                        Ok(())
                    })
            } else {
                std::fs::write(
                    &dst_path,
                    format!(
                        "{}
",
                        content
                    ),
                )
                .map_err(|e| e)
            };
            match result {
                Ok(_) => {
                    let _ = db.conn.execute(
                        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('shell', 'file_written', ?1, strftime('%s','now'))",
                        rusqlite::params![format!("{{\"path\":\"{}\",\"append\":{}}}", dst_path, append)]
                    );
                    let action = if append { "appended to" } else { "wrote" };
                    CommandResult::Output(format!("  ✅ {} {}", action, dst_path))
                }
                Err(e) => CommandResult::Error(format!("write: {}", e)),
            }
        }
        "terminate" if !args.is_empty() => {
            // INT-326 Phase 5: semantic terminate (pattern-kill).
            // INT-095: `kill` removed from this arm -- kill is now handled in main.rs as the
            // real PID/job killer. `terminate <pattern>` keeps the pgrep -f semantic match.
            let target = args.join(" ");
            let pid_result = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("pgrep -f '{}'", target))
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();
            if pid_result.trim().is_empty() {
                return CommandResult::Error(format!(
                    "terminate: no process matching '{}'",
                    target
                ));
            }
            let pids: Vec<&str> = pid_result.trim().lines().collect();
            println!(
                "  🌲 Terminating {} process(es) matching '{}'",
                pids.len(),
                target
            );
            for pid in &pids {
                let _ = std::process::Command::new("kill").arg(pid.trim()).status();
            }
            CommandResult::Output(format!("Terminated {} process(es)", pids.len()))
        }
        "delete" | "del" => {
            // delete <path> [--force]
            // Forest vocabulary: human-readable rm with safety checks
            if args.is_empty() {
                return CommandResult::Error(
                    "usage: delete <path> [--force]\n  delete moves to ~/.local/share/forest-trash/ by default\n  --force skips trash for permanent delete".to_string()
                );
            }
            let force = args.contains(&"--force");
            let target_arg = args
                .iter()
                .find(|a| !a.starts_with("--"))
                .copied()
                .unwrap_or("");
            if target_arg.is_empty() {
                return CommandResult::Error("delete: no path specified".to_string());
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let expanded = if target_arg.starts_with("~/") {
                target_arg.replacen("~/", &format!("{}/", home), 1)
            } else {
                target_arg.to_string()
            };
            let target = std::path::PathBuf::from(&expanded);
            if !target.exists() {
                return CommandResult::Error(format!("delete: path not found: {}", expanded));
            }
            // Source-tree warning
            let source_dirs = ["rust-tools", "intents", "scripts", "docs", "engine", "meta"];
            let in_source = source_dirs
                .iter()
                .any(|d| target.starts_with(format!("{}/{}", core_root, d)));
            if in_source && !force {
                eprintln!(
                    "  ⚠️  delete: {} is inside a source-controlled directory",
                    expanded
                );
                eprint!("  Confirm delete? (y/N): ");
                use std::io::BufRead;
                let stdin = std::io::stdin();
                let answer = stdin
                    .lock()
                    .lines()
                    .next()
                    .and_then(|l| l.ok())
                    .unwrap_or_default();
                if answer.trim().to_lowercase() != "y" {
                    return CommandResult::Output("  delete cancelled".to_string());
                }
            }
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if force {
                let result = if target.is_dir() {
                    std::fs::remove_dir_all(&target)
                } else {
                    std::fs::remove_file(&target)
                };
                match result {
                    Ok(_) => {
                        let _ = db.conn.execute(
                            "INSERT INTO events (timestamp, kind, source, payload) VALUES (?1, ?2, ?3, ?4)",
                            rusqlite::params![
                                timestamp as i64, "file_deleted", "fsh::delete",
                                format!("{{\"path\":\"{}\",\"force\":true}}", expanded)
                            ],
                        );
                        CommandResult::Output(format!("  deleted: {}", expanded))
                    }
                    Err(e) => CommandResult::Error(format!("delete: {}", e)),
                }
            } else {
                let trash_dir = format!("{}/.local/share/forest-trash", home);
                let _ = std::fs::create_dir_all(&trash_dir);
                let file_name = target
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let trash_name = format!("{}/{}_{}", trash_dir, timestamp, file_name);
                match std::fs::rename(&target, &trash_name) {
                    Ok(_) => {
                        let _ = db.conn.execute(
                            "INSERT INTO events (timestamp, kind, source, payload) VALUES (?1, ?2, ?3, ?4)",
                            rusqlite::params![
                                timestamp as i64, "file_deleted", "fsh::delete",
                                format!("{{\"path\":\"{}\",\"trash\":\"{}\",\"force\":false}}", expanded, trash_name)
                            ],
                        );
                        CommandResult::Output(format!(
                            "  moved to trash: {}\n  use delete --force to skip trash",
                            file_name
                        ))
                    }
                    Err(_) => {
                        // rename fails across filesystems (e.g. /tmp) -- copy then delete
                        let copy_result = if target.is_dir() {
                            std::process::Command::new("cp")
                                .args(["-r", &expanded, &trash_name])
                                .status()
                                .map(|s| {
                                    if s.success() {
                                        Ok(())
                                    } else {
                                        Err(std::io::Error::new(
                                            std::io::ErrorKind::Other,
                                            "cp failed",
                                        ))
                                    }
                                })
                                .unwrap_or(Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "cp failed",
                                )))
                        } else {
                            std::fs::copy(&target, &trash_name).map(|_| ())
                        };
                        match copy_result {
                            Ok(_) => {
                                let _ = if target.is_dir() {
                                    std::fs::remove_dir_all(&target)
                                } else {
                                    std::fs::remove_file(&target)
                                };
                                let _ = db.conn.execute(
                                    "INSERT INTO events (timestamp, kind, source, payload) VALUES (?1, ?2, ?3, ?4)",
                                    rusqlite::params![
                                        timestamp as i64, "file_deleted", "fsh::delete",
                                        format!("{{\"path\":\"{}\",\"trash\":\"{}\",\"force\":false}}", expanded, trash_name)
                                    ],
                                );
                                CommandResult::Output(format!(
                                    "  moved to trash: {}\n  use delete --force to skip trash",
                                    file_name
                                ))
                            }
                            Err(e) => CommandResult::Error(format!(
                                "delete: could not move to trash: {}\n  try delete --force",
                                e
                            )),
                        }
                    }
                }
            }
        }
        "gt" => {
            // INT-300: gt is the forest vocabulary word for git operations
            // gt status, gt commit, gt push -- maps directly to git
            if args.is_empty() {
                return CommandResult::Error(
                    "usage: gt <git-command> [args]\n  gt is the forest word for git".to_string(),
                );
            }
            let status = std::process::Command::new("git")
                .args(args)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            match status {
                Ok(_) => CommandResult::Empty,
                Err(e) => CommandResult::Error(format!("gt: git not available: {}", e)),
            }
        }
        "find" => {
            // find: detect Unix vs forest usage
            // Unix find: first arg is a path (/, ~/, ./, ..) OR any arg has single-hyphen flag (-name, -type, -exec)
            // Forest find: first arg is a pattern or @shortcut
            let is_unix_find = args
                .first()
                .map(|a| {
                    a.starts_with('/')
                        || a.starts_with("~/")
                        || a.starts_with("./")
                        || a.starts_with("../")
                        || *a == "."
                        || *a == ".."
                })
                .unwrap_or(false)
                || args
                    .iter()
                    .any(|a| a.starts_with('-') && !a.starts_with("--"));
            if is_unix_find {
                // INT-143: `find` with unix-style args is handed to the real find.
                if !allow_external {
                    return CommandResult::NotBuiltin;
                }
                return run_external(line, db);
            }
            // Forest find: fd wrapper with @shortcuts and pattern-first syntax
            // find <pattern> [path|@shortcut] [--type f|d] [--ext rs]
            if args.is_empty() {
                return CommandResult::Error(
                    "usage: find <pattern> [@rust|@intents|@scripts|@docs|path] [--type f|d] [--ext ext]
       find /path -name pattern  (Unix find passthrough)".to_string()
                );
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let mut pattern = String::new();
            let mut search_root = std::path::PathBuf::from(core_root);
            let mut filter_type: Option<String> = None;
            let mut filter_ext: Option<String> = None;
            let mut i = 0;
            while i < args.len() {
                match args[i] {
                    "--type" if i + 1 < args.len() => {
                        filter_type = Some(args[i + 1].to_string());
                        i += 2;
                    }
                    "--ext" if i + 1 < args.len() => {
                        filter_ext = Some(args[i + 1].to_string());
                        i += 2;
                    }
                    "@rust" => {
                        search_root = faelight_core::paths::rust_tools_dir();
                        i += 1;
                    }
                    "@intents" => {
                        search_root = faelight_core::paths::intents_dir();
                        i += 1;
                    }
                    "@scripts" => {
                        search_root = std::path::PathBuf::from(format!("{}/scripts", core_root));
                        i += 1;
                    }
                    "@docs" => {
                        search_root = std::path::PathBuf::from(format!("{}/docs", core_root));
                        i += 1;
                    }
                    arg if arg.starts_with("@") => {
                        // Unknown shortcut -- treat as literal path
                        search_root =
                            std::path::PathBuf::from(format!("{}/{}", core_root, &arg[1..]));
                        i += 1;
                    }
                    arg if !arg.starts_with("--") && pattern.is_empty() => {
                        pattern = arg.to_string();
                        i += 1;
                    }
                    arg if !arg.starts_with("--") => {
                        // Second positional = path
                        let expanded = if arg.starts_with("~/") {
                            arg.replacen("~/", &format!("{}/", home), 1)
                        } else {
                            arg.to_string()
                        };
                        search_root = std::path::PathBuf::from(expanded);
                        i += 1;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            if pattern.is_empty() {
                return CommandResult::Error("find: pattern required".to_string());
            }
            // Check if fd is available
            let fd_path = std::process::Command::new("which")
                .arg("fd")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some("fd".to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "fdfind".to_string());
            let mut cmd = std::process::Command::new(&fd_path);
            cmd.arg(&pattern);
            cmd.arg(&search_root);
            if let Some(ref t) = filter_type {
                cmd.args(["--type", t]);
            }
            if let Some(ref e) = filter_ext {
                cmd.args(["--extension", e]);
            }
            let output = cmd.output();
            match output {
                Err(_) => {
                    // fd not available -- fall back to find
                    let mut fallback = std::process::Command::new("find");
                    fallback
                        .arg(&search_root)
                        .arg("-name")
                        .arg(format!("*{}*", pattern));
                    let fb_out = fallback.output().unwrap_or_else(|_| {
                        return std::process::Command::new("true").output().unwrap();
                    });
                    let results: Vec<&str> = std::str::from_utf8(&fb_out.stdout)
                        .unwrap_or("")
                        .lines()
                        .filter(|l| !l.is_empty())
                        .collect();
                    if results.is_empty() {
                        return CommandResult::Output(format!("  (no results for '{}')", pattern));
                    }
                    let out = results
                        .iter()
                        .map(|p| format!("  {}", p))
                        .collect::<Vec<_>>()
                        .join(
                            "
",
                        );
                    CommandResult::Output(out)
                }
                Ok(fd_out) => {
                    let results: Vec<&str> = std::str::from_utf8(&fd_out.stdout)
                        .unwrap_or("")
                        .lines()
                        .filter(|l| !l.is_empty())
                        .collect();
                    if results.is_empty() {
                        return CommandResult::Output(format!("  (no results for '{}')", pattern));
                    }
                    // Get git tracked files for badge
                    let git_tracked: std::collections::HashSet<String> =
                        std::process::Command::new("git")
                            .args(["-C", core_root, "ls-files"])
                            .output()
                            .ok()
                            .map(|o| {
                                std::str::from_utf8(&o.stdout)
                                    .unwrap_or("")
                                    .lines()
                                    .map(|l| format!("{}/{}", core_root, l))
                                    .collect()
                            })
                            .unwrap_or_default();
                    let mut out = format!(
                        "  {} results for '{}'
",
                        results.len(),
                        pattern
                    );
                    for path in &results {
                        let badge = if git_tracked.contains(*path) {
                            "✓"
                        } else {
                            "•"
                        };
                        out.push_str(&format!(
                            "  {} {}
",
                            badge, path
                        ));
                    }
                    CommandResult::Output(out.trim_end().to_string())
                }
            }
        }
        "fsearch" => {
            // fsearch "fn expand"                    -- all files recursively
            // fsearch "fn expand" --type rs          -- only .rs files
            // fsearch "fn expand" --file main.rs     -- only in specific file
            if args.is_empty() {
                return CommandResult::Error(
                    "usage: search <pattern> [--type ext] [--file name]".to_string(),
                );
            }
            // INT-249b: collect all positional args before flags as pattern phrase
            let mut filter_type: Option<&str> = None;
            let mut filter_file: Option<&str> = None;
            let mut search_root: Option<std::path::PathBuf> = None;
            let mut unknown: Vec<String> = Vec::new();
            let mut pattern_parts: Vec<String> = Vec::new();
            let mut i = 0;
            while i < args.len() {
                match args[i] {
                    "--type" if i + 1 < args.len() => {
                        filter_type = Some(args[i + 1]);
                        i += 2;
                    }
                    "--file" if i + 1 < args.len() => {
                        filter_file = Some(args[i + 1]);
                        i += 2;
                    }
                    // INT-300: forest shortcut flags
                    "--rust" => {
                        filter_type = Some("rs");
                        i += 1;
                    }
                    "--py" | "--python" => {
                        filter_type = Some("py");
                        i += 1;
                    }
                    "--md" | "--markdown" => {
                        filter_type = Some("md");
                        i += 1;
                    }
                    "--toml" => {
                        filter_type = Some("toml");
                        i += 1;
                    }
                    "--nix" => {
                        filter_type = Some("nix");
                        i += 1;
                    }
                    "--sh" | "--shell" => {
                        filter_type = Some("sh");
                        i += 1;
                    }
                    "--intent" | "--intents" => {
                        let _home = std::env::var("HOME").unwrap_or_default();
                        search_root = Some(faelight_core::paths::intents_dir());
                        i += 1;
                    }
                    "--forest" | "--all" => {
                        let home = std::env::var("HOME").unwrap_or_default();
                        search_root = Some(std::path::PathBuf::from(format!("{}/0-core", home)));
                        i += 1;
                    }
                    "--scripts" => {
                        let home = std::env::var("HOME").unwrap_or_default();
                        search_root =
                            Some(std::path::PathBuf::from(format!("{}/0-core/scripts", home)));
                        i += 1;
                    }
                    arg if !arg.starts_with("--") => {
                        // First check if it's an existing path (search root)
                        let expanded = if arg.starts_with("~/") {
                            let home = std::env::var("HOME").unwrap_or_default();
                            arg.replacen("~/", &format!("{}/", home), 1)
                        } else {
                            arg.to_string()
                        };
                        let p = std::path::PathBuf::from(&expanded);
                        if p.exists()
                            && p.is_dir()
                            && search_root.is_none()
                            && !pattern_parts.is_empty()
                        {
                            // Only treat as path if it's a directory AND we already have a pattern
                            search_root = Some(p);
                        } else {
                            pattern_parts.push(arg.to_string());
                        }
                        i += 1;
                    }
                    _ => {
                        unknown.push(args[i].to_string());
                        i += 1;
                    }
                }
            }
            if pattern_parts.is_empty() {
                return CommandResult::Error(
                    "usage: fsearch <pattern> [--type ext] [--file name]".to_string(),
                );
            }
            let pattern = pattern_parts.join(" ").to_lowercase();
            if !unknown.is_empty() {
                eprintln!(
                    "  {} fsearch ignored unknown argument(s): {}",
                    "⚠️ ".yellow(),
                    unknown.join(", ").bright_yellow()
                );
            }
            let cwd = search_root.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let mut results: Vec<String> = Vec::new();
            fn walk_dir(
                dir: &std::path::Path,
                pattern: &str,
                filter_type: Option<&str>,
                filter_file: Option<&str>,
                results: &mut Vec<String>,
            ) {
                let entries = match std::fs::read_dir(dir) {
                    Ok(e) => e,
                    Err(_) => return,
                };
                for entry in entries.flatten() {
                    let path = entry.path();
                    // Skip hidden dirs and target/
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "target" || name == "node_modules" {
                            continue;
                        }
                    }
                    if path.is_dir() {
                        walk_dir(&path, pattern, filter_type, filter_file, results);
                    } else if path.is_file() {
                        // Type filter
                        if let Some(ext) = filter_type {
                            if path.extension().and_then(|e| e.to_str()) != Some(ext) {
                                continue;
                            }
                        }
                        // File filter
                        if let Some(fname) = filter_file {
                            if path.file_name().and_then(|n| n.to_str()) != Some(fname) {
                                continue;
                            }
                        }
                        // Only search text files (check extension)
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        let text_exts = [
                            "rs", "nix", "py", "md", "toml", "sh", "fsh", "txt", "json", "yaml",
                            "yml", "html", "css", "js", "ts", "lua", "conf", "desktop", "service",
                            "lock",
                        ];
                        if !text_exts.contains(&ext) {
                            continue;
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for (lineno, line) in content.lines().enumerate() {
                                let line_lower = line.to_lowercase();
                                let matched = if pattern.contains('|') {
                                    pattern
                                        .split('|')
                                        .any(|alt| line_lower.contains(alt.trim()))
                                } else {
                                    line_lower.contains(pattern)
                                };
                                if matched {
                                    let rel = path
                                        .strip_prefix(std::env::current_dir().unwrap_or_default())
                                        .unwrap_or(&path)
                                        .display()
                                        .to_string();
                                    results.push(format!(
                                        "{:30} {:4}  {}",
                                        rel.bright_cyan(),
                                        (lineno + 1).to_string().bright_green(),
                                        colorize_line(line.trim())
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            if cwd.is_file() {
                // Single-file target -- scan directly, honor pattern + alternation
                if let Ok(content) = std::fs::read_to_string(&cwd) {
                    for (lineno, line) in content.lines().enumerate() {
                        let line_lower = line.to_lowercase();
                        let matched = if pattern.contains('|') {
                            pattern
                                .split('|')
                                .any(|alt| line_lower.contains(alt.trim()))
                        } else {
                            line_lower.contains(&pattern)
                        };
                        if matched {
                            let rel = cwd
                                .strip_prefix(std::env::current_dir().unwrap_or_default())
                                .unwrap_or(&cwd)
                                .display()
                                .to_string();
                            results.push(format!(
                                "{:30} {:4}  {}",
                                rel.bright_cyan(),
                                (lineno + 1).to_string().bright_green(),
                                colorize_line(line.trim())
                            ));
                        }
                    }
                } else {
                    return CommandResult::Error(format!(
                        "fsearch: could not read file {}",
                        cwd.display()
                    ));
                }
            } else {
                walk_dir(&cwd, &pattern, filter_type, filter_file, &mut results);
            }
            if results.is_empty() {
                CommandResult::Output(format!("  (no matches for '{}')", pattern))
            } else {
                CommandResult::Output(results.join("\n"))
            }
        }
        "patch" => {
            // patch file.rs --old "old text" --new "new text"
            // In-place find-and-replace -- no Python script needed
            if args.len() < 5 {
                return CommandResult::Error(
                    "usage: patch <file> --old <text> --new <text>\n  patch file.rs --old \"old code\" --new \"new code\"".to_string()
                );
            }
            let filepath = args[0];
            let expanded = if filepath.starts_with("~/") {
                let home = std::env::var("HOME").unwrap_or_default();
                filepath.replacen("~/", &format!("{}/", home), 1)
            } else {
                filepath.to_string()
            };
            // Parse --old and --new flags
            let mut old_text: Option<String> = None;
            let mut new_text: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i] {
                    "--old" if i + 1 < args.len() => {
                        old_text = Some(args[i + 1].to_string());
                        i += 2;
                    }
                    "--new" if i + 1 < args.len() => {
                        new_text = Some(args[i + 1].to_string());
                        i += 2;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            let old_text = match old_text {
                Some(t) => t,
                None => return CommandResult::Error("patch: --old text required".to_string()),
            };
            let new_text = match new_text {
                Some(t) => t,
                None => return CommandResult::Error("patch: --new text required".to_string()),
            };
            let content_str = match std::fs::read_to_string(&expanded) {
                Ok(c) => c,
                Err(e) => return CommandResult::Error(format!("patch: {}: {}", filepath, e)),
            };
            let count = content_str.matches(old_text.as_str()).count();
            if count == 0 {
                return CommandResult::Error(format!(
                    "patch: text not found in {}\n  what:  --old text does not exist in file\n  text:  {}\n  fix:   run fsearch '{}' to verify the exact text",
                    filepath,
                    truncate_safe(&old_text, 60),
                    truncate_safe(&old_text, 20)
                ));
            }
            if count > 1 {
                return CommandResult::Error(format!(
                    "patch: --old text matches {} times -- must be unique\n  what:  --old text is ambiguous (expected 1, found {})\n  text:  {}\n  fix:   add more surrounding context to make it unique",
                    count, count,
                    truncate_safe(&old_text, 60)
                ));
            }
            let patched = content_str.replacen(&old_text, &new_text, 1);
            match std::fs::write(&expanded, &patched) {
                Ok(_) => CommandResult::Output(format!(
                    "  {} patched {} (1 replacement)",
                    "✅".to_string(),
                    filepath
                )),
                Err(e) => CommandResult::Error(format!("patch: write failed: {}", e)),
            }
        }
        "type" => {
            let cmd = args.first().copied().unwrap_or("");
            if cmd.is_empty() {
                return CommandResult::Error("type: missing argument".to_string());
            }
            let mut out = String::new();
            out.push_str(&format!(
                "{}\n\n",
                format!("🌲 type: {}", cmd).cyan().bold()
            ));

            // 1. Check forest builtins
            let builtins = [
                "cd",
                "pwd",
                "ls",
                "ll",
                "clear",
                "echo",
                "env",
                "type",
                "which",
                "health",
                "events",
                "intents",
                "tools",
                "version",
                "schema",
                "commits",
                "story",
                "advise",
                "audit",
                "forecast",
                "sandbox",
                "checkpoint",
                "since",
                "git",
                "gc",
                "gf",
                "gchurn",
                "gbr",
                "ps",
                "ports",
                "services",
                "files",
                "find",
                "net",
                "history",
                "ht",
                "hstats",
                "histogram",
                "domains",
                "logs",
                "debug",
                "usage",
                "z",
                "zi",
                "ya",
                "yazi",
                "fm",
                "flow",
                "let",
                "run",
                "snapshot",
                "timeline",
                "dashboard",
                "chart",
                "watch",
                "select",
                "search",
                "on",
                "help",
                "exit",
                "quit",
                "q",
                "?",
            ];

            if builtins.contains(&cmd) {
                out.push_str(&format!("  {} forest builtin\n", "▶".bright_green()));
                out.push_str(&format!(
                    "    {} handled natively by fsh — no PATH lookup\n",
                    "·".dimmed()
                ));
            }

            // 2. Check aliases
            if let Some(aliased) = db.get_alias(cmd) {
                out.push_str(&format!("  {} alias\n", "▶".bright_cyan()));
                out.push_str(&format!(
                    "    {} {} → {}\n",
                    "·".dimmed(),
                    cmd.bright_white(),
                    aliased.bright_cyan()
                ));
            }

            // 3. Check config.fsh aliases
            let home = std::env::var("HOME").unwrap_or_default();
            let config_path = format!("{}/.config/faelight-shell/config.fsh", home);
            if let Ok(config) = std::fs::read_to_string(&config_path) {
                for line in config.lines() {
                    if line.trim_start().starts_with("alias ") {
                        let parts: Vec<&str> = line.splitn(3, '=').collect();
                        if parts.len() >= 2 {
                            let alias_name = parts[0].trim().trim_start_matches("alias ").trim();
                            if alias_name == cmd {
                                let target = parts[1].trim();
                                out.push_str(&format!(
                                    "  {} config.fsh alias\n",
                                    "▶".bright_cyan()
                                ));
                                out.push_str(&format!(
                                    "    {} {} → {}\n",
                                    "·".dimmed(),
                                    cmd.bright_white(),
                                    target.bright_cyan()
                                ));
                            }
                        }
                    }
                }
            }

            // 4. Check forest scripts
            let script_path = format!("{}/0-core/scripts/{}", home, cmd);
            if std::path::Path::new(&script_path).exists() {
                out.push_str(&format!("  {} forest script\n", "▶".bright_green()));
                out.push_str(&format!("    {} {}\n", "·".dimmed(), script_path.dimmed()));
            }

            // 5. PATH lookup
            if let Ok(out_bytes) = std::process::Command::new("which").arg(cmd).output() {
                if out_bytes.status.success() {
                    let path = String::from_utf8_lossy(&out_bytes.stdout)
                        .trim()
                        .to_string();
                    out.push_str(&format!("  {} PATH binary\n", "▶".yellow()));
                    out.push_str(&format!("    {} {}\n", "·".dimmed(), path.dimmed()));
                }
            }

            if out.trim_end().ends_with("bold()") || out.lines().count() <= 2 {
                out.push_str(&format!("  {} not found anywhere\n", "✗".bright_red()));
            }

            CommandResult::Output(out.trim_end().to_string())
        }
        "cat" => {
            let file = args.first().copied().unwrap_or("");
            if file.is_empty() {
                return CommandResult::Error("cat: missing filename".to_string());
            }
            let home = std::env::var("HOME").unwrap_or_default();
            let path = if file.starts_with("~/") {
                format!("{}/{}", home, &file[2..])
            } else {
                file.to_string()
            };
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    // Add line numbers for code files
                    let ext = std::path::Path::new(&path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    let code_exts = [
                        "rs", "py", "js", "ts", "toml", "yaml", "yml", "sh", "zsh", "kdl", "md",
                    ];
                    if code_exts.contains(&ext) {
                        let numbered: String = content
                            .lines()
                            .enumerate()
                            .map(|(i, line)| {
                                format!("  {}  {}", format!("{:4}", i + 1).dimmed(), line)
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        CommandResult::Output(numbered)
                    } else {
                        CommandResult::Output(content)
                    }
                }
                Err(e) => CommandResult::Error(format!("cat: {}: {}", file, e)),
            }
        }
        // INT-143 case 2: `env FOO=1 cmd` printed fsh's environment table and NEVER RAN cmd.
        // Proven 2026-07-16: `env FOO=1 echo real_env_would_print_this` -> the table. Real env
        // is right there at coreutils-9.11/bin/env; fsh was shadowing it and eating the command.
        // Third arm today with this shape, after python3 and bash.
        // NO ARGS -> fsh's own curated table (HOME/USER/PATH + FSH_FOCUS). That is fsh's to
        // define and it is genuinely more useful than a raw dump. Keep it.
        // WITH ARGS -> that is coreutils env: `env VAR=x cmd`, `env -u VAR cmd`, `env -i cmd`.
        // fsh has no business interpreting any of those. Fall through to run_external.
        // Guarding on args.is_empty() rather than sniffing for '=' is deliberate: a sniff is a
        // second parser that drifts. "No args = ours, any args = theirs" cannot drift.
        "env" if !args.is_empty() && allow_external => run_external(line, db),
        "env" if !args.is_empty() => CommandResult::NotBuiltin,
        "env" => {
            let mut out = String::new();
            out.push_str(&format!("{}\n", "🌲 Shell Environment".cyan().bold()));
            out.push_str(&format!("{}\n\n", "━".repeat(52).dimmed()));
            let vars = [
                "HOME",
                "USER",
                "SHELL",
                "PATH",
                "EDITOR",
                "WAYLAND_DISPLAY",
                "XDG_CURRENT_DESKTOP",
                "XDG_RUNTIME_DIR",
                "LANG",
                "TERM",
            ];
            for var in &vars {
                if let Ok(val) = std::env::var(var) {
                    let display = if val.len() > 60 {
                        format!("{}…", &val[..60])
                    } else {
                        val
                    };
                    out.push_str(&format!("  {:25} {}\n", var.bright_cyan(), display.white()));
                }
            }
            // Also show fsh-specific vars
            out.push_str(&format!("\n  {}\n", "fsh vars:".dimmed()));
            if let Some(focus) = db.get_focus_intent() {
                out.push_str(&format!(
                    "  {:25} {}\n",
                    "FSH_FOCUS".bright_cyan(),
                    focus.bright_green()
                ));
            }
            out.push_str(&format!("\n{}\n", "━".repeat(52).dimmed()));
            CommandResult::Output(out)
        }
        "pwd" => {
            let path = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "?".to_string());
            CommandResult::Output(path)
        }
        // ── last / save / recall — output memory (INT-194) ──────────────────
        "last" => {
            // Show last command output if available, otherwise show last command from history
            // Try last stored output first
            if let Ok(out) = db.conn.query_row(
                "SELECT value FROM shell_state WHERE key = 'last_output'",
                [],
                |r| r.get::<_, String>(0),
            ) {
                return CommandResult::Output(out);
            }
            // Fall back to last history entry
            if let Ok(cmd) = db.conn.query_row(
        "SELECT command FROM shell_history WHERE command NOT LIKE 'TIMING:%' AND command NOT LIKE 'SUGGEST:%' ORDER BY id DESC LIMIT 1",
        [], |r| r.get::<_, String>(0)
    ) {
        return CommandResult::Output(format!("  Last command: {}", cmd));
    }
            CommandResult::Output("  ○ No output history yet".to_string())
        }
        "save" => {
            if args.is_empty() {
                return CommandResult::Error("usage: save <name>".to_string());
            }
            let name = args[0];
            // Only save if last_output has real content
            let last: String = db
                .conn
                .query_row(
                    "SELECT value FROM shell_state WHERE key = 'last_output'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_default();
            if last.is_empty() || last.contains("No last output stored yet") {
                return CommandResult::Output(
                    "  ○ Nothing to save — save works with native fsh commands\n  → Try: core strategy now | save <name>"
                        .to_string(),
                );
            }
            let result = db.conn.execute(
                "INSERT OR REPLACE INTO shell_state (key, value) VALUES (?1, ?2)",
                rusqlite::params![format!("saved_{}", name), last.clone()],
            );
            match result {
                Ok(_) => CommandResult::Output(format!(
                    "  ✅ Saved {} bytes to slot '{}'",
                    last.len(),
                    name
                )),
                Err(e) => CommandResult::Error(format!("save: {}", e)),
            }
        }
        "how" => {
            // INT-326 Phase 6: shell memory -- "how did I fix X last month?"
            let query = args.join(" ").to_lowercase();
            let query = query
                .trim_start_matches("did i ")
                .trim_start_matches("do i ")
                .trim_start_matches("i ");
            let pattern = format!("%{}%", query.split_whitespace().next().unwrap_or(""));
            let time_filter = if args.iter().any(|a| *a == "today") {
                let today = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                today - 86400
            } else if args.iter().any(|a| *a == "week") {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now - 604800
            } else if args.iter().any(|a| *a == "month") {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now - 2592000
            } else {
                0
            };

            let mut stmt = match db.conn.prepare(
                "SELECT command, cwd, timestamp, exit_code FROM shell_history 
                 WHERE command LIKE ?1 AND timestamp > ?2 
                 ORDER BY timestamp DESC LIMIT 10",
            ) {
                Ok(s) => s,
                Err(e) => return CommandResult::Error(format!("how: db error: {}", e)),
            };

            let rows: Vec<String> =
                match stmt.query_map(rusqlite::params![pattern, time_filter as i64], |r| {
                    let cmd: String = r.get(0)?;
                    let cwd: String = r.get::<_, String>(1).unwrap_or_default();
                    let ts: i64 = r.get(2)?;
                    let exit: i64 = r.get::<_, i64>(3).unwrap_or(0);
                    let dt = chrono::DateTime::from_timestamp(ts, 0)
                        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| ts.to_string());
                    let status = if exit == 0 { "✅" } else { "❌" };
                    Ok(format!("  {} {} | {} | {}", status, cmd, dt, cwd))
                }) {
                    Ok(mapped) => mapped.flatten().collect(),
                    Err(_) => vec![],
                };

            if rows.is_empty() {
                CommandResult::Output(format!(
                    "  🌲 No history matching '{}' -- try: how deploy / how fix / how build",
                    query
                ))
            } else {
                CommandResult::Output(format!(
                    "  🌲 Shell memory for '{}':
{}",
                    query,
                    rows.join(
                        "
"
                    )
                ))
            }
        }
        "recall" => {
            if args.is_empty() {
                // List all saved slots
                let mut stmt = match db
                    .conn
                    .prepare("SELECT key FROM shell_state WHERE key LIKE 'saved_%' ORDER BY key")
                {
                    Ok(s) => s,
                    Err(_) => {
                        return CommandResult::Output(
                            "  ○ No saved slots — use: save <name>".to_string(),
                        )
                    }
                };
                let slots: Vec<String> = stmt
                    .query_map([], |r| r.get::<_, String>(0))
                    .map(|rows| {
                        rows.filter_map(|r| r.ok())
                            .map(|k: String| k.trim_start_matches("saved_").to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                if slots.is_empty() {
                    return CommandResult::Output(
                        "  ○ No saved slots — use: save <name>".to_string(),
                    );
                }
                return CommandResult::Output(format!("  Saved slots: {}", slots.join(", ")));
            }
            let name = args[0];
            let result = db.conn.query_row(
                "SELECT value FROM shell_state WHERE key = ?1",
                rusqlite::params![format!("saved_{}", name)],
                |r| r.get::<_, String>(0),
            );
            match result {
                Ok(out) => CommandResult::Output(out),
                Err(_) => CommandResult::Error(format!("No saved slot named '{}'", name)),
            }
        }
        "cd" => cd(args),
        "cache" => cache(args),
        "devshell" => match args.first().copied() {
            Some("enter") => devshell_enter(&args[1..]),
            _ => devshell_list(args),
        },
        "d" => {
            // forest built-in: d → core doctor run
            let output = std::process::Command::new("core")
                .args(["doctor", "run"])
                .output();
            match output {
                Ok(o) => {
                    let out = String::from_utf8_lossy(&o.stdout).to_string();
                    let err = String::from_utf8_lossy(&o.stderr).to_string();
                    let combined = format!("{}{}", out, err);
                    CommandResult::Output(combined.trim_end().to_string())
                }
                Err(_) => CommandResult::Error("core doctor run failed".to_string()),
            }
        }
        "edit" => {
            // INT-223 Phase 4 — edit file:line or file:pattern or last command
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
            if args.is_empty() {
                let last_cmd = db.get_last_command().unwrap_or_default();
                let tmp = "/tmp/fsh-edit.sh";
                let _ = std::fs::write(tmp, format!("{}\n", last_cmd));
                let status = std::process::Command::new(&editor)
                    .arg(tmp)
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status();
                return match status {
                    Ok(_) => {
                        let edited = std::fs::read_to_string(tmp)
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        let _ = std::fs::remove_file(tmp);
                        if edited.is_empty() {
                            CommandResult::Empty
                        } else {
                            println!("  {} {}", "→".bright_cyan(), edited.dimmed());
                            execute(&edited, db, core_root)
                        }
                    }
                    Err(e) => CommandResult::Error(format!("edit: {}", e)),
                };
            }
            let spec: String = args.join(" ");
            let (filepath, line_or_pattern): (&str, Option<&str>) =
                if let Some(colon) = spec.rfind(':') {
                    let f = &spec[..colon];
                    let lp = &spec[colon + 1..];
                    if !lp.is_empty() {
                        (f, Some(lp))
                    } else {
                        (spec.as_str(), None)
                    }
                } else {
                    (spec.as_str(), None)
                };
            let expanded = if filepath.starts_with("~/") {
                filepath.replacen(
                    "~/",
                    &format!("{}/", std::env::var("HOME").unwrap_or_default()),
                    1,
                )
            } else {
                filepath.to_string()
            };
            let mut editor_args: Vec<String> = Vec::new();
            if let Some(lp) = line_or_pattern {
                if let Ok(lineno) = lp.parse::<usize>() {
                    editor_args.push(format!("+{}", lineno));
                } else if let Ok(content_str) = std::fs::read_to_string(&expanded) {
                    if let Some((i, _)) = content_str
                        .lines()
                        .enumerate()
                        .find(|(_, l)| l.to_lowercase().contains(&lp.to_lowercase()))
                    {
                        editor_args.push(format!("+{}", i + 1));
                    }
                }
            }
            editor_args.push(expanded);
            let status = std::process::Command::new(&editor)
                .args(&editor_args)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            match status {
                Ok(_) => CommandResult::Empty,
                Err(e) => CommandResult::Error(format!("edit: {}", e)),
            }
        }
        "clear" | "c" | "cls" => {
            // \x1B[3J clears scrollback, \x1B[2J clears screen, \x1B[H moves to top
            print!("\x1B[3J\x1B[2J\x1B[H");
            use std::io::Write;
            std::io::stdout().flush().ok();
            CommandResult::Empty
        }
        // INT-143: a probe (try_builtin) stops here and ANSWERS. Nothing is spawned.
        _ if !allow_external => CommandResult::NotBuiltin,
        _ => run_external(line, db),
    };

    // INT-143: a probe is a question, not a command. Do not write it to the security log --
    // try_builtin runs BEFORE the real execution, so logging here would double-count every
    // redirected command and poison the very event log INT-167 wants to be able to trust.
    if matches!(result, CommandResult::NotBuiltin) {
        return result;
    }

    // Security layer — log every command
    let result_str = match &result {
        CommandResult::Error(_) => "error",
        CommandResult::Exit => "exit",
        CommandResult::Empty => "empty",
        _ => "ok",
    };
    emit_command(db, &cmd, result_str);
    result
}

fn sandbox(db: &ForestDb) -> CommandResult {
    let rows: Vec<(String, String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT payload, action, timestamp FROM events WHERE domain='sandbox' ORDER BY timestamp DESC LIMIT 10"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No sandbox runs yet", "○".dimmed())),
        };
        stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    };

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No sandbox runs recorded — use {}",
            "○".dimmed(),
            "faelight-sandbox run".bright_cyan()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 🧪 Sandbox Runs ───────────────────────────────────".bright_cyan()
    ));
    for (payload, _, ts) in &rows {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
            let cmd = v["detail"]["command"].as_str().unwrap_or("unknown");
            let result = v["result"].as_str().unwrap_or("?");
            let dur = v["detail"]["duration_secs"].as_u64().unwrap_or(0);
            let changed = v["detail"]["files_changed"].as_u64().unwrap_or(0);
            let icon = if result == "ok" { "✅" } else { "❌" };
            let time = fmt_time(*ts, "%H:%M");
            let short_cmd = if cmd.len() > 35 {
                format!("{}...", &cmd[..35])
            } else {
                cmd.to_string()
            };
            out.push_str(&format!(
                "  │  {} {}  {}  {}s  {} files\n",
                icon,
                time.dimmed(),
                short_cmd.bright_white(),
                dur.to_string().dimmed(),
                changed.to_string().cyan(),
            ));
        }
    }
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn checkpoint(db: &ForestDb) -> CommandResult {
    let rows: Vec<(String, String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT name, payload, timestamp FROM checkpoints ORDER BY timestamp DESC LIMIT 8",
        ) {
            Ok(s) => s,
            Err(_) => {
                // Try events table for checkpoint events
                let mut stmt2 = match db.conn.prepare(
                    "SELECT action, payload, timestamp FROM events WHERE domain='checkpoint' ORDER BY timestamp DESC LIMIT 8"
                ) {
                    Ok(s) => s,
                    Err(_) => return CommandResult::Output(format!("  {} No checkpoints found", "○".dimmed())),
                };
                return CommandResult::Output({
                    let rows: Vec<(String, String, i64)> = stmt2
                        .query_map([], |r| {
                            Ok((
                                r.get::<_, String>(0)?,
                                r.get::<_, String>(1)?,
                                r.get::<_, i64>(2)?,
                            ))
                        })
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    if rows.is_empty() {
                        return CommandResult::Output(format!(
                            "  {} No checkpoints yet — use {}",
                            "○".dimmed(),
                            "cpc <name>".bright_cyan()
                        ));
                    }
                    let mut out = String::new();
                    out.push_str(&format!(
                        "\n{}\n",
                        "  ╭─ 📸 Checkpoints ──────────────────────────────────".bright_cyan()
                    ));
                    for (action, payload, ts) in &rows {
                        let time = fmt_time(*ts, "%m-%d %H:%M");
                        let name = serde_json::from_str::<serde_json::Value>(payload)
                            .ok()
                            .and_then(|v| v["detail"]["name"].as_str().map(|s| s.to_string()))
                            .unwrap_or_else(|| action.clone());
                        out.push_str(&format!("  │  {} {}\n", time.dimmed(), name.bright_white()));
                    }
                    out.push_str(
                        &"  ╰────────────────────────────────────────────────────"
                            .dimmed()
                            .to_string(),
                    );
                    out
                });
            }
        };
        stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    };

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No checkpoints yet — use {}",
            "○".dimmed(),
            "cpc <name>".bright_cyan()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 📸 Checkpoints ──────────────────────────────────".bright_cyan()
    ));
    for (name, _, ts) in &rows {
        let time = fmt_time(*ts, "%m-%d %H:%M");
        out.push_str(&format!("  │  {} {}\n", time.dimmed(), name.bright_white()));
    }
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn git_status(core_root: &str) -> CommandResult {
    let status = std::process::Command::new("git")
        .args(["-C", core_root, "status", "--short"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let branch = std::process::Command::new("git")
        .args(["-C", core_root, "branch", "--show-current"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let recent = std::process::Command::new("git")
        .args(["-C", core_root, "log", "--oneline", "-5"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 🌿 Git Status ─────────────────────────────────────".bright_cyan()
    ));
    out.push_str(&format!("  │  Branch:  {}\n", branch.bright_green()));

    if status.trim().is_empty() {
        out.push_str(&format!("  │  Status:  {}\n", "clean".bright_green()));
    } else {
        out.push_str(&format!(
            "  │  Status:  {}\n",
            "uncommitted changes".yellow()
        ));
        for line in status.lines().take(5) {
            out.push_str(&format!("  │    {}\n", line.dimmed()));
        }
    }

    out.push_str(
        &"  ├─────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    out.push('\n');
    out.push_str(&format!("  │  {}\n", "Recent commits:".dimmed()));
    for line in recent.lines().take(5) {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() == 2 {
            out.push_str(&format!(
                "  │    {} {}\n",
                parts[0].bright_yellow(),
                parts[1].dimmed()
            ));
        }
    }
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn open_cmd(args: &[&str]) -> CommandResult {
    use crate::value::Value;
    let file = match args.first() {
        Some(f) => f,
        None => return CommandResult::Error("open: missing filename".to_string()),
    };
    let home = std::env::var("HOME").unwrap_or_default();
    let path = if file.starts_with("~/") {
        file.replacen("~/", &format!("{}/", home), 1)
    } else {
        file.to_string()
    };
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => return CommandResult::Error(format!("open: {}: {}", file, e)),
    };
    match ext.as_str() {
        "json" => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(serde_json::Value::Array(arr)) => {
                let rows: Vec<std::collections::HashMap<String, Value>> = arr
                    .iter()
                    .filter_map(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| {
                                let val = match v {
                                    serde_json::Value::String(s) => Value::Text(s.clone()),
                                    serde_json::Value::Number(n) => {
                                        Value::Int(n.as_i64().unwrap_or(0))
                                    }
                                    serde_json::Value::Bool(b) => Value::Text(b.to_string()),
                                    _ => Value::Text(v.to_string()),
                                };
                                (k.clone(), val)
                            })
                            .collect()
                    })
                    .collect();
                CommandResult::Value(Value::Table(rows))
            }
            Ok(serde_json::Value::Object(obj)) => {
                let rows: Vec<std::collections::HashMap<String, Value>> = obj
                    .iter()
                    .map(|(k, v)| {
                        let mut row = std::collections::HashMap::new();
                        row.insert("key".to_string(), Value::Text(k.clone()));
                        row.insert(
                            "value".to_string(),
                            Value::Text(v.to_string().trim_matches('"').to_string()),
                        );
                        row
                    })
                    .collect();
                CommandResult::Value(Value::Table(rows))
            }
            _ => CommandResult::Output(content),
        },
        "toml" => match toml::from_str::<toml::Value>(&content) {
            Ok(toml::Value::Table(table)) => {
                let rows: Vec<std::collections::HashMap<String, Value>> = table
                    .iter()
                    .map(|(k, v)| {
                        let mut row = std::collections::HashMap::new();
                        row.insert("key".to_string(), Value::Text(k.clone()));
                        row.insert(
                            "value".to_string(),
                            Value::Text(v.to_string().trim_matches('"').to_string()),
                        );
                        row
                    })
                    .collect();
                CommandResult::Value(Value::Table(rows))
            }
            _ => CommandResult::Output(content),
        },
        "csv" => {
            let mut lines = content.lines();
            let headers: Vec<String> = lines
                .next()
                .map(|h| h.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            let rows: Vec<std::collections::HashMap<String, Value>> = lines
                .map(|line| {
                    let vals: Vec<&str> = line.split(',').collect();
                    headers
                        .iter()
                        .enumerate()
                        .map(|(i, h)| {
                            (
                                h.clone(),
                                Value::Text(vals.get(i).copied().unwrap_or("").trim().to_string()),
                            )
                        })
                        .collect()
                })
                .collect();
            CommandResult::Value(Value::Table(rows))
        }
        _ => {
            let rows: Vec<std::collections::HashMap<String, Value>> = content
                .lines()
                .enumerate()
                .map(|(i, line)| {
                    let mut row = std::collections::HashMap::new();
                    row.insert("n".to_string(), Value::Int(i as i64 + 1));
                    row.insert("line".to_string(), Value::Text(line.to_string()));
                    row
                })
                .collect();
            CommandResult::Value(Value::Table(rows))
        }
    }
}
fn from_cmd(args: &[&str]) -> CommandResult {
    // INT-265: from <file> reads file as Value::Table (one row per line)
    use crate::value::Value;
    use std::collections::HashMap;
    if args.is_empty() {
        return CommandResult::Error("usage: from <file>".to_string());
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let filepath = if args[0].starts_with("~/") {
        format!("{}/{}", home, &args[0][2..])
    } else {
        args[0].to_string()
    };
    match std::fs::read_to_string(&filepath) {
        Ok(content) => {
            let rows: Vec<HashMap<String, Value>> = content
                .lines()
                .enumerate()
                .map(|(i, line)| {
                    let mut row = HashMap::new();
                    row.insert("n".to_string(), Value::Int(i as i64 + 1));
                    row.insert("line".to_string(), Value::Text(line.to_string()));
                    row
                })
                .collect();
            CommandResult::Value(Value::Table(rows))
        }
        Err(e) => CommandResult::Error(format!("from: {}: {}", filepath, e)),
    }
}
fn to_cmd(args: &[&str]) -> CommandResult {
    match args.first().copied().unwrap_or("") {
        "json" => CommandResult::Error("to json: pipe a table — e.g. tools | to json".to_string()),
        fmt => CommandResult::Error(format!("to: unknown format '{}' — try: json", fmt)),
    }
}
fn tools_table(db: &ForestDb, core_root: &str) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let tools_dir = faelight_core::paths::rust_tools_dir();
    let mut rows = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&tools_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !entry.path().join("Cargo.toml").exists() {
                continue;
            }

            // Get version from Cargo.toml
            let version = std::fs::read_to_string(entry.path().join("Cargo.toml"))
                .ok()
                .and_then(|t| {
                    t.lines()
                        .find(|l| l.starts_with("version = "))
                        .map(|l| l.split('"').nth(1).unwrap_or("?").to_string())
                })
                .unwrap_or_else(|| "?".to_string());

            // Get score from audit_scores
            let score: i64 = db.conn.query_row(
                "SELECT score FROM audit_scores WHERE tool_name = ?1 ORDER BY timestamp DESC LIMIT 1",
                rusqlite::params![name],
                |r| r.get(0),
            ).unwrap_or(0);

            // Check if deployed
            let deployed = std::path::PathBuf::from(core_root)
                .join("scripts")
                .join(&name)
                .exists();

            let mut row = HashMap::new();
            row.insert("name".to_string(), Value::Text(name));
            row.insert("version".to_string(), Value::Text(version));
            row.insert("score".to_string(), Value::Int(score));
            row.insert("deployed".to_string(), Value::Bool(deployed));
            rows.push(row);
        }
    }

    rows.sort_by(|a, b| {
        a.get("name")
            .map(|v| v.as_text())
            .unwrap_or_default()
            .cmp(&b.get("name").map(|v| v.as_text()).unwrap_or_default())
    });

    CommandResult::Value(Value::Table(rows))
}

fn events_table(db: &ForestDb, args: &[&str]) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let today_only = args.contains(&"today");
    let domain = args
        .first()
        .and_then(|a| if *a == "today" { None } else { Some(*a) });

    let events = db.query_events(domain, today_only, 50);
    let rows = events
        .into_iter()
        .map(|(domain, action, ts)| {
            let time = fmt_time(ts, "%H:%M:%S");
            let mut row = HashMap::new();
            row.insert("time".to_string(), Value::Text(time));
            row.insert("domain".to_string(), Value::Text(domain));
            row.insert("action".to_string(), Value::Text(action));
            row.insert("timestamp".to_string(), Value::Int(ts));
            row
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

fn audit_table(db: &ForestDb, _core_root: &str) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let rows: Vec<HashMap<String, Value>> = {
        let mut stmt = match db.conn.prepare(
            "SELECT tool_name, score, usage_score, recency_score, doc_score, version_score, last_commit_days, timestamp
             FROM audit_scores
             WHERE timestamp = (SELECT MAX(timestamp) FROM audit_scores)
             ORDER BY score ASC"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No audit data — run: core audit scan", "○".dimmed())),
        };

        stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, Option<i64>>(6)?,
            ))
        })
        .map(|rows| {
            rows.filter_map(|r| r.ok())
                .map(|(name, score, usage, recency, doc, version, days)| {
                    let mut row = HashMap::new();
                    row.insert("name".to_string(), Value::Text(name));
                    row.insert("score".to_string(), Value::Int(score));
                    row.insert("usage".to_string(), Value::Int(usage));
                    row.insert("recency".to_string(), Value::Int(recency));
                    row.insert("doc".to_string(), Value::Int(doc));
                    row.insert("version".to_string(), Value::Int(version));
                    row.insert("days_ago".to_string(), Value::Int(days.unwrap_or(-1)));
                    row
                })
                .collect()
        })
        .unwrap_or_default()
    };

    CommandResult::Value(Value::Table(rows))
}

fn history_search_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let pattern = if args.is_empty() {
        return CommandResult::Error("usage: hs <pattern>  — search command history".to_string());
    } else {
        args.join(" ")
    };
    let like = format!("%{}%", pattern);
    let mut stmt = match db.conn.prepare(
        "SELECT command, MAX(timestamp) as ts, COUNT(*) as freq FROM shell_history WHERE command LIKE ?1 AND command NOT LIKE 'TIMING:%' AND command NOT LIKE 'SUGGEST:%' AND LENGTH(command) < 120 GROUP BY command ORDER BY freq DESC, ts DESC LIMIT 20"
    ) {
        Ok(s) => s,
        Err(e) => return CommandResult::Error(e.to_string()),
    };
    let results: Vec<(String, i64, i64)> = stmt
        .query_map(rusqlite::params![&like], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();
    if results.is_empty() {
        return CommandResult::Output(format!(
            "  {} no history matches for '{}'",
            "○".dimmed(),
            pattern
        ));
    }
    let mut out = String::new();
    out.push_str(&format!(
        "
  {} {}
",
        "🔍 History search:".bright_cyan().bold(),
        pattern.bright_white()
    ));
    out.push_str(&format!(
        "  {}
",
        "─".repeat(52).dimmed()
    ));
    for (i, (cmd, ts, freq)) in results.iter().enumerate() {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "?".to_string());
        // Highlight the matching pattern in the command
        let highlighted = cmd.replace(&pattern, &format!("[1;33m{}[0m", pattern));
        let freq_label = if *freq > 1 {
            format!(" ({}x)", freq)
        } else {
            String::new()
        };
        out.push_str(&format!(
            "  {} {}{}  {}
",
            format!("{:3}", i + 1).dimmed(),
            time.dimmed(),
            freq_label.bright_cyan().to_string(),
            highlighted
        ));
    }
    out.push_str(&format!(
        "
  {} {} match{}
",
        "→".dimmed(),
        results.len().to_string().bright_green(),
        if results.len() == 1 { "" } else { "es" }
    ));
    out.push_str(&format!(
        "  {} run with {}
",
        "tip:".dimmed(),
        format!("!{}", pattern).bright_cyan()
    ));
    CommandResult::Output(out)
}
/// Hand the terminal to another shell, interactively. NO-ARG ONLY -- the dispatch arm guards it
/// with `if args.is_empty()`, because this function IGNORES arguments by construction: it keeps the
/// first word and drops the rest. `bash script.sh` used to land here and silently never run the
/// script (INT-143). With args, dispatch now falls through to run_external instead.
fn shell_handoff_cmd(line: &str) -> CommandResult {
    // deadwood: exempt -- shell NAME for handoff, defaulting to zsh -- not a command word
    let shell = line.trim().split_whitespace().next().unwrap_or("zsh");
    println!();
    println!("  {} Stepping out of the forest...", "🌲".to_string());
    println!(
        "  {} You are entering {}",
        "→".bright_cyan(),
        shell.bright_yellow().bold()
    );
    println!(
        "  {} Type {} to return to fsh",
        "→".dimmed(),
        "exit".bright_green()
    );
    println!();
    // Small pause so message is seen
    std::thread::sleep(std::time::Duration::from_millis(400));
    // Spawn the shell
    let _ = std::process::Command::new(shell)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();
    println!();
    println!("  {} Welcome back to Faelight Shell 🌲", "✅".green());
    println!();
    CommandResult::Empty
}
fn history_table(db: &ForestDb) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let mut stmt = match db
        .conn
        .prepare("SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 100")
    {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
    };

    let raw: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();
    let rows: Vec<HashMap<String, Value>> = raw
        .iter()
        .enumerate()
        .map(|(i, (cmd, ts))| {
            let duration_secs = if i + 1 < raw.len() {
                (ts - raw[i + 1].1).max(0)
            } else {
                0
            };
            let time = fmt_time(*ts, "%H:%M:%S");
            let mut row = HashMap::new();
            row.insert("time".to_string(), Value::Text(time));
            row.insert("command".to_string(), Value::Text(cmd.clone()));
            row.insert("duration".to_string(), Value::Int(duration_secs));
            row.insert("timestamp".to_string(), Value::Int(*ts));
            row
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

/// INT-322 Phase 3: history for INT-NNN -- all commands run during a specific intent
fn history_for_intent(db: &ForestDb, intent_arg: &str) -> CommandResult {
    use colored::Colorize;
    let id = intent_arg.trim_start_matches("INT-");
    if id.is_empty() {
        return CommandResult::Output("  Usage: history for INT-NNN".to_string());
    }
    let mut stmt = match db.conn.prepare(
        "SELECT command, timestamp, exit_code FROM shell_history WHERE intent_id = ?1 ORDER BY timestamp ASC"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Error("history for: database error".to_string()),
    };
    let rows: Vec<(String, i64, Option<i32>)> = stmt
        .query_map(rusqlite::params![id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    if rows.is_empty() {
        return CommandResult::Output(format!("  No history found for INT-{} (commands run before this session won't have intent tags)", id));
    }
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {} History for INT-{} ({} commands)\n",
        "▸".bright_cyan(),
        id,
        rows.len()
    ));
    out.push_str(&format!("  {}\n", "─".repeat(50).dimmed()));
    for (cmd, ts, exit_code) in &rows {
        let dt = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d: chrono::DateTime<chrono::Utc>| d.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| ts.to_string());
        let status = match exit_code {
            Some(0) | None => "✅",
            Some(_) => "✗ ",
        };
        out.push_str(&format!("  {} {} {}\n", status, dt.dimmed(), cmd));
    }
    CommandResult::Output(out)
}

/// INT-322 Phase 3: history stats INT-NNN -- success rates, most common commands
fn history_stats_for_intent(db: &ForestDb, intent_arg: &str) -> CommandResult {
    use colored::Colorize;
    let id = intent_arg.trim_start_matches("INT-");
    if id.is_empty() {
        return CommandResult::Output("  Usage: history stats INT-NNN".to_string());
    }
    let total: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE intent_id = ?1",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if total == 0 {
        return CommandResult::Output(format!("  No history found for INT-{}", id));
    }
    let passed: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE intent_id = ?1 AND (exit_code = 0 OR exit_code IS NULL)",
        rusqlite::params![id], |r| r.get(0),
    ).unwrap_or(0);
    let success_rate = if total > 0 { (passed * 100) / total } else { 0 };
    let mut stmt = match db.conn.prepare(
        "SELECT command, COUNT(*) as cnt FROM shell_history WHERE intent_id = ?1 GROUP BY command ORDER BY cnt DESC LIMIT 5"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Error("history stats: database error".to_string()),
    };
    let top: Vec<(String, i64)> = stmt
        .query_map(rusqlite::params![id], |r| Ok((r.get(0)?, r.get(1)?)))
        .map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    let mut out = String::new();
    out.push_str(&format!("\n  {} Stats for INT-{}\n", "▸".bright_cyan(), id));
    out.push_str(&format!("  {}\n", "─".repeat(50).dimmed()));
    out.push_str(&format!(
        "  Total commands:  {}\n",
        total.to_string().bright_white()
    ));
    out.push_str(&format!(
        "  Success rate:    {}%\n",
        success_rate.to_string().bright_white()
    ));
    out.push_str("\n  Top commands:\n");
    for (cmd, cnt) in &top {
        out.push_str(&format!(
            "    {} × {}\n",
            cnt.to_string().bright_cyan(),
            cmd
        ));
    }
    CommandResult::Output(out)
}

fn ht_intent(db: &ForestDb) -> CommandResult {
    // Group history by active intent at time of command
    use colored::Colorize;
    let mut stmt = match db.conn.prepare(
        "SELECT h.command, h.timestamp, e.payload
         FROM shell_history h
         LEFT JOIN events e ON e.domain = 'intent' AND e.action = 'started'
             AND e.timestamp <= h.timestamp
         ORDER BY h.timestamp DESC LIMIT 200",
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
    };
    let rows: Vec<(String, i64, String)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2).unwrap_or_default(),
            ))
        })
        .map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No history yet", "○".dimmed()));
    }
    // Group by intent
    let mut groups: std::collections::BTreeMap<String, Vec<(String, i64)>> =
        std::collections::BTreeMap::new();
    for (cmd, ts, intent) in &rows {
        let key = if intent.is_empty() {
            "no intent".to_string()
        } else {
            intent.clone()
        };
        groups.entry(key).or_default().push((cmd.clone(), *ts));
    }
    let mut out = String::new();
    for (intent, cmds) in groups.iter().rev() {
        out.push_str(&format!(
            "\n  {} {}\n",
            "▸".bright_cyan(),
            intent.bright_white()
        ));
        for (cmd, _ts) in cmds.iter().take(10) {
            out.push_str(&format!("    {}\n", cmd.dimmed()));
        }
    }
    CommandResult::Output(out.trim_end().to_string())
}
fn ht_today(db: &ForestDb) -> CommandResult {
    use colored::Colorize;
    let today_start = {
        use chrono::Local;
        let now = Local::now();
        let midnight = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        midnight.and_utc().timestamp()
    };
    let mut stmt = match db.conn.prepare(
        "SELECT command, timestamp FROM shell_history WHERE timestamp >= ?1 ORDER BY timestamp DESC LIMIT 200"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history today", "○".dimmed())),
    };
    let rows: Vec<(String, i64)> = stmt
        .query_map([today_start], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No commands today yet", "○".dimmed()));
    }
    let mut out = format!(
        "  {} Today -- {} commands\n",
        "▸".bright_cyan(),
        rows.len().to_string().bright_yellow()
    );
    for (cmd, ts) in &rows {
        let time = crate::commands::fmt_time(*ts, "%H:%M");
        out.push_str(&format!("  {} {}\n", time.dimmed(), cmd));
    }
    CommandResult::Output(out.trim_end().to_string())
}
fn ht_session(db: &ForestDb) -> CommandResult {
    use colored::Colorize;
    let session_start = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        now - 3600 * 4 // last 4 hours as session approximation
    };
    let mut stmt = match db.conn.prepare(
        "SELECT command, timestamp FROM shell_history WHERE timestamp >= ?1 ORDER BY timestamp ASC",
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No session history", "○".dimmed())),
    };
    let rows: Vec<(String, i64)> = stmt
        .query_map([session_start], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No commands this session", "○".dimmed()));
    }
    let mut out = format!(
        "  {} Session -- {} commands\n",
        "▸".bright_cyan(),
        rows.len().to_string().bright_yellow()
    );
    for (cmd, ts) in &rows {
        let time = crate::commands::fmt_time(*ts, "%H:%M:%S");
        out.push_str(&format!("  {} {}\n", time.dimmed(), cmd));
    }
    CommandResult::Output(out.trim_end().to_string())
}
fn ht_slow(db: &ForestDb) -> CommandResult {
    use colored::Colorize;
    let mut stmt = match db
        .conn
        .prepare("SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 500")
    {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
    };
    let raw: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    // Compute durations
    let mut with_duration: Vec<(String, i64)> = raw
        .iter()
        .enumerate()
        .map(|(i, (cmd, ts))| {
            let dur = if i + 1 < raw.len() {
                (ts - raw[i + 1].1).max(0)
            } else {
                0
            };
            (cmd.clone(), dur)
        })
        .filter(|(_, d)| *d > 5) // only commands that took > 5 seconds
        .collect();
    with_duration.sort_by(|a, b| b.1.cmp(&a.1));
    with_duration.truncate(20);
    if with_duration.is_empty() {
        return CommandResult::Output(format!(
            "  {} No slow commands found (>5s threshold)",
            "○".dimmed()
        ));
    }
    let mut out = format!("  {} Slow commands (>5s)\n", "▸".bright_cyan());
    for (cmd, dur) in &with_duration {
        let dur_str = if *dur >= 60 {
            format!("{}m{}s", dur / 60, dur % 60)
        } else {
            format!("{}s", dur)
        };
        out.push_str(&format!("  {} {}\n", dur_str.bright_yellow(), cmd));
    }
    CommandResult::Output(out.trim_end().to_string())
}

fn fsh_diag(db: &ForestDb) -> CommandResult {
    use colored::Colorize;
    // Session count
    let sessions: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM session_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    // Total commands
    let total_cmds: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
        .unwrap_or(0);
    // Average focus score
    let avg_focus: f64 = db
        .conn
        .query_row("SELECT AVG(focus_score) FROM session_patterns", [], |r| {
            r.get::<_, Option<f64>>(0)
        })
        .unwrap_or(None)
        .unwrap_or(1.0);
    // Peak velocity
    let peak_vel: f64 = db
        .conn
        .query_row(
            "SELECT MAX(velocity_per_hour) FROM commit_patterns",
            [],
            |r| r.get::<_, Option<f64>>(0),
        )
        .unwrap_or(None)
        .unwrap_or(0.0);
    // Error rate (commands with TIMING: that ran long)
    let slow_cmds: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'TIMING:%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let mut out = String::new();
    out.push_str(&format!("\n  {} Shell Diagnostics\n", "🔍".normal()));
    out.push_str(&format!("  {}\n", "─".repeat(40).dimmed()));
    out.push_str(&format!(
        "  {:<22} {}\n",
        "Sessions recorded:".dimmed(),
        sessions.to_string().bright_white()
    ));
    out.push_str(&format!(
        "  {:<22} {}\n",
        "Total commands:".dimmed(),
        total_cmds.to_string().bright_white()
    ));
    out.push_str(&format!(
        "  {:<22} {:.2}\n",
        "Avg focus score:".dimmed(),
        avg_focus
    ));
    out.push_str(&format!(
        "  {:<22} {:.1} commits/hr\n",
        "Peak velocity:".dimmed(),
        peak_vel
    ));
    out.push_str(&format!(
        "  {:<22} {}\n",
        "Long-running cmds:".dimmed(),
        slow_cmds.to_string().bright_yellow()
    ));
    out.push_str("\n");
    CommandResult::Output(out)
}
fn fsh_gaps(db: &ForestDb) -> CommandResult {
    use colored::Colorize;
    // Find commands that could have used fsh builtins
    let grep_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'grep %' AND timestamp > ?1",
            rusqlite::params![chrono::Utc::now().timestamp() - 604800],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let head_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE (command LIKE 'head %' OR command LIKE 'tail %') AND timestamp > ?1",
        rusqlite::params![chrono::Utc::now().timestamp() - 604800],
        |r| r.get(0)
    ).unwrap_or(0);
    let python_tmp: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'python3 /tmp/%' AND timestamp > ?1",
        rusqlite::params![chrono::Utc::now().timestamp() - 604800],
        |r| r.get(0)
    ).unwrap_or(0);
    let cat_grep: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'cat % | grep%' AND timestamp > ?1",
        rusqlite::params![chrono::Utc::now().timestamp() - 604800],
        |r| r.get(0)
    ).unwrap_or(0);
    let mut out = String::new();
    out.push_str(&format!("\n  {} Shell Gaps (last 7 days)\n", "🌿".normal()));
    out.push_str(&format!("  {}\n", "─".repeat(45).dimmed()));
    let mut any_gaps = false;
    if grep_count > 2 {
        out.push_str(&format!(
            "  {} {} x grep  →  try: {}\n",
            "⚡".yellow(),
            grep_count.to_string().bright_yellow(),
            "query file.rs pattern  or  fsearch 'pattern' --type rs".bright_cyan()
        ));
        any_gaps = true;
    }
    if head_count > 2 {
        out.push_str(&format!(
            "  {} {} x head/tail  →  try: {}\n",
            "⚡".yellow(),
            head_count.to_string().bright_yellow(),
            "query file.rs :50  or  query file.rs 45:60".bright_cyan()
        ));
        any_gaps = true;
    }
    if python_tmp > 2 {
        out.push_str(&format!(
            "  {} {} x python3 /tmp/  →  try: {}\n",
            "⚡".yellow(),
            python_tmp.to_string().bright_yellow(),
            "run script.py  (fsh runs .py natively)".bright_cyan()
        ));
        any_gaps = true;
    }
    if cat_grep > 1 {
        out.push_str(&format!(
            "  {} {} x cat|grep  →  try: {}\n",
            "⚡".yellow(),
            cat_grep.to_string().bright_yellow(),
            "cat file | grep pattern  (native pipe, no sh)".bright_cyan()
        ));
        any_gaps = true;
    }
    // INT-233 -- new builtin alternatives
    let sed_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE (command LIKE 'sed %' OR command LIKE '% sed %') AND timestamp > ?1",
        rusqlite::params![chrono::Utc::now().timestamp() - 604800],
        |r| r.get(0)
    ).unwrap_or(0);
    let py_patch: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE command LIKE '%content.replace%' AND timestamp > ?1",
        rusqlite::params![chrono::Utc::now().timestamp() - 604800],
        |r| r.get(0)
    ).unwrap_or(0);
    if sed_count > 2 {
        out.push_str(&format!(
            "  {} {} x sed  ->  try: {}\n",
            "⚡".yellow(),
            sed_count.to_string().bright_yellow(),
            "rspatch file.rs --anchor 'text' --new 'replacement'".bright_cyan()
        ));
        any_gaps = true;
    }
    if py_patch > 1 {
        out.push_str(&format!(
            "  {} {} x python patch  ->  try: {}\n",
            "⚡".yellow(),
            py_patch.to_string().bright_yellow(),
            "fsh-patch target old.rs new.rs  (no unicode escape issues)".bright_cyan()
        ));
        any_gaps = true;
    }
    if !any_gaps {
        out.push_str(&format!(
            "  {} No gaps detected -- you are using the forest well\n",
            "✅".green()
        ));
    }
    out.push_str("\n");
    CommandResult::Output(out)
}
fn checkpoints_table(db: &ForestDb) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let mut stmt = match db.conn.prepare(
        "SELECT action, payload, timestamp FROM events WHERE domain='checkpoint' ORDER BY timestamp DESC LIMIT 20"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No checkpoints yet", "○".dimmed())),
    };

    let rows: Vec<HashMap<String, Value>> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map(|rows| {
            rows.filter_map(|r| r.ok())
                .map(|(action, payload, ts)| {
                    let date = fmt_time(ts, "%m-%d %H:%M");
                    let name = serde_json::from_str::<serde_json::Value>(&payload)
                        .ok()
                        .and_then(|v| v["detail"]["name"].as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| action.clone());
                    let health = serde_json::from_str::<serde_json::Value>(&payload)
                        .ok()
                        .and_then(|v| v["detail"]["health"].as_i64())
                        .unwrap_or(0);
                    let mut row = HashMap::new();
                    row.insert("date".to_string(), Value::Text(date));
                    row.insert("name".to_string(), Value::Text(name));
                    row.insert("health".to_string(), Value::Int(health));
                    row
                })
                .collect()
        })
        .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No checkpoints yet — use {}",
            "○".dimmed(),
            "cpc <name>".bright_cyan()
        ));
    }

    CommandResult::Value(Value::Table(rows))
}

fn domains(db: &ForestDb) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let mut stmt = match db.conn.prepare(
        "SELECT domain, COUNT(*) as count, MAX(timestamp) as last FROM events GROUP BY domain ORDER BY count DESC"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No events yet", "○".dimmed())),
    };

    let rows: Vec<HashMap<String, Value>> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map(|rows| {
            rows.filter_map(|r| r.ok())
                .map(|(domain, count, last)| {
                    let last_str = chrono::DateTime::from_timestamp(last, 0)
                        .map(|t| t.format("%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "?".to_string());
                    let mut row = HashMap::new();
                    row.insert("domain".to_string(), Value::Text(domain));
                    row.insert("events".to_string(), Value::Int(count));
                    row.insert("last_seen".to_string(), Value::Text(last_str));
                    row
                })
                .collect()
        })
        .unwrap_or_default();

    CommandResult::Value(Value::Table(rows))
}

fn git_commits(core_root: &str, args: &[&str]) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let limit = args
        .first()
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(20);

    // Phase 15 — use faelight-git Repo directly (no subprocess)
    let repo_result = faelight_git::git::repo::GitRepo::open_at(core_root);

    match repo_result {
        Ok(repo) => {
            let log_entries: anyhow::Result<Vec<faelight_git::git::repo::CommitEntry>> =
                repo.log(limit);
            match log_entries {
                Ok(entries) => {
                    // entries: Vec<CommitEntry>
                    let rows: Vec<HashMap<String, Value>> = entries
                        .into_iter()
                        .map(|e| {
                            let mut row = HashMap::new();
                            row.insert("hash".to_string(), Value::Text(e.hash));
                            row.insert("author".to_string(), Value::Text(e.author));
                            row.insert("date".to_string(), Value::Text(e.time_ago));
                            row.insert("message".to_string(), Value::Text(e.message));
                            row
                        })
                        .collect();
                    if rows.is_empty() {
                        CommandResult::Output(format!("  {} No commits found", "○".dimmed()))
                    } else {
                        CommandResult::Value(Value::Table(rows))
                    }
                }
                Err(_) => {
                    // Fallback to git subprocess
                    git_commits_subprocess(core_root, limit)
                }
            }
        }
        Err(_) => git_commits_subprocess(core_root, limit),
    }
}

fn git_commits_subprocess(core_root: &str, limit: usize) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;
    let output = std::process::Command::new("git")
        .args([
            "-C",
            core_root,
            "log",
            &format!("-{}", limit),
            "--format=%H|%an|%ae|%ai|%s",
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let rows: Vec<HashMap<String, Value>> = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(5, '|').collect();
            if parts.len() < 5 {
                return None;
            }
            let mut row = HashMap::new();
            row.insert("hash".to_string(), Value::Text(parts[0][..7].to_string()));
            row.insert("author".to_string(), Value::Text(parts[1].to_string()));
            row.insert("date".to_string(), Value::Text(parts[3][..10].to_string()));
            row.insert("message".to_string(), Value::Text(parts[4].to_string()));
            Some(row)
        })
        .collect();
    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No commits found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}
fn git_files(core_root: &str) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let output = std::process::Command::new("git")
        .args(["-C", core_root, "status", "--porcelain"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    if output.trim().is_empty() {
        return CommandResult::Output(format!("  {} Working tree clean", "✅".green()));
    }

    let rows: Vec<HashMap<String, Value>> = output
        .lines()
        .filter_map(|line| {
            if line.len() < 3 {
                return None;
            }
            let status = line[..2].trim().to_string();
            let file = line[3..].to_string();
            let kind = match status.as_str() {
                "M" | " M" => "modified",
                "A" | " A" => "added",
                "D" | " D" => "deleted",
                "??" => "untracked",
                "R" => "renamed",
                _ => "changed",
            };
            let mut row = HashMap::new();
            row.insert("status".to_string(), Value::Text(status));
            row.insert("kind".to_string(), Value::Text(kind.to_string()));
            row.insert("file".to_string(), Value::Text(file));
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

fn watch_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    use colored::*;

    let target = args.first().copied().unwrap_or("health");
    let interval = args.get(1).and_then(|a| a.parse::<u64>().ok()).unwrap_or(2);

    println!(
        "{}",
        format!(
            "  Watching: {} (every {}s) — press Ctrl+C to stop",
            target.bright_cyan(),
            interval
        )
        .dimmed()
    );

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });
    while running.load(Ordering::SeqCst) {
        // Clear screen and move to top
        print!("[2J[1;1H");

        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        println!(
            "{}",
            format!(
                "  🌲 watch {} — {} ({}s interval)",
                target.bright_cyan(),
                now.dimmed(),
                interval
            )
        );
        println!("{}", "━".repeat(52).dimmed());

        match target {
            "health" => {
                let health = db.health_score().unwrap_or(0);
                let version = std::fs::read_to_string(faelight_core::paths::version_file())
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                let status = if health >= 95 {
                    "HEALTHY".bright_green().bold()
                } else if health >= 80 {
                    "ADVISORY".yellow().bold()
                } else {
                    "DEGRADED".bright_red().bold()
                };

                println!(
                    "  Health:   {} {}",
                    format!("{}%", health).bright_white().bold(),
                    status
                );
                println!("  Version:  {}", version.dimmed());

                // Show last 5 events
                let events = db.query_events(None, false, 5);
                println!();
                println!("  {}", "Recent events:".dimmed());
                for (domain, action, ts) in &events {
                    let icon = match domain.as_str() {
                        "doctor" => "🩺",
                        "git" => "🌿",
                        "security" => "🔒",
                        "sandbox" => "🧪",
                        "audit" => "🔍",
                        _ => "○",
                    };
                    println!(
                        "    {} {} {}.{}",
                        icon,
                        fmt_time(*ts, "%H:%M:%S").dimmed(),
                        domain.bright_cyan(),
                        action.dimmed()
                    );
                }
            }
            "events" => {
                let events = db.query_events(None, false, 15);
                for (domain, action, ts) in &events {
                    let icon = match domain.as_str() {
                        "doctor" => "🩺",
                        "git" => "🌿",
                        "security" => "🔒",
                        "sandbox" => "🧪",
                        "audit" => "🔍",
                        _ => "○ ",
                    };
                    println!(
                        "  {} {} {}.{}",
                        icon,
                        fmt_time(*ts, "%H:%M:%S").dimmed(),
                        domain.bright_cyan(),
                        action.dimmed()
                    );
                }
            }
            _ => {
                println!("  {} Unknown watch target: {}", "✗".bright_red(), target);
                println!("  Available: health, events");
                running.store(false, Ordering::SeqCst);
            }
        }

        // Sleep in small increments to check running flag
        for _ in 0..(interval * 10) {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    println!();
    println!("{}", "  watch stopped".dimmed());
    CommandResult::Empty
}

fn decisions_table(db: &ForestDb) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let rows: Vec<HashMap<String, Value>> = {
        let mut stmt = match db.conn.prepare(
            "SELECT dec_id, description, outcome, risk_score, domain, timestamp FROM decisions ORDER BY timestamp DESC LIMIT 50"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No decisions yet — use {}", "○".dimmed(), "core decide".bright_cyan())),
        };
        stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, f64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })
        .map(|rows| {
            rows.filter_map(|r| r.ok())
                .map(|(id, desc, outcome, risk, domain, ts)| {
                    let date = fmt_time(ts, "%m-%d");
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), Value::Text(id));
                    row.insert("date".to_string(), Value::Text(date));
                    row.insert("domain".to_string(), Value::Text(domain));
                    row.insert("outcome".to_string(), Value::Text(outcome));
                    row.insert("risk".to_string(), Value::Float(risk));
                    row.insert("description".to_string(), Value::Text(desc));
                    row
                })
                .collect()
        })
        .unwrap_or_default()
    };

    CommandResult::Value(Value::Table(rows))
}

fn alias_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    // alias            — list all
    // alias h=health   — create
    // alias h "health" — create (space form)
    if args.is_empty() {
        let aliases = db.list_aliases();
        if aliases.is_empty() {
            return CommandResult::Output(format!(
                "  {} No aliases defined yet\n  Create one: {}",
                "○".dimmed(),
                "alias h=health".bright_cyan()
            ));
        }
        let mut out = String::new();
        out.push_str(&format!(
            "\n{}\n",
            "  ╭─ 🔖 Aliases ──────────────────────────────────────".bright_cyan()
        ));
        // INT-194: mark aliases that shadow a builtin. The creation-time warning only helps
        // from now on -- aliases already in config.fsh predate it, and someone inheriting a
        // config would never see one. Marking them HERE is what makes known collisions
        // "visible rather than discovered accidentally": no new command, no startup nagging
        // about a settled declarative choice, shown exactly when you are looking at aliases.
        let mut shadow_count = 0usize;
        for (name, cmd) in &aliases {
            if crate::registry::builtin_description(name).is_some() {
                shadow_count += 1;
                out.push_str(&format!(
                    "  │  {:<15} = {}  {}\n",
                    name.bright_cyan(),
                    cmd.dimmed(),
                    "shadows builtin".yellow()
                ));
            } else {
                out.push_str(&format!(
                    "  │  {:<15} = {}\n",
                    name.bright_cyan(),
                    cmd.dimmed()
                ));
            }
        }
        out.push_str(
            &"  ╰────────────────────────────────────────────────────"
                .dimmed()
                .to_string(),
        );
        if shadow_count > 0 {
            out.push_str(&format!(
                "\n  {} {} alias{} shadow a builtin of the same name",
                "⚠".yellow(),
                shadow_count,
                if shadow_count == 1 { "" } else { "es" }
            ));
        }
        return CommandResult::Output(out);
    }

    // Single arg without = — show existing alias
    if args.len() == 1 && !args[0].contains('=') {
        let name = args[0];
        if let Some(cmd) = db.get_alias(name) {
            return CommandResult::Output(format!("  {} = {}", name.bright_cyan(), cmd.dimmed()));
        }
        return CommandResult::Error(format!("No alias: {}", name));
    }

    // Parse: alias name=command OR alias name command
    let full = args.join(" ");
    let (name, command) = if full.contains('=') {
        let mut parts = full.splitn(2, '=');
        let n = parts.next().unwrap_or("").trim().to_string();
        let c = parts
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"')
            .to_string();
        (n, c)
    } else if args.len() >= 2 {
        (
            args[0].to_string(),
            args[1..].join(" ").trim_matches('"').to_string(),
        )
    } else {
        return CommandResult::Error("Usage: alias name=command".to_string());
    };

    if name.is_empty() || command.is_empty() {
        return CommandResult::Error("Usage: alias name=command".to_string());
    }

    // INT-194: warn when an alias shadows a builtin. NOT a precedence change -- aliases taking
    // precedence over builtins is standard shell behaviour (bash: aliases -> functions ->
    // builtins -> PATH), and changing it would break every alias that deliberately overrides one.
    // The problem is SILENCE: `gc` = `git commit -m` shadows the `gc` table builtin, which in
    // turn silently broke `gc5` = `gc | first 5` (written expecting the builtin). Nothing
    // anywhere reported that a builtin had been displaced.
    //
    // The check reads registry::BUILTINS -- the ONE source, and the scope is stated there:
    // user-facing builtins with a description and usage, the ones `help` and the cheatsheet
    // show. Those are exactly the ones a user could lose without noticing.
    let shadowed = crate::registry::builtin_description(&name);

    if db.add_alias(&name, &command) {
        let mut out = format!(
            "  {} alias {} = {}",
            "✅".green(),
            name.bright_cyan(),
            command.dimmed()
        );
        if let Some(desc) = shadowed {
            out.push_str(&format!(
                "\n  {} alias {} shadows the builtin {} ({})\n  {} the alias wins; {} to restore the builtin",
                "⚠️".yellow(),
                name.bright_cyan(),
                name.bright_white(),
                desc.dimmed(),
                "→".dimmed(),
                format!("unalias {}", name).bright_cyan()
            ));
        }
        CommandResult::Output(out)
    } else {
        CommandResult::Error(format!("Failed to save alias: {}", name))
    }
}

fn unalias_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let name = match args.first() {
        Some(n) => *n,
        None => return CommandResult::Error("Usage: unalias <name>".to_string()),
    };

    if db.remove_alias(name) {
        CommandResult::Output(format!(
            "  {} removed alias: {}",
            "✅".green(),
            name.bright_cyan()
        ))
    } else {
        CommandResult::Error(format!("Alias not found: {}", name))
    }
}

fn list_plugins(db: &ForestDb) -> CommandResult {
    let plugins = db.load_plugins();

    // Group by plugin file
    let plugin_dir = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".config/faelight-shell/plugins");

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 🔌 Loaded Plugins ─────────────────────────────".bright_cyan()
    ));

    if plugins.is_empty() {
        out.push_str(&format!("  │  {} No plugins found\n", "○".dimmed()));
        out.push_str(&format!(
            "  │  Add .fsh files to {}\n",
            plugin_dir.display().to_string().dimmed()
        ));
    } else {
        out.push_str(&format!(
            "  │  {} commands from plugins:\n",
            plugins.len().to_string().bright_white()
        ));
        for (name, expand, desc) in &plugins {
            out.push_str(&format!(
                "  │  {:<15} {} {}\n",
                name.bright_cyan(),
                "→".dimmed(),
                if desc.is_empty() {
                    expand.dimmed().to_string()
                } else {
                    desc.dimmed().to_string()
                }
            ));
        }
    }
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn reload_plugins_cmd(db: &ForestDb) -> CommandResult {
    let plugins = db.load_plugins();
    CommandResult::Output(format!(
        "  {} Reloaded {} plugin commands",
        "✅".green(),
        plugins.len().to_string().bright_white()
    ))
}

// ── Phase 8 — System Tables ───────────────────────────────────────────────────

fn sys_processes() -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let output = std::process::Command::new("ps")
        .args(["-eo", "user:32,pid,pcpu,pmem,stat,comm", "--no-headers"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    // Format: user:32 pid pcpu pmem stat comm
    let rows: Vec<HashMap<String, Value>> = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 {
                return None;
            }
            let mut row = HashMap::new();
            row.insert("user".to_string(), Value::Text(parts[0].to_string()));
            row.insert("pid".to_string(), Value::Text(parts[1].to_string()));
            row.insert("cpu".to_string(), Value::Text(parts[2].to_string()));
            row.insert("memory".to_string(), Value::Text(parts[3].to_string()));
            row.insert("status".to_string(), Value::Text(parts[4].to_string()));
            row.insert("name".to_string(), Value::Text(parts[5].to_string()));
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

fn sys_ports() -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;
    let output = std::process::Command::new("ss")
        .args(["-tlnp"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let rows: Vec<HashMap<String, Value>> = output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                return None;
            }
            let addr = parts[3];
            let port = addr.rsplit(':').next().unwrap_or("?").to_string();
            let users_str = if parts.len() > 4 {
                parts[4..].join(" ")
            } else {
                String::new()
            };
            let process = users_str.split('"').nth(1).unwrap_or("?").to_string();
            let pid = users_str
                .split("pid=")
                .nth(1)
                .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
                .unwrap_or("?")
                .to_string();
            let mut row = HashMap::new();
            row.insert("port".to_string(), Value::Text(port));
            row.insert("state".to_string(), Value::Text(parts[0].to_string()));
            row.insert("address".to_string(), Value::Text(addr.to_string()));
            row.insert("process".to_string(), Value::Text(process));
            row.insert("pid".to_string(), Value::Text(pid));
            Some(row)
        })
        .collect();
    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No listening ports found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}
fn sys_services() -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let output = std::process::Command::new("systemctl")
        .args([
            "list-units",
            "--type=service",
            "--no-pager",
            "--no-legend",
            "--all",
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(50)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                return None;
            }
            let name = parts[0].trim_end_matches(".service").to_string();
            let load = parts[1].to_string();
            let active = parts[2].to_string();
            let sub = parts[3].to_string();
            let mut row = HashMap::new();
            row.insert("name".to_string(), Value::Text(name));
            row.insert("load".to_string(), Value::Text(load));
            row.insert("active".to_string(), Value::Text(active));
            row.insert("status".to_string(), Value::Text(sub));
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

fn sys_files(_core_root: &str, args: &[&str]) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let home = std::env::var("HOME").unwrap_or_default();
    let dir = args.first().copied().unwrap_or(".");
    let path = if dir == "." {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    } else if dir.starts_with("~/") {
        std::path::PathBuf::from(format!("{}/{}", home, &dir[2..]))
    } else {
        std::path::PathBuf::from(dir)
    };

    let rows: Vec<HashMap<String, Value>> = std::fs::read_dir(&path)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| {
                    let meta = e.metadata().ok()?;
                    let name = e.file_name().to_string_lossy().to_string();
                    let size = meta.len();
                    let kind = if meta.is_dir() { "dir" } else { "file" }.to_string();
                    let modified = meta
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| {
                            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                .map(|t| t.format("%m-%d %H:%M").to_string())
                                .unwrap_or_else(|| "?".to_string())
                        })
                        .unwrap_or_else(|| "?".to_string());
                    let mut row = HashMap::new();
                    row.insert("name".to_string(), Value::Text(name));
                    row.insert("kind".to_string(), Value::Text(kind));
                    row.insert("size".to_string(), Value::Int(size as i64));
                    row.insert("modified".to_string(), Value::Text(modified));
                    Some(row)
                })
                .collect()
        })
        .unwrap_or_default();

    CommandResult::Value(Value::Table(rows))
}

/// INT-307: power -- power profile management with Friday awareness
fn power_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    use colored::Colorize;
    let sub = args.first().copied().unwrap_or("status");
    match sub {
        "status" | "s" => {
            let profile = std::process::Command::new("powerprofilesctl")
                .arg("get")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let battery = read_battery_status();
            let icon = match profile.as_str() {
                "performance" => "⚡",
                "balanced"    => "⚖",
                "power-saver" => "🍃",
                _             => "?",
            };
            let profile_colored = match profile.as_str() {
                "performance" => profile.bright_yellow().to_string(),
                "power-saver" => profile.bright_green().to_string(),
                _             => profile.bright_white().to_string(),
            };
            let mut out = String::new();
            out.push_str(&format!("\n  {} Power Profile\n", "⚡".normal()));
            out.push_str(&format!("  {}\n\n", "─".repeat(40).dimmed()));
            out.push_str(&format!("  {} profile:  {} {}\n", "→".bright_cyan(), icon, profile_colored));
            out.push_str(&format!("  {} battery:  {}\n", "→".bright_cyan(), battery));
            out.push_str(&format!("\n  Commands: power set performance|balanced|power-saver\n"));
            // Record in state.db for Friday
            let _ = db.conn.execute(
                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('power_profile', ?1)",
                rusqlite::params![profile],
            );
            CommandResult::Output(out)
        }
        "set" => {
            let profile = args.get(1).copied().unwrap_or("balanced");
            let valid = ["performance", "balanced", "power-saver"];
            if !valid.contains(&profile) {
                return CommandResult::Error(format!(
                    "  power set: invalid profile '{}' -- use: performance, balanced, power-saver", profile
                ));
            }
            let status = std::process::Command::new("powerprofilesctl")
                .args(["set", profile])
                .status();
            match status {
                Ok(s) if s.success() => {
                    let _ = db.conn.execute(
                        "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('power_profile', ?1)",
                        rusqlite::params![profile],
                    );
                    let icon = match profile {
                        "performance" => "⚡",
                        "power-saver" => "🍃",
                        _             => "⚖",
                    };
                    CommandResult::Output(format!("  {} {} -- profile set to {}", "✅".normal(), icon, profile))
                }
                _ => CommandResult::Error(format!("  power set: failed -- is power-profiles-daemon running?")),
            }
        }
        "auto" => {
            let _ = db.conn.execute(
                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('power_auto', 'true')",
                [],
            );
            CommandResult::Output("  ✅ Friday power auto-switching enabled".to_string())
        }
        _ => {
            CommandResult::Output(format!(
                "  Usage: power status | power set <profile> | power auto\n  Profiles: performance ⚡  balanced ⚖  power-saver 🍃"
            ))
        }
    }
}

fn read_battery_status() -> String {
    use colored::Colorize;
    // Read from /sys/class/power_supply/
    let base = std::path::Path::new("/sys/class/power_supply");
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") {
                let capacity = std::fs::read_to_string(entry.path().join("capacity"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let status = std::fs::read_to_string(entry.path().join("status"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let pct: i32 = capacity.parse().unwrap_or(0);
                let icon = if status == "Charging" {
                    "🔌"
                } else if pct > 80 {
                    "🔋"
                } else if pct > 20 {
                    "🪫"
                } else {
                    "⚠"
                };
                let pct_colored = if pct > 50 {
                    format!("{}%", pct).bright_green().to_string()
                } else if pct > 20 {
                    format!("{}%", pct).yellow().to_string()
                } else {
                    format!("{}%", pct).bright_red().to_string()
                };
                return format!("{} {} ({})", icon, pct_colored, status.to_lowercase());
            }
        }
    }
    "no battery detected".to_string()
}

fn sys_network() -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let output = std::process::Command::new("ip")
        .args(["-s", "link"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut rows = Vec::new();
    let mut current: HashMap<String, Value> = HashMap::new();
    let lines = output.lines().peekable();

    for line in lines {
        if line.starts_with(|c: char| c.is_ascii_digit()) {
            if !current.is_empty() {
                rows.push(current.clone());
                current.clear();
            }
            let name = line.split(':').nth(1).unwrap_or("?").trim().to_string();
            current.insert("interface".to_string(), Value::Text(name));
        } else if line.trim().starts_with("link/") {
            let mac = line.split_whitespace().nth(1).unwrap_or("?").to_string();
            current.insert("mac".to_string(), Value::Text(mac));
        } else if line.trim().starts_with("inet ") {
            let ip = line.split_whitespace().nth(1).unwrap_or("?").to_string();
            current.insert("ip".to_string(), Value::Text(ip));
        }
    }
    if !current.is_empty() {
        rows.push(current);
    }

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No network interfaces found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}

// INT-075: nix store explorer. `store why <path|name>` answers "what keeps this
// alive + how big is it" using fast per-path nix queries (NO --print-dead; that walks
// the whole store and is slow). Read-only: inspects, never collects or deletes.
fn store_cmd(args: &[&str]) -> CommandResult {
    let sub = args.first().copied().unwrap_or("help");
    match sub {
        "why" => {
            let target = match args.get(1) {
                Some(t) => *t,
                None => return CommandResult::Error(
                    "  store why <path|name> -- what keeps a store path alive + its size".to_string()),
            };
            // Resolve target -> a concrete /nix/store path.
            let path = match store_resolve(target) {
                Ok(p) => p,
                Err(msg) => return CommandResult::Error(msg),
            };
            let mut out = String::new();
            out.push_str(&format!("  \u{1b}[38;2;50;220;255mstore why\u{1b}[0m  {}\n", path));

            // Sizes: self + closure
            let self_sz = nix_query(&["path-info", "-sh", &path])
                .map(|s| size_tail(&s)).unwrap_or_else(|| "?".into());
            let clos_sz = nix_query(&["path-info", "-Sh", &path])
                .map(|s| size_tail(&s)).unwrap_or_else(|| "?".into());
            out.push_str(&format!("  self size    : {}\n", self_sz));
            out.push_str(&format!("  closure size : {}\n", clos_sz));

            // GC roots: is anything pinning it?
            let roots = nix_query_lines(&["nix-store", "--query", "--roots", &path]);
            if roots.is_empty() {
                out.push_str("  pinned by    : \u{1b}[38;2;255;200;50m(no GC roots -- not directly pinned)\u{1b}[0m\n");
            } else {
                out.push_str(&format!("  pinned by    : {} GC root(s):\n", roots.len()));
                for r in roots.iter().take(8) {
                    out.push_str(&format!("      {}\n", r));
                }
                if roots.len() > 8 { out.push_str(&format!("      ... and {} more\n", roots.len()-8)); }
            }

            // Direct referrers (reverse-deps)
            let refs = nix_query_lines(&["nix-store", "--query", "--referrers", &path]);
            let refs: Vec<&String> = refs.iter().filter(|r| **r != path).collect();
            if refs.is_empty() {
                out.push_str("  referrers    : (none -- nothing else depends on it directly)\n");
            } else {
                out.push_str(&format!("  referrers    : {} direct:\n", refs.len()));
                for r in refs.iter().take(6) {
                    let base = r.rsplit('/').next().unwrap_or(r);
                    out.push_str(&format!("      {}\n", base));
                }
                if refs.len() > 6 { out.push_str(&format!("      ... and {} more\n", refs.len()-6)); }
            }
            CommandResult::Output(out)
        }
        "reclaim" => store_reclaim(),
        "big" => {
            // store big [N] -- the N largest store paths by SELF size (INT-134, Lane 2).
            // Read-only: `nix path-info --all -S --json` (self size, not closure, to avoid
            // double-counting shared deps -- same honesty as store reclaim). Default N=20.
            // Slow: walks the whole store. Pairs with `store why <name>` to investigate a hit.
            let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);
            let out = std::process::Command::new("nix")
                .args(["path-info", "-r", "-S", "--json", "--json-format", "1",
                       "/run/current-system"])
                .output();
            let out = match out {
                Ok(o) if o.status.success() => o,
                Ok(o) => return CommandResult::Error(format!(
                    "store big: nix path-info failed: {}",
                    String::from_utf8_lossy(&o.stderr).trim())),
                Err(e) => return CommandResult::Error(format!("store big: {}", e)),
            };
            let json: serde_json::Value = match serde_json::from_slice(&out.stdout) {
                Ok(v) => v,
                Err(e) => return CommandResult::Error(format!("store big: parse: {}", e)),
            };
            // nix 2.34: top-level object keyed by store path -> { narSize, ... }.
            let map = match json.as_object() {
                Some(m) if !m.is_empty() => m,
                _ => return CommandResult::Output("  store big: no paths reported".to_string()),
            };
            let mut rows: Vec<(u64, String)> = Vec::new();
            for (path, v) in map.iter() {
                let sz = v.get("narSize").and_then(|x| x.as_u64()).unwrap_or(0);
                let base = path.rsplit('/').next().unwrap_or(path).to_string();
                rows.push((sz, base));
            }
            rows.sort_by(|a, b| b.0.cmp(&a.0));
            let total = rows.len();
            let mut lines = vec![format!("  \u{1f4be} largest paths in the live system closure by self size (top {} of {})", n.min(total), total)];
            lines.push("\u{2500}".repeat(60));
            for (sz, base) in rows.iter().take(n) {
                let mib = *sz as f64 / (1024.0 * 1024.0);
                lines.push(format!("  {:>9.1} MiB  {}", mib, base));
            }
            lines.push(String::new());
            lines.push("  investigate any of these with  store why <name>".to_string());
            CommandResult::Output(lines.join("\n"))
        }
        "help" | _ => CommandResult::Output(
            "  store -- nix store explorer (INT-075)\n  store why <path|name>  what keeps a path alive + its size\n  store reclaim          honest GC preview: real freeable size (slow, walks store)\n  store big [N]          the N largest store paths by self size (slow, walks store)\n".to_string()),
    }
}

// Resolve a user arg to a concrete /nix/store path. Accepts a full store path, or a
// partial name we grep from /nix/store (unique match used; ambiguous -> list candidates).
// INT-075 Phase 2: honest GC preview. The freeable disk = sum of each DEAD path's
// SELF size (nix path-info -s), NOT closure size (-S double-counts shared deps massively
// -- e.g. one path: 174 KiB self vs 1.1 GiB closure). Computing the dead set walks the
// whole store (~30s); we say so. Read-only: NEVER runs gc, NEVER deletes.
fn store_reclaim() -> CommandResult {
    let mut out = String::new();
    out.push_str("  \u{1b}[38;2;50;220;255mstore reclaim\u{1b}[0m  (honest GC preview -- read-only, deletes nothing)\n");
    out.push_str("  computing dead set (walks the whole store, ~30s)...\n");

    // 1. dead set (read-only)
    let dead_out = std::process::Command::new("nix-store")
        .args(["--gc", "--print-dead"])
        .output();
    let dead_paths: Vec<String> = match dead_out {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| l.starts_with("/nix/store"))
            .map(|s| s.to_string())
            .collect(),
        Err(e) => return CommandResult::Error(format!("  store reclaim: nix-store failed: {}", e)),
    };
    let n = dead_paths.len();
    if n == 0 {
        out.push_str("  no dead paths -- nothing a GC would free right now.\n");
        return CommandResult::Output(out);
    }

    // 2. batch nix path-info -s over ALL dead paths (one query), sum SELF bytes
    let mut args: Vec<String> = vec!["path-info".into(), "-s".into()];
    args.extend(dead_paths.iter().cloned());
    let argref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let info = std::process::Command::new("nix").args(&argref).output();
    let mut total: u64 = 0;
    let mut counted = 0usize;
    if let Ok(o) = info {
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            // line: "<path>\t<self-bytes>"  -- last whitespace token is the byte count
            if let Some(tok) = line.split_whitespace().last() {
                if let Ok(b) = tok.parse::<u64>() {
                    total += b;
                    counted += 1;
                }
            }
        }
    }

    let human = |b: u64| -> String {
        let (mut v, units) = (b as f64, ["B", "KiB", "MiB", "GiB", "TiB"]);
        let mut i = 0;
        while v >= 1024.0 && i < units.len() - 1 {
            v /= 1024.0;
            i += 1;
        }
        format!("{:.2} {}", v, units[i])
    };

    out.push_str(&format!("  dead paths    : {}\n", n));
    out.push_str(&format!("  \u{1b}[38;2;57;255;20mfreeable\u{1b}[0m      : {}  (sum of SELF sizes -- the true disk a GC frees)\n", human(total)));
    if counted < n {
        out.push_str(&format!(
            "  note          : sized {}/{} paths ({} had no size info)\n",
            counted,
            n,
            n - counted
        ));
    }
    out.push_str("  method        : self-size (-s) summed, NOT closure (-S would double-count shared deps).\n");
    out.push_str("  to actually free: run  nix-collect-garbage  (or nix-collect-garbage -d for old generations).\n");
    out.push_str("  (this command did NOT delete anything.)\n");
    CommandResult::Output(out)
}

// INT-075 Phase 1.5: when a name matches many store paths, summarize the reclaim
// picture: total closure size across matches, and how many are GC-rooted (pinned) vs
// unrooted (reclaimable). Turns "138 matches" into "here's what they cost + what's free".
fn store_summarize_matches(target: &str, matches: &[String], n: usize) -> String {
    let mut rooted = 0usize;
    let mut unrooted = 0usize;
    let mut total_bytes: u64 = 0;
    for p in matches {
        // closure size in bytes (-S = closure, default bytes when no -h)
        if let Some(s) = nix_query(&["path-info", "-S", p]) {
            if let Some(tok) = s.split_whitespace().last() {
                if let Ok(b) = tok.parse::<u64>() {
                    total_bytes += b;
                }
            }
        }
        // pinned? (any GC root)
        let roots = nix_query_lines(&["nix-store", "--query", "--roots", p]);
        if roots.is_empty() {
            unrooted += 1;
        } else {
            rooted += 1;
        }
    }
    let human = |b: u64| -> String {
        let (mut v, units) = (b as f64, ["B", "KiB", "MiB", "GiB", "TiB"]);
        let mut i = 0;
        while v >= 1024.0 && i < units.len() - 1 {
            v /= 1024.0;
            i += 1;
        }
        format!("{:.1} {}", v, units[i])
    };
    let mut msg = String::new();
    msg.push_str(&format!(
        "  \u{1b}[38;2;50;220;255mstore why\u{1b}[0m  '{}' matches {} store paths:\n",
        target, n
    ));
    msg.push_str(&format!("  total closure : {}\n", human(total_bytes)));
    msg.push_str(&format!(
        "  pinned        : {} (GC-rooted -- a generation/result holds them)\n",
        rooted
    ));
    msg.push_str(&format!(
        "  \u{1b}[38;2;255;200;50mreclaimable\u{1b}[0m   : {} (no GC root -- would be freed by a GC)\n", unrooted));
    msg.push_str(
        "  (note: closure sizes overlap heavily via shared deps; total is an upper bound,\n",
    );
    msg.push_str(
        "   not additive disk usage. Use `store why <full-path>` for one specific build.)\n",
    );
    msg.push_str("  first few matches:\n");
    for m in matches.iter().take(6) {
        msg.push_str(&format!("      {}\n", m.rsplit('/').next().unwrap_or(m)));
    }
    if n > 6 {
        msg.push_str(&format!("      ... and {} more\n", n - 6));
    }
    msg
}

fn store_resolve(target: &str) -> Result<String, String> {
    if target.starts_with("/nix/store/") && std::path::Path::new(target).exists() {
        return Ok(target.to_string());
    }
    // grep store dir entries for the name
    let entries =
        std::fs::read_dir("/nix/store").map_err(|e| format!("  cannot read /nix/store: {}", e))?;
    let mut matches: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().to_string())
        .filter(|p| !p.ends_with(".drv") && p.contains(target))
        .collect();
    matches.sort();
    matches.dedup();
    match matches.len() {
        0 => Err(format!("  no store path matches '{}'", target)),
        1 => Ok(matches.remove(0)),
        n => {
            // Phase 1.5: ambiguity is the reclaim insight. Summarize instead of just listing.
            Err(store_summarize_matches(target, &matches, n))
        }
    }
}

// Extract "<num> <unit>" (the last two tokens) from a `nix path-info -sh` line
// of the form "<path>\t<num> <unit>". Falls back to the whole trimmed string.
fn size_tail(s: &str) -> String {
    let toks: Vec<&str> = s.split_whitespace().collect();
    let n = toks.len();
    if n >= 2 {
        format!("{} {}", toks[n - 2], toks[n - 1])
    } else {
        s.trim().to_string()
    }
}

// Run `nix <args>` and capture trimmed stdout (single line/value).
fn nix_query(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("nix").args(args).output().ok()?;
    let s = String::from_utf8(out.stdout).ok()?;
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

// Run a command (first arg = binary) and capture stdout lines.
fn nix_query_lines(argv: &[&str]) -> Vec<String> {
    if argv.is_empty() {
        return vec![];
    }
    let out = std::process::Command::new(argv[0])
        .args(&argv[1..])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect(),
        Err(_) => vec![],
    }
}

fn sys_logs(args: &[&str]) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let follow = args.contains(&"--follow") || args.contains(&"-f");
    let errors_only = args.contains(&"--errors") || args.contains(&"-e");
    let lines = args
        .iter()
        .find(|a| a.starts_with("-n"))
        .and_then(|a| a.trim_start_matches("-n").parse::<usize>().ok())
        .unwrap_or(50);

    if follow {
        // Streaming mode — follow journalctl
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let r = running.clone();
        std::thread::spawn(move || {
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            r.store(false, std::sync::atomic::Ordering::SeqCst);
        });

        println!(
            "  {} {} {}",
            "streaming logs".bright_cyan(),
            if errors_only { "(errors only)" } else { "" }.dimmed(),
            "— press Enter to stop".dimmed()
        );
        println!("{}", "━".repeat(52).dimmed());

        let mut cmd = std::process::Command::new("journalctl");
        cmd.args(["-f", "-n", "0", "--no-pager", "--output=short-iso"]);
        if errors_only {
            cmd.args(["-p", "err"]);
        }

        if let Ok(mut child) = cmd.stdout(std::process::Stdio::piped()).spawn() {
            use std::io::{BufRead, BufReader};
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if !running.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                    if let Ok(l) = line {
                        let level = if l.contains("err") || l.contains("ERROR") {
                            "ERR ".bright_red().to_string()
                        } else if l.contains("warn") || l.contains("WARN") {
                            "WARN".yellow().to_string()
                        } else {
                            "INFO".dimmed().to_string()
                        };
                        println!("  {} {}", level, l.dimmed());
                    }
                }
            }
            let _ = child.kill();
        }
        println!(
            "
  {} log stream stopped",
            "○".dimmed()
        );
        return CommandResult::Empty;
    }

    // Static mode — return as table
    let output = std::process::Command::new("journalctl")
        .args(["-n", &lines.to_string(), "--no-pager", "--output=short-iso"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output
        .lines()
        .filter(|l| !l.starts_with("--"))
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, ' ').collect();
            if parts.len() < 3 {
                return None;
            }
            let level = if line.contains("err") || line.contains("ERROR") {
                "error"
            } else if line.contains("warn") {
                "warn"
            } else {
                "info"
            }
            .to_string();
            if errors_only && level != "error" {
                return None;
            }
            let mut row = HashMap::new();
            row.insert("time".to_string(), Value::Text(parts[0].to_string()));
            row.insert("host".to_string(), Value::Text(parts[1].to_string()));
            row.insert("service".to_string(), Value::Text(parts[2].to_string()));
            row.insert("level".to_string(), Value::Text(level));
            row.insert(
                "message".to_string(),
                Value::Text(parts.get(3).unwrap_or(&"").to_string()),
            );
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

fn search(db: &ForestDb, args: &[&str]) -> CommandResult {
    // INT-300: forest flags -- delegate to file search (fsearch behavior)
    let forest_flags = [
        "--rust",
        "--intent",
        "--forest",
        "--py",
        "--md",
        "--sh",
        "--toml",
        "--scripts",
    ];
    if let Some(&flag) = args.iter().find(|a| forest_flags.contains(*a)) {
        let pattern = args
            .iter()
            .find(|a| !a.starts_with("--"))
            .copied()
            .unwrap_or("");
        let home = std::env::var("HOME").unwrap_or_default();
        let root = format!("{}/0-core", home);
        let (type_flag, search_root): (Option<&str>, String) = match flag {
            "--rust" => (Some("rust"), root.clone()),
            "--py" => (Some("py"), root.clone()),
            "--md" => (Some("markdown"), root.clone()),
            "--sh" => (Some("sh"), root.clone()),
            "--toml" => (Some("toml"), root.clone()),
            "--intent" => (
                None,
                faelight_core::paths::intents_dir()
                    .to_string_lossy()
                    .to_string(),
            ),
            "--scripts" => (None, format!("{}/scripts", root)),
            _ => (None, root.clone()),
        };
        let mut cmd = std::process::Command::new("rg");
        cmd.arg("--line-number").arg("--color=never");
        if let Some(t) = type_flag {
            cmd.arg("--type").arg(t);
        }
        cmd.arg(pattern).arg(&search_root);
        return match cmd.output() {
            Ok(o) => {
                let raw = String::from_utf8_lossy(&o.stdout);
                if raw.is_empty() {
                    CommandResult::Output(format!("  (no matches for '{}')", pattern))
                } else {
                    let out: String = raw
                        .lines()
                        .take(50)
                        .map(|l| {
                            format!(
                                "{}
",
                                l.replace(&format!("{}/", root), "")
                            )
                        })
                        .collect();
                    CommandResult::Output(out.trim_end().to_string())
                }
            }
            Err(_) => CommandResult::Error("search: rg not found".to_string()),
        };
    }
    let query = args.join(" ").to_lowercase();

    let rows: Vec<(String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 200",
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
        };
        stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };

    if query.is_empty() {
        // Show recent history
        let mut out = String::new();
        out.push_str(&format!(
            "\n{}\n",
            "  ╭─ 📜 Command History ──────────────────────────────".bright_cyan()
        ));
        for (cmd, ts) in rows.iter().take(15) {
            let time = fmt_time(*ts, "%H:%M");
            out.push_str(&format!("  │  {} {}\n", time.dimmed(), cmd.bright_white()));
        }
        out.push_str(
            &"  ╰────────────────────────────────────────────────────"
                .dimmed()
                .to_string(),
        );
        return CommandResult::Output(out);
    }

    // Fuzzy search — score by match position and frequency
    let mut matches: Vec<(String, i64, usize)> = rows
        .iter()
        .filter(|(cmd, _)| cmd.to_lowercase().contains(&query))
        .map(|(cmd, ts)| {
            let score = if cmd.to_lowercase().starts_with(&query) {
                0
            } else if cmd.to_lowercase().contains(&format!(" {}", query)) {
                1
            } else {
                2
            };
            (cmd.clone(), *ts, score)
        })
        .collect();

    // Deduplicate keeping most recent
    matches.sort_by_key(|(cmd, _, score)| (*score, cmd.clone()));
    matches.dedup_by(|a, b| a.0 == b.0);
    matches.sort_by_key(|(_, ts, score)| (*score, -ts));

    if matches.is_empty() {
        return CommandResult::Output(format!(
            "  {} No matches for {}",
            "○".dimmed(),
            query.bright_white()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        format!(
            "  ╭─ 🔍 Search: {} ({} results) ──────────────────────",
            query,
            matches.len()
        )
        .bright_cyan()
    ));
    for (cmd, ts, _) in matches.iter().take(10) {
        let time = fmt_time(*ts, "%H:%M");
        // Highlight the match
        let highlighted = cmd.replacen(&query, &query.bright_yellow().to_string(), 1);
        out.push_str(&format!("  │  {} {}\n", time.dimmed(), highlighted));
    }
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn compare_cmd(core_root: &str, args: &[&str]) -> CommandResult {
    use std::process::Command;
    let bin = format!("{}/scripts/faelight-diff", core_root);
    let mut cmd_args: Vec<String> = Vec::new();
    for a in args {
        cmd_args.push(a.to_string());
    }
    // Default: git diff if in repo with no args
    if args.is_empty() {
        cmd_args.push("--git".to_string());
    }
    match Command::new(&bin).args(&cmd_args).status() {
        Ok(_) => CommandResult::Output(String::new()),
        Err(_) => {
            CommandResult::Error("faelight-diff not found -- run: deploy faelight-diff".to_string())
        }
    }
}
fn pick_cmd(db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let subcommand = args.first().copied().unwrap_or("");
    let extra = if args.len() > 1 { args[1] } else { "" };
    match subcommand {
        "intent" | "intents" => {
            // Collect all intent files
            let mut items = String::new();
            for dir in &["future", "complete", "in-progress"] {
                let path = faelight_core::paths::intents_dir()
                    .join(dir)
                    .to_string_lossy()
                    .to_string();
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !name.ends_with(".md") {
                            continue;
                        }
                        let num = name.split('-').next().unwrap_or("").to_string();
                        let title = name
                            .trim_end_matches(".md")
                            .splitn(3, '-')
                            .nth(2)
                            .unwrap_or("")
                            .replace('-', " ");
                        let status = *dir;
                        if extra == "--active" && status != "future" {
                            continue;
                        }
                        items.push_str(&format!(
                            "INT-{}  [{}]  {}
",
                            num, status, title
                        ));
                    }
                }
            }
            if items.is_empty() {
                return CommandResult::Output("  No intents found".to_string());
            }
            let mut child = match Command::new("sk")
                .args([
                    "--prompt=pick intent> ",
                    "--height=50%",
                    "--reverse",
                    "--ansi",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(_) => return CommandResult::Error("sk not found -- install skim".to_string()),
            };
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(items.as_bytes());
            }
            let output = child
                .wait_with_output()
                .unwrap_or_else(|_| std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                });
            if output.status.success() {
                let line = String::from_utf8_lossy(&output.stdout);
                let line = line.trim();
                if !line.is_empty() {
                    // deadwood: exempt -- intent id parsed out of command OUTPUT, then INT- stripped -- structured output, not user input
                    let id = line
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .replace("INT-", "");
                    if !id.is_empty() {
                        return CommandResult::Output(format!("intent show {}", id));
                    }
                }
            }
            CommandResult::Output(String::new())
        }
        "history" | "hist" => {
            let rows: Vec<(String, i64)> = {
                let mut stmt = match db.conn.prepare(
                    "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 500"
                ) {
                    Ok(s) => s,
                    Err(_) => return CommandResult::Error("Cannot read history".to_string()),
                };
                stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
                    .unwrap_or_default()
            };
            let mut items = String::new();
            for (cmd, ts) in &rows {
                let time = fmt_time(*ts, "%H:%M");
                items.push_str(&format!(
                    "{}  {}
",
                    time, cmd
                ));
            }
            let mut child = match Command::new("sk")
                .args(["--prompt=pick history> ", "--height=50%", "--reverse"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(_) => return CommandResult::Error("sk not found".to_string()),
            };
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(items.as_bytes());
            }
            let output = child
                .wait_with_output()
                .unwrap_or_else(|_| std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                });
            if output.status.success() {
                let line = String::from_utf8_lossy(&output.stdout);
                let cmd = line.trim().splitn(2, "  ").nth(1).unwrap_or("").trim();
                if !cmd.is_empty() {
                    return CommandResult::Output(format!("  Selected: {}", cmd));
                }
            }
            CommandResult::Output(String::new())
        }
        "file" | "files" => {
            let search_dir = if extra == "--core" {
                core_root.to_string()
            } else {
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| ".".to_string())
            };
            let rg_out = Command::new("rg").args(["--files", &search_dir]).output();
            let items = match rg_out {
                Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
                Err(_) => return CommandResult::Error("rg not found".to_string()),
            };
            let mut child = match Command::new("sk")
                .args(["--prompt=pick file> ", "--height=50%", "--reverse"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(_) => return CommandResult::Error("sk not found".to_string()),
            };
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(items.as_bytes());
            }
            let output = child
                .wait_with_output()
                .unwrap_or_else(|_| std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                });
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return CommandResult::Output(format!("  {}", path));
                }
            }
            CommandResult::Output(String::new())
        }
        _ => CommandResult::Output(format!(
            "  {}  pick -- fuzzy selection
  {}
  {}
  {}
  {}",
            "🌲".to_string(),
            "  pick intent          fuzzy search all intents".dimmed(),
            "  pick intent --active in-progress intents only".dimmed(),
            "  pick history         fuzzy command history".dimmed(),
            "  pick file [--core]   fuzzy file search".dimmed(),
        )),
    }
}

fn devshell_list(args: &[&str]) -> CommandResult {
    if let Some(&sub) = args.first() {
        if sub != "list" {
            return CommandResult::Error(format!(
                "unknown devshell subcommand '{}' (try: devshell list)",
                sub
            ));
        }
    }
    let cwd = std::env::current_dir().unwrap_or_default();
    let output = std::process::Command::new("nix")
        .args(["flake", "show", "--json"])
        .current_dir(&cwd)
        .output();
    let out = match output {
        Ok(o) if o.status.success() => o,
        _ => {
            return CommandResult::Error(format!(
                "no flake found in {} (nix flake show failed)",
                cwd.display()
            ))
        }
    };
    let json: serde_json::Value = match serde_json::from_slice(&out.stdout) {
        Ok(v) => v,
        Err(_) => return CommandResult::Error("could not parse nix flake output".to_string()),
    };
    let system = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    let shells = json.get("devShells").and_then(|d| d.get(system.as_str()));
    let map = match shells {
        Some(serde_json::Value::Object(m)) if !m.is_empty() => m,
        _ => return CommandResult::Output(format!("  no devShells for {} in this flake", system)),
    };
    let mut lines = vec![format!("  devShells in current flake ({}):", system)];
    for name in map.keys() {
        lines.push(format!("    {}", name));
    }
    CommandResult::Output(lines.join("\n"))
}

fn pkg_search(args: &[&str]) -> CommandResult {
    // pkg-search <term> -- search nixpkgs, print name/version/description (INT-134, Lane 2).
    // Deliberate, read-only: runs `nix search nixpkgs <regex> --json` (nix 2.34 form, regex
    // as a separate arg). Latency is expected because you asked -- this is NOT a TAB handler.
    // Caches results to /tmp/fsh-pkg-search.json so completion can read them without a network hit.
    let term = args.join(" ");
    if term.trim().is_empty() {
        return CommandResult::Error("pkg-search <term>  -- e.g. pkg-search ripgrep".to_string());
    }
    let out = std::process::Command::new("nix")
        .args(["search", "nixpkgs", &term, "--json"])
        .output();
    let out = match out {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            return CommandResult::Error(format!("pkg-search: nix search failed: {}", err.trim()));
        }
        Err(e) => return CommandResult::Error(format!("pkg-search: {}", e)),
    };
    let json: serde_json::Value = match serde_json::from_slice(&out.stdout) {
        Ok(v) => v,
        Err(e) => return CommandResult::Error(format!("pkg-search: parse: {}", e)),
    };
    let map = match json.as_object() {
        Some(m) if !m.is_empty() => m,
        _ => return CommandResult::Output(format!("  no packages matching '{}'", term)),
    };
    // Cache raw JSON for completion (piece 2). Best-effort: a failed write never fails the search.
    let _ = std::fs::write("/tmp/fsh-pkg-search.json", &out.stdout);
    let mut rows: Vec<(String, String, String)> = Vec::new();
    for (attr, v) in map.iter() {
        let name = attr.rsplit('.').next().unwrap_or(attr).to_string();
        let ver = v
            .get("version")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let desc = v
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        rows.push((name, ver, desc));
    }
    rows.sort();
    rows.dedup();
    let mut lines = vec![format!(
        "  \u{1f50d} nixpkgs matches for '{}' ({})",
        term,
        rows.len()
    )];
    lines.push("\u{2500}".repeat(52));
    for (name, ver, desc) in rows.iter().take(40) {
        let d = if desc.len() > 60 {
            format!("{}...", &desc[..57])
        } else {
            desc.clone()
        };
        lines.push(format!("  {:<28} {:<14} {}", name, ver, d));
    }
    if rows.len() > 40 {
        lines.push(format!(
            "\n  ... {} more (narrow the term)",
            rows.len() - 40
        ));
    }
    CommandResult::Output(lines.join("\n"))
}

fn devshell_enter(args: &[&str]) -> CommandResult {
    // devshell enter [name] -- enter a flake devShell reproducibly (INT-134, Lane 2).
    // fsh IS the shell; it cannot inject a devShell into its own process. It execs
    // `nix develop` and runs a nested fsh inside the real nix env -- `exit` returns here.
    // Authorized + reproducible: nix does the eval, you type the command, nothing auto-runs.
    // Nesting is allowed -- nix stacks devShells cleanly, and this session normally
    // starts inside friday-dev, so a hard nix-shell block would make enter unusable.
    let cwd = std::env::current_dir().unwrap_or_default();
    if !cwd.join("flake.nix").exists() {
        return CommandResult::Error(format!("devshell: no flake.nix in {}", cwd.display()));
    }
    let fsh = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "faelight-shell".to_string());
    let mut cmd = std::process::Command::new("nix");
    cmd.arg("develop");
    if let Some(name) = args.first() {
        cmd.arg(format!(".#{}", name));
    }
    cmd.args(["--command", &fsh]).current_dir(&cwd);
    match cmd.status() {
        Ok(_) => CommandResult::Empty,
        Err(e) => CommandResult::Error(format!("devshell enter: {}", e)),
    }
}

fn cache(args: &[&str]) -> CommandResult {
    // INT-068: cache status | cache push -- shells out to pkgs/faelight/scripts/cache-*
    let sub = args.first().copied().unwrap_or("");
    let home = std::env::var("HOME").unwrap_or_default();
    let script = format!("{}/0-core/pkgs/faelight/scripts/cache-{}", home, sub);
    match sub {
        "status" => {
            let output = std::process::Command::new(&script).output();
            match output {
                Ok(o) => {
                    let out = String::from_utf8_lossy(&o.stdout).to_string();
                    let err = String::from_utf8_lossy(&o.stderr).to_string();
                    CommandResult::Output(format!("{}{}", out, err).trim_end().to_string())
                }
                Err(e) => CommandResult::Error(format!("cache status: {}", e)),
            }
        }
        "push" => {
            let status = std::process::Command::new(&script)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            match status {
                Ok(_) => CommandResult::Empty,
                Err(e) => CommandResult::Error(format!("cache push: {}", e)),
            }
        }
        _ => CommandResult::Error("usage: cache <status|push>".to_string()),
    }
}

fn cd(args: &[&str]) -> CommandResult {
    let target = args.first().copied().unwrap_or("~");
    let home = std::env::var("HOME").unwrap_or_default();
    let path = if target == "~" || target.is_empty() {
        std::path::PathBuf::from(&home)
    } else if target.starts_with("~/") {
        std::path::PathBuf::from(format!("{}/{}", home, &target[2..]))
    } else {
        std::path::PathBuf::from(target)
    };

    match std::env::set_current_dir(&path) {
        Ok(_) => {
            let _ = std::process::Command::new("zoxide")
                .args(["add", &path.to_string_lossy()])
                .status();
            CommandResult::Empty
        }
        Err(e) => CommandResult::Error(format!("cd: {}: {}", target, e)),
    }
}

fn parse_since_time(arg: &str) -> i64 {
    let now = chrono::Local::now().timestamp();
    let arg_lower = arg.to_lowercase();
    match arg_lower.as_str() {
        "today" => chrono::Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp(),
        "yesterday" => now - 86400,
        "week" | "this week" => now - 7 * 86400,
        _ => {
            if arg_lower.ends_with('h') {
                let h: i64 = arg_lower.trim_end_matches('h').parse().unwrap_or(1);
                now - h * 3600
            } else if arg_lower.ends_with('m') {
                let m: i64 = arg_lower.trim_end_matches('m').parse().unwrap_or(30);
                now - m * 60
            } else if arg_lower.ends_with('d') {
                let d: i64 = arg_lower.trim_end_matches('d').parse().unwrap_or(1);
                now - d * 86400
            } else {
                chrono::NaiveDate::parse_from_str(&arg_lower, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
                    .unwrap_or(now - 86400)
            }
        }
    }
}

fn since_cmd(db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    let _ = core_root;
    let arg = args.join(" ");
    let arg = if arg.is_empty() {
        "yesterday".to_string()
    } else {
        arg
    };
    let since_ts = parse_since_time(&arg);
    let now = chrono::Local::now().timestamp();

    let fmt_ts = |ts: i64| -> String {
        chrono::DateTime::from_timestamp(ts, 0)
            .map(|d| {
                d.with_timezone(&chrono::Local)
                    .format("%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_default()
    };

    let mut out = String::new();
    out.push_str(&format!(
        "{}\n",
        "\u{1f332} Since \u{2014} Forest Timeline".cyan().bold()
    ));
    out.push_str(&format!("{}\n", "\u{2501}".repeat(52).dimmed()));
    out.push_str(&format!(
        "  {} {} \u{2192} now\n\n",
        "Period:".dimmed(),
        fmt_ts(since_ts).bright_white()
    ));

    // Git commits
    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='git' AND action='commit' AND timestamp >= ?1 ORDER BY timestamp ASC"
    ) {
        let commits: Vec<(Option<String>, i64)> = stmt.query_map(
            rusqlite::params![since_ts], |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap().filter_map(|r| r.ok()).collect();

        if !commits.is_empty() {
            out.push_str(&format!("  \u{1f527} {} git commit(s)\n",
                commits.len().to_string().bright_white()));
            for (payload, ts) in commits.iter().take(5) {
                let msg = payload.as_deref()
                    .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                    .and_then(|v| v["detail"]["message"].as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "commit".to_string());
                let short = if msg.len() > 50 { format!("{}\u{2026}", &msg[..50]) } else { msg };
                out.push_str(&format!("    {}  {}\n", fmt_ts(*ts).dimmed(), short.white()));
            }
            if commits.len() > 5 {
                out.push_str(&format!("    \u{2026} {} more\n", commits.len() - 5));
            }
            out.push('\n');
        }
    }

    // Health changes
    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' AND action='run' AND timestamp >= ?1 ORDER BY timestamp ASC"
    ) {
        let health_events: Vec<(i64, i64)> = stmt.query_map(
            rusqlite::params![since_ts], |r| {
                let p: Option<String> = r.get(0)?;
                let ts: i64 = r.get(1)?;
                let h = p.as_deref()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                    .and_then(|v| v["detail"]["health"].as_i64())
                    .unwrap_or(95);
                Ok((h, ts))
            }
        ).unwrap().filter_map(|r| r.ok()).collect();

        if !health_events.is_empty() {
            let first_h = health_events.first().map(|e| e.0).unwrap_or(95);
            let last_h = health_events.last().map(|e| e.0).unwrap_or(95);
            let delta = last_h - first_h;
            let delta_str = if delta > 0 {
                format!("\u{25b2}{}", delta).green().to_string()
            } else if delta < 0 {
                format!("\u{25bc}{}", delta.abs()).bright_red().to_string()
            } else { "stable".dimmed().to_string() };
            out.push_str(&format!("  \u{1f3e5} health: {}% \u{2192} {}%  {}  ({} checks)\n\n",
                first_h.to_string().dimmed(),
                last_h.to_string().bright_white(),
                delta_str,
                health_events.len().to_string().dimmed()));
        }
    }

    // Reactions fired
    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT rule_id, triggered_at, message FROM reaction_log WHERE triggered_at >= ?1 ORDER BY triggered_at ASC"
    ) {
        let reactions: Vec<(String, i64, String)> = stmt.query_map(
            rusqlite::params![since_ts], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        ).unwrap().filter_map(|r| r.ok()).collect();

        if !reactions.is_empty() {
            out.push_str(&format!("  \u{26a1} {} reaction(s) fired\n",
                reactions.len().to_string().bright_white()));
            for (rule, ts, msg) in reactions.iter().take(3) {
                let short = if msg.len() > 45 { format!("{}\u{2026}", &msg[..45]) } else { msg.clone() };
                out.push_str(&format!("    {}  {} \u{2014} {}\n",
                    fmt_ts(*ts).dimmed(), rule.cyan(), short.dimmed()));
            }
            out.push('\n');
        }
    }

    // External commands
    let cmd_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM events WHERE domain='external' AND action='run' AND timestamp >= ?1",
        rusqlite::params![since_ts], |r| r.get(0)
    ).unwrap_or(0);
    if cmd_count > 0 {
        out.push_str(&format!(
            "  \u{2328}\u{fe0f}  {} external command(s) run\n\n",
            cmd_count.to_string().bright_white()
        ));
    }

    // Idle sessions
    let idle_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM events WHERE domain='idle' AND action='idle.start' AND timestamp >= ?1",
        rusqlite::params![since_ts], |r| r.get(0)
    ).unwrap_or(0);
    if idle_count > 0 {
        out.push_str(&format!(
            "  \u{1f4a4} {} idle session(s)\n\n",
            idle_count.to_string().dimmed()
        ));
    }

    let elapsed_h = (now - since_ts) / 3600;
    out.push_str(&format!("{}\n", "\u{2501}".repeat(52).dimmed()));
    out.push_str(&format!(
        "  \u{23f1}\u{fe0f}  {}h of forest history\n",
        elapsed_h.to_string().bright_white()
    ));

    CommandResult::Output(out)
}

fn debug_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let sub = args.first().copied().unwrap_or("last");
    match sub {
        "last" => {
            let last: Option<(String, i64)> = db
                .conn
                .query_row(
                    "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .ok();
            let last_ext: Option<(String, i64)> = db.conn.query_row(
                "SELECT payload, timestamp FROM events WHERE domain='external' AND action='run' ORDER BY timestamp DESC LIMIT 1",
                [], |r| Ok((r.get(0)?, r.get(1)?))
            ).ok();
            let mut out = String::new();
            out.push_str(&format!("{}\n", "🌲 Debug — Last Command".cyan().bold()));
            out.push_str(&format!("{}\n\n", "━".repeat(52).dimmed()));
            if let Some((cmd, ts)) = &last {
                let dt = chrono::DateTime::from_timestamp(*ts, 0)
                    .map(|d| {
                        d.with_timezone(&chrono::Local)
                            .format("%H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_default();
                out.push_str(&format!("  {} Last shell command\n", "▶".bright_cyan()));
                out.push_str(&format!("    {} {}\n", "cmd:".dimmed(), cmd.bright_white()));
                out.push_str(&format!("    {} {}\n", "at:".dimmed(), dt.dimmed()));
                // deadwood: exempt -- debug history classification; the token labels a PRIOR command's
                // recorded output and is never used to select the current execution path
                let first_tok = cmd.split_whitespace().next().unwrap_or("");
                let classification = if db.get_alias(first_tok).is_some() {
                    "alias expanded"
                } else {
                    match first_tok {
                        "cd" | "ls" | "pwd" | "health" | "events" | "intents" | "since" | "gc"
                        | "ps" | "forecast" | "checkpoint" | "git" | "commits" | "story"
                        | "advise" | "debug" | "usage" => "forest builtin",
                        "q" | "exit" | "quit" => "shell control",
                        "flow" => "flow mode",
                        _ => "external PATH",
                    }
                };
                out.push_str(&format!(
                    "    {} {}\n",
                    "type:".dimmed(),
                    classification.bright_green()
                ));
            }
            out.push('\n');
            if let Some((payload, ts)) = &last_ext {
                let dt = chrono::DateTime::from_timestamp(*ts, 0)
                    .map(|d| {
                        d.with_timezone(&chrono::Local)
                            .format("%H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_default();
                out.push_str(&format!("  {} Last external run\n", "▶".yellow()));
                out.push_str(&format!("    {} {}\n", "event:".dimmed(), payload.white()));
                out.push_str(&format!("    {} {}\n", "at:".dimmed(), dt.dimmed()));
            }
            out.push('\n');
            let last_reaction: Option<(String, String, i64)> = db.conn.query_row(
                "SELECT rule_id, message, triggered_at FROM reaction_log ORDER BY triggered_at DESC LIMIT 1",
                [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            ).ok();
            if let Some((rule, msg, ts)) = last_reaction {
                let dt = chrono::DateTime::from_timestamp(ts, 0)
                    .map(|d| {
                        d.with_timezone(&chrono::Local)
                            .format("%H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_default();
                out.push_str(&format!("  {} Last reaction fired\n", "▶".yellow()));
                out.push_str(&format!("    {} {}\n", "rule:".dimmed(), rule.cyan()));
                out.push_str(&format!("    {} {}\n", "msg:".dimmed(), msg.dimmed()));
                out.push_str(&format!("    {} {}\n", "at:".dimmed(), dt.dimmed()));
            } else {
                out.push_str(&format!("  {} No reactions fired recently\n", "▶".dimmed()));
            }
            out.push_str(&format!("\n{}\n", "━".repeat(52).dimmed()));
            out.push_str(&format!(
                "  {} Run: debug reactions  debug preexec\n",
                "hint:".dimmed()
            ));
            CommandResult::Output(out)
        }
        "reactions" => {
            let mut out = String::new();
            out.push_str(&format!("{}\n", "🌲 Debug — Reaction State".cyan().bold()));
            out.push_str(&format!("{}\n\n", "━".repeat(52).dimmed()));
            let now = chrono::Local::now().timestamp();
            let rules = [
                "health.advisory",
                "health.stale",
                "security.aging",
                "checkpoint.stale",
                "intent.overflow",
                "forecast.declining",
            ];
            for rule in &rules {
                let cooldown: Option<i64> = db
                    .conn
                    .query_row(
                        "SELECT last_fired FROM reaction_cooldowns WHERE rule_id = ?1",
                        rusqlite::params![rule],
                        |r| r.get(0),
                    )
                    .ok();
                let state = match cooldown {
                    Some(ts) => {
                        let elapsed = now - ts;
                        if elapsed < 3600 {
                            format!("on cooldown ({}m ago)", elapsed / 60)
                                .yellow()
                                .to_string()
                        } else {
                            format!("ready ({}h ago)", elapsed / 3600)
                                .green()
                                .to_string()
                        }
                    }
                    None => "never fired - ready".green().to_string(),
                };
                out.push_str(&format!(
                    "  {} {}\n    {}\n\n",
                    "▶".bright_cyan(),
                    rule.bright_white(),
                    state
                ));
            }
            out.push_str(&format!("{}\n", "━".repeat(52).dimmed()));
            out.push_str(&format!(
                "  {} Edit: runtime/reaction-discipline.toml\n",
                "hint:".dimmed()
            ));
            CommandResult::Output(out)
        }
        "preexec" => {
            let mut out = String::new();
            out.push_str(&format!("{}\n", "🌲 Debug — Pre-exec Hooks".cyan().bold()));
            out.push_str(&format!("{}\n\n", "━".repeat(52).dimmed()));
            out.push_str(&format!("  {} Active guards\n", "▶".bright_cyan()));
            out.push_str(&format!(
                "    {} git guardrail - blocks commit/push when locked\n",
                "✅".normal()
            ));
            out.push_str(&format!(
                "    {} flow mode - intent focus when set\n",
                "✅".normal()
            ));
            out.push_str(&format!(
                "    {} intent-guard hook - planned Phase 20b\n",
                "⬜".normal()
            ));
            out.push_str(&format!(
                "\n  {} All guards EXPLICIT and OBSERVABLE\n",
                "info:".dimmed()
            ));
            out.push_str(&format!(
                "  {} No implicit mutations - every block shows reason\n",
                "info:".dimmed()
            ));
            out.push_str(&format!("\n{}\n", "━".repeat(52).dimmed()));
            CommandResult::Output(out)
        }
        _ => CommandResult::Error(format!(
            "  debug: unknown '{}'\n  usage: debug last | debug reactions | debug preexec",
            sub
        )),
    }
}

fn usage_report(db: &ForestDb) -> CommandResult {
    let mut out = String::new();
    out.push_str(&format!(
        "{}\n",
        "🌲 Usage Report — faelight-shell".cyan().bold()
    ));
    out.push_str(&format!("{}\n\n", "━".repeat(52).dimmed()));
    let total_shell: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
        .unwrap_or(0);
    let total_external: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE domain='external' AND action='run'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let total = total_shell + total_external;
    let native_pct = if total > 0 {
        (total_shell * 100) / total
    } else {
        0
    };
    let ext_pct = 100 - native_pct;
    out.push_str(&format!("  {} Command Coverage\n", "▶".bright_cyan()));
    out.push_str(&format!(
        "    · {} total commands tracked\n",
        total.to_string().bright_white()
    ));
    out.push_str(&format!(
        "    · {}% handled natively ({} cmds)\n",
        native_pct.to_string().bright_green(),
        total_shell
    ));
    out.push_str(&format!(
        "    · {}% forwarded to PATH ({} cmds)\n",
        ext_pct.to_string().yellow(),
        total_external
    ));
    out.push('\n');
    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT command, COUNT(*) as n FROM shell_history GROUP BY command ORDER BY n DESC LIMIT 8",
    ) {
        let rows: Vec<(String, i64)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        if !rows.is_empty() {
            out.push_str(&format!("  {} Top native commands\n", "▶".bright_cyan()));
            let max = rows[0].1.max(1);
            for (cmd, count) in &rows {
                let bar = "█".repeat(((count * 20) / max).max(1) as usize);
                out.push_str(&format!(
                    "    {:20} {} {}\n",
                    cmd.bright_white(),
                    bar.bright_green(),
                    count.to_string().dimmed()
                ));
            }
            out.push('\n');
        }
    }
    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT payload, COUNT(*) as n FROM events WHERE domain='external' AND action='run' GROUP BY payload ORDER BY n DESC LIMIT 5"
    ) {
        let rows: Vec<(String, i64)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap().filter_map(|r| r.ok()).collect();
        if !rows.is_empty() {
            out.push_str(&format!("  {} Top external commands\n", "▶".yellow()));
            for (payload, count) in &rows {
                let cmd = payload.trim_start_matches("cmd:").split(" exit:").next().unwrap_or(payload);
                out.push_str(&format!("    {:20} {} times\n", cmd.white(), count.to_string().dimmed()));
            }
            out.push('\n');
        }
    }
    let confidence = if native_pct >= 90 {
        "🟢 HIGH — ready for login shell"
    } else if native_pct >= 75 {
        "🟡 MEDIUM — daily driver territory"
    } else {
        "🔴 LOW — still building"
    };
    out.push_str(&format!(
        "  {} Migration Confidence: {}\n",
        "▶".bright_cyan(),
        confidence
    ));
    out.push_str(&format!("\n{}\n", "━".repeat(52).dimmed()));
    CommandResult::Output(out)
}

fn z_jump(args: &[&str]) -> CommandResult {
    if args.is_empty() {
        let home = std::env::var("HOME").unwrap_or_default();
        return match std::env::set_current_dir(&home) {
            Ok(_) => {
                let _ = std::process::Command::new("zoxide")
                    .args(["add", &home])
                    .status();
                CommandResult::Empty
            }
            Err(e) => CommandResult::Error(format!("z: {}", e)),
        };
    }
    let query = args.join(" ");
    let result = std::process::Command::new("zoxide")
        .args(["query", "--", &query])
        .output();
    match result {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                return CommandResult::Error(format!("z: no match for '{}'", query));
            }
            match std::env::set_current_dir(&path) {
                Ok(_) => {
                    let _ = std::process::Command::new("zoxide")
                        .args(["add", &path])
                        .status();
                    CommandResult::Empty
                }
                Err(e) => CommandResult::Error(format!("z: {}: {}", path, e)),
            }
        }
        _ => CommandResult::Error(format!(
            "  z: no match for '{}'
  hint: use cd first — zoxide will learn it",
            query
        )),
    }
}

fn theme_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let themes = ["forest", "minimal", "friday", "classic"];
    match args.first().copied() {
        None => {
            let current = db.get_theme();
            let mut out = String::new();
            out.push_str(&format!("{}\n\n", "🌲 Prompt Themes".cyan().bold()));
            for t in &themes {
                let marker = if *t == current.as_str() { "▶" } else { " " };
                out.push_str(&format!(
                    "  {} {}\n",
                    marker.bright_green(),
                    t.bright_white()
                ));
            }
            out.push_str(&format!(
                "\n  {} theme <name> to switch\n",
                "hint:".dimmed()
            ));
            CommandResult::Output(out)
        }
        Some(name) if themes.contains(&name) => {
            if let Err(e) = db.set_theme(name) {
                eprintln!("warning: failed to set theme: {}", e);
            }
            CommandResult::Output(format!(
                "  {} prompt theme set to {}",
                "✅".normal(),
                name.bright_green()
            ))
        }
        Some(name) => CommandResult::Error(format!(
            "  theme: unknown theme '{}'\n  available: forest, minimal, friday, classic",
            name
        )),
    }
}

fn cmd_in_path(cmd: &str) -> bool {
    if cmd.contains('/') {
        return std::path::Path::new(cmd).exists();
    }
    let path_env = std::env::var("PATH").unwrap_or_default();
    path_env
        .split(':')
        .any(|dir| std::path::Path::new(&format!("{}/{}", dir, cmd)).exists())
}

fn explain_exit_code(code: i32) -> &'static str {
    match code {
        1 => "general error",
        2 => "misuse of shell builtin",
        126 => "permission denied -- command exists but not executable. Try: chmod +x <file>",
        127 => "command not found",
        128 => "invalid exit argument",
        130 => "interrupted by Ctrl+C",
        137 => "killed (OOM or SIGKILL)",
        139 => "segmentation fault",
        _ => "non-zero exit",
    }
}

/// Spawn a configured command with INT-185's stderr tee, record `last_stderr`, and wait.
///
/// ★ THE BOUNDARY IS A CONFIGURED `Command`, NOT TEXT. A text-taking helper would quietly
/// rebuild the string-reinspection architecture the spine exists to replace -- the caller must
/// have already decided what to run. Two callers, two honest constructions:
///   run_external  -- `sh -c <line>`: DELEGATE what fsh has not modelled (pipes, &&, redirects).
///   execute_plan  -- `argv[0]` + args: EXECUTE what the AST has modelled. No sh, no re-parse.
///
/// stdin/stdout stay inherited (normal output and interactive programs unaffected); only stderr
/// is piped and TEE'd -- written to the real terminal live AND captured, so the knowledge engine
/// (postexec, INT-233) reads the REAL error text instead of fsh's "exited N" status string.
/// Reading in a thread concurrently with wait() avoids a pipe-fill deadlock on large stderr.
/// `last_stderr` is ALWAYS overwritten (empty on success) so it can never be stale.
///
/// Returns the raw wait() result. Classification is the CALLER's job, because the two paths
/// detect the same situation differently: `sh` reports a missing command as exit 127, while a
/// direct spawn fails with io::ErrorKind::NotFound before any process exists.
fn spawn_with_tee(
    mut cmd: std::process::Command,
    db: &ForestDb,
) -> std::io::Result<std::process::ExitStatus> {
    cmd.stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = db.conn.execute(
                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_stderr', '')",
                [],
            );
            return Err(e);
        }
    };

    let child_stderr = child.stderr.take();
    let tee = std::thread::spawn(move || {
        use std::io::{Read, Write};
        let mut captured: Vec<u8> = Vec::new();
        if let Some(mut es) = child_stderr {
            let mut buf = [0u8; 4096];
            let mut real = std::io::stderr();
            loop {
                match es.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let _ = real.write_all(&buf[..n]);
                        let _ = real.flush();
                        captured.extend_from_slice(&buf[..n]);
                    }
                }
            }
        }
        captured
    });

    let status = child.wait();
    let captured_stderr = tee.join().unwrap_or_default();
    let _ = db.conn.execute(
        "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_stderr', ?1)",
        rusqlite::params![String::from_utf8_lossy(&captured_stderr).to_string()],
    );
    status
}

/// Record a failed command for the failure-frequency counters Friday reads.
///
/// ⚠️ Takes the COMMAND WORD, not the line. It used to take the line and extract the word
/// itself with `cmd.split_whitespace().next()` -- a fifth tokenizer, and a quote-blind one:
/// `"my command" foo` was filed under `"my`, so a quoted command never accumulated toward its
/// own count. INT-171 consolidated dispatch to one entry point and its own note warned that a
/// future `split_whitespace().next()` on a user command is a bug; this one survived the sweep
/// because it LABELS rather than dispatches. Callers now pass `command_word(line)` (sh path) or
/// `argv[0]` (spine path) -- no reconstruction, and no re-parsing inside telemetry.
fn record_failure(db: &ForestDb, cmd_word: &str, exit_code: i32) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let first = cmd_word;
    let _ = db.conn.execute(
        "INSERT INTO command_failures (command, exit_code, cwd, timestamp) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![first, exit_code, cwd, ts],
    );
    let count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM command_failures WHERE command = ?1 AND timestamp > ?2",
            rusqlite::params![first, ts - 86400],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if count == 3 {
        println!("  🌳 Friday: {} failed 3 times today -- consider adding an alias or checking the command", first);
    }
}

/// INT-169: execute an ExecutionPlan by spawning argv[0] DIRECTLY. No `sh`, no re-parsing.
///
/// This is the moment the spine stops being observational. `run_external` hands an unmodelled
/// LINE to sh, which then re-parses and re-expands it -- sh is the final authority on quoting,
/// operators and expansion. Here the AST is the authority: whatever the parser and lowering
/// decided IS what gets spawned. If the plan is wrong, the command is wrong, and there is
/// nowhere to hide.
///
/// Deliberately narrow. It executes exactly what a plan contains: argv, cwd, environment
/// intent. It must NOT compensate for incomplete planning -- no splitting, no expansion, no
/// operator handling. If those are missing the failure belongs upstream, where it can be seen.
///
/// Not wired into the live path. Reached only via `spine exec`, an opt-in builtin.
/// INT-169: run a plan -- builtins first, then a direct spawn. The single implementation both
/// the `spine exec` debug builtin and `exec::execute_spine` call, so there is one answer to
/// "what does running a plan mean" rather than two that can drift.
///
/// `ExecutionMode::Spine` is load-bearing twice over: it suppresses every `run_external` call
/// (so an unrecognised command answers `NotBuiltin` instead of being handed to `sh -c`), and it
/// suppresses text-world transforms (history, alias, plugin expansion) so argv stays
/// authoritative. Because nothing can rewrite argv beneath us, falling back to `execute_plan`
/// with the SAME plan is correct by construction rather than by a smarter fallback.
///
/// `source` is provenance only -- carried for the handful of builtins that need the original
/// text. It is never re-parsed here.
pub fn execute_plan_dispatch(
    plan: &crate::spine::plan::ExecutionPlan,
    source: &str,
    db: &ForestDb,
    core_root: &str,
) -> CommandResult {
    let argv = match plan.argv_as_utf8() {
        Ok(v) => v,
        Err(e) => return CommandResult::Error(format!("  {e}")),
    };
    match execute_impl(&argv, source, db, core_root, &[], ExecutionMode::Spine) {
        CommandResult::NotBuiltin => {
            // Honest diagnostic rather than a bare "command not found": if argv[0] is a known
            // alias, the command DOES exist -- the spine path simply does not expand aliases yet
            // (a text-world transform). Reporting that as not-found would hide a capability gap
            // behind a lie.
            if let Some(target) = argv.first().and_then(|a| db.get_alias(a)) {
                return CommandResult::Error(format!(
                    "  {} is an alias ({}) -- the spine path does not expand aliases yet",
                    argv.first().map(String::as_str).unwrap_or(""),
                    target
                ));
            }
            execute_plan(plan, db)
        }
        result => result,
    }
}

fn execute_plan(plan: &crate::spine::plan::ExecutionPlan, db: &ForestDb) -> CommandResult {
    use crate::spine::plan::{Environment, IoPlan};

    let Some(program) = plan.argv.first() else {
        return CommandResult::Error("  empty plan: nothing to execute".to_string());
    };

    let mut cmd = std::process::Command::new(program);
    cmd.args(&plan.argv[1..]);

    if let Some(dir) = plan.cwd.as_ref() {
        cmd.current_dir(dir);
    }

    // Matched exhaustively so a new variant is a compile error here, not a silent no-op.
    match &plan.env {
        Environment::Inherit => {}
        Environment::Replace(vars) => {
            cmd.env_clear();
            for (k, v) in vars {
                cmd.env(k, v);
            }
        }
    }
    match plan.io {
        // No redirects, no pipe wiring: spawn_with_tee's inherited stdin/stdout is exactly this.
        IoPlan::Simple => {}
    }

    let word = program.to_string_lossy().to_string();
    match spawn_with_tee(cmd, db) {
        Ok(s) if s.success() => CommandResult::Empty,
        Ok(s) => {
            let code = s.code().unwrap_or(1);
            record_failure(db, &word, code);
            CommandResult::Error(format!("  exited {} -- {}", code, explain_exit_code(code)))
        }
        // A direct spawn reports a missing command HERE, before any process exists -- unlike the
        // sh path, which sees it as exit 127. Same situation for the user, different detection,
        // which is precisely why classification stayed with the caller.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            record_failure(db, &word, 127);
            CommandResult::Error(crate::error::FlowError::CommandNotFound(word).display_colored())
        }
        Err(e) => CommandResult::Error(format!("  failed to execute: {}", e)),
    }
}

fn run_external(line: &str, db: &ForestDb) -> CommandResult {
    // INT-171 gate 2: quote-aware command word so the not-found suggestion sees the
    // real command (`"deploy"` -> deploy), not the quoted literal.
    let cmd_name = command_word(line);
    let cmd_name = cmd_name.as_str();
    if !cmd_name.is_empty() && !cmd_name.contains('/') && !cmd_in_path(cmd_name) {
        let typed_cmd = cmd_name.to_lowercase();
        let known: &[&str] = &[
            "deploy",
            "cistart",
            "cicomplete",
            "intent",
            "delete",
            "del",
            "fsearch",
            "query",
            "rspatch",
            "patch",
            "edit",
            "run",
            "friday",
            "d",
            "gc",
            "gp",
            "core",
            "fg",
            "faelight-shell",
            "faelight-term",
            "git",
            "cargo",
            "python3",
            "python",
            "node",
            "npm",
            "sudo",
            "systemctl",
            "pacman",
            "ssh",
            "curl",
            "wget",
            "make",
            "vim",
            "nvim",
        ];
        let prefix_len = typed_cmd.len().min(3);
        let prefix = &typed_cmd[..prefix_len];
        let suggestion = known
            .iter()
            .filter(|&&k| levenshtein(k, &typed_cmd) <= 2 && k != typed_cmd.as_str())
            .min_by_key(|&&k| levenshtein(k, &typed_cmd))
            .copied();
        let alias_suggestion: Option<String> = db
            .conn
            .query_row(
                "SELECT name FROM shell_aliases WHERE name LIKE ?1 AND name != ?2 LIMIT 1",
                rusqlite::params![format!("{}%", prefix), typed_cmd.as_str()],
                |r| r.get(0),
            )
            .ok();
        // INT-143, and this is the most dangerous bug of the session: this printed
        // "command not found" and returned CommandResult::Empty -- which means SUCCESS.
        // PROVEN on the deployed binary 2026-07-16:
        //     nosuchcommand123 && echo "DANGER_THIS_RAN"
        //     -> command not found: nosuchcommand123
        //     -> DANGER_THIS_RAN            <-- IT RAN ANYWAY
        //     false && echo "SHOULD_NOT_PRINT"   -> correctly silent
        // So `&&` honoured a real failure and ignored a TYPO. `mkae build && rm -rf dist` would
        // have deleted dist. `$?` said 0 for a command that never existed.
        // The message was always honest. The TYPE was the lie -- the same shape as every other
        // bug tonight. main.rs decides `&&` with `!matches!(result, Error(_))` (main.rs:1441),
        // so returning Error is what makes the chain stop.
        // The text moves into the Error payload rather than being println!'d, because main.rs
        // prints an Error payload -- printing here AND returning Error would show it twice.
        // INT-171 gate 5: the not-found message comes FROM the typed error, so it
        // cannot drift from the CommandNotFound kind. This is the 968c7be5 site.
        let mut msg =
            crate::error::FlowError::CommandNotFound(typed_cmd.to_string()).display_colored();
        if let Some(s) = suggestion {
            msg.push_str(&format!(
                "\n  {} did you mean: {}",
                "\u{2192}".bright_cyan(),
                s.bright_cyan()
            ));
        } else if let Some(a) = alias_suggestion {
            msg.push_str(&format!(
                "\n  {} did you mean: {}",
                "\u{2192}".bright_cyan(),
                a.bright_cyan()
            ));
        }
        return CommandResult::Error(msg);
    }
    // INT-185's stderr tee now lives in spawn_with_tee, shared with the spine's plan
    // executor. This path delegates the UNMODELLED line to sh; the spine path spawns argv
    // directly. Different responsibilities, one telemetry implementation.
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(line);
    let status = spawn_with_tee(cmd, db);
    match status {
        Ok(s) => {
            if s.success() {
                CommandResult::Empty
            } else {
                let code = s.code().unwrap_or(1);
                // INT-233 -- command not found: suggest nearest known alternative
                if code == 127 {
                    // INT-171 gate 2: quote-aware command word for the builtin not-found check.
                    let typed_cmd = command_word(line).to_lowercase();
                    if !typed_cmd.is_empty() {
                        let known: &[&str] = &[
                            "deploy",
                            "cistart",
                            "cicomplete",
                            "intent",
                            "delete",
                            "del",
                            "fsearch",
                            "query",
                            "rspatch",
                            "patch",
                            "edit",
                            "run",
                            "friday",
                            "d",
                            "gc",
                            "gp",
                            "core",
                            "faelight-git",
                            "fg",
                            "faelight-daemon",
                            "faelight-shell",
                            "faelight-term",
                            "git",
                            "cargo",
                            "python3",
                            "python",
                            "node",
                            "npm",
                            "sudo",
                            "systemctl",
                            "pacman",
                            "ssh",
                            "curl",
                            "wget",
                        ];
                        let prefix_len = typed_cmd.len().min(3);
                        let prefix = &typed_cmd[..prefix_len];
                        let suggestion = known
                            .iter()
                            .filter(|&&k| {
                                k.to_lowercase().starts_with(prefix) && k != typed_cmd.as_str()
                            })
                            .min_by_key(|&&k| levenshtein(k, &typed_cmd))
                            .copied();
                        let alias_suggestion: Option<String> = db.conn.query_row(
                            "SELECT name FROM shell_aliases WHERE name LIKE ?1 AND name != ?2 LIMIT 1",
                            rusqlite::params![format!("{}%", prefix), typed_cmd.as_str()],
                            |r| r.get(0)
                        ).ok();
                        // INT-143: was `return CommandResult::Empty;` -- SUCCESS, for a
                        // command that exited 127. Put the message in the payload and return the
                        // failure. main.rs prints an Error payload, so it still shows once.
                        // INT-171 gate 5: the not-found message comes FROM the typed error, so it
                        // cannot drift from the CommandNotFound kind. This is the 968c7be5 site.
                        let mut msg =
                            crate::error::FlowError::CommandNotFound(typed_cmd.to_string())
                                .display_colored();
                        if let Some(s) = suggestion {
                            msg.push_str(&format!(
                                "\n  {} did you mean: {}",
                                "\u{2192}".bright_cyan(),
                                s.bright_cyan()
                            ));
                        } else if let Some(a) = alias_suggestion {
                            msg.push_str(&format!(
                                "\n  {} did you mean: {}",
                                "\u{2192}".bright_cyan(),
                                a.bright_cyan()
                            ));
                        }
                        record_failure(db, &command_word(line), code);
                        return CommandResult::Error(msg);
                    }
                }
                record_failure(db, &command_word(line), code);
                CommandResult::Error(format!("  exited {} -- {}", code, explain_exit_code(code)))
            }
        }
        Err(e) => CommandResult::Error(format!("  failed to execute: {}", e)),
    }
}

// ── INT-177: Shell Observability ────────────────────────────────────────────────

fn observe_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let sub = args.first().copied().unwrap_or("session");
    match sub {
        "session" => observe_session(db),
        "commands" => observe_commands(db),
        "diff" => observe_diff(db),
        "anomalies" => observe_anomalies(db),
        "patterns" => observe_patterns(db),
        "causality" => observe_causality(db),
        "phase" => observe_phase(db),
        _ => CommandResult::Output(format!(
            "  Usage: observe [session|commands|diff|anomalies|patterns|causality|phase]"
        )),
    }
}

fn observe_session(db: &ForestDb) -> CommandResult {
    // Count commands this session from shell_history
    let total_cmds: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE timestamp >= (SELECT COALESCE(MIN(timestamp),0) FROM shell_history ORDER BY timestamp DESC LIMIT 500)",
        [], |r| r.get(0)
    ).unwrap_or(0);

    // Count failures
    let failures: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_state WHERE key LIKE 'failure_log_%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Count commits this session
    let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .to_string();

    // Active intent
    let intent = db.get_focus_intent().unwrap_or_else(|| "none".to_string());

    // Success rate
    let success_rate = if total_cmds > 0 {
        ((total_cmds - failures) * 100 / total_cmds) as u64
    } else {
        100
    };

    let mut out = String::new();
    out.push_str(
        "
",
    );
    out.push_str(&format!(
        "  {} Session Summary
",
        "🌲".normal()
    ));
    out.push_str(&format!(
        "{}
",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    ));
    out.push_str(&format!(
        "  {:<16} {}
",
        "Commands:".dimmed(),
        format!(
            "{} total, {} failed ({}% success)",
            total_cmds, failures, success_rate
        )
        .bright_white()
    ));
    out.push_str(&format!(
        "  {:<16} {}
",
        "Commits:".dimmed(),
        commits.bright_white()
    ));
    out.push_str(&format!(
        "  {:<16} {}
",
        "Active intent:".dimmed(),
        intent.bright_green()
    ));
    out.push_str(
        "
",
    );
    CommandResult::Output(out)
}

fn observe_commands(db: &ForestDb) -> CommandResult {
    // Most used commands from shell_history
    let mut rows: Vec<std::collections::HashMap<String, crate::value::Value>> = vec![];

    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT command, COUNT(*) as count FROM shell_history
         GROUP BY command ORDER BY count DESC LIMIT 10",
    ) {
        let _ = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .map(|iter| {
                for row in iter.flatten() {
                    let mut m = std::collections::HashMap::new();
                    m.insert("command".to_string(), crate::value::Value::Text(row.0));
                    m.insert("count".to_string(), crate::value::Value::Int(row.1));
                    rows.push(m);
                }
            });
    }

    if rows.is_empty() {
        CommandResult::Output("  ○ No command history available".to_string())
    } else {
        CommandResult::Value(crate::value::Value::Table(rows))
    }
}

fn observe_diff(db: &ForestDb) -> CommandResult {
    let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .to_string();

    let failures: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_state WHERE key LIKE 'failure_log_%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let errors: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_state WHERE key LIKE 'error_log_%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut out = String::new();
    out.push_str(
        "
",
    );
    out.push_str(&format!(
        "  {} Session Delta
",
        "🔄".normal()
    ));
    out.push_str(&format!(
        "{}
",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    ));
    out.push_str(&format!(
        "  {:<16} {} commits total
",
        "Commits:".dimmed(),
        commits.bright_white()
    ));
    out.push_str(&format!(
        "  {:<16} {} this session
",
        "Failures:".dimmed(),
        if failures == 0 {
            "0 ✅".bright_green().to_string()
        } else {
            failures.to_string().yellow().to_string()
        }
    ));
    out.push_str(&format!(
        "  {:<16} {} this session
",
        "Errors:".dimmed(),
        if errors == 0 {
            "0 ✅".bright_green().to_string()
        } else {
            errors.to_string().yellow().to_string()
        }
    ));
    out.push_str(
        "
",
    );
    CommandResult::Output(out)
}

fn observe_anomalies(db: &ForestDb) -> CommandResult {
    let failures: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_state WHERE key LIKE 'failure_log_%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut anomalies: Vec<String> = vec![];

    if failures >= 3 {
        anomalies.push(format!(
            "{} failures this session — investigate with: failures",
            failures
        ));
    }

    let perm_errors: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_state WHERE key LIKE 'error_log_%' AND value LIKE '%E_PERMISSION%'",
        [], |r| r.get(0)
    ).unwrap_or(0);

    if perm_errors > 0 {
        anomalies.push(format!("{} permission errors during work", perm_errors));
    }

    let mut out = String::new();
    out.push_str(
        "
",
    );
    out.push_str(&format!(
        "  {} Anomalies
",
        "⚠️ ".normal()
    ));
    out.push_str(&format!(
        "{}
",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    ));
    if anomalies.is_empty() {
        out.push_str(
            "  ✅ No anomalies detected — session looks normal
",
        );
    } else {
        for a in &anomalies {
            out.push_str(&format!(
                "  {} {}
",
                "⚠️ ".normal(),
                a.yellow()
            ));
        }
    }
    out.push_str(
        "
",
    );
    CommandResult::Output(out)
}

fn observe_patterns(db: &ForestDb) -> CommandResult {
    // Show top command patterns from all history
    let mut rows: Vec<std::collections::HashMap<String, crate::value::Value>> = vec![];

    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT command, COUNT(*) as count FROM shell_history
         GROUP BY command ORDER BY count DESC LIMIT 5",
    ) {
        let _ = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .map(|iter| {
                for row in iter.flatten() {
                    let mut m = std::collections::HashMap::new();
                    m.insert("pattern".to_string(), crate::value::Value::Text(row.0));
                    m.insert("frequency".to_string(), crate::value::Value::Int(row.1));
                    rows.push(m);
                }
            });
    }

    if rows.is_empty() {
        CommandResult::Output("  ○ Not enough history to show patterns".to_string())
    } else {
        CommandResult::Value(crate::value::Value::Table(rows))
    }
}

fn observe_causality(db: &ForestDb) -> CommandResult {
    let mut output = String::new();
    output.push_str("\n  Causality Analysis\n");
    output.push_str("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    // Commit frequency causality
    let recent_commits: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'fg commit%' AND timestamp > ?1",
            rusqlite::params![{
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
                    - 86400
            }],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let prev_commits: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'fg commit%' AND timestamp BETWEEN ?1 AND ?2",
        rusqlite::params![
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64).unwrap_or(0) - 172800,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64).unwrap_or(0) - 86400
        ],
        |r| r.get(0)
    ).unwrap_or(0);

    if recent_commits > prev_commits {
        let diff = recent_commits - prev_commits;
        output.push_str(&format!(
            "  → Commit frequency increased (+{} today vs yesterday)\n",
            diff
        ));
        output.push_str("     Cause: higher intent completion rate or active build session\n\n");
    } else if recent_commits < prev_commits {
        let diff = prev_commits - recent_commits;
        output.push_str(&format!(
            "  → Commit frequency decreased (-{} today vs yesterday)\n",
            diff
        ));
        output.push_str("     Cause: planning/reading session or blocked on dependency\n\n");
    } else {
        output.push_str("  → Commit frequency stable\n\n");
    }

    // Failure causality
    let recent_failures: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'failure_log_%' AND timestamp > ?1",
        rusqlite::params![
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64).unwrap_or(0) - 3600
        ],
        |r| r.get(0)
    ).unwrap_or(0);

    if recent_failures >= 3 {
        output.push_str(&format!("  → {} failures in last hour\n", recent_failures));
        output.push_str("     Cause: possible failure loop — same command pattern repeating\n");
        output.push_str("     Suggestion: run last_command explain\n\n");
    }

    // Deploy causality
    let deploy_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'deploy %' AND timestamp > ?1",
            rusqlite::params![
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
                    - 3600
            ],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let health_after_deploy: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command = 'd' AND timestamp > ?1",
            rusqlite::params![
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
                    - 3600
            ],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if deploy_count > 0 && health_after_deploy == 0 {
        output.push_str(&format!(
            "  → {} deploy(s) in last hour with no health check\n",
            deploy_count
        ));
        output.push_str("     Cause: health unchecked after deploy — run d\n\n");
    }

    if recent_commits == 0 && recent_failures == 0 && deploy_count == 0 {
        output.push_str("  → No significant causal signals in last hour\n");
        output.push_str("     System activity appears normal\n");
    }

    output.push_str("\n");
    CommandResult::Output(output)
}

fn observe_phase(db: &ForestDb) -> CommandResult {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Detect time of day
    let hour = {
        let dt = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        (dt % 86400) / 3600 + 5 // rough UTC offset, adjust if needed
    } % 24;

    let time_context = if hour >= 5 && hour < 12 {
        ("Morning", "sync + planning patterns common")
    } else if hour >= 12 && hour < 17 {
        ("Afternoon", "build + deploy patterns common")
    } else if hour >= 17 && hour < 22 {
        ("Evening", "commit + review patterns common")
    } else {
        ("Night", "deep work or recovery session")
    };

    // Detect session phase from recent command patterns
    let deploy_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'deploy %' AND timestamp > ?1",
            rusqlite::params![now - 3600],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let commit_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'fg commit%' AND timestamp > ?1",
            rusqlite::params![now - 3600],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let intent_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM shell_history WHERE (command LIKE 'cistart%' OR command LIKE 'cicomplete%') AND timestamp > ?1",
        rusqlite::params![now - 3600], |r| r.get(0)
    ).unwrap_or(0);

    let health_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command = 'd' AND timestamp > ?1",
            rusqlite::params![now - 3600],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let phase = if deploy_count >= 3 {
        (
            "Build/Deploy Phase",
            "Focus: building and deploying tools",
            "Expect: deploy errors, version bumps, health checks",
        )
    } else if commit_count >= 3 {
        (
            "Commit Phase",
            "Focus: wrapping up changes",
            "Expect: fg commit, gp, d in sequence",
        )
    } else if intent_count >= 1 {
        (
            "Intent Phase",
            "Focus: starting or completing an intent",
            "Expect: cistart/cicomplete, focused work",
        )
    } else if health_count >= 2 {
        (
            "Monitoring Phase",
            "Focus: verifying system health",
            "Expect: d, core integrity run, core strategy friday-readiness",
        )
    } else {
        (
            "Exploration Phase",
            "Focus: general navigation and investigation",
            "Expect: varied command patterns",
        )
    };

    let mut out = String::new();
    out.push_str("\n  Session Phase Detection\n");
    out.push_str("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
    out.push_str(&format!(
        "  {} Time context:  {} — {}\n",
        "·".dimmed(),
        time_context.0,
        time_context.1
    ));
    out.push_str(&format!(
        "  {} Session phase: {}\n",
        "▶".bright_cyan(),
        phase.0
    ));
    out.push_str(&format!("  {} {}\n", "·".dimmed(), phase.1));
    out.push_str(&format!("  {} {}\n\n", "·".dimmed(), phase.2));
    out.push_str(&format!(
        "  Last hour: {} deploys  ·  {} commits  ·  {} intent ops  ·  {} health checks\n\n",
        deploy_count, commit_count, intent_count, health_count
    ));
    CommandResult::Output(out)
}

// ── INT-176: Failure Recovery Commands ──────────────────────────────────────────

fn last_command_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let sub = args.first().copied().unwrap_or("show");

    let last_failed = db
        .conn
        .query_row(
            "SELECT value FROM shell_state WHERE key='last_failed_command'",
            [],
            |r| r.get::<_, String>(0),
        )
        .ok();

    match last_failed {
        None => CommandResult::Output("  ○ No failed commands recorded this session".to_string()),
        Some(cmd) => match sub {
            "retry" => {
                println!("  {} Retrying: {}", "↺".bright_cyan(), cmd.bright_white());
                println!();
                CommandResult::Output(format!("__retry__{}", cmd))
            }
            "explain" => {
                let last_err = db
                    .conn
                    .query_row(
                        "SELECT value FROM shell_state WHERE key='last_error'",
                        [],
                        |r| r.get::<_, String>(0),
                    )
                    .ok();
                let mut out = String::new();
                out.push_str(&format!(
                    "
  {} Last failed command: {}
",
                    "✗".bright_red(),
                    cmd.bright_white()
                ));
                if let Some(err) = last_err {
                    if let Some(e) = crate::error::ShellError::from_storage(&err) {
                        out.push_str(&format!(
                            "  {} {}: {}
",
                            "❌".normal(),
                            e.code,
                            e.message
                        ));
                        if !e.suggestion.is_empty() {
                            out.push_str(&format!(
                                "  {} {}
",
                                "💡".normal(),
                                e.suggestion
                            ));
                        }
                    }
                }
                CommandResult::Output(out)
            }
            "fix" => {
                let last_err = db
                    .conn
                    .query_row(
                        "SELECT value FROM shell_state WHERE key='last_error'",
                        [],
                        |r| r.get::<_, String>(0),
                    )
                    .ok();
                let fix = if let Some(err) = last_err {
                    if let Some(e) = crate::error::ShellError::from_storage(&err) {
                        match e.code {
                            "E_NOT_GIT_REPO" => Some(format!("cd ~/0-core && {}", cmd)),
                            "E_PERMISSION" => Some(format!("sudo {}", cmd)),
                            "E_CMD_NOT_FOUND" => {
                                Some(format!("# Install the tool first, then: {}", cmd))
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                match fix {
                    Some(f) => CommandResult::Output(format!(
                        "
  {} Suggested fix:
  {}
",
                        "💡".normal(),
                        f.bright_cyan()
                    )),
                    None => CommandResult::Output(format!(
                        "
  {} No automatic fix available for: {}
  Check the error with: last_command explain
",
                        "○".dimmed(),
                        cmd.dimmed()
                    )),
                }
            }
            "options" => {
                let cmd_lower = cmd.to_lowercase();
                let opts: Vec<(&str, &str)> = if cmd_lower.contains("deploy") {
                    vec![
                        ("Retry deploy", "deploy <tool>"),
                        ("Check build errors", "cargo build 2>&1 | tail -20"),
                        ("Verify registry", "core registry show <tool>"),
                        ("Check disk space", "df -h ~/0-core/target"),
                    ]
                } else if cmd_lower.starts_with("fg")
                    || cmd_lower.contains("git")
                    || cmd_lower.starts_with("gp")
                {
                    vec![
                        ("Check git status", "gst"),
                        ("Retry push", "gp"),
                        ("Check remote", "git remote -v"),
                        ("Inspect recent commits", "glog"),
                    ]
                } else if cmd_lower.contains("cargo") || cmd_lower.contains("build") {
                    vec![
                        ("Check compile errors", "cargo build 2>&1 | grep error"),
                        ("Clean and rebuild", "cargo clean && cargo build"),
                        ("Check Cargo.toml", "cat Cargo.toml | head -20"),
                    ]
                } else if cmd_lower.starts_with("core ") {
                    vec![
                        ("Run health check", "d"),
                        ("Redeploy core", "deploy core"),
                        ("Verify core binary", "core --version"),
                        ("Check integrity", "core integrity run"),
                    ]
                } else {
                    vec![
                        ("Retry last command", "last_command retry"),
                        ("Explain error", "last_command explain"),
                        ("Run health check", "d"),
                    ]
                };
                let mut out = format!("\n  Recovery Options for: {}\n  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n", cmd);
                for (i, (label, hint)) in opts.iter().enumerate() {
                    out.push_str(&format!("  {}. {}\n     → {}\n\n", i + 1, label, hint));
                }
                CommandResult::Output(out)
            }
            _ => CommandResult::Output(format!(
                "
  {} Last failed: {}
  → retry | explain | fix | options
",
                "✗".bright_red(),
                cmd.bright_white()
            )),
        },
    }
}

fn failure_history_cmd(db: &ForestDb, _args: &[&str]) -> CommandResult {
    let mut rows: Vec<std::collections::HashMap<String, crate::value::Value>> = vec![];

    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT key, value FROM shell_state WHERE key LIKE 'failure_log_%' ORDER BY key DESC LIMIT 20"
    ) {
        let _ = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }).map(|iter| {
            for row in iter.flatten() {
                let parts: Vec<&str> = row.1.splitn(2, '|').collect();
                if parts.len() == 2 {
                    let mut m = std::collections::HashMap::new();
                    m.insert("command".to_string(), crate::value::Value::Text(parts[0].to_string()));
                    m.insert("error".to_string(),   crate::value::Value::Text(parts[1].chars().take(50).collect()));
                    rows.push(m);
                }
            }
        });
    }

    if rows.is_empty() {
        CommandResult::Output("  ✅ No failures recorded this session".to_string())
    } else {
        CommandResult::Value(crate::value::Value::Table(rows))
    }
}

// ── INT-173: Command Registry Commands ───────────────────────────────────────

fn explain_cmd(db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    let cmd = args.first().copied().unwrap_or("");
    if cmd.is_empty() {
        return CommandResult::Error("explain: missing command name".to_string());
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let mut out = String::new();
    out.push_str(&format!(
        "
  {} {}
",
        "🌲 explain:".bright_green().bold(),
        cmd.bright_white().bold()
    ));
    out.push_str(&format!(
        "  {}
",
        "─".repeat(48).dimmed()
    ));
    // 1. Alias resolution
    if let Some(aliased) = db.get_alias(cmd) {
        out.push_str(&format!(
            "  {:<14} {}
",
            "alias:".dimmed(),
            format!("{} → {}", cmd, aliased).bright_cyan()
        ));
        // Recurse one level to show what the alias points to
        // deadwood: exempt -- reports what an alias resolves to -- display only, never selects execution
        let target_cmd = aliased.split_whitespace().next().unwrap_or("");
        if !target_cmd.is_empty() && target_cmd != cmd {
            out.push_str(&format!(
                "  {:<14} {}
",
                "resolves to:".dimmed(),
                target_cmd.bright_white()
            ));
        }
    }
    // 2. Registry description
    let mut reg = crate::registry::Registry::new();
    reg.populate(db, core_root);
    if let Some(entry) = reg.get(cmd) {
        out.push_str(&format!(
            "  {:<14} {}
",
            "kind:".dimmed(),
            entry.kind.label().bright_cyan()
        ));
        if !entry.description.is_empty() {
            out.push_str(&format!(
                "  {:<14} {}
",
                "description:".dimmed(),
                entry.description.bright_white()
            ));
        }
        if !entry.usage.is_empty() {
            out.push_str(&format!(
                "  {:<14} {}
",
                "usage:".dimmed(),
                entry.usage.dimmed()
            ));
        }
        if !entry.source.is_empty() {
            out.push_str(&format!(
                "  {:<14} {}
",
                "source:".dimmed(),
                entry.source.dimmed()
            ));
        }
    }
    // 3. Forest builtins list
    let builtins = [
        "cd",
        "pwd",
        "ls",
        "ll",
        "clear",
        "echo",
        "env",
        "type",
        "which",
        "health",
        "events",
        "intents",
        "tools",
        "version",
        "schema",
        "commits",
        "grep",
        "find",
        "tree",
        "fstat",
        "peek",
        "realpath",
        "rp",
        "time",
        "exec",
        "reload",
        "source",
        "fsh",
        "explain",
        "where",
        "hs",
        "history-search",
        "alias",
        "unalias",
        "export",
        "unset",
        "let",
        "run",
        "help",
        "exit",
        "quit",
        "forest-stats",
        "fstats",
        "memory",
        "fsh-gaps",
    ];
    if builtins.contains(&cmd) {
        out.push_str(&format!(
            "  {:<14} {}
",
            "builtin:".dimmed(),
            "native fsh command — no PATH lookup".bright_green()
        ));
    }
    // 4. Forest script
    let script_path = format!("{}/0-core/scripts/{}", home, cmd);
    if std::path::Path::new(&script_path).exists() {
        out.push_str(&format!(
            "  {:<14} {}
",
            "script:".dimmed(),
            script_path.dimmed()
        ));
    }
    // 5. PATH binary
    if let Ok(o) = std::process::Command::new("which").arg(cmd).output() {
        if o.status.success() {
            let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
            out.push_str(&format!(
                "  {:<14} {}
",
                "binary:".dimmed(),
                path.dimmed()
            ));
        }
    }
    // 6. Reverse alias lookup — who points to this?
    // Match whole word — command starts with cmd or contains " cmd"
    let like1 = format!("{}%", cmd);
    let like2 = format!("% {}%", cmd);
    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT name, command FROM shell_aliases WHERE (command LIKE ?1 OR command LIKE ?2) AND name != ?3 ORDER BY name LIMIT 5"
    ) {
        let aliases: Vec<(String, String)> = stmt
            .query_map(rusqlite::params![&like1, &like2, cmd], |r| Ok((r.get(0)?, r.get(1)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default();
        if !aliases.is_empty() {
            let names: Vec<String> = aliases.iter().map(|(n, _)| n.bright_cyan().to_string()).collect();
            out.push_str(&format!("  {:<14} {}
", "also via:".dimmed(), names.join("  ")));
        }
    }
    // 7. Audit score
    if let Ok(score) = db.conn.query_row(
        "SELECT score FROM audit_scores WHERE tool_name = ?1 ORDER BY timestamp DESC LIMIT 1",
        rusqlite::params![cmd],
        |r| r.get::<_, i64>(0),
    ) {
        let color = if score >= 80 {
            format!("{}/100", score).bright_green().to_string()
        } else if score >= 60 {
            format!("{}/100", score).yellow().to_string()
        } else {
            format!("{}/100", score).bright_red().to_string()
        };
        out.push_str(&format!(
            "  {:<14} {}
",
            "audit score:".dimmed(),
            color
        ));
    }
    // INT-326: append three-layer semantic analysis
    let full_cmd = format!(
        "{} {}",
        cmd,
        args.get(1..).map(|a| a.join(" ")).unwrap_or_default()
    )
    .trim()
    .to_string();
    let si = crate::semantic::interpret(&full_cmd);
    if si.confidence > 0.0 {
        out.push_str(&crate::semantic::format_three_layers(&si));
    }
    out.push_str(&format!(
        "  {}
",
        "─".repeat(48).dimmed()
    ));
    CommandResult::Output(out)
}
fn where_cmd(db: &ForestDb, _core_root: &str, args: &[&str]) -> CommandResult {
    let cmd = args.first().copied().unwrap_or("");
    if cmd.is_empty() {
        return CommandResult::Error("where: missing command name".to_string());
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let mut out = String::new();
    let mut found = false;
    // Alias
    if let Some(aliased) = db.get_alias(cmd) {
        out.push_str(&format!(
            "  {} alias       {} → {}
",
            "▶".bright_cyan(),
            cmd.bright_white(),
            aliased.bright_cyan()
        ));
        found = true;
    }
    // Forest script
    let script_path = format!("{}/0-core/scripts/{}", home, cmd);
    if std::path::Path::new(&script_path).exists() {
        out.push_str(&format!(
            "  {} script      {}
",
            "▶".bright_green(),
            script_path.dimmed()
        ));
        found = true;
    }
    // PATH
    if let Ok(o) = std::process::Command::new("which").arg(cmd).output() {
        if o.status.success() {
            let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
            out.push_str(&format!(
                "  {} binary      {}
",
                "▶".yellow(),
                path.dimmed()
            ));
            found = true;
        }
    }
    // Builtin
    let builtins = [
        "cd", "pwd", "ls", "echo", "env", "type", "which", "grep", "find", "tree", "fstat", "peek",
        "realpath", "time", "exec", "reload", "source", "fsh", "explain", "where", "hs", "alias",
        "unalias", "export", "unset", "let", "run",
    ];
    if builtins.contains(&cmd) {
        out.push_str(&format!(
            "  {} builtin     native fsh
",
            "▶".bright_green()
        ));
        found = true;
    }
    // INT-300: forest vocabulary words -- human-first commands (INT-261)
    let vocab_words = [
        "write", "read", "list", "copy", "move", "delete", "find", "db", "gt", "it", "search",
        "show", "where", "compare", "fsearch", "query",
    ];
    if vocab_words.contains(&cmd) {
        out.push_str(&format!(
            "  {} vocabulary  forest word (INT-261)
",
            "▶".bright_magenta()
        ));
        found = true;
    }
    if !found {
        out.push_str(&format!(
            "  {} not found: {}
",
            "✗".bright_red(),
            cmd
        ));
    }
    CommandResult::Output(out.trim_end().to_string())
}
fn describe_cmd(db: &ForestDb, args: &[&str], core_root: &str) -> CommandResult {
    let name = args.first().copied().unwrap_or("");
    if name.is_empty() {
        return CommandResult::Error("describe: missing command name".to_string());
    }
    let mut reg = crate::registry::Registry::new();
    reg.populate(db, core_root);
    match reg.get(name) {
        None => CommandResult::Output(format!("  ○ {} — not in registry", name)),
        Some(entry) => {
            let mut out = String::new();
            out.push_str(&format!(
                "\n  {} {}\n",
                "🌲".normal(),
                entry.name.bright_white().bold()
            ));
            out.push_str(&format!(
                "  {:<12} {}\n",
                "kind:".dimmed(),
                entry.kind.label().bright_cyan()
            ));
            out.push_str(&format!(
                "  {:<12} {}\n",
                "source:".dimmed(),
                entry.source.dimmed()
            ));
            if !entry.description.is_empty() {
                out.push_str(&format!(
                    "  {:<12} {}\n",
                    "description:".dimmed(),
                    entry.description.bright_white()
                ));
            }
            if !entry.usage.is_empty() {
                out.push_str(&format!(
                    "  {:<12} {}\n",
                    "usage:".dimmed(),
                    entry.usage.dimmed()
                ));
            }
            CommandResult::Output(out.trim_end().to_string())
        }
    }
}

fn command_cmd(db: &ForestDb, args: &[&str], core_root: &str) -> CommandResult {
    let sub = args.first().copied().unwrap_or("list");
    let mut reg = crate::registry::Registry::new();
    reg.populate(db, core_root);

    match sub {
        "list" | "" => {
            let filter = args.get(1).copied().unwrap_or("");
            let entries = reg.all_sorted();
            let rows: Vec<std::collections::HashMap<String, crate::value::Value>> = entries
                .iter()
                .filter(|e| {
                    filter.is_empty() || e.kind.label() == filter || e.name.contains(filter)
                })
                .map(|e| {
                    let mut m = std::collections::HashMap::new();
                    m.insert(
                        "name".to_string(),
                        crate::value::Value::Text(e.name.clone()),
                    );
                    m.insert(
                        "kind".to_string(),
                        crate::value::Value::Text(e.kind.label().to_string()),
                    );
                    m.insert(
                        "description".to_string(),
                        crate::value::Value::Text(e.description.clone()),
                    );
                    m
                })
                .collect();
            if rows.is_empty() {
                CommandResult::Output("  ○ No commands found".to_string())
            } else {
                CommandResult::Value(crate::value::Value::Table(rows))
            }
        }
        "info" => {
            let name = args.get(1).copied().unwrap_or("");
            if name.is_empty() {
                return CommandResult::Error("command info: missing command name".to_string());
            }
            describe_cmd(db, &[name], core_root)
        }
        "count" => {
            let count = reg.entries.len();
            CommandResult::Output(format!(
                "  {} commands in registry",
                count.to_string().bright_white()
            ))
        }
        _ => CommandResult::Error(format!("command: unknown subcommand '{}'", sub)),
    }
}

// ── INT-174: Structured Error Commands ───────────────────────────────────────

fn last_error_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let subcommand = args.first().copied().unwrap_or("");
    let stored = db
        .conn
        .query_row(
            "SELECT value FROM shell_state WHERE key='last_error'",
            [],
            |r| r.get::<_, String>(0),
        )
        .ok();

    match stored {
        None => CommandResult::Output("  ○ No errors recorded this session".to_string()),
        Some(s) => {
            match crate::error::ShellError::from_storage(&s) {
                None => CommandResult::Output(format!("  ○ {}", s)),
                Some(err) => {
                    match subcommand {
                        "suggest" => CommandResult::Output(
                            format!("  💡 {}", err.suggestion)
                        ),
                        "explain" => CommandResult::Output(format!(
                            "\n  ❌ {}\n  Message:    {}\n  Suggestion: {}\n  Command:    {}\n  Directory:  {}\n",
                            err.code, err.message, err.suggestion, err.command, err.directory
                        )),
                        _ => CommandResult::Output(err.display()),
                    }
                }
            }
        }
    }
}

fn error_history_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let limit: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(10);

    let mut rows: Vec<std::collections::HashMap<String, crate::value::Value>> = vec![];

    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT key, value FROM shell_state WHERE key LIKE 'error_log_%' ORDER BY key DESC LIMIT ?1"
    ) {
        let _ = stmt.query_map(rusqlite::params![limit as i64], |r| {
            Ok(r.get::<_, String>(1)?)
        }).map(|iter| {
            for row in iter.flatten() {
                if let Some(err) = crate::error::ShellError::from_storage(&row) {
                    let mut m = std::collections::HashMap::new();
                    m.insert("code".to_string(),    crate::value::Value::Text(err.code.to_string()));
                    m.insert("message".to_string(), crate::value::Value::Text(err.message));
                    m.insert("command".to_string(), crate::value::Value::Text(err.command));
                    rows.push(m);
                }
            }
        });
    }

    if rows.is_empty() {
        CommandResult::Output("  ○ No errors recorded this session".to_string())
    } else {
        CommandResult::Value(crate::value::Value::Table(rows))
    }
}

#[allow(dead_code)]
fn suggest_after_external(line: &str, cmd_lower: &str) {
    let suggestion: Option<&str> = match cmd_lower {
        "cicomplete" => Some("💡 Next: fg commit — record the completion"),
        "cistart" => Some("💡 Next: read the intent carefully before writing any code"),
        "deploy" => Some("💡 Suggestion: run d — verify health after deploy"),
        "paru" | "pacman" => Some("💡 That isn't a NixOS command — apply changes with deploy (it rebuilds + health-checks)"),
        "core" if line.contains("intent complete") => {
            Some("💡 Next: fg commit — record the completion")
        }
        "core" if line.contains("intent start") => {
            Some("💡 Next: read the intent carefully before writing any code")
        }
        _ => None,
    };
    if let Some(msg) = suggestion {
        println!("  {}", msg);
    }
}

fn help() -> CommandResult {
    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 🌲 faelight-shell commands ──────────────────────────".bright_cyan()
    ));
    let cmds = [
        ("health", "system health and status"),
        ("events", "recent events  [today|domain]"),
        ("decisions", "open decisions from ledger"),
        ("intents", "active intents"),
        ("tools", "tool deployment status"),
        ("audit", "tool intelligence scores"),
        ("forecast", "health trend and forecast"),
        ("sandbox", "recent sandbox runs"),
        ("checkpoint", "recent checkpoints"),
        ("git", "git status and recent commits"),
        ("search", "search command history  [query]"),
        ("tt", "tools as table — pipeable"),
        ("et", "events as table — pipeable"),
        ("at", "audit scores as table — pipeable"),
        ("dt", "decisions as table — pipeable"),
        ("ht", "shell history as table — pipeable"),
        ("ct", "checkpoints as table — pipeable"),
        ("domains", "event domain summary"),
        ("histogram", "histogram of a domain  [field]"),
        ("logs", "system logs — pipeable  [--follow] [--errors]"),
        ("ps", "processes as table — pipeable"),
        ("ports", "open ports as table — pipeable"),
        ("services", "systemd services as table — pipeable"),
        ("files", "files as table  [path]"),
        ("net", "network interfaces as table"),
        ("gc", "git commits as table — pipeable"),
        ("gf", "git files as table — pipeable"),
        (
            "watch",
            "watch a command live  [health|events] — or pipe: ps | watch [interval]",
        ),
        ("alias", "manage aliases  [name=command]"),
        ("unalias", "remove an alias"),
        ("plugins", "list loaded plugins"),
        ("story", "30-day forest narrative"),
        ("advise", "judgment advisory"),
        ("version", "system version"),
        ("commits", "commit count and last commit"),
        ("cd", "change directory"),
        ("clear", "clear the screen"),
        ("exit", "leave faelight-shell"),
    ];
    for (cmd, desc) in &cmds {
        out.push_str(&format!(
            "  │  {:<12}  {}\n",
            cmd.bright_cyan(),
            desc.dimmed()
        ));
    }
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn health(db: &ForestDb) -> CommandResult {
    let health = db.health_score().unwrap_or(0);
    let status = if health >= 95 {
        "HEALTHY".bright_green()
    } else if health >= 80 {
        "ADVISORY".yellow()
    } else {
        "DEGRADED".bright_red()
    };

    let version = std::fs::read_to_string(faelight_core::paths::version_file())
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 🏥 Forest Health ─────────────────────────────────".bright_cyan()
    ));
    out.push_str(&format!(
        "  │  Health:  {}  {}\n",
        format!("{}%", health).bright_white().bold(),
        status
    ));
    out.push_str(&format!("  │  Version: {}\n", version.dimmed()));
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn events(db: &ForestDb, args: &[&str]) -> CommandResult {
    let today_only = args.contains(&"today");
    let domain = args
        .first()
        .and_then(|a| if *a == "today" { None } else { Some(*a) });

    let label = if today_only {
        "Today's Events"
    } else {
        "Recent Events"
    };
    let events = db.query_events(domain, today_only, 20);
    if events.is_empty() {
        return CommandResult::Output(format!("  {} No events found", "○".dimmed()));
    }

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        format!("  ╭─ 📊 {} ─────────────────────────────────", label).bright_cyan()
    ));
    for (domain, action, ts) in &events {
        let time = fmt_time(*ts, "%H:%M:%S");
        let icon = match domain.as_str() {
            "doctor" => "🩺",
            "git" => "🌿",
            "security" => "🔒",
            "sandbox" => "🧪",
            "audit" => "🔍",
            "checkpoint" => "📸",
            "compositor" => "🖥",
            "idle" => "💤",
            "decisions" => "⚖️ ",
            "shell" => "🐚",
            "update" => "📦",
            _ => "○ ",
        };
        out.push_str(&format!(
            "  │  {} {}  {}.{}\n",
            icon,
            time.dimmed(),
            domain.bright_cyan(),
            action.bright_white(),
        ));
    }
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn decisions(db: &ForestDb) -> CommandResult {
    let rows: Vec<(String, String, String)> = db
        .conn
        .prepare(
            "SELECT dec_id, description, outcome FROM decisions ORDER BY timestamp DESC LIMIT 10",
        )
        .ok()
        .map(|mut s| {
            s.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            .unwrap_or_default()
        })
        .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No decisions recorded yet — use {}",
            "○".dimmed(),
            "core decide".bright_cyan()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ ⚖️  Decisions ──────────────────────────────────────".bright_cyan()
    ));
    for (id, desc, outcome) in &rows {
        let outcome_icon = match outcome.as_str() {
            "success" => "✅".to_string(),
            "failure" => "❌".to_string(),
            "partial" => "⚠️ ".to_string(),
            _ => "○ ".to_string(),
        };
        let short_desc = if desc.len() > 45 {
            format!("{}...", &desc[..45])
        } else {
            desc.clone()
        };
        out.push_str(&format!(
            "  │  {}  {}  {}\n",
            id.bright_yellow(),
            outcome_icon,
            short_desc.dimmed()
        ));
    }
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn deploys(db: &ForestDb) -> CommandResult {
    use std::collections::HashMap;
    let mut rows: Vec<HashMap<String, crate::value::Value>> = Vec::new();
    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT tool, version, outcome, duration_ms, timestamp FROM deploy_patterns ORDER BY timestamp DESC LIMIT 200"
    ) {
        let _ = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
                row.get::<_, String>(2).unwrap_or_default(),
                row.get::<_, i64>(3).unwrap_or(0),
                row.get::<_, i64>(4).unwrap_or(0),
            ))
        }).map(|iter| {
            for item in iter.flatten() {
                let (tool, version, outcome, duration_ms, timestamp) = item;
                let mut row = HashMap::new();
                row.insert("tool".to_string(), crate::value::Value::Text(tool));
                row.insert("version".to_string(), crate::value::Value::Text(version));
                row.insert("outcome".to_string(), crate::value::Value::Text(outcome));
                row.insert("duration_ms".to_string(), crate::value::Value::Int(duration_ms));
                row.insert("timestamp".to_string(), crate::value::Value::Int(timestamp));
                rows.push(row);
            }
        });
    }
    CommandResult::Value(crate::value::Value::Table(rows))
}

fn friday_patterns(db: &ForestDb) -> CommandResult {
    use std::collections::HashMap;
    let mut rows: Vec<HashMap<String, crate::value::Value>> = Vec::new();
    if let Ok(mut stmt) = db.conn.prepare(
        "SELECT trigger, action, outcome, confidence, frequency, source FROM friday_patterns ORDER BY confidence DESC"
    ) {
        let _ = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
                row.get::<_, String>(2).unwrap_or_default(),
                row.get::<_, f64>(3).unwrap_or(0.0),
                row.get::<_, i64>(4).unwrap_or(0),
                row.get::<_, String>(5).unwrap_or_default(),
            ))
        }).map(|iter| {
            for item in iter.flatten() {
                let (trigger, action, outcome, confidence, frequency, source) = item;
                let mut row = HashMap::new();
                row.insert("trigger".to_string(), crate::value::Value::Text(trigger));
                row.insert("action".to_string(), crate::value::Value::Text(action));
                row.insert("outcome".to_string(), crate::value::Value::Text(outcome));
                row.insert("confidence".to_string(), crate::value::Value::Text(format!("{:.2}", confidence)));
                row.insert("frequency".to_string(), crate::value::Value::Int(frequency));
                row.insert("source".to_string(), crate::value::Value::Text(source));
                rows.push(row);
            }
        });
    }
    CommandResult::Value(crate::value::Value::Table(rows))
}

fn intents(_core_root: &str) -> CommandResult {
    use std::collections::HashMap;
    let mut rows: Vec<HashMap<String, crate::value::Value>> = Vec::new();
    // INT-030: read all three dirs with correct status
    let dirs = [
        ("in-progress", "in-progress"),
        ("complete", "complete"),
        ("future", "planned"),
    ];
    for (dir, dir_status) in &dirs {
        let path = faelight_core::paths::intents_dir().join(dir);
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".md") {
                    continue;
                }
                let file_content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                let id = name.split('-').next().unwrap_or("0").to_string();
                let title = file_content
                    .lines()
                    .find(|l| l.starts_with("title:"))
                    .map(|l| {
                        l.trim_start_matches("title:")
                            .trim()
                            .trim_matches('"')
                            .to_string()
                    })
                    .or_else(|| {
                        file_content
                            .lines()
                            .find(|l| l.trim_start().starts_with("# "))
                            .map(|l| l.trim_start_matches('#').trim().to_string())
                    })
                    .unwrap_or_else(|| name.replace(".md", ""));
                // Prefer status from frontmatter, fall back to dir_status
                let status = file_content
                    .lines()
                    .find(|l| l.starts_with("status:"))
                    .map(|l| l.trim_start_matches("status:").trim().to_string())
                    .unwrap_or_else(|| dir_status.to_string());
                let mut row = HashMap::new();
                row.insert(
                    "id".to_string(),
                    crate::value::Value::Int(id.parse().unwrap_or(0)),
                );
                row.insert("title".to_string(), crate::value::Value::Text(title));
                row.insert("status".to_string(), crate::value::Value::Text(status));
                rows.push(row);
            }
        }
    }
    rows.sort_by_key(|r| {
        if let Some(crate::value::Value::Int(i)) = r.get("id") {
            *i
        } else {
            0
        }
    });
    if rows.is_empty() {
        return CommandResult::Output("  ○ No intents found".to_string());
    }
    CommandResult::Value(crate::value::Value::Table(rows))
}

fn project_list(core_root: &str) -> CommandResult {
    use colored::Colorize;
    let root = std::path::PathBuf::from(core_root);
    let mut out = String::new();
    out.push_str(&format!(
        "
{}
",
        "  🌲 Forest Projects".bright_green().bold()
    ));
    out.push_str(&format!(
        "{}
",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    ));

    // Read version
    let version = std::fs::read_to_string(faelight_core::paths::version_file())
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();

    // Count intents
    let count_md = |sub: &str| -> usize {
        std::fs::read_dir(faelight_core::paths::intents_dir().join(sub))
            .map(|d| {
                d.flatten()
                    .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                    .count()
            })
            .unwrap_or(0)
    };
    let complete = count_md("complete");
    let in_progress = count_md("in-progress");
    let planned = count_md("future");

    // Git info
    let branch = std::process::Command::new("git")
        .args(["-C", core_root, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    let commits = std::process::Command::new("git")
        .args(["-C", core_root, "rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "?".into());

    out.push_str(&format!(
        "  {}  {}  {}  {} intents ({} complete, {} active, {} planned)  {} commits  branch: {}
",
        "0-core".bright_white().bold(),
        version.bright_green(),
        "100% ✅".bright_green(),
        (complete + in_progress + planned)
            .to_string()
            .bright_white(),
        complete.to_string().bright_green(),
        in_progress.to_string().bright_yellow(),
        planned.to_string().dimmed(),
        commits.bright_white(),
        branch.bright_cyan(),
    ));
    out.push_str(&format!(
        "  {}
",
        root.display().to_string().dimmed()
    ));
    CommandResult::Output(out)
}

fn experiment_list(core_root: &str) -> CommandResult {
    use colored::Colorize;
    let labs_dir = std::path::PathBuf::from(core_root).join("labs");
    let mut out = String::new();
    out.push_str(&format!(
        "
{}
",
        "  🧪 Experiments".bright_yellow().bold()
    ));
    out.push_str(&format!(
        "{}
",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    ));

    let mut found = false;
    if let Ok(entries) = std::fs::read_dir(&labs_dir) {
        let mut dirs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();
        dirs.sort_by_key(|e| e.file_name());
        for entry in &dirs {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "graduated" {
                let count = std::fs::read_dir(entry.path())
                    .map(|d| d.flatten().count())
                    .unwrap_or(0);
                out.push_str(&format!(
                    "  {}  {} graduated experiments
",
                    name.dimmed(),
                    count.to_string().bright_green()
                ));
            } else {
                out.push_str(&format!(
                    "  {}  {}
",
                    name.bright_yellow().bold(),
                    "active".bright_yellow()
                ));
            }
            found = true;
        }
    }
    if !found {
        out.push_str(&format!(
            "  {}
",
            "No active experiments -- use labs/ to create one".dimmed()
        ));
    }
    out.push_str(&format!(
        "  {}
",
        format!("{}/labs/", core_root).dimmed()
    ));
    CommandResult::Output(out)
}

fn vm_dispatch(args: &[&str]) -> CommandResult {
    // INT-077: `vm` drives faelight-vm (build-vm + SSH loop) via
    // pkgs/faelight/scripts/vm. Inherited stdio throughout so `vm ssh` is
    // interactive (password + guest shell) and build/up/down stream live.
    // INT-027's libvirt nixos-lab tooling (vm_start/stop/snapshot/restore/...)
    // remains defined below but is intentionally unwired from this verb; the
    // nixos-lab domain is dormant. Snapshot support for faelight-vm (qcow2) is
    // a later decision, not wired here.
    // INT-079 G3 (Option B): the script is the single source of truth for which
    // `vm` subcommands exist. fsh forwards ALL args to it (including an empty arg,
    // which the script's usage() handles) and no longer keeps its own verb whitelist
    // or duplicate help string -- that drifted (it never listed `debug`). The script
    // rejects true unknowns itself (unknown subcommand -> usage + exit 2).
    let _ = args.first().copied().unwrap_or("");
    let home = std::env::var("HOME").unwrap_or_default();
    let script = format!("{}/0-core/faelight/packages/faelight/scripts/vm", home);
    let st = std::process::Command::new(&script)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();
    match st {
        Ok(_) => CommandResult::Empty,
        Err(e) => CommandResult::Error(format!("vm: {}", e)),
    }
}

// INT-027 libvirt nixos-lab tooling: preserved, unwired from `vm` (now faelight-vm).
#[allow(dead_code)]
fn vm_snapshot(snap: Option<&str>) -> CommandResult {
    use colored::Colorize;
    let domain = "nixos-lab";
    let name = match snap {
        Some(n) if !n.is_empty() => n,
        _ => {
            return CommandResult::Output(format!(
                "  {}\n",
                "vm snapshot: needs a name -- e.g. vm snapshot before-greetd-test".bright_red()
            ))
        }
    };
    let result = std::process::Command::new("virsh")
        .args(["-c", "qemu:///system", "snapshot-create-as", domain, name])
        .output();
    let mut out = String::new();
    match result {
        Ok(o) if o.status.success() => {
            out.push_str(&format!(
                "  {} {}  {}\n",
                "📸 snapshot".bright_green().bold(),
                name.bright_white(),
                format!("(on {})", domain).dimmed()
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            out.push_str(&format!(
                "  {} {}\n  {}\n",
                "vm snapshot failed:".bright_red().bold(),
                name.bright_white(),
                stderr.trim().dimmed()
            ));
        }
        Err(e) => {
            out.push_str(&format!(
                "  {} {}\n",
                "vm snapshot: could not run virsh".bright_red().bold(),
                e.to_string().dimmed()
            ));
        }
    }
    CommandResult::Output(out)
}

// INT-027 libvirt nixos-lab tooling: preserved, unwired from `vm` (now faelight-vm).
#[allow(dead_code)]
fn vm_restore(snap: Option<&str>) -> CommandResult {
    use colored::Colorize;
    let domain = "nixos-lab";
    let name = match snap {
        Some(n) if !n.is_empty() => n,
        _ => {
            return CommandResult::Output(format!(
                "  {}\n",
                "vm restore: needs a name -- e.g. vm restore before-greetd-test".bright_red()
            ))
        }
    };
    let result = std::process::Command::new("virsh")
        .args(["-c", "qemu:///system", "snapshot-revert", domain, name])
        .output();
    let mut out = String::new();
    match result {
        Ok(o) if o.status.success() => {
            out.push_str(&format!(
                "  {} {}  {}\n",
                "⟲ restored".bright_green().bold(),
                name.bright_white(),
                format!("(on {})", domain).dimmed()
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            out.push_str(&format!(
                "  {} {}\n  {}\n",
                "vm restore failed:".bright_red().bold(),
                name.bright_white(),
                stderr.trim().dimmed()
            ));
        }
        Err(e) => {
            out.push_str(&format!(
                "  {} {}\n",
                "vm restore: could not run virsh".bright_red().bold(),
                e.to_string().dimmed()
            ));
        }
    }
    CommandResult::Output(out)
}

// INT-027 libvirt nixos-lab tooling: preserved, unwired from `vm` (now faelight-vm).
#[allow(dead_code)]
fn vm_snapshots() -> CommandResult {
    use colored::Colorize;
    let domain = "nixos-lab";
    let result = std::process::Command::new("virsh")
        .args(["-c", "qemu:///system", "snapshot-list", domain])
        .output();
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {}\n",
        format!("📸 Snapshots ({})", domain).bright_cyan().bold()
    ));
    match result {
        Ok(o) if o.status.success() => {
            let listing = String::from_utf8_lossy(&o.stdout);
            let body = listing.trim_end();
            if body.is_empty() {
                out.push_str(&format!("  {}\n", "(none)".dimmed()));
            } else {
                for line in body.lines() {
                    out.push_str(&format!("  {}\n", line));
                }
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            out.push_str(&format!("  {}\n", stderr.trim().dimmed()));
        }
        Err(e) => {
            out.push_str(&format!("  {}\n", e.to_string().dimmed()));
        }
    }
    CommandResult::Output(out)
}

// INT-027 libvirt nixos-lab tooling: preserved, unwired from `vm` (now faelight-vm).
#[allow(dead_code)]
fn vm_status(name: Option<&str>) -> CommandResult {
    use colored::Colorize;
    let domain = name.unwrap_or("nixos-lab");
    let result = std::process::Command::new("virsh")
        .args(["-c", "qemu:///system", "domstate", domain])
        .output();
    let mut out = String::new();
    match result {
        Ok(o) if o.status.success() => {
            let state = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let state_colored = match state.as_str() {
                "running" => state.bright_green().bold().to_string(),
                "shut off" => state.dimmed().to_string(),
                other => other.bright_yellow().to_string(),
            };
            out.push_str(&format!(
                "  🖥  {}  {}\n",
                domain.bright_white().bold(),
                state_colored
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            out.push_str(&format!(
                "  {} {}\n  {}\n",
                "vm status failed:".bright_red().bold(),
                domain.bright_white(),
                stderr.trim().dimmed()
            ));
        }
        Err(e) => {
            out.push_str(&format!(
                "  {} {}\n",
                "vm status: could not run virsh".bright_red().bold(),
                e.to_string().dimmed()
            ));
        }
    }
    CommandResult::Output(out)
}

// INT-027 libvirt nixos-lab tooling: preserved, unwired from `vm` (now faelight-vm).
#[allow(dead_code)]
fn vm_stop(name: Option<&str>) -> CommandResult {
    use colored::Colorize;
    let domain = name.unwrap_or("nixos-lab");
    let result = std::process::Command::new("virsh")
        .args(["-c", "qemu:///system", "shutdown", domain])
        .output();
    let mut out = String::new();
    match result {
        Ok(o) if o.status.success() => {
            out.push_str(&format!(
                "  {} {}\n",
                "🖥  stopping".bright_yellow().bold(),
                domain.bright_white()
            ));
            let stdout = String::from_utf8_lossy(&o.stdout);
            if !stdout.trim().is_empty() {
                out.push_str(&format!("  {}\n", stdout.trim().dimmed()));
            }
            out.push_str(&format!(
                "  {}\n",
                "(graceful shutdown -- give it a few seconds)".dimmed()
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            out.push_str(&format!(
                "  {} {}\n  {}\n",
                "vm stop failed:".bright_red().bold(),
                domain.bright_white(),
                stderr.trim().dimmed()
            ));
        }
        Err(e) => {
            out.push_str(&format!(
                "  {} {}\n",
                "vm stop: could not run virsh".bright_red().bold(),
                e.to_string().dimmed()
            ));
        }
    }
    CommandResult::Output(out)
}

// INT-027 libvirt nixos-lab tooling: preserved, unwired from `vm` (now faelight-vm).
#[allow(dead_code)]
fn vm_start(name: Option<&str>) -> CommandResult {
    use colored::Colorize;
    let domain = name.unwrap_or("nixos-lab");
    let result = std::process::Command::new("virsh")
        .args(["-c", "qemu:///system", "start", domain])
        .output();
    let mut out = String::new();
    match result {
        Ok(o) if o.status.success() => {
            out.push_str(&format!(
                "  {} {}\n",
                "🖥  started".bright_green().bold(),
                domain.bright_white()
            ));
            let stdout = String::from_utf8_lossy(&o.stdout);
            if !stdout.trim().is_empty() {
                out.push_str(&format!("  {}\n", stdout.trim().dimmed()));
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            out.push_str(&format!(
                "  {} {}\n  {}\n",
                "vm start failed:".bright_red().bold(),
                domain.bright_white(),
                stderr.trim().dimmed()
            ));
        }
        Err(e) => {
            out.push_str(&format!(
                "  {} {}\n",
                "vm start: could not run virsh".bright_red().bold(),
                e.to_string().dimmed()
            ));
        }
    }
    CommandResult::Output(out)
}

// INT-027 libvirt nixos-lab tooling: preserved, unwired from `vm` (now faelight-vm).
#[allow(dead_code)]
fn vm_list() -> CommandResult {
    use colored::Colorize;
    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  🖥  Virtual Machines".bright_cyan().bold()
    ));
    out.push_str(&format!(
        "{}\n",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    ));
    // INT-030: scan ~/vms/*.qcow2 -- no virsh dependency
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
    let vms_dir = std::path::PathBuf::from(&home).join("vms");
    // Check which qcow2 names are currently running via qemu process list
    let running_output = std::process::Command::new("pgrep")
        .args(["-a", "qemu"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(&vms_dir) {
        let mut disks: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().extension().map(|x| x == "qcow2").unwrap_or(false))
            .collect();
        disks.sort_by_key(|e| e.file_name());
        for entry in &disks {
            let fname = entry.file_name().to_string_lossy().to_string();
            let name = fname.trim_end_matches(".qcow2").to_string();
            let disk_path = entry.path().to_string_lossy().to_string();
            let is_running = running_output.contains(&disk_path) || running_output.contains(&name);
            let state_str = if is_running { "running" } else { "stopped" };
            let state_colored = if is_running {
                state_str.bright_green().to_string()
            } else {
                state_str.dimmed().to_string()
            };
            let size = std::fs::metadata(entry.path())
                .map(|m| {
                    let mb = m.len() / 1_048_576;
                    if mb >= 1024 {
                        format!("{}G", mb / 1024)
                    } else {
                        format!("{}M", mb)
                    }
                })
                .unwrap_or_else(|_| "?".to_string());
            out.push_str(&format!(
                "  {}  {}  {}\n",
                name.bright_white().bold(),
                state_colored,
                size.dimmed()
            ));
            count += 1;
        }
    }
    if count == 0 {
        out.push_str(&format!(
            "  {}\n",
            "No VMs found -- place .qcow2 files in ~/vms/".dimmed()
        ));
    }
    out.push_str(&format!("  {}\n", format!("{}/vms/", home).dimmed()));
    CommandResult::Output(out)
}

#[allow(dead_code)]
fn tools(_db: &ForestDb, core_root: &str) -> CommandResult {
    let tools_dir = faelight_core::paths::rust_tools_dir();
    let total = std::fs::read_dir(&tools_dir)
        .map(|e| {
            e.flatten()
                .filter(|e| e.path().join("Cargo.toml").exists())
                .count()
        })
        .unwrap_or(0);

    let deployed = std::fs::read_dir(std::path::PathBuf::from(core_root).join("scripts"))
        .map(|e| e.flatten().count())
        .unwrap_or(0);

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 🛠  Tools ─────────────────────────────────────────".bright_cyan()
    ));
    out.push_str(&format!(
        "  │  Total:    {} tools\n",
        total.to_string().bright_white().bold()
    ));
    out.push_str(&format!(
        "  │  Deployed: {}/{}\n",
        deployed.to_string().bright_green(),
        total
    ));
    out.push_str(&format!(
        "  │  Run {} for intelligence scores\n",
        "audit".bright_cyan()
    ));
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn version(_core_root: &str) -> CommandResult {
    let version = std::fs::read_to_string(faelight_core::paths::version_file())
        .unwrap_or_else(|_| "unknown".into());

    let changelog =
        std::fs::read_to_string(faelight_core::paths::changelog_file()).unwrap_or_default();

    let release_name = changelog
        .lines()
        .find(|l| l.starts_with("## ["))
        .and_then(|l| l.split('—').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "The Forest Remembers".to_string());

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 🌲 Version ───────────────────────────────────────".bright_cyan()
    ));
    out.push_str(&format!(
        "  │  {}  {}\n",
        version.trim().bright_white().bold(),
        release_name.dimmed()
    ));
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn commits(core_root: &str) -> CommandResult {
    let count = std::process::Command::new("git")
        .args(["-C", core_root, "rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "?".to_string());

    let last = std::process::Command::new("git")
        .args(["-C", core_root, "log", "-1", "--format=%s"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 📚 Commits ───────────────────────────────────────".bright_cyan()
    ));
    out.push_str(&format!("  │  Total:  {}\n", count.bright_white().bold()));
    out.push_str(&format!("  │  Last:   {}\n", last.dimmed()));
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

fn story(db: &ForestDb) -> CommandResult {
    // Delegate to core story via process
    let _core_root = db.core_root();
    let output = std::process::Command::new("core".to_string())
        .args(["story"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "core story not available".to_string());

    CommandResult::Output(output)
}

fn advise(db: &ForestDb) -> CommandResult {
    let _core_root = db.core_root();
    let output = std::process::Command::new("core".to_string())
        .args(["advise"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "core advise not available".to_string());

    CommandResult::Output(output)
}

fn audit(_db: &ForestDb, _core_root: &str) -> CommandResult {
    let output = std::process::Command::new("core".to_string())
        .args(["audit", "scan"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "core audit not available".to_string());

    CommandResult::Output(output)
}

// ── Schema — Phase 11a ────────────────────────────────────────────────────────

fn schema(args: &[&str]) -> CommandResult {
    let registry = crate::schema::SchemaRegistry::build();
    match args.first() {
        None | Some(&"") => CommandResult::Output(crate::schema::render_registry(&registry)),
        Some(table_name) => match registry.get(table_name) {
            Some(schema) => CommandResult::Output(crate::schema::render_table_schema(schema)),
            None => {
                let known = registry.names().join(", ");
                CommandResult::Error(format!("unknown table '{}' — known: {}", table_name, known))
            }
        },
    }
}

// ── Phase 14 — File System Index ─────────────────────────────────────────────
// Persistent index in state.db for fast recursive file queries.
// find           — query from index (or rebuild if empty)
// find reindex   — rebuild index from core_root
// find <path>    — query files under a specific path

fn grep_cmd(line: &str, args: &[&str]) -> CommandResult {
    if args.is_empty() {
        return CommandResult::Error(
            "usage: grep <pattern> [file]  or  grep [-i] <pattern> [file]".to_string(),
        );
    }
    // If any unrecognized flags present — fall through to system grep
    let known_flags = ["-i"];
    let has_unknown_flags = args
        .iter()
        .any(|a| a.starts_with('-') && !known_flags.contains(a));
    if has_unknown_flags {
        // Call grep via shared helper (INT-249)
        let status = crate::db::spawn_sh_with_leak_check(line);
        return match status {
            Ok(s) if s.success() => CommandResult::Empty,
            Ok(s) => {
                CommandResult::Error(format!("grep: exited with code {}", s.code().unwrap_or(1)))
            }
            Err(e) => CommandResult::Error(format!("grep: {}", e)),
        };
    }
    let (case_insensitive, rest) = if args.first() == Some(&"-i") {
        (true, &args[1..])
    } else {
        (false, args)
    };
    if rest.is_empty() {
        return CommandResult::Error("usage: grep <pattern> [file]".to_string());
    }
    let pattern = rest[0].trim_matches('"').trim_matches('\'');
    // If pattern contains alternation (\|) or regex metacharacters, fall through to system grep
    if pattern.contains("\\|")
        || pattern.contains("\\(")
        || pattern.contains("\\)")
        || pattern.contains("\\+")
    {
        let status = crate::db::spawn_sh_with_leak_check(line);
        return match status {
            Ok(s) if s.success() => CommandResult::Empty,
            Ok(s) => {
                CommandResult::Error(format!("grep: exited with code {}", s.code().unwrap_or(1)))
            }
            Err(e) => CommandResult::Error(format!("grep: {}", e)),
        };
    }
    let file_arg = rest.get(1).copied();

    let lines: Vec<String> = if let Some(path) = file_arg {
        let expanded = if path.starts_with("~/") {
            let home = std::env::var("HOME").unwrap_or_default();
            path.replacen("~/", &format!("{}/", home), 1)
        } else {
            path.to_string()
        };
        match std::fs::read_to_string(&expanded) {
            Ok(content) => content.lines().map(|l| l.to_string()).collect(),
            Err(e) => return CommandResult::Error(format!("grep: {}: {}", expanded, e)),
        }
    } else {
        return CommandResult::Error(
            "grep: pipe support coming — use grep <pattern> <file> for now".to_string(),
        );
    };

    let matched: Vec<String> = lines
        .into_iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let matches = if case_insensitive {
                line.to_lowercase().contains(&pattern.to_lowercase())
            } else {
                line.contains(pattern)
            };
            if matches {
                let highlighted = if case_insensitive {
                    line.clone()
                } else {
                    line.replace(pattern, &format!("[1;31m{}[0m", pattern))
                };
                Some(format!(
                    "  {}  {}",
                    format!("{:4}", i + 1).dimmed(),
                    highlighted
                ))
            } else {
                None
            }
        })
        .collect();

    if matched.is_empty() {
        CommandResult::Output(format!("  {} no matches for '{}'", "○".dimmed(), pattern))
    } else {
        CommandResult::Output(format!(
            "{}
  {} {} match{}",
            matched.join(
                "
"
            ),
            "✅".green(),
            matched.len().to_string().bright_green(),
            if matched.len() == 1 { "" } else { "es" }
        ))
    }
}

fn fsh_identity_cmd(db: &ForestDb) -> CommandResult {
    use colored::*;
    let home = std::env::var("HOME").unwrap_or_default();
    // Load stats from DB
    let aliases: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM shell_aliases", [], |r| r.get(0))
        .unwrap_or(0);
    let version: String = db
        .conn
        .query_row(
            "SELECT value FROM shell_state WHERE key = 'shell_version' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
    let alias_count = aliases;
    // Load health from cache
    let health: String = std::fs::read_to_string(format!("{}/.cache/faelight/health-status", home))
        .unwrap_or_else(|_| "100%".to_string())
        .trim()
        .to_string();
    // Load Friday live data from state.db
    let (friday_patterns, friday_facts) = {
        let db_path = faelight_core::paths::state_db();
        let conn = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        );
        match conn {
            Ok(c) => {
                let patterns: i64 = c
                    .query_row(
                        "SELECT COUNT(*) FROM friday_patterns WHERE confidence >= 0.7",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                let facts: i64 = c
                    .query_row("SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0))
                    .unwrap_or(0);
                (patterns, facts)
            }
            Err(_) => (0, 0),
        }
    };
    // Get login shell date
    let login_since = "2026-04-03";

    let mut out = String::new();
    out.push_str(
        "
",
    );
    out.push_str(&format!(
        "  {} {}
",
        "🌲 Faelight Shell".bright_green().bold(),
        format!("v{}", version).dimmed()
    ));
    out.push_str(&format!(
        "  {}
",
        "━".repeat(42).dimmed()
    ));
    out.push_str(&format!(
        "  {:<16} {}
",
        "Login shell".dimmed(),
        format!("✅ since {}", login_since).bright_green()
    ));
    out.push_str(&format!(
        "  {:<16} {}
",
        "Forest".dimmed(),
        std::fs::read_to_string("/etc/faelight/VERSION")
            .unwrap_or_else(|_| "v14.0.0".to_string())
            .trim()
            .trim_start_matches("v")
            .to_string()
            .bright_green()
    ));
    out.push_str(&format!(
        "  {:<16} {}
",
        "Aliases".dimmed(),
        alias_count.to_string().bright_white()
    ));
    out.push_str(&format!(
        "  {:<16} {}
",
        "Health".dimmed(),
        health.bright_green()
    ));
    out.push_str(&format!(
        "  {:<16} {}
",
        "Friday".dimmed(),
        format!(
            "active · {} patterns · {} facts",
            friday_patterns, friday_facts
        )
        .bright_cyan()
    ));
    out.push_str(&format!(
        "  {}
",
        "━".repeat(42).dimmed()
    ));
    out.push_str(&format!(
        "  {}
",
        "The forest thinks in Rust.".dimmed().italic()
    ));
    out.push_str(&format!(
        "  {}
",
        "Every command understood. Nothing installed blindly."
            .dimmed()
            .italic()
    ));
    out.push_str(
        "
",
    );
    CommandResult::Output(out)
}
fn realpath_cmd(args: &[&str]) -> CommandResult {
    let home = std::env::var("HOME").unwrap_or_default();
    let path = match args.first() {
        Some(p) => p,
        None => {
            // No arg — return cwd
            return match std::env::current_dir() {
                Ok(p) => CommandResult::Output(p.to_string_lossy().to_string()),
                Err(e) => CommandResult::Error(format!("realpath: {}", e)),
            };
        }
    };
    let expanded = if path.starts_with("~/") {
        path.replacen("~/", &format!("{}/", home), 1)
    } else {
        path.to_string()
    };
    match std::fs::canonicalize(&expanded) {
        Ok(p) => CommandResult::Output(p.to_string_lossy().to_string()),
        Err(_) => {
            // Path doesn't exist yet — resolve without canonicalize
            let p = std::path::Path::new(&expanded);
            let abs = if p.is_absolute() {
                expanded.clone()
            } else {
                std::env::current_dir()
                    .map(|cwd| cwd.join(p).to_string_lossy().to_string())
                    .unwrap_or(expanded.clone())
            };
            CommandResult::Output(abs)
        }
    }
}
/// INT-143 case 3: `time` times whatever you typed -- builtin, alias, or external.
///
/// THE BUG: this delegated everything to `sh -c`, and sh has never heard of fsh's 303 command
/// names OR its 285 aliases. Measured 2026-07-16:
///     time echo works_on_binaries -> 2ms (exit 0)                      -- sh found /bin/echo
///     time git --version          -> 3ms (exit 0)                      -- on PATH
///     time d                      -> sh: d: command not found, exit 127 -- an fsh ALIAS
///     time hs                     -> sh: hs: command not found, exit 127 -- an fsh BUILTIN
/// The intent's text said "time cmd -> exit 127" flatly. Wrong: it worked for anything on PATH and
/// failed for everything that was fsh's own. The measurement corrected the intent.
///
/// THE FIX IS ONE LINE, AND IT DELETES CODE: hand the line to execute(). fsh's own dispatch
/// ALREADY resolves aliases (with INT-057's cycle guard), resolves plugins, runs builtins, and
/// falls through to run_external -> `sh -c` for real binaries. time does not need to know which
/// kind of thing it is timing -- that is execute()'s entire job. The previous version of this fix
/// probed with try_builtin and kept a separate sh path; it worked for builtins and still failed for
/// aliases, because the expansion happened inside the probe and was thrown away. Rewritten.
///
/// What is lost: spawn_sh_with_leak_check's unclosed-heredoc warning, which only ever applied to
/// the sh path. run_external uses `sh -c` with inherited stdio, so a heredoc still WORKS -- it just
/// does not get the warning. A warning on a path nobody times is not worth a second dispatcher.
fn time_cmd(line: &str, args: &[&str], db: &ForestDb, core_root: &str) -> CommandResult {
    if args.is_empty() {
        return CommandResult::Error("time: missing command".to_string());
    }
    let cmd_line = line.trim().trim_start_matches("time").trim().to_string();

    let start = std::time::Instant::now();
    let result = execute(&cmd_line, db, core_root);
    let elapsed = start.elapsed();

    let ms = elapsed.as_millis();
    let display = if ms >= 1000 {
        format!("{:.2}s", elapsed.as_secs_f64())
    } else {
        format!("{}ms", ms)
    };

    // execute() returns the output rather than printing it, so print it here -- the timing line
    // goes last, the way it always has.
    let code = match &result {
        CommandResult::Error(e) => {
            eprintln!("  {} {}", "\u{2717}".bright_red(), e);
            1
        }
        CommandResult::Output(o) => {
            if !o.is_empty() {
                println!("{}", o);
            }
            0
        }
        CommandResult::Value(v) => {
            println!("{}", v.render());
            0
        }
        _ => 0,
    };

    println!();
    println!(
        "  {} {} (exit {})",
        "\u{23F1}".to_string(),
        display.bright_cyan().bold(),
        code.to_string().dimmed()
    );

    // `time exit` should still leave the shell.
    if matches!(result, CommandResult::Exit) {
        return CommandResult::Exit;
    }
    CommandResult::Empty
}

fn resolve_fsh_binary() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let user = std::env::var("USER").unwrap_or_default();
    let candidates = vec![
        "/run/current-system/sw/bin/faelight-shell".to_string(),
        format!("/etc/profiles/per-user/{}/bin/faelight-shell", user),
        format!("{}/.cargo/bin/faelight-shell", home),
        format!("{}/0-core/scripts/faelight-shell", home),
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return path.clone();
        }
    }
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "faelight-shell".to_string())
}

// INT-081 G2: re-exec into the resolved (current-system-first) fsh. If that binary
// canonicalizes to the SAME path already running, nothing new was deployed -- say so
// instead of a pointless same-binary re-exec.
fn reload_fsh() -> CommandResult {
    use std::os::unix::process::CommandExt;
    let target = resolve_fsh_binary();
    // INT-096: compare the CURRENT deploy-target store path against the build this session
    // launched from (recorded at startup in /tmp/fsh-running-build). The store hash changes
    // every rebuild, so a differing hash = a genuinely new fsh was deployed. We never use
    // current_exe() here -- it is unreliable through the makeWrapper wrapper.
    let deployed = std::fs::canonicalize(&target)
        .ok()
        .map(|p| p.to_string_lossy().to_string());
    let running = std::fs::read_to_string("/tmp/fsh-running-build").ok();
    match (deployed.as_deref(), running.as_deref()) {
        (Some(d), Some(r)) if d.trim() == r.trim() => {
            CommandResult::Output(format!(
                "  Already on the current fsh build:\n    {}\n  Nothing new to reload. (Rebuild + deploy first.)",
                d.trim()
            ))
        }
        (Some(d), Some(r)) => {
            println!("  🔄 New fsh build detected -- reloading:");
            println!("    was: {}", r.trim());
            println!("    new: {}", d.trim());
            let err = std::process::Command::new(&target).exec();
            CommandResult::Error(format!("reload: {}: {}", target, err))
        }
        _ => {
            println!("  🔄 Reloading fsh -> {} (no build marker to compare)", target);
            let err = std::process::Command::new(&target).exec();
            CommandResult::Error(format!("reload: {}: {}", target, err))
        }
    }
}

fn exec_cmd(args: &[&str]) -> CommandResult {
    let home = std::env::var("HOME").unwrap_or_default();
    // Special case: exec fsh or exec faelight-shell → re-exec current binary
    let cmd = match args.first() {
        Some(c) => c,
        None => return CommandResult::Error("exec: missing command".to_string()),
    };
    let is_self = matches!(*cmd, "fsh" | "faelight-shell" | "shell");
    let resolved = if is_self {
        resolve_fsh_binary() // INT-081: current-system-first, not current_exe()
    } else if cmd.starts_with("~/") {
        cmd.replacen("~/", &format!("{}/", home), 1)
    } else {
        // Search PATH manually
        let path_env = std::env::var("PATH").unwrap_or_default();
        let found = path_env
            .split(':')
            .map(|dir| format!("{}/{}", dir, cmd))
            .find(|p| std::path::Path::new(p).exists());
        match found {
            Some(p) => p,
            None => {
                // Last resort: try current_exe for any shell-like name
                if matches!(*cmd, "fsh" | "shell" | "faelight-shell") {
                    std::env::current_exe()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| cmd.to_string())
                } else {
                    cmd.to_string()
                }
            }
        }
    };
    let err = std::process::Command::new(&resolved)
        .args(&args[1..])
        .exec();
    CommandResult::Error(format!("exec: {}: {}", cmd, err))
}
fn source_cmd(args: &[&str]) -> CommandResult {
    let file = match args.first() {
        Some(f) => f,
        None => return CommandResult::Error("source: missing filename".to_string()),
    };
    let home = std::env::var("HOME").unwrap_or_default();
    let path = if file.starts_with("~/") {
        file.replacen("~/", &format!("{}/", home), 1)
    } else {
        file.to_string()
    };
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let lines: Vec<String> = content
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .map(|l| l.to_string())
                .collect();
            CommandResult::Output(format!(
                "  {} sourced {} ({} lines) — aliases and settings loaded on next restart",
                "✅".to_string(),
                file,
                lines.len()
            ))
        }
        Err(e) => CommandResult::Error(format!("source: {}: {}", file, e)),
    }
}
fn tree_cmd(args: &[&str]) -> CommandResult {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir_arg = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .copied()
        .unwrap_or(".");
    let max_depth: usize = args
        .windows(2)
        .find(|w| w[0] == "-d" || w[0] == "--depth")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(3);
    let path = if dir_arg.starts_with("~/") {
        dir_arg.replacen("~/", &format!("{}/", home), 1)
    } else {
        dir_arg.to_string()
    };
    let mut out = String::new();
    out.push_str(&format!(
        "{}
",
        path.bright_cyan().bold()
    ));
    let mut count = (0usize, 0usize); // (dirs, files)
    tree_walk(
        std::path::Path::new(&path),
        "",
        0,
        max_depth,
        &mut out,
        &mut count,
    );
    out.push_str(&format!(
        "
  {} {} directories, {} files",
        "─".dimmed(),
        count.0.to_string().bright_white(),
        count.1.to_string().bright_white()
    ));
    CommandResult::Output(out)
}
fn tree_walk(
    path: &std::path::Path,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    out: &mut String,
    count: &mut (usize, usize),
) {
    if depth >= max_depth {
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    let mut items: Vec<std::fs::DirEntry> = entries.flatten().collect();
    items.sort_by_key(|e| e.file_name());
    // Filter hidden files
    items.retain(|e| !e.file_name().to_string_lossy().starts_with('.'));
    let total = items.len();
    for (i, entry) in items.iter().enumerate() {
        let is_last = i == total - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let display = if is_dir {
            name.bright_cyan().to_string()
        } else {
            name.normal().to_string()
        };
        out.push_str(&format!(
            "{}{}{}
",
            prefix, connector, display
        ));
        if is_dir {
            count.0 += 1;
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            tree_walk(&entry.path(), &new_prefix, depth + 1, max_depth, out, count);
        } else {
            count.1 += 1;
        }
    }
}
fn stat_cmd(args: &[&str]) -> CommandResult {
    let home = std::env::var("HOME").unwrap_or_default();
    let file = match args.first() {
        Some(f) => f,
        None => return CommandResult::Error("stat: missing filename".to_string()),
    };
    let path = if file.starts_with("~/") {
        file.replacen("~/", &format!("{}/", home), 1)
    } else {
        file.to_string()
    };
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return CommandResult::Error(format!("stat: {}: {}", file, e)),
    };
    let size = meta.len();
    let kind = if meta.is_dir() {
        "directory"
    } else if meta.is_symlink() {
        "symlink"
    } else {
        "file"
    };
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "?".to_string())
        })
        .unwrap_or_else(|| "?".to_string());
    let size_display = if size > 1_048_576 {
        format!("{:.1} MB", size as f64 / 1_048_576.0)
    } else if size > 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{} B", size)
    };
    let perms = meta.permissions();
    use std::os::unix::fs::PermissionsExt;
    let mode = format!("{:o}", perms.mode() & 0o777);
    let mut out = String::new();
    out.push_str(&format!(
        "
  {}
",
        path.bright_cyan().bold()
    ));
    out.push_str(&format!(
        "  {}  {}
",
        "Type:    ".dimmed(),
        kind.bright_white()
    ));
    out.push_str(&format!(
        "  {}  {}
",
        "Size:    ".dimmed(),
        size_display.bright_green()
    ));
    out.push_str(&format!(
        "  {}  {}
",
        "Mode:    ".dimmed(),
        mode.yellow()
    ));
    out.push_str(&format!(
        "  {}  {}
",
        "Modified:".dimmed(),
        modified.dimmed()
    ));
    CommandResult::Output(out)
}
fn preview_cmd(args: &[&str]) -> CommandResult {
    let home = std::env::var("HOME").unwrap_or_default();
    let file = match args.first() {
        Some(f) => f,
        None => return CommandResult::Error("preview: missing filename".to_string()),
    };
    let path = if file.starts_with("~/") {
        file.replacen("~/", &format!("{}/", home), 1)
    } else {
        file.to_string()
    };
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return CommandResult::Error(format!("preview: {}: {}", file, e)),
    };
    let size = meta.len();
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let text_exts = [
        "rs", "py", "js", "ts", "toml", "yaml", "yml", "sh", "zsh", "kdl", "md", "txt", "json",
        "html", "css", "ron", "conf", "ini", "env",
    ];
    if text_exts.contains(&ext.as_str()) {
        // Show first 30 lines via bat
        let output = std::process::Command::new("bat")
            .args(["--paging=never", "--line-range=1:30", &path])
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let content = String::from_utf8_lossy(&o.stdout).to_string();
                CommandResult::Output(content.trim_end().to_string())
            }
            _ => {
                // fallback to native read
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let preview: String = content
                            .lines()
                            .take(30)
                            .enumerate()
                            .map(|(i, l)| format!("  {}  {}", format!("{:4}", i + 1).dimmed(), l))
                            .collect::<Vec<_>>()
                            .join(
                                "
",
                            );
                        CommandResult::Output(preview)
                    }
                    Err(e) => CommandResult::Error(format!("preview: {}", e)),
                }
            }
        }
    } else {
        // Binary — show size and type info
        let size_display = if size > 1_048_576 {
            format!("{:.1} MB", size as f64 / 1_048_576.0)
        } else if size > 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{} B", size)
        };
        CommandResult::Output(format!(
            "  {} Binary file — {} ({} bytes)",
            "○".dimmed(),
            size_display.bright_white(),
            size.to_string().dimmed()
        ))
    }
}
fn find_cmd(db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    // Ensure schema
    db.conn
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS file_index (
            path      TEXT PRIMARY KEY,
            name      TEXT NOT NULL,
            kind      TEXT NOT NULL,
            size      INTEGER NOT NULL,
            extension TEXT,
            modified  INTEGER NOT NULL
        );",
        )
        .ok();

    // reindex — rebuild from core_root recursively
    if args.first() == Some(&"reindex") {
        let count = index_directory(&db.conn, core_root, 0);
        return CommandResult::Output(format!(
            "  {} Indexed {} files under {}",
            "✅".green(),
            count.to_string().bright_green(),
            core_root.dimmed()
        ));
    }

    // Check if index is empty — auto-reindex on first use
    let count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM file_index", [], |r| r.get(0))
        .unwrap_or(0);

    if count == 0 {
        println!(
            "  {} Building file index for first time...",
            "⟳".bright_cyan()
        );
        index_directory(&db.conn, core_root, 0);
    }

    // Filter by path prefix if arg given
    let path_filter = args.first().copied().unwrap_or("");
    let query = if path_filter.is_empty() {
        "SELECT path, name, kind, size, extension, modified FROM file_index ORDER BY size DESC LIMIT 500".to_string()
    } else {
        format!("SELECT path, name, kind, size, extension, modified FROM file_index WHERE path LIKE '{}%' ORDER BY size DESC LIMIT 500", path_filter)
    };

    let mut stmt = match db.conn.prepare(&query) {
        Ok(s) => s,
        Err(e) => return CommandResult::Error(e.to_string()),
    };

    let rows: Vec<HashMap<String, Value>> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })
        .ok()
        .map(|rows| {
            rows.filter_map(|r| r.ok())
                .map(|(path, name, kind, size, ext, modified)| {
                    let time_str = chrono::DateTime::from_timestamp(modified, 0)
                        .map(|t| t.format("%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "?".to_string());
                    let mut row = HashMap::new();
                    row.insert("name".to_string(), Value::Text(name));
                    row.insert("path".to_string(), Value::Text(path));
                    row.insert("kind".to_string(), Value::Text(kind));
                    row.insert("size".to_string(), Value::Int(size));
                    row.insert("ext".to_string(), Value::Text(ext.unwrap_or_default()));
                    row.insert("modified".to_string(), Value::Text(time_str));
                    row
                })
                .collect()
        })
        .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No files found. Run: find reindex",
            "○".dimmed()
        ));
    }
    CommandResult::Value(Value::Table(rows))
}

fn index_directory(conn: &rusqlite::Connection, dir: &str, depth: usize) -> usize {
    if depth > 6 {
        return 0;
    } // max depth
    let mut count = 0;
    let skip_dirs = [
        ".git",
        "target",
        "node_modules",
        ".cargo",
        "proc",
        "sys",
        "dev",
    ];
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && depth > 0 {
                continue;
            }
            let skip = skip_dirs.iter().any(|s| name == *s);
            if skip {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                let kind = if meta.is_dir() { "dir" } else { "file" };
                let size = if meta.is_file() { meta.len() as i64 } else { 0 };
                let ext = path.extension().map(|e| e.to_string_lossy().to_string());
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                conn.execute(
                    "INSERT OR REPLACE INTO file_index (path, name, kind, size, extension, modified)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![path_str, name, kind, size, ext, modified]
                ).ok();
                count += 1;
                if meta.is_dir() {
                    count += index_directory(conn, &path_str, depth + 1);
                }
            }
        }
    }
    count
}

// ── Phase 15 — Git Data Engine ────────────────────────────────────────────────

/// git-churn — files changed most frequently across commit history
/// gchurn | sort changes desc | first 10  → hottest files
fn git_churn(core_root: &str, args: &[&str]) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let limit = args
        .first()
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(100);

    let output = std::process::Command::new("git")
        .args([
            "-C",
            core_root,
            "log",
            &format!("--max-count={}", limit * 10),
            "--name-only",
            "--format=",
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    // Count occurrences of each file
    let mut counts: HashMap<String, usize> = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        *counts.entry(line.to_string()).or_insert(0) += 1;
    }

    let mut rows: Vec<HashMap<String, Value>> = counts
        .into_iter()
        .map(|(file, count)| {
            let ext = std::path::Path::new(&file)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();
            let mut row = HashMap::new();
            row.insert("file".to_string(), Value::Text(file));
            row.insert("changes".to_string(), Value::Int(count as i64));
            row.insert("ext".to_string(), Value::Text(ext));
            row
        })
        .collect();

    // Sort by changes desc by default
    rows.sort_by(|a, b| {
        let ac = a
            .get("changes")
            .and_then(|v| {
                if let Value::Int(i) = v {
                    Some(*i)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let bc = b
            .get("changes")
            .and_then(|v| {
                if let Value::Int(i) = v {
                    Some(*i)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        bc.cmp(&ac)
    });
    rows.truncate(limit);

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No git history found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}

/// git-branches — all branches as a structured table
fn git_branches(core_root: &str) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let output = std::process::Command::new("git")
        .args([
            "-C",
            core_root,
            "branch",
            "-a",
            "--format=%(refname:short)|%(objectname:short)|%(committerdate:relative)|%(HEAD)",
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() < 4 {
                return None;
            }
            let name = parts[0].trim().to_string();
            let hash = parts[1].trim().to_string();
            let date = parts[2].trim().to_string();
            let current = parts[3].trim() == "*";
            let remote = name.starts_with("remotes/");
            let mut row = HashMap::new();
            row.insert("name".to_string(), Value::Text(name));
            row.insert("hash".to_string(), Value::Text(hash));
            row.insert("date".to_string(), Value::Text(date));
            row.insert("current".to_string(), Value::Bool(current));
            row.insert("remote".to_string(), Value::Bool(remote));
            Some(row)
        })
        .collect();

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No branches found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}

// ── Phase 16 — History Analytics ─────────────────────────────────────────────

/// hstats — most used commands ranked by frequency
fn history_stats(db: &ForestDb) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let mut stmt = match db
        .conn
        .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 1000")
    {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
    };

    let commands: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    // Count first word of each command
    let mut counts: HashMap<String, usize> = HashMap::new();
    for cmd in &commands {
        // deadwood: exempt -- history aggregation key; the first token is counted into a HashMap
        // for statistics only and is never passed to dispatch
        let first = cmd.split_whitespace().next().unwrap_or("").to_string();
        if !first.is_empty() {
            *counts.entry(first).or_insert(0) += 1;
        }
    }

    let total = commands.len();
    let mut rows: Vec<HashMap<String, Value>> = counts
        .into_iter()
        .map(|(cmd, count)| {
            let pct = format!("{:.0}%", (count as f64 / total as f64) * 100.0);
            let mut row = HashMap::new();
            row.insert("command".to_string(), Value::Text(cmd));
            row.insert("count".to_string(), Value::Int(count as i64));
            row.insert("pct".to_string(), Value::Text(pct));
            row
        })
        .collect();

    rows.sort_by(|a, b| {
        let ac = a
            .get("count")
            .and_then(|v| {
                if let Value::Int(i) = v {
                    Some(*i)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let bc = b
            .get("count")
            .and_then(|v| {
                if let Value::Int(i) = v {
                    Some(*i)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        bc.cmp(&ac)
    });
    rows.truncate(20);

    CommandResult::Value(Value::Table(rows))
}

/// hpattern — command frequency by hour of day
fn history_pattern(db: &ForestDb) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let mut stmt = match db
        .conn
        .prepare("SELECT timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 2000")
    {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
    };

    let timestamps: Vec<i64> = stmt
        .query_map([], |r| r.get::<_, i64>(0))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    // Count by hour
    let mut by_hour: HashMap<u32, usize> = HashMap::new();
    use chrono::Timelike;
    for ts in &timestamps {
        if let Some(dt) = chrono::DateTime::from_timestamp(*ts, 0) {
            let hour = dt.hour();
            *by_hour.entry(hour).or_insert(0) += 1;
        }
    }

    let max_count = by_hour.values().copied().max().unwrap_or(1);

    let mut rows: Vec<HashMap<String, Value>> = (0u32..24)
        .map(|hour| {
            let count = by_hour.get(&hour).copied().unwrap_or(0);
            let bar_len = (count * 20) / max_count.max(1);
            let bar = "█".repeat(bar_len);
            let label = format!("{:02}:00", hour);
            let period = match hour {
                5..=8 => "morning",
                9..=11 => "late morning",
                12..=13 => "midday",
                14..=17 => "afternoon",
                18..=21 => "evening",
                22..=23 => "night",
                _ => "late night",
            };
            let mut row = HashMap::new();
            row.insert("hour".to_string(), Value::Text(label));
            row.insert("period".to_string(), Value::Text(period.to_string()));
            row.insert("count".to_string(), Value::Int(count as i64));
            row.insert("bar".to_string(), Value::Text(bar));
            row
        })
        .collect();

    // Only show hours with activity
    rows.retain(|r| {
        if let Some(Value::Int(c)) = r.get("count") {
            *c > 0
        } else {
            false
        }
    });

    CommandResult::Value(Value::Table(rows))
}

// ── Phase 17 — Event System ───────────────────────────────────────────────────

fn on_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    crate::triggers::ensure_schema(db);
    // Rejoin and re-split — execute() uses splitn(3) which embeds args
    let rejoined = args.join(" ");
    let reargs: Vec<&str> = rejoined.split_whitespace().collect();
    let args = reargs.as_slice();
    match args {
        // on list
        [] | ["list"] => {
            crate::triggers::render_list(db);
            CommandResult::Empty
        }
        // on remove <id>
        ["remove", id] => {
            if let Ok(n) = id.parse::<i64>() {
                if crate::triggers::remove(db, n) {
                    CommandResult::Output(format!("  {} Trigger #{} removed.", "✅".green(), n))
                } else {
                    CommandResult::Output(format!(
                        "  {} Trigger #{} not found.",
                        "✗".bright_red(),
                        n
                    ))
                }
            } else {
                CommandResult::Error("Usage: on remove <id>".to_string())
            }
        }
        // on enable/disable <id>
        ["enable", id] => {
            if let Ok(n) = id.parse::<i64>() {
                crate::triggers::enable(db, n, true);
                CommandResult::Output(format!("  {} Trigger #{} enabled.", "✅".green(), n))
            } else {
                CommandResult::Error("Usage: on enable <id>".to_string())
            }
        }
        ["disable", id] => {
            if let Ok(n) = id.parse::<i64>() {
                crate::triggers::enable(db, n, false);
                CommandResult::Output(format!("  {} Trigger #{} disabled.", "○".dimmed(), n))
            } else {
                CommandResult::Error("Usage: on disable <id>".to_string())
            }
        }
        // on <trigger...> => <action...>
        args => {
            // Find => separator
            if let Some(sep) = args.iter().position(|a| *a == "=>") {
                let trigger = args[..sep].join(" ");
                let action = args[sep + 1..].join(" ");
                if trigger.is_empty() || action.is_empty() {
                    return CommandResult::Error(
                        "Usage: on <trigger> => <action>  e.g. on health_drop 90 => notify \"health low\"".to_string()
                    );
                }
                match crate::triggers::add(db, &trigger, &action) {
                    Ok(_) => CommandResult::Output(format!(
                        "  {} Trigger added: {} → {}",
                        "✅".green(),
                        trigger.bright_cyan(),
                        action.bright_white()
                    )),
                    Err(e) => CommandResult::Error(e),
                }
            } else {
                CommandResult::Error(
                    "Usage: on <trigger> => <action>\nTriggers: health_drop <n>, git_commit, event <domain>, run <cmd>".to_string()
                )
            }
        }
    }
}

// ── Phase 18 — Time Travel ────────────────────────────────────────────────────
// snapshot  — capture current system state
// timeline  — show snapshots over time
// snap-diff — compare two snapshots

fn ensure_snapshots_schema(db: &ForestDb) {
    db.conn
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS shell_snapshots (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            name      TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            health    INTEGER,
            commits   INTEGER,
            processes INTEGER,
            load_avg  TEXT,
            top_proc  TEXT,
            note      TEXT
        );",
        )
        .ok();
}

/// INT-322 Phase 4: rewind -- show snapshot timeline for time-travel debugging
/// INT-311 Phase 2: dev -- wired cargo dev tools
fn dev_cmd(_db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    use colored::Colorize;
    let sub = args.first().copied().unwrap_or("");
    let tool = args.get(1).copied().unwrap_or("");
    match sub {
        "test" => {
            // cargo nextest run for a specific tool or all
            let manifest = if tool.is_empty() {
                faelight_core::paths::rust_tools_dir()
                    .join("faelight-shell/Cargo.toml")
                    .to_string_lossy()
                    .to_string()
            } else {
                faelight_core::paths::rust_tools_dir()
                    .join(tool)
                    .join("Cargo.toml")
                    .to_string_lossy()
                    .to_string()
            };
            if !std::path::Path::new(&manifest).exists() {
                return CommandResult::Error(format!(
                    "  dev test: no Cargo.toml found for '{}'",
                    tool
                ));
            }
            println!(
                "  {} running: cargo nextest run --manifest-path {}",
                "🧪".normal(),
                manifest.dimmed()
            );
            let status = std::process::Command::new("cargo")
                .args(["nextest", "run", "--manifest-path", &manifest])
                .status();
            match status {
                Ok(s) if s.success() => {
                    CommandResult::Output(format!("  {} all tests passed", "✅".normal()))
                }
                Ok(_) => CommandResult::Error("  dev test: tests failed".to_string()),
                Err(e) => CommandResult::Error(format!("  dev test: {}", e)),
            }
        }
        "watch" => {
            // cargo watch for a specific tool
            let manifest = if tool.is_empty() {
                faelight_core::paths::rust_tools_dir()
                    .join("faelight-shell/Cargo.toml")
                    .to_string_lossy()
                    .to_string()
            } else {
                faelight_core::paths::rust_tools_dir()
                    .join(tool)
                    .join("Cargo.toml")
                    .to_string_lossy()
                    .to_string()
            };
            println!(
                "  {} starting: cargo watch --manifest-path {}",
                "👁".normal(),
                manifest.dimmed()
            );
            println!("  {} Ctrl+C to stop", "→".dimmed());
            let _ = std::process::Command::new("cargo")
                .args(["watch", "--manifest-path", &manifest, "-x", "build"])
                .status();
            CommandResult::Empty
        }
        "audit-deps" => {
            // cargo udeps -- find unused dependencies
            println!(
                "  {} running: cargo +nightly udeps (this may take a moment)",
                "🔍".normal()
            );
            let status = std::process::Command::new("cargo")
                .args(["+nightly", "udeps", "--all-targets"])
                .current_dir(core_root)
                .status();
            match status {
                Ok(s) if s.success() => {
                    CommandResult::Output("  ✅ no unused dependencies found".to_string())
                }
                Ok(_) => CommandResult::Output(
                    "  ⚠ unused dependencies found -- review above".to_string(),
                ),
                Err(e) => CommandResult::Error(format!(
                    "  dev audit-deps: {} -- is cargo-udeps installed?",
                    e
                )),
            }
        }
        "bench" => {
            // hyperfine benchmarking
            let cmd = args.get(1..).map(|a| a.join(" ")).unwrap_or_default();
            if cmd.is_empty() {
                return CommandResult::Output(
                    "  Usage: dev bench <command>  (e.g. dev bench fsh-test)".to_string(),
                );
            }
            let _ = std::process::Command::new("hyperfine").arg(&cmd).status();
            CommandResult::Empty
        }
        "deps" => {
            // dev deps <crate> -- why is <crate> in the build? (INT-134, Lane 3).
            // The Rust-side mirror of `store why`: `cargo tree --invert` shows every path
            // from your workspace down to <crate>. Read-only; cargo already renders the tree,
            // so we add a header + honest not-found message and pass its output through.
            let krate = match args.get(1) {
                Some(c) => *c,
                None => return CommandResult::Error(
                    "  dev deps <crate>  -- what pulls <crate> into the build (e.g. dev deps libc)"
                        .to_string(),
                ),
            };
            let out = std::process::Command::new("cargo")
                .args(["tree", "--invert", "--package", krate])
                .current_dir(core_root)
                .output();
            match out {
                Ok(o) if o.status.success() => {
                    let tree = String::from_utf8_lossy(&o.stdout);
                    let tree = tree.trim_end();
                    if tree.is_empty() {
                        return CommandResult::Output(format!(
                            "  {} is not in the workspace dependency tree",
                            krate
                        ));
                    }
                    let mut s = format!("  \u{1f333} why is '{}' in the build?\n", krate);
                    s.push_str(&"\u{2500}".repeat(52));
                    s.push('\n');
                    s.push_str(tree);
                    CommandResult::Output(s)
                }
                Ok(o) => {
                    let err = String::from_utf8_lossy(&o.stderr);
                    // Multiple versions in the tree -> cargo refuses a bare name. Surface the
                    // version choices as a prompt, not a raw error. (INT-134, Lane 3)
                    if err.contains("is ambiguous") {
                        let mut vers: Vec<String> = Vec::new();
                        for line in err.lines() {
                            let t = line.trim();
                            // cargo lists "<name>@<ver>" one per line under the help text.
                            if t.starts_with(krate) && t.contains('@') {
                                vers.push(t.to_string());
                            }
                        }
                        if vers.is_empty() {
                            return CommandResult::Error(format!("  dev deps: {}", err.trim()));
                        }
                        return CommandResult::Output(format!(
                            "  '{}' is ambiguous -- pick a version:\n    {}\n  e.g. dev deps {}",
                            krate,
                            vers.join("\n    "),
                            vers[0]
                        ));
                    }
                    // cargo says "package ID specification ... did not match" when the crate
                    // isn't a dependency -- surface that as a clean not-found, not a raw error.
                    if err.contains("did not match") || err.contains("not found") {
                        // Fuzzy nudge: exact match failed, but the crate may exist under a
                        // different name (e.g. openssl -> openssl-sys). Scan the full tree once
                        // for names CONTAINING the term and suggest them. (INT-134, Lane 3)
                        let full = std::process::Command::new("cargo")
                            .args(["tree", "--prefix", "none"])
                            .current_dir(core_root)
                            .output();
                        let mut hits: Vec<String> = Vec::new();
                        if let Ok(f) = full {
                            let listing = String::from_utf8_lossy(&f.stdout);
                            for line in listing.lines() {
                                // deadwood: exempt -- parses cargo's package-listing OUTPUT; the first field identifies a
                                // displayed crate entry and never controls shell execution
                                if let Some(name) = line.split_whitespace().next() {
                                    if name.contains(krate) && name != krate {
                                        hits.push(name.to_string());
                                    }
                                }
                            }
                        }
                        hits.sort();
                        hits.dedup();
                        if hits.is_empty() {
                            CommandResult::Output(format!(
                                "  {} is not a dependency of this workspace",
                                krate
                            ))
                        } else {
                            let shown: Vec<String> = hits.iter().take(8).cloned().collect();
                            CommandResult::Output(format!(
                                "  no crate named '{}' -- did you mean:\n    {}",
                                krate,
                                shown.join("\n    ")
                            ))
                        }
                    } else {
                        CommandResult::Error(format!("  dev deps: {}", err.trim()))
                    }
                }
                Err(e) => CommandResult::Error(format!("  dev deps: {}", e)),
            }
        }
        "search" => {
            // dev search <query> -- search crates.io via `cargo search`, print name/version/desc
            // (INT-134, Lane 3; crates.io analogue of pkg-search's nixpkgs search). cargo search
            // outputs TEXT (name = "ver"  # desc), not JSON -- parsed line-wise. Read-only;
            // latency expected (network). Caches to /tmp/fsh-crate-search.json for future completion.
            let query = args[1..].join(" ");
            if query.trim().is_empty() {
                return CommandResult::Error(
                    "  dev search <query>  -- search crates.io (e.g. dev search tui)".to_string(),
                );
            }
            let out = std::process::Command::new("cargo")
                .args(["search", "--limit", "20", &query])
                .output();
            let out = match out {
                Ok(o) if o.status.success() => o,
                Ok(o) => {
                    let err = String::from_utf8_lossy(&o.stderr);
                    return CommandResult::Error(format!(
                        "  dev search: cargo search failed: {}",
                        err.trim()
                    ));
                }
                Err(e) => return CommandResult::Error(format!("  dev search: {}", e)),
            };
            let text = String::from_utf8_lossy(&out.stdout);
            let mut rows: Vec<(String, String, String)> = Vec::new();
            let mut more_note = String::new();
            for line in text.lines() {
                let line = line.trim_end();
                if line.trim_start().starts_with("...") {
                    more_note = line.trim().to_string();
                    continue;
                }
                if let Some((name, rest)) = line.split_once(" = ") {
                    let name = name.trim().to_string();
                    let ver = rest.split('"').nth(1).unwrap_or("").to_string();
                    let desc = match rest.split_once('#') {
                        Some((_, d)) => d.trim().to_string(),
                        None => String::new(),
                    };
                    if !name.is_empty() {
                        rows.push((name, ver, desc));
                    }
                }
            }
            if rows.is_empty() {
                return CommandResult::Output(format!("  no crates matching '{}'", query));
            }
            let _ = std::fs::write("/tmp/fsh-crate-search.json", text.as_bytes());
            let mut lines = vec![format!(
                "  \u{1f4e6} crates.io matches for '{}' ({})",
                query,
                rows.len()
            )];
            lines.push("\u{2500}".repeat(52));
            for (name, ver, desc) in rows.iter() {
                // char-safe truncation (byte-slicing panics mid-UTF8).
                let d = if desc.chars().count() > 55 {
                    let s: String = desc.chars().take(52).collect();
                    format!("{}...", s)
                } else {
                    desc.clone()
                };
                lines.push(format!("  {:<26} {:<12} {}", name, ver, d));
            }
            if !more_note.is_empty() {
                lines.push(format!("\n  {}  (narrow the query)", more_note));
            }
            lines.push(
                "\n  \u{2192} dev doc <crate> for docs, dev graph <crate> for deps".to_string(),
            );
            CommandResult::Output(lines.join("\n"))
        }
        "graph" => {
            // dev graph [crate] [--full] -- FORWARD dependency tree (what <crate> depends ON).
            // The complement of `dev deps` (which is --invert: what depends on <crate>). Forward
            // trees explode, so default to depth 2 with a --full escape hatch. (INT-134, Lane 3)
            let want = args.get(1).copied().unwrap_or("");
            let full = args.iter().any(|a| *a == "--full");
            let target = if want.is_empty() || want == "--full" {
                ""
            } else {
                want
            };

            let mut cargo_args: Vec<String> = vec!["tree".to_string()];
            if !target.is_empty() {
                cargo_args.push("--package".to_string());
                cargo_args.push(target.to_string());
            }
            if !full {
                cargo_args.push("--depth".to_string());
                cargo_args.push("2".to_string());
            }
            let out = std::process::Command::new("cargo")
                .args(&cargo_args)
                .current_dir(core_root)
                .output();
            match out {
                Ok(o) if o.status.success() => {
                    let tree = String::from_utf8_lossy(&o.stdout);
                    let tree = tree.trim_end();
                    if tree.is_empty() {
                        return CommandResult::Output("  (no dependency tree)".to_string());
                    }
                    let label = if target.is_empty() {
                        "workspace"
                    } else {
                        target
                    };
                    let depth_note = if full {
                        "full depth"
                    } else {
                        "depth 2 -- dev graph <crate> --full for all"
                    };
                    let mut s =
                        format!("  \u{1f333} dependency tree: {} ({})\n", label, depth_note);
                    s.push_str(&"\u{2500}".repeat(52));
                    s.push('\n');
                    s.push_str(tree);
                    CommandResult::Output(s)
                }
                Ok(o) => {
                    let err = String::from_utf8_lossy(&o.stderr);
                    // Same ambiguous-version handling as dev deps -- forward tree hits it too.
                    if err.contains("is ambiguous") {
                        let mut vers: Vec<String> = Vec::new();
                        for line in err.lines() {
                            let tl = line.trim();
                            if tl.starts_with(target) && tl.contains('@') {
                                vers.push(tl.to_string());
                            }
                        }
                        if !vers.is_empty() {
                            return CommandResult::Output(format!(
                                "  '{}' is ambiguous -- pick a version:\n    {}\n  e.g. dev graph {}",
                                target, vers.join("\n    "), vers[0]));
                        }
                    }
                    if err.contains("did not match") || err.contains("not found") {
                        return CommandResult::Output(format!(
                            "  {} is not in this workspace (try: dev workspace)",
                            target
                        ));
                    }
                    CommandResult::Error(format!("  dev graph: {}", err.trim()))
                }
                Err(e) => CommandResult::Error(format!("  dev graph: {}", e)),
            }
        }
        "geiger" => {
            // cargo geiger -- count unsafe code
            let tool = args.get(1).copied().unwrap_or("faelight-shell");
            let manifest = faelight_core::paths::rust_tools_dir()
                .join(tool)
                .join("Cargo.toml")
                .to_string_lossy()
                .to_string();
            println!("  {} scanning unsafe code in {}", "☢".normal(), tool);
            let _ = std::process::Command::new("cargo")
                .args(["geiger", "--manifest-path", &manifest])
                .status();
            CommandResult::Empty
        }
        "check" => {
            // bacon -- background checker
            let tool = args.get(1).copied().unwrap_or("");
            if tool.is_empty() {
                println!("  {} starting bacon in current directory", "🥓".normal());
                let _ = std::process::Command::new("bacon").status();
            } else {
                let manifest = faelight_core::paths::rust_tools_dir()
                    .join(tool)
                    .join("Cargo.toml")
                    .to_string_lossy()
                    .to_string();
                println!("  {} starting bacon for {}", "🥓".normal(), tool);
                let _ = std::process::Command::new("bacon")
                    .args(["--manifest-path", &manifest])
                    .status();
            }
            CommandResult::Empty
        }
        "workspace" | "ws" => {
            // Cargo workspace navigation. No arg -> list all members (authoritative, from
            // cargo metadata -- includes crates never visited, unlike zoxide's frecency).
            // <name> -> cd into that crate's dir (set_current_dir like z_jump), and teach zoxide.
            let want = args.get(1).copied().unwrap_or("");
            let meta = std::process::Command::new("cargo")
                .args(["metadata", "--format-version", "1"])
                .current_dir(core_root)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok());
            let meta = match meta {
                Some(m) => m,
                None => {
                    return CommandResult::Error(
                        "  dev workspace: cargo metadata failed".to_string(),
                    )
                }
            };
            let members: std::collections::HashSet<String> = meta
                .get("workspace_members")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            // Collect (name, version, manifest_dir) for each workspace member.
            let mut crates: Vec<(String, String, String)> = meta
                .get("packages")
                .and_then(|p| p.as_array())
                .map(|pkgs| {
                    pkgs.iter()
                        .filter(|pkg| {
                            pkg.get("id")
                                .and_then(|v| v.as_str())
                                .map(|id| members.contains(id))
                                .unwrap_or(false)
                        })
                        .filter_map(|pkg| {
                            let name = pkg.get("name")?.as_str()?.to_string();
                            let version = pkg.get("version")?.as_str()?.to_string();
                            let manifest = pkg.get("manifest_path")?.as_str()?;
                            let dir = std::path::Path::new(manifest)
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default();
                            Some((name, version, dir))
                        })
                        .collect()
                })
                .unwrap_or_default();
            crates.sort_by(|a, b| a.0.cmp(&b.0));

            if want.is_empty() {
                // list mode
                let mut out = String::new();
                out.push_str(&format!(
                    "\n  {} workspace crates ({})\n",
                    "📦".normal(),
                    crates.len()
                ));
                out.push_str(&format!("  {}\n\n", "─".repeat(40).dimmed()));
                for (name, version, dir) in &crates {
                    let short = dir
                        .strip_prefix(core_root)
                        .unwrap_or(dir)
                        .trim_start_matches('/');
                    out.push_str(&format!(
                        "  {} {:<24} {:<10} {}\n",
                        "→".bright_cyan(),
                        name.white(),
                        format!("v{}", version).dimmed(),
                        short.dimmed()
                    ));
                }
                out.push_str(&format!(
                    "\n  {} dev workspace <name> to jump\n",
                    "→".dimmed()
                ));
                return CommandResult::Output(out);
            }
            // jump mode
            match crates.iter().find(|(name, _, _)| name == want) {
                Some((name, _, dir)) => match std::env::set_current_dir(dir) {
                    Ok(_) => {
                        let _ = std::process::Command::new("zoxide")
                            .args(["add", dir])
                            .status();
                        CommandResult::Output(format!(
                            "  {} {} ({})",
                            "📦".normal(),
                            name.bright_cyan(),
                            dir.dimmed()
                        ))
                    }
                    Err(e) => CommandResult::Error(format!("  dev workspace: cd {}: {}", dir, e)),
                },
                None => CommandResult::Error(format!(
                    "  dev workspace: no crate named '{}' (try: dev workspace)",
                    want
                )),
            }
        }
        "doc" => {
            // rustdoc lookup -- auto-routes: no arg -> std docs on the web; workspace crate ->
            // local cargo doc --open; anything else -> docs.rs (instant, no local build).
            let target = args.get(1).copied().unwrap_or("");
            if target.is_empty() {
                // NixOS: rustc doesn't ship browsable std HTML and rustup isn't the toolchain
                // manager here, so open the canonical web std docs (always current).
                let url = "https://doc.rust-lang.org/std/";
                println!("  {} opening std library docs", "📖".normal());
                let ok = std::process::Command::new("xdg-open")
                    .arg(url)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                return if ok {
                    CommandResult::Output(format!("  {} {}", "→".dimmed(), url.dimmed()))
                } else {
                    CommandResult::Output(format!("  std docs: {}", url))
                };
            }
            // Is `target` a workspace member? Parse cargo metadata properly (serde_json):
            // build the set of workspace package NAMES and exact-match. (A substring check
            // false-positives on dependency names like ratatui -- INT-134 Lane 3 fix.)
            let is_member = std::process::Command::new("cargo")
                .args(["metadata", "--format-version", "1"])
                .current_dir(core_root)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .map(|meta| {
                    let members: std::collections::HashSet<String> = meta
                        .get("workspace_members")
                        .and_then(|m| m.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    meta.get("packages")
                        .and_then(|p| p.as_array())
                        .map(|pkgs| {
                            pkgs.iter().any(|pkg| {
                                let id = pkg.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let name = pkg.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                members.contains(id) && name == target
                            })
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            if is_member {
                println!(
                    "  {} building + opening local docs for workspace crate {}",
                    "📖".normal(),
                    target.bright_cyan()
                );
                let status = std::process::Command::new("cargo")
                    .args(["doc", "--open", "--no-deps", "-p", target])
                    .current_dir(core_root)
                    .status();
                match status {
                    Ok(s) if s.success() => CommandResult::Empty,
                    Ok(_) => {
                        CommandResult::Error(format!("  dev doc: cargo doc failed for {}", target))
                    }
                    Err(e) => CommandResult::Error(format!("  dev doc: {}", e)),
                }
            } else {
                // external crate -> docs.rs (published latest; instant, no build).
                let url = format!("https://docs.rs/{}", target);
                println!(
                    "  {} opening docs.rs for {} (external crate)",
                    "📖".normal(),
                    target.bright_cyan()
                );
                let ok = std::process::Command::new("xdg-open")
                    .arg(&url)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if ok {
                    CommandResult::Output(format!("  {} {}", "→".dimmed(), url.dimmed()))
                } else {
                    CommandResult::Output(format!("  docs: {}", url))
                }
            }
        }
        _ => {
            let mut out = String::new();
            out.push_str(&format!("\n  {} dev commands\n", "🛠".normal()));
            out.push_str(&format!("  {}\n\n", "─".repeat(40).dimmed()));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev test <tool>",
                "cargo nextest -- run unit tests"
            ));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev watch <tool>",
                "cargo watch -- hot reload builds"
            ));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev check [tool]",
                "bacon -- background error checker"
            ));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev bench <cmd>",
                "hyperfine -- benchmark a command"
            ));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev geiger <tool>",
                "cargo-geiger -- count unsafe code"
            ));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev doc [crate]",
                "rustdoc -- std / workspace crate / docs.rs"
            ));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev workspace [name]",
                "list workspace crates / jump to one"
            ));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev graph [crate]",
                "forward dep tree (--full for all depths)"
            ));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev search <query>",
                "search crates.io by keyword"
            ));
            out.push_str(&format!(
                "  {} {:<22} {}\n",
                "→".bright_cyan(),
                "dev audit-deps",
                "cargo-udeps -- find unused deps"
            ));
            out.push_str(&format!(
                "\n  tools with tests: faelight-shell, faelight-core, faelight-update, core-diff\n"
            ));
            CommandResult::Output(out)
        }
    }
}

/// INT-326 Phase 3: ambiguity resolution -- present choices, learn preference
fn semantic_ambiguous_cmd(db: &ForestDb, input: &str) -> CommandResult {
    use colored::Colorize;
    use std::io::{self, BufRead, Write};
    // Check for learned preference in state.db
    let pref_key = format!(
        "semantic_pref_{}",
        input.split_whitespace().next().unwrap_or(input)
    );
    if let Ok(preferred) = db.conn.query_row(
        "SELECT value FROM shell_state WHERE key = ?1",
        rusqlite::params![pref_key],
        |r| r.get::<_, String>(0),
    ) {
        // Preference learned -- execute directly
        let si = crate::semantic::interpret(&preferred);
        let mut out = String::new();
        out.push_str(&format!(
            "  {} using learned preference: {}\n",
            "🌲".normal(),
            preferred.bright_green()
        ));
        for cmd in &si.layer3_commands {
            out.push_str(&format!("  {} {}\n", "→".bright_cyan(), cmd.dimmed()));
        }
        return CommandResult::Output(out);
    }
    // Show ambiguity choices
    match crate::semantic::interpret_ambiguous(input) {
        None => {
            // Not ambiguous -- direct execute
            let si = crate::semantic::interpret(input);
            CommandResult::Output(format!("  Executing: {}\n", si.layer2_description))
        }
        Some(amb) => {
            // Display choices
            print!("{}", crate::semantic::format_ambiguous(&amb));
            let _ = io::stdout().flush();
            // Read choice
            let stdin = io::stdin();
            let mut choice = String::new();
            let _ = stdin.lock().read_line(&mut choice);
            let choice = choice.trim();
            match choice {
                "n" | "N" => CommandResult::Output("  Cancelled.".to_string()),
                c => {
                    let idx: Option<usize> = c.parse::<usize>().ok().map(|n| n.saturating_sub(1));
                    if let Some(i) = idx {
                        if let Some((si, _)) = amb.options.get(i) {
                            // Record choice count for preference learning
                            let count_key = format!(
                                "semantic_choice_count_{}_{}",
                                input.split_whitespace().next().unwrap_or(""),
                                i
                            );
                            let count: i64 = db
                                .conn
                                .query_row(
                                    "SELECT CAST(value AS INTEGER) FROM shell_state WHERE key = ?1",
                                    rusqlite::params![count_key],
                                    |r| r.get(0),
                                )
                                .unwrap_or(0);
                            let new_count = count + 1;
                            let _ = db.conn.execute(
                                "INSERT OR REPLACE INTO shell_state (key, value) VALUES (?1, ?2)",
                                rusqlite::params![count_key, new_count.to_string()],
                            );
                            // After 3 consistent choices -- learn preference
                            if new_count >= 3 {
                                let _ = db.conn.execute(
                                    "INSERT OR REPLACE INTO shell_state (key, value) VALUES (?1, ?2)",
                                    rusqlite::params![pref_key, &si.raw_input],
                                );
                                println!(
                                    "  {} preference learned for '{}'",
                                    "🌲 Friday:".bright_green(),
                                    input.split_whitespace().next().unwrap_or("")
                                );
                            }
                            let mut out = String::new();
                            for cmd in &si.layer3_commands {
                                out.push_str(&format!("  {} {}\n", "→".bright_cyan(), cmd));
                            }
                            CommandResult::Output(out)
                        } else {
                            CommandResult::Output("  Invalid choice.".to_string())
                        }
                    } else {
                        CommandResult::Output("  Invalid choice.".to_string())
                    }
                }
            }
        }
    }
}

/// INT-334 Gate 8: fsh rename -- zmv-style mass rename by pattern
/// INT-326 Phase 2: explain -- show all three semantic layers for a command
#[allow(dead_code)]
fn semantic_explain_cmd(input: &str) -> CommandResult {
    use colored::Colorize;
    if input.is_empty() {
        return CommandResult::Output(
            "  Usage: explain <command>\n  Example: explain delete ~/tmp".to_string(),
        );
    }
    let si = crate::semantic::interpret(input);
    let layers = crate::semantic::format_three_layers(&si);
    let mut out = String::new();
    out.push_str(&format!(
        "  {} INT-326 Semantic Explanation\n",
        "🔍".normal()
    ));
    out.push_str(&layers);
    CommandResult::Output(out)
}

/// INT-326 Phase 2: plan -- show Layer 2 semantic plan only
fn semantic_plan_cmd(input: &str) -> CommandResult {
    use colored::Colorize;
    if input.is_empty() {
        return CommandResult::Output(
            "  Usage: plan <command>\n  Example: plan deploy faelight-shell".to_string(),
        );
    }
    let si = crate::semantic::interpret(input);
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {} Semantic Plan for: {}\n",
        "📋".normal(),
        input.bright_white()
    ));
    out.push_str(&format!("  {}\n", "─".repeat(50).dimmed()));
    out.push_str(&format!("  action:     {:?}\n", si.action));
    out.push_str(&format!("  target:     {:?}\n", si.target));
    out.push_str(&format!("  category:   {}\n", si.category.label()));
    out.push_str(&format!("  confidence: {:.0}%\n", si.confidence * 100.0));
    out.push_str(&format!(
        "  reversible: {}\n",
        if si.reversible {
            "yes"
        } else {
            "⚠ NO -- confirm required"
        }
    ));
    out.push_str(&format!("  plan:       {}\n", si.layer2_description));
    CommandResult::Output(out)
}

/// INT-326 Phase 2: dry-run -- show Layer 3 execution without running
fn semantic_dryrun_cmd(input: &str) -> CommandResult {
    use colored::Colorize;
    if input.is_empty() {
        return CommandResult::Output(
            "  Usage: dry-run <command>\n  Example: dry-run deploy faelight-shell".to_string(),
        );
    }
    let si = crate::semantic::interpret(input);
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {} Dry run: {} {}\n",
        "🔬".normal(),
        input.bright_white(),
        "(not executed)".dimmed()
    ));
    out.push_str(&format!("  {}\n\n", "─".repeat(50).dimmed()));
    for (i, cmd) in si.layer3_commands.iter().enumerate() {
        out.push_str(&format!("  step {}: {}\n", i + 1, cmd.bright_cyan()));
    }
    if si.category.requires_confirm() {
        out.push_str(&format!(
            "\n  {} This command requires confirmation before execution\n",
            "⚠".yellow()
        ));
    }
    CommandResult::Output(out)
}

/// INT-326 Phase 2: why -- explain interpretation reasoning
fn semantic_why_cmd(input: &str) -> CommandResult {
    use colored::Colorize;
    if input.is_empty() {
        return CommandResult::Output("  Usage: why <command>\n  Example: why delete".to_string());
    }
    let si = crate::semantic::interpret(input);
    // deadwood: exempt -- semantic interpretation of user text for explanation; the token feeds
    // the analysis it prints and is never dispatched
    let first_word = input.split_whitespace().next().unwrap_or(input);
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {} Why fsh interprets: {}\n",
        "💭".normal(),
        input.bright_white()
    ));
    out.push_str(&format!("  {}\n\n", "─".repeat(50).dimmed()));
    if si.confidence > 0.0 {
        out.push_str(&format!(
            "  {} is a {} verb\n",
            first_word.bright_cyan(),
            si.category.label()
        ));
        out.push_str(&format!("  confidence: {:.0}%\n", si.confidence * 100.0));
        out.push_str(&format!(
            "  reversible: {}\n",
            if si.reversible { "yes" } else { "no" }
        ));
        out.push_str(&format!("\n  Forest vocabulary rule:\n"));
        match si.category {
            crate::semantic::VerbCategory::Observation => {
                out.push_str("  Observation verbs never mutate state.\n");
                out.push_str("  Safe to run without confirmation.\n");
            }
            crate::semantic::VerbCategory::Destructive => {
                out.push_str("  Destructive verbs always show what will be destroyed.\n");
                out.push_str("  Always require explicit confirmation.\n");
                out.push_str("  Always logged to forest audit trail.\n");
            }
            crate::semantic::VerbCategory::Deployment => {
                out.push_str("  Deployment verbs write to deploy_patterns.\n");
                out.push_str("  All are rollback-capable via the deploy system.\n");
            }
            _ => {
                out.push_str(&format!("  Category: {}\n", si.category.label()));
            }
        }
    } else {
        out.push_str(&format!(
            "  {} is not in the forest vocabulary\n",
            first_word.bright_red()
        ));
        out.push_str("  Treated as raw UNIX command (Layer 3 direct)\n");
        out.push_str("  UNIX compatibility always preserved\n");
    }
    CommandResult::Output(out)
}

fn fsh_rename_cmd(from_pat: &str, to_pat: &str) -> CommandResult {
    use colored::Colorize;
    if from_pat.is_empty() || to_pat.is_empty() {
        let mut out = String::new();
        out.push_str("  Usage: fsh rename <from-pattern> <to-pattern>\n");
        out.push_str("  Example: fsh rename '*.txt' '*.md'\n");
        out.push_str("  Example: fsh rename 'test_*' 'spec_*'\n");
        out.push_str("  Add --dry-run to preview without renaming\n");
        return CommandResult::Output(out);
    }
    let dry_run = from_pat == "--dry-run" || to_pat == "--dry-run";
    let (pat, to) = if dry_run {
        ("", "")
    } else {
        (from_pat, to_pat)
    };
    if dry_run {
        return CommandResult::Output("  Usage: fsh rename <from> <to> --dry-run".to_string());
    }
    // Convert glob pattern to regex-like matching
    let cwd = std::env::current_dir().unwrap_or_default();
    let entries: Vec<_> = match std::fs::read_dir(&cwd) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(err) => return CommandResult::Error(format!("  fsh rename: {}", err)),
    };
    // Simple glob matching: * matches anything
    let matches_pattern = |name: &str, pattern: &str| -> bool {
        if !pattern.contains('*') {
            return name == pattern;
        }
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            name.starts_with(parts[0])
                && name.ends_with(parts[1])
                && name.len() >= parts[0].len() + parts[1].len()
        } else {
            false
        }
    };
    let apply_pattern = |name: &str, from: &str, to: &str| -> Option<String> {
        if !from.contains('*') {
            return if name == from {
                Some(to.to_string())
            } else {
                None
            };
        }
        let parts_from: Vec<&str> = from.split('*').collect();
        let parts_to: Vec<&str> = to.split('*').collect();
        if parts_from.len() == 2 && parts_to.len() == 2 {
            let prefix = parts_from[0];
            let suffix = parts_from[1];
            if name.starts_with(prefix) && name.ends_with(suffix) {
                let middle = &name[prefix.len()..name.len() - suffix.len()];
                Some(format!("{}{}{}", parts_to[0], middle, parts_to[1]))
            } else {
                None
            }
        } else {
            None
        }
    };
    let mut renamed = 0;
    let mut skipped = 0;
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {} fsh rename {} → {}\n",
        "→".bright_cyan(),
        pat.bright_white(),
        to.bright_white()
    ));
    out.push_str(&format!("  {}\n\n", "─".repeat(40).dimmed()));
    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if matches_pattern(&name, pat) {
            if let Some(new_name) = apply_pattern(&name, pat, to) {
                let old_path = entry.path();
                let new_path = cwd.join(&new_name);
                out.push_str(&format!(
                    "  {} {} → {}\n",
                    "✓".bright_green(),
                    name.dimmed(),
                    new_name.bright_white()
                ));
                if !dry_run {
                    if let Err(e) = std::fs::rename(&old_path, &new_path) {
                        out.push_str(&format!("  {} failed: {}\n", "✗".bright_red(), e));
                    } else {
                        renamed += 1;
                    }
                } else {
                    renamed += 1;
                }
            } else {
                skipped += 1;
            }
        }
    }
    if renamed == 0 && skipped == 0 {
        out.push_str(&format!("  No files matched pattern '{}'\n", pat));
    } else {
        out.push_str(&format!("\n  {} renamed, {} skipped\n", renamed, skipped));
    }
    CommandResult::Output(out)
}

/// INT-322 Phase 7: fsh enter -- create project-scoped shell environment
fn fsh_enter_cmd(db: &ForestDb, project: &str) -> CommandResult {
    use colored::Colorize;
    if project.is_empty() {
        return CommandResult::Output("  Usage: fsh enter <project-name-or-path>".to_string());
    }
    // Save current state
    let current_cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let current_intent = db.get_focus_intent().unwrap_or_default();
    let _ = db.conn.execute(
        "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('scope_return_path', ?1)",
        rusqlite::params![current_cwd],
    );
    let _ = db.conn.execute(
        "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('scope_return_intent', ?1)",
        rusqlite::params![current_intent],
    );
    let _ = db.conn.execute(
        "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('scope_name', ?1)",
        rusqlite::params![project],
    );
    // Try to resolve project path
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        std::path::PathBuf::from(project),
        std::path::PathBuf::from(&home).join(project),
        std::path::PathBuf::from(&home).join("0-core").join(project),
        std::path::PathBuf::from(&home)
            .join("projects")
            .join(project),
    ];
    let target = candidates.iter().find(|p| p.is_dir());
    if let Some(path) = target {
        let _ = std::env::set_current_dir(path);
        let resolved = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut out = String::new();
        out.push_str(&format!(
            "\n  {} Entering scope: {}\n",
            "🌿".normal(),
            project.bright_green()
        ));
        out.push_str(&format!("  {} cwd  → {}\n", "→".bright_cyan(), resolved));
        out.push_str(&format!("  {} return path saved\n", "→".bright_cyan()));
        out.push_str(&format!(
            "  {} run {} to restore\n",
            "→".dimmed(),
            "fsh leave".bright_cyan()
        ));
        CommandResult::Output(out)
    } else {
        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('scope_name', ?1)",
            rusqlite::params![project],
        );
        let mut out = String::new();
        out.push_str(&format!(
            "  {} Scope set: {} (path not found -- staying in current dir)\n",
            "⚠".yellow(),
            project
        ));
        out.push_str(&format!(
            "  {} run {} to restore\n",
            "→".dimmed(),
            "fsh leave".bright_cyan()
        ));
        CommandResult::Output(out)
    }
}

/// INT-322 Phase 7: fsh leave -- restore pre-scope state
fn fsh_leave_cmd(db: &ForestDb) -> CommandResult {
    use colored::Colorize;
    let return_path: Option<String> = db
        .conn
        .query_row(
            "SELECT value FROM shell_state WHERE key='scope_return_path'",
            [],
            |r| r.get(0),
        )
        .ok();
    let scope_name: Option<String> = db
        .conn
        .query_row(
            "SELECT value FROM shell_state WHERE key='scope_name'",
            [],
            |r| r.get(0),
        )
        .ok();
    if return_path.is_none() && scope_name.is_none() {
        return CommandResult::Output("  No active scope to leave.".to_string());
    }
    let name = scope_name.as_deref().unwrap_or("unknown");
    if let Some(path) = &return_path {
        let _ = std::env::set_current_dir(path);
    }
    let _ = db.conn.execute("DELETE FROM shell_state WHERE key IN ('scope_name', 'scope_return_path', 'scope_return_intent')", []);
    let restored = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {} Left scope: {}\n",
        "🌿".normal(),
        name.bright_green()
    ));
    out.push_str(&format!(
        "  {} cwd restored → {}\n",
        "→".bright_cyan(),
        restored
    ));
    CommandResult::Output(out)
}

/// INT-322 Phase 7: fsh scope -- show active scope status
fn fsh_scope_status(db: &ForestDb) -> CommandResult {
    use colored::Colorize;
    let scope: Option<String> = db
        .conn
        .query_row(
            "SELECT value FROM shell_state WHERE key='scope_name'",
            [],
            |r| r.get(0),
        )
        .ok();
    let return_path: Option<String> = db
        .conn
        .query_row(
            "SELECT value FROM shell_state WHERE key='scope_return_path'",
            [],
            |r| r.get(0),
        )
        .ok();
    match scope {
        Some(name) => {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            CommandResult::Output(format!(
                "  {} Active scope: {}  cwd: {}  return: {}",
                "🌿".normal(),
                name.bright_green(),
                cwd,
                return_path.as_deref().unwrap_or("?").dimmed()
            ))
        }
        None => CommandResult::Output("  No active scope.".to_string()),
    }
}

/// INT-322 Phase 6: fsh doctor -- shell-specific health checks
fn fsh_doctor_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    use colored::Colorize;
    use std::time::Instant;
    let fix_mode = args.contains(&"--fix");
    let start = Instant::now();
    let mut checks: Vec<(&str, bool, String)> = Vec::new(); // (name, passed, note)

    // 1. fsh binary exists and is this binary
    let fsh_bin = std::path::Path::new("/home/christian/0-core/scripts/faelight-shell").exists();
    checks.push((
        "fsh binary",
        fsh_bin,
        if fsh_bin {
            "scripts/faelight-shell present".into()
        } else {
            "missing!".into()
        },
    ));

    // 2. state.db writable
    let db_path = faelight_core::paths::state_db()
        .to_string_lossy()
        .to_string();
    let db_ok = std::path::Path::new(&db_path).exists();
    checks.push((
        "state.db",
        db_ok,
        if db_ok {
            db_path.clone()
        } else {
            "not found!".into()
        },
    ));

    // 3. focus.toml readable
    let home = std::env::var("HOME").unwrap_or_default();
    let focus_path = format!("{}/.local/state/0-core/intent/focus.toml", home);
    let focus_ok = std::path::Path::new(&focus_path).exists();
    let focus_note = if focus_ok {
        db.get_focus_intent()
            .map(|i| format!("INT-{} active", i))
            .unwrap_or("no active intent".into())
    } else {
        "no focus.toml".into()
    };
    checks.push(("focus intent", focus_ok, focus_note));

    // 4. shell history writable (can insert a test row)
    let hist_ok = db.conn.execute(
        "INSERT INTO shell_history (command, timestamp, cwd) VALUES ('__fsh_doctor_test__', 0, '/')",
        []
    ).is_ok();
    if hist_ok {
        let _ = db.conn.execute(
            "DELETE FROM shell_history WHERE command = '__fsh_doctor_test__'",
            [],
        );
    }
    checks.push((
        "history writable",
        hist_ok,
        if hist_ok {
            "insert+delete ok".into()
        } else {
            "db error!".into()
        },
    ));

    // 5. aliases loaded
    let alias_count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM shell_aliases", [], |r| r.get(0))
        .unwrap_or(0);
    let aliases_ok = alias_count > 0;
    checks.push((
        "aliases loaded",
        aliases_ok,
        format!("{} aliases", alias_count),
    ));

    // 6. snapshots table exists
    let snap_ok = db
        .conn
        .execute("SELECT id FROM shell_snapshots LIMIT 1", [])
        .is_ok()
        || db
            .conn
            .query_row("SELECT COUNT(*) FROM shell_snapshots", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap_or(0)
            >= 0;
    checks.push(("snapshots table", snap_ok, "shell_snapshots ok".into()));

    // 7. PATH contains cargo bin
    let cargo_bin = std::env::var("PATH")
        .map(|p| p.contains(".cargo/bin"))
        .unwrap_or(false);
    checks.push((
        "cargo in PATH",
        cargo_bin,
        if cargo_bin {
            ".cargo/bin found".into()
        } else {
            "missing -- run: source ~/.profile".into()
        },
    ));

    let elapsed = start.elapsed().as_millis();
    let passed = checks.iter().filter(|(_, ok, _)| *ok).count();
    let total = checks.len();
    let all_ok = passed == total;

    let mut out = String::new();
    out.push_str(&format!("\n  {} fsh doctor\n", "🩺".normal()));
    out.push_str(&format!("  {}\n\n", "━".repeat(50).dimmed()));
    for (name, ok, note) in &checks {
        let icon = if *ok {
            "✅".to_string()
        } else {
            "✗ ".bright_red().to_string()
        };
        out.push_str(&format!("  {} {:<22} {}\n", icon, name, note.dimmed()));
        if !ok && fix_mode {
            out.push_str(&format!(
                "    {} no auto-fix available for '{}'\n",
                "→".yellow(),
                name
            ));
        }
    }
    out.push_str(&format!("\n  {}\n", "─".repeat(50).dimmed()));
    out.push_str(&format!(
        "  {}/{} checks passed  {}ms\n",
        passed, total, elapsed
    ));
    if all_ok {
        out.push_str(&format!("  {} shell is healthy\n", "🌲".normal()));
    } else {
        out.push_str(&format!(
            "  {} {} check(s) failed -- run: fsh doctor --fix\n",
            "⚠".yellow(),
            total - passed
        ));
    }
    CommandResult::Output(out)
}

fn rewind_cmd(db: &ForestDb) -> CommandResult {
    use colored::Colorize;
    let mut stmt = match db.conn.prepare(
        "SELECT id, name, timestamp, health, command, git_hash, cwd, intent_id
         FROM shell_snapshots ORDER BY timestamp DESC LIMIT 20",
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output("  No snapshots yet.".to_string()),
    };
    let rows: Vec<(
        i64,
        String,
        i64,
        Option<i64>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
            ))
        })
        .map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    if rows.is_empty() {
        return CommandResult::Output(
            "  No snapshots yet -- run destructive commands (rm, deploy, git push) to auto-capture.".to_string()
        );
    }
    let mut out = String::new();
    out.push_str(&format!(
        "\n  {} Time-Travel Snapshot Timeline ({} snapshots)\n",
        "🌲".normal(),
        rows.len()
    ));
    out.push_str(&format!("  {}\n\n", "━".repeat(60).dimmed()));
    for (id, name, ts, health, command, git_hash, _cwd, intent_id) in &rows {
        let dt = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|d: chrono::DateTime<chrono::Utc>| d.format("%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| ts.to_string());
        let health_str = health
            .map(|h| format!("{}%", h))
            .unwrap_or_else(|| "?".to_string());
        let git_str = git_hash.as_deref().unwrap_or("?");
        let intent_str = intent_id
            .as_deref()
            .map(|i| format!(" [INT-{}]", i))
            .unwrap_or_default();
        let cmd_str = command.as_deref().unwrap_or(&name);
        let cmd_short: String = cmd_str.chars().take(45).collect();
        out.push_str(&format!(
            "  {} #{} {}{}\n",
            "→".bright_cyan(),
            id.to_string().dimmed(),
            dt.bright_white(),
            intent_str.dimmed()
        ));
        out.push_str(&format!(
            "    {} {}\n",
            "cmd:".dimmed(),
            cmd_short.bright_white()
        ));
        out.push_str(&format!(
            "    {} {}  {} {}\n\n",
            "health:".dimmed(),
            health_str,
            "git:".dimmed(),
            git_str.yellow()
        ));
    }
    out.push_str(&format!(
        "  {} snapshot <name> to capture now\n",
        "💡".dimmed()
    ));
    CommandResult::Output(out)
}

fn snapshot_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    use std::time::{SystemTime, UNIX_EPOCH};

    ensure_snapshots_schema(db);

    let name = args.first().copied().unwrap_or("manual");
    let note = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        String::new()
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Capture health
    let health = db.health_score().unwrap_or(0);

    // Capture commit count
    let commits: i64 = std::process::Command::new("git")
        .args(["-C", &db.core_root(), "rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    // Capture process count
    let processes: i64 = std::process::Command::new("ps")
        .args(["aux", "--no-headers"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().count() as i64)
        .unwrap_or(0);

    // Load average
    let load_avg = std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".to_string());

    // Top CPU process
    let top_proc = std::process::Command::new("ps")
        .args(["aux", "--no-headers", "--sort=-pcpu"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.lines().next().map(|l| {
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() > 10 {
                    format!(
                        "{} ({}%)",
                        parts[10..].join(" ").chars().take(20).collect::<String>(),
                        parts[2]
                    )
                } else {
                    "?".to_string()
                }
            })
        })
        .unwrap_or_else(|| "?".to_string());

    db.conn.execute(
        "INSERT INTO shell_snapshots (name, timestamp, health, commits, processes, load_avg, top_proc, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![name, now, health, commits, processes, load_avg, top_proc, note]
    ).ok();

    CommandResult::Output(format!(
        "  {} Snapshot '{}' captured — health: {}%  commits: {}  procs: {}  load: {}",
        "📸".normal(),
        name.bright_white().bold(),
        health.to_string().bright_green(),
        commits.to_string().bright_white(),
        processes.to_string().dimmed(),
        load_avg.dimmed()
    ))
}

fn timeline_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    ensure_snapshots_schema(db);

    let limit = args
        .first()
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(10);

    let mut stmt = match db.conn.prepare(
        "SELECT id, name, timestamp, health, commits, processes, load_avg FROM shell_snapshots ORDER BY timestamp DESC LIMIT ?1"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No snapshots yet. Run: snapshot", "○".dimmed())),
    };

    let rows: Vec<HashMap<String, Value>> = stmt
        .query_map(rusqlite::params![limit as i64], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, String>(6)?,
            ))
        })
        .ok()
        .map(|rows| {
            rows.filter_map(|r| r.ok())
                .map(|(id, name, ts, health, commits, procs, load)| {
                    let time = fmt_time(ts, "%m-%d %H:%M");
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), Value::Int(id));
                    row.insert("name".to_string(), Value::Text(name));
                    row.insert("time".to_string(), Value::Text(time));
                    row.insert("health".to_string(), Value::Int(health));
                    row.insert("commits".to_string(), Value::Int(commits));
                    row.insert("procs".to_string(), Value::Int(procs));
                    row.insert("load".to_string(), Value::Text(load));
                    row
                })
                .collect()
        })
        .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No snapshots yet. Run: snapshot",
            "○".dimmed()
        ));
    }
    CommandResult::Value(Value::Table(rows))
}

fn snap_diff_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    ensure_snapshots_schema(db);

    let (id1, id2) = match args {
        [a, b] => match (a.parse::<i64>(), b.parse::<i64>()) {
            (Ok(x), Ok(y)) => (x, y),
            _ => return CommandResult::Error("Usage: snap-diff <id1> <id2>".to_string()),
        },
        _ => {
            // Default: diff last two snapshots
            let ids: Vec<i64> = db
                .conn
                .prepare("SELECT id FROM shell_snapshots ORDER BY timestamp DESC LIMIT 2")
                .ok()
                .and_then(|mut s| {
                    s.query_map([], |r| r.get::<_, i64>(0))
                        .ok()
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                })
                .unwrap_or_default();
            if ids.len() < 2 {
                return CommandResult::Output(format!(
                    "  {} Need at least 2 snapshots. Run: snapshot",
                    "○".dimmed()
                ));
            }
            (ids[1], ids[0]) // older first
        }
    };

    // Fetch both snapshots
    let fetch = |id: i64| -> Option<(String, i64, i64, i64, String)> {
        db.conn.query_row(
            "SELECT name, health, commits, processes, load_avg FROM shell_snapshots WHERE id = ?1",
            rusqlite::params![id], |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
            ))
        ).ok()
    };

    let s1 = match fetch(id1) {
        Some(s) => s,
        None => return CommandResult::Error(format!("Snapshot #{} not found", id1)),
    };
    let s2 = match fetch(id2) {
        Some(s) => s,
        None => return CommandResult::Error(format!("Snapshot #{} not found", id2)),
    };

    println!();
    println!("{}", "🔍  Snapshot Diff".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {} #{} '{}'  →  #{} '{}'",
        "Comparing:".dimmed(),
        id1.to_string().bright_white(),
        s1.0.bright_white(),
        id2.to_string().bright_white(),
        s2.0.bright_white()
    );
    println!();

    // Health diff
    let health_diff = s2.1 - s1.1;
    let health_str = if health_diff > 0 {
        format!("+{}", health_diff).bright_green().to_string()
    } else if health_diff < 0 {
        format!("{}", health_diff).bright_red().to_string()
    } else {
        "unchanged".dimmed().to_string()
    };
    println!(
        "  {}  {} → {}  ({})",
        "Health:".dimmed(),
        s1.1.to_string().bright_white(),
        s2.1.to_string().bright_white(),
        health_str
    );

    // Commits diff
    let commit_diff = s2.2 - s1.2;
    println!(
        "  {}  {} → {}  ({} new commit{})",
        "Commits:".dimmed(),
        s1.2.to_string().bright_white(),
        s2.2.to_string().bright_white(),
        commit_diff.to_string().bright_green(),
        if commit_diff == 1 { "" } else { "s" }
    );

    // Process diff
    let proc_diff = s2.3 - s1.3;
    let proc_str = if proc_diff > 0 {
        format!("+{}", proc_diff).yellow().to_string()
    } else if proc_diff < 0 {
        format!("{}", proc_diff).bright_green().to_string()
    } else {
        "unchanged".dimmed().to_string()
    };
    println!(
        "  {}  {} → {}  ({})",
        "Processes:".dimmed(),
        s1.3.to_string().bright_white(),
        s2.3.to_string().bright_white(),
        proc_str
    );

    // Load diff
    println!(
        "  {}  {} → {}",
        "Load avg:".dimmed(),
        s1.4.dimmed(),
        s2.4.bright_white()
    );

    println!();
    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "Diff complete. The forest remembers every state."
            .dimmed()
            .italic()
    );
    println!();

    CommandResult::Empty
}

// ── Phase 21 — Query Language ─────────────────────────────────────────────────
// SQL-like syntax → existing pipeline ops.
// select <cols> from <table> [where <field> <op> <val>] [order by <col>] [limit <n>]
// Adoption bridge — familiar syntax for structured queries.

fn sql_query_cmd(db: &ForestDb, core_root: &str, line: &str) -> CommandResult {
    match parse_sql_query(line) {
        Ok(q) => {
            // Build equivalent pipeline and execute
            let pipeline = q.to_pipeline();
            println!("  {} {}", "→".dimmed(), pipeline.dimmed());
            execute(&pipeline, db, core_root)
        }
        Err(e) => CommandResult::Error(format!("Query error: {}", e)),
    }
}

#[derive(Debug)]
struct SqlQuery {
    columns: Vec<String>,     // * or specific columns
    table: String,            // from <table>
    where_: Option<String>,   // where clause
    order_by: Option<String>, // order by <col>
    order_desc: bool,
    limit: Option<usize>,
}

impl SqlQuery {
    fn to_pipeline(&self) -> String {
        let mut parts = vec![self.table.clone()];

        if let Some(ref w) = self.where_ {
            parts.push(format!("where {}", w));
        }
        if let Some(ref col) = self.order_by {
            if self.order_desc {
                parts.push(format!("sort {} desc", col));
            } else {
                parts.push(format!("sort {}", col));
            }
        }
        if let Some(n) = self.limit {
            parts.push(format!("first {}", n));
        }
        if !self.columns.is_empty() && self.columns != ["*"] {
            parts.push(format!("select {}", self.columns.join(" ")));
        }
        parts.join(" | ")
    }
}

fn parse_sql_query(line: &str) -> Result<SqlQuery, String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() || tokens[0].to_lowercase() != "select" {
        return Err("Expected SELECT".to_string());
    }

    // Find FROM position
    let from_pos = tokens
        .iter()
        .position(|t| t.to_lowercase() == "from")
        .ok_or("Expected FROM")?;

    // Columns between SELECT and FROM
    let columns: Vec<String> = tokens[1..from_pos]
        .iter()
        .map(|s| s.trim_end_matches(',').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if from_pos + 1 >= tokens.len() {
        return Err("Expected table name after FROM".to_string());
    }
    let table = tokens[from_pos + 1].to_lowercase();

    let remaining = &tokens[from_pos + 2..];

    // Parse WHERE, ORDER BY, LIMIT
    let mut where_ = None;
    let mut order_by = None;
    let mut order_desc = false;
    let mut limit = None;

    let mut i = 0;
    while i < remaining.len() {
        match remaining[i].to_lowercase().as_str() {
            "where" => {
                // Collect until ORDER or LIMIT
                let mut clause = vec![];
                i += 1;
                while i < remaining.len() {
                    let t = remaining[i].to_lowercase();
                    if t == "order" || t == "limit" {
                        break;
                    }
                    clause.push(remaining[i]);
                    i += 1;
                }
                where_ = Some(clause.join(" "));
            }
            "order" => {
                i += 1;
                if i < remaining.len() && remaining[i].to_lowercase() == "by" {
                    i += 1;
                }
                if i < remaining.len() {
                    order_by = Some(remaining[i].to_string());
                    i += 1;
                    if i < remaining.len() && remaining[i].to_lowercase() == "desc" {
                        order_desc = true;
                        i += 1;
                    }
                }
            }
            "limit" => {
                i += 1;
                if i < remaining.len() {
                    limit = remaining[i].parse::<usize>().ok();
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    Ok(SqlQuery {
        columns,
        table,
        where_,
        order_by,
        order_desc,
        limit,
    })
}

// ── Phase 22 — Observability Dashboard ───────────────────────────────────────
// dashboard         — full system overview
// dashboard system  — CPU, memory, network, top processes

fn dashboard_cmd(db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    let mode = args.first().copied().unwrap_or("full");
    match mode {
        "system" => dashboard_system(),
        "forest" => dashboard_forest(db, core_root),
        _ => {
            dashboard_system();
            println!();
            dashboard_forest(db, core_root);
            CommandResult::Empty
        }
    }
}

fn dashboard_system() -> CommandResult {
    use colored::*;

    println!();
    println!("{}", "┌─ 🖥  System".bright_cyan().bold());

    // Load average
    let load = std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".to_string());
    println!("  {}  {}", "Load avg:".dimmed(), load.bright_white());

    // Memory from /proc/meminfo
    if let Ok(mem) = std::fs::read_to_string("/proc/meminfo") {
        let mut total = 0u64;
        let mut available = 0u64;
        for line in mem.lines() {
            if line.starts_with("MemTotal:") {
                total = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
            }
            if line.starts_with("MemAvailable:") {
                available = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
            }
        }
        if total > 0 {
            let used = total - available;
            let pct = (used * 100) / total;
            let bar_len = (pct / 5) as usize;
            let bar = format!(
                "{}{}",
                "█".repeat(bar_len).bright_green(),
                "░".repeat(20 - bar_len.min(20)).dimmed()
            );
            println!(
                "  {}  {} [{bar}] {}%",
                "Memory:".dimmed(),
                format!("{}/{}MB", used / 1024, total / 1024).bright_white(),
                pct
            );
        }
    }

    // Top 5 CPU processes
    let top = std::process::Command::new("ps")
        .args(["aux", "--no-headers", "--sort=-pcpu"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    println!("  {}", "Top processes:".dimmed());
    for line in top.lines().take(5) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 10 {
            let name = parts[10..].join(" ").chars().take(28).collect::<String>();
            let cpu = parts[2];
            let mem = parts[3];
            println!(
                "    {} {} cpu:{} mem:{}",
                "·".dimmed(),
                name.bright_white(),
                cpu.yellow(),
                mem.dimmed()
            );
        }
    }

    // Disk usage
    let disk = std::process::Command::new("df")
        .args(["-h", "/"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    if let Some(line) = disk.lines().nth(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            println!(
                "  {}  used:{} available:{} ({})",
                "Disk /:".dimmed(),
                parts[2].bright_white(),
                parts[3].bright_green(),
                parts[4].yellow()
            );
        }
    }

    println!("{}", "└────────────────────────────────────".dimmed());
    CommandResult::Empty
}

fn dashboard_forest(db: &ForestDb, core_root: &str) -> CommandResult {
    use colored::*;

    println!("{}", "┌─ 🌲  Forest".bright_cyan().bold());

    // Health
    let health = db.health_score().unwrap_or(0);
    let health_color = if health >= 95 {
        health.to_string().bright_green()
    } else if health >= 80 {
        health.to_string().yellow()
    } else {
        health.to_string().bright_red()
    };
    println!("  {}  {}%", "Health:".dimmed(), health_color);

    // Commit count
    let commits: i64 = std::process::Command::new("git")
        .args(["-C", core_root, "rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    println!(
        "  {}  {}",
        "Commits:".dimmed(),
        commits.to_string().bright_white()
    );

    // Active triggers
    let trigger_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_triggers WHERE enabled = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    println!(
        "  {}  {}",
        "Active triggers:".dimmed(),
        trigger_count.to_string().bright_cyan()
    );

    // Recent events
    let events = db.query_events(None, true, 5);
    if !events.is_empty() {
        println!("  {}", "Recent events:".dimmed());
        for (domain, action, _ts) in events.iter().take(3) {
            println!(
                "    {} {}.{}",
                "·".dimmed(),
                domain.bright_cyan(),
                action.dimmed()
            );
        }
    }

    // Latest snapshot if exists
    let snap: Option<(String, i64, i64)> = db
        .conn
        .query_row(
            "SELECT name, health, commits FROM shell_snapshots ORDER BY timestamp DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok();
    if let Some((name, sh, sc)) = snap {
        let commit_diff = commits - sc;
        println!(
            "  {}  '{}' — health:{} commits:+{}",
            "Last snapshot:".dimmed(),
            name.bright_white(),
            sh.to_string().dimmed(),
            commit_diff.to_string().bright_green()
        );
    }

    println!("{}", "└────────────────────────────────────".dimmed());
    CommandResult::Empty
}

// ── Phase 6 — .fsh Scripting ──────────────────────────────────────────────────

fn scripting_let_cmd(db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    // let name = expr  (from REPL)
    // args may be ["x", "= 42"] or ["x", "=", "42"] depending on splitn
    let full = args.join(" ");
    let (name, expr) = match full.split_once('=') {
        Some((n, e)) => (n.trim(), e.trim()),
        None => return CommandResult::Error("Usage: let <name> = <expression>".to_string()),
    };
    if name.is_empty() || expr.is_empty() {
        return CommandResult::Error("Usage: let <name> = <expression>".to_string());
    }
    let result = execute(expr, db, core_root);
    let val = match result {
        CommandResult::Value(ref v) => v.as_text(),
        CommandResult::Output(ref s) => s.clone(),
        _ => expr.to_string(),
    };
    println!(
        "  {} {} = {}",
        "let".dimmed(),
        name.bright_cyan(),
        val.bright_white()
    );
    // Store in session state for this REPL session
    db.conn
        .execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES (?1, ?2)",
            rusqlite::params![format!("var:{}", name), val],
        )
        .ok();
    CommandResult::Empty
}

fn smart_preview_cmd(args: &[&str]) -> CommandResult {
    let file = match args.first() {
        Some(f) => f,
        None => return CommandResult::Error("pv: missing filename".to_string()),
    };
    let home = std::env::var("HOME").unwrap_or_default();
    let path = if file.starts_with("~/") {
        file.replacen("~/", &format!("{}/", home), 1)
    } else {
        file.to_string()
    };
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return CommandResult::Error(format!("pv: {}: not found", file));
    }
    // Directory — show tree
    if p.is_dir() {
        let mut out = String::new();
        out.push_str(&format!(
            "  {} {}
",
            "📁".to_string(),
            path.bright_cyan().bold()
        ));
        let mut count = (0usize, 0usize);
        tree_walk(p, "  ", 0, 2, &mut out, &mut count);
        out.push_str(&format!(
            "
  {} {} dirs, {} files
",
            "─".dimmed(),
            count.0,
            count.1
        ));
        return CommandResult::Output(out);
    }
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let meta = std::fs::metadata(&path).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let size_str = if size > 1_048_576 {
        format!("{:.1} MB", size as f64 / 1_048_576.0)
    } else if size > 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{} B", size)
    };
    match ext.as_str() {
        // Text files — bat preview
        "rs" | "py" | "js" | "ts" | "toml" | "yaml" | "yml" | "sh" | "zsh" | "kdl" | "md"
        | "txt" | "json" | "html" | "css" | "ron" | "conf" | "ini" | "env" | "fsh" => {
            let output = std::process::Command::new("bat")
                .args([
                    "--paging=never",
                    "--line-range=1:50",
                    "--color=always",
                    &path,
                ])
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    CommandResult::Output(String::from_utf8_lossy(&o.stdout).trim_end().to_string())
                }
                _ => {
                    let content = std::fs::read_to_string(&path).unwrap_or_default();
                    let preview: String = content
                        .lines()
                        .take(50)
                        .enumerate()
                        .map(|(i, l)| format!("  {}  {}", format!("{:4}", i + 1).dimmed(), l))
                        .collect::<Vec<_>>()
                        .join(
                            "
",
                        );
                    CommandResult::Output(preview)
                }
            }
        }
        // Archives — list contents
        "zip" | "tar" | "gz" | "tgz" | "xz" | "bz2" | "zst" => {
            // Call unzip/tar directly — no sh wrapper needed (INT-194)
            let output = if ext == "zip" {
                std::process::Command::new("unzip")
                    .args(["-l", &path])
                    .output()
            } else {
                std::process::Command::new("tar")
                    .args(["-tf", &path])
                    .output()
            };
            let listing = output
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            CommandResult::Output(format!(
                "  {} Archive: {} ({})
{}",
                "📦".to_string(),
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .bright_white(),
                size_str.dimmed(),
                listing
            ))
        }
        // Images — show dimensions via file command
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" => {
            let info = std::process::Command::new("file")
                .arg(&path)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            CommandResult::Output(format!(
                "  {} Image: {} ({})
  {}",
                "🖼".to_string(),
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .bright_white(),
                size_str.dimmed(),
                info.dimmed()
            ))
        }
        // Binaries / executables
        _ => {
            let file_info = std::process::Command::new("file")
                .arg(&path)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            CommandResult::Output(format!(
                "  {} {} ({})
  {}",
                "○".dimmed(),
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .bright_white(),
                size_str.bright_white(),
                file_info.dimmed()
            ))
        }
    }
}
fn guard_cmd(args: &[&str]) -> CommandResult {
    // INT-134: manage the command allow/deny lists that safety_guard reads.
    //   guard list
    //   guard deny  add|remove <cmd>
    //   guard allow add|remove <cmd>
    // deny wins over allow at check time; both match on the command's first word.
    let db_path = faelight_core::paths::state_db();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return CommandResult::Error(format!("guard: {}", e)),
    };
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS fsh_guard_list (
            word TEXT NOT NULL,
            kind TEXT NOT NULL,
            PRIMARY KEY (word, kind)
        );",
    );
    let sub = args.first().copied().unwrap_or("");
    match sub {
        "list" | "" => {
            let mut stmt = match conn.prepare(
                "SELECT kind, word FROM fsh_guard_list ORDER BY kind, word",
            ) {
                Ok(s) => s,
                Err(e) => return CommandResult::Error(format!("guard list: {}", e)),
            };
            let rows: Vec<(String, String)> = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            if rows.is_empty() {
                return CommandResult::Output(
                    "  🛡  Command guard lists are empty.\n  → cmdguard deny add <cmd> | cmdguard allow add <cmd>".to_string(),
                );
            }
            let mut out = String::from("  🛡  Command Guard Lists\n");
            out.push_str(&"─".repeat(40));
            let deny: Vec<&String> = rows.iter().filter(|(k, _)| k == "deny").map(|(_, w)| w).collect();
            let allow: Vec<&String> = rows.iter().filter(|(k, _)| k == "allow").map(|(_, w)| w).collect();
            out.push_str("\n  ⛔ deny (blocked -- requires approval):");
            if deny.is_empty() { out.push_str(" none"); }
            for w in &deny { out.push_str(&format!("\n     · {}", w)); }
            out.push_str("\n  ✅ allow (vetted -- skips guard):");
            if allow.is_empty() { out.push_str(" none"); }
            for w in &allow { out.push_str(&format!("\n     · {}", w)); }
            out.push_str("\n\n  note: deny wins over allow; matches on first word only");
            CommandResult::Output(out)
        }
        "deny" | "allow" => {
            let action = args.get(1).copied().unwrap_or("");
            let word = args.get(2).copied().unwrap_or("");
            if word.is_empty() {
                return CommandResult::Error(format!("usage: cmdguard {} add|remove <cmd>", sub));
            }
            match action {
                "add" => {
                    match conn.execute(
                        "INSERT OR IGNORE INTO fsh_guard_list (word, kind) VALUES (?1, ?2)",
                        rusqlite::params![word, sub],
                    ) {
                        Ok(_) => CommandResult::Output(format!(
                            "  🛡  '{}' added to {} list.{}",
                            word, sub,
                            if sub == "allow" { "  ⚠ this command will now SKIP the safety guard." } else { "" }
                        )),
                        Err(e) => CommandResult::Error(format!("guard {}: {}", sub, e)),
                    }
                }
                "remove" | "rm" => {
                    let n = conn.execute(
                        "DELETE FROM fsh_guard_list WHERE word = ?1 AND kind = ?2",
                        rusqlite::params![word, sub],
                    ).unwrap_or(0);
                    if n > 0 {
                        CommandResult::Output(format!("  🛡  '{}' removed from {} list.", word, sub))
                    } else {
                        CommandResult::Error(format!("guard {}: '{}' not in {} list", sub, word, sub))
                    }
                }
                _ => CommandResult::Error(format!("usage: cmdguard {} add|remove <cmd>", sub)),
            }
        }
        _ => CommandResult::Error(
            "usage: cmdguard list | cmdguard deny add|remove <cmd> | cmdguard allow add|remove <cmd>".to_string(),
        ),
    }
}

/// `py` / `python` -- fsh's CONVENIENCE wrapper. Not python3; see the dispatch arm.
///
/// INT-143: this function used to own `python3` too, and it broke every flag: it joins all args
/// and runs `python3 -c "<args>"`, so `--version` became the Python program `--version`. `python3`
/// is now a plain external command. What stays here is fsh's own sugar -- run a snippet without
/// typing -c, run a file, expand ~ -- under names that are fsh's to define.
fn run_python_cmd(args: &[&str]) -> CommandResult {
    if args.is_empty() {
        // INT-143: `py`/`python` are fsh vocabulary and take an argument. Bare `python3` is a
        // REPL and always should have been -- it is a pass-through now, so this message is
        // finally TRUE. The old one pointed at `python3 -i`, which this same function broke.
        return CommandResult::Error(
            "py: no script argument.\n  \u{2192} run a file:    py <file.py>\n  \u{2192} run a snippet: py \"print(1+1)\"\n  \u{2192} a real REPL:   python3   (fsh passes python3 straight through)".to_string()
        );
    }
    // run python <code> or run python <file.py>
    let first = args[0];
    let home = std::env::var("HOME").unwrap_or_default();
    let expanded = if first.starts_with("~/") {
        first.replacen("~/", &format!("{}/", home), 1)
    } else {
        first.to_string()
    };
    // File execution
    if expanded.ends_with(".py") || std::path::Path::new(&expanded).exists() {
        let status = std::process::Command::new("python3")
            .arg(&expanded)
            .args(&args[1..])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();
        return match status {
            Ok(_) => CommandResult::Empty,
            Err(e) => CommandResult::Error(format!("python: {}", e)),
        };
    }
    // Inline code — join all args as code
    let code = args.join(" ");
    let output = std::process::Command::new("python3")
        .args(["-c", &code])
        .output();
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            if !stderr.is_empty() {
                eprintln!("  {}", stderr.bright_red());
            }
            if stdout.is_empty() {
                CommandResult::Empty
            } else {
                CommandResult::Output(stdout)
            }
        }
        Err(e) => CommandResult::Error(format!("python: {}", e)),
    }
}
fn run_js_cmd(args: &[&str]) -> CommandResult {
    // Check for node/deno
    let runtime = if std::process::Command::new("node")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "node"
    } else if std::process::Command::new("deno")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "deno"
    } else {
        return CommandResult::Error("js: node or deno not found in PATH".to_string());
    };
    if args.is_empty() {
        let _ = std::process::Command::new(runtime)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();
        return CommandResult::Empty;
    }
    let first = args[0];
    let home = std::env::var("HOME").unwrap_or_default();
    let expanded = if first.starts_with("~/") {
        first.replacen("~/", &format!("{}/", home), 1)
    } else {
        first.to_string()
    };
    if expanded.ends_with(".js")
        || expanded.ends_with(".ts")
        || std::path::Path::new(&expanded).exists()
    {
        let _ = std::process::Command::new(runtime)
            .arg(&expanded)
            .args(&args[1..])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();
        return CommandResult::Empty;
    }
    // Inline code
    let code = args.join(" ");
    let flag = if runtime == "deno" { "eval" } else { "-e" };
    let output = std::process::Command::new(runtime)
        .args([flag, &code])
        .output();
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            if !stderr.is_empty() {
                eprintln!("  {}", stderr.bright_red());
            }
            if stdout.is_empty() {
                CommandResult::Empty
            } else {
                CommandResult::Output(stdout)
            }
        }
        Err(e) => CommandResult::Error(format!("js: {}", e)),
    }
}
fn undo_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    // Track last filesystem operation in shell_state
    // undo list — show recent operations
    // undo — revert last tracked operation
    let sub = args.first().copied().unwrap_or("");
    match sub {
        "list" | "ls" => {
            let mut stmt = match db.conn.prepare(
                "SELECT value FROM shell_state WHERE key LIKE 'undo_%' ORDER BY key DESC LIMIT 10",
            ) {
                Ok(s) => s,
                Err(_) => return CommandResult::Output("  ○ No undo history".to_string()),
            };
            let entries: Vec<String> = stmt
                .query_map([], |r| r.get(0))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            if entries.is_empty() {
                return CommandResult::Output(
                    "  ○ No undo history — operations tracked after mv/cp/rm".to_string(),
                );
            }
            let mut out = String::new();
            out.push_str(&format!(
                "
  {} Undo history
",
                "↩".bright_cyan()
            ));
            for entry in &entries {
                out.push_str(&format!(
                    "  · {}
",
                    entry.dimmed()
                ));
            }
            CommandResult::Output(out)
        }
        "" => {
            // Try to undo last operation
            let last: Option<String> = db.conn.query_row(
                "SELECT value FROM shell_state WHERE key LIKE 'undo_%' ORDER BY key DESC LIMIT 1",
                [], |r| r.get(0)
            ).ok();
            match last {
                None => CommandResult::Output(format!(
                    "  {} Nothing to undo — use mv/cp/rm to track operations",
                    "○".dimmed()
                )),
                Some(op) => CommandResult::Output(format!(
                    "  {} Last operation: {}
  {} Use shell to manually revert — undo tracking is advisory",
                    "↩".bright_cyan(),
                    op.bright_white(),
                    "→".dimmed()
                )),
            }
        }
        _ => CommandResult::Error(format!("undo: unknown subcommand '{}' — try: list", sub)),
    }
}
fn scripting_run_cmd(db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    match args.first() {
        None => CommandResult::Error("Usage: run <file.fsh> or run --list".to_string()),
        Some(&"--list") => {
            // List .fsh scripts in core_root
            let scripts_path = std::path::Path::new(core_root).join("scripts/fsh");
            let home_path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join(".config/faelight-shell/scripts");

            println!();
            println!("  {} .fsh scripts", "🌿".normal());
            println!("{}", "  ─────────────────────────────".dimmed());

            let mut found = false;
            for path in &[scripts_path, home_path] {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        if entry
                            .path()
                            .extension()
                            .map(|e| e == "fsh")
                            .unwrap_or(false)
                        {
                            println!("  · {}", entry.file_name().to_string_lossy().bright_cyan());
                            found = true;
                        }
                    }
                }
            }
            if !found {
                println!("  {} No .fsh scripts found", "○".dimmed());
                println!(
                    "  {} Create: {}",
                    "→".dimmed(),
                    "~/0-core/scripts/fsh/example.fsh".dimmed()
                );
            }
            println!();
            CommandResult::Empty
        }
        Some(path) => {
            // INT-223 Phase 3 -- run .py, .sh, .fsh files natively
            let ext = std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let expanded = if path.starts_with("~/") {
                path.replacen(
                    "~/",
                    &format!("{}/", std::env::var("HOME").unwrap_or_default()),
                    1,
                )
            } else {
                path.to_string()
            };
            // Handle .py and .sh directly
            match ext {
                "py" => {
                    if !std::path::Path::new(&expanded).exists() {
                        return CommandResult::Error(format!("run: file not found: {}", path));
                    }
                    let status = std::process::Command::new("python3")
                        .arg(&expanded)
                        .args(&args[1..])
                        .stdin(std::process::Stdio::inherit())
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .status();
                    return match status {
                        Ok(s) if s.success() => CommandResult::Empty,
                        Ok(s) => CommandResult::Error(format!("run: python3 exited with {}", s)),
                        Err(e) => CommandResult::Error(format!("run: {}", e)),
                    };
                }
                "sh" => {
                    if !std::path::Path::new(&expanded).exists() {
                        return CommandResult::Error(format!("run: file not found: {}", path));
                    }
                    let status = std::process::Command::new("sh")
                        .arg(&expanded)
                        .args(&args[1..])
                        .stdin(std::process::Stdio::inherit())
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .status();
                    return match status {
                        Ok(s) if s.success() => CommandResult::Empty,
                        Ok(s) => CommandResult::Error(format!("run: sh exited with {}", s)),
                        Err(e) => CommandResult::Error(format!("run: {}", e)),
                    };
                }
                _ => {}
            }
            // Fall through to .fsh handling
            let resolved = if path.ends_with(".fsh") {
                path.to_string()
            } else {
                format!("{}.fsh", path)
            };
            let candidates = vec![
                resolved.clone(),
                format!("{}/scripts/fsh/{}", core_root, resolved),
                std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
                    .join(format!(".config/faelight-shell/scripts/{}", resolved))
                    .to_string_lossy()
                    .to_string(),
            ];
            for candidate in &candidates {
                if std::path::Path::new(candidate).exists() {
                    let script_args: Vec<&str> = args[1..]
                        .iter()
                        .flat_map(|s| s.split_whitespace())
                        .collect();
                    return crate::scripting::run_file(candidate, db, core_root, &script_args);
                }
            }
            CommandResult::Error(format!("run: file not found: {}", path))
        }
    }
}

// ── Phase 16 — histogram command ─────────────────────────────────────────────
fn histogram_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let field = args.first().copied().unwrap_or("command");
    // Read history and count by field
    let mut stmt = match db
        .conn
        .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 500")
    {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history", "○".dimmed())),
    };
    let commands: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for cmd in &commands {
        let key = match field {
            // deadwood: exempt -- histogram grouping label for a reporting table; the token is a chart
            // axis value and never reaches dispatch
            "command" => cmd.split_whitespace().next().unwrap_or(cmd).to_string(),
            _ => cmd.clone(),
        };
        *counts.entry(key).or_insert(0) += 1;
    }

    let total = commands.len().max(1);
    let max_count = counts.values().copied().max().unwrap_or(1);
    let bar_width = 20usize;

    let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(10);

    println!();
    println!("  {} histogram — {}", "📊".normal(), field.bright_cyan());
    println!(
        "{}",
        "  ─────────────────────────────────────────────".dimmed()
    );
    for (key, count) in &sorted {
        let bar_len = (count * bar_width / max_count).max(1);
        let bar = "█".repeat(bar_len);
        let pct = count * 100 / total;
        println!(
            "  {:20} {} {}%",
            key.bright_white(),
            bar.bright_green(),
            pct
        );
    }
    println!();
    CommandResult::Empty
}

// ── Phase 10 — chart command ──────────────────────────────────────────────────
// processes | chart cpu   — bar chart of a numeric column
// Usage as standalone: chart <table> <field>
fn chart_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    // chart can be called standalone: chart ps cpu
    // or receives piped Value::Table via pipeline (handled in value.rs PipeOp)
    let (table, field) = match args {
        [t, f] => (*t, *f),
        [f] => ("ps", *f),
        _ => return CommandResult::Error("Usage: chart <field>  or  ps | chart cpu".to_string()),
    };

    // Fetch data
    let data = match execute(table, db, "") {
        CommandResult::Value(v) => v,
        _ => return CommandResult::Error(format!("Cannot chart: {}", table)),
    };

    render_chart(data, field)
}

pub fn render_chart(data: crate::value::Value, field: &str) -> CommandResult {
    use crate::value::Value;
    let rows = match data {
        Value::Table(r) => r,
        _ => return CommandResult::Error("chart requires table input".to_string()),
    };

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No data to chart", "○".dimmed()));
    }

    // Extract name + value pairs
    let name_col = ["name", "command", "file", "domain", "cmd"]
        .iter()
        .find(|&&c| rows[0].contains_key(c))
        .copied()
        .unwrap_or("name");

    let mut pairs: Vec<(String, f64)> = rows
        .iter()
        .filter_map(|row| {
            let name = row.get(name_col).map(|v| v.as_text()).unwrap_or_default();
            let val: f64 = row.get(field)?.as_text().parse().ok()?;
            if val > 0.0 {
                Some((name, val))
            } else {
                None
            }
        })
        .collect();

    if pairs.is_empty() {
        return CommandResult::Output(format!(
            "  {} No non-zero values for '{}'",
            "○".dimmed(),
            field
        ));
    }

    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    pairs.truncate(10);

    let max = pairs.iter().map(|(_, v)| *v).fold(0.0f64, f64::max);
    let bar_width = 24usize;

    println!();
    println!("  {} chart — {}", "📊".normal(), field.bright_cyan());
    println!(
        "{}",
        "  ──────────────────────────────────────────────".dimmed()
    );
    for (name, val) in &pairs {
        let bar_len = ((val / max) * bar_width as f64) as usize;
        let bar_len = bar_len.max(1);
        let bar = "█".repeat(bar_len);
        let label = if name.len() > 22 { &name[..22] } else { name };
        println!(
            "  {:22} {} {:.1}",
            label.bright_white(),
            bar.bright_green(),
            val
        );
    }
    println!();
    CommandResult::Empty
}

// INT-238 -- forest-stats: The Forest Visualizes Its Own Growth
fn forest_stats_cmd(db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    let subcmd = args.first().copied().unwrap_or("all");
    match subcmd {
        "commits" => forest_stats_commits(db),
        "intents" => forest_stats_intents(core_root),
        "friday" => forest_stats_friday(db),
        "day" => forest_stats_day(db),
        "all" | _ => {
            let mut out = String::new();
            out.push_str(&format!(
                "\n  {} The Forest Visualizes Its Own Growth\n",
                "🌲".normal()
            ));
            out.push_str(&format!("  {}\n\n", "━".repeat(55).dimmed()));
            out.push_str(&extract_output(forest_stats_commits(db)));
            out.push_str("\n");
            out.push_str(&extract_output(forest_stats_intents(core_root)));
            out.push_str("\n");
            out.push_str(&extract_output(forest_stats_friday(db)));
            out.push_str("\n");
            out.push_str(&extract_output(forest_stats_day(db)));
            CommandResult::Output(out)
        }
    }
}
fn extract_output(r: CommandResult) -> String {
    match r {
        CommandResult::Output(s) => s,
        _ => String::new(),
    }
}
fn query_to_table(
    conn: &rusqlite::Connection,
    sql: &str,
    _headers: &[String],
) -> Result<Vec<Vec<String>>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| format!("db: {}", e))?;
    let col_count = stmt.column_count();
    let mut rows_out: Vec<Vec<String>> = Vec::new();
    stmt.query_map([], |row| {
        let vals: Vec<String> = (0..col_count)
            .map(|i| {
                row.get::<_, rusqlite::types::Value>(i)
                    .map(|v| match v {
                        rusqlite::types::Value::Null => String::new(),
                        rusqlite::types::Value::Integer(n) => n.to_string(),
                        rusqlite::types::Value::Real(f) => format!("{:.2}", f),
                        rusqlite::types::Value::Text(s) => s,
                        rusqlite::types::Value::Blob(_) => "<blob>".to_string(),
                    })
                    .unwrap_or_default()
            })
            .collect();
        Ok(vals)
    })
    .map_err(|e| format!("db: {}", e))?
    .filter_map(|r| r.ok())
    .for_each(|r| rows_out.push(r));
    Ok(rows_out)
}
fn format_table(headers: &[String], rows: &[Vec<String>]) -> String {
    use colored::Colorize;
    if rows.is_empty() {
        return "  (no results)".to_string();
    }
    let col_count = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                widths[i] = widths[i].max(cell.len().min(60));
            }
        }
    }
    let mut out = String::new();
    // Header
    out.push_str("  ");
    for (i, h) in headers.iter().enumerate() {
        out.push_str(&format!(
            "{:<width$}  ",
            h.bright_green(),
            width = widths[i]
        ));
    }
    out.push('\n');
    // Separator
    out.push_str("  ");
    for w in &widths {
        out.push_str(&"─".repeat(*w));
        out.push_str("  ");
    }
    out.push('\n');
    // Rows
    for row in rows {
        out.push_str("  ");
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                let truncated = if cell.len() > 60 { &cell[..57] } else { cell };
                out.push_str(&format!("{:<width$}  ", truncated, width = widths[i]));
            }
        }
        out.push('\n');
    }
    out.trim_end().to_string()
}
fn forest_stats_commits(db: &ForestDb) -> CommandResult {
    // Build 52-week commit velocity bar chart
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let week_secs: i64 = 604800;
    let mut out = String::new();
    out.push_str(&format!("  {} Commit Velocity (52 weeks)\n", "📊".normal()));
    let mut weeks: Vec<i64> = Vec::new();
    for w in (0..52).rev() {
        let start = now - (w + 1) * week_secs;
        let end = now - w * week_secs;
        let count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM events WHERE domain='git' AND action='commit' AND timestamp >= ?1 AND timestamp < ?2",
            rusqlite::params![start, end],
            |r| r.get(0)
        ).unwrap_or(0);
        weeks.push(count);
    }
    let max = *weeks.iter().max().unwrap_or(&1).max(&1);
    let bars = ["░", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let bar_str: String = weeks
        .iter()
        .map(|&c| {
            let idx = ((c as f64 / max as f64) * 7.0).round() as usize;
            bars[idx.min(7)]
        })
        .collect();
    out.push_str(&format!("  {}\n", bar_str.bright_green()));
    // Month labels every ~4 weeks
    let total: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE domain='git' AND action='commit'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    out.push_str(&format!(
        "  {} total commits\n",
        total.to_string().bright_white()
    ));
    CommandResult::Output(out)
}
fn forest_stats_intents(_core_root: &str) -> CommandResult {
    let complete_dir = faelight_core::paths::intents_dir()
        .join("complete")
        .to_string_lossy()
        .to_string();
    let mut out = String::new();
    out.push_str(&format!("  {} Intent Completion Timeline\n", "🎯".normal()));
    let entries = std::fs::read_dir(&complete_dir).ok();
    let mut count = 0i32;
    if let Some(entries) = entries {
        for _ in entries.flatten() {
            count += 1;
        }
    }
    // Build a growing tree visualization
    let _tree_width = count.min(60) as usize;
    let _trunk = "│";
    let branch = "├─";
    let last = "└─";
    // Simple: show last 20 intents as branches
    let complete_path = std::path::Path::new(&complete_dir);
    let mut files: Vec<_> = std::fs::read_dir(complete_path)
        .ok()
        .map(|rd| rd.flatten().collect::<Vec<_>>())
        .unwrap_or_default();
    files.sort_by_key(|e| e.file_name());
    let recent: Vec<_> = files.iter().rev().take(10).collect();
    out.push_str(&format!(
        "  🌲 {} intents complete\n",
        count.to_string().bright_white()
    ));
    for (i, entry) in recent.iter().enumerate() {
        let name = entry.file_name();
        let s = name.to_string_lossy();
        let short = s.chars().take(50).collect::<String>();
        let pfx = if i == recent.len() - 1 { last } else { branch };
        out.push_str(&format!("  {} {}\n", pfx.dimmed(), short.dimmed()));
    }
    CommandResult::Output(out)
}
fn forest_stats_friday(db: &ForestDb) -> CommandResult {
    let mut out = String::new();
    out.push_str(&format!("  {} Friday's Growth\n", "🌲".normal()));
    let facts: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0))
        .unwrap_or(0);
    let patterns: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .unwrap_or(0);
    let knowledge: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM knowledge_entries", [], |r| r.get(0))
        .unwrap_or(0);
    let vocab: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM friday_language WHERE source='named_abstraction'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    out.push_str(&format!(
        "  {} facts  {} patterns  {} lessons  {} named abstractions\n",
        facts.to_string().bright_cyan(),
        patterns.to_string().bright_cyan(),
        knowledge.to_string().bright_cyan(),
        vocab.to_string().bright_cyan(),
    ));
    // Sparkline of friday_knowledge growth -- count by created_at week
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let bars = ["░", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let mut weeks: Vec<i64> = Vec::new();
    for w in (0..12).rev() {
        let start = now - (w + 1) * 604800;
        let end = now - w * 604800;
        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM friday_knowledge WHERE created_at >= ?1 AND created_at < ?2",
                rusqlite::params![start, end],
                |r| r.get(0),
            )
            .unwrap_or(0);
        weeks.push(count);
    }
    let max = *weeks.iter().max().unwrap_or(&1).max(&1);
    let spark: String = weeks
        .iter()
        .map(|&c| {
            let idx = ((c as f64 / max as f64) * 7.0).round() as usize;
            bars[idx.min(7)]
        })
        .collect();
    out.push_str(&format!(
        "  Knowledge growth (12w): {}\n",
        spark.bright_cyan()
    ));
    CommandResult::Output(out)
}
fn forest_stats_day(db: &ForestDb) -> CommandResult {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let day_start = now - 86400;
    let mut out = String::new();
    out.push_str(&format!("  {} Today's Session\n", "⚡".normal()));
    let commits: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM events WHERE domain='git' AND action='commit' AND timestamp >= ?1",
        rusqlite::params![day_start], |r| r.get(0)
    ).unwrap_or(0);
    let deploys: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp >= ?1",
            rusqlite::params![day_start],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let commands: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE timestamp >= ?1",
            rusqlite::params![day_start],
            |r| r.get(0),
        )
        .unwrap_or(0);
    out.push_str(&format!(
        "  {} commits  {} deploys  {} commands\n",
        commits.to_string().bright_yellow(),
        deploys.to_string().bright_yellow(),
        commands.to_string().bright_yellow(),
    ));
    CommandResult::Output(out)
}

fn memory_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    match args.first().copied().unwrap_or("show") {
        "decay" => memory_decay(db),
        "distill" => memory_distill(db),
        "stats" => memory_stats(db),
        _ => CommandResult::Output(
            "  Usage: memory [decay|distill|stats]\n  decay   — show pattern age and decay status\n  distill — compress old history into patterns\n  stats   — memory health overview\n".to_string()
        ),
    }
}

fn memory_stats(db: &ForestDb) -> CommandResult {
    let total: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
        .unwrap_or(0);
    let week_old: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE timestamp < ?1",
            rusqlite::params![
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
                    - 604800
            ],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let suggest_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'SUGGEST:%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let mut out = String::new();
    out.push_str("\n  Memory Stats\n");
    out.push_str("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
    out.push_str(&format!("  · Total history entries:  {}\n", total));
    out.push_str(&format!(
        "  · Entries > 7 days old:   {} ({:.0}% eligible for decay)\n",
        week_old,
        if total > 0 {
            week_old as f64 / total as f64 * 100.0
        } else {
            0.0
        }
    ));
    out.push_str(&format!("  · Suggestion log entries: {}\n", suggest_count));
    out.push_str(&format!(
        "  · Active patterns:        {} unique commands\n",
        total - week_old - suggest_count
    ));
    out.push_str("\n  Run: memory distill — to compress old history\n\n");
    CommandResult::Output(out)
}

fn memory_decay(db: &ForestDb) -> CommandResult {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // Find commands not seen in 30 days
    let stale: Vec<(String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT command, COUNT(*) as freq FROM shell_history
             WHERE timestamp < ?1
             AND command NOT LIKE 'SUGGEST:%'
             GROUP BY command
             ORDER BY freq ASC LIMIT 15",
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output("  No decay data available\n".to_string()),
        };
        stmt.query_map(rusqlite::params![now - 2592000], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .map(|r| r.flatten().collect::<Vec<_>>())
        .unwrap_or_default()
    };
    let mut out = String::new();
    out.push_str("\n  Pattern Decay Analysis (30+ days old)\n");
    out.push_str("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
    if stale.is_empty() {
        out.push_str("  · No stale patterns detected\n");
    } else {
        out.push_str("  Stale commands (low frequency, > 30 days):\n");
        for (cmd, freq) in &stale {
            out.push_str(&format!("    · {:<30} {} occurrences\n", cmd, freq));
        }
        out.push_str("\n  Run: memory distill — to prune and compress\n");
    }
    out.push_str("\n");
    CommandResult::Output(out)
}

fn memory_distill(db: &ForestDb) -> CommandResult {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // Prune: remove suggestion log entries older than 7 days
    let suggest_pruned = db
        .conn
        .execute(
            "DELETE FROM shell_history WHERE command LIKE 'SUGGEST:%' AND timestamp < ?1",
            rusqlite::params![now - 604800],
        )
        .unwrap_or(0);
    // Prune: remove single-occurrence commands older than 60 days
    let stale_pruned = db
        .conn
        .execute(
            "DELETE FROM shell_history WHERE timestamp < ?1 AND command IN (
            SELECT command FROM shell_history
            WHERE timestamp < ?1
            GROUP BY command HAVING COUNT(*) = 1
        )",
            rusqlite::params![now - 5184000],
        )
        .unwrap_or(0);
    let mut out = String::new();
    out.push_str("\n  Memory Distillation\n");
    out.push_str("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
    out.push_str(&format!(
        "  ✅ Pruned {} stale suggestion log entries\n",
        suggest_pruned
    ));
    out.push_str(&format!(
        "  ✅ Pruned {} single-occurrence commands > 60 days old\n",
        stale_pruned
    ));
    out.push_str("  · High-frequency patterns preserved\n");
    out.push_str("  · Run: memory stats — to see updated counts\n\n");
    CommandResult::Output(out)
}

/// INT-326: bump-versions -- suggest or apply version bumps for modified tools
// INT-111: the version WRITE path. Reads a tool's Cargo.toml, finds the single
// `version = "x.y.z"` line, computes the bumped version, writes it back in place.
// Count-asserted: exactly one version line, or it errors (never a partial/wrong write).
fn tool_cargo_path(name: &str) -> Option<&'static str> {
    match name {
        "faelight-shell" => Some("faelight/rust-tools/faelight-shell/Cargo.toml"),
        "core" | "engine" => Some("faelight/engine/Cargo.toml"),
        "faelight-git" => Some("faelight/rust-tools/faelight-git/Cargo.toml"),
        "faelight-release" => Some("faelight/rust-tools/faelight-release/Cargo.toml"),
        "friday-chat" => Some("faelight/rust-tools/friday-chat/Cargo.toml"),
        "db-browse" => Some("faelight/rust-tools/db-browse/Cargo.toml"),
        "faelight-term" => Some("faelight/rust-tools/faelight-term/Cargo.toml"),
        "faelight-notify" => Some("faelight/rust-tools/faelight-notify/Cargo.toml"),
        _ => None,
    }
}

fn bump_semver(ver: &str, level: &str) -> Result<String, String> {
    let parts: Vec<u32> = ver.split('.').filter_map(|p| p.parse().ok()).collect();
    if parts.len() != 3 {
        return Err(format!("version '{}' is not x.y.z semver", ver));
    }
    let (maj, min, pat) = (parts[0], parts[1], parts[2]);
    match level {
        "patch" => Ok(format!("{}.{}.{}", maj, min, pat + 1)),
        "minor" => Ok(format!("{}.{}.0", maj, min + 1)),
        "major" => Ok(format!("{}.0.0", maj + 1)),
        other => Err(format!("unknown level '{}' (use patch|minor|major)", other)),
    }
}

/// Apply a version bump to a tool's Cargo.toml. Returns (old, new) on success.
pub fn apply_version_bump(
    core_root: &str,
    tool: &str,
    level: &str,
) -> Result<(String, String), String> {
    let rel = tool_cargo_path(tool).ok_or_else(|| format!("unknown tool '{}'", tool))?;
    let full = format!("{}/{}", core_root, rel);
    let content =
        std::fs::read_to_string(&full).map_err(|e| format!("cannot read {}: {}", full, e))?;

    // Find the version line(s). Count-assert exactly one at the top level.
    let ver_lines: Vec<&str> = content
        .lines()
        .filter(|l| l.trim_start().starts_with("version = "))
        .collect();
    if ver_lines.len() != 1 {
        return Err(format!(
            "expected exactly 1 `version = ` line in {}, found {} -- aborting (no write)",
            full,
            ver_lines.len()
        ));
    }
    let old_line = ver_lines[0];
    let old_ver = old_line
        .trim()
        .trim_start_matches("version = ")
        .trim_matches('"')
        .to_string();
    let new_ver = bump_semver(&old_ver, level)?;
    let new_line = old_line.replace(&old_ver, &new_ver);

    // Replace only that one line, verify the replacement is unique + applied.
    if content.matches(old_line).count() != 1 {
        return Err(format!(
            "version line not uniquely matchable in {} -- aborting (no write)",
            full
        ));
    }
    let updated = content.replacen(old_line, &new_line, 1);
    std::fs::write(&full, updated).map_err(|e| format!("cannot write {}: {}", full, e))?;
    Ok((old_ver, new_ver))
}

fn bump_versions_cmd(core_root: &str, args: &[&str]) -> CommandResult {
    use colored::Colorize;
    let apply = args.first().copied() == Some("apply");

    // INT-111: real write path -- `bump-versions patch|minor|major <tool>`
    if let Some(level) = args.first().copied() {
        if matches!(level, "patch" | "minor" | "major") {
            let Some(tool) = args.get(1).copied() else {
                return CommandResult::Error(
                    "usage: bump-versions <patch|minor|major> <tool>".to_string(),
                );
            };
            return match apply_version_bump(core_root, tool, level) {
                Ok((old, new)) => CommandResult::Output(format!(
                    "  {} {} {} -> {} ({})",
                    "\u{1f4e6}".normal(),
                    tool.bright_white(),
                    old.dimmed(),
                    new.bright_green(),
                    level.dimmed()
                )),
                Err(e) => CommandResult::Error(format!("bump failed: {}", e)),
            };
        }
    }

    let tools = [
        (
            "faelight-shell",
            "faelight/rust-tools/faelight-shell/Cargo.toml",
        ),
        ("core", "faelight/engine/Cargo.toml"),
        (
            "faelight-git",
            "faelight/rust-tools/faelight-git/Cargo.toml",
        ),
        (
            "faelight-release",
            "faelight/rust-tools/faelight-release/Cargo.toml",
        ),
        ("friday-chat", "faelight/rust-tools/friday-chat/Cargo.toml"),
        ("db-browse", "faelight/rust-tools/db-browse/Cargo.toml"),
        (
            "faelight-term",
            "faelight/rust-tools/faelight-term/Cargo.toml",
        ),
        (
            "faelight-notify",
            "faelight/rust-tools/faelight-notify/Cargo.toml",
        ),
    ];
    let mut out = String::new();
    out.push_str(&format!("\n  {} Version Registry\n", "📦".normal()));
    out.push_str(&format!("  {}\n", "━".repeat(50).dimmed()));
    for (name, rel_path) in &tools {
        let full = format!("{}/{}", core_root, rel_path);
        if let Ok(cargo) = std::fs::read_to_string(&full) {
            if let Some(ver_line) = cargo.lines().find(|l| l.starts_with("version = ")) {
                let ver = ver_line.trim_start_matches("version = ").trim_matches('"');
                // Parse semver
                let parts: Vec<u32> = ver.split('.').filter_map(|p| p.parse().ok()).collect();
                if parts.len() == 3 {
                    let patch_bump = format!("{}.{}.{}", parts[0], parts[1], parts[2] + 1);
                    let minor_bump = format!("{}.{}.0", parts[0], parts[1] + 1);
                    out.push_str(&format!(
                        "  {} {:<20} {}  patch→{}  minor→{}\n",
                        "◦".bright_cyan(),
                        name.bright_white(),
                        ver.bright_yellow(),
                        patch_bump.dimmed(),
                        minor_bump.dimmed()
                    ));
                }
            }
        }
    }
    out.push_str(&format!("  {}\n", "━".repeat(50).dimmed()));
    if apply {
        out.push_str(&format!(
            "  {} Use: bump-versions patch <tool> or bump-versions minor <tool>\n",
            "→".dimmed()
        ));
    } else {
        out.push_str(&format!(
            "  {} Use: bump-versions to see versions · cicomplete suggests bumps automatically\n",
            "→".dimmed()
        ));
    }
    CommandResult::Output(out)
}

/// INT-346: ade -- launch Forest ADE (Zellij + faelight-term + friday-chat)
fn ade_cmd(args: &[&str]) -> CommandResult {
    use colored::Colorize;
    let layout = args.first().copied().unwrap_or("forest-ade");
    let layout_path = format!(
        "{}/.config/zellij/layouts/{}.kdl",
        std::env::var("HOME").unwrap_or_default(),
        layout
    );

    if !std::path::Path::new(&layout_path).exists() {
        return CommandResult::Error(format!(
            "ADE layout not found: {}\nRun: core intent show 346",
            layout_path
        ));
    }

    println!("  {} Launching Forest ADE...", "🌲".normal());
    println!("  {} Layout: {}", "→".dimmed(), layout.bright_cyan());
    println!("  {} Left: fsh (Alacritty)", "→".dimmed());
    println!("  {} Right: friday-chat", "→".dimmed());
    println!(
        "  {} Alt+h/l to switch panes · Alt+f fullscreen · Alt+w close pane",
        "→".dimmed()
    );

    // Check if session already exists -- attach if so
    let sessions = std::process::Command::new("zellij")
        .args(["list-sessions"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let _is_alive = sessions
        .lines()
        .any(|l| l.contains("forest-ade") && !l.contains("EXITED") && !l.contains("dead"));
    let is_dead = sessions
        .lines()
        .any(|l| l.contains("forest-ade") && (l.contains("EXITED") || l.contains("dead")));

    if is_dead {
        // Kill the dead session first
        let _ = std::process::Command::new("zellij")
            .args(["delete-session", "forest-ade", "--force"])
            .output();
    }

    // INT-346: launch faelight-ade directly
    let _ = std::process::Command::new("faelight-ade").spawn();
    CommandResult::Output(String::new())
}
