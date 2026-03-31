// faelight-shell v0.1.0
// Forest-native structured shell environment
// INT-120 Phase 1 — REPL skeleton
//
// "A forest deserves a shell that knows it is a forest."
// "Not text streams. Not configuration. Structured wisdom."

mod commands;
mod db;
mod exec;
mod output;
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
use rustyline::{error::ReadlineError, CompletionType, Config, EditMode, Editor};
use std::collections::HashMap;

/// Split a line on `;` separators, respecting quoted strings.
/// "cmd1; cmd2; cmd3" → ["cmd1", "cmd2", "cmd3"]
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

/// Detect and strip redirection from a command line.
/// Returns (cleaned_line, Some((path, append))) or (line, None)
fn detect_redirect(line: &str) -> (String, Option<(String, bool)>) {
    // Match >> before > (order matters)
    if let Some(idx) = line.rfind(" >> ") {
        let cmd = line[..idx].trim().to_string();
        let path = line[idx + 4..].trim().to_string();
        return (cmd, Some((path, true)));
    }
    if let Some(idx) = line.rfind(" > ") {
        let cmd = line[..idx].trim().to_string();
        let path = line[idx + 3..].trim().to_string();
        return (cmd, Some((path, false)));
    }
    (line.to_string(), None)
}

/// Expand $VAR and ${VAR} references in a line.
/// Reads from shell_vars first, then std::env.
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
    input.lines().map(|line| {
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
                '"'  if !in_single => in_double = !in_double,
                '#'  if !in_single && !in_double => {
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
    // Connect to state.db
    let db = db::ForestDb::open()?;
    let core_root = db.core_root();

    // Phase 15 — load config.fsh
    config::ensure_default();
    let cfg = config::load();

    // Print welcome
    print_welcome(&core_root);
    let _session_start = std::time::Instant::now();
    let mut _session_commands: usize = 0;
    let mut _session_pipelines: usize = 0;

    // Phase 16 — configured interactive editor
    let rl_config = Config::builder()
        .max_history_size(10000)?
        .history_ignore_dups(true)?
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let helper = completion::ForestHelper::new();
    let mut rl: Editor<completion::ForestHelper, _> = Editor::with_config(rl_config)?;
    rl.set_helper(Some(helper));
    // Ctrl+L handled in REPL loop via clear command

    // Apply config aliases and settings
    config::apply(&cfg, &db);

    // Load history from state.db
    db.load_history(&mut rl);

    // Phase 8 — job table
    let mut job_table = jobs::JobTable::new();

    // Phase 17 — prompt context tracking
    let last_duration_ms: Option<u64> = None;
    let last_exit_code: Option<i32> = None;

    // Phase 10 — shell variable table
    let mut shell_vars: HashMap<String, String> = HashMap::new();

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

        let prompt_str = prompt::render_line(&db);

        match rl.readline(&prompt_str) {
            Ok(line) => {
                // Strip comments before any processing
                let line = strip_comments(line.trim());
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                // Save to history
                let _ = rl.add_history_entry(&line);
                _session_commands += 1;
                if line.contains(" | ") {
                    _session_pipelines += 1;
                }
                db.save_history_entry(&line);
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
                                            println!("  {} usage: flow focus INT-NNN", "\u{2717}".bright_red());
                                        } else if !arg.starts_with("INT-") {
                                            println!("  {} must be INT-NNN format", "\u{2717}".bright_red());
                                        } else {
                                            fdb.set_focus_intent(arg);
                                            println!("  {} focus set -> {}", "\u{1f332}".normal(), arg.bright_green().bold());
                                        }
                                    }
                                    "clear" => {
                                        fdb.clear_focus_intent();
                                        println!("  {} focus cleared", "\u{25cb}".dimmed());
                                    }
                                    "status" | "" => {
                                        match fdb.get_focus_intent() {
                                            Some(intent) => {
                                                println!();
                                                println!("  {} {}", "Active focus:".dimmed(), intent.bright_green().bold());
                                                println!("  {} flow clear  to release", "hint:".dimmed());
                                                println!();
                                            }
                                            None => {
                                                println!("  {} no active focus -- use: flow focus INT-NNN", "\u{25cb}".dimmed());
                                            }
                                        }
                                    }
                                    _ => {
                                        println!("  {} unknown subcommand: {}", "\u{2717}".bright_red(), sub);
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
                                    println!("  {} Core is LOCKED — editing blocked", "🔒".normal());
                                    println!("  {} No commits, pushes or changes allowed while locked", "✗".bright_red());
                                    println!("  {} Run: unlock-core  — then make your changes", "→".bright_cyan());
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
                                    println!("  {} Core is LOCKED — editing blocked", "🔒".normal());
                                    println!("  {} Run: unlock-core  — then commit", "→".bright_cyan());
                                    println!();
                                    continue 'repl;
                                }
                                _ => {}
                            }
                        }
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

                    // Phase 10 — expand $VARS before alias resolution
                    let line = expand_vars(line, &shell_vars);
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
                    let in_quotes = line.contains('"') && {
                        let mut inside = false;
                        let mut last_pipe_in_quotes = false;
                        for ch in line.chars() {
                            if ch == '"' {
                                inside = !inside;
                            }
                            if ch == '|' && inside {
                                last_pipe_in_quotes = true;
                            }
                        }
                        last_pipe_in_quotes
                    };
                    let has_pipe = !in_quotes && line.contains(" | ");
                    let pipeline_ops = if has_pipe {
                        value::parse_pipeline(line)
                    } else {
                        vec![]
                    };
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
                        let id = line
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(1);
                        job_table.fg(id);
                        continue;
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

                    // Phase 13 — Redirection: detect > and >> before execution
                    let (line, redirect) = detect_redirect(line);
                    let line = line.as_str();
                    // Re-parse pipeline after stripping redirect
                    let in_quotes2 = line.contains('"') && {
                        let mut inside = false;
                        let mut last_pipe_in_quotes = false;
                        for ch in line.chars() {
                            if ch == '"' {
                                inside = !inside;
                            }
                            if ch == '|' && inside {
                                last_pipe_in_quotes = true;
                            }
                        }
                        last_pipe_in_quotes
                    };
                    let has_pipe2 = !in_quotes2 && line.contains(" | ");
                    let pipeline_ops = if has_pipe2 {
                        value::parse_pipeline(line)
                    } else {
                        pipeline_ops
                    };
                    let base_cmd = if has_pipe2 {
                        line.split(" | ").next().unwrap_or(line).to_string()
                    } else {
                        base_cmd
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
                        let fc = base_cmd.split_whitespace().next().unwrap_or("").to_lowercase();
                        fc == "yazi" || fc == "faelight-fm"
                    };
                    let base_cmd = if is_fm_cmd {
                        format!("{} --cwd-file {}", base_cmd, fm_cwd_file.display())
                    } else {
                        base_cmd
                    };
                    let cmd_output: Option<String> =
                        match exec::execute_with_context(&base_cmd, &db, &core_root, &cfg.before_rules) {
                            commands::CommandResult::Exit => break 'repl,
                            commands::CommandResult::Value(v) if !pipeline_ops.is_empty() => {
                                let result = value::apply_pipeline(v, &pipeline_ops);
                                Some(result.render())
                            }
                            commands::CommandResult::Value(v) => Some(v.render()),
                            commands::CommandResult::Output(out) => Some(out),
                            commands::CommandResult::Empty => None,
                            commands::CommandResult::Error(e) => {
                                eprintln!("{} {}", colored::Colorize::bright_red("✗"), e);
                                None
                            }
                        };

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

    // Save session state on exit
    session::SessionMemory::save(&core_root, None);
    println!(
        "{}",
        colored::Colorize::dimmed("  🌲 The forest remembers.")
    );
    Ok(())
}

fn print_welcome(core_root: &str) {
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
                .join(".cache/faelight/health-status"))
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
        let categories = ["complete", "decisions", "experiments", "philosophy",
                          "future", "cancelled", "deferred", "incidents", "active"];
        let mut complete = 0usize;
        let mut planned = 0usize;
        for cat in &categories {
            if let Ok(entries) = std::fs::read_dir(intent_dir.join(cat)) {
                for entry in entries.flatten() {
                    if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            if content.contains("status: complete") { complete += 1; }
                            else if content.contains("status: planned") { planned += 1; }
                        }
                    }
                }
            }
        }
        (complete, planned)
    };
    // Count tools from registry — mirrors doctor check_path_resilience logic exactly
    let tool_count = std::fs::read_to_string(root.join("01-registry/tools.toml"))
        .map(|t| t.lines()
            .filter(|l| l.starts_with("name = "))
            .count())
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
    let db_path = root.join("runtime/state.db");
    let quote = if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        // Get last shown index
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS shell_state (key TEXT PRIMARY KEY, value TEXT)",
            [],
        );
        let last_idx: usize = conn
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
        let _ = conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_quote_idx', ?1)",
            rusqlite::params![next_idx.to_string()],
        );
        quotes[next_idx]
    } else {
        let commit_num: usize = commits.trim().parse().unwrap_or(0);
        quotes[commit_num % quotes.len()]
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
                    if !content.contains("status: in-progress") { return None; }
                    Some(e.file_name()
                        .to_string_lossy()
                        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '-')
                        .trim_end_matches(".md")
                        .replace('-', " ")
                        .to_string())
                })
                .collect();
            in_progress.sort();
            if in_progress.is_empty() { None } else {
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
                        db.set_focus_intent(&intent_key);
                    }
                }
            }
        }
    }
    println!();
    // Session memory + digest
    if let Some(mem) = session::SessionMemory::load(core_root) {
        // Phase 23 — restore last working directory
        if let Some(ref last_dir) = mem.last_dir {
            let path = std::path::Path::new(last_dir);
            if path.exists() && path.is_dir() {
                let _ = std::env::set_current_dir(path);
            }
        }
        let msg = session::render(&mem, core_root);
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

    println!(
        "  {} for commands  ·  {} to exit",
        "help".bright_cyan(),
        "q".dimmed()
    );
    println!();
}
