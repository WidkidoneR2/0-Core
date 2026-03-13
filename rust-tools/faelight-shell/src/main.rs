// faelight-shell v0.1.0
// Forest-native structured shell environment
// INT-120 Phase 1 — REPL skeleton
//
// "A forest deserves a shell that knows it is a forest."
// "Not text streams. Not configuration. Structured wisdom."

mod commands;
mod db;
mod output;
mod prompt;
mod value;

use anyhow::Result;
use rustyline::{error::ReadlineError, DefaultEditor};

fn main() -> Result<()> {
    // Connect to state.db
    let db = db::ForestDb::open()?;
    let core_root = db.core_root();

    // Print welcome
    print_welcome(&core_root);

    // Build readline editor
    let mut rl = DefaultEditor::new()?;

    // Load history from state.db
    db.load_history(&mut rl);

    // REPL loop
    loop {
        let prompt_str = prompt::render(&db);

        match rl.readline(&prompt_str) {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() { continue; }

                // Save to history
                let _ = rl.add_history_entry(&line);
                db.save_history_entry(&line);

                // Execute
                // Parse pipeline if | present
                let has_pipe = line.contains(" | ");
                let pipeline_ops = if has_pipe {
                    value::parse_pipeline(&line)
                } else {
                    vec![]
                };
                let base_cmd = if has_pipe {
                    line.splitn(2, " | ").next().unwrap_or(&line).to_string()
                } else {
                    line.clone()
                };

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
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("exit");
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
    let version = std::fs::read_to_string(
        std::path::PathBuf::from(core_root).join("00-meta/VERSION")
    ).unwrap_or_else(|_| "unknown".into());
    let version = version.trim();

    println!();
    println!("{}", colored::Colorize::bright_cyan(
        "  ╭─ 🌲 faelight-shell ─────────────────────────────────╮"
    ));
    println!("  │  Forest-native shell  {}                    │",
        colored::Colorize::bright_white(version));
    println!("  │  Type {} for commands                            │",
        colored::Colorize::bright_cyan("help"));
    println!("{}", colored::Colorize::bright_cyan(
        "  ╰─────────────────────────────────────────────────────╯"
    ));
    println!();
}
