// INT-253 v2: gt -- Linear git workflow, like fg sync but smarter
// No TUI. Just clean guided terminal output with forest colors.
use colored::Colorize;
use std::io::{self, Write};
use std::process::Command;
pub fn run_git_tui(core_root: &str, active_intent: Option<&str>) {
    println!();
    println!("  {} gt -- forest git workflow", "🌿".green());
    println!("  {}", "━".repeat(48).dimmed());
    println!();
    // Phase 1: Status
    let status = get_status(core_root);
    let staged = status
        .iter()
        .filter(|(s, _)| s.starts_with('A') || s.starts_with('M') || s.starts_with('D'))
        .count();
    let unstaged = status
        .iter()
        .filter(|(s, _)| s.ends_with('M') || s.ends_with('D'))
        .count();
    let untracked = status.iter().filter(|(s, _)| *s == "??").count();
    if status.is_empty() {
        println!("  {} Working tree clean -- nothing to commit", "✅".green());
        println!();
        // Check ahead/behind
        let ab = get_ahead_behind(core_root);
        if !ab.is_empty() {
            println!("  {} {}", "📤".yellow(), ab);
            if prompt("  Push to origin? (y/n): ") {
                do_push(core_root);
            }
        }
        println!();
        return;
    }
    // Show file summary
    println!("  {} Status", "📊".bright_cyan());
    if staged > 0 {
        println!("    {} staged", staged.to_string().green());
    }
    if unstaged > 0 {
        println!("    {} unstaged", unstaged.to_string().yellow());
        for (s, path) in status
            .iter()
            .filter(|(s, _)| s.ends_with('M') || s.ends_with('D'))
        {
            println!("      {} {}", s.yellow(), path.dimmed());
        }
    }
    if untracked > 0 {
        println!("    {} untracked", untracked.to_string().dimmed());
        for (_, path) in status.iter().filter(|(s, _)| *s == "??") {
            println!("      {} {}", "?".dimmed(), path.dimmed());
        }
    }
    println!();
    // Phase 2: Stage all
    if prompt("  Stage all changes? (y/n): ") {
        let out = Command::new("git")
            .args(["-C", core_root, "add", "-A"])
            .status();
        match out {
            Ok(s) if s.success() => println!("  {} All changes staged", "✅".green()),
            _ => {
                println!("  {} Stage failed", "❌".red());
                return;
            }
        }
    } else {
        println!("  {} Cancelled", "○".dimmed());
        return;
    }
    println!();
    // Phase 3: Commit message
    let prefix = active_intent
        .map(|i| format!("{}: ", i))
        .unwrap_or_default();
    print!(
        "  {} Commit message ({}): ",
        "💬".bright_cyan(),
        prefix.dimmed()
    );
    io::stdout().flush().ok();
    let mut msg = String::new();
    io::stdin().read_line(&mut msg).ok();
    let msg = msg.trim();
    if msg.is_empty() {
        println!("  {} Empty message -- cancelled", "○".dimmed());
        return;
    }
    let full_msg = if prefix.is_empty() {
        msg.to_string()
    } else {
        format!("{}{}", prefix, msg)
    };
    // Preview
    println!();
    println!("  {}", "─".repeat(48).dimmed());
    println!("  {}", full_msg.white());
    println!("  {}", "─".repeat(48).dimmed());
    println!();
    if !prompt("  Create commit? (y/n): ") {
        println!("  {} Cancelled", "○".dimmed());
        return;
    }
    // Phase 4: Commit
    let commit = Command::new("git")
        .args(["-C", core_root, "commit", "-m", &full_msg])
        .status();
    match commit {
        Ok(s) if s.success() => println!("  {} Committed", "✅".green()),
        _ => {
            println!("  {} Commit failed", "❌".red());
            return;
        }
    }
    println!();
    // Phase 5: Push
    if prompt("  Push to origin? (y/n): ") {
        do_push(core_root);
    }
    println!();
    println!("  {} {}", "🌲".green(), "The forest remembers.".dimmed());
    println!();
}
fn get_status(core_root: &str) -> Vec<(String, String)> {
    let out = match Command::new("git")
        .args(["-C", core_root, "status", "--porcelain"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.len() >= 3)
        .map(|l| {
            let status = l[..2].trim().to_string();
            let path = l[3..].trim().to_string();
            (status, path)
        })
        .collect()
}
fn get_ahead_behind(core_root: &str) -> String {
    let out = Command::new("git")
        .args(["-C", core_root, "status", "--short", "--branch"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .to_string()
        })
        .unwrap_or_default();
    if out.contains("ahead") || out.contains("behind") {
        out.split('[')
            .nth(1)
            .and_then(|s| s.split(']').next())
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    }
}
fn do_push(core_root: &str) {
    println!("  {} Pushing...", "📤".yellow());
    let _ = Command::new("git").args(["-C", core_root, "push"]).status();
}
fn prompt(msg: &str) -> bool {
    print!("{}", msg);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_lowercase() == "y"
}
