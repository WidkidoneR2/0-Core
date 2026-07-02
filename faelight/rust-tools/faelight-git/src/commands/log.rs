//! Commit history — native git2, visual graph

use crate::git::GitRepo;
use anyhow::Result;
use colored::*;
use std::io::{self, Write};

pub fn run(count: Option<usize>) -> Result<()> {
    let repo = GitRepo::open()?;
    let n = count.unwrap_or(20);
    let branch = repo.current_branch()?;
    let (ahead, behind) = repo.ahead_behind()?;
    let entries = repo.log(n)?;

    if entries.is_empty() {
        println!("{}", "  ℹ️  No commits found".yellow());
        return Ok(());
    }

    // ── Header ────────────────────────────────────────────────
    println!("{}", "🌲 faelight-git log".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());

    // Branch status line
    let sync = match (ahead, behind) {
        (0, 0) => "synced".green().to_string(),
        (a, 0) => format!("↑{} ahead", a).yellow().to_string(),
        (0, b) => format!("↓{} behind", b).red().to_string(),
        (a, b) => format!("↑{} ↓{}", a, b).red().to_string(),
    };
    println!("  {} {}  {}", " ".dimmed(), branch.green().bold(), sync);
    println!();

    // ── Commit graph ──────────────────────────────────────────
    let total = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == total - 1;

        // Graph line character
        let glyph = if i == 0 {
            "◉".cyan().bold() // HEAD
        } else {
            "○".dimmed()
        };

        let connector = if is_last {
            " ".to_string()
        } else {
            "│".dimmed().to_string()
        };

        // Detect intent reference in message
        let has_intent = entry.message.contains("Intent:");
        let intent_mark = if has_intent {
            " 󰍉".cyan().to_string()
        } else {
            String::new()
        };

        // Detect conventional commit type for color
        let message = colorize_message(&entry.message);

        println!(
            "  {} {} {}  {}  {}{}",
            glyph,
            entry.hash.yellow().bold(),
            entry.time_ago.dimmed(),
            entry.author.cyan().dimmed(),
            message,
            intent_mark,
        );

        if !is_last {
            println!("  {}", connector);
        }
    }

    // ── Footer ────────────────────────────────────────────────
    println!();
    println!("{}", "━".repeat(60).dimmed());
    println!(
        "  {} {}  {} to show more",
        "showing".dimmed(),
        format!("{} commits", total).white(),
        "faelight-git log -n <count>".to_string().dimmed()
    );
    println!();

    // ── Interactive diff ──────────────────────────────────────
    print!("  Inspect a commit? (hash or Enter to skip): ");
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    let response = response.trim();

    if !response.is_empty() {
        show_commit(response)?;
    }

    Ok(())
}

/// Color commit message by conventional commit type
fn colorize_message(msg: &str) -> String {
    let prefixes = [
        ("feat:", "cyan"),
        ("fix:", "green"),
        ("chore:", "yellow"),
        ("refactor:", "blue"),
        ("docs:", "white"),
        ("test:", "magenta"),
        ("perf:", "cyan"),
        ("style:", "white"),
        ("build:", "yellow"),
        ("ci:", "yellow"),
        ("revert:", "red"),
        ("BREAKING:", "red"),
    ];

    for (prefix, color) in prefixes {
        if let Some(rest) = msg.strip_prefix(prefix) {
            let typed = match color {
                "cyan" => prefix.cyan().bold().to_string(),
                "green" => prefix.green().bold().to_string(),
                "yellow" => prefix.yellow().bold().to_string(),
                "blue" => prefix.blue().bold().to_string(),
                "magenta" => prefix.magenta().bold().to_string(),
                "red" => prefix.red().bold().to_string(),
                _ => prefix.white().bold().to_string(),
            };
            return format!("{}{}", typed, rest.white());
        }
    }

    msg.white().to_string()
}

fn show_commit(hash: &str) -> Result<()> {
    let repo = GitRepo::open()?;

    println!();
    println!("{}", format!("  ── commit {} ──", hash).cyan().bold());

    // Try to get diff stat from our native impl
    if let Ok(stat) = repo.diff_stat(hash) {
        println!("  {}", stat.dimmed())
    }

    println!();

    // Shell out only for the colored diff display — no native alternative needed
    let show = std::process::Command::new("git")
        .args(["show", "--color=always", "--stat", hash])
        .output()?;

    if show.status.success() {
        print!("{}", String::from_utf8_lossy(&show.stdout));
    } else {
        println!("{}", format!("  ❌ Commit '{}' not found", hash).red());
    }

    Ok(())
}
