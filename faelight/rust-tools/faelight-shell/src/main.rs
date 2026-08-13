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
                    // INT-203: LENGTH WAS THE ONLY CHECK, and that is the whole bug. A space is one character,
                    // so `{ .. }` parsed as a character range from space to space and expanded to a single
                    // space -- silently eating any brace group written with the range operator inside it. A Rust
                    // match pattern pasted through a heredoc arrived with its braces replaced by three spaces,
                    // and three patch scripts reported success while grep and rustc disagreed, because the text
                    // was corrupted in transit rather than the write failing.
                    //
                    // Requiring both endpoints to be ASCII LETTERS is the whole repair. `{a..z}` and
                    // `{A..Z}` still expand. `{1..5}` never reached here -- the integer branch above
                    // claims it when both sides parse. A mixed `{1..a}` now stays literal, which is what
                    // bash does. A space, a dot or a quote can no longer be a range endpoint.
                    //
                    // NOT FIXED HERE, and recorded rather than hidden: this function is still neither quote-aware
                    // nor heredoc-aware, so `echo "{a..c}"` still expands inside quotes. That is the
                    // INT-196 class -- code inferring shell structure from raw text -- and it needs its own
                    // evidence. See INT-203.
                    if lc.len() == 1
                        && rc.len() == 1
                        && lc[0].is_ascii_alphabetic()
                        && rc[0].is_ascii_alphabetic()
                    {
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
/// INT-196 M2-M4: the command word the SAFETY GUARD judges, derived from the parser.
///
/// THE AUTHORITY HIERARCHY, and it falls back ONE LAYER DOWN THE SAME PIPELINE rather than to a
/// text heuristic. That is the whole point of this intent: the guard never invents its own reading
/// of the line.
///   Complete            -- the first Word of the parsed Command, which is the strongest answer.
///   Incomplete/Refused  -- the scanner first Word token. Still parser-owned. A refusal means the
///                          parser declines OWNERSHIP, not that the input is harmless, and an
///                          incomplete multi-line paste still executes via pty_exec.
///   Invalid             -- None. No word, no guard decision, and no execution follows either.
///
/// ⚠️ THE SCANNER TOKEN IS TAKEN AS IT COMES. If it ever stops matching what the guard lists
/// expect, the answer is a SHARED REPRESENTATION -- never a second mini command_word here. That
/// would rebuild the exact derivation this intent removed, one layer lower.
///
/// ⚠️ An OPERATOR-leading line yields None. A line beginning with a pipe has no command word, and
/// inventing one would be a judgement the scanner deliberately refuses to make.
/// INT-197: the EXECUTABLE IDENTITY the safety policy matches on.
///
/// The three guard lists match BARE NAMES -- core, git, cargo, cat, ls, cd. An expanded alias can
/// present an absolute path: `d` expands to a full store path ending in core, and three of the 284
/// aliases do this. Without normalization, evaluating the expanded form would drop those out of the
/// safe list and start prompting on a harmless doctor run.
///
/// NORMALIZED ONCE, HERE, rather than teaching three lists to recognise paths. Three copies of one
/// rule is the disease this codebase has spent a month removing.
///
/// THE BOUNDARY, and it is deliberate rather than lossy by accident: policy identity is the
/// executable, normalized to its basename. ARGUMENTS ARE NOT RESOLVED RECURSIVELY, including
/// scripts passed to interpreters -- so `rebuild`, which expands to bash running a script, is
/// identified as `bash`. If the guard began interpreting scripts, wrappers or aliases semantically
/// it would have crossed from executable policy into command interpretation. Distinguishing bash
/// with a known script from arbitrary bash is a NEW CAPABILITY needing a richer command identity,
/// not something to smuggle into a basename.
fn policy_identity(word: &str) -> String {
    if !word.contains(std::path::MAIN_SEPARATOR) {
        return word.to_string();
    }
    std::path::Path::new(word)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| word.to_string())
}

fn guard_command_word(line: &str) -> Option<String> {
    // MEASURED 2026-08-12, and it bounds what the AST arm can be said to do. Disabling this
    // arm entirely and letting the scanner answer everything left the suite at 151/151,
    // including both gen-432 guard cases. So the two layers AGREE on every input the suite
    // contains, and no test here can distinguish them.
    //
    // THAT IS EQUIVALENCE, NOT CAUSAL NECESSITY. The AST word is the primary source in the
    // implementation; it has never been shown to matter observably. The hierarchy is kept
    // because it is the architecture -- and because this arm is what will carry a Word with
    // parts once expansion moves onto the spine, where the scanner token cannot follow.
    //
    // A property that holds only by coincidence between two layers is exactly the kind that
    // stops holding silently, which is why it is recorded here rather than assumed.
    use crate::spine::lexer::{LexResult, TokenKind};
    use crate::spine::parser::ParseResult;

    let from_tokens = || -> Option<String> {
        match crate::spine::lexer::lex(line) {
            LexResult::Complete(tokens) => tokens
                .first()
                .filter(|t| t.kind == TokenKind::Word)
                .map(|t| t.text.clone()),
            LexResult::Incomplete(_) => None,
        }
    };

    match crate::spine::parser::parse(line) {
        ParseResult::Complete(node) => match &node.node {
            crate::spine::ast::AstNode::Command(cmd) => cmd
                .words
                .first()
                .and_then(|w| match w.node.parts.first() {
                    Some(crate::spine::ast::WordPart::Literal { text, .. }) => Some(text.clone()),
                    _ => None,
                })
                .or_else(from_tokens),
            _ => from_tokens(),
        },
        ParseResult::Incomplete(_) | ParseResult::Refused(_) => from_tokens(),
        ParseResult::Invalid(_) => None,
    }
}

pub(crate) fn is_repl_state_command(line: &str) -> bool {
    // INT-196: THE SECOND WORD COMES FROM THE TOKENIZER TOO. The command word was already derived
    // quote-aware; the word beside it was read off the raw line with split_whitespace, so a quoted
    // job spec arrived with its quote attached and the checks below failed on it.
    //
    // ⚠️ THIS DECIDES ROUTING EXCLUSION, and engine.rs asks it four times including the spine
    // router. A wrong answer routes a line that should have been excluded, so `kill "%1"` reached
    // the real kill with a job spec instead of a PID. Same shape as the gen 432 guard defect: the
    // command word quote-aware, its neighbour not.
    //
    // commands::tokenize is INT-171 gate 1, the single quote-aware tokenizer, and command_word is
    // built on it -- so both words now come from one owner rather than two derivations.
    let first = commands::command_word(line);
    let tokens = commands::tokenize(line);
    let second: &str = tokens.get(1).map(|s| s.as_str()).unwrap_or("");
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

/// INT-201 gate 4: is this process a `-c` invocation? Read by run_external so the one-deploy-cycle
/// measurement can tell a non-interactive fallback from an interactive one.
///
/// A static rather than a parameter because run_external sits at the bottom of the ordering, and
/// threading a flag through every caller would touch far more than the thing being measured.
/// INT-201: shell UI that must not contaminate a non-interactive stdout.
///
/// `-c`'s stdout belongs to the program being run. A progress line or an assignment confirmation on
/// it turns `fsh -c 'X=1; echo $X'` into two lines where a caller expects one. The diagnostics rule
/// already decided where this goes: any non-program output from a non-interactive invocation belongs
/// on stderr, so it is still visible to someone watching a script run and invisible to a pipe.
/// BUG-298-1 / INT-201: expand a leading or mid-string `~/` against HOME.
///
/// Lifted verbatim out of the base_cmd path so the assignment path can use the SAME rule. It was
/// inline there, which is why `x=~/test` stored a literal tilde while `cat ~/test` worked -- one
/// expander, one caller, and the other path silently without it.
///
/// ⚠️ THE TWO BRANCHES ARE NOT INTERCHANGEABLE and the else is not a fallback: a leading `~/` is
/// replaced positionally, while an interior one is matched on a PRECEDING SPACE so that a path like
/// /var/tmp~/x is left alone. Kept byte-identical to the original rather than tidied, because
/// changing base_cmd behaviour while fixing assignments is the sort of thing that hides for months.
/// INT-201: a child's status as a shell reports it -- code, or 128+signal when it was killed.
///
/// `.code()` is None on Unix for a signalled process, and this codebase wrote `.unwrap_or(1)` at more
/// than ten sites, so every interrupted command answered 1 no matter which signal ended it. The `-c`
/// handler had the same defect in its own copy and was fixed on its own in 3c5220be; this is the rest
/// of the family, given one owner so the next site cannot drift.
pub(crate) fn exit_status_code(status: &std::process::ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    match status.code() {
        Some(c) => c,
        None => status.signal().map(|s| 128 + s).unwrap_or(1),
    }
}

pub(crate) fn expand_tilde(s: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    if s.starts_with("~/") {
        format!("{}{}", home, &s[1..])
    } else {
        s.replace(" ~/", &format!(" {}/", home))
    }
}

pub(crate) fn ui_line(s: String) {
    if IS_DASH_C.load(std::sync::atomic::Ordering::SeqCst) {
        eprintln!("{}", s);
    } else {
        println!("{}", s);
    }
}

pub(crate) static IS_DASH_C: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

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
            // ⚠️ A MISSING OPERAND IS A USAGE ERROR, NOT A SUCCESS. This exited 0, so `fsh -c` with
            // nothing after it reported that a command nobody supplied had run fine. bash exits 2
            // with a message and so does this now.
            let Some(cmd_str) = args.get(c_pos + 1) else {
                eprintln!("fsh: -c requires an argument");
                std::process::exit(2);
            };
            // ⭐ INT-201 GATE 4: `-c` EXECUTES fsh, NOT sh.
            //
            // This delegated the whole string to /bin/sh, so nothing fsh does applied to it: no
            // aliases, no spine router, no digit guard, no job table. The suite was structurally
            // blind to every bug below the router for months because it knocked on this door.
            //
            // It now calls the SAME run_input the REPL calls -- the same function, not a copy of the
            // ordering. Duplicating it here is the second-orderer problem this change exists to
            // prevent.
            //
            // ⚠️ NO `-c` FALLBACK, DELIBERATELY. The chain is run_input -> routing -> engine ->
            // run_external -> sh, so the delegation already has an owner one layer down. A second
            // one here would be another escape path rather than instrumentation of the first.
            //
            // ★ THE CWD REQUIREMENT IS MET BY CONSTRUCTION: this never reaches repl_main, so the
            // forest-home default never runs and the caller's directory is simply inherited.
            IS_DASH_C.store(true, std::sync::atomic::Ordering::SeqCst);
            let cmd_str = cmd_str.clone();
            let RuntimeInit {
                db,
                cfg,
                applied: _,
                diagnostics,
            } = runtime_init()?;
            // Diagnostics on stderr: stdout belongs to the program, which is what keeps
            // `fsh -c 'echo hi' | wc -l` answering 1.
            for e in &diagnostics {
                eprintln!("{}", e);
            }
            let core_root = db.core_root();
            let mut engine = crate::engine::Engine::new(db, core_root, cfg.before_rules);
            let mut job_table = jobs::JobTable::new();
            let mut shown: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut last_intent: Option<String> = None;
            let _ = run_input(
                &mut engine,
                &cmd_str,
                &mut job_table,
                &mut shown,
                &mut last_intent,
                0,
            );
            std::process::exit(engine.last_exit().unwrap_or(0));
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

/// INT-206: may the shell stay in the directory it was SPAWNED in?
///
/// fsh starts in the forest home on purpose, and restores its last directory on purpose. Both are
/// deliberate and neither is being removed. What was missing is a way OUT: a harness that spawns fsh
/// with a chosen working directory had no way to make that stick, so fsh-test asked for /tmp and
/// silently got the repository -- which is how two conformance files came to be written into it on
/// every run.
///
/// ⚠️ AN EXPLICIT SIGNAL IS THE ONLY HONEST MECHANISM HERE. A spawned process cannot tell a chosen
/// working directory from an inherited one, and the obvious heuristic -- honour it unless this is an
/// interactive login shell -- does not work either, because fsh-test drives a REAL PTY, so stdin is
/// a terminal inside the harness too.
///
/// Defaults to false: unset, every existing behaviour is unchanged.
fn keep_launch_cwd() -> bool {
    std::env::var("FSH_KEEP_CWD")
        .map(|v| v != "0")
        .unwrap_or(false)
}

/// INT-201 gate 4: the ONE implementation of input -> routing -> execution ordering.
///
/// Extracted from repl_main VERBATIM -- the ordering and behaviour are unchanged, and the
/// INT-196 derivations moved with it rather than being cleaned on the way. That separation is
/// deliberate: cleaning them here would smuggle a different change into this one.
///
/// WHY IT EXISTS AT ALL: `-c` needs something to call. The alternative was duplicating this
/// ordering for the non-interactive door, which is the second-orderer problem INT-171, INT-193
/// and INT-196 all exist to eliminate. Moving it costs nothing; duplicating it costs the
/// invariant.
///
/// The engine is the eventual owner. It is not the owner yet because five main.rs free
/// functions -- split_into_segments, detect_redirect, expand_vars, friday_next_cmd_hint and
/// friday_proactive_message -- would have to cross the crate boundary in the same change.
fn run_input(
    mut engine: &mut crate::engine::Engine,
    line: &str,
    mut job_table: &mut jobs::JobTable,
    mut shown_friday_suggestions: &mut std::collections::HashSet<String>,
    mut last_friday_intent: &mut Option<String>,
    _session_commands: usize,
) -> crate::engine::SegmentOutcome {
    let segments: Vec<(String, Option<bool>)> = split_into_segments(&line);
    // INT-201 ownership: owned by the current command line execution.
    // Tracks &&/|| chaining state and is reset for each input line, so it is
    // deliberately NOT engine state -- it never outlives one line.
    let mut prev_op: Option<bool> = None;
    let segment_count = segments.len();
    if segment_count > 2 {
        ui_line(format!(
            "  {} {} commands",
            "○".bright_cyan(),
            segment_count
        ));
    }
    for (seg_idx, (segment, op)) in segments.iter().enumerate() {
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
        if engine.chain_skips(prev_op) {
            prev_op = *op;
            continue;
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
                || (_snap_tok == "git" && (line.contains(" push") || line.contains(" reset")));
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
                                println!("  {} usage: flow focus INT-NNN", "\u{2717}".bright_red());
                            } else if !arg.starts_with("INT-") {
                                println!("  {} must be INT-NNN format", "\u{2717}".bright_red());
                            } else {
                                if let Err(e) = fdb.set_focus_intent(arg) {
                                    eprintln!("warning: failed to set focus intent: {}", e);
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
                                eprintln!("warning: failed to clear focus intent: {}", e);
                            }
                            println!("  {} focus cleared", "\u{25cb}".dimmed());
                        }
                        "status" | "" => match fdb.get_focus_intent() {
                            Some(intent) => {
                                println!();
                                println!(
                                    "  {} {}",
                                    "Active focus:".dimmed(),
                                    intent.bright_green().bold()
                                );
                                println!("  {} flow clear  to release", "hint:".dimmed());
                                println!();
                            }
                            None => {
                                println!(
                                    "  {} no active focus -- use: flow focus INT-NNN",
                                    "\u{25cb}".dimmed()
                                );
                            }
                        },
                        _ => {
                            println!("  {} unknown subcommand: {}", "\u{2717}".bright_red(), sub);
                            println!("  usage: flow | flow focus INT-NNN | flow clear");
                        }
                    }
                }
                // INT-169: record the status rather than leaving the PREVIOUS command's.
                // the flow command completed. A stale code here is invisible today, but `&&`
                // is about to read this value to decide whether the next part runs.
                engine.set_last_exit(Some(0));
                continue;
            }
        }

        // INT-220 Gate 11 -- friday dismiss: negative learning
        if let Some(outcome) = engine.try_friday_dismiss(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        // INT-203 fix -- route friday subcommands to core friday
        if let Some(outcome) = engine.try_friday_subcommand(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        // INT-220 -- friday <question>: ask Friday about the forest
        // INT-342: db-browse -- launch state.db TUI browser
        if let Some(outcome) = engine.try_db_browse(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        // INT-279 FQL: friday where/show/explain/recall direct queries
        if let Some(outcome) = engine.try_friday_query(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        // INT-278 -- friday chat: launch Friday Chat TUI (intercept first)
        if let Some(outcome) = engine.try_friday_chat(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        if let Some(outcome) = engine.try_friday_ask(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }

        // Natural language ?prefix
        if let Some(outcome) = engine.try_nl_query(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }

        // Execute
        // `trimmed` moved into the engine with the assignment body, but the spine-exec
        // debug entry below still wants it. A one-line derivation is cheaper to repeat
        // than to thread back out, and it cannot drift from what the method computes.
        let trimmed = line.trim();
        if let Some(outcome) = engine.try_let(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        if let Some(outcome) = engine.try_export(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }

        if let Some(outcome) = engine.try_unset(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        // persist VAR — save variable to state.db for cross-session persistence
        if let Some(outcome) = engine.try_persist(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
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
        if let Some(outcome) = engine.try_spine_exec(trimmed) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }

        // INT-285 BUG 2 FIX: shell control structures bypass fsh expansion
        // for/while/until/if/case go to sh with variables unexpanded
        // INT-195: canonical, quote-aware derivation. Bound first because a String
        // will not match &str literal patterns inside matches!.
        if let Some(outcome) = engine.try_shell_construct(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        // BUG-298-2: heredoc — route << blocks to sh -c before
        // alias expansion or any other processing touches the line.
        if let Some(outcome) = engine.try_heredoc(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
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
        let line = engine.expand_aliases(line);
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
        // INT-169: DEFAULT ON, and INT-201 (2026-08-05) settled what the variable MEANS. It is not a
        // fallback shell and cannot be one: the inline redirect and pipeline executors were deleted
        // once the spine claimed every form of both, so legacy no longer implements them. FSH_SPINE=0
        // is a MIGRATION AID -- a way to compare routing -- and lines legacy cannot run are refused
        // with a message naming what is missing. Generation rollback is the real escape hatch.
        // Flipped once the evidence stopped improving from testing: 107/107 through the
        // router, the migration audit at zero unexpected and zero feature gaps, and the
        // counters showing the spine claims what it owns and declines what it refuses.
        // What remains is answerable only by real use, which needs the default to be on.
        // ⚠️ REPL-STATE COMMANDS ARE NOT ROUTABLE. `jobs`, `fg <n>` and `kill %n` read and mutate
        // the JobTable that lives in this loop, which spine dispatch has no path to -- so the
        // spine can PARSE them and can never RUN them. Excluded before the attempt rather than
        // handled inside it: the router's contract is that a claim means ownership.
        match engine.route_through_spine(line, &mut job_table) {
            crate::engine::RouteOutcome::Handled => continue,
            crate::engine::RouteOutcome::ExitShell => {
                return crate::engine::SegmentOutcome::ExitShell
            }
            crate::engine::RouteOutcome::Declined => {}
        }
        // INT-169 INCREMENT 3: THE ASSIGNMENT PREFIX MOVED BELOW THE ROUTER.
        //
        // This ran at the TOP of the loop, so `VAR=x cmd` never reached the spine -- the last
        // demonstrated route to legacy on this build. The spine now owns the common form and puts
        // the value on the CHILD, so nothing global is mutated and nothing needs restoring.
        //
        // THE HANDLER STAYS, AND IS NOT A DUPLICATE. The spine deliberately refuses two shapes and
        // legacy is the right owner of both: a BARE `FOO=1`, which persists for the session and
        // describes no process, and a value expanding to several words, which bash keeps as a
        // literal pattern. Deleting this would break the first and silently change the second.
        //
        // EXACTLY ONE BEHAVIOUR CHANGES, and it was ruled before the move rather than discovered
        // after it: `FOO=1 echo $FOO` now prints the OLD value, matching bash. Expansion happens
        // in the shell before the child exists, which is what a prefix assignment means. Legacy
        // set the variables first and expanded afterwards, so it printed the new one.
        //
        // AN ALIAS CHANGE WAS EXPECTED HERE AND DOES NOT HAPPEN, which is worth recording because
        // the reasoning that predicted it was wrong. `expand_aliases` looks up
        // `command_word(line)`, and for `FOO=1 ll` that word is `FOO=1` -- so the prefix hides the
        // alias from the LOOKUP, not from the routing. Measured on gen 484 and on this tree:
        // unexpanded on both. fsh therefore still diverges from bash here, for a reason that has
        // nothing to do with where this handler sits, and closing it is its own question.
        if let Some(outcome) = engine.try_inline_assignment(line, &raw_line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        // INT-201: the expansion pipeline moved to the engine; the REPL keeps the
        // continue, because a failglob refusal is control flow and this loop owns it.
        let line = match engine.expand_line(line) {
            Some(l) => l,
            None => continue,
        };
        let line = line.as_str();

        // INT-265: forest/query pipelines. Moved into the engine 2026-08-05 (INT-201) --
        // dispatch deliberately unchanged; the REPL still asks, the engine now answers.
        if let Some(outcome) = engine.try_query_executor(line) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
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

        // A TRAILING `&` CANNOT SURVIVE detect_redirect. It splits at the last unquoted `>`
        // and takes everything right of it as the TARGET, so `cmd > f &` yields a target of
        // `f &` -- a file whose NAME ends in an ampersand. The stripped line then no longer
        // ends with ` &`, so try_background never fires: the command runs in the FOREGROUND,
        // registers no job, writes to the junk filename, and reports nothing. Four wrong
        // things, all silent. Same family as the quoted-`>` bug that created a file named `b"`.
        //
        // REFUSE RATHER THAN GUESS. Fixing it here would mean teaching the legacy background
        // path to apply redirects, duplicating configure_file_io -- the second owner that
        // background_command's doc exists to prevent. The real fix is the spine backgrounding
        // a pipeline, which deletes the decline that sends these lines here at all.
        //
        // SAFE BY CONSTRUCTION: the spine's background attempt sits ~200 lines above and
        // `continue 'segments` on success, so this can only ever see a line the spine declined.
        if let Some((ref t, _)) = redirect_info {
            if t.trim_end().ends_with('&') {
                eprintln!(
                                "{} backgrounding a redirected command is not supported here -- the `&` was absorbed into the redirect target ({:?})",
                                "x".bright_red(),
                                t
                            );
                eprintln!("  the spine handles `cmd > file &`; a backgrounded PIPELINE is not supported yet");
                engine.set_last_exit(Some(1));
                continue;
            }
        }
        // EXECUTOR (a) WAS HERE -- two hundred and one lines of file opening, append handling, a
        // hundred-and-nineteen-line match over the opened file, and INT-172's sh delegation for `2>`.
        // The spine claims every redirect form probed with the router trace on: `> f`, `>> f`, `2> f`,
        // `2>&1`, `> f 2>&1`, `< f`, and a pipe into a file. `echo a >` is reported by the spine itself
        // since 23a6e306. So under default routing nothing arrives here.
        //
        // ⚠️ AND IT TOOK A DEAD PARAMETER WITH IT. This block ended in an unconditional
        // `continue 'segments`, so the `redirect` handed to execute_and_record was permanently None and
        // the write hoisted into it never ran. Deleting the block is what makes that visible.
        //
        // FSH_SPINE=0 still arrives, and says what it does not implement rather than half-doing it --
        // the migration-aid contract, same as pipelines.
        if redirect_info.is_some() {
            engine.refuse_unimplemented("redirects");
            continue;
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

        // A BACKGROUNDED PIPELINE IS REFUSED, NOT HALF-RUN. The spine claims `cmd &` and runs it
        // correctly, but try_spine_background_command returns None for a MULTI-STAGE pipeline --
        // "a pipeline cannot be backgrounded yet: it needs every stage spawned and only the LAST
        // child registered, which is a different executor" -- and AstNode::Background refuses at
        // both lowering entries. So the line lands here, where the pipeline executor splits on
        // " | " and hands the final stage its arguments with the `&` still attached.
        //
        // Measured 2026-08-05 on the deployed shell: `echo hi | cat &` printed
        // `cat: '&': No such file or directory`, ran in the FOREGROUND, and registered no job,
        // while `sleep 4 &` was claimed by the spine and registered correctly. The boundary is
        // exactly "the spine will not background a pipeline".
        //
        // Refusing trades a confusing half-result for a stated limitation. The repair is the
        // spine learning to background a pipeline -- spawn every stage, register only the last
        // child -- which deletes both this branch and the decline that routes here.
        //
        // SAFE BY CONSTRUCTION: has_pipe is already `!in_quotes && contains(" | ")`, so
        // `echo "a | b" &` cannot reach it, and anything the spine CLAIMED left the segment
        // loop some four hundred lines above.
        if has_pipe && line.trim_end().ends_with(" &") {
            eprintln!(
                            "{} backgrounding a pipeline is not supported yet -- the `&` would reach the last stage as an argument",
                            "x".bright_red()
                        );
            eprintln!("  the spine backgrounds a single command (`cmd &`); a PIPELINE cannot be backgrounded yet");
            engine.set_last_exit(Some(1));
            continue;
        }
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
                    Ok(status) => Some(crate::exit_status_code(&status)),
                    Err(_) => Some(1),
                },
            );
            continue;
        }
        // EXECUTOR (b) WAS HERE -- two hundred and forty-seven lines that built a child chain by
        // splitting the raw line on " | " and wiring the pipes by hand. The spine owns pipelines now:
        // two and a half thousand of them in the migration audit, every form probed with the router
        // trace was claimed, and backgrounded pipelines joined them at bb4adb88.
        //
        // SO THIS POINT IS UNREACHABLE UNDER DEFAULT ROUTING -- a claimed line left the segment loop
        // four hundred lines above. Only FSH_SPINE=0 arrives here, and that variable is a MIGRATION
        // AID rather than a fallback shell: its job is to let you compare routing, not to be a second
        // implementation of the shell. So legacy names what it no longer implements instead of
        // splitting text and hoping, which is how `echo hi | cat &` came to hand `cat` an ampersand.
        //
        // ⚠️ THE CONDITION IS EXACTLY THE ONE THE EXECUTOR USED. execute_and_record still receives
        // pipeline_ops and has_external_op, so refusing on any pipe would take work that still runs.
        if has_external_op {
            engine.refuse_unimplemented("pipelines");
            continue;
        }
        let base_cmd = if has_pipe {
            line.split(" | ").next().unwrap_or(line).to_string()
        } else {
            line.to_string()
        };

        // Phase 9: a `| watch` pipeline streams until interrupted. INT-201 moved the
        // body into the engine; the caller keeps the continue.
        if let Some(outcome) = engine.try_streaming(&base_cmd, &pipeline_ops) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }

        // Phase 8 — Job control commands
        // INT-195: canonical, quote-aware derivation.
        if let Some(outcome) = engine.try_jobs(line, Some(&mut job_table)) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        if let Some(outcome) = engine.try_fg(line, Some(&mut job_table)) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }
        if let Some(outcome) = engine.try_kill(line, Some(&mut job_table)) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }

        // Phase 8 — Background job: detect trailing &
        if let Some(outcome) = engine.try_background(line, Some(&mut job_table)) {
            match outcome {
                crate::engine::SegmentOutcome::Next => continue,
                crate::engine::SegmentOutcome::ExitShell => {
                    return crate::engine::SegmentOutcome::ExitShell
                }
            }
        }

        // Phase 13 — Redirection: already done early, use redirect_early
        let line = line;
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
                    let right_result = commands::execute(&table, engine.db(), engine.core_root());
                    if let commands::CommandResult::Value(value::Value::Table(rows)) = right_result
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
                Ok(status) => Some(crate::exit_status_code(&status)),
                // sh could not be launched. Same reasoning as the pipeline arms below:
                // leaving the code untouched recreates the stale-state bug.
                Err(_) => Some(1),
            });
            continue;
        }
        // BUG-298-1: expand tilde in base_cmd before dispatch.
        let base_cmd = expand_tilde(&base_cmd);
        // INT-201: per-execution work. Advisories stay BELOW, in their current order.
        let outcome = engine::execute_and_record(
            &mut engine,
            &raw_line,
            &base_cmd,
            original_line,
            &pipeline_ops,
            has_external_op,
            is_fm_cmd,
            &fm_cwd_file,
        );
        if outcome == engine::SegmentOutcome::ExitShell {
            return crate::engine::SegmentOutcome::ExitShell;
        }
        // INT-194 — Prediction-aware suggestions (pattern detection)
        friday_next_cmd_hint(&engine, &base_cmd, &mut shown_friday_suggestions);
        // INT-296 Phase 5 (pre-migration citation): consecutive failure detection.
        friday_failure_hint(&engine, &base_cmd, &mut shown_friday_suggestions);
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
        friday_daemon_event(
            &mut engine,
            &base_cmd,
            health,
            &mut shown_friday_suggestions,
            &mut last_friday_intent,
        );
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
        friday_proactive_message(&engine, _session_commands);
    } // end 'segments loop
    crate::engine::SegmentOutcome::Next
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
    // ⚠️ INT-204: SAY IT WHEN THE DATABASE IS NOT THE CANONICAL ONE. FAELIGHT_STATE_DB exists so the
    // test harness can give each run its own database instead of borrowing the user's, but a variable
    // that redirects the database is the more dangerous cousin of the one that started this intent --
    // FSH_CONFIG leaked out of an inline assignment into a child process and silently changed
    // behaviour. A leak here costs history, aliases and session memory, and would look like amnesia
    // rather than a misconfiguration.
    //
    // So it is announced rather than resolved quietly. This compares the RESOLVED path against the
    // default instead of reading the variable again: the decision keeps one owner in paths.rs, and
    // this message cannot drift out of agreement with it.
    {
        let resolved = faelight_core::paths::state_db();
        if resolved != faelight_core::paths::runtime_dir().join("state.db") {
            eprintln!(
                "  {} using a NON-CANONICAL database: {}",
                colored::Colorize::bright_yellow("!"),
                resolved.display()
            );
            eprintln!(
                "    history, aliases and session memory come from there, not your usual one."
            );
            eprintln!("    unset FAELIGHT_STATE_DB to go back.");
        }
    }
    let core_root = db.core_root();
    // Start in ~/0-core by default. INT-201 2026-08-07: this call was here twice, identically,
    // with the comment between the two copies -- one was dead and is deleted.
    //
    // NOTE this OVERRIDES the directory fsh was spawned in, and so does the last_dir restore near
    // the end of the banner. Both are deliberate ("keep work in forest home"), but there is no way
    // to opt out, so a harness that spawns fsh with a chosen working directory cannot make it stick.
    // INT-206 fixed that: FSH_KEEP_CWD suppresses this and the last_dir restore below, so a caller
    // that chose a working directory keeps it. Unset -- which is every interactive session -- the
    // forest-home default is exactly as it was.
    if !keep_launch_cwd() {
        let _ = std::env::set_current_dir(&core_root);
    }

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
    // INT-134 Lane UX: `set edit_mode = vi` in config.fsh selects modal editing. Emacs stays the
    // default -- and note that "emacs" here means READLINE KEYBINDINGS (Ctrl+A, Ctrl+E, Ctrl+K,
    // Ctrl+W), not the editor. Nothing is installed and nothing is required; it is what this shell
    // has always done and what every readline shell does unless told otherwise.
    //
    // A TYPO MUST NOT BE SILENT. An unrecognised value warns and falls back rather than quietly
    // doing nothing -- a setting that accepts anything and honours only some values is a trap, and
    // the user would have no way to tell `edit_mode = vim` from `edit_mode = vi` failing.
    let edit_mode = match cfg
        .settings
        .iter()
        .find(|(k, _)| k == "edit_mode")
        .map(|(_, v)| v.trim().to_lowercase())
    {
        None => EditMode::Emacs,
        Some(v) if v == "vi" || v == "vim" => EditMode::Vi,
        Some(v) if v == "emacs" => EditMode::Emacs,
        Some(other) => {
            eprintln!(
                "  config: edit_mode '{}' is not recognised -- using emacs (valid: vi, emacs)",
                other
            );
            EditMode::Emacs
        }
    };
    let rl_config = Config::builder()
        .max_history_size(10000)?
        .history_ignore_dups(true)?
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .completion_show_all_if_ambiguous(true)
        .edit_mode(edit_mode)
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
                // INT-209: THE COMMENT PRE-PASS IS GONE. The canonical scanner recognises a comment
                // as a lexical state, so stripping here made two owners implement one rule -- and
                // this one ran FIRST, which is why both doors agreed for the wrong reason: not
                // because the scanner governed both, but because a pre-pass beat it to the REPL.
                //
                // Measured before removing: a probe on this line fired for `echo hi # tail` and for
                // a comment-only line, so it was live rather than already-dead code.
                let line = line.trim().to_string();
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
                // INT-196 M1-M4: THE GUARD WORD COMES FROM THE PARSER, and the fallback goes
                // ONE LAYER DOWN THE SAME PIPELINE rather than back to source heuristics.
                //   Complete            -> the first Word of the parsed Command
                //   Incomplete/Refused  -> the scanner first token, which is still parser-owned
                //   Invalid             -> no word, and therefore no guard decision
                //
                // M1: this is the only parse performed for the guard. The router parses again at
                // main.rs:1329, and that is NOT this parse -- it runs later, inside run_input, on
                // one alias-expanded SEGMENT rather than on the whole raw line.
                // INT-197: THE GUARD JUDGES THE EXPANDED LINE, not the line as typed.
                //
                // The gap this closes: an alias whose expansion is a gated command was not gated.
                // The word derived from `chx -R /etc/x` is chx, which matches no deny entry, no
                // allow entry and no safe entry -- so the guard returned None and the executor then
                // expanded it to a recursive chmod on /etc and ran it.
                //
                // THE CALL DOES NOT MOVE, which the intent warned against. Alias expansion lives in
                // run_input, called AFTER this, so moving the guard there would put it below the
                // multi-line branch and leave a pasted block unguarded -- a property INT-196 M8
                // proved holds today. Instead the guard expands a COPY here and judges that.
                //
                // ⚠️ STATED LIMITATION: COMPOUND COMMANDS ARE NOT PER-SEGMENT EXPANDED.
                // expand_aliases resolves the first command word of the string it is given, and
                // run_input later processes segments independently. So in `a && zap`, the alias zap
                // is OUTSIDE this gate guarantee. Closing that means the guard enumerating segments
                // itself, which would put a SECOND command-segmentation path inside the security
                // boundary, able to disagree with the executor. That is its own intent, and it
                // needs a ruling on who owns segment enumeration -- not a wider input string here.
                let guard_line = engine.expand_aliases(&line);
                let guard_word = guard_command_word(&guard_line).map(|w| policy_identity(&w));
                // INT-196 M6: RECORD WHAT THE UNIVERSAL GUARD JUDGED, so the heredoc site below can
                // assert it is asking about the SAME string. Reading the control flow says `line` is
                // not rebound between the two; this makes the shell say so at runtime, under the pty
                // suite, which is the evidence the gate asks for.
                let m6_universal_line = line.clone();
                let m6_universal_word = guard_word.clone();
                // INT-246: safety_guard -- check BEFORE any execution path
                if let Some(word) = guard_word.as_deref() {
                    if let Some(warning) = safety_guard::check(&line, word) {
                        if !safety_guard::challenge_gate(&warning) {
                            engine.set_last_exit(Some(1));
                            continue 'repl;
                        }
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
                // INT-196: ASK THE OWNER, DO NOT RE-DERIVE. This tested the raw string for the
                // pattern, then split the raw string to name a delimiter, then checked quoting with
                // starts_with -- three derivations of one question that find_heredoc_intro already
                // answers, and answers quote-aware since the scanner fix.
                //
                // THE COST WAS VISIBLE, not theoretical. A quoted pair inside an ordinary argument
                // matched the raw test, so the branch fired on a line with no heredoc at all: it
                // printed a tip naming a delimiter cut out of quoted text, and it routed the command
                // through pty_exec instead of the normal path. It worked by luck rather than by
                // design.
                if let Some((delimiter, is_quoted)) = crate::expand::find_heredoc_intro(&line) {
                    // Warn if delimiter is unquoted -- sh will expand backticks
                    if !is_quoted && !delimiter.is_empty() {
                        println!(
                            "  {} heredoc tip: use << '{}'  to prevent backtick expansion",
                            "💡".normal(),
                            delimiter
                        );
                    }
                    // INT-249b/Path-3: run the heredoc via PTY so we get colored output
                    // AND the chance to scan each line for delimiter-leak warnings.
                    //
                    // INT-196: the word is derived by the SAME helper the universal guard uses, so
                    // there is one derivation rule rather than two. M6 asks whether this call is
                    // redundant at all -- `line` is not reassigned between the universal guard and
                    // here -- and that gate stays OPEN deliberately. An attempt to delete it was
                    // reverted once already because both verification probes were unrunnable, and
                    // reading the control flow is not the evidence the gate asks for.
                    // INT-196 M6: A TRIPWIRE ON A PATH PROVEN UNREACHABLE, kept deliberately.
                    // Four probes and a static check showed no genuine heredoc reaches here -- the
                    // multi-line branch above takes them all. These assertions cost nothing in
                    // release, and if anything ever DOES arrive they fail loudly rather than
                    // silently guarding a line the universal guard never saw.
                    debug_assert_eq!(
                        m6_universal_line, line,
                        "M6: the heredoc site sees a DIFFERENT line than the universal guard judged"
                    );
                    debug_assert_eq!(
                        m6_universal_word,
                        guard_command_word(&line),
                        "M6: the heredoc site derives a DIFFERENT word than the universal guard did"
                    );
                    if let Some(warning) = guard_command_word(&line)
                        .as_deref()
                        .and_then(|w| safety_guard::check(&line, w))
                    {
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
                match run_input(
                    &mut engine,
                    &line,
                    &mut job_table,
                    &mut shown_friday_suggestions,
                    &mut last_friday_intent,
                    _session_commands,
                ) {
                    crate::engine::SegmentOutcome::ExitShell => break 'repl,
                    crate::engine::SegmentOutcome::Next => {}
                }
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
        // INT-206: the restore is skipped entirely when the caller chose the directory. Guarding
        // the whole block rather than the set_current_dir inside it, because the fallback below is
        // itself an override -- an unusable last_dir sends the shell to the forest home, which is
        // exactly what a harness asking for /tmp does not want either.
        if let Some(ref last_dir) = mem.last_dir.as_ref().filter(|_| !keep_launch_cwd()) {
            let path = std::path::Path::new(last_dir.as_str());
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

/// Friday's consecutive-failure hint -- ADVISORY, interactive-only.
///
/// ⚠️ INT-201: lifted out of the postexec tail so its inputs are STATED rather than ambient.
/// The dedupe set is session state and belongs to the REPL, not to execution -- a `-c` caller
/// must never reach this. The INT-296 citation is pre-migration and is kept verbatim.
fn friday_failure_hint(
    engine: &engine::Engine,
    base_cmd: &str,
    shown: &mut std::collections::HashSet<String>,
) {
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
        if !shown.contains(&fail_key) {
            shown.insert(fail_key);
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

/// Friday's next-command prediction hint -- ADVISORY, interactive-only.
/// After each command, check if there is a strong "next command" pattern
///
/// ⚠️ INT-201: lifted out of the postexec tail so its inputs are STATED rather than ambient.
/// The dedupe set is session state owned by the REPL -- a `-c` caller must never reach this.
fn friday_next_cmd_hint(
    engine: &engine::Engine,
    base_cmd: &str,
    shown: &mut std::collections::HashSet<String>,
) {
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
                let hints_enabled = engine
                    .db()
                    .conn
                    .query_row(
                        "SELECT value FROM shell_state WHERE key='config.friday_hints'",
                        [],
                        |r| r.get::<_, String>(0),
                    )
                    .unwrap_or_else(|_| "on".to_string());
                if hints_enabled != "off" {
                    // INT-246: deduplicate hints -- only show once per session
                    let hint_key = format!("hint_{}", suggestion);
                    if !shown.contains(&hint_key) {
                        shown.insert(hint_key);
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

/// Friday's every-tenth-command proactive message -- ADVISORY, interactive-only.
///
/// INT-203 Phase 2 + INT-277: Friday proactive message with attention scoring
///
/// ⚠️ INT-201: lifted out of the postexec tail. The counter is SESSION state owned by the REPL,
/// and a `-c` caller has no session to count -- so this must never run for one.
fn friday_proactive_message(engine: &engine::Engine, session_commands: usize) {
    if session_commands % 10 == 0 && session_commands > 0 {
        let pattern: Option<(String, String, f64)> = engine.db().conn.query_row(
            "SELECT trigger, action, confidence FROM friday_patterns WHERE confidence >= 0.7 ORDER BY confidence DESC LIMIT 1",
            [], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?))
        ).ok();
        if let Some((trigger, action, conf)) = pattern {
            // INT-277: compute attention score before speaking
            let seen_count: i64 = engine
                .db()
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM friday_attention WHERE event_type = 'pattern_match'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);
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
            let attention_score =
                (novelty * risk * strategic_relevance * uncertainty * temporal_pressure).powf(0.2);
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
}

/// INT-220 -- Send FridayEvent to daemon socket (fire and forget)
///
/// Returns `true` only when the caller must skip the rest of the segment. A THROTTLED MESSAGE
/// DOES NOT: it stops printing and nothing else. Until INT-201 this returned early from four
/// levels inside the reply handler, which abandoned the forest-insights display and the periodic
/// session message for that segment -- a quiet Friday silently cost you the rest of postexec.
fn friday_daemon_event(
    engine: &mut engine::Engine,
    base_cmd: &str,
    health: Option<i64>,
    shown: &mut std::collections::HashSet<String>,
    last_intent: &mut Option<String>,
) {
    let cmd_str = base_cmd;
    // Read exit status from cache file written above
    let exit_code: i32 = {
        let cache_dir = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".cache/faelight");
        let status =
            std::fs::read_to_string(cache_dir.join("last-exit-status")).unwrap_or_default();
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
        if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(sock_path) {
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
            if reader.read_line(&mut resp).is_ok() && resp.contains("FridaySpeak") {
                if (resp.contains("\"low\"")
                    || resp.contains("\"medium\"")
                    || resp.contains("\"high\""))
                    && resp.contains("\"message\":\"")
                {
                    if let Some(msg) = resp.split("\"message\":\"").nth(1) {
                        if let Some(msg) = msg.split('"').next() {
                            if !msg.is_empty() && msg != "null" {
                                // INT-246: once per intent -- only speak when intent changed
                                let current_intent =
                                    engine.db().get_focus_intent().map(|i| format!("{}", i));
                                if current_intent == *last_intent && last_intent.is_some() {
                                    return; // throttled: stay quiet, but let postexec run
                                }
                                // INT-246: never repeat same suggestion in a session
                                if shown.contains(msg) {
                                    return; // throttled: stay quiet, but let postexec run
                                }
                                shown.insert(msg.to_string());
                                *last_intent = current_intent;
                                println!();
                                let tier = if resp.contains("\"high\"") {
                                    ("RECOMMEND", "78%")
                                } else if resp.contains("\"medium\"") {
                                    ("SUGGEST", "62%")
                                } else {
                                    ("SUGGEST", "54%")
                                };
                                println!("  🌲 Friday: {}  ·  {} · {}", msg, tier.0, tier.1);
                            }
                        }
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod brace_expansion_tests {
    use super::expand_braces;

    #[test]
    fn letter_and_number_ranges_still_expand() {
        assert_eq!(expand_braces("{a..e}"), "a b c d e");
        assert_eq!(expand_braces("{1..4}"), "1 2 3 4");
        assert_eq!(expand_braces("pre {a..c} post"), "pre a b c post");
    }

    #[test]
    fn a_space_is_not_a_range_endpoint() {
        // INT-203: this is the exact text that was being eaten -- a Rust match pattern.
        let pattern = "Executed { .. } => continue";
        assert_eq!(
            expand_braces(pattern),
            pattern,
            "brace group with spaces must stay literal"
        );
    }

    #[test]
    fn punctuation_and_mixed_kinds_stay_literal() {
        for s in ["{. ..}", "{1..a}", "{a..1}", "Foo { ..default}"] {
            assert_eq!(expand_braces(s), s, "should not expand: {s}");
        }
    }
}

/// INT-196 SITE 1: is_repl_state_command derives the command word quote-aware and then reads its
/// SECOND word from the raw line with split_whitespace. These cases measure whether that matters.
///
/// The predicate decides ROUTING EXCLUSION -- engine.rs asks it four times, including the spine
/// router at 977 -- so a wrong answer routes a line that should have been excluded, or excludes one
/// that should have routed.
#[cfg(test)]
mod repl_state_command_tests {
    use super::is_repl_state_command;

    #[test]
    fn bare_jobs_is_repl_state() {
        assert!(is_repl_state_command("jobs"));
    }

    #[test]
    fn a_job_spec_kill_is_repl_state() {
        assert!(is_repl_state_command("kill %1"));
    }

    #[test]
    fn a_pid_kill_is_not_repl_state() {
        assert!(
            !is_repl_state_command("kill 1234"),
            "INT-095: a PID belongs to the real kill"
        );
    }

    #[test]
    fn fg_with_a_job_id_is_repl_state() {
        assert!(is_repl_state_command("fg 1"));
    }

    #[test]
    fn fg_commit_is_not_repl_state() {
        assert!(
            !is_repl_state_command("fg commit"),
            "fg commit must reach the alias"
        );
    }

    /// THE QUESTION. bash treats a quoted job spec as a job spec. Here the second word is read from
    /// the raw line, so it arrives with the quote still attached and the check fails.
    #[test]
    fn a_quoted_job_spec_kill_is_still_repl_state() {
        assert!(is_repl_state_command("kill \"%1\""));
    }

    /// Same question for fg, where the second word must parse as a number.
    #[test]
    fn a_quoted_fg_job_id_is_still_repl_state() {
        assert!(is_repl_state_command("fg \"1\""));
    }
}

/// INT-196 M10: the measured cost of the guard parse, stated as a number.
///
/// Measured HERE rather than at the REPL, because instrumentation in the loop is invisible to the
/// dash-c door and a single keystroke is not a repeatable sample. This calls the same function the
/// guard calls, on shapes taken from real use.
#[cfg(test)]
mod guard_cost {
    #[test]
    fn measure_guard_parse_cost() {
        let lines = [
            "echo one",
            "ls -la /tmp",
            "git status --short",
            "cargo build -p faelight-shell",
            "grep -rn pattern src/",
        ];
        let iters = 2000;
        let t0 = std::time::Instant::now();
        let mut sink = 0usize;
        for _ in 0..iters {
            for l in lines.iter() {
                if let Some(w) = super::guard_command_word(l) {
                    sink += w.len();
                }
            }
        }
        let total = t0.elapsed();
        let calls = iters * lines.len();
        let per = total.as_nanos() as f64 / calls as f64;
        println!(
            "GUARD PARSE COST: {calls} calls, {per:.0} ns each, {total:?} total (sink {sink})"
        );
        assert!(sink > 0, "the calls must have produced words");
    }
}
