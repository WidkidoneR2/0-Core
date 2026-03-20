// faelight-shell v0.1.0
// Forest-native structured shell environment
// INT-120 Phase 1 — REPL skeleton
//
// "A forest deserves a shell that knows it is a forest."
// "Not text streams. Not configuration. Structured wisdom."

mod commands;
mod db;
mod output;
use colored::Colorize;
mod prompt;
mod value;
mod completion;
mod schema;
mod nl;

use anyhow::Result;
use rustyline::{error::ReadlineError, Editor};

fn main() -> Result<()> {
    // Connect to state.db
    let db = db::ForestDb::open()?;
    let core_root = db.core_root();

    // Print welcome
    print_welcome(&core_root);
    let _session_start = std::time::Instant::now();
    let mut _session_commands: usize = 0;
    let mut _session_pipelines: usize = 0;

    // Build readline editor
    let helper = completion::ForestHelper::new();
    let mut rl: Editor<completion::ForestHelper, _> = Editor::new()?;
    rl.set_helper(Some(helper));

    // Load history from state.db
    db.load_history(&mut rl);

    // REPL loop
    loop {
        let prompt_str = prompt::render_line(&db);

        match rl.readline(&prompt_str) {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() { continue; }

                // Save to history
                let _ = rl.add_history_entry(&line);
                _session_commands += 1;
                if line.contains(" | ") { _session_pipelines += 1; }
                db.save_history_entry(&line);

                // Natural language ?prefix
                if line.starts_with('?') && line.len() > 1 {
                    let query = line[1..].trim();
                    match nl::translate(query) {
                        Some(t) => {
                            print!("{}", nl::render_translation(&t));
                            use std::io::BufRead;
                            let stdin = std::io::stdin();
                            let answer = stdin.lock().lines().next()
                                .and_then(|l| l.ok())
                                .unwrap_or_default()
                                .trim()
                                .to_lowercase();
                            if answer == "y" || answer.is_empty() {
                                println!();
                                match commands::execute(&t.pipeline, &db, &core_root) {
                                    commands::CommandResult::Value(v) => println!("{}", v.render()),
                                    commands::CommandResult::Output(o) => println!("{}", o),
                                    commands::CommandResult::Error(e) => eprintln!("  ✗ {}", e),
                                    _ => {}
                                }
                            } else {
                                println!("  ○ cancelled");
                            }
                        }
                        None => {
                            eprintln!("  ✗ no pattern matched — try: ?memory hogs, ?biggest files");
                        }
                    }
                    continue;
                }

                // Execute
                // Expand aliases before pipeline parsing
                let first_word = line.split_whitespace().next().unwrap_or("").to_lowercase();
                let line = if let Some(aliased) = db.get_alias(&first_word) {
                    let rest: String = line.splitn(2, ' ').nth(1).map(|s| format!(" {}", s)).unwrap_or_default();
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
                        if ch == '"' { inside = !inside; }
                        if ch == '|' && inside { last_pipe_in_quotes = true; }
                    }
                    last_pipe_in_quotes
                };
                let has_pipe = !in_quotes && line.contains(" | ");
                let pipeline_ops = if has_pipe {
                    value::parse_pipeline(&line)
                } else {
                    vec![]
                };
                let base_cmd = if has_pipe {
                    line.splitn(2, " | ").next().unwrap_or(&line).to_string()
                } else {
                    line.to_string()
                };

                // Phase 9 — Streaming: detect | watch at end of pipeline
                let is_streaming = pipeline_ops.last()
                    .map(|op| matches!(op, value::PipeOp::Watch { .. }))
                    .unwrap_or(false);

                if is_streaming {
                    // Strip watch from pipeline ops
                    let stream_ops: Vec<value::PipeOp> = pipeline_ops.iter()
                        .take(pipeline_ops.len() - 1)
                        .cloned()
                        .collect();
                    let interval = pipeline_ops.last()
                        .and_then(|op| if let value::PipeOp::Watch { interval } = op {
                            Some(*interval)
                        } else { None })
                        .unwrap_or(2);

                    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                    // Use a background thread to watch for Enter key to stop
                    let r = running.clone();
                    std::thread::spawn(move || {
                        let mut input = String::new();
                        let _ = std::io::stdin().read_line(&mut input);
                        r.store(false, std::sync::atomic::Ordering::SeqCst);
                    });

                    println!("  {} {} {}",
                        "streaming".bright_cyan(),
                        base_cmd.dimmed(),
                        format!("({}s interval — Ctrl+C to stop)", interval).dimmed()
                    );

                    while running.load(std::sync::atomic::Ordering::SeqCst) {
                        print!("[2J[H"); // clear screen
                        let now = chrono::Local::now().format("%H:%M:%S").to_string();
                        println!("  {} {} {}",
                            "🌲 live".bright_cyan(),
                            base_cmd.dimmed(),
                            now.dimmed()
                        );
                        println!("{}", "━".repeat(52).dimmed());
                        match commands::execute(&base_cmd, &db, &core_root) {
                            commands::CommandResult::Value(v) => {
                                let result = if !stream_ops.is_empty() {
                                    value::apply_pipeline(v, &stream_ops)
                                } else { v };
                                println!("{}", result.render());
                            }
                            commands::CommandResult::Output(out) => println!("{}", out),
                            _ => {}
                        }
                        for _ in 0..(interval * 10) {
                            if !running.load(std::sync::atomic::Ordering::SeqCst) { break; }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                    println!("
  {} stream stopped", "○".dimmed());
                    continue;
                }

                match commands::execute(&base_cmd, &db, &core_root) {
                    commands::CommandResult::Exit => break,
                    commands::CommandResult::Value(v) if !pipeline_ops.is_empty() => {
                        let result = value::apply_pipeline(v, &pipeline_ops);
                        println!("{}", result.render());
                    }
                    commands::CommandResult::Value(v) => println!("{}", v.render()),
                    commands::CommandResult::Output(out) => println!("{}", out),
                    commands::CommandResult::Empty => {}
                    commands::CommandResult::Error(e) => {
                        eprintln!("{} {}", colored::Colorize::bright_red("✗"), e);
                    }
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

    println!("{}", colored::Colorize::dimmed("  🌲 The forest remembers."));
    Ok(())
}

fn print_welcome(core_root: &str) {
    use colored::Colorize;
    use std::path::PathBuf;

    let root = PathBuf::from(core_root);

    let version = std::fs::read_to_string(root.join("00-meta/VERSION"))
        .unwrap_or_else(|_| "unknown".into()).trim().to_string();

    let changelog = std::fs::read_to_string(root.join("00-meta/CHANGELOG.md"))
        .unwrap_or_default();
    let theme = changelog.lines()
        .find(|l| l.starts_with(&format!("## [{}]", version)))
        .and_then(|l| l.split(" — ").nth(1))
        .and_then(|s| s.split('(').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "The Living Forest".to_string());

    let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
        .unwrap_or_default().trim().to_string();

    let health_num: u32 = std::fs::read_to_string(root.join("runtime/cache/health.txt"))
        .unwrap_or_else(|_| "95".into()).trim()
        .trim_end_matches('%').parse().unwrap_or(95);

    let health_display = if health_num >= 95 {
        format!("{}% ✅", health_num).bright_green().to_string()
    } else if health_num >= 80 {
        format!("{}% ⚠", health_num).yellow().to_string()
    } else {
        format!("{}% ❌", health_num).bright_red().to_string()
    };

    let complete_count = std::fs::read_dir(root.join("intents/complete"))
        .map(|d| d.count()).unwrap_or(0);
    let planned_count = std::fs::read_dir(root.join("intents/future"))
        .map(|d| d.count()).unwrap_or(0);
    let tool_count = std::fs::read_dir(root.join("scripts"))
        .map(|d| d.flatten().filter(|e| e.path().is_file()).count())
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
            []
        );
        let last_idx: usize = conn.query_row(
            "SELECT value FROM shell_state WHERE key='last_quote_idx'",
            [], |r| r.get::<_,String>(0)
        ).ok().and_then(|v| v.parse().ok()).unwrap_or(999);

        // Pick next quote (skip last shown)
        let next_idx = {
            let mut idx = (last_idx + 1) % quotes.len();
            if idx == last_idx { idx = (idx + 1) % quotes.len(); }
            idx
        };
        let _ = conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_quote_idx', ?1)",
            rusqlite::params![next_idx.to_string()]
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
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  {} intents complete  ·  {} commits",
        complete_count.to_string().bright_white(),
        commits.bright_white()
    );
    println!("  {}  ·  {} tools  ·  {} planned",
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
            let stale: Vec<&str> = t.lines()
                .filter(|l| l.starts_with("name = "))
                .filter_map(|l| l.split('"').nth(1))
                .collect();
            stale.first().map(|s| s.to_string()).unwrap_or_default()
        })
        .unwrap_or_default();

    // Show today's focus from most recent in-progress intent
    let focus_intent = std::fs::read_dir(root.join("intents/future"))
        .ok()
        .and_then(|mut d| d.next())
        .and_then(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy()
            .trim_start_matches(|c: char| c.is_ascii_digit() || c == '-')
            .trim_end_matches(".md")
            .replace('-', " ")
            .to_string()
        );

    if let Some(ref focus) = focus_intent {
        println!("  {} {}", "Today:".dimmed(), focus.bright_white());
    }
    println!();
    println!("  {} for commands  ·  {} to exit",
        "help".bright_cyan(), "q".dimmed()
    );
    println!();
}
