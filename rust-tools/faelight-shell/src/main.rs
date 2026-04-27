// faelight-shell v0.1.0
// Forest-native structured shell environment
// INT-120 Phase 1 — REPL skeleton
//
// "A forest deserves a shell that knows it is a forest."
// "Not text streams. Not configuration. Structured wisdom."

mod commands;
mod db;
mod error;
mod exec;
mod output;
mod registry;
#[cfg(test)]
mod tests;
use colored::Colorize;
mod completion;
mod config;
mod digest;
mod jobs;
mod nl;
mod prompt;
mod schema;
mod scripting;
mod session;
mod triggers;
mod value;

use anyhow::Result;
use chrono::{Datelike, Timelike};
use rustyline::{error::ReadlineError, CompletionType, Config, EditMode, Editor};
use std::collections::HashMap;

/// Split a line on `;` separators, respecting quoted strings.
/// "cmd1; cmd2; cmd3" → ["cmd1", "cmd2", "cmd3"]
fn normalize_input(s: &str) -> String {
    s.replace("‘", "'")
        .replace("’", "'")
        .replace("“", "\"")
        .replace("”", "\"")
        .replace("–", "-")
        .replace("—", "--")
}

fn expand_subshells(line: &str) -> String {
    let trigger: &str = &('$'.to_string() + "(");
    if !line.contains(trigger) {
        return line.to_string();
    }
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '(' {
            i += 2;
            let mut depth = 1usize;
            let mut inner = String::new();
            while i < chars.len() && depth > 0 {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                }
                if depth > 0 {
                    inner.push(chars[i]);
                }
                i += 1;
            }
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&inner)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            result.push_str(&output);
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

fn split_semicolons(line: &str) -> Vec<String> {
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
                    segments.push(seg);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let seg = current.trim().to_string();
    if !seg.is_empty() {
        segments.push(seg);
    }
    if segments.is_empty() {
        segments.push(line.trim().to_string());
    }
    segments
}

/// Split a line on && and || operators (respecting quotes)
/// Returns Vec<(cmd, operator)> where operator is None for last cmd,
/// Some(true) for && (run next if success), Some(false) for || (run next if fail)
fn split_logical(line: &str) -> Vec<(String, Option<bool>)> {
    let mut result = vec![];
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
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
            '&' if !in_quote && i + 1 < chars.len() && chars[i + 1] == '&' => {
                let seg = current.trim().to_string();
                if !seg.is_empty() {
                    result.push((seg, Some(true)));
                }
                current.clear();
                i += 2; // skip &&
                continue;
            }
            '|' if !in_quote && i + 1 < chars.len() && chars[i + 1] == '|' => {
                let seg = current.trim().to_string();
                if !seg.is_empty() {
                    result.push((seg, Some(false)));
                }
                current.clear();
                i += 2; // skip ||
                continue;
            }
            _ => current.push(ch),
        }
        i += 1;
    }
    let seg = current.trim().to_string();
    if !seg.is_empty() {
        result.push((seg, None));
    }
    if result.is_empty() {
        result.push((line.trim().to_string(), None));
    }
    result
}

/// Detect and strip redirection from a command line.
/// Returns (cleaned_line, Some((path, append))) or (line, None)
fn detect_redirect(line: &str) -> (String, Option<(String, bool)>) {
    // Match 2>/dev/null and 2>file FIRST
    if line.contains(" 2>/dev/null")
        || line.contains(" 2>&1")
        || (line.contains(" 2>") && !line.contains(" 2>="))
    {
        // Return the line as-is but signal that it needs special handling
        // The caller will handle 2> patterns natively
        return (line.to_string(), Some(("__stderr__".to_string(), false)));
    }
    // Match >> before > (order matters)
    if let Some(idx) = line.rfind(" >> ") {
        let path = line[idx + 4..].trim().to_string();
        // Only treat as redirect if path looks like a file (not a number/comparison)
        if !path.is_empty()
            && !path
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            let cmd = line[..idx].trim().to_string();
            return (cmd, Some((path, true)));
        }
    }
    if let Some(idx) = line.rfind(" > ") {
        let path = line[idx + 3..].trim().to_string();
        // Only treat as redirect if:
        // - path is not empty
        // - path does not start with a digit (comparison like > 70)
        // - path does not start with = (>= comparison)
        // - it is not inside a pipe segment before a command
        let first_char = path.chars().next();
        let is_comparison = first_char
            .map(|c| c.is_ascii_digit() || c == '=')
            .unwrap_or(false);
        if !path.is_empty() && !is_comparison {
            let cmd = line[..idx].trim().to_string();
            return (cmd, Some((path, false)));
        }
    }
    (line.to_string(), None)
}

/// Expand $VAR and ${VAR} references in a line.
/// Reads from shell_vars first, then std::env.
fn expand_globs(line: &str) -> String {
    // Only expand if line contains * or ? outside of quotes
    if !line.contains('*') && !line.contains('?') {
        return line.to_string();
    }
    let mut result_parts: Vec<String> = vec![];
    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in parts {
        // Check if part is quoted
        if (part.starts_with('"') && part.ends_with('"'))
            || (part.starts_with('\'') && part.ends_with('\''))
        {
            result_parts.push(part.to_string());
            continue;
        }
        if part.contains('*') || part.contains('?') {
            // Expand tilde
            let expanded = if part.starts_with("~/") {
                let home = std::env::var("HOME").unwrap_or_default();
                part.replacen("~", &home, 1)
            } else {
                part.to_string()
            };
            // Use glob crate pattern matching via std::fs
            let pattern_path = std::path::Path::new(&expanded);
            let parent = {
                let p = pattern_path.parent().unwrap_or(std::path::Path::new("."));
                if p.as_os_str().is_empty() {
                    std::path::Path::new(".")
                } else {
                    p
                }
            };
            let file_pattern = pattern_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(part);
            let mut matches: Vec<String> = vec![];
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if glob_match(file_pattern, &name_str) {
                        let p = entry.path().to_string_lossy().to_string();
                        let p = p.strip_prefix("./").unwrap_or(&p).to_string();
                        matches.push(p);
                    }
                }
            }
            matches.sort();
            if matches.is_empty() {
                result_parts.push(part.to_string());
            } else {
                result_parts.extend(matches);
            }
        } else {
            result_parts.push(part.to_string());
        }
    }
    result_parts.join(" ")
}

fn glob_match(pattern: &str, name: &str) -> bool {
    // Simple glob: * matches anything, ? matches one char
    let mut pi = 0;
    let mut ni = 0;
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let mut star_pi = usize::MAX;
    let mut star_ni = 0;
    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
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
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

fn expand_vars(line: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            i += 1;
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

// Check if 0-core is locked (immutable flag set)
fn is_core_locked(core_root: &str) -> bool {
    std::process::Command::new("lsattr")
        .args(["-d", core_root])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("----i"))
        .unwrap_or(false)
}

// Strip # comments — only at start of line or after whitespace, never inside strings
fn strip_comments(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                return String::new();
            }
            let mut in_single = false;
            let mut in_double = false;
            let mut comment_pos = None;
            for (i, ch) in line.char_indices() {
                match ch {
                    '\'' if !in_double => in_single = !in_single,
                    '"' if !in_single => in_double = !in_double,
                    '#' if !in_single && !in_double => {
                        if i == 0 || line[..i].ends_with(|c: char| c.is_whitespace()) {
                            comment_pos = Some(i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            match comment_pos {
                Some(pos) => line[..pos].trim_end().to_string(),
                None => line.to_string(),
            }
        })
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() -> Result<()> {
    // Spawn REPL with 64MB stack — prevents stack overflow in deep command chains
    let result = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .name("faelight-repl".into())
        .spawn(|| repl_main())?
        .join()
        .map_err(|_| anyhow::anyhow!("REPL thread panicked"))?;
    result
}

fn repl_main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // POSIX -c flag support — required for login shell compatibility
    // niri-session does: exec bash -c "exec -l '$SHELL' -c '$0 -l $*'"
    // We must handle: faelight-shell -c "command string"
    if let Some(c_pos) = args.iter().position(|a| a == "-c") {
        if let Some(cmd_str) = args.get(c_pos + 1) {
            // Execute the command string via sh and exit
            let status = std::process::Command::new("/bin/sh")
                .args(["-c", cmd_str])
                .status()
                .unwrap_or_else(|_| std::process::exit(1));
            std::process::exit(status.code().unwrap_or(0));
        }
        std::process::exit(0);
    }

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
        // Ensure scripts/ is in PATH
        if let Ok(home) = std::env::var("HOME") {
            let scripts = format!("{}/0-core/scripts", home);
            let cargo_bin = format!("{}/.cargo/bin", home);
            let current_path = std::env::var("PATH").unwrap_or_default();
            if !current_path.contains(&scripts) {
                std::env::set_var(
                    "PATH",
                    format!("{}:{}:{}", scripts, cargo_bin, current_path),
                );
            }
        }
    }
    // Connect to state.db
    let db = db::ForestDb::open()?;
    let core_root = db.core_root();
    let _ = std::env::set_current_dir(&core_root);
    // Start in ~/0-core by default
    let _ = std::env::set_current_dir(&core_root);

    // Phase 15 — load config.fsh
    config::ensure_default();
    let cfg = config::load();

    // Print welcome
    print_welcome(&core_root, &db);
    // Write journal session-start entry
    let _ = std::process::Command::new("core")
        .args(["journal", "session-start"])
        .output();
    // INT-242: export forest state to /etc/faelight/ for login screen
    let _ = std::process::Command::new("faelight-export").output();
    let _session_start = std::time::Instant::now();
    let mut _session_commands: usize = 0;
    let mut _session_pipelines: usize = 0;
    let mut _session_deploys: usize = 0;
    let mut _session_commits: usize = 0;
    let mut _session_failed: usize = 0;

    // Phase 16 — configured interactive editor
    let rl_config = Config::builder()
        .max_history_size(10000)?
        .history_ignore_dups(true)?
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let helper = completion::ForestHelper::new(&db);
    let mut rl: Editor<completion::ForestHelper<'_>, _> = Editor::with_config(rl_config)?;
    rl.set_helper(Some(helper));
    // Ctrl+L handled in REPL loop via clear command

    // Apply config aliases and settings
    config::apply(&cfg, &db);
    // INT-233 -- validate config.fsh on load, surface errors immediately
    {
        let errors = config::validate();
        if !errors.is_empty() {
            println!("  {} config.fsh syntax errors:", "⚠️".normal());
            for e in &errors {
                println!("{}", e);
            }
        }
    }

    // INT-173 — build command registry on startup
    let mut registry = registry::Registry::new();
    registry.populate(&db, &core_root);

    // Load history from state.db
    db.load_history(&mut rl);

    // Phase 8 — job table
    let mut job_table = jobs::JobTable::new();

    // Phase 17 — prompt context tracking
    let last_duration_ms: Option<u64> = None;
    let mut last_exit_code: Option<i32> = None;

    // Phase 10 — shell variable table
    let mut shell_vars: HashMap<String, String> = HashMap::new();
    // Restore persisted variables from state.db
    {
        {
            let _ = db.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS shell_persist (key TEXT PRIMARY KEY, value TEXT NOT NULL)"
        );
            if let Ok(mut stmt) = db.conn.prepare("SELECT key, value FROM shell_persist") {
                let rows: Vec<(String, String)> = stmt
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
                    .unwrap_or_default();
                for (k, v) in rows {
                    std::env::set_var(&k, &v);
                    shell_vars.insert(k, v);
                }
            }
        }
    }

    // REPL loop
    'repl: loop {
        // Phase 8 — announce completed background jobs before prompt
        job_table.check_completed();

        // Phase 17 — render two-line context above input
        let ctx = prompt::PromptContext {
            last_duration_ms,
            last_exit_code,
            job_count: job_table.job_count(),
        };
        prompt::render_context(&db, &ctx);

        let prompt_str = prompt::render_line(&db, last_exit_code);

        match rl.readline(&prompt_str) {
            Ok(line) => {
                // Check reload signal at TOP of loop — before any processing
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
                        format!("{}/0-core/scripts/faelight-shell", home),
                        format!("{}/.cargo/bin/faelight-shell", home),
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
                    match db.get_last_command() {
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
                    match db.get_command_matching(pattern) {
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
                let line = normalize_input(&line);
                let line = normalize_input(&line);
                if let Err(e) = db.save_history_entry(&line) {
                    eprintln!("warning: failed to save history: {}", e);
                }
                let mut heredoc_handled = false;
                // Heredoc: detect << and delegate to sh with inherited stdin
                if line.contains(" << ") {
                    // Warn if delimiter is unquoted -- sh will expand backticks
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
                    let status = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&line)
                        .stdin(std::process::Stdio::inherit())
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .status();
                    let _ = status;
                    heredoc_handled = true;
                }
                if heredoc_handled {
                    continue 'repl;
                }
                // Phase 14 — multi-command: split on ; before execution
                let segments = split_semicolons(&line);
                let segment_count = segments.len();
                if segment_count > 1 {
                    println!("  {} {} commands", "○".bright_cyan(), segment_count);
                }
                for (seg_idx, segment) in segments.iter().enumerate() {
                    if segment_count > 1 {
                        println!(
                            "  {} {}",
                            format!("[{}/{}]", seg_idx + 1, segment_count).dimmed(),
                            segment.dimmed()
                        );
                    }
                    // Handle && and || logical operators
                    let logical_parts = split_logical(segment);
                    if logical_parts.len() > 1 {
                        let mut _last_success = true;
                        for (lcmd, op) in &logical_parts {
                            // Decide whether to run this command
                            let should_run = match op {
                                _ => true, // Always run first cmd
                            };
                            // For subsequent cmds, check previous result
                            let _ = should_run;
                            // Run via sh -c for now to handle complex cases
                            let status = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(lcmd)
                                .envs(std::env::vars())
                                .status();
                            _last_success = status.map(|s| s.success()).unwrap_or(false);
                            // Check if we should continue
                            if let Some(is_and) = op {
                                if *is_and && !_last_success {
                                    break;
                                } // && stops on failure
                                if !is_and && _last_success {
                                    break;
                                } // || stops on success
                            }
                        }
                        continue;
                    }
                    let line = segment.as_str();
                    // Phase 18b — Flow mode: earliest intercept
                    {
                        let ftok = line.split_whitespace().next().unwrap_or("");
                        if ftok == "flow" {
                            let sub = line.split_whitespace().nth(1).unwrap_or("");
                            let arg = line.split_whitespace().nth(2).unwrap_or("");
                            if let Ok(fdb) = crate::db::ForestDb::open() {
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
                            continue 'repl;
                        }
                    }

                    // Phase 20b — Git guardrail: block commit/push when core is locked
                    {
                        let ftok = line.split_whitespace().next().unwrap_or("");
                        let stok = line.split_whitespace().nth(1).unwrap_or("");
                        let in_core = std::env::current_dir()
                            .map(|d| d.starts_with(&core_root))
                            .unwrap_or(false);
                        if ftok == "git" && in_core && is_core_locked(&core_root) {
                            match stok {
                                "commit" | "push" | "add" | "rm" | "reset" | "rebase" | "merge" => {
                                    println!();
                                    println!(
                                        "  {} Core is LOCKED — editing blocked",
                                        "🔒".normal()
                                    );
                                    println!(
                                        "  {} No commits, pushes or changes allowed while locked",
                                        "✗".bright_red()
                                    );
                                    println!(
                                        "  {} Run: unlock-core  — then make your changes",
                                        "→".bright_cyan()
                                    );
                                    println!();
                                    continue 'repl;
                                }
                                _ => {}
                            }
                        }
                        // Also block fg commit/push when locked
                        if ftok == "fg" && in_core && is_core_locked(&core_root) {
                            match stok {
                                "commit" | "push" | "sync" => {
                                    println!();
                                    println!(
                                        "  {} Core is LOCKED — editing blocked",
                                        "🔒".normal()
                                    );
                                    println!(
                                        "  {} Run: unlock-core  — then commit",
                                        "→".bright_cyan()
                                    );
                                    println!();
                                    continue 'repl;
                                }
                                _ => {}
                            }
                        }
                    }
                    // INT-220 Gate 11 -- friday dismiss: negative learning
                    if line == "friday dismiss" || line.starts_with("friday dismiss ") {
                        let trigger = if line == "friday dismiss" {
                            "null".to_string()
                        } else {
                            format!("\"{}\"", line[15..].trim().replace('"', "'"))
                        };
                        let home_dir = std::env::var("HOME").unwrap_or_default();
                        let sock_path = format!("{}/.local/state/0-core/daemon.sock", home_dir);
                        let dismiss_json = format!(
                            r#"{{"id":3,"payload":{{"FridayDismiss":{{"pattern_trigger":{}}}}}}}"#,
                            trigger
                        );
                        if std::path::Path::new(&sock_path).exists() {
                            use std::io::{BufRead, BufReader, Write};
                            if let Ok(mut stream) =
                                std::os::unix::net::UnixStream::connect(&sock_path)
                            {
                                stream
                                    .set_write_timeout(Some(std::time::Duration::from_millis(200)))
                                    .ok();
                                stream
                                    .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                                    .ok();
                                let _ = stream.write_all(dismiss_json.as_bytes());
                                let _ = stream.write_all(b"\n");
                                let mut reader = BufReader::new(&stream);
                                let mut resp = String::new();
                                if reader.read_line(&mut resp).is_ok()
                                    && resp.contains("FridaySpeak")
                                {
                                    if let Some(msg) = resp.split("\"message\":\"").nth(1) {
                                        if let Some(msg) = msg.split('"').next() {
                                            if !msg.is_empty() && msg != "null" {
                                                println!("  \u{1f332} Friday: {}", msg);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        continue 'repl;
                    }
                    // INT-203 fix -- route friday subcommands to core friday
                    if line.starts_with("friday ") {
                        let rest = line[7..].trim();
                        let subcmds = [
                            "status",
                            "suggest",
                            "observe",
                            "extract-patterns",
                            "update-personality",
                            "seed-knowledge",
                            "learning-loop",
                            "vocabulary",
                            "propose-intent",
                            "phase2-init",
                            "phase2-status",
                            "plan",
                            "temporal-models",
                            "detect-temporal-patterns",
                            "resolve-contradictions",
                            "health-forecast",
                            "interrupt-level",
                            "cross-intent-patterns",
                            "phase2-status-full",
                        ];
                        let is_sub = subcmds.iter().any(|s| rest == *s)
                            || rest.starts_with("name-abstraction ")
                            || rest.starts_with("ask ");
                        if is_sub {
                            let mut cmd = std::process::Command::new("core");
                            cmd.arg("friday");
                            if rest.starts_with("ask ") {
                                cmd.arg("ask");
                                cmd.arg(rest[4..].trim());
                            } else {
                                for a in rest.split_whitespace() {
                                    cmd.arg(a);
                                }
                            }
                            let _ = cmd.status();
                            continue 'repl;
                        }
                    }
                    // INT-220 -- friday <question>: ask Friday about the forest
                    if line.starts_with("friday")
                        && (line == "friday" || line.starts_with("friday "))
                    {
                        let question = if line == "friday" {
                            "what should I work on next?".to_string()
                        } else {
                            line[7..].trim().to_string()
                        };
                        println!("  \u{1f332} Friday: {}", "thinking...".dimmed());
                        let home_dir = std::env::var("HOME").unwrap_or_default();
                        let sock_path = format!("{}/.local/state/0-core/daemon.sock", home_dir);
                        let q_escaped = question.replace('"', "'");
                        let query_json = format!(
                            r#"{{"id":2,"payload":{{"FridayQuery":{{"question":"{}","context":null}}}}}}"#,
                            q_escaped
                        );
                        if std::path::Path::new(&sock_path).exists() {
                            use std::io::{BufRead, BufReader, Write};
                            if let Ok(mut stream) =
                                std::os::unix::net::UnixStream::connect(&sock_path)
                            {
                                stream
                                    .set_write_timeout(Some(std::time::Duration::from_millis(500)))
                                    .ok();
                                stream
                                    .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                                    .ok();
                                let _ = stream.write_all(query_json.as_bytes());
                                let _ = stream.write_all(b"\n");
                                let mut reader = BufReader::new(&stream);
                                let mut resp = String::new();
                                if reader.read_line(&mut resp).is_ok() && !resp.is_empty() {
                                    if resp.contains("FridayAnswer") {
                                        if let Some(ans) = resp.split(r#""answer":""#).nth(1) {
                                            let ans =
                                                ans.split('"').next().unwrap_or("").to_string();
                                            println!();
                                            println!("  \u{1f332} Friday: {}", ans.bright_white());
                                            println!();
                                        }
                                    }
                                }
                            }
                        } else {
                            println!("  \u{26a0}  Friday daemon not running -- start with: faelight-daemon &");
                        }
                        continue 'repl;
                    }

                    // Natural language ?prefix
                    if line.starts_with('?') && line.len() > 1 {
                        let query = line[1..].trim();
                        // Phase 25 — auto-diagnose for complex queries
                        if nl::is_diagnostic(query) {
                            println!();
                            println!(
                                "  {} Auto-diagnosing: {}",
                                "🔍".normal(),
                                query.bright_white()
                            );
                            println!();
                            let steps = nl::auto_diagnose(query);
                            for step in &steps {
                                println!("  {} {}", "→".bright_cyan(), step.dimmed());
                                // Parse pipeline ops from step
                                let pipe_parts: Vec<&str> = step.splitn(2, " | ").collect();
                                let base = pipe_parts[0].trim();
                                let pipeline_ops = if pipe_parts.len() > 1 {
                                    value::parse_pipeline(&format!(
                                        "x | {}",
                                        pipe_parts[1..].join(" | ")
                                    ))
                                } else {
                                    vec![]
                                };
                                // Resolve joins
                                let pipeline_ops: Vec<value::PipeOp> = pipeline_ops
                                    .into_iter()
                                    .map(|op| {
                                        if let value::PipeOp::Join { table, on } = op {
                                            let right_result =
                                                commands::execute(&table, &db, &core_root);
                                            if let commands::CommandResult::Value(
                                                value::Value::Table(rows),
                                            ) = right_result
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
                                match commands::execute(base, &db, &core_root) {
                                    commands::CommandResult::Value(v)
                                        if !pipeline_ops.is_empty() =>
                                    {
                                        println!(
                                            "{}",
                                            value::apply_pipeline(v, &pipeline_ops).render()
                                        );
                                    }
                                    commands::CommandResult::Value(v) => println!("{}", v.render()),
                                    commands::CommandResult::Output(o) => println!("{}", o),
                                    _ => {}
                                }
                                println!();
                            }
                            continue;
                        }
                        let custom_patterns = nl::load_toml_patterns(&core_root);
                        match nl::translate_with_custom(query, &custom_patterns) {
                            Some(t) => {
                                print!("{}", nl::render_translation(&t));
                                use std::io::BufRead;
                                let stdin = std::io::stdin();
                                let answer = stdin
                                    .lock()
                                    .lines()
                                    .next()
                                    .and_then(|l| l.ok())
                                    .unwrap_or_default()
                                    .trim()
                                    .to_lowercase();
                                if answer == "y" || answer.is_empty() {
                                    println!();
                                    match commands::execute(&t.pipeline, &db, &core_root) {
                                        commands::CommandResult::Value(v) => {
                                            println!("{}", v.render())
                                        }
                                        commands::CommandResult::Output(o) => println!("{}", o),
                                        commands::CommandResult::Error(e) => eprintln!("  ✗ {}", e),
                                        _ => {}
                                    }
                                } else {
                                    println!("  ○ cancelled");
                                }
                            }
                            None => {
                                eprintln!(
                                    "  ✗ no pattern matched — try: ?memory hogs, ?biggest files"
                                );
                            }
                        }
                        continue;
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
                        let value_is_quoted = (after_eq.starts_with('"')
                            && after_eq.ends_with('"'))
                            || (after_eq.starts_with('\'') && after_eq.ends_with('\''));
                        let no_space_after = !after_eq.contains(' ');
                        no_space_before && (no_space_after || value_is_quoted)
                    };
                    if is_standalone_assign {
                        let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            let name = parts[0];
                            let valid = !name.is_empty()
                                && name
                                    .chars()
                                    .next()
                                    .map(|c| c.is_ascii_uppercase() || c == '_')
                                    .unwrap_or(false)
                                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                            if valid {
                                let val =
                                    parts[1].trim_matches('\"').trim_matches('\'').to_string();
                                let expanded = expand_vars(&val, &shell_vars);
                                std::env::set_var(name, &expanded);
                                shell_vars.insert(name.to_string(), expanded.clone());
                                println!(
                                    "  {} {} = {}",
                                    "→".bright_cyan(),
                                    name.bright_white(),
                                    expanded.dimmed()
                                );
                                continue 'repl;
                            }
                        }
                    }
                    // Inline env var assignment: KEY=val cmd  or  KEY=val KEY2=val cmd
                    {
                        let mut temp_vars: Vec<(String, String)> = vec![];
                        let mut rest = trimmed;
                        loop {
                            // Match WORD=value at start (no spaces around =, WORD is [A-Z_][A-Z0-9_]*)
                            let maybe_var = rest.split_whitespace().next().unwrap_or("");
                            if let Some(eq) = maybe_var.find('=') {
                                let name = &maybe_var[..eq];
                                let valid = !name.is_empty()
                                    && name
                                        .chars()
                                        .next()
                                        .map(|c| c.is_ascii_uppercase() || c == '_')
                                        .unwrap_or(false)
                                    && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                                if valid {
                                    let val = maybe_var[eq + 1..]
                                        .trim_matches('\"')
                                        .trim_matches('\'')
                                        .to_string();
                                    let expanded = expand_vars(&val, &shell_vars);
                                    temp_vars.push((name.to_string(), expanded));
                                    rest = rest[maybe_var.len()..].trim_start();
                                    continue;
                                }
                            }
                            break;
                        }
                        if !temp_vars.is_empty() {
                            // Set vars in environment
                            for (k, v) in &temp_vars {
                                std::env::set_var(k, v);
                                shell_vars.insert(k.clone(), v.clone());
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
                                continue 'repl;
                            }
                            let rest = expand_vars(rest, &shell_vars);
                            let result = exec::execute_with_context(
                                &rest,
                                &db,
                                &core_root,
                                &cfg.before_rules,
                            );
                            match result {
                                commands::CommandResult::Exit => break 'repl,
                                commands::CommandResult::Error(e) => {
                                    eprintln!("  {} {}", colored::Colorize::bright_red("✗"), e);
                                }
                                commands::CommandResult::Output(out) => println!("{}", out),
                                _ => {}
                            }
                            continue 'repl;
                        }
                    }

                    if let Some(rest) = trimmed.strip_prefix("let ") {
                        // let x = "value"  or  let x = value
                        if let Some(eq) = rest.find(" = ") {
                            let name = rest[..eq].trim().to_string();
                            let val = rest[eq + 3..]
                                .trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string();
                            let expanded = expand_vars(&val, &shell_vars);
                            println!(
                                "  {} {} = {}",
                                "→".bright_cyan(),
                                name.bright_white(),
                                expanded.dimmed()
                            );
                            shell_vars.insert(name, expanded);
                        } else {
                            eprintln!("  {} usage: let <name> = <value>", "✗".bright_red());
                        }
                        continue;
                    }
                    if let Some(rest) = trimmed.strip_prefix("export ") {
                        // export EDITOR=nvim  or  export EDITOR = nvim
                        let (name, val) = if let Some(eq) = rest.find('=') {
                            (
                                rest[..eq].trim(),
                                rest[eq + 1..].trim().trim_matches('"').trim_matches('\''),
                            )
                        } else {
                            (rest.trim(), "")
                        };
                        let expanded = expand_vars(val, &shell_vars);
                        std::env::set_var(name, &expanded);
                        shell_vars.insert(name.to_string(), expanded.clone());
                        println!(
                            "  {} export {} = {}",
                            "→".bright_cyan(),
                            name.bright_white(),
                            expanded.dimmed()
                        );
                        continue;
                    }

                    if let Some(rest) = trimmed.strip_prefix("unset ") {
                        let name = rest.trim();
                        shell_vars.remove(name);
                        std::env::remove_var(name);
                        println!("  {} unset {}", "→".bright_cyan(), name.bright_white(),);
                        continue;
                    }
                    // persist VAR — save variable to state.db for cross-session persistence
                    if let Some(rest) = trimmed.strip_prefix("persist ") {
                        let name = rest.trim();
                        let env_val = std::env::var(name).ok();
                        if let Some(val) = shell_vars
                            .get(name)
                            .or_else(|| env_val.as_deref().map(|v| shell_vars.get(v)).flatten())
                            .or(env_val.as_ref())
                        {
                            let val = val.clone();
                            let _ = db.conn.execute(
                                "INSERT OR REPLACE INTO shell_persist (key, value) VALUES (?1, ?2)",
                                rusqlite::params![name, &val],
                            );
                            println!(
                                "  {} {} persisted across sessions",
                                "→".bright_cyan(),
                                name.bright_white()
                            );
                        } else {
                            println!(
                                "  {} variable '{}' not set — use: export {}=value first",
                                "⚠️ ".yellow(),
                                name,
                                name
                            );
                        }
                        continue;
                    }
                    // Phase 10 — expand $VARS before alias resolution
                    let line = expand_vars(line, &shell_vars);
                    // Subshell expansion
                    let line = expand_subshells(&line);
                    // Subshell expansion
                    let line = expand_subshells(&line);
                    // Glob expansion — expand *.rs, *.md etc
                    let line = expand_globs(&line);
                    let line = line.as_str();

                    // Expand aliases before pipeline parsing
                    let first_word = line.split_whitespace().next().unwrap_or("").to_lowercase();
                    let line = if let Some(aliased) = db.get_alias(&first_word) {
                        let rest: String = line
                            .split_once(' ')
                            .map(|x| x.1)
                            .map(|s| format!(" {}", s))
                            .unwrap_or_default();
                        format!("{}{}", aliased, rest)
                    } else {
                        line.to_string()
                    };
                    let line = line.as_str();

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
                        let working_line = if redirect_target == "__stderr__" {
                            line
                        } else {
                            line_stripped.as_str()
                        };
                        let (cmd_part, stderr_to_stdout, stderr_file) =
                            if working_line.contains(" 2>&1") {
                                let cleaned = working_line.replace(" 2>&1", "").trim().to_string();
                                // Also strip any stdout redirect
                                let (c2, _) = detect_redirect(&cleaned);
                                (c2, true, None)
                            } else if let Some(idx) = working_line.find(" 2>/dev/null") {
                                (
                                    working_line[..idx].trim().to_string(),
                                    false,
                                    Some("/dev/null".to_string()),
                                )
                            } else if let Some(idx) = working_line.find(" 2>") {
                                let after = working_line[idx + 3..].trim().to_string();
                                (working_line[..idx].trim().to_string(), false, Some(after))
                            } else {
                                (line_stripped.clone(), false, None)
                            };
                        // If it's a pure stderr redirect, handle separately
                        let is_stderr_only = redirect_target == "__stderr__";
                        // Open output file
                        // For stderr-only redirects, handle without opening stdout file
                        if is_stderr_only {
                            let stderr_stdio = if let Some(ref sf_path) = stderr_file {
                                std::fs::File::create(sf_path)
                                    .map(std::process::Stdio::from)
                                    .unwrap_or(std::process::Stdio::inherit())
                            } else {
                                std::process::Stdio::inherit()
                            };
                            let _ = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&cmd_part)
                                .stdin(std::process::Stdio::inherit())
                                .stdout(std::process::Stdio::inherit())
                                .stderr(stderr_stdio)
                                .envs(std::env::vars())
                                .status();
                            continue 'repl;
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
                                use std::os::fd::FromRawFd;
                                use std::os::unix::io::IntoRawFd;
                                // Try fsh builtins first
                                let builtin_result = commands::execute(&cmd_part, &db, &core_root);
                                let builtin_out = match builtin_result {
                                    commands::CommandResult::Output(o) => Some(o),
                                    commands::CommandResult::Value(v) => Some(v.render()),
                                    _ => None,
                                };
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
                                    let parts: Vec<&str> = cmd_part.trim().splitn(2, ' ').collect();
                                    if !parts.is_empty() {
                                        let mut cmd = std::process::Command::new(parts[0]);
                                        if parts.len() > 1 {
                                            cmd.args(parts[1].split_whitespace());
                                        }
                                        if stderr_to_stdout {
                                            // 2>&1: open file twice for both stdout and stderr
                                            let fd = f.into_raw_fd();
                                            let stdout_f =
                                                unsafe { std::fs::File::from_raw_fd(fd) };
                                            // Reopen same file for stderr
                                            let stderr_f = if is_append {
                                                std::fs::OpenOptions::new()
                                                    .append(true)
                                                    .create(true)
                                                    .open(redirect_target)
                                            } else {
                                                std::fs::OpenOptions::new()
                                                    .write(true)
                                                    .create(true)
                                                    .open(redirect_target)
                                            };
                                            if let Ok(sf) = stderr_f {
                                                let _ = cmd
                                                    .stdout(std::process::Stdio::from(stdout_f))
                                                    .stderr(std::process::Stdio::from(sf))
                                                    .status();
                                            }
                                        } else if let Some(ref sf_path) = stderr_file {
                                            // 2>file: stderr to different file
                                            let sf = std::fs::File::create(sf_path).ok();
                                            let _ = cmd
                                                .stdout(std::process::Stdio::from(f))
                                                .stderr(
                                                    sf.map(std::process::Stdio::from)
                                                        .unwrap_or(std::process::Stdio::inherit()),
                                                )
                                                .status();
                                        } else {
                                            // Normal stdout redirect
                                            let _ = cmd
                                                .stdout(std::process::Stdio::from(f))
                                                .stderr(std::process::Stdio::inherit())
                                                .status();
                                        }
                                    }
                                } // end else external
                            }
                            Err(e) => eprintln!("fsh: redirect error: {}", e),
                        }
                        continue 'repl;
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
                                    for ch in part.chars() {
                                        match ch {
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
                                let owned_args: Vec<String> = tokens[1..].to_vec();
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
                                // INT-233: try fsh builtin first -- only if first stage, not a path
                                let builtin_name = if raw_cmd.contains('/') {
                                    raw_cmd.split('/').last().unwrap_or(&raw_cmd).to_string()
                                } else {
                                    raw_cmd.clone()
                                };
                                // Only attempt builtin path on first stage and non-path commands
                                let try_builtin = idx == 0 && !raw_cmd.contains('/');
                                let builtin_line = if args.is_empty() {
                                    builtin_name.clone()
                                } else {
                                    format!("{} {}", builtin_name, args.join(" "))
                                };
                                let builtin_out = if try_builtin {
                                    match commands::execute(&builtin_line, &db, &core_root) {
                                        commands::CommandResult::Output(o) => Some(o),
                                        commands::CommandResult::Value(v) => Some(v.render()),
                                        _ => None,
                                    }
                                } else {
                                    None
                                };
                                if let Some(out) = builtin_out {
                                    if is_last {
                                        println!("{}", out);
                                    } else {
                                        // Pipe builtin output to remaining external pipeline stages
                                        let remaining = pipe_parts[idx + 1..].join(" | ");
                                        use std::io::Write;
                                        let mut child = std::process::Command::new("sh")
                                            .arg("-c")
                                            .arg(&remaining)
                                            .stdin(std::process::Stdio::piped())
                                            .stdout(std::process::Stdio::inherit())
                                            .stderr(std::process::Stdio::inherit())
                                            .spawn()
                                            .ok();
                                        if let Some(ref mut c) = child {
                                            if let Some(ref mut stdin) = c.stdin.take() {
                                                let _ = stdin.write_all(out.as_bytes());
                                            }
                                            let _ = c.wait();
                                        }
                                        // Already handled -- skip sh fallback
                                        for mut child in children {
                                            let _ = child.wait();
                                        }
                                        continue 'repl;
                                    }
                                } else {
                                    match std::process::Command::new(cmd_name)
                                        .args(&args)
                                        .stdin(stdin_src)
                                        .stdout(stdout_dst)
                                        .stderr(std::process::Stdio::inherit())
                                        .spawn()
                                    {
                                        Ok(mut child) => {
                                            if !is_last {
                                                prev_stdout = child.stdout.take();
                                            }
                                            children.push(child);
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "  {} pipe stage '{}' failed: {}",
                                                "✗".bright_red(),
                                                cmd_name,
                                                e
                                            );
                                            eprintln!(
                                                "  {} check: is '{}' a valid command?",
                                                "·".dimmed(),
                                                cmd_name
                                            );
                                            pipe_ok = false;
                                            break;
                                        }
                                    }
                                }
                            }
                            if pipe_ok {
                                for mut child in children {
                                    let _ = child.wait();
                                }
                                continue 'repl;
                            }
                        }
                        // Fallback to sh for complex cases
                        let sh_output = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(original_line)
                            .stdin(std::process::Stdio::inherit())
                            .stdout(std::process::Stdio::inherit())
                            .stderr(std::process::Stdio::inherit())
                            .status();
                        if let Err(e) = sh_output {
                            eprintln!("fsh: pipe error: {}", e);
                        }
                        continue 'repl;
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
                            match commands::execute(&base_cmd, &db, &core_root) {
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
                        continue 'repl;
                    }

                    // Phase 8 — Job control commands
                    let first_tok = line.split_whitespace().next().unwrap_or("");
                    if first_tok == "jobs" {
                        job_table.list();
                        continue;
                    }
                    if first_tok == "fg" {
                        let second = line.split_whitespace().nth(1).unwrap_or("");
                        // Only intercept as job control if second token is a number
                        // fg commit, fg push, etc. → fall through to execute_with_context
                        if second.is_empty() || second.parse::<usize>().is_ok() {
                            let id = second.parse::<usize>().unwrap_or(1);
                            job_table.fg(id);
                            continue;
                        }
                        // Otherwise fall through — fg commit etc. handled by alias
                    }
                    if first_tok == "kill" {
                        let arg = line.split_whitespace().nth(1).unwrap_or("");
                        let id = arg.trim_start_matches('%').parse::<usize>().unwrap_or(0);
                        if id > 0 {
                            job_table.kill_job(id);
                        } else {
                            println!("  usage: kill %<job_id>");
                        }
                        continue;
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
                                let right_result = commands::execute(&table, &db, &core_root);
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
                        let fc = base_cmd
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_lowercase();
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
                        let sh_output = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(line)
                            .stdout(std::process::Stdio::inherit())
                            .stderr(std::process::Stdio::inherit())
                            .status();
                        let _ = sh_output;
                        continue 'repl;
                    }
                    let _cmd_timer_start = std::time::Instant::now();
                    let cmd_output: Option<String> = match exec::execute_with_context(
                        &base_cmd,
                        &db,
                        &core_root,
                        &cfg.before_rules,
                    ) {
                        commands::CommandResult::Exit => break 'repl,
                        commands::CommandResult::Value(v)
                            if !pipeline_ops.is_empty() && !has_external_op =>
                        {
                            let result = value::apply_pipeline(v, &pipeline_ops);
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
                                    Some(stdout)
                                }
                                Err(_) => None,
                            }
                        }
                        commands::CommandResult::Value(v) => Some(v.render()),
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
                                    Some(stdout)
                                }
                                Err(_) => Some(out),
                            }
                        }
                        commands::CommandResult::Output(out) => {
                            last_exit_code = Some(0);
                            Some(out)
                        }
                        commands::CommandResult::Empty => {
                            last_exit_code = Some(0);
                            None
                        }
                        commands::CommandResult::Error(e) => {
                            eprintln!("{} {}", colored::Colorize::bright_red("✗"), e);
                            last_exit_code = Some(1);
                            None
                        }
                    };
                    // Command timing intelligence — warn if command is unusually slow (INT-194)
                    {
                        let elapsed_ms = _cmd_timer_start.elapsed().as_millis() as i64;
                        let cmd_key = base_cmd.split_whitespace().next().unwrap_or(&base_cmd);
                        if elapsed_ms > 500 {
                            let _ = db.conn.execute(
                                "INSERT INTO shell_history (command, timestamp) VALUES (?1, ?2)",
                                rusqlite::params![
                                    format!("TIMING:{}:{}", cmd_key, elapsed_ms),
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_secs() as i64)
                                        .unwrap_or(0)
                                ],
                            );
                            let avg_ms: Option<f64> = db.conn.query_row(
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
                                std::process::Command::new("faelight-notify")
                                    .arg("--title")
                                    .arg("Long command finished")
                                    .arg("--body")
                                    .arg(&msg)
                                    .spawn()
                                    .ok();
                            }
                        }
                    }
                    // INT-194 — Prediction-aware suggestions (pattern detection)
                    // After each command, check if there is a strong "next command" pattern
                    {
                        let cmd_key = base_cmd
                            .split_whitespace()
                            .next()
                            .unwrap_or(&base_cmd)
                            .to_string();
                        // Only suggest for meaningful commands, not builtins
                        let skip_suggest = matches!(
                            cmd_key.as_str(),
                            "d" | "ls" | "cd" | "echo" | "cat" | "help" | "exit" | "q" | "clear"
                        );
                        if !skip_suggest {
                            // Find what command most often follows this one
                            let next_cmd: Option<String> = db
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
                                let freq: i64 = db
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
                                    println!(
                                        "  {} you usually run {} next",
                                        "💡".normal(),
                                        suggestion.bright_cyan()
                                    );
                                }
                            }
                        }
                    }
                    // Store last output for `last` command (INT-194)
                    if let Some(ref out) = cmd_output {
                        if !out.is_empty() {
                            let _ = db.conn.execute(
                                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_output', ?1)",
                                rusqlite::params![out],
                            );
                        }
                    }
                    // INT-201 — Track last command exit status for faelight-term indicator
                    {
                        // Update last_exit_code from output content
                        if let Some(ref out) = cmd_output {
                            if out.starts_with("✗")
                                || out.contains("error")
                                || out.contains("not found")
                            {
                                last_exit_code = Some(1);
                            } else {
                                last_exit_code = Some(0);
                            }
                        }
                        let exit_ok = match &cmd_output {
                            Some(out) => !out.starts_with("✗") && !out.contains("error"),
                            None => last_exit_code.map(|c| c == 0).unwrap_or(true),
                        };
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
                    triggers::ensure_schema(&db);
                    let health = db.health_score();
                    let trigger_ctx = triggers::TriggerContext {
                        last_command: base_cmd.clone(),
                        health_score: health,
                        last_domain: None,
                    };
                    triggers::evaluate(&db, &trigger_ctx, &core_root);
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
                        last_exit_code = Some(exit_code);
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
                                                    println!();
                                                    println!("  \u{1f332} Friday: {}", msg);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // 🌲 Forest speaks — surface contextd insights after every command
                    {
                        let insight: Option<(i64, String, String, f64)> = db
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
                            let _ = db.conn.execute(
                                "UPDATE forest_insights SET shown = 1 WHERE id = ?1",
                                rusqlite::params![id],
                            );
                        }
                    }
                    // INT-203 Phase 2: Friday proactive message
                    if _session_commands % 10 == 0 && _session_commands > 0 {
                        let pattern: Option<(String, String, f64)> = db.conn.query_row(
                            "SELECT trigger, action, confidence FROM friday_patterns WHERE confidence >= 0.7 ORDER BY confidence DESC LIMIT 1",
                            [], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?))
                        ).ok();
                        if let Some((trigger, action, conf)) = pattern {
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
    session::SessionMemory::save(&core_root, None, &db);
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
    let distinct_intents: i64 = db.conn.query_row(
        "SELECT COUNT(DISTINCT intent_id) FROM commit_patterns WHERE timestamp > ?1 AND intent_id != ''",
        rusqlite::params![session_start_ts],
        |r| r.get(0),
    ).unwrap_or(1);
    let focus_score: f64 = if distinct_intents <= 1 {
        1.0
    } else {
        1.0 / distinct_intents as f64
    };
    let _ = db.conn.execute_batch(
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
    let _ = db.conn.execute(
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
            std::fs::read_dir(std::path::PathBuf::from(&core_root).join("intents/future"))
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

fn print_welcome(core_root: &str, db: &crate::db::ForestDb) {
    use colored::Colorize;
    use std::path::PathBuf;

    let root = PathBuf::from(core_root);

    let version = std::fs::read_to_string(root.join("00-meta/VERSION"))
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();

    let changelog = std::fs::read_to_string(root.join("00-meta/CHANGELOG.md")).unwrap_or_default();
    let theme = changelog
        .lines()
        .find(|l| l.starts_with(&format!("## [{}]", version)))
        .and_then(|l| l.split(" — ").nth(1))
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
        format!("{}% ✅", health_num).bright_green().to_string()
    } else if health_num >= 80 {
        format!("{}% ⚠", health_num).yellow().to_string()
    } else {
        format!("{}% ❌", health_num).bright_red().to_string()
    };

    // Count intents by scanning all categories — mirrors doctor check_intents logic exactly
    let (complete_count, planned_count) = {
        let intent_dir = root.join("intents");
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
    let tool_count = std::fs::read_to_string(root.join("01-registry/tools.toml"))
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
    println!("  {}", "🌲 The forest stirs...".bright_green().dimmed());
    println!();
    println!("  {} — {}", version.bright_green().bold(), theme.dimmed());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!(
        "  {} intents complete  ·  {} commits",
        complete_count.to_string().bright_white(),
        commits.bright_white()
    );
    println!(
        "  {}  ·  {} tools  ·  {} planned",
        health_display,
        tool_count.to_string().bright_white(),
        planned_count.to_string().dimmed()
    );
    println!();
    println!("  {}", format!("\"{}\"", quote).dimmed());
    println!();
    // Today's Focus — lowest audit score tool
    let _focus = std::fs::read_to_string(root.join("01-registry/tools.toml"))
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
    let focus_intent: Option<String> = std::fs::read_dir(root.join("intents/future"))
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
        println!("  {} {}", "Today:".dimmed(), focus.bright_white());
        // Auto-persist detected intent so prompt.rs can read it
        if let Ok(db) = crate::db::ForestDb::open() {
            // Only write if no conscious focus already set
            if db.get_focus_intent().is_none() {
                // Extract INT-NNN from filename — only if first token is numeric
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
    if let Some(mem) = session::SessionMemory::load(core_root, &db) {
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
        let msg = session::render(&mem, core_root, &db);
        if !msg.is_empty() {
            println!("{}", msg);
        }
        // INT-143 Phase 1 — forest digest on long gaps
        if digest::should_show(&mem) {
            let _db_path = root.join("runtime/state.db");
            if let Ok(db) = crate::db::ForestDb::open() {
                let d = digest::render(&mem, &db, core_root);
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
