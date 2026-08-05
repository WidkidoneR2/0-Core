#![allow(clippy::all)]
// faelight-shell v0.1.0
// Forest-native structured shell environment
// INT-120 Phase 1 — REPL skeleton
//
// "A forest deserves a shell that knows it is a forest."
// "Not text streams. Not configuration. Structured wisdom."

mod cheatsheet_tui;
mod commands;
mod db;
mod engine;
mod error;
mod exec;
mod git_tui;
mod health_tui;
mod history;
mod history_tui;
mod intent_tui;
mod output;
mod pty_exec;
mod registry;
mod safety_guard;
#[cfg(test)]
mod tests;
mod triage;
use colored::Colorize;
mod completion;
mod config;
mod digest;
mod expand;
mod jobs;
mod nl;
mod prompt;
mod schema;
mod scripting;
mod semantic;
mod session;
mod spine;
mod triggers;
mod value;
use expand::*;

use anyhow::Result;
use chrono::{Datelike, Timelike};
use rustyline::{error::ReadlineError, CompletionType, Config, EditMode, Editor};
use std::collections::VecDeque;

/// Split a line on `;` separators, respecting quoted strings.
/// "cmd1; cmd2; cmd3" → ["cmd1", "cmd2", "cmd3"]
/// INT-124: refresh the doctor health event if it's stale (older than this boot).
/// Cheap on the common path -- a timestamp compare. Only runs `core doctor run`
/// (silently) when the latest doctor event predates the current boot, so the
/// splash never shows a pre-reboot health number. Output is captured/discarded --
/// we want the event written, not the dashboard printed on login.
fn refresh_health_if_stale(core_root: &str, db: &crate::db::ForestDb) {
    let _ = core_root;
    // Latest doctor event timestamp (0 if none).
    let last_event_ts: i64 = db
        .conn
        .query_row(
            "SELECT timestamp FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // System boot time from /proc/stat (btime line).
    let boot_ts: i64 = std::fs::read_to_string("/proc/stat")
        .ok()
        .and_then(|txt| {
            txt.lines()
                .find(|l| l.starts_with("btime "))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse::<i64>().ok())
        })
        .unwrap_or(0);

    // Fresh = event recorded after this boot. Skip the doctor run (cheap path).
    if last_event_ts != 0 && last_event_ts >= boot_ts {
        return;
    }

    // Stale (or no event) -- refresh in the BACKGROUND so the prompt never blocks.
    // INT-176: this used to be .output() (spawn AND WAIT), which blocked the first
    // post-reboot prompt on the full ~696ms `core doctor run` health scan. Now we
    // spawn-and-detach: the doctor run writes its event async, the banner shows the
    // last-known health for that one launch, and the prompt renders immediately.
    // Reverses INT-124's fresh-at-splash blocking (recorded in both intents): a
    // one-launch-stale health number is invisible; a 700ms block is felt every reboot.
    let _ = std::process::Command::new("core")
        .args(["doctor", "run"])
        .env("NO_COLOR", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

fn expand_braces(s: &str) -> String {
    // Expand {N..M} and {a..z} sequences without regex
    if !s.contains('{') {
        return s.to_string();
    }
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            if let Some(close) = chars[i + 1..].iter().position(|&c| c == '}') {
                let inner: String = chars[i + 1..i + 1 + close].iter().collect();
                if let Some(dotdot) = inner.find("..") {
                    let left = &inner[..dotdot];
                    let right = &inner[dotdot + 2..];
                    if let (Ok(start_n), Ok(end_n)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        let expanded: Vec<String> = if start_n <= end_n {
                            (start_n..=end_n).map(|n| n.to_string()).collect()
                        } else {
                            (end_n..=start_n).rev().map(|n| n.to_string()).collect()
                        };
                        result.push_str(&expanded.join(" "));
                        i += close + 2;
                        continue;
                    }
                    let lc: Vec<char> = left.chars().collect();
                    let rc: Vec<char> = right.chars().collect();
                    if lc.len() == 1 && rc.len() == 1 {
                        let ls = lc[0] as u8;
                        let rs = rc[0] as u8;
                        let expanded: Vec<String> = if ls <= rs {
                            (ls..=rs).map(|c| (c as char).to_string()).collect()
                        } else {
                            (rs..=ls).rev().map(|c| (c as char).to_string()).collect()
                        };
                        result.push_str(&expanded.join(" "));
                        i += close + 2;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// INT-200: pub(crate) so the migration audit can split an entry the SAME way the REPL does.
/// A second splitter in the audit would drift from this one the first time either changed --
/// and the audit measuring a program the shell never runs is a mistake already made six times.
/// Does this line address the REPL's own job state, rather than describing a program to run?
///
/// ★ ONE OWNER FOR THE RULE. INT-169's router claims any line the spine can parse, and `jobs`
/// parses perfectly well as a command -- so it was claimed, dispatched, and died as
/// "command not found" from the moment routing became the default. The table still filled and the
/// prompt still counted, which is why it went unnoticed: job control was half-dead, not dead.
///
/// ⚠️ THE CONDITIONS ARE NOT A NAME LIST, and that is the whole subtlety. `fg commit` must reach
/// the alias, and `kill <PID>` must reach the REAL kill -- INT-095 records what happens otherwise:
/// a PID parsed as a job id made `vm down` a silent no-op and risked two VMs. So the predicate
/// mirrors Phase 8's own guards, and Phase 8 CALLS THIS rather than repeating them. Two copies of
/// one rule is the split-brain INT-193 existed to end.
pub(crate) fn is_repl_state_command(line: &str) -> bool {
    let first = commands::command_word(line);
    let second = line.split_whitespace().nth(1).unwrap_or("");
    match first.as_str() {
        "jobs" => true,
        // Job-control `fg` takes a job id or nothing; anything else is a different command.
        "fg" => second.is_empty() || second.parse::<usize>().is_ok(),
        // ONLY `kill %N` is a job-spec. Every other form belongs to the real kill.
        "kill" => second.starts_with('%'),
        _ => false,
    }
}

/// How a LINE becomes the segments the shell executes: semicolon parts, each further split on
/// `&&`/`||` -- EXCEPT where the construct is atomic.
///
/// ★ ONE OWNER, TWO CALLERS. The REPL loop and the migration audit both need this, and when they
/// each did it inline they both got it wrong the same way: `split_semicolons` deliberately keeps
/// `if …; then …; fi`, `for …; do …; done` and piped whiles WHOLE (INT-285 BUG 2, because sh must
/// receive them as one unit), and running `split_logical` over the result cut them at the `&&`
/// anyway. `if true; then echo A && echo B; fi` became two fragments and sh reported a syntax
/// error. Four days live, missed because every chain test used simple commands.
///
/// ⚠️ THE ATOMIC RULES ARE NOT REPEATED HERE. `split_semicolons_marked` reports what it protected;
/// this only decides what to do about it. Teaching `split_logical` the same rules would be a
/// second owner of a rule that already has a bug history.
pub(crate) fn split_into_segments(line: &str) -> Vec<(String, Option<bool>)> {
    split_semicolons_marked(line)
        .into_iter()
        .flat_map(|(seg, atomic)| {
            if atomic {
                // No operator: an atomic construct is one segment that always runs, and its
                // internal `&&` belongs to sh, not to the shell's own chain logic.
                vec![(seg, None)]
            } else {
                split_logical(&seg)
            }
        })
        .collect()
}

pub(crate) fn split_semicolons_marked(line: &str) -> Vec<(String, bool)> {
    // INT-285 BUG 2 FIX: for/while/until loops are atomic -- never split at semicolons
    // The entire construct is passed to sh for execution as one unit
    let trimmed = line.trim();
    let is_loop = trimmed.starts_with("for ")
        || trimmed.starts_with("while ")
        || trimmed.starts_with("until ");
    if is_loop
        && (trimmed.contains("; do")
            || trimmed.contains(";do")
            || trimmed.contains(
                "
do",
            ))
    {
        // TRUE = atomic: sh must receive this construct whole. The caller uses this to
        // skip `split_logical`, which knows nothing about `then`/`fi`/`done`.
        return vec![(trimmed.to_string(), true)];
    }
    // if/then/else/fi constructs are atomic -- never split at semicolons
    let is_if = trimmed.starts_with("if ");
    if is_if && (trimmed.contains("; then") || trimmed.contains(";then")) && trimmed.ends_with("fi")
    {
        // TRUE = atomic: sh must receive this construct whole. The caller uses this to
        // skip `split_logical`, which knows nothing about `then`/`fi`/`done`.
        return vec![(trimmed.to_string(), true)];
    }
    // piped while loops are atomic: "cmd | while ...; do ...; done"
    let has_piped_while =
        trimmed.contains("| while ") && trimmed.contains("; do") && trimmed.ends_with("done");
    if has_piped_while {
        // TRUE = atomic: sh must receive this construct whole. The caller uses this to
        // skip `split_logical`, which knows nothing about `then`/`fi`/`done`.
        return vec![(trimmed.to_string(), true)];
    }
    let mut segments = vec![];
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';
    for ch in line.chars() {
        match ch {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = ch;
                current.push(ch);
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
                current.push(ch);
            }
            ';' if !in_quote => {
                let seg = current.trim().to_string();
                if !seg.is_empty() {
                    segments.push((seg, false));
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let seg = current.trim().to_string();
    if !seg.is_empty() {
        segments.push((seg, false));
    }
    if segments.is_empty() {
        segments.push((line.trim().to_string(), false));
    }
    segments
}

/// INT-097: true if `needle` appears in `line` OUTSIDE any quotes.
/// Mirrors split_semicolons' quote tracking so operators (|||, etc.) inside
/// quoted strings (grep patterns, regexes) are NOT treated as shell operators.
fn contains_outside_quotes(line: &str, needle: &str) -> bool {
    let nbytes = needle.as_bytes();
    if nbytes.is_empty() {
        return false;
    }
    let chars: Vec<char> = line.chars().collect();
    let mut in_quote = false;
    let mut quote_char = ' ';
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if !in_quote && (ch == '"' || ch == '\'') {
            in_quote = true;
            quote_char = ch;
            i += 1;
            continue;
        }
        if in_quote && ch == quote_char {
            in_quote = false;
            i += 1;
            continue;
        }
        if !in_quote {
            // try to match needle starting at i
            let mut j = 0;
            let mut k = i;
            let mut matched = true;
            for nb in needle.chars() {
                if k >= chars.len() || chars[k] != nb {
                    matched = false;
                    break;
                }
                k += 1;
                j += 1;
            }
            let _ = nbytes;
            let _ = j;
            if matched {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Split a line on && and || operators (respecting quotes)
/// Returns Vec<(cmd, operator)> where operator is None for last cmd,
/// Some(true) for && (run next if success), Some(false) for || (run next if fail)

/// INT-267: Execute commands in parallel, return labeled output
fn run_parallel(commands: &[String]) -> bool {
    use std::sync::{Arc, Mutex};
    use std::thread;
    if commands.is_empty() {
        return true;
    }
    println!("  ∴ Running {} commands in parallel...", commands.len());
    let results: Arc<Mutex<Vec<(String, bool, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];
    for cmd in commands {
        let cmd = cmd.trim().to_string();
        if cmd.is_empty() {
            continue;
        }
        let results = Arc::clone(&results);
        let label = cmd.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
        let handle = thread::spawn(move || {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .envs(std::env::vars())
                .output();
            match output {
                Ok(o) => {
                    let success = o.status.success();
                    let mut out = String::from_utf8_lossy(&o.stdout).to_string();
                    let err = String::from_utf8_lossy(&o.stderr).to_string();
                    if !err.is_empty() {
                        out.push_str(&err);
                    }
                    results.lock().unwrap().push((label, success, out));
                }
                Err(e) => {
                    results
                        .lock()
                        .unwrap()
                        .push((label, false, format!("error: {}", e)));
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.join();
    }
    let results = results.lock().unwrap();
    let mut all_success = true;
    for (label, success, output) in results.iter() {
        let icon = if *success { "✅" } else { "❌" };
        println!("  {} [{}]", icon, label);
        for line in output.lines() {
            println!("    {}", line);
        }
        if !success {
            all_success = false;
        }
    }
    all_success
}
/// INT-267: Parse parallel { } block -- handles both multiline and single-line

/// INT-268: Natural language translation for ? prefix
/// Pattern-based translation without LLM -- forest-specific rules
fn translate_natural_language(input: &str) -> Option<(String, f64)> {
    let q = input.trim().to_lowercase();
    // Rule table: (pattern_words, command, confidence)
    let rules: &[(&[&str], &str, f64)] = &[
        // Build rules
        (&["build", "core"], "cargo build -p core", 0.95),
        (&["build", "shell"], "cargo build -p faelight-shell", 0.95),
        (&["build", "everything"], "cargo build --workspace", 0.90),
        (&["build", "workspace"], "cargo build --workspace", 0.90),
        // Deploy rules
        (&["deploy", "core"], "deploy core", 0.95),
        (&["deploy", "shell"], "deploy faelight-shell", 0.95),
        (
            &["deploy", "everything"],
            "parallel {deploy core; deploy faelight-shell; deploy faelight-term}",
            0.85,
        ),
        // Git rules
        (
            &["what", "changed", "today"],
            "git log --since=today --oneline",
            0.90,
        ),
        (
            &["show", "changed", "today"],
            "git log --since=today --oneline",
            0.90,
        ),
        (&["recent", "commits"], "git log --oneline -10", 0.88),
        (&["last", "commit"], "git log --oneline -1", 0.92),
        // Health/status
        (&["show", "health"], "d", 0.95),
        (&["check", "health"], "d", 0.95),
        (&["system", "status"], "d", 0.90),
        (&["how", "healthy"], "d", 0.88),
        // Intent
        (&["working", "on"], "intent show --active", 0.90),
        (&["active", "intent"], "intent show --active", 0.92),
        (
            &["what", "intents"],
            "intent list --status in-progress",
            0.88,
        ),
        // File operations
        (&["find", "rust", "file"], "fsearch", 0.80),
        (&["list", "files"], "list files in .", 0.85),
        (&["show", "files"], "list files in .", 0.85),
        // Friday
        (&["friday", "status"], "core friday status", 0.95),
        (&["friday", "decisions"], "core friday decisions", 0.95),
        (
            &["friday", "self", "review"],
            "core friday self-review",
            0.92,
        ),
        // Sessions
        (&["saved", "sessions"], "session list", 0.92),
        (&["show", "sessions"], "session list", 0.90),
        // Security
        (&["security", "scan"], "core security scan", 0.92),
        (&["audit"], "cargo audit", 0.88),
        // Misc
        (&["shell", "info"], "fsh", 0.90),
        (&["fsh", "info"], "fsh", 0.90),
        (&["cheatsheet"], "cheat", 0.95),
        (&["help"], "cheat", 0.88),
        (
            &["parallel", "deploy"],
            "parallel {deploy core; deploy faelight-shell}",
            0.85,
        ),
    ];
    // Score each rule by how many pattern words appear in the query
    let mut best_cmd = None;
    let mut best_score: f64 = 0.0;
    let mut best_conf: f64 = 0.0;
    for (patterns, cmd, conf) in rules {
        let matches = patterns.iter().filter(|p| q.contains(**p)).count();
        if matches == 0 {
            continue;
        }
        let score = (matches as f64 / patterns.len() as f64) * conf;
        if score > best_score {
            best_score = score;
            best_conf = *conf;
            best_cmd = Some(*cmd);
        }
    }
    if best_score >= 0.6 {
        Some((best_cmd.unwrap().to_string(), best_conf))
    } else {
        None
    }
}

/// Detect and strip redirection from a command line.
/// Returns (cleaned_line, Some((path, append))) or (line, None)

/// Expand $VAR and ${VAR} references in a line.
/// Reads from shell_vars first, then std::env.

/// INT-245 #8: token-level glob expansion within an unquoted segment.
/// Extracted from the original expand_globs body; logic unchanged for parts
/// that lack quotes.

pub(crate) fn expand_vars(
    line: &str,
    vars: &std::collections::HashMap<String, String>,
    last_exit: Option<i32>,
) -> String {
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut in_single = false;
    let mut in_double = false;
    while i < chars.len() {
        // INT-245: track quote state so single-quoted regions suppress $-expansion
        // (matching POSIX). Double-quoted regions DO expand variables. Outside any
        // quotes also expands.
        if chars[i] == '\'' && !in_double {
            in_single = !in_single;
            result.push(chars[i]);
            i += 1;
            continue;
        }
        if chars[i] == '"' && !in_single {
            in_double = !in_double;
            result.push(chars[i]);
            i += 1;
            continue;
        }
        if in_single {
            // Inside single quotes: no expansion at all, even $? and $$.
            result.push(chars[i]);
            i += 1;
            continue;
        }
        // INT-245 #12: POSIX backslash escape inside double quotes — only handle \$
        // here because expand_vars is the only thing that expands $. The other escapes
        // (\" \\ \`) are left alone for sh to handle, since pre-expanding them here
        // would cause double-parsing problems (sh would re-interpret unescaped quotes).
        if in_double && chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '$' {
            result.push('$');
            i += 2;
            continue;
        }
        if chars[i] == '$' && i + 1 < chars.len() {
            i += 1;
            // INT-245: special vars $? (exit code), $$ (pid). These are not alphanumeric
            // so they would otherwise fall into the empty-name branch and print literally.
            match chars[i] {
                '?' => {
                    let code = last_exit.unwrap_or(0);
                    result.push_str(&code.to_string());
                    i += 1;
                    continue;
                }
                '$' => {
                    result.push_str(&std::process::id().to_string());
                    i += 1;
                    continue;
                }
                _ => {}
            }
            // ${VAR} form
            if chars[i] == '{' {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '}' {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                if i < chars.len() {
                    i += 1;
                } // skip }
                let val = vars
                    .get(&name)
                    .cloned()
                    .or_else(|| std::env::var(&name).ok())
                    .unwrap_or_default();
                result.push_str(&val);
            } else {
                // $VAR form — read until non-alphanumeric/underscore
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                if name.is_empty() {
                    result.push('$');
                } else {
                    let val = vars
                        .get(&name)
                        .cloned()
                        .or_else(|| std::env::var(&name).ok())
                        .unwrap_or_default();
                    result.push_str(&val);
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

// Strip # comments — only at start of line or after whitespace, never inside strings
/// INT-249b: detect if a multi-line buffer is a complete shell command.
#[allow(dead_code)]

/// INT-245 #13: replace single- and double-quoted regions with spaces so they
/// don't contribute to keyword counting in is_complete_command. Caller uses this
/// to avoid sentences like "files for deploy" tripping the for/done balance
/// check and hanging fsh in continuation mode.

#[allow(dead_code)]
#[allow(dead_code)]

// INT-045: apply direnv environment for the current directory.
// Uses "direnv export json" -- a flat JSON object mapping env var names to
// string values (set) or null (unset). direnv keeps its own DIRENV_DIFF and
// DIRENV_WATCHES bookkeeping vars in that object; setting them like any other
// key is what lets stateful unload work when leaving a direnv directory.
fn apply_direnv() {
    let cwd = std::env::current_dir().unwrap_or_default();
    let output = std::process::Command::new("direnv")
        .args(["export", "json"])
        .current_dir(&cwd)
        .output();
    let out = match output {
        Ok(o) => o,
        Err(_) => return,
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return;
    }
    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(trimmed) {
        for (key, val) in map {
            match val {
                serde_json::Value::Null => std::env::remove_var(&key),
                serde_json::Value::String(s) => std::env::set_var(&key, s),
                _ => {}
            }
        }
    }
}

fn main() -> Result<()> {
    // INT-299: reset SIGPIPE to SIG_DFL — prevents REPL panic on broken pipe
    // ls ~/path | head -5 would previously panic with 'failed printing to stdout'
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    // Spawn REPL with 64MB stack — prevents stack overflow in deep command chains
    // INT-299: -c flag -- fsh -c "cmd" runs non-interactively and exits
    {
        let args: Vec<String> = std::env::args().collect();
        // ⚠️ ONE HANDLER, MERGED FROM TWO (2026-08-03). A second copy lived in repl_main and
        // differed in three ways that mattered: it searched the args for `-c` rather than
        // requiring position 1, it used `/bin/sh` rather than PATH, and it exited 0 on a missing
        // operand. The tolerant search is kept because `-l ... -c` is a legitimate POSIX
        // invocation -- INT-299's comment cited `exec -l '$SHELL' -c ...` from a niri session that
        // no longer exists, but removing a guard because today's config does not need it is how
        // the next person gets locked out. `/bin/sh` is kept because on NixOS it is one of only
        // two stable absolute paths.
        //
        // ⚠️⚠️ AND THIS DELEGATES TO sh, WHICH MEANS `fsh -c` IS NOT fsh: no aliases, no spine
        // router, no digit guard, no job table. That is a DESIGN QUESTION still open, not an
        // oversight -- see INT-200. It is why the conformance suite had to stop using this door.
        if let Some(c_pos) = args.iter().position(|a| a == "-c") {
            if let Some(cmd_str) = args.get(c_pos + 1) {
                let status = std::process::Command::new("/bin/sh")
                    .args(["-c", cmd_str])
                    .status()
                    .unwrap_or_else(|_| std::process::exit(1));
                std::process::exit(status.code().unwrap_or(0));
            }
            std::process::exit(0);
        }
    }
    // INT-092 Phase 3: --refresh-cheatsheet rebuilds command_registry and exits.
    // Called by the deploy script so the cheatsheet never refossilizes.
    {
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|a| a == "--refresh-cheatsheet") {
            let db_path = faelight_core::paths::state_db();
            match rusqlite::Connection::open(&db_path) {
                Ok(conn) => match cheatsheet_tui::refresh_registry(&conn) {
                    Ok(stats) => {
                        println!(
                            "  🔄 cheatsheet refreshed: {} aliases, {} builtins, {} keybinds synced",
                            stats.aliases, stats.builtins, stats.keybinds
                        );
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("  ✗ cheatsheet refresh failed: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("  ✗ could not open state.db: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    // INT-140: --triage-deploy [logfile] classifies deploy output and exits.
    // Read-only; called by the deploy script AFTER the rebuild. Never alters
    // deploy success/failure -- the deploy script keeps its own exit status.
    {
        let args: Vec<String> = std::env::args().collect();
        if let Some(pos) = args.iter().position(|a| a == "--triage-deploy") {
            let logfile = args.get(pos + 1).map(|s| s.as_str());
            let code = triage::run_triage(logfile);
            std::process::exit(code);
        }
    }
    let result = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .name("faelight-repl".into())
        .spawn(|| repl_main())?
        .join()
        .map_err(|_| anyhow::anyhow!("REPL thread panicked"))?;
    result
}

/// Everything the fsh LANGUAGE needs in order to execute correctly, and nothing an interactive
/// session wants. INT-200.
///
/// ★ THE BOUNDARY THIS ENCODES: a shell start used to mean "begin an interactive session", with no
/// other option available -- the clearest evidence being that startup changes the working directory
/// to the forest root. Correct when you are opening a terminal; fatal for `fsh -c 'pwd'`, which
/// must inherit the caller's directory. Splitting the two is what lets one binary mean one
/// language without every non-interactive invocation paying for a prompt it will never draw.
///
/// ⚠️ THIS FUNCTION PRINTS NOTHING. It returns what it did and lets the caller announce it, because
/// any non-program output from a non-interactive invocation belongs on stderr and only the front
/// end knows which front end it is.
///
/// ⚠️ NOT HERE, ON PURPOSE: the cwd change, direnv, the welcome banner, the health refresh, session
/// bookkeeping, the two startup subprocesses, the line editor, history and keybinds.
struct RuntimeInit {
    db: db::ForestDb,
    cfg: config::ShellConfig,
    applied: config::ApplyReport,
    diagnostics: Vec<String>,
}

fn runtime_init() -> Result<RuntimeInit> {
    let db = db::ForestDb::open()?;
    config::ensure_default();
    let cfg = config::load();
    // ⚠️ ORDER IS LOAD-BEARING: `apply` WRITES shell_aliases, and the command registry READS it.
    let applied = config::apply(&cfg, &db);
    // Diagnostics, not control flow -- validate reports on the runtime, it does not change it.
    let diagnostics = config::validate();
    Ok(RuntimeInit {
        db,
        cfg,
        applied,
        diagnostics,
    })
}

fn repl_main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Phase 19 — Login shell support
    // If invoked as login shell (argv[0] starts with '-' or --login flag),
    // source /etc/profile and ~/.profile to set up environment
    let is_login = args.get(0).map(|a| a.starts_with('-')).unwrap_or(false)
        || args.contains(&"--login".to_string())
        || args.contains(&"-l".to_string());
    if is_login {
        // Source /etc/profile
        if std::path::Path::new("/etc/profile").exists() {
            let _ = std::process::Command::new("sh")
                .args(["-c", "source /etc/profile 2>/dev/null || true"])
                .status();
        }
        // Source ~/.profile if it exists
        if let Ok(home) = std::env::var("HOME") {
            let profile = std::path::PathBuf::from(&home).join(".profile");
            if profile.exists() {
                let _ = std::process::Command::new("sh")
                    .args([
                        "-c",
                        &format!("source {} 2>/dev/null || true", profile.display()),
                    ])
                    .status();
            }
        }
        // Ensure NixOS paths are in PATH
        if let Ok(home) = std::env::var("HOME") {
            let cargo_bin = format!("{}/.cargo/bin", home);
            let nix_system = "/run/current-system/sw/bin".to_string();
            let nix_user = format!(
                "/etc/profiles/per-user/{}/bin",
                std::env::var("USER").unwrap_or_default()
            );
            let current_path = std::env::var("PATH").unwrap_or_default();
            if !current_path.contains(&nix_system) {
                std::env::set_var(
                    "PATH",
                    format!("{}:{}:{}:{}", nix_system, nix_user, cargo_bin, current_path),
                );
            }
        }
    }
    // INT-045: direnv hook -- apply environment for the startup directory
    apply_direnv();
    // Connect to state.db
    // INT-200: RUNTIME INIT, and it is deliberately everything the LANGUAGE needs and nothing
    // the SESSION wants. Destructured immediately so the two hundred lines below keep using the
    // same local names -- the diff stays in the init block instead of spreading through the
    // whole function.
    let RuntimeInit {
        db,
        cfg,
        applied,
        diagnostics,
    } = runtime_init()?;
    // INT-096: record which fsh build this session launched from, so `reload` can tell
    // whether a newer build was deployed. The deploy symlink canonicalizes to a store path
    // whose hash changes on every rebuild -- that hash IS the build identity (current_exe()
    // is unreliable here because the deployed binary is makeWrapper-wrapped).
    if let Ok(p) = std::fs::canonicalize("/run/current-system/sw/bin/faelight-shell") {
        let _ = std::fs::write("/tmp/fsh-running-build", p.to_string_lossy().as_bytes());
    }
    let core_root = db.core_root();
    let _ = std::env::set_current_dir(&core_root);
    // Start in ~/0-core by default
    let _ = std::env::set_current_dir(&core_root);

    // INT-201: the engine takes ownership of the resources from here down. core_root is
    // computed first because it is derived from db, which moves.
    let mut engine = crate::engine::Engine::new(db, core_root, cfg.before_rules);

    // INT-124: refresh health BEFORE the welcome header renders, so the splash
    // never shows a stale (pre-boot) health number. Cheap unless the event is stale.
    refresh_health_if_stale(engine.core_root(), engine.db());
    // Print welcome
    print_welcome(engine.core_root(), engine.db());
    // Write journal session-start entry
    let _ = std::process::Command::new("core")
        .args(["journal", "session-start"])
        .output();
    // INT-242: export forest state to /etc/faelight/ for login screen
    let _ = std::process::Command::new("faelight-export").output();
    let _session_start = std::time::Instant::now();
    let mut _session_commands: usize = 0;
    let mut _session_pipelines: usize = 0;
    // INT-246: session deduplication -- suggestions never repeated in same session
    let mut shown_friday_suggestions: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    // INT-246: once per context switch -- Friday speaks when intent changes, not every command
    let mut last_friday_intent: Option<String> = None;
    let mut _session_deploys: usize = 0;
    let mut _session_commits: usize = 0;
    let mut _session_failed: usize = 0;

    // Phase 16 — configured interactive editor
    let rl_config = Config::builder()
        .max_history_size(10000)?
        .history_ignore_dups(true)?
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .completion_show_all_if_ambiguous(true)
        .edit_mode(EditMode::Emacs)
        .build();
    // INT-201: the helper takes its OWN handle. Borrowing it from the engine here would
    // hold an immutable borrow for the entire session, since rustyline keeps the helper.
    let db_handle = engine.db_handle();
    let helper = completion::ForestHelper::new(&db_handle);
    let mut rl: Editor<completion::ForestHelper<'_>, _> = Editor::with_config(rl_config)?;
    rl.set_helper(Some(helper));
    // Ctrl+L handled in REPL loop via clear command

    // Apply config aliases and settings
    // The announcement moved OUT of `apply` (INT-200): a runtime step must not emit UI, or a
    // non-interactive caller inherits it on stdout. Printed here, unchanged, so the interactive
    // banner is byte-identical to before.
    if applied.pruned > 0 {
        println!(
            "  {} reconciled - {} runtime alias{} pruned to config.fsh",
            "✓".bright_green(),
            applied.pruned,
            if applied.pruned == 1 { "" } else { "es" }
        );
    }
    if applied.aliases > 0 || applied.settings > 0 {
        println!(
            "  {} config.fsh — {} alias{}  {} setting{}",
            "✓".bright_green(),
            applied.aliases,
            if applied.aliases == 1 { "" } else { "es" },
            applied.settings,
            if applied.settings == 1 { "" } else { "s" },
        );
    }
    // INT-233 -- validate config.fsh on load, surface errors immediately.
    //
    // ⚠️ THE INTERACTIVE FRONT END PRINTS THESE TO STDOUT, where the user is looking. A
    // non-interactive invocation must send the same diagnostics to STDERR instead: stdout is
    // program output, and a broken config must not appear in a caller's pipeline.
    if !diagnostics.is_empty() {
        println!("  {} config.fsh syntax errors:", "⚠️".normal());
        for e in &diagnostics {
            println!("{}", e);
        }
    }

    // INT-173 — build command registry on startup
    let mut registry = registry::Registry::new();
    registry.populate(engine.db(), engine.core_root());

    // Load history from state.db
    engine.db().load_history(&mut rl);
    // INT-250: bind Ctrl+R to a custom ConditionalEventHandler that sets a flag
    // and accepts the line. After readline returns, we check the flag and run TUI.
    use rustyline::{Cmd, KeyCode as RKeyCode, KeyEvent as RKeyEvent, Modifiers};
    use rustyline::{ConditionalEventHandler, Event, EventContext, EventHandler, RepeatCount};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    struct HSearchHandler {
        triggered: Arc<AtomicBool>,
    }
    impl ConditionalEventHandler for HSearchHandler {
        fn handle(
            &self,
            _evt: &Event,
            _n: RepeatCount,
            _positive: bool,
            _ctx: &EventContext,
        ) -> Option<Cmd> {
            self.triggered.store(true, Ordering::SeqCst);
            Some(Cmd::AcceptLine)
        }
    }

    let hsearch_triggered = Arc::new(AtomicBool::new(false));
    rl.bind_sequence(
        RKeyEvent(RKeyCode::Char('r'), Modifiers::CTRL),
        EventHandler::Conditional(Box::new(HSearchHandler {
            triggered: hsearch_triggered.clone(),
        })),
    );
    // INT-258: Ctrl+D opens health TUI
    struct HHealthHandler {
        triggered: Arc<AtomicBool>,
    }
    impl ConditionalEventHandler for HHealthHandler {
        fn handle(
            &self,
            _evt: &Event,
            _n: RepeatCount,
            _positive: bool,
            _ctx: &EventContext,
        ) -> Option<Cmd> {
            self.triggered.store(true, Ordering::SeqCst);
            Some(Cmd::AcceptLine)
        }
    }
    let hhealth_triggered = Arc::new(AtomicBool::new(false));
    rl.bind_sequence(
        RKeyEvent(RKeyCode::Char('d'), Modifiers::CTRL),
        EventHandler::Conditional(Box::new(HHealthHandler {
            triggered: hhealth_triggered.clone(),
        })),
    );
    // INT-253: Ctrl+G opens git TUI
    struct HGitHandler {
        triggered: Arc<AtomicBool>,
    }
    impl ConditionalEventHandler for HGitHandler {
        fn handle(
            &self,
            _evt: &Event,
            _n: RepeatCount,
            _positive: bool,
            _ctx: &EventContext,
        ) -> Option<Cmd> {
            self.triggered.store(true, Ordering::SeqCst);
            Some(Cmd::AcceptLine)
        }
    }
    let hgit_triggered = Arc::new(AtomicBool::new(false));
    rl.bind_sequence(
        RKeyEvent(RKeyCode::Char('g'), Modifiers::CTRL),
        EventHandler::Conditional(Box::new(HGitHandler {
            triggered: hgit_triggered.clone(),
        })),
    );

    // Phase 8 — job table
    // INT-201 ownership: owned by the shell session.
    // Passed into line execution as Option<&mut JobTable> when executing a command
    // line -- None for non-interactive callers, where a backgrounded job would die
    // with the process. The parameter arrives with the executor extraction.
    let mut job_table = jobs::JobTable::new();

    // Phase 17 — prompt context tracking
    #[allow(unused_assignments)]
    let mut last_duration_ms: Option<u64> = None;
    let mut last_dir: std::path::PathBuf = std::path::PathBuf::new();
    let mut last_history_id: Option<i64> = None;
    let mut last_command_start: Option<std::time::Instant> = None;

    // Phase 10 — shell variable table
    // Restore persisted variables from state.db
    {
        {
            let _ = engine.db().conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS shell_persist (key TEXT PRIMARY KEY, value TEXT NOT NULL)"
        );
            // INT-201: collect first, insert after. The prepared statement borrows the engine's
            // database and the Result temporary lives to the end of the block, so inserting
            // inside it would need &mut engine while that borrow is still outstanding.
            let persisted: Vec<(String, String)> = match engine
                .db()
                .conn
                .prepare("SELECT key, value FROM shell_persist")
            {
                Ok(mut stmt) => stmt
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
                    .unwrap_or_default(),
                Err(_) => Vec::new(),
            };
            for (k, v) in persisted {
                std::env::set_var(&k, &v);
                engine.set_var(k, v);
            }
        }
    }

    // REPL loop
    // INT-099: queue for multi-command paste blocks (run each as if typed)
    let mut pending: VecDeque<String> = VecDeque::new();
    'repl: loop {
        // INT-250: backfill completion data for the prior command.
        // Compute duration from start time captured at submit.
        let elapsed = last_command_start
            .take()
            .map(|t| t.elapsed().as_millis() as u64);
        if let Some(id) = last_history_id.take() {
            engine
                .db()
                .update_history_completion(id, engine.last_exit(), elapsed);
        }
        // INT-296: record CommandBlock to state.db
        if let Some(ref cmd) = last_history_id.as_ref().map(|_| ()).and(None::<String>) {
            let _ = cmd; // placeholder
        }
        {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let session = std::env::var("FSH_SESSION_ID").unwrap_or_else(|_| "unknown".to_string());
            let _ = engine.db().conn.execute(
                "INSERT INTO term_commands (session_id, working_dir, exit_code, duration_ms, command) \
                 SELECT ?, ?, ?, ?, command FROM shell_history WHERE command NOT LIKE 'TIMING:%' ORDER BY id DESC LIMIT 1",
                rusqlite::params![session, cwd,
                    engine.last_exit().unwrap_or(0),
                    elapsed.map(|e| e as i64).unwrap_or(0)],
            );
        }
        // INT-296: OSC 133 D -- command end with exit code
        let exit_for_osc = engine.last_exit().unwrap_or(0);
        print!("{}", prompt::osc133_command_end(exit_for_osc));
        last_duration_ms = elapsed;
        // Phase 8 — announce completed background jobs before prompt
        job_table.check_completed();

        // Phase 17 — render two-line context above input
        let ctx = prompt::PromptContext {
            last_duration_ms,
            last_exit_code: engine.last_exit(),
            job_count: job_table.job_count(),
        };
        print!("{}", prompt::OSC133_PROMPT_START);
        prompt::render_context(engine.db(), &ctx);

        // INT-045: re-evaluate direnv when the working directory changes
        if let Ok(cur_dir) = std::env::current_dir() {
            if cur_dir != last_dir {
                apply_direnv();
                last_dir = cur_dir;
            }
        }
        let prompt_str = prompt::render_line(engine.db(), engine.last_exit());

        // INT-249b: multi-line aware read - accumulates until command is complete
        // INT-099: run any queued paste-block command as if freshly typed.
        let read_result = if let Some(queued) = pending.pop_front() {
            Ok(queued)
        } else {
            {
                let mut buffer = String::new();
                let mut first = true;
                let mut heredoc_delim: Option<String> = None;
                loop {
                    let p_owned = if !first {
                        heredoc_delim
                            .as_ref()
                            .map(|delim| format!("  heredoc({})> ", delim))
                    } else {
                        None
                    };
                    let p = if first {
                        prompt_str.as_str()
                    } else if let Some(ref s) = p_owned {
                        // BUG-298-2: show delimiter so user knows what to type
                        s.as_str()
                    } else {
                        "  ... "
                    };
                    match rl.readline(p) {
                        Ok(line) => {
                            // INT-250: check Ctrl+R flag set by HSearchHandler
                            if hsearch_triggered.swap(false, Ordering::SeqCst) {
                                // line contains whatever user had typed before Ctrl+R - use as initial query
                                if let Some(selected) = history_tui::run_history_search(&line) {
                                    break Ok(selected);
                                } else {
                                    // Cancelled - return empty to skip dispatch, fresh prompt next iteration
                                    break Ok(String::new());
                                }
                            }
                            // INT-258: Ctrl+D opens health TUI
                            if hhealth_triggered.swap(false, Ordering::SeqCst) {
                                health_tui::run_health_tui(engine.core_root());
                                break Ok(String::new());
                            }
                            // INT-253: Ctrl+G opens git TUI
                            if hgit_triggered.swap(false, Ordering::SeqCst) {
                                let active =
                                    engine.db().get_focus_intent().map(|i| format!("INT-{}", i));
                                git_tui::run_git_tui(engine.core_root(), active.as_deref());
                                break Ok(String::new());
                            }
                            if !buffer.is_empty() {
                                buffer.push('\n');
                            }
                            buffer.push_str(&line);
                            let (complete, reason) = is_complete_command(&buffer);
                            if complete {
                                // INT-099: split a multi-command block; first now, rest queued.
                                let mut cmds = split_into_commands(&buffer);
                                if cmds.len() > 1 {
                                    let first = cmds.remove(0);
                                    for c in cmds.into_iter().rev() {
                                        pending.push_front(c);
                                    }
                                    break Ok(first);
                                }
                                break Ok(buffer);
                            }
                            // BUG-298-2: track heredoc delimiter for prompt
                            if reason == "unclosed heredoc" && heredoc_delim.is_none() {
                                heredoc_delim = find_heredoc_delimiter(&buffer);
                            }
                            first = false;
                        }
                        Err(e) => break Err(e),
                    }
                }
            }
        };
        match read_result {
            Ok(line) => {
                // Check reload signal at TOP of loop — before any processing
                // INT-296: OSC 133 B -- command input received
                print!("{}", prompt::OSC133_PROMPT_END);
                if std::path::Path::new("/tmp/fsh-reload-signal").exists() {
                    let _ = std::fs::remove_file("/tmp/fsh-reload-signal");
                    println!(
                        "  {} New fsh version detected — reloading...",
                        "🔄".to_string()
                    );
                    use std::os::unix::process::CommandExt;
                    // Try known deploy paths in order
                    let home = std::env::var("HOME").unwrap_or_default();
                    let candidates = vec![
                        "/run/current-system/sw/bin/faelight-shell".to_string(),
                        format!(
                            "/etc/profiles/per-user/{}/bin/faelight-shell",
                            std::env::var("USER").unwrap_or_default()
                        ),
                        format!("{}/.cargo/bin/faelight-shell", home),
                        format!("{}/0-core/scripts/faelight-shell", home),
                    ];
                    let mut exec_err = None;
                    for path in &candidates {
                        if std::path::Path::new(path).exists() {
                            exec_err = Some(std::process::Command::new(path).exec());
                            break;
                        }
                    }
                    // fallback to current_exe
                    if let Ok(exe) = std::env::current_exe() {
                        let _ = std::process::Command::new(exe).exec();
                    }
                    eprintln!("  ✗ reload failed: {:?}", exec_err);
                }
                // Strip comments before any processing
                let line = strip_comments(line.trim());
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                // !! — expand to last command before saving history
                // !<pattern> — search history for pattern and run
                let line = if line.trim() == "!!" {
                    match engine.db().get_last_command() {
                        Some(last) => {
                            println!("  {}", last.as_str());
                            last
                        }
                        None => {
                            eprintln!("  fsh: no previous command");
                            continue;
                        }
                    }
                } else if line.trim().starts_with('!')
                    && line.trim().len() > 1
                    && !line.trim().starts_with("!!")
                {
                    let pattern = &line.trim()[1..];
                    match engine.db().get_command_matching(pattern) {
                        Some(found) => {
                            println!("  {}", found.as_str());
                            found
                        }
                        None => {
                            eprintln!("  fsh: no history match for: {}", pattern);
                            continue;
                        }
                    }
                } else {
                    line
                };
                // Save to history
                let _ = rl.add_history_entry(&line);
                _session_commands += 1;
                if line.starts_with("deploy") {
                    _session_deploys += 1;
                }
                if line.starts_with("fg commit") {
                    _session_commits += 1;
                }
                if line.contains(" | ") {
                    _session_pipelines += 1;
                }
                // INT-229: abbreviation expansion
                // INT-260: cheat opens cheatsheet TUI
                // INT-092: cheat --refresh rebuilds command_registry from live sources
                if line.trim() == "cheat --refresh" {
                    let db_path = faelight_core::paths::state_db();
                    match rusqlite::Connection::open(&db_path) {
                        Ok(conn) => match cheatsheet_tui::refresh_registry(&conn) {
                            Ok(stats) => println!(
                                "  🔄 cheatsheet refreshed: {} aliases, {} builtins, {} keybinds synced",
                                stats.aliases, stats.builtins, stats.keybinds
                            ),
                            Err(e) => eprintln!("  ✗ refresh failed: {}", e),
                        },
                        Err(e) => eprintln!("  ✗ could not open state.db: {}", e),
                    }
                    continue 'repl;
                }
                if line.trim() == "cheat" {
                    cheatsheet_tui::run_cheatsheet_tui(engine.core_root());
                    continue 'repl;
                }
                // INT-254: it opens intent ledger TUI
                if line.trim() == "it" {
                    intent_tui::run_intent_tui(engine.core_root());
                    continue 'repl;
                }
                // INT-253: gt opens git TUI
                if line.trim() == "gt" {
                    let active = engine.db().get_focus_intent().map(|i| format!("INT-{}", i));
                    git_tui::run_git_tui(engine.core_root(), active.as_deref());
                    continue 'repl;
                }
                let line = match line.trim() {
                    "gc" => {
                        println!("  {} fg commit", "→".bright_cyan());
                        "fg commit".to_string()
                    }
                    "gp" => {
                        println!("  {} git push", "→".bright_cyan());
                        "git push".to_string()
                    }
                    "dep" => {
                        println!("  {} deploy", "→".bright_cyan());
                        "deploy".to_string()
                    }
                    s if s.starts_with("ds ") => {
                        let rest = &s[3..];
                        let expanded = format!("cistart {}", rest);
                        println!("  {} {}", "→".bright_cyan(), expanded);
                        expanded
                    }
                    s if s.starts_with("dc ") => {
                        let rest = &s[3..];
                        let expanded = format!("cicomplete {}", rest);
                        println!("  {} {}", "→".bright_cyan(), expanded);
                        expanded
                    }
                    _ => line,
                };
                // INT-296: OSC 133 C -- output start
                print!("{}", prompt::OSC133_OUTPUT_START);
                let line = normalize_input(&line);
                let line = normalize_input(&line);
                let line = expand_braces(&line);
                match engine.db().save_history_entry(&line) {
                    Ok(id) => { last_history_id = Some(id); last_command_start = Some(std::time::Instant::now()); }
                    Err(e) => eprintln!("warning: history save failed after retry ({}): consider running: sqlite3 ~/0-core/runtime/state.db \"PRAGMA wal_checkpoint(TRUNCATE)\"", e),
                }
                // INT-246: safety_guard -- check BEFORE any execution path
                if let Some(warning) = safety_guard::check(&line) {
                    if !safety_guard::challenge_gate(&warning) {
                        engine.set_last_exit(Some(1));
                        continue 'repl;
                    }
                }
                // INT-249b/Path-3: multi-line buffer (heredoc, control structure, backslash
                // continuation). Route through pty_exec so we get colored output AND
                // line-by-line scanning for INT-249 delimiter-leak warnings.
                if line.contains('\n') {
                    let exit = pty_exec::run_with_capture_and_scan(&line);
                    engine.set_last_exit(Some(exit));
                    match engine.db().save_history_entry(&line) {
                        Ok(id) => {
                            last_history_id = Some(id);
                        }
                        Err(e) => eprintln!("warning: failed to save history: {}", e),
                    }
                    continue 'repl;
                }
                let mut heredoc_handled = false;
                // Heredoc: detect << and delegate to sh with inherited stdin
                if line.contains(" << ") {
                    // Warn if delimiter is unquoted -- sh will expand backticks
                    // deadwood: exempt -- heredoc delimiter extraction -- for `cat << EOF` the command word is cat and the delimiter is EOF; command_word() would return the wrong object
                    let delimiter = line
                        .split(" << ")
                        .nth(1)
                        .unwrap_or("")
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim();
                    let is_quoted = delimiter.starts_with('\'') || delimiter.starts_with('"');
                    if !is_quoted && !delimiter.is_empty() {
                        println!(
                            "  {} heredoc tip: use << '{}'  to prevent backtick expansion",
                            "💡".normal(),
                            delimiter
                        );
                    }
                    // INT-249b/Path-3: run the heredoc via PTY so we get colored output
                    // AND the chance to scan each line for delimiter-leak warnings.
                    if let Some(warning) = safety_guard::check(&line) {
                        if !safety_guard::challenge_gate(&warning) {
                            engine.set_last_exit(Some(1));
                            continue 'repl;
                        }
                    }
                    let exit = pty_exec::run_with_capture_and_scan(&line);
                    engine.set_last_exit(Some(exit));
                    heredoc_handled = true;
                }
                if heredoc_handled {
                    continue 'repl;
                }
                // INT-268: natural language ? prefix
                if line.trim_start().starts_with('?') {
                    let query = line.trim_start().trim_start_matches('?').trim();
                    if query.is_empty() {
                        println!("  usage: ? <natural language query>");
                        println!("  examples: ? show health | ? what changed today | ? deploy everything");
                        continue 'repl;
                    }
                    match translate_natural_language(query) {
                        Some((cmd, confidence)) => {
                            println!(
                                "  Friday translates ({:.0}% confidence):",
                                confidence * 100.0
                            );
                            println!("    -> {}", cmd);
                            print!("  Run this? (y/N): ");
                            use std::io::Write;
                            let _ = std::io::stdout().flush();
                            let mut answer = String::new();
                            let _ = std::io::stdin().read_line(&mut answer);
                            if answer.trim().to_lowercase() == "y" {
                                let _ = engine.db().conn.execute(
                                    "INSERT INTO shell_history (command, timestamp) VALUES (?1, strftime('%s','now'))",
                                    rusqlite::params![cmd]
                                );
                                let result =
                                    commands::execute(&cmd, engine.db(), engine.core_root());
                                match result {
                                    commands::CommandResult::Output(s) => println!("{}", s),
                                    commands::CommandResult::Error(e, _) => eprintln!("  x {}", e),
                                    _ => {}
                                }
                            } else {
                                println!("  Cancelled");
                            }
                        }
                        None => {
                            println!("  Friday doesn't know how to translate: {}", query);
                            println!("  Try: core friday ask \"{}\"", query);
                        }
                    }
                    continue 'repl;
                }
                // INT-267: parallel { } block detection
                if line.trim_start().starts_with("parallel") {
                    if let Some(cmds) = parse_parallel_block(&line) {
                        run_parallel(&cmds);
                        continue 'repl;
                    }
                }
                // INT-267: ||| parallel operator
                if contains_outside_quotes(&line, "|||") {
                    let parts: Vec<String> = line
                        .split("|||")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if parts.len() >= 2 {
                        run_parallel(&parts);
                        continue 'repl;
                    }
                }
                // Phase 14 — multi-command: split on ; before execution
                // Phase 14 -- every command the shell runs is ONE entry here: semicolon
                // parts, then boolean-chain parts within each. Flattened deliberately so a chained
                // command takes the identical path a standalone one does.
                let segments: Vec<(String, Option<bool>)> = split_into_segments(&line);
                // INT-201 ownership: owned by the current command line execution.
                // Tracks &&/|| chaining state and is reset for each input line, so it is
                // deliberately NOT engine state -- it never outlives one line.
                let mut prev_op: Option<bool> = None;
                let segment_count = segments.len();
                if segment_count > 2 {
                    println!("  {} {} commands", "○".bright_cyan(), segment_count);
                }
                'segments: for (seg_idx, (segment, op)) in segments.iter().enumerate() {
                    // INT-307: restore power profile after compilation
                    if seg_idx == 0 {
                        if let Ok(prev) = engine.db().conn.query_row(
                            "SELECT value FROM shell_state WHERE key = 'power_profile_prev'",
                            [],
                            |r| r.get::<_, String>(0),
                        ) {
                            if !prev.is_empty() {
                                let _ = std::process::Command::new("powerprofilesctl")
                                    .args(["set", &prev])
                                    .status();
                                let _ = engine.db().conn.execute(
                                    "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('power_profile', ?1)",
                                    rusqlite::params![prev],
                                );
                                let _ = engine.db().conn.execute(
                                    "DELETE FROM shell_state WHERE key = 'power_profile_prev'",
                                    [],
                                );
                                eprintln!("  Friday: restored {} profile", prev);
                            }
                        }
                    }
                    let mut _children_pre: Vec<std::process::Child> = Vec::new(); // ensures children is fresh each segment iteration
                    if segment_count > 2 {
                        println!(
                            "  {} {}",
                            format!("[{}/{}]", seg_idx + 1, segment_count).dimmed(),
                            segment.dimmed()
                        );
                    }
                    // INT-169: THE SKIP DECISION -- all that remains of the boolean-chain
                    // branch that used to live here. It ran its OWN reduced dispatch, so a chained
                    // command never reached variable expansion, alias resolution, `export`, the
                    // spine router, or any of the ~1,100 lines below. `cd` had been hand-patched
                    // into it and worked; nothing else was. Splitting on `&&` now happens ABOVE the
                    // loop, so each part flows through the SAME path a standalone command does and
                    // the duplication is gone rather than relocated.
                    //
                    // The operator belongs to the part BEFORE it, so the decision uses the PREVIOUS
                    // part's operator against the previous result: `&&` runs on success, `||` on
                    // failure. `prev_op` is updated before either branch, because the tail below has
                    // its own early exits and would otherwise skip the update.
                    //
                    // ⚠️ `;` needs no special case: split_logical gives the last part of each
                    // semicolon group `None`, so the chain resets at every boundary by construction.
                    // ★ THE SINGLE SOURCE OF TRUTH FOR FLOW, inherited from INT-171 gate 5. That gate put it
                    // in CommandResult::is_failure() after bug 968c7be5, where a failure returned a non-Error
                    // variant and a scattered inline check read it as success. The method is gone -- this was
                    // its only caller -- but the RULE it existed to enforce is not: whether a command failed
                    // is decided in ONE place, and that place is now `last_exit_code`, which is also what `$?`
                    // reports and what bash consults for the same decision. Do NOT re-derive success from a
                    // result variant at another call site; that divergence IS the bug 968c7be5 was.
                    if let Some(is_and) = prev_op {
                        let succeeded = engine.last_exit().unwrap_or(0) == 0;
                        if is_and != succeeded {
                            prev_op = *op;
                            continue 'segments;
                        }
                    }
                    prev_op = *op;
                    let line = segment.as_str();
                    // INT-191: the USER BOUNDARY, captured once and never mutated. Everything
                    // below rebinds `line` -- vars, subshells, globs, then aliases -- and Rust
                    // shadowing creates NEW bindings rather than rewriting this one, so this value
                    // survives intact to the execution boundary.
                    // ⚠️ Deliberately NOT named `original_line`: that name is already taken at ~2472
                    // by a value captured AFTER all four expansions, and its drift from "original"
                    // to "original relative to what follows" is the exact confusion this records
                    // against. `raw` means exactly what the user typed, including any FOO=1 prefix,
                    // because that is what the ExecContext field it feeds is documented to hold.
                    let raw_line = segment.to_string();
                    // INT-322 Phase 4: auto-snapshot before destructive commands
                    {
                        // INT-195: canonical, quote-aware derivation. `"rmdir" /path` used to
                        // present `"rmdir`, which is absent from the destructive list below, so NO
                        // snapshot was captured. Witnessed on gen 432 against command_snapshots:
                        // the bare form produced auto-rmdir, the quoted form produced no row.
                        let snap_word = commands::command_word(line);
                        let _snap_tok = snap_word.as_str();
                        let _is_destructive = [
                            "rm",
                            "rmdir",
                            "mv",
                            "deploy",
                            "cicomplete",
                            "dc",
                            "sudo",
                            "dd",
                        ]
                        .contains(&_snap_tok)
                            || (_snap_tok == "git"
                                && (line.contains(" push") || line.contains(" reset")));
                        if _is_destructive {
                            let _iid = engine.db().get_focus_intent();
                            engine.db().capture_snapshot(line, _iid.as_deref());
                        }
                        // INT-307 Phase 2: Friday power switching on compilation
                        {
                            let _compile_tok_owned = commands::command_word(line);
                            let _compile_tok = _compile_tok_owned.as_str();
                            let _is_compile = _compile_tok == "cargo"
                                && (line.contains(" build")
                                    || line.contains(" check")
                                    || line.contains(" test")
                                    || line.contains(" nextest"));
                            if _is_compile {
                                let _prev = engine
                                    .db()
                                    .conn
                                    .query_row(
                                        "SELECT value FROM shell_state WHERE key = 'power_profile'",
                                        [],
                                        |r| r.get::<_, String>(0),
                                    )
                                    .unwrap_or_else(|_| "balanced".to_string());
                                let _ = engine.db().conn.execute(
                                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('power_profile_prev', ?1)",
                                rusqlite::params![_prev],
                            );
                                let _ = std::process::Command::new("powerprofilesctl")
                                    .args(["set", "performance"])
                                    .status();
                                eprintln!("  Friday: switching to performance for compilation");
                            }
                        }
                    }
                    // Phase 18b — Flow mode: earliest intercept
                    {
                        // INT-195: canonical, quote-aware derivation -- see the census in 195.
                        let ftok_owned = commands::command_word(line);
                        let ftok = ftok_owned.as_str();
                        if ftok == "flow" {
                            let sub = line.split_whitespace().nth(1).unwrap_or("");
                            let arg = line.split_whitespace().nth(2).unwrap_or("");
                            {
                                let fdb = engine.db();
                                match sub {
                                    "focus" => {
                                        if arg.is_empty() {
                                            println!(
                                                "  {} usage: flow focus INT-NNN",
                                                "\u{2717}".bright_red()
                                            );
                                        } else if !arg.starts_with("INT-") {
                                            println!(
                                                "  {} must be INT-NNN format",
                                                "\u{2717}".bright_red()
                                            );
                                        } else {
                                            if let Err(e) = fdb.set_focus_intent(arg) {
                                                eprintln!(
                                                    "warning: failed to set focus intent: {}",
                                                    e
                                                );
                                            }
                                            println!(
                                                "  {} focus set -> {}",
                                                "\u{1f332}".normal(),
                                                arg.bright_green().bold()
                                            );
                                        }
                                    }
                                    "clear" => {
                                        if let Err(e) = fdb.clear_focus_intent() {
                                            eprintln!(
                                                "warning: failed to clear focus intent: {}",
                                                e
                                            );
                                        }
                                        println!("  {} focus cleared", "\u{25cb}".dimmed());
                                    }
                                    "status" | "" => {
                                        match fdb.get_focus_intent() {
                                            Some(intent) => {
                                                println!();
                                                println!(
                                                    "  {} {}",
                                                    "Active focus:".dimmed(),
                                                    intent.bright_green().bold()
                                                );
                                                println!(
                                                    "  {} flow clear  to release",
                                                    "hint:".dimmed()
                                                );
                                                println!();
                                            }
                                            None => {
                                                println!("  {} no active focus -- use: flow focus INT-NNN", "\u{25cb}".dimmed());
                                            }
                                        }
                                    }
                                    _ => {
                                        println!(
                                            "  {} unknown subcommand: {}",
                                            "\u{2717}".bright_red(),
                                            sub
                                        );
                                        println!("  usage: flow | flow focus INT-NNN | flow clear");
                                    }
                                }
                            }
                            // INT-169: record the status rather than leaving the PREVIOUS command's.
                            // the flow command completed. A stale code here is invisible today, but `&&`
                            // is about to read this value to decide whether the next part runs.
                            engine.set_last_exit(Some(0));
                            continue 'segments;
                        }
                    }

                    // INT-220 Gate 11 -- friday dismiss: negative learning
                    if let Some(outcome) = engine.try_friday_dismiss(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    // INT-203 fix -- route friday subcommands to core friday
                    if let Some(outcome) = engine.try_friday_subcommand(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    // INT-220 -- friday <question>: ask Friday about the forest
                    // INT-342: db-browse -- launch state.db TUI browser
                    if let Some(outcome) = engine.try_db_browse(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    // INT-279 FQL: friday where/show/explain/recall direct queries
                    if let Some(outcome) = engine.try_friday_query(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    // INT-278 -- friday chat: launch Friday Chat TUI (intercept first)
                    if let Some(outcome) = engine.try_friday_chat(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    if let Some(outcome) = engine.try_friday_ask(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }

                    // Natural language ?prefix
                    if let Some(outcome) = engine.try_nl_query(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }

                    // Execute
                    // Phase 10 — handle let and export before anything else
                    let trimmed = line.trim();

                    // Standalone VAR=value (no command) — treat as export
                    let is_standalone_assign = trimmed.contains('=') && {
                        let eq_pos = trimmed.find('=').unwrap_or(0);
                        let before_eq = &trimmed[..eq_pos];
                        let after_eq = trimmed[eq_pos + 1..].trim();
                        let no_space_before = !before_eq.contains(' ');
                        // INT-143: `starts_with('"') && ends_with('"')` is TRUE for
                        //     QEMU_OPTS="-machine q35,smm=on" echo "$QEMU_OPTS"
                        // -- the LINE merely begins and ends with a quote. That misread an inline
                        // VAR=x cmd as a STANDALONE assignment, stored `-machine q35,smm=on" echo "`
                        // as the value, and never ran echo. Proven 2026-07-16 on the debug binary.
                        // Require the opening quote's MATCHING PARTNER to be the last character:
                        // `"a b"` is standalone; `"a b" cmd` is not.
                        let value_is_quoted = {
                            let mut cs = after_eq.chars();
                            match cs.next() {
                                Some(q @ ('"' | '\'')) => {
                                    after_eq[1..].find(q).map(|i| i + 1)
                                        == Some(after_eq.len().saturating_sub(1))
                                }
                                _ => false,
                            }
                        };
                        let no_space_after = !after_eq.contains(' ');
                        // INT-100: a value that is a complete command substitution
                        // $( ... ) counts as a single standalone value even though it
                        // contains spaces/pipes -- route it to the non-truncating
                        // standalone path (below) instead of the whitespace-splitting
                        // inline path. Balanced-paren check keeps `A=$(x) B=$(y)` out.
                        let value_is_cmdsub = {
                            let a = after_eq;
                            if a.starts_with("$(") && a.ends_with(')') {
                                let mut depth = 0i32;
                                let mut ok = true;
                                for (idx, c) in a.char_indices() {
                                    if c == '(' {
                                        depth += 1;
                                    } else if c == ')' {
                                        depth -= 1;
                                        // closes early (before end) => not a single sub
                                        if depth == 0 && idx != a.len() - 1 {
                                            ok = false;
                                            break;
                                        }
                                    }
                                }
                                ok && depth == 0
                            } else {
                                false
                            }
                        };
                        no_space_before && (no_space_after || value_is_quoted || value_is_cmdsub)
                    };
                    if is_standalone_assign {
                        let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            let name = parts[0];
                            let valid = !name.is_empty()
                                && name
                                    .chars()
                                    .next()
                                    .map(|c| c.is_ascii_alphabetic() || c == '_')
                                    .unwrap_or(false)
                                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                            if valid {
                                let val =
                                    parts[1].trim_matches('\"').trim_matches('\'').to_string();
                                // INT-100: expand $VAR first, then run $(...) subshells
                                // so `NAME=$(cmd)` stores the command OUTPUT, not the literal.
                                let expanded = expand_vars(&val, engine.vars(), engine.last_exit());
                                let expanded = expand_subshells(&expanded);
                                std::env::set_var(name, &expanded);
                                engine.set_var(name.to_string(), expanded.clone());
                                println!(
                                    "  {} {} = {}",
                                    "→".bright_cyan(),
                                    name.bright_white(),
                                    expanded.dimmed()
                                );
                                continue 'segments;
                            }
                        }
                    }
                    // Inline env var assignment: KEY=val cmd  or  KEY=val KEY2=val cmd
                    {
                        let mut temp_vars: Vec<(String, String)> = vec![];
                        let mut rest = trimmed;
                        loop {
                            // Match WORD=value at start (no spaces around =, WORD is [A-Z_][A-Z0-9_]*)
                            // INT-143 BUG A: this was `rest.split_whitespace().next()`, which does
                            // not know what a quote is. `FOO="a b" cmd` -> first token `FOO="a`,
                            // so the value was truncated to `a` and the remainder `b"` was then run
                            // AS A COMMAND ("command not found: b\"").
                            // THE INCIDENT (2026-07-15): QEMU_OPTS="-machine q35,smm=on" vm up left
                            // QEMU_OPTS="-machine" behind. An hour later the vm script's
                            // ${QEMU_OPTS:-} prepended that fragment, producing `-machine -machine
                            // q35,smm=on`. FOUR VM boots failed. The blame went to the firmware, the
                            // launcher, and the Secure Boot config in turn. None were at fault.
                            // Scan for the first token QUOTE-AWARE, the same way tokenize_args
                            // (commands/mod.rs) and tokenize (exec.rs) already do -- a space inside
                            // quotes does not end the token.
                            let maybe_var = {
                                let mut end = 0usize;
                                let mut in_quote = false;
                                let mut quote_char = ' ';
                                for (i, ch) in rest.char_indices() {
                                    match ch {
                                        '"' | '\'' if !in_quote => {
                                            in_quote = true;
                                            quote_char = ch;
                                        }
                                        c if in_quote && c == quote_char => in_quote = false,
                                        ' ' if !in_quote => break,
                                        _ => {}
                                    }
                                    end = i + ch.len_utf8();
                                }
                                &rest[..end]
                            };
                            if let Some(eq) = maybe_var.find('=') {
                                let name = &maybe_var[..eq];
                                let valid = !name.is_empty()
                                    && name
                                        .chars()
                                        .next()
                                        .map(|c| c.is_ascii_alphabetic() || c == '_')
                                        .unwrap_or(false)
                                    && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                                if valid {
                                    let val = maybe_var[eq + 1..]
                                        .trim_matches('\"')
                                        .trim_matches('\'')
                                        .to_string();
                                    // INT-100: also run $(...) subshells for inline KEY=$(x) cmd
                                    let expanded =
                                        expand_vars(&val, engine.vars(), engine.last_exit());
                                    let expanded = expand_subshells(&expanded);
                                    temp_vars.push((name.to_string(), expanded));
                                    rest = rest[maybe_var.len()..].trim_start();
                                    continue;
                                }
                            }
                            break;
                        }
                        if !temp_vars.is_empty() {
                            // INT-143 BUG B, and it is the worse of the two: fsh set these vars and
                            // NEVER UNSET THEM. `FOO=1 echo hi` left FOO=1 in the session forever.
                            // Proven 2026-07-16: `FOO143=1 echo scoping_test; echo [$FOO143]` -> [1].
                            // POSIX says VAR=x cmd scopes VAR TO THAT COMMAND. That is why `unset
                            // QEMU_OPTS` fixed the 2026-07-15 incident instantly -- the value should
                            // never have outlived the command that carried it.
                            // Bug A truncated the value; bug B made it PERMANENT. The disaster needed
                            // both. Capture the prior values here, before anything is overwritten.
                            let saved: Vec<(String, Option<String>, Option<String>)> = temp_vars
                                .iter()
                                .map(|(k, _)| {
                                    (k.clone(), std::env::var(k).ok(), engine.var(k).cloned())
                                })
                                .collect();
                            // Set vars in environment
                            for (k, v) in &temp_vars {
                                std::env::set_var(k, v);
                                engine.set_var(k.clone(), v.clone());
                            }
                            if rest.is_empty() {
                                // Standalone VAR=value — just set and confirm
                                for (k, v) in &temp_vars {
                                    println!(
                                        "  {} {} = {}",
                                        "→".bright_cyan(),
                                        k.bright_white(),
                                        v.dimmed()
                                    );
                                }
                                // INT-169: record the status rather than leaving the PREVIOUS command's.
                                // the assignment succeeded. A stale code here is invisible today, but `&&`
                                // is about to read this value to decide whether the next part runs.
                                engine.set_last_exit(Some(0));
                                continue 'segments;
                            }
                            // Vars are set BEFORE expansion so `FOO=1 echo $FOO` still prints 1.
                            let rest_expanded =
                                expand_vars(rest, engine.vars(), engine.last_exit());
                            // INT-191: `raw_line` is the whole segment INCLUDING the FOO=1
                            // prefix, because the field is documented as exactly what the user
                            // typed and the assignment is part of that.
                            let result = exec::execute_with_context(
                                &raw_line,
                                &rest_expanded,
                                engine.db(),
                                engine.core_root(),
                                engine.before_rules(),
                            );
                            // INT-143 BUG B: the command is done -- put the environment back exactly
                            // as it was. A var that did not exist before is REMOVED, not left empty;
                            // a var that had a value gets that value back. This runs whether the
                            // command succeeded or failed, because a FAILED command has even less
                            // business mutating durable state. The intent's words: "Silent state
                            // mutation on a FAILED command is indefensible."
                            // NOTE the standalone path above returns before reaching here -- a bare
                            // `FOO=1` with no command SHOULD persist. That is a different statement
                            // and it keeps its old behaviour.
                            for (k, prev_env, prev_shell) in &saved {
                                match prev_env {
                                    Some(prev) => std::env::set_var(k, prev),
                                    None => std::env::remove_var(k),
                                }
                                match prev_shell {
                                    Some(prev) => {
                                        engine.set_var(k.clone(), prev.clone());
                                    }
                                    None => {
                                        let _ = engine.remove_var(k);
                                    }
                                }
                            }
                            // INT-191: this path opened a lifecycle too -- `execute_with_context`
                            // inserts before preexec -- so it must close one. Completing BEFORE the
                            // match is safe here, unlike the REPL site: none of these arms sets
                            // `last_exit_code`, so nothing is learned by waiting. The exit code
                            // mirrors what the REPL match means for the same variants rather than
                            // reading a stale value.
                            let execution_id = result.execution_id;
                            let exec_state = exec::execution_state(&result.result);
                            let exec_code = match &result.result {
                                commands::CommandResult::Exit => None,
                                commands::CommandResult::Error(_, _) => Some(1),
                                _ => Some(0),
                            };
                            if let Err(e) = engine.db().complete_command_execution(
                                &crate::db::ExecutionCompletion {
                                    session_id: exec::session_id(),
                                    execution_id,
                                    executed_text: Some(&rest_expanded),
                                    state: exec_state,
                                    exit_code: exec_code,
                                    duration_ms: None,
                                    finished_at: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs()
                                        as i64,
                                },
                            ) {
                                eprintln!("warning: failed to close command_execution record: {e}");
                            }
                            match result.result {
                                commands::CommandResult::Exit => break 'repl,
                                commands::CommandResult::Error(e, _) => {
                                    eprintln!("  {} {}", colored::Colorize::bright_red("✗"), e);
                                }
                                commands::CommandResult::Output(out) => println!("{}", out),
                                _ => {}
                            }
                            continue 'segments;
                        }
                    }

                    if let Some(outcome) = engine.try_let(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    if let Some(outcome) = engine.try_export(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }

                    if let Some(outcome) = engine.try_unset(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    // persist VAR — save variable to state.db for cross-session persistence
                    if let Some(outcome) = engine.try_persist(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    // INT-169 DEBUG ENTRY: run one line through the SPINE path end to end --
                    // parse, lower with the real session variables, then execute with the SAME
                    // preexec/postexec hooks the text path gets.
                    //
                    // DELIBERATELY OPT-IN. The REPL still routes every normal command through
                    // the text path. Whether to route user commands through the spine is a
                    // MIGRATION decision, separate from proving the path works, and the first
                    // real flip should be a one-line routing change rather than a discovery
                    // exercise.
                    //
                    // PLACED HERE ON PURPOSE: above expand_vars, so the line arrives UNEXPANDED
                    // and the spine's own resolver actually does the work. Below it, $MY_VAR
                    // would already be a literal and the test would prove nothing -- the same
                    // way `spine parse` cannot demonstrate variable recognition.
                    //
                    // Hyphenated because `spine exec` (with a space) is caught by the builtin
                    // dispatch in commands/mod.rs, which has no session state. Two entry points,
                    // two capabilities: `spine exec` = no vars, no hooks; `spine-exec` = the
                    // full path. The former becomes redundant once the flip lands.
                    if let Some(rest) = trimmed.strip_prefix("spine-exec ") {
                        let source = rest.trim();
                        if source.is_empty() {
                            println!("  usage: spine-exec <command>");
                            continue;
                        }
                        let shell = engine.shell_context();
                        let result = exec::execute_spine_source(
                            source,
                            &shell,
                            engine.db(),
                            engine.core_root(),
                            engine.before_rules(),
                        );
                        if engine.absorb_result(result, "spine-exec")
                            == crate::engine::SegmentOutcome::ExitShell
                        {
                            break 'repl;
                        }
                        continue;
                    }
                    // Phase 10 — expand $VARS before alias resolution
                    // INT-285 BUG 2 FIX: shell control structures bypass fsh expansion
                    // for/while/until/if/case go to sh with variables unexpanded
                    // INT-195: canonical, quote-aware derivation. Bound first because a String
                    // will not match &str literal patterns inside matches!.
                    let shell_construct_word = commands::command_word(line);
                    let shell_construct = matches!(
                        shell_construct_word.as_str(),
                        "for" | "while" | "until" | "if" | "case"
                    );
                    if shell_construct {
                        let status = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(line)
                            .status();
                        engine.set_last_exit(match status {
                            Ok(s) => s.code(),
                            Err(_) => Some(1),
                        });
                        continue;
                    }
                    // BUG-298-2: heredoc — route << blocks to sh -c before
                    // alias expansion or any other processing touches the line.
                    if let Some(outcome) = engine.try_heredoc(line) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    // INT-169 blocker 6: MOVED ABOVE THE EXPANSIONS. This ran last, AFTER
                    // vars, substitutions and globs -- so an alias BODY was a separate,
                    // entirely unexpanded language fragment. Measured on the deployed shell:
                    // `alias t='echo [$HOME]'; t` printed [$HOME] literally, and the same for
                    // $(...) and *.md. The typed line was expanded before the alias was even
                    // known, and the body it produced was never expanded at all.
                    //
                    // ⚠️ Placed BELOW the two sh -c escape hatches on purpose: shell
                    // constructs and heredocs both inspect the RAW line and delegate before
                    // any processing, so moving above them would change what they catch.
                    //
                    // ⚠️ The cat_with_redirect decision travels WITH the call because the
                    // bypass mechanism IS not-expanding: leaving it below would leave nothing
                    // to bypass and would reintroduce BUG-298-4.
                    // Expand aliases before pipeline parsing
                    // INT-171 gate 2: command word is quote-aware (`"ll" foo` -> ll), so the
                    // alias lookup below resolves a quoted command instead of missing it.
                    let first_word = commands::command_word(line).to_lowercase();
                    // BUG-298-4: bypass bat alias for cat when redirect OR bat-unsupported flags
                    // Flags bat doesn't support: -A (show-all), -v, -e, -t, -n, -b
                    let cat_with_redirect = first_word == "cat" && {
                        let has_redirect = line.contains(" > ") || line.contains(" >> ");
                        let bat_unsupported = ["-A", "-v", "-e", "-t", "-n", "-b"]
                            .iter()
                            .any(|f| line.split_whitespace().any(|w| w == *f));
                        has_redirect || bat_unsupported
                    };
                    let line = if cat_with_redirect {
                        line.to_string()
                    } else {
                        commands::expand_aliases(line, engine.db())
                    };
                    let line = line.as_str();
                    // INT-169 blocker 6: THE ROUTING POINT. Placed HERE, above expand_vars,
                    // because the spine performs variable, substitution and glob expansion
                    // ITSELF -- routing below these would hand it text legacy had already
                    // transformed, making its three capabilities dead code and rebuilding the
                    // string re-inspection the spine exists to end. Aliases are already
                    // resolved above, which is what the alias-order move earlier today was for.
                    //
                    // `None` means NOT MINE: fall through with the source untouched, exactly as
                    // if routing did not exist. Legacy must receive what it would have received.
                    // INT-169: DEFAULT ON. The variable is now an ESCAPE HATCH, not an opt-in --
                    // `FSH_SPINE=0` restores legacy routing instantly, and generation rollback remains.
                    // Flipped once the evidence stopped improving from testing: 107/107 through the
                    // router, the migration audit at zero unexpected and zero feature gaps, and the
                    // counters showing the spine claims what it owns and declines what it refuses.
                    // What remains is answerable only by real use, which needs the default to be on.
                    // ⚠️ REPL-STATE COMMANDS ARE NOT ROUTABLE. `jobs`, `fg <n>` and `kill %n` read and mutate
                    // the JobTable that lives in this loop, which spine dispatch has no path to -- so the
                    // spine can PARSE them and can never RUN them. Excluded before the attempt rather than
                    // handled inside it: the router's contract is that a claim means ownership.
                    let spine_on = !is_repl_state_command(line)
                        && std::env::var("FSH_SPINE").map(|v| v != "0").unwrap_or(true);
                    let spine_trace = std::env::var_os("FSH_SPINE_TRACE").is_some();
                    // ⚠️ TWO REASONS THE SPINE IS OFF, and saying the wrong one is worse than silence: the
                    // env var is an escape hatch the user chose, while the exclusion is structural. This
                    // printed "disabled by FSH_SPINE=0" for every `jobs` and `kill %n` with the variable
                    // unset, which would send a future reader hunting an environment problem that is not
                    // there.
                    if spine_trace && !spine_on {
                        if is_repl_state_command(line) {
                            eprintln!(
                                "  [spine-router] excluded: REPL-state command -- legacy owns it"
                            );
                        } else {
                            eprintln!("  [spine-router] disabled by FSH_SPINE=0 -- legacy routing");
                        }
                    }
                    if spine_on {
                        let shell = engine.shell_context();
                        // INT-200: BACKGROUND IS TRIED FIRST, because it is the one construct whose result is
                        // not a CommandResult -- it is a live child that must be REGISTERED rather than waited
                        // on. exec.rs builds the configured Command; the job table lives here, so the handoff
                        // happens here and neither side learns the other's job.
                        //
                        // ⚠️ A redirected background line yields None and falls through to legacy untouched.
                        // Claiming it while dropping the redirect would be worse than the bug being fixed.
                        if let Some(attempt) = exec::try_spine_background_command(
                            line,
                            &shell,
                            engine.db(),
                            engine.core_root(),
                            engine.before_rules(),
                        ) {
                            match attempt {
                                Ok((command, label)) => {
                                    if spine_trace {
                                        eprintln!("  [spine-router] claimed (background): {line}");
                                    }
                                    match job_table.register(command, &label) {
                                        Ok(_) => engine.set_last_exit(Some(0)),
                                        Err(e) => {
                                            eprintln!("{} {}", "x".bright_red(), e);
                                            engine.set_last_exit(Some(1));
                                        }
                                    }
                                    continue 'segments;
                                }
                                Err(e) => {
                                    eprintln!("{} spine: {e:?}", "x".bright_red());
                                    engine.set_last_exit(Some(1));
                                    continue 'segments;
                                }
                            }
                        }
                        if let Some(result) = exec::try_execute_spine_source(
                            line,
                            &shell,
                            engine.db(),
                            engine.core_root(),
                            engine.before_rules(),
                        ) {
                            if spine_trace {
                                eprintln!("  [spine-router] claimed: {line}");
                            }
                            if engine.absorb_result(result, "spine")
                                == crate::engine::SegmentOutcome::ExitShell
                            {
                                break 'repl;
                            }
                            continue;
                        }
                        if spine_trace {
                            eprintln!("  [spine-router] declined: {line}");
                        }
                    }
                    let line = expand_vars(line, engine.vars(), engine.last_exit());
                    // Subshell expansion
                    let line = expand_subshells(&line);
                    // Glob expansion — expand *.rs, *.md etc
                    // INT-097: failglob -- if any unquoted glob matched nothing,
                    // report it clearly and skip the command (no cryptic literal-* OS error,
                    // no bogus Friday suggestion via the error path).
                    let unmatched = find_unmatched_globs(&line);
                    if !unmatched.is_empty() {
                        for pat in &unmatched {
                            println!("  no matches for pattern: {}", pat);
                        }
                        engine.set_last_exit(Some(1));
                        continue;
                    }
                    let line = expand_globs(&line);
                    let line = line.as_str();

                    // INT-265: Forest pipeline detection
                    {
                        // INT-171 gate 2: quote-aware command word for forest-pipeline detection.
                        let first = commands::command_word(line);
                        let first = first.as_str();
                        let forest_sources = [
                            "from",
                            "list",
                            "find",
                            "db",
                            "intents",
                            "deploys",
                            "friday",
                            "ps",
                            "processes",
                            "files",
                            "tools",
                            "events",
                            "deploys",
                        ];
                        let has_pipe = line.contains(" | ");
                        if forest_sources.contains(&first) && has_pipe {
                            let explain = line.contains("--explain");
                            let clean_line =
                                line.replace(" --explain", "").replace("--explain", "");
                            let clean_line = clean_line.as_str();
                            let parts: Vec<&str> = clean_line.splitn(2, " | ").collect();
                            let source_cmd = parts[0].trim();
                            let stage_text = parts.get(1).copied().unwrap_or("").to_string();
                            let pipe_rest = if parts.len() > 1 {
                                format!("_source | {}", parts[1])
                            } else {
                                "_source".to_string()
                            };
                            let source_result =
                                commands::execute(source_cmd, engine.db(), engine.core_root());
                            // INT-169: default to success, then let the Error arm below override with the
                            // REAL code. Without this the whole branch left `$?` reporting the previous
                            // command -- invisible today, load-bearing once `&&` reads it.
                            engine.set_last_exit(Some(0));
                            match source_result {
                                commands::CommandResult::Value(v) => {
                                    let source_count = match &v {
                                        value::Value::Table(rows) => rows.len(),
                                        _ => 1,
                                    };
                                    let ops = value::parse_pipeline(&pipe_rest);
                                    if explain {
                                        use colored::Colorize;
                                        let stage_labels: Vec<String> = stage_text
                                            .split(" | ")
                                            .map(|s| s.trim().to_string())
                                            .collect();
                                        let (result, stats) = value::apply_pipeline_with_stats(
                                            v,
                                            &ops,
                                            &stage_labels,
                                        );
                                        println!("{}", result.render());
                                        println!();
                                        println!("  {} pipeline explain", "─".repeat(10).dimmed());
                                        println!(
                                            "  {:<28} {} rows",
                                            "source".bright_cyan(),
                                            source_count
                                        );
                                        for stat in &stats {
                                            let slow = if stat.duration_ms > 100 {
                                                "  ⚠ slow"
                                            } else {
                                                ""
                                            };
                                            let zero = if stat.row_count == 0 {
                                                " ← zero rows!"
                                            } else {
                                                ""
                                            };
                                            println!(
                                                "  {:<28} {} rows  {}ms{}{}",
                                                stat.label.bright_cyan(),
                                                stat.row_count,
                                                stat.duration_ms,
                                                slow,
                                                zero
                                            );
                                        }
                                    } else {
                                        let result = value::apply_pipeline(v, &ops);
                                        println!("{}", result.render());
                                    }
                                }
                                commands::CommandResult::Output(out) => println!("{}", out),
                                commands::CommandResult::Error(e, code) => {
                                    eprintln!("  x {}", e);
                                    engine.set_last_exit(Some(code));
                                }
                                _ => {}
                            }
                            continue 'segments;
                        }
                    }

                    // Parse pipeline — only split on | when NOT inside quotes
                    // Helper: check if ANY pipe is outside quotes
                    let has_unquoted_pipe = || -> bool {
                        let mut inside = false;
                        let bytes = line.as_bytes();
                        for i in 0..bytes.len().saturating_sub(2) {
                            if bytes[i] == b'"' {
                                inside = !inside;
                            }
                            if !inside
                                && bytes[i] == b' '
                                && bytes[i + 1] == b'|'
                                && bytes.get(i + 2) == Some(&b' ')
                            {
                                return true;
                            }
                        }
                        false
                    };
                    let in_quotes = !has_unquoted_pipe();
                    // Save original line (with quotes) before redirect stripping
                    let original_line = line;
                    // Handle redirects natively — no sh delegation
                    let (line_stripped, redirect_info) = detect_redirect(line);
                    if let Some((ref redirect_target, is_append)) = redirect_info {
                        // Check for stderr redirect (2> or 2>&1) — use original line
                        // INT-172 RESTORATION (2026-07-17): hand `sh` the WHOLE line for any `2>`.
                        // This code used to hand-parse `2>` and it TRUNCATED THE LINE. Every arm did
                        //     cmd_part = working_line[..idx]
                        // which takes the text LEFT of the `2>` token as the entire command and throws
                        // away everything to its right -- INCLUDING THE PIPE. Measured on the deployed
                        // binary 2026-07-17 (INT-172 gate 1):
                        //     echo hello 2>/dev/null | grep -c hello   -> `hello`  (POSIX: 1)
                        //     echo X > /tmp/o.txt 2>&1                 -> terminal, NO FILE
                        //     echo hello 2>/tmp/err | grep -c hello    -> left a file named
                        //                                                 'err | grep -c hello'
                        // working_line[idx+3..] became the FILENAME. The pipeline became a path.
                        //
                        // THIS IS NOT A NEW FIX. It is a RESTORATION of nine lines deleted at 91f8f65f
                        // on 2026-04-05 ("native stderr redirect -- no more sh fallback"), which read:
                        //     // Delegate to sh for reliable redirect handling
                        //     Command::new("sh").arg("-c").arg(line).status();
                        // It was reliable. 159 lines replaced 9, and it has been silently corrupting
                        // commands ever since -- born broken, never once repaired across 103 days and
                        // two distros. `git log -S 'detect_redirect'` is the receipt. See INT-172.
                        //
                        // WHY THIS IS CORRECT: the branch below ALREADY spawns `sh -c`. fsh was never
                        // avoiding a subprocess -- it was calling sh with a MANGLED argument. sh has a
                        // real parser. Give it the untouched line and the redirect AND the pipe both
                        // work, at zero new cost. detect_redirect's `2>` arm returns the line whole,
                        // so line_stripped IS the whole line here.
                        //
                        // RELATION TO INT-171: this is neither a holding patch nor the consolidation.
                        // The `2>` handling stops PARSING and becomes a ROUTER -- one boolean saying
                        // "this line has a 2>, give it to sh whole". 171's inventory goes from five
                        // parsers to four parsers and a router. A deletion makes 171's job SMALLER.
                        //
                        // KNOWN LIMIT, recorded not hidden (INT-143's convention): fsh BUILTINS plus
                        // `2>` go to sh, and sh does not know `d` or `intl`. That is ALREADY true today
                        // -- this branch never called try_builtin -- so it is a ceiling, not a
                        // regression. Closing it means routing through try_builtin with real fd
                        // plumbing, which is INT-171's job, not this one.
                        let cmd_part = line_stripped.clone();
                        // INT-245 #10: caught by detect_redirect — no target after > or >>
                        if redirect_target == "__redirect_error_no_target__" {
                            // INT-171 gate 6: render the parse error with a caret under the
                            // offending `>` via miette, instead of a bare one-line message.
                            eprint!("{}", crate::error::render_redirect_error(line));
                            engine.set_last_exit(Some(2));
                            continue 'segments;
                        }
                        // If it's a pure stderr redirect, handle separately
                        let is_stderr_only = redirect_target == "__stderr__";
                        // Open output file
                        // For stderr-only redirects, handle without opening stdout file
                        if is_stderr_only {
                            let st = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&cmd_part)
                                .stdin(std::process::Stdio::inherit())
                                .stdout(std::process::Stdio::inherit())
                                .stderr(std::process::Stdio::inherit())
                                .envs(std::env::vars())
                                .status();
                            // INT-189: sh ran this; sh knows how it ended. Discarding the status left
                            // `$?` reporting whatever failed before it.
                            engine.set_last_exit(Some(st.ok().and_then(|s| s.code()).unwrap_or(1)));
                            continue 'segments;
                        }
                        let file = if is_append {
                            std::fs::OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(redirect_target)
                        } else {
                            std::fs::OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(redirect_target)
                        };
                        match file {
                            Ok(f) => {
                                // INT-143: ASK whether this is a builtin -- do not find out by
                                // running it. This line used to be commands::execute(), which for a
                                // NON-builtin fell through to run_external -> `sh -c <line>`, which
                                // RAN THE COMMAND. It returned Empty, `_ => None` read that as "not
                                // a builtin", and the else-branch below SPAWNED IT AGAIN.
                                // EVERY external `cmd > file` RAN TWICE. Proven 2026-07-16:
                                //     rm -rf /tmp/dirtest; mkdir /tmp/dirtest > /tmp/mk.txt
                                //     -> mkdir: cannot create directory ...: File exists
                                // The dir did not exist. Run 1 made it; run 2 failed. `curl -X POST
                                // > log` posted twice. `git push > out` pushed twice.
                                // try_builtin() answers NotBuiltin instead of spawning.
                                // INT-193 / BUG-298-4: the cat bypass means "use the REAL cat
                                // for this invocation". It used to skip only the bat alias at the
                                // prompt; the executor then re-expanded cat anyway, so the probe
                                // answered NotBuiltin and /bin/cat ran BY ACCIDENT. With a single
                                // owner the probe now sees `cat` unexpanded and matches fsh's cat
                                // BUILTIN, whose returned string picks up a trailing newline --
                                // measured 9 bytes in, 10 out. Skipping the probe here is the
                                // bypass finally saying what it always meant.
                                // ⚠️ A PIPELINE IS NOT A BUILTIN INVOCATION. `echo hi | cat > f` wrote the literal
                                // text `hi | cat` into the file: try_builtin matched `echo`, which took the rest
                                // of the line as ARGUMENTS and dutifully printed them. External-first pipelines
                                // were fine (`uname | cat > f` works) only because nothing matched here and the
                                // line fell through to `sh -c` below, where a real parser handles the pipe.
                                //
                                // It never errored -- it wrote something PLAUSIBLE -- so it corrupted output read
                                // by other commands rather than failing visibly. Live since the redirect branch
                                // was written; not a routing regression (FSH_SPINE=0 reproduces it).
                                //
                                // The detection already existed: `in_quotes` above means "no unquoted pipe", and
                                // the logical-chain branch has made the same check since INT-109. This is that
                                // check, in the one branch that lacked it.
                                let line_has_pipe = !in_quotes;
                                let builtin_result = if cat_with_redirect || line_has_pipe {
                                    commands::CommandResult::NotBuiltin
                                } else {
                                    commands::try_builtin(
                                        &cmd_part,
                                        engine.db(),
                                        engine.core_root(),
                                    )
                                };
                                let is_builtin =
                                    !matches!(builtin_result, commands::CommandResult::NotBuiltin);
                                let builtin_out = match builtin_result {
                                    commands::CommandResult::Output(o) => Some(o),
                                    commands::CommandResult::Value(v) => Some(v.render()),
                                    _ => None,
                                };
                                // INT-143: a builtin that PRINTED instead of returning (ls, d) gives
                                // Empty and no captured output. It has ALREADY RUN. Spawning an
                                // external twin here is the double-execution bug -- so stop.
                                // KNOWN LIMIT, recorded not hidden: `ls > file` still leaks its
                                // pretty listing to the terminal, because fsh's ls builtin prints
                                // directly. The file now gets nothing rather than /bin/ls's output.
                                // Fixing THAT means every print-directly builtin returning a String
                                // instead -- a refactor across 227 match arms. See INT-143's ceiling.
                                if is_builtin && builtin_out.is_none() {
                                    // INT-169: record the status rather than leaving the PREVIOUS command's.
                                    // INT-143: the builtin already ran and printed; it did not fail. A stale code here is invisible today, but `&&`
                                    // is about to read this value to decide whether the next part runs.
                                    engine.set_last_exit(Some(0));
                                    continue 'segments;
                                }
                                if let Some(out) = builtin_out {
                                    use std::io::Write;
                                    if is_append {
                                        if let Ok(mut f2) = std::fs::OpenOptions::new()
                                            .append(true)
                                            .create(true)
                                            .open(redirect_target)
                                        {
                                            let _ = f2.write_all(out.as_bytes());
                                            let _ = f2.write_all(b"\n");
                                        }
                                    } else {
                                        if let Ok(mut f2) = std::fs::OpenOptions::new()
                                            .write(true)
                                            .create(true)
                                            .truncate(true)
                                            .open(redirect_target)
                                        {
                                            let _ = f2.write_all(out.as_bytes());
                                            let _ = f2.write_all(b"\n");
                                        }
                                    }
                                } else {
                                    // INT-143: hand it to `sh -c` whole, exactly as run_external
                                    // does. This used to be:
                                    //     let parts = cmd_part.trim().splitn(2, ' ').collect();
                                    //     Command::new(parts[0]).args(parts[1].split_whitespace())
                                    // which WORD-SPLIT the arguments and kept the quote characters
                                    // as literal text. Measured 2026-07-16:
                                    //     printf 'a b' > /tmp/q1.txt   -> file contains:  'a
                                    // printf received two tokens, `'a` and `b'`, and warned about
                                    // "excess arguments". That silently mangled a .gitignore edit
                                    // earlier the same day and nobody noticed until we looked.
                                    // A shell's argument parsing is sh's job and sh is correct at
                                    // it. We already trust it in run_external -- trust it here too,
                                    // instead of maintaining a second, worse parser.
                                    if !cmd_part.trim().is_empty() {
                                        let mut cmd = std::process::Command::new("sh");
                                        cmd.arg("-c").arg(cmd_part.trim());
                                        // INT-172 (2026-07-17): the `if stderr_to_stdout` and `else if stderr_file`
                                        // branches that stood here were UNREACHABLE and are deleted. detect_redirect
                                        // returns __stderr__ for ANY line containing ` 2>` (its clause 2 precedes the
                                        // `>>` and `>` clauses), and is_stderr_only early-returns above. So this path
                                        // is only ever reached by lines with `>` and NO `2>`, where stderr_to_stdout is
                                        // always false and stderr_file always None. The 2>&1-to-file code was written
                                        // on purpose, looked correct, and could never run -- which is exactly why
                                        // `cmd > f 2>&1` wrote no file. That behavior now comes from sh, above.
                                        let _ = cmd
                                            .stdout(std::process::Stdio::from(f))
                                            .stderr(std::process::Stdio::inherit())
                                            .status();
                                    }
                                } // end else external
                            }
                            Err(e) => eprintln!("fsh: redirect error: {}", e),
                        }
                        continue 'segments;
                    }
                    let line = line_stripped.as_str();
                    let has_pipe = !in_quotes && line.contains(" | ");
                    let pipeline_ops = if has_pipe {
                        value::parse_pipeline(line)
                    } else {
                        vec![]
                    };
                    // If any pipeline op is external (e.g. head, tail, wc),
                    // pass the entire command to sh instead of handling natively
                    let has_external_op = pipeline_ops
                        .iter()
                        .any(|op| matches!(op, value::PipeOp::External(_)));
                    // Native pipe execution -- no sh fallback for external pipe chains
                    // If any pipe stage is a shell construct (while/for/if/until), pass to sh
                    let has_shell_construct = has_pipe
                        && original_line.split(" | ").skip(1).any(|stage| {
                            let s = stage.trim();
                            s.starts_with("while ")
                                || s.starts_with("for ")
                                || s.starts_with("if ")
                                || s.starts_with("until ")
                        });
                    if has_shell_construct {
                        // INT-189: sh ran this; sh knows how it ended. The status was discarded.
                        engine.set_last_exit(
                            match std::process::Command::new("sh")
                                .arg("-c")
                                .arg(segment.as_str()) // use raw unexpanded segment
                                .stdin(std::process::Stdio::inherit())
                                .stdout(std::process::Stdio::inherit())
                                .stderr(std::process::Stdio::inherit())
                                .envs(std::env::vars())
                                .status()
                            {
                                Ok(status) => Some(status.code().unwrap_or(1)),
                                Err(_) => Some(1),
                            },
                        );
                        continue 'segments;
                    }
                    if has_external_op {
                        let pipe_parts: Vec<&str> = original_line.split(" | ").collect();
                        if pipe_parts.len() >= 2 {
                            // Chain processes natively using Rust pipes
                            let mut prev_stdout: Option<std::process::ChildStdout> = None;
                            let mut children: Vec<std::process::Child> = Vec::new();
                            let mut pipe_ok = true;
                            for (idx, part) in pipe_parts.iter().enumerate() {
                                let part = part.trim();
                                // Quote-aware tokenization for pipe parts
                                let tokens: Vec<String> = {
                                    let mut toks = Vec::new();
                                    let mut cur = String::new();
                                    let mut in_q = false;
                                    let mut qc = ' ';
                                    // INT-299: track backslash escape inside double quotes
                                    // \" inside "..." must NOT close the quote
                                    let mut prev_backslash = false;
                                    for ch in part.chars() {
                                        if prev_backslash && in_q && qc == '"' {
                                            cur.push(ch);
                                            prev_backslash = false;
                                            continue;
                                        }
                                        prev_backslash = false;
                                        match ch {
                                            '\\' if in_q && qc == '"' => {
                                                prev_backslash = true;
                                                cur.push(ch);
                                            }
                                            '"' | '\'' if !in_q => {
                                                in_q = true;
                                                qc = ch;
                                            }
                                            c if in_q && c == qc => {
                                                in_q = false;
                                            }
                                            ' ' if !in_q => {
                                                if !cur.is_empty() {
                                                    toks.push(cur.clone());
                                                    cur.clear();
                                                }
                                            }
                                            c => cur.push(c),
                                        }
                                    }
                                    if !cur.is_empty() {
                                        toks.push(cur);
                                    }
                                    toks
                                };
                                let raw_cmd = match tokens.first() {
                                    Some(c) => c.clone(),
                                    None => {
                                        pipe_ok = false;
                                        break;
                                    }
                                };
                                // Expand tilde in command path
                                let expanded_cmd = if raw_cmd.starts_with("~/") {
                                    let home = std::env::var("HOME").unwrap_or_default();
                                    format!("{}/{}", home, &raw_cmd[2..])
                                } else {
                                    raw_cmd.clone()
                                };
                                let cmd_name = expanded_cmd.as_str();
                                // BUG-298-1: expand tilde in pipe-stage arguments
                                let owned_args: Vec<String> = tokens[1..]
                                    .iter()
                                    .map(|s| {
                                        if s.starts_with("~/") {
                                            let home = std::env::var("HOME").unwrap_or_default();
                                            format!("{}{}", home, &s[1..])
                                        } else if s.as_str() == "~" {
                                            std::env::var("HOME").unwrap_or_default()
                                        } else {
                                            s.clone()
                                        }
                                    })
                                    .collect();
                                let args: Vec<&str> =
                                    owned_args.iter().map(|s| s.as_str()).collect();
                                let is_last = idx == pipe_parts.len() - 1;
                                let stdin_src = match prev_stdout.take() {
                                    Some(stdout) => std::process::Stdio::from(stdout),
                                    None => std::process::Stdio::inherit(),
                                };
                                let stdout_dst = if is_last {
                                    std::process::Stdio::inherit()
                                } else {
                                    std::process::Stdio::piped()
                                };
                                // INT-249b: external-first dispatch.
                                // Vocabulary words always route to fsh builtin first (INT-266).
                                let vocab_builtins = [
                                    "write", "read", "list", "copy", "move", "delete", "find",
                                    "db", "gt", "it", "intents", "deploys", "friday",
                                ];
                                if vocab_builtins.contains(&cmd_name) && idx == 0 {
                                    let cmd_str = raw_cmd.trim().to_string();
                                    let builtin_result = commands::execute(
                                        &cmd_str,
                                        engine.db(),
                                        engine.core_root(),
                                    );
                                    // INT-169: default to success, then let the Error arm below override with the
                                    // REAL code. Without this the whole branch left `$?` reporting the previous
                                    // command -- invisible today, load-bearing once `&&` reads it.
                                    engine.set_last_exit(Some(0));
                                    match builtin_result {
                                        commands::CommandResult::Output(out) => println!("{}", out),
                                        commands::CommandResult::Error(e, code) => {
                                            engine.set_last_exit(Some(code));
                                            eprintln!("  ✗ {}", e)
                                        }
                                        commands::CommandResult::Value(v) => {
                                            if !pipeline_ops.is_empty() && !has_external_op {
                                                let result =
                                                    crate::value::apply_pipeline(v, &pipeline_ops);
                                                println!("{}", result.render());
                                            } else {
                                                println!("{}", v.render());
                                            }
                                        }
                                        _ => {}
                                    }
                                    continue 'segments;
                                }
                                // Try spawning as external process. If that fails (cmd not in PATH),
                                // try as fsh builtin via commands::execute. Never run both.
                                let spawn_result = std::process::Command::new(cmd_name)
                                    .args(&args)
                                    .stdin(stdin_src)
                                    .stdout(stdout_dst)
                                    .stderr(std::process::Stdio::inherit())
                                    .spawn();
                                match spawn_result {
                                    Ok(mut child) => {
                                        if !is_last {
                                            prev_stdout = child.stdout.take();
                                        }
                                        children.push(child);
                                    }
                                    Err(_) => {
                                        // Not on PATH -- try as fsh builtin (idx==0 only, since
                                        // builtins can't accept piped stdin from prev stage in this design)
                                        if idx == 0 && !raw_cmd.contains('/') {
                                            let builtin_line = if args.is_empty() {
                                                raw_cmd.clone()
                                            } else {
                                                format!("{} {}", raw_cmd, args.join(" "))
                                            };
                                            let builtin_out = match commands::execute(
                                                &builtin_line,
                                                engine.db(),
                                                engine.core_root(),
                                            ) {
                                                commands::CommandResult::Output(o) => Some(o),
                                                commands::CommandResult::Value(v) => {
                                                    Some(v.render())
                                                }
                                                _ => None,
                                            };
                                            if let Some(out) = builtin_out {
                                                if is_last {
                                                    println!("{}", out);
                                                } else {
                                                    let remaining =
                                                        pipe_parts[idx + 1..].join(" | ");
                                                    use std::io::Write;
                                                    let mut child =
                                                        std::process::Command::new("sh")
                                                            .arg("-c")
                                                            .arg(&remaining)
                                                            .stdin(std::process::Stdio::piped())
                                                            .stdout(std::process::Stdio::inherit())
                                                            .stderr(std::process::Stdio::inherit())
                                                            .spawn()
                                                            .ok();
                                                    if let Some(ref mut c) = child {
                                                        if let Some(ref mut stdin) = c.stdin.take()
                                                        {
                                                            let _ = stdin.write_all(out.as_bytes());
                                                        }
                                                        // INT-189: `c` runs the REMAINING pipeline, so it is the LAST stage and its
                                                        // status is the pipeline's. The earlier children are reaped below only to
                                                        // avoid zombies -- their statuses are not the result.
                                                        engine.set_last_exit(Some(
                                                            c.wait()
                                                                .ok()
                                                                .and_then(|s| s.code())
                                                                .unwrap_or(1),
                                                        ));
                                                    }
                                                    for mut child in children {
                                                        let _ = child.wait();
                                                    }
                                                    continue 'segments;
                                                }
                                            } else {
                                                eprintln!("  pipe stage '{}' failed: not found in PATH or fsh builtins", cmd_name);
                                                pipe_ok = false;
                                                break;
                                            }
                                        } else {
                                            eprintln!(
                                                "  pipe stage '{}' failed: not found",
                                                cmd_name
                                            );
                                            pipe_ok = false;
                                            break;
                                        }
                                    }
                                }
                            }
                            if pipe_ok {
                                // INT-189: `children` is pushed in pipe_parts order, so the LAST
                                // wait is the last stage -- the status a POSIX shell reports for a
                                // pipeline. Every ExitStatus here was discarded, so `echo hi | grep
                                // zzz` reported the PREVIOUS command's result. Note `pipe_ok` is not
                                // the answer: it tracks whether the pipeline could be ASSEMBLED, and
                                // stays true when grep simply finds nothing.
                                let mut final_status = None;
                                for mut child in children {
                                    final_status = child.wait().ok();
                                }
                                engine.set_last_exit(Some(
                                    final_status.and_then(|s| s.code()).unwrap_or(1),
                                ));
                                continue 'segments;
                            }
                        }
                        // Fallback to sh (INT-249)
                        // INT-189: reached when the native pipeline could not be ASSEMBLED. That
                        // failure explains why the fallback happened; it must not overwrite what the
                        // fallback actually did. The code comes from the execution that ran.
                        engine.set_last_exit(
                            match crate::db::spawn_sh_with_leak_check(original_line) {
                                Ok(status) => Some(status.code().unwrap_or(1)),
                                Err(e) => {
                                    eprintln!("fsh: pipe error: {}", e);
                                    Some(1)
                                }
                            },
                        );
                        continue 'segments;
                    }
                    let base_cmd = if has_pipe {
                        line.split(" | ").next().unwrap_or(line).to_string()
                    } else {
                        line.to_string()
                    };

                    // Phase 9 — Streaming: detect | watch at end of pipeline
                    let is_streaming = pipeline_ops
                        .last()
                        .map(|op| matches!(op, value::PipeOp::Watch { .. }))
                        .unwrap_or(false);

                    if is_streaming {
                        // Strip watch from pipeline ops
                        let stream_ops: Vec<value::PipeOp> = pipeline_ops
                            .iter()
                            .take(pipeline_ops.len() - 1)
                            .cloned()
                            .collect();
                        let interval = pipeline_ops
                            .last()
                            .and_then(|op| {
                                if let value::PipeOp::Watch { interval } = op {
                                    Some(*interval)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(2);

                        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                        // Use a background thread to watch for Enter key to stop
                        let r = running.clone();
                        std::thread::spawn(move || {
                            let mut input = String::new();
                            let _ = std::io::stdin().read_line(&mut input);
                            r.store(false, std::sync::atomic::Ordering::SeqCst);
                        });

                        println!(
                            "  {} {} {}",
                            "streaming".bright_cyan(),
                            base_cmd.dimmed(),
                            format!("({}s interval — Ctrl+C to stop)", interval).dimmed()
                        );

                        while running.load(std::sync::atomic::Ordering::SeqCst) {
                            print!("[2J[H"); // clear screen
                            let now = chrono::Local::now().format("%H:%M:%S").to_string();
                            println!(
                                "  {} {} {}",
                                "🌲 live".bright_cyan(),
                                base_cmd.dimmed(),
                                now.dimmed()
                            );
                            println!("{}", "━".repeat(52).dimmed());
                            match commands::execute(&base_cmd, engine.db(), engine.core_root()) {
                                commands::CommandResult::Value(v) => {
                                    let result = if !stream_ops.is_empty() {
                                        value::apply_pipeline(v, &stream_ops)
                                    } else {
                                        v
                                    };
                                    println!("{}", result.render());
                                }
                                commands::CommandResult::Output(out) => println!("{}", out),
                                _ => {}
                            }
                            for _ in 0..(interval * 10) {
                                if !running.load(std::sync::atomic::Ordering::SeqCst) {
                                    break;
                                }
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                        }
                        println!(
                            "
  {} stream stopped",
                            "○".dimmed()
                        );
                        // INT-169: record the status rather than leaving the PREVIOUS command's.
                        // the stream ended normally. A stale code here is invisible today, but `&&`
                        // is about to read this value to decide whether the next part runs.
                        engine.set_last_exit(Some(0));
                        continue 'segments;
                    }

                    // Phase 8 — Job control commands
                    // INT-195: canonical, quote-aware derivation.
                    if let Some(outcome) = engine.try_jobs(line, Some(&mut job_table)) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    if let Some(outcome) = engine.try_fg(line, Some(&mut job_table)) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }
                    if let Some(outcome) = engine.try_kill(line, Some(&mut job_table)) {
                        match outcome {
                            crate::engine::SegmentOutcome::Next => continue 'segments,
                            crate::engine::SegmentOutcome::ExitShell => break 'repl,
                        }
                    }

                    // Phase 8 — Background job: detect trailing &
                    let segment_trimmed = line.trim_end();
                    if segment_trimmed.ends_with(" &") || segment_trimmed == "&" {
                        let cmd_part = segment_trimmed.trim_end_matches(" &").trim();
                        if !cmd_part.is_empty() {
                            let mut parts = cmd_part.splitn(2, ' ');
                            let cmd = parts.next().unwrap_or("").to_string();
                            let args: Vec<String> = parts
                                .next()
                                .map(|a| a.split_whitespace().map(|s| s.to_string()).collect())
                                .unwrap_or_default();
                            let _ = job_table.spawn(&cmd, &args);
                        }
                        continue;
                    }

                    // Phase 13 — Redirection: already done early, use redirect_early
                    let line = line;
                    let redirect = redirect_info;
                    // Re-parse pipeline after stripping redirect
                    // Helper: check if ANY pipe is outside quotes
                    let has_unquoted_pipe2 = || -> bool {
                        let mut inside = false;
                        let bytes = line.as_bytes();
                        for i in 0..bytes.len().saturating_sub(2) {
                            if bytes[i] == b'"' {
                                inside = !inside;
                            }
                            if !inside
                                && bytes[i] == b' '
                                && bytes[i + 1] == b'|'
                                && bytes.get(i + 2) == Some(&b' ')
                            {
                                return true;
                            }
                        }
                        false
                    };
                    let in_quotes2 = !has_unquoted_pipe2();
                    let has_pipe2 = !in_quotes2 && line.contains(" | ");
                    let pipeline_ops = if has_pipe2 {
                        value::parse_pipeline(line)
                    } else {
                        pipeline_ops
                    };
                    let base_cmd = if has_pipe2 {
                        line.split(" | ").next().unwrap_or(line).to_string()
                    } else {
                        // Use redirect-stripped line as base_cmd
                        line.to_string()
                    };

                    // Resolve join ops — execute right-side tables before pipeline runs
                    let pipeline_ops: Vec<value::PipeOp> = pipeline_ops
                        .into_iter()
                        .map(|op| {
                            if let value::PipeOp::Join { table, on } = op {
                                let right_result =
                                    commands::execute(&table, engine.db(), engine.core_root());
                                if let commands::CommandResult::Value(value::Value::Table(rows)) =
                                    right_result
                                {
                                    value::PipeOp::JoinData { rows, on }
                                } else {
                                    value::PipeOp::JoinData { rows: vec![], on }
                                }
                            } else {
                                op
                            }
                        })
                        .collect();

                    // Phase 20b: inject --cwd-file for yazi/fm before execute
                    let fm_cwd_file = std::env::temp_dir().join("fsh-cwd.tmp");
                    let is_fm_cmd = {
                        // INT-195: canonical command derivation. Lowercasing is intentionally
                        // preserved until flip blocker 8 revisits normalization policy.
                        let fc = commands::command_word(&base_cmd).to_lowercase();
                        fc == "yazi" || fc == "faelight-fm"
                    };
                    let base_cmd = if is_fm_cmd {
                        format!("{} --cwd-file {}", base_cmd, fm_cwd_file.display())
                    } else {
                        base_cmd
                    };
                    // Raw shell pipe (not forest pipe ops) — run entire line via sh
                    // This prevents E_EXIT_NONZERO noise when left side of pipe fails
                    if has_pipe2 && pipeline_ops.is_empty() {
                        // INT-189: this is THE path for an ordinary shell pipeline -- `ls | wc`,
                        // `false | cat` -- and it short-circuits before the CommandResult match, so
                        // none of the arms below ever see it. `spawn_sh_with_leak_check` returns
                        // io::Result<ExitStatus>; the call site used to discard it with `let _ =`,
                        // leaving `last_exit_code` carrying the PREVIOUS command's result on the
                        // most common pipeline form in the shell.
                        engine.set_last_exit(match crate::db::spawn_sh_with_leak_check(line) {
                            Ok(status) => Some(status.code().unwrap_or(1)),
                            // sh could not be launched. Same reasoning as the pipeline arms below:
                            // leaving the code untouched recreates the stale-state bug.
                            Err(_) => Some(1),
                        });
                        continue 'segments;
                    }
                    // BUG-298-1: expand tilde in base_cmd before dispatch
                    let base_cmd = {
                        let home = std::env::var("HOME").unwrap_or_default();
                        if base_cmd.starts_with("~/") {
                            format!("{}{}", home, &base_cmd[1..])
                        } else {
                            base_cmd.replace(" ~/", &format!(" {}/", home))
                        }
                    };
                    let _cmd_timer_start = std::time::Instant::now();
                    let execution = exec::execute_with_context(
                        &raw_line,
                        &base_cmd,
                        engine.db(),
                        engine.core_root(),
                        engine.before_rules(),
                    );
                    let execution_id = execution.execution_id;
                    // INT-191: the state is derived from a BORROW, because the match below MOVES
                    // `execution.result` and the outcome would be unavailable afterwards.
                    let exec_state = exec::execution_state(&execution.result);
                    let cmd_output: Option<String> =
                        match execution.result {
                            commands::CommandResult::Exit => {
                                // INT-191: `break` escapes before the completion below, so this arm
                                // closes its own lifecycle. exit_code is None DELIBERATELY -- this arm
                                // never sets `last_exit_code`, so passing it would record the PREVIOUS
                                // command's result, which is the stale-value bug INT-189 removed.
                                // EXEC_EXIT already carries the meaning; no process exited.
                                if let Err(e) = engine.db().complete_command_execution(
                                    &crate::db::ExecutionCompletion {
                                        session_id: exec::session_id(),
                                        execution_id,
                                        executed_text: Some(&base_cmd),
                                        state: crate::db::EXEC_EXIT,
                                        exit_code: None,
                                        duration_ms: Some(
                                            _cmd_timer_start.elapsed().as_millis() as u64
                                        ),
                                        finished_at: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs()
                                            as i64,
                                    },
                                ) {
                                    eprintln!(
                                    "warning: failed to close exit command_execution record: {e}"
                                );
                                }
                                break 'repl;
                            }
                            commands::CommandResult::Value(v)
                                if !pipeline_ops.is_empty() && !has_external_op =>
                            {
                                // INT-189: `apply_pipeline` returns `Value`, not `Result`, so an
                                // in-process value pipeline cannot report failure. 0 is not a chosen
                                // policy here, it is the only coherent answer the type permits.
                                // ⚠️ If that signature ever becomes fallible, this arm must change with
                                // it -- a silent 0 over a real error would be the INT-189 bug returning.
                                let result = value::apply_pipeline(v, &pipeline_ops);
                                engine.set_last_exit(Some(0));
                                Some(result.render())
                            }
                            commands::CommandResult::Value(_) if has_external_op => {
                                // Pipeline contains external commands — pass full line to sh
                                let sh_output = std::process::Command::new("sh")
                                    .arg("-c")
                                    .arg(original_line)
                                    .output();
                                match sh_output {
                                    Ok(o) => {
                                        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                                        let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                                        if !stderr.is_empty() {
                                            eprint!("{}", stderr);
                                        }
                                        // INT-189: sh ALREADY computed this. The status was sitting
                                        // in `Output` beside stdout and stderr the whole time and was
                                        // simply never read, so `last_exit_code` carried over stale from
                                        // the previous command. Nothing is being decided here: the
                                        // semantics are whatever /bin/sh reported. `.code()` is None
                                        // only if sh ITSELF was signalled -- a signalled child already
                                        // arrives as 128+N through sh's own status.
                                        engine.set_last_exit(Some(o.status.code().unwrap_or(1)));
                                        Some(stdout)
                                    }
                                    Err(_) => {
                                        // sh could not be launched at all. Leaving the code untouched
                                        // would recreate the stale-state bug by another route.
                                        engine.set_last_exit(Some(1));
                                        None
                                    }
                                }
                            }
                            commands::CommandResult::Value(v) => {
                                // INT-189: rendering a value is success; this arm previously left
                                // `last_exit_code` carrying the previous command's result.
                                engine.set_last_exit(Some(0));
                                Some(v.render())
                            }
                            commands::CommandResult::Output(out) if !pipeline_ops.is_empty() => {
                                // External command with pipe — reconstruct full pipeline and run via sh
                                let sh_output = std::process::Command::new("sh")
                                    .arg("-c")
                                    .arg(original_line)
                                    .output();
                                match sh_output {
                                    Ok(o) => {
                                        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                                        let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                                        if !stderr.is_empty() {
                                            eprint!("{}", stderr);
                                        }
                                        // INT-189: inherit sh's status -- see the note on the Value
                                        // arm above. Same omission, same repair.
                                        engine.set_last_exit(Some(o.status.code().unwrap_or(1)));
                                        Some(stdout)
                                    }
                                    Err(_) => {
                                        engine.set_last_exit(Some(1));
                                        Some(out)
                                    }
                                }
                            }
                            commands::CommandResult::Output(out) => {
                                engine.set_last_exit(Some(0));
                                Some(out)
                            }
                            commands::CommandResult::Empty => {
                                engine.set_last_exit(Some(0));
                                None
                            }
                            commands::CommandResult::Error(e, code) => {
                                eprintln!("{} {}", colored::Colorize::bright_red("✗"), e);
                                // INT-169: the REAL status, not an assumed 1. `ls /nonexistent` exits 2 and
                                // printed "exited 2" while `$?` reported 1 -- the code was formatted into
                                // the message and thrown away. It travels on the variant now.
                                engine.set_last_exit(Some(code));
                                None
                            }
                            // INT-143: UNREACHABLE BY CONSTRUCTION, not by luck. This match is fed by
                            // exec::execute_with_context, which dispatches through commands::execute
                            // (exec.rs:554), and execute() always passes allow_external: true -- so the
                            // NotBuiltin arm in execute_impl cannot fire on this path. Only
                            // try_builtin() can produce this variant.
                            // Handled as Empty rather than todo!() or unreachable!(): BOTH PANIC, and a
                            // panic here closes the shell. The codebase already knows this -- see
                            // truncate_safe in commands/mod.rs, written so a multibyte anchor "never
                            // panics the shell via an out-of-bounds byte slice (a panic here closes
                            // fsh)". If a future refactor ever routes try_builtin through here, the
                            // honest failure is a silent no-op, not a dead terminal.
                            commands::CommandResult::NotBuiltin => {
                                engine.set_last_exit(Some(0));
                                None
                            }
                        };
                    // INT-191: close the lifecycle HERE, where the exit code finally exists.
                    // postexec could not do it: the pipeline arms above decide the code after
                    // `execute_with_context` has already returned.
                    if let Err(e) =
                        engine
                            .db()
                            .complete_command_execution(&crate::db::ExecutionCompletion {
                                session_id: exec::session_id(),
                                execution_id,
                                executed_text: Some(&base_cmd),
                                state: exec_state,
                                exit_code: engine.last_exit(),
                                duration_ms: Some(_cmd_timer_start.elapsed().as_millis() as u64),
                                finished_at: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64,
                            })
                    {
                        eprintln!("warning: failed to close command_execution record: {e}");
                    }
                    // Command timing intelligence — warn if command is unusually slow (INT-194)
                    {
                        let elapsed_ms = _cmd_timer_start.elapsed().as_millis() as i64;
                        let cmd_key_owned = commands::command_word(&base_cmd);
                        let cmd_key = cmd_key_owned.as_str();
                        if elapsed_ms > 500 {
                            let _ = engine.db().conn.execute(
                                "INSERT INTO shell_history (command, timestamp) VALUES (?1, ?2)",
                                rusqlite::params![
                                    format!("TIMING:{}:{}", cmd_key, elapsed_ms),
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_secs() as i64)
                                        .unwrap_or(0)
                                ],
                            );
                            let avg_ms: Option<f64> = engine.db().conn.query_row(
                                "SELECT AVG(CAST(SUBSTR(command, INSTR(command, ':', INSTR(command, ':')+1)+1) AS REAL))
                                 FROM shell_history WHERE command LIKE ?1 ORDER BY id DESC LIMIT 20",
                                rusqlite::params![format!("TIMING:{}:%", cmd_key)],
                                |r| r.get(0)
                            ).ok().flatten();
                            if let Some(avg) = avg_ms {
                                if avg > 100.0 && elapsed_ms as f64 > avg * 2.0 {
                                    println!("  {} {} took {}ms — {:.0}x slower than usual ({:.0}ms avg)",
                                        "⚠️ ".normal(),
                                        cmd_key.bright_yellow(),
                                        elapsed_ms,
                                        elapsed_ms as f64 / avg,
                                        avg
                                    );
                                }
                            }
                            // Long command notification -- >30s fires faelight-notify
                            if elapsed_ms > 30_000 {
                                let secs = elapsed_ms / 1000;
                                let msg = format!("{} finished in {}s", cmd_key, secs);
                                // INT-299: reap child in thread to prevent zombie process
                                if let Ok(mut child) = std::process::Command::new("faelight-notify")
                                    .arg("--title")
                                    .arg("Long command finished")
                                    .arg("--body")
                                    .arg(&msg)
                                    .spawn()
                                {
                                    std::thread::spawn(move || {
                                        let _ = child.wait();
                                    });
                                }
                            }
                        }
                    }
                    // INT-194 — Prediction-aware suggestions (pattern detection)
                    // After each command, check if there is a strong "next command" pattern
                    {
                        let cmd_key = commands::command_word(&base_cmd);
                        // Only suggest for meaningful commands, not builtins
                        let skip_suggest = matches!(
                            cmd_key.as_str(),
                            "d" | "ls" | "cd" | "echo" | "cat" | "help" | "exit" | "q" | "clear"
                        );
                        if !skip_suggest {
                            // Find what command most often follows this one
                            let next_cmd: Option<String> = engine
                                .db()
                                .conn
                                .query_row(
                                    "SELECT next_cmd, COUNT(*) as freq
                                 FROM (
                                   SELECT h2.command as next_cmd
                                   FROM shell_history h1
                                   JOIN shell_history h2 ON h2.id = h1.id + 1
                                   WHERE h1.command LIKE ?1
                                   AND h2.command NOT LIKE ?1
                                   AND length(h2.command) > 2
                                 )
                                 GROUP BY next_cmd ORDER BY freq DESC LIMIT 1",
                                    rusqlite::params![format!("{}%", cmd_key)],
                                    |r| r.get(0),
                                )
                                .ok();
                            if let Some(suggestion) = next_cmd {
                                // Only show if it appears often (check count >= 3)
                                let freq: i64 = engine
                                    .db()
                                    .conn
                                    .query_row(
                                        "SELECT COUNT(*) FROM shell_history h1
                                     JOIN shell_history h2 ON h2.id = h1.id + 1
                                     WHERE h1.command LIKE ?1 AND h2.command = ?2",
                                        rusqlite::params![format!("{}%", cmd_key), &suggestion],
                                        |r| r.get(0),
                                    )
                                    .unwrap_or(0);
                                if freq >= 3 {
                                    // Check friday_hints setting -- off = silent learning
                                    let hints_enabled = engine.db().conn.query_row(
                                        "SELECT value FROM shell_state WHERE key='config.friday_hints'",
                                        [],
                                        |r| r.get::<_, String>(0),
                                    ).unwrap_or_else(|_| "on".to_string());
                                    if hints_enabled != "off" {
                                        // INT-246: deduplicate hints -- only show once per session
                                        let hint_key = format!("hint_{}", suggestion);
                                        if !shown_friday_suggestions.contains(&hint_key) {
                                            shown_friday_suggestions.insert(hint_key);
                                            println!(
                                                "  {} you usually run {} next",
                                                "💡".normal(),
                                                suggestion.bright_cyan()
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // INT-296 Phase 5: Friday consecutive failure detection
                    {
                        let fail_cmd = commands::command_word(&base_cmd);
                        let consecutive: i64 = engine
                            .db()
                            .conn
                            .query_row(
                                "SELECT COUNT(*) FROM (
                                SELECT exit_code FROM term_commands
                                WHERE command LIKE ?1
                                ORDER BY id DESC LIMIT 3
                            ) AS recent WHERE exit_code != 0",
                                rusqlite::params![format!("{}%", fail_cmd)],
                                |r| r.get(0),
                            )
                            .unwrap_or(0);
                        if consecutive >= 3 {
                            let fail_key = format!("fail3_{}", fail_cmd);
                            if !shown_friday_suggestions.contains(&fail_key) {
                                shown_friday_suggestions.insert(fail_key);
                                println!(
                                    "  🌲 Friday: {} failed {} times in a row -- check the command",
                                    fail_cmd, consecutive
                                );
                                let notify_body = format!(
                                    "{} failed {} times in a row -- Friday suggests checking the command",
                                    fail_cmd, consecutive
                                );
                                let _ = std::process::Command::new("notify-send")
                                    .args(["🌲 Friday", &notify_body])
                                    .spawn();
                            }
                        }
                    }
                    // Store last output for `last` command (INT-194)
                    if let Some(ref out) = cmd_output {
                        if !out.is_empty() {
                            let _ = engine.db().conn.execute(
                                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_output', ?1)",
                                rusqlite::params![out],
                            );
                        }
                    }
                    // INT-201 — Track last command exit status for faelight-term indicator
                    {
                        // FIXED: this block used to RE-DERIVE success by scanning the output
                        // text for the cross-mark prefix / "error" / "not found", and then
                        // OVERWROTE last_exit_code with that guess. The guess was a SECOND
                        // SOURCE OF TRUTH and was wrong in BOTH directions: a successful
                        // command whose legitimate output mentions the word "error" (e.g. a
                        // report COUNTING parse errors) was recorded as a failure, and a
                        // genuinely failed builtin whose message lacks those words was recorded
                        // as a success. That corrupted term_commands.exit_code, which Friday's
                        // three-failures-in-a-row detector reads -- the shell was learning from
                        // fabricated observations.
                        //
                        // The verdict is ALREADY correct: the CommandResult match above sets
                        // last_exit_code (Output/Empty/NotBuiltin -> 0, Error -> 1). This block
                        // now only CONSUMES it. The faelight-term cache write is kept; only the
                        // re-derivation is gone.
                        //
                        // KNOWN GAP, recorded not hidden: four arms of that match never set
                        // last_exit_code at all (both Value arms, and the two arms that spawn
                        // `sh` for pipelines and discard its status), so on those paths the
                        // value carries over from the previous command. The string scan was
                        // crudely papering over that; removing it makes the staleness VISIBLE
                        // rather than guessed. Fixing it touches pipeline execution semantics
                        // (is pipeline status the last command? the first failure?) and belongs
                        // in its own intent with its own verification -- deliberately NOT
                        // bundled with a telemetry-corruption fix.
                        let exit_ok = engine.last_exit().map(|c| c == 0).unwrap_or(true);
                        let status_val = if exit_ok { "success" } else { "failure" };
                        let cache_dir =
                            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
                                .join(".cache/faelight");
                        let _ = std::fs::create_dir_all(&cache_dir);
                        let _ = std::fs::write(cache_dir.join("last-exit-status"), status_val);
                    }
                    // Write to file if redirect was detected, otherwise print
                    if let Some(output) = cmd_output {
                        if let Some((ref path, append)) = redirect {
                            use std::io::Write;
                            let home = std::env::var("HOME").unwrap_or_default();
                            let full_path = if path.starts_with("~/") {
                                format!("{}/{}", home, &path[2..])
                            } else {
                                path.clone()
                            };
                            let file = std::fs::OpenOptions::new()
                                .write(true)
                                .create(true)
                                .append(append)
                                .truncate(!append)
                                .open(&full_path);
                            match file {
                                Ok(mut f) => {
                                    let _ = f.write_all(output.as_bytes());
                                    let _ = f.write_all(
                                        b"
",
                                    );
                                    let mode = if append { ">>" } else { ">" };
                                    println!(
                                        "  {} {} {}",
                                        "○".bright_cyan(),
                                        mode.dimmed(),
                                        full_path.bright_white()
                                    );
                                }
                                Err(e) => eprintln!("  ✗ redirect failed: {}", e),
                            }
                        } else {
                            println!("{}", output);
                        }
                    }
                    // Phase 20b — apply cwd after yazi/fm exits
                    if is_fm_cmd {
                        if let Ok(cwd) = std::fs::read_to_string(&fm_cwd_file) {
                            let cwd = cwd.trim();
                            if !cwd.is_empty() {
                                let _ = std::env::set_current_dir(cwd);
                            }
                        }
                        let _ = std::fs::remove_file(&fm_cwd_file);
                    }
                    // Phase 17 — evaluate triggers after every command
                    triggers::ensure_schema(engine.db());
                    let health = engine.db().health_score();
                    let trigger_ctx = triggers::TriggerContext {
                        last_command: base_cmd.clone(),
                        health_score: health,
                        last_domain: None,
                    };
                    triggers::evaluate(engine.db(), &trigger_ctx, engine.core_root());
                    // INT-220 -- Send FridayEvent to daemon socket (fire and forget)
                    {
                        let cmd_str = base_cmd.clone();
                        // Read exit status from cache file written above
                        let exit_code: i32 = {
                            let cache_dir =
                                std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
                                    .join(".cache/faelight");
                            let status =
                                std::fs::read_to_string(cache_dir.join("last-exit-status"))
                                    .unwrap_or_default();
                            if status.trim() == "success" {
                                0
                            } else {
                                1
                            }
                        };
                        engine.set_last_exit(Some(exit_code));
                        let now_ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        let home_dir = std::env::var("HOME").unwrap_or_default();
                        let sock_path_buf = format!("{}/.local/state/0-core/daemon.sock", home_dir);
                        let sock_path = sock_path_buf.as_str();
                        // Build JSON safely -- escape special chars in command
                        let cmd_escaped = cmd_str
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('\n', "\\n")
                            .replace('\r', "\\r");
                        let event_json = format!(
                            "{{\"id\":1,\"payload\":{{\"FridayEvent\":{{\"command\":\"{}\",\"exit_code\":{},\"duration_ms\":0,\"intent\":null,\"health\":{},\"timestamp\":{}}}}}}}",
                            cmd_escaped,
                            exit_code, health.unwrap_or(100), now_ts
                        );
                        if std::path::Path::new(sock_path).exists() {
                            use std::io::{BufRead, BufReader, Write};
                            if let Ok(mut stream) =
                                std::os::unix::net::UnixStream::connect(sock_path)
                            {
                                stream
                                    .set_write_timeout(Some(std::time::Duration::from_millis(100)))
                                    .ok();
                                stream
                                    .set_read_timeout(Some(std::time::Duration::from_millis(1000)))
                                    .ok();
                                let _ = stream.write_all(event_json.as_bytes());
                                let _ = stream.write_all(b"\n");
                                // Gate 7 -- read FridaySpeak response inline
                                let mut reader = BufReader::new(&stream);
                                let mut resp = String::new();
                                if reader.read_line(&mut resp).is_ok()
                                    && resp.contains("FridaySpeak")
                                {
                                    if (resp.contains("\"low\"")
                                        || resp.contains("\"medium\"")
                                        || resp.contains("\"high\""))
                                        && resp.contains("\"message\":\"")
                                    {
                                        if let Some(msg) = resp.split("\"message\":\"").nth(1) {
                                            if let Some(msg) = msg.split('"').next() {
                                                if !msg.is_empty() && msg != "null" {
                                                    // INT-246: once per intent -- only speak when intent changed
                                                    let current_intent = engine
                                                        .db()
                                                        .get_focus_intent()
                                                        .map(|i| format!("{}", i));
                                                    if current_intent == last_friday_intent
                                                        && last_friday_intent.is_some()
                                                    {
                                                        continue;
                                                    }
                                                    // INT-246: never repeat same suggestion in a session
                                                    if shown_friday_suggestions.contains(msg) {
                                                        continue;
                                                    }
                                                    shown_friday_suggestions
                                                        .insert(msg.to_string());
                                                    last_friday_intent = current_intent;
                                                    println!();
                                                    let tier = if resp.contains("\"high\"") {
                                                        ("RECOMMEND", "78%")
                                                    } else if resp.contains("\"medium\"") {
                                                        ("SUGGEST", "62%")
                                                    } else {
                                                        ("SUGGEST", "54%")
                                                    };
                                                    println!(
                                                        "  🌲 Friday: {}  ·  {} · {}",
                                                        msg, tier.0, tier.1
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // 🌲 Forest speaks — surface insightd insights after every command
                    {
                        let insight: Option<(i64, String, String, f64)> = engine
                            .db()
                            .conn
                            .query_row(
                                "SELECT id, signal, detail, importance FROM forest_insights
                             WHERE shown = 0 AND importance >= 0.65
                             ORDER BY importance DESC, created_at DESC LIMIT 1",
                                [],
                                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                            )
                            .ok();
                        if let Some((id, _signal, detail, importance)) = insight {
                            use colored::Colorize;
                            let icon = if importance >= 0.85 { "⚡" } else { "💬" };
                            println!();
                            println!(
                                "  {} {} {}",
                                icon,
                                "forest:".bright_cyan().dimmed(),
                                detail.bright_white()
                            );
                            let _ = engine.db().conn.execute(
                                "UPDATE forest_insights SET shown = 1 WHERE id = ?1",
                                rusqlite::params![id],
                            );
                        }
                    }
                    // INT-203 Phase 2 + INT-277: Friday proactive message with attention scoring
                    if _session_commands % 10 == 0 && _session_commands > 0 {
                        let pattern: Option<(String, String, f64)> = engine.db().conn.query_row(
                            "SELECT trigger, action, confidence FROM friday_patterns WHERE confidence >= 0.7 ORDER BY confidence DESC LIMIT 1",
                            [], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?))
                        ).ok();
                        if let Some((trigger, action, conf)) = pattern {
                            // INT-277: compute attention score before speaking
                            let seen_count: i64 = engine.db().conn.query_row(
                                "SELECT COUNT(*) FROM friday_attention WHERE event_type = 'pattern_match'",
                                [], |r| r.get(0),
                            ).unwrap_or(0);
                            let novelty = match seen_count {
                                0 => 1.0f64,
                                1..=2 => 0.7,
                                3..=10 => 0.4,
                                _ => 0.15,
                            };
                            let risk = 0.4f64; // pattern suggestion is informational
                            let strategic_relevance = if conf >= 0.95 { 0.8 } else { 0.5 };
                            let uncertainty = 1.0 - conf;
                            let temporal_pressure = 0.3f64;
                            let attention_score = (novelty
                                * risk
                                * strategic_relevance
                                * uncertainty
                                * temporal_pressure)
                                .powf(0.2);
                            let spoke = attention_score >= 0.6;
                            // Record in friday_attention
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64;
                            let _ = engine.db().conn.execute(
                                "INSERT INTO friday_attention (timestamp, event_type, event_detail, novelty, risk, strategic_relevance, uncertainty, temporal_pressure, attention_score, threshold, spoke) VALUES (?1,'pattern_match',?2,?3,?4,?5,?6,?7,?8,0.6,?9)",
                                rusqlite::params![now, format!("{} -> {}", trigger, action), novelty, risk, strategic_relevance, uncertainty, temporal_pressure, attention_score, if spoke { 1 } else { 0 }],
                            );
                            if spoke {
                                use colored::Colorize;
                                println!();
                                println!(
                                    "  🌲 Friday: When {} → {} ({:.0}%)",
                                    trigger.bright_cyan(),
                                    action.bright_white(),
                                    conf * 100.0
                                );
                            }
                        }
                    }
                } // end 'segments loop
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C — clear line, return to prompt
                println!();
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    // Write journal daily summary on exit
    let _ = std::process::Command::new("core")
        .args(["journal", "daily-summary"])
        .output();
    // Save session state on exit
    session::SessionMemory::save(engine.core_root(), None, engine.db());
    // INT-208: Log session pattern with focus_score
    let _session_duration = _session_start.elapsed().as_secs() / 60;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let dow = chrono::Local::now().weekday().num_days_from_monday() as i64;
    let hour = chrono::Local::now().hour() as i64;
    // Compute focus_score: 1.0 = all commits on one intent, lower = spread across many
    let session_start_ts = now - (_session_duration as i64 * 60);
    let distinct_intents: i64 = engine.db().conn.query_row(
        "SELECT COUNT(DISTINCT intent_id) FROM commit_patterns WHERE timestamp > ?1 AND intent_id != ''",
        rusqlite::params![session_start_ts],
        |r| r.get(0),
    ).unwrap_or(1);
    let focus_score: f64 = if distinct_intents <= 1 {
        1.0
    } else {
        1.0 / distinct_intents as f64
    };
    let _ = engine.db().conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS session_patterns (
            id TEXT PRIMARY KEY,
            day_of_week INTEGER,
            hour_start INTEGER,
            hour_end INTEGER,
            commit_count INTEGER,
            recorded_at INTEGER,
            focus_score REAL,
            deploy_count INTEGER,
            command_count INTEGER,
            duration_minutes INTEGER
        );",
    );
    let _ = engine.db().conn.execute(
        "INSERT OR REPLACE INTO session_patterns (id, day_of_week, hour_start, hour_end, commit_count, recorded_at, focus_score, deploy_count, command_count, duration_minutes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![now.to_string(), dow, hour, hour, _session_commits as i64, now, focus_score, _session_deploys as i64, _session_commands as i64, _session_duration as i64],
    );
    // INT-229: Session summary on exit
    {
        use colored::Colorize;
        let dur_str = if _session_duration >= 60 {
            format!("{}h{}m", _session_duration / 60, _session_duration % 60)
        } else {
            format!("{}m", _session_duration)
        };
        let active_intent: String =
            std::fs::read_dir(faelight_core::paths::intents_dir().join("in-progress"))
                .map(|d| {
                    d.filter_map(|e| e.ok())
                        .filter(|e| {
                            std::fs::read_to_string(e.path())
                                .map(|c| c.contains("status: in-progress"))
                                .unwrap_or(false)
                        })
                        .filter_map(|e| {
                            let n = e.file_name().to_string_lossy().to_string();
                            let num = n.split('-').next()?.to_string();
                            Some(format!("INT-{}", num))
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
        println!();
        // INT-169: MIGRATION, not compatibility. A green suite proves a command behaved the
        // same; it cannot say whether the spine ran it or the router declined and legacy did.
        // Printed once per session and only on request, so normal use stays quiet.
        if std::env::var_os("FSH_SPINE_METRICS").is_some() {
            if let Some(report) = exec::spine_routing_report() {
                println!("{report}");
            }
        }
        println!("  🌲 Session complete");
        println!(
            "  {} commands  ·  {} deploys  ·  {} commits  ·  {}",
            _session_commands,
            _session_deploys,
            _session_commits,
            dur_str.bright_green()
        );
        if !active_intent.is_empty() {
            println!("  Active: {}", active_intent.bright_cyan());
        }
        println!();
    }
    println!(
        "{}",
        colored::Colorize::dimmed("  🌲 The forest remembers.")
    );
    Ok(())
}

// Faelight truecolor helpers -- neon candy palette
#[allow(dead_code)]
fn fc(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}
fn fc_bold(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[1m\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}
fn fc_dim(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[2m\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

fn print_welcome(core_root: &str, db: &crate::db::ForestDb) {
    use colored::Colorize;
    use std::path::PathBuf;

    let root = PathBuf::from(core_root);

    let version = std::fs::read_to_string(root.join("faelight/meta/VERSION"))
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();

    let changelog =
        std::fs::read_to_string(root.join("faelight/meta/CHANGELOG.md")).unwrap_or_default();
    let theme = changelog
        .lines()
        .find(|l| l.starts_with(&format!("## [{}]", version)))
        .and_then(|l| {
            if l.contains(" — ") {
                l.split(" — ").nth(1)
            } else {
                l.split(" -- ").nth(1)
            }
        })
        .and_then(|s| s.split('(').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "The Living Forest".to_string());

    let commits = std::process::Command::new("git")
        .args(["-C", core_root, "rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "?".to_string());

    let health_num: u32 = std::fs::read_to_string(
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".cache/faelight/health-status"),
    )
    .unwrap_or_else(|_| "95".into())
    .trim()
    .trim_end_matches('%')
    .parse()
    .unwrap_or(95);

    let health_display = if health_num >= 95 {
        fc_bold(57, 255, 20, &format!("{}% ✅", health_num))
    } else if health_num >= 80 {
        fc_bold(255, 200, 0, &format!("{}% ⚠", health_num))
    } else {
        fc_bold(255, 70, 70, &format!("{}% ❌", health_num))
    };

    // Count intents by scanning all categories — mirrors doctor check_intents logic exactly
    let (complete_count, planned_count) = {
        let intent_dir = faelight_core::paths::intents_dir();
        let categories = [
            "complete",
            "decisions",
            "experiments",
            "philosophy",
            "future",
            "cancelled",
            "deferred",
            "incidents",
            "active",
        ];
        let mut complete = 0usize;
        let mut planned = 0usize;
        for cat in &categories {
            if let Ok(entries) = std::fs::read_dir(intent_dir.join(cat)) {
                for entry in entries.flatten() {
                    if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            if content.contains("status: complete") {
                                complete += 1;
                            } else if content.contains("status: planned") {
                                planned += 1;
                            }
                        }
                    }
                }
            }
        }
        (complete, planned)
    };
    // Count tools from registry — mirrors doctor check_path_resilience logic exactly
    let tool_count = std::fs::read_to_string(faelight_core::paths::tools_registry())
        .map(|t| t.lines().filter(|l| l.starts_with("name = ")).count())
        .unwrap_or(0);

    let quotes = [
        "Nothing runs without explicit human authorization.",
        "The forest remembers. The human decides.",
        "Every tool is understood. Nothing is installed blindly.",
        "Freedom without structure is not empowerment — it is entropy.",
        "A forest that knows itself can survive anything.",
        "The roots hold. The branches grow.",
        "Every commit is intentional. Every tool has a purpose.",
        "Understanding over convenience. Always.",
        "The forest does not fear the storm. It knows how to grow back.",
        "A wise forest studies its own rings.",
        "The last sibling came home.",
        "Not text streams. Not configuration. Structured wisdom.",
    ];
    // Rotate quotes via state.db — never repeat consecutively
    let quote = {
        let _ = db.conn.execute(
            "CREATE TABLE IF NOT EXISTS shell_state (key TEXT PRIMARY KEY, value TEXT)",
            [],
        );
        let last_idx: usize = db
            .conn
            .query_row(
                "SELECT value FROM shell_state WHERE key='last_quote_idx'",
                [],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(999);
        // Pick next quote (skip last shown)
        let next_idx = {
            let mut idx = (last_idx + 1) % quotes.len();
            if idx == last_idx {
                idx = (idx + 1) % quotes.len();
            }
            idx
        };
        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_quote_idx', ?1)",
            rusqlite::params![next_idx.to_string()],
        );
        quotes[next_idx]
    };
    println!();
    // ── detect current compositor from WAYLAND_DISPLAY / XDG_SESSION_DESKTOP ──
    let compositor = std::env::var("XDG_SESSION_DESKTOP")
        .or_else(|_| std::env::var("XDG_CURRENT_DESKTOP"))
        .unwrap_or_else(|_| {
            std::env::var("WAYLAND_DISPLAY")
                .map(|_| "wayland".to_string())
                .unwrap_or_else(|_| "tty".to_string())
        });
    // ── detect NixOS generation ──
    let nix_gen = std::process::Command::new("nixos-rebuild")
        .args(["list-generations", "--json"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            // find "current":true entry and get generation number
            let v: serde_json::Value = serde_json::from_str(&s).ok()?;
            v.as_array()?
                .iter()
                .find(|g| g["current"].as_bool().unwrap_or(false))
                .and_then(|g| g["generation"].as_u64())
                .map(|n| n.to_string())
        })
        .unwrap_or_else(|| "?".to_string());
    // ── header ──
    println!(
        "  {}  {}",
        fc_bold(57, 255, 20, "🌲 Faelight Forest"),
        fc_dim(140, 220, 100, &format!("gen {} · {}", nix_gen, compositor))
    );
    println!();
    println!(
        "  {}",
        fc_bold(57, 255, 20, &format!("{} -- {}", version, theme))
    );
    // ── neon separator ──
    println!(
        "  {}",
        fc(
            34,
            200,
            80,
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        )
    );
    // ── stats row ──
    println!(
        "  {} {}  {} {}  {} {}  {} {}",
        fc_bold(57, 255, 20, &complete_count.to_string()),
        fc_dim(120, 200, 100, "done"),
        fc_bold(255, 200, 50, &commits),
        fc_dim(180, 160, 80, "commits"),
        fc_bold(50, 220, 255, &tool_count.to_string()),
        fc_dim(100, 180, 200, "tools"),
        fc_dim(160, 140, 220, &format!("{} planned", planned_count)),
        fc_dim(100, 100, 140, "")
    );
    // ── health row ──
    println!(
        "  {}  {}",
        &health_display,
        fc_dim(100, 160, 100, "system health")
    );
    println!();
    // ── philosophy quote -- bold not dimmed ──
    println!("  {}", fc_bold(180, 130, 255, &format!("\"{}\"", quote)));
    println!();
    // Today's Focus — lowest audit score tool
    let _focus = std::fs::read_to_string(faelight_core::paths::tools_registry())
        .map(|t| {
            // Find tool with lowest score hint from name patterns
            let stale: Vec<&str> = t
                .lines()
                .filter(|l| l.starts_with("name = "))
                .filter_map(|l| l.split('"').nth(1))
                .collect();
            stale.first().map(|s| s.to_string()).unwrap_or_default()
        })
        .unwrap_or_default();

    // Show today's focus from actual in-progress intents only
    let focus_intent: Option<String> =
        std::fs::read_dir(faelight_core::paths::intents_dir().join("future"))
            .ok()
            .and_then(|entries| {
                let mut in_progress: Vec<String> = entries
                    .flatten()
                    .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                    .filter_map(|e| {
                        let content = std::fs::read_to_string(e.path()).ok()?;
                        if !content.contains("status: in-progress") {
                            return None;
                        }
                        Some(
                            e.file_name()
                                .to_string_lossy()
                                .trim_start_matches(|c: char| c.is_ascii_digit() || c == '-')
                                .trim_end_matches(".md")
                                .replace('-', " ")
                                .to_string(),
                        )
                    })
                    .collect();
                in_progress.sort();
                if in_progress.is_empty() {
                    None
                } else {
                    Some(in_progress.join(", "))
                }
            });

    if let Some(ref focus) = focus_intent {
        println!(
            "  {}  {}",
            fc_dim(255, 180, 50, "Today:"),
            fc_bold(255, 230, 100, focus)
        );
        // Auto-persist detected intent so prompt.rs can read it
        {
            // Only write if no conscious focus already set
            if db.get_focus_intent().is_none() {
                // Extract INT-NNN from filename — only if first token is numeric
                // deadwood: exempt -- focus INTENT IDENTIFIER extraction; the numeric id is ledger metadata read from the focus file, not shell input and not a command selector
                if let Some(int_id) = focus.split_whitespace().next() {
                    if int_id.chars().all(|c| c.is_ascii_digit()) {
                        let intent_key = format!("INT-{}", int_id);
                        if let Err(e) = db.set_focus_intent(&intent_key) {
                            eprintln!("warning: failed to set focus intent: {}", e);
                        }
                    }
                }
            }
        }
    }
    println!();
    // Session memory + digest
    if let Some(mem) = session::SessionMemory::load(core_root, db) {
        // Phase 23 — restore last working directory
        if let Some(ref last_dir) = mem.last_dir {
            let path = std::path::Path::new(last_dir);
            // Always restore to core_root — keep work in forest home
            let restore_path = if path.exists()
                && path.is_dir()
                && !last_dir.contains("/engine/src")
                && !last_dir.contains("/rust-tools/")
            {
                path
            } else {
                std::path::Path::new(core_root)
            };
            let _ = std::env::set_current_dir(restore_path);
        }
        let msg = session::render(&mem, core_root, db);
        if !msg.is_empty() {
            println!("{}", msg);
        }
        // INT-143 Phase 1 — forest digest on long gaps
        if digest::should_show(&mem) {
            {
                let d = digest::render(&mem, db, core_root);
                if !d.is_empty() {
                    println!("{}", d);
                }
            }
        }
    }

    // INT-207 L1 — Show alignment score on session start
    {
        let align: Option<f64> = db.conn.query_row(
            "SELECT AVG(score) FROM alignment_checks WHERE checked_at > (strftime('%s','now') - 604800)",
            [], |r| r.get(0)
        ).ok().flatten();
        if let Some(score) = align {
            let pct = (score * 100.0) as i64;
            let colored = if pct >= 80 {
                format!("{}%", pct).bright_green()
            } else if pct >= 60 {
                format!("{}%", pct).bright_yellow()
            } else {
                format!("{}%", pct).bright_red()
            };
            println!("  {} alignment: {}", "🧭".normal(), colored);
        }
    }
    println!(
        "  {} for commands  ·  {} to exit",
        "help".bright_cyan(),
        "q".dimmed()
    );
    println!();
}
