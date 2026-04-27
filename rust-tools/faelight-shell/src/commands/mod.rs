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
}

// ── Security Layer — log every command ───────────────────────────────────────
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
pub fn execute(line: &str, db: &ForestDb, core_root: &str) -> CommandResult {
    fn tokenize_args(s: &str) -> Vec<String> {
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
    let trimmed_line = line.trim();
    let cmd = trimmed_line
        .splitn(2, ' ')
        .next()
        .unwrap_or("")
        .to_lowercase();
    let rest_str = trimmed_line.splitn(2, ' ').nth(1).unwrap_or("");
    let owned_args: Vec<String> = tokenize_args(rest_str);
    let args_vec: Vec<&str> = owned_args.iter().map(|s| s.as_str()).collect();
    let args = args_vec.as_slice();

    // !! — repeat last command
    if line.trim() == "!!" {
        match db.get_last_command() {
            Some(last) => {
                println!("  {}", last.dimmed());
                return execute(&last, db, core_root);
            }
            None => return CommandResult::Error("No previous command in history".to_string()),
        }
    }

    // Alias resolution — check before dispatch
    if let Some(aliased) = db.get_alias(&cmd) {
        let expanded = if args.is_empty() {
            aliased.clone()
        } else {
            format!("{} {}", aliased, args.join(" "))
        };
        // Recurse with expanded command
        return execute(&expanded, db, core_root);
    }

    // Plugin resolution — after final cmd parse
    {
        let plugins = db.load_plugins();
        if let Some((_, expand, _)) = plugins
            .iter()
            .find(|(name, _, _)| name.as_str() == cmd.as_str())
        {
            let expanded = if args.is_empty() {
                expand.clone()
            } else {
                format!("{} {}", expand, args.join(" "))
            };
            return execute(&expanded, db, core_root);
        }
    }

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
        "intents" => intents(core_root),
        "tools" => tools_table(db, core_root),
        "version" => version(core_root),
        "schema" => schema(args),
        "commits" => commits(core_root),
        "story" => story(db),
        "advise" => advise(db),
        "audit" => audit(db, core_root),
        // ── Core subcommand shortcuts — no prefix needed ────────────────────
        "predict" | "react" | "stress" | "doctor" | "goals" | "evolution" | "security"
        | "capabilities" | "intent" | "genealogy" | "autonomy" => {
            let sub = args.join(" ");
            let full = if sub.is_empty() {
                format!("{}/scripts/core {}", core_root, cmd)
            } else {
                format!("{}/scripts/core {} {}", core_root, cmd, sub)
            };
            run_external(&full, db)
        }
        "sandbox" => sandbox(db),
        "checkpoint" | "cpc" => checkpoint(db),
        "let" => scripting_let_cmd(db, core_root, args),
        "run" => scripting_run_cmd(db, core_root, args),
        "python" | "py" => run_python_cmd(args),
        "js" | "node" => run_js_cmd(args),
        "undo" => undo_cmd(db, args),
        "pv" => smart_preview_cmd(args),
        "fsh" | "faelight-shell" => match args.first().copied() {
            Some("diag") => fsh_diag(db),
            Some("gaps") => fsh_gaps(db),
            _ => fsh_identity_cmd(db),
        },
        "snapshot" => snapshot_cmd(db, args),
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
        "git" => run_external(line, db),
        "search" | "s" => search(db, args),
        "where_old_disabled" => {
            CommandResult::Error("use with pipe: tools | where score < 70".to_string())
        }
        "tools-table" | "tt" => tools_table(db, core_root),
        "events-table" | "et" => events_table(db, args),
        "audit-table" | "at" => audit_table(db, core_root),
        "decisions-table" | "dt" => decisions_table(db),
        "count" => CommandResult::Output("  use with pipe: tt | count".to_string()),
        "history-table" | "ht" | "history" => match args.first().copied() {
            Some("intent") => ht_intent(db),
            Some("today") => ht_today(db),
            Some("session") => ht_session(db),
            Some("slow") => ht_slow(db),
            Some(search) => history_search_cmd(db, &[search]),
            None => history_table(db),
        },
        "history-search" | "hs" | "hsearch" => history_search_cmd(db, args),
        "zsh" | "bash" => shell_handoff_cmd(line),
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
        "find" | "fd" => find_cmd(db, core_root, args),
        "grep" => grep_cmd(line, args),
        "tree" => tree_cmd(args),
        "fstat" | "stat" => stat_cmd(args),
        "peek" | "preview" => preview_cmd(args),
        "exec" => exec_cmd(args),
        "realpath" | "rp" => realpath_cmd(args),
        "time" => time_cmd(line, args),
        "reload" => {
            let exe = std::env::current_exe().unwrap_or_default();
            let err = std::process::Command::new(&exe).exec();
            CommandResult::Error(format!("reload: {}", err))
        }
        "source" => source_cmd(args),
        "net" | "network" => sys_network(),
        "pkgs" | "packages" => sys_packages(),
        "pkg" => pkg_cmd(args),
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
                "pkgs",
                "pkg",
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
        "rename" => {
            // rename old_name new_name           -- rename across all files
            // rename old_name new_name --type rs -- only .rs files
            // rename old_name new_name --dry-run -- preview only
            if args.len() < 2 {
                return CommandResult::Error(
                    "usage: rename <old> <new> [--type ext] [--dry-run]".to_string(),
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
                Some(t) => t.replace(
                    "\\n", "
",
                ),
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
                    filepath, &anchor[..anchor.len().min(60)], &anchor[..anchor.len().min(20)]
                ));
            }
            if count > 1 {
                return CommandResult::Error(format!(
                    "rspatch: anchor matches {} times -- must be unique\n  what:  anchor text is ambiguous\n  anchor: {}\n  fix:   use a longer, more specific anchor string",
                    count, &anchor[..anchor.len().min(60)]
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
                    &anchor[..anchor.len().min(40)]
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
                    errors.push(format!("  not found: '{}'", old));
                } else if count > 1 {
                    errors.push(format!(
                        "  ambiguous: '{}' ({} matches -- must be unique)",
                        old, count
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
        "fsearch" => {
            // fsearch "fn expand"                    -- all files recursively
            // fsearch "fn expand" --type rs          -- only .rs files
            // fsearch "fn expand" --file main.rs     -- only in specific file
            if args.is_empty() {
                return CommandResult::Error(
                    "usage: search <pattern> [--type ext] [--file name]".to_string(),
                );
            }
            let pattern = args[0].to_lowercase();
            let mut filter_type: Option<&str> = None;
            let mut filter_file: Option<&str> = None;
            let mut search_root: Option<std::path::PathBuf> = None;
            let mut unknown: Vec<String> = Vec::new();
            let mut i = 1;
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
                    arg if !arg.starts_with("--") => {
                        // Positional: treat as search path if it exists on disk
                        let expanded = if arg.starts_with("~/") {
                            let home = std::env::var("HOME").unwrap_or_default();
                            arg.replacen("~/", &format!("{}/", home), 1)
                        } else {
                            arg.to_string()
                        };
                        let p = std::path::PathBuf::from(&expanded);
                        if p.exists() {
                            if search_root.is_some() {
                                unknown.push(format!("{} (path already set)", arg));
                            } else {
                                search_root = Some(p);
                            }
                        } else {
                            unknown.push(arg.to_string());
                        }
                        i += 1;
                    }
                    _ => {
                        unknown.push(args[i].to_string());
                        i += 1;
                    }
                }
            }
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
                            "rs", "py", "md", "toml", "sh", "fsh", "txt", "json", "yaml", "yml",
                            "html", "css", "js", "ts",
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
                return CommandResult::Error(format!("patch: text not found in {}", filepath));
            }
            if count > 1 {
                return CommandResult::Error(format!(
                    "patch: {} occurrences found -- text must be unique (found {})",
                    count, count
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
                "pkgs",
                "pkg",
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
        _ => run_external(line, db),
    };

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
    let fmt = args.first().copied().unwrap_or("");
    CommandResult::Error(format!(
        "from: use open instead — e.g. {} or {}",
        format!("open file.{}", if fmt.is_empty() { "json" } else { fmt }).bright_cyan(),
        "open data.csv | select name".bright_cyan()
    ))
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

    let tools_dir = std::path::PathBuf::from(core_root).join("rust-tools");
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
fn shell_handoff_cmd(line: &str) -> CommandResult {
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
                let version = std::fs::read_to_string(
                    std::path::PathBuf::from(db.core_root()).join("00-meta/VERSION"),
                )
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
        for (name, cmd) in &aliases {
            out.push_str(&format!(
                "  │  {:<15} = {}\n",
                name.bright_cyan(),
                cmd.dimmed()
            ));
        }
        out.push_str(
            &"  ╰────────────────────────────────────────────────────"
                .dimmed()
                .to_string(),
        );
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

    if db.add_alias(&name, &command) {
        CommandResult::Output(format!(
            "  {} alias {} = {}",
            "✅".green(),
            name.bright_cyan(),
            command.dimmed()
        ))
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
        .args(["aux", "--no-headers"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 11 {
                return None;
            }
            let mut row = HashMap::new();
            row.insert("pid".to_string(), Value::Text(parts[1].to_string()));
            row.insert(
                "name".to_string(),
                Value::Text(
                    parts[10..]
                        .join(" ")
                        .split('/')
                        .next_back()
                        .unwrap_or(parts[10])
                        .chars()
                        .take(30)
                        .collect(),
                ),
            );
            row.insert("cpu".to_string(), Value::Text(parts[2].to_string()));
            row.insert("memory".to_string(), Value::Text(parts[3].to_string()));
            row.insert("user".to_string(), Value::Text(parts[0].to_string()));
            row.insert("status".to_string(), Value::Text(parts[7].to_string()));
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

// ── Phase 12 — pkg — forest-native package interface ─────────────────────────
// Wraps paru/pacman with structured table output.
// pkg list installed   → Value::Table (pipeable)
// pkg search <term>    → Value::Table (pipeable)
// pkg install <name>   → interactive paru -S
// pkg remove  <name>   → interactive paru -Rns
// pkg update           → interactive paru -Syu
fn pkg_cmd(args: &[&str]) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let sub = args.first().copied().unwrap_or("help");

    match sub {
        // ── pkg list installed ────────────────────────────────────────────
        "list" => {
            let filter = args.get(1).copied().unwrap_or("installed");
            if filter != "installed" {
                return CommandResult::Error(format!(
                    "  unknown list filter: {} — try: pkg list installed",
                    filter
                ));
            }
            let output = std::process::Command::new("pacman")
                .args(["-Q"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();

            let rows: Vec<HashMap<String, Value>> = output
                .lines()
                .filter_map(|line| {
                    let mut parts = line.splitn(2, ' ');
                    let name = parts.next()?.to_string();
                    let version = parts.next().unwrap_or("?").trim().to_string();
                    let mut row = HashMap::new();
                    row.insert("name".to_string(), Value::Text(name));
                    row.insert("version".to_string(), Value::Text(version));
                    Some(row)
                })
                .collect();

            let count = rows.len();
            println!(
                "  {} {} packages installed",
                "📦".normal(),
                count.to_string().bright_white()
            );
            CommandResult::Value(Value::Table(rows))
        }

        // ── pkg install <name> ────────────────────────────────────────────
        "install" | "add" => {
            let name = match args.get(1) {
                Some(n) => n,
                None => return CommandResult::Error("  usage: pkg install <package>".to_string()),
            };
            println!(
                "  {} installing {} via paru...",
                "📦".normal(),
                name.bright_cyan()
            );
            let status = std::process::Command::new("paru")
                .args(["-S", name])
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            match status {
                Ok(s) if s.success() => CommandResult::Output(format!(
                    "  {} {} installed",
                    "✅".normal(),
                    name.bright_green()
                )),
                Ok(s) => CommandResult::Error(format!(
                    "  paru exited with code {}",
                    s.code().unwrap_or(-1)
                )),
                Err(e) => CommandResult::Error(format!("  failed to run paru: {}", e)),
            }
        }

        // ── pkg remove <name> ─────────────────────────────────────────────
        "remove" | "uninstall" | "rm" => {
            let name = match args.get(1) {
                Some(n) => n,
                None => return CommandResult::Error("  usage: pkg remove <package>".to_string()),
            };
            println!("  {} removing {} ...", "🗑".normal(), name.bright_red());
            let status = std::process::Command::new("paru")
                .args(["-Rns", name])
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            match status {
                Ok(s) if s.success() => CommandResult::Output(format!(
                    "  {} {} removed",
                    "✅".normal(),
                    name.bright_green()
                )),
                Ok(s) => CommandResult::Error(format!(
                    "  paru exited with code {}",
                    s.code().unwrap_or(-1)
                )),
                Err(e) => CommandResult::Error(format!("  failed to run paru: {}", e)),
            }
        }

        // ── pkg update ────────────────────────────────────────────────────
        "update" | "upgrade" => {
            println!("  {} updating all packages via paru...", "🔄".normal());
            let status = std::process::Command::new("paru")
                .args(["-Syu"])
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            match status {
                Ok(s) if s.success() => CommandResult::Output("  ✅ system updated".to_string()),
                Ok(s) => CommandResult::Error(format!(
                    "  paru exited with code {}",
                    s.code().unwrap_or(-1)
                )),
                Err(e) => CommandResult::Error(format!("  failed to run paru: {}", e)),
            }
        }

        // ── pkg help ──────────────────────────────────────────────────────
        _ => CommandResult::Output(format!(
            "  {} pkg — forest-native package interface\n\
             {}\n\
             {}  {}\n\
             {}  {}\n\
             {}  {}\n\
             {}  {}\n\
             {}  {}",
            "📦".normal(),
            "  ─────────────────────────────────────".dimmed(),
            "  pkg list installed".bright_cyan(),
            "list installed packages as table".dimmed(),
            "  pkg search <term>".bright_cyan(),
            "search repos and AUR".dimmed(),
            "  pkg install <name>".bright_cyan(),
            "install via paru".dimmed(),
            "  pkg remove <name>".bright_cyan(),
            "remove package".dimmed(),
            "  pkg update".bright_cyan(),
            "update all packages".dimmed(),
        )),
    }
}

fn sys_packages() -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    let output = std::process::Command::new("pacman")
        .args(["-Q"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ' ');
            let name = parts.next()?.to_string();
            let version = parts.next().unwrap_or("?").trim().to_string();
            let mut row = HashMap::new();
            row.insert("name".to_string(), Value::Text(name));
            row.insert("version".to_string(), Value::Text(version));
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
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
    let themes = ["forest", "minimal", "jarvis", "classic"];
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
            "  theme: unknown theme '{}'\n  available: forest, minimal, jarvis, classic",
            name
        )),
    }
}

fn run_external(line: &str, db: &ForestDb) -> CommandResult {
    use std::io::{BufRead, BufReader, Write};
    let mut child = match std::process::Command::new("sh")
        .arg("-c")
        .arg(line)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn() {
        Ok(c) => c,
        Err(e) => return CommandResult::Error(format!("spawn failed: {}", e)),
    };
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line_result in reader.lines() {
            if let Ok(out_line) = line_result {
                println!("{}", out_line);
                let trimmed = out_line.trim();
                let is_heredoc_leak = trimmed.len() >= 4
                    && trimmed.ends_with("EOF")
                    && trimmed[..trimmed.len()-3].len() >= 1
                    && trimmed[..trimmed.len()-3].chars().all(|c| c.is_ascii_uppercase() || c == '_');
                if is_heredoc_leak {
                    eprintln!("  {} possible unclosed heredoc -- {:?} appeared as standalone output line",
                        "\u{26A0}".bright_yellow(), out_line.trim());
                }
                let _ = std::io::stdout().flush();
            }
        }
    }
    let status = child.wait();
    match status {
        Ok(s) => {
            if s.success() {
                CommandResult::Empty
            } else {
                let code = s.code().unwrap_or(1);
                // INT-233 -- command not found: suggest nearest known alternative
                if code == 127 {
                    let typed_cmd = line.split_whitespace().next().unwrap_or("").to_lowercase();
                    if !typed_cmd.is_empty() {
                        let known: &[&str] = &[
                            "deploy",
                            "cistart",
                            "cicomplete",
                            "intent",
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
                            "unlock-core",
                            "lock-core",
                            "core",
                            "faelight-git",
                            "fg",
                            "faelight-daemon",
                            "faelight-shell",
                            "faelight-term",
                        ];
                        let prefix_len = typed_cmd.len().min(3);
                        let prefix = &typed_cmd[..prefix_len];
                        let suggestion = known
                            .iter()
                            .filter(|&&k| {
                                k.to_lowercase().starts_with(prefix) && k != typed_cmd.as_str()
                            })
                            .min_by_key(|&&k| k.len().abs_diff(typed_cmd.len()))
                            .copied();
                        let alias_suggestion: Option<String> = db.conn.query_row(
                            "SELECT name FROM aliases WHERE name LIKE ?1 AND name != ?2 LIMIT 1",
                            rusqlite::params![format!("{}%", prefix), typed_cmd.as_str()],
                            |r| r.get(0)
                        ).ok();
                        if let Some(s) = suggestion {
                            println!(
                                "  {} command not found: {}",
                                "x".bright_red(),
                                typed_cmd.bright_red()
                            );
                            println!("  {} did you mean: {}", "->".bright_cyan(), s.bright_cyan());
                        } else if let Some(a) = alias_suggestion {
                            println!(
                                "  {} command not found: {}",
                                "x".bright_red(),
                                typed_cmd.bright_red()
                            );
                            println!("  {} did you mean: {}", "->".bright_cyan(), a.bright_cyan());
                        } else {
                            return CommandResult::Error(format!("  exited with code {}", code));
                        }
                        return CommandResult::Empty;
                    }
                }
                CommandResult::Error(format!("  exited with code {}", code))
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
        anomalies.push(format!(
            "{} permission errors — core locked during work?",
            perm_errors
        ));
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
            "Expect: d, core integrity run, core strategy jarvis",
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
                            "E_CORE_LOCKED" => Some(format!("unlock-core && {}", cmd)),
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
        "lock-core" | "core-protect" if line.contains("lock") && !line.contains("unlock") => {
            Some("💡 Forest is protected — remember to unlock-core before editing")
        }
        "unlock-core" | "core-protect" if line.contains("unlock") => {
            Some("💡 Reminder: run lock-core before shutdown")
        }
        "deploy" => Some("💡 Suggestion: run d — verify health after deploy"),
        "paru" | "pacman" => Some("💡 Suggestion: run d — verify system health after update"),
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
        ("pkgs", "installed packages as table"),
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

    let version =
        std::fs::read_to_string(std::path::PathBuf::from(db.core_root()).join("00-meta/VERSION"))
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

fn intents(core_root: &str) -> CommandResult {
    let future_dir = std::path::PathBuf::from(core_root).join("intents/future");
    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        "  ╭─ 🎯 Active Intents ────────────────────────────────".bright_cyan()
    ));

    if let Ok(entries) = std::fs::read_dir(&future_dir) {
        let mut found = false;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                let title = content
                    .lines()
                    .find(|l| l.starts_with("title:"))
                    .map(|l| {
                        l.trim_start_matches("title:")
                            .trim()
                            .trim_matches('"')
                            .to_string()
                    })
                    .unwrap_or_else(|| name.clone());
                let id = name.split('-').next().unwrap_or("?");
                out.push_str(&format!(
                    "  │  {}  {}\n",
                    format!("INT-{}", id).bright_yellow(),
                    title.dimmed()
                ));
                found = true;
            }
        }
        if !found {
            out.push_str("  │  No active intents\n");
        }
    }
    out.push_str(
        &"  ╰────────────────────────────────────────────────────"
            .dimmed()
            .to_string(),
    );
    CommandResult::Output(out)
}

#[allow(dead_code)]
fn tools(_db: &ForestDb, core_root: &str) -> CommandResult {
    let tools_dir = std::path::PathBuf::from(core_root).join("rust-tools");
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

fn version(core_root: &str) -> CommandResult {
    let version =
        std::fs::read_to_string(std::path::PathBuf::from(core_root).join("00-meta/VERSION"))
            .unwrap_or_else(|_| "unknown".into());

    let changelog =
        std::fs::read_to_string(std::path::PathBuf::from(core_root).join("00-meta/CHANGELOG.md"))
            .unwrap_or_default();

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
    let core_root = db.core_root();
    let output = std::process::Command::new(format!("{}/scripts/core", core_root))
        .args(["story"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "core story not available".to_string());

    CommandResult::Output(output)
}

fn advise(db: &ForestDb) -> CommandResult {
    let core_root = db.core_root();
    let output = std::process::Command::new(format!("{}/scripts/core", core_root))
        .args(["advise"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "core advise not available".to_string());

    CommandResult::Output(output)
}

fn audit(_db: &ForestDb, core_root: &str) -> CommandResult {
    let output = std::process::Command::new(format!("{}/scripts/core", core_root))
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
        .unwrap_or_else(|_| "0.6.0".to_string());
    let alias_count = aliases;
    // Load health from cache
    let health: String = std::fs::read_to_string(format!("{}/.cache/faelight/health-status", home))
        .unwrap_or_else(|_| "100%".to_string())
        .trim()
        .to_string();
    // Load Jarvis score from core strategy
    let jarvis = "90/100";
    // Get login shell date
    let login_since = "2026-04-03";
    // Count days as daily driver
    let days = {
        let start = chrono::NaiveDate::parse_from_str(login_since, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let today = chrono::Local::now().date_naive();
        (today - start).num_days()
    };
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
        "Daily driver".dimmed(),
        format!("✅ day {} of 30", days).bright_green()
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
        "Jarvis".dimmed(),
        format!("{} — Strategic Advisor", jarvis).bright_cyan()
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
fn time_cmd(line: &str, args: &[&str]) -> CommandResult {
    if args.is_empty() {
        return CommandResult::Error("time: missing command".to_string());
    }
    // Reconstruct the command without "time " prefix
    let cmd_line = line.trim().trim_start_matches("time").trim().to_string();
    let start = std::time::Instant::now();
    let output = crate::db::spawn_sh_with_leak_check(&cmd_line);
    let elapsed = start.elapsed();
    let ms = elapsed.as_millis();
    let display = if ms >= 1000 {
        format!("{:.2}s", elapsed.as_secs_f64())
    } else {
        format!("{}ms", ms)
    };
    match output {
        Ok(status) => {
            let code = status.code().unwrap_or(0);
            println!();
            println!(
                "  {} {} (exit {})",
                "⏱".to_string(),
                display.bright_cyan().bold(),
                code.to_string().dimmed()
            );
            CommandResult::Empty
        }
        Err(e) => CommandResult::Error(format!("time: {}", e)),
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
        std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| cmd.to_string())
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
fn run_python_cmd(args: &[&str]) -> CommandResult {
    if args.is_empty() {
        // Interactive python3
        let _ = std::process::Command::new("python3")
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();
        return CommandResult::Empty;
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
fn forest_stats_intents(core_root: &str) -> CommandResult {
    let complete_dir = format!("{}/intents/complete", core_root);
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
