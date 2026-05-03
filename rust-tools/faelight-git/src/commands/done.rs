//! fg done -- stage all, commit with active intent prefix, push. No prompts.
//! INT-256: the fastest path from work to pushed commit.
use crate::git::GitRepo;
use crate::is_locked;
use anyhow::{bail, Result};
use colored::*;
fn get_active_intent() -> Option<String> {
    // Scan intents/future/ for in-progress intents -- same as fsh prompt
    let home = std::env::var("HOME").ok()?;
    let future_dir = format!("{}/0-core/intents/future", home);
    let entries = std::fs::read_dir(&future_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") { continue; }
        let content = std::fs::read_to_string(entry.path()).ok()?;
        if content.contains("status: in-progress") {
            // Extract ID from filename like "256-faelight-git..."
            let id = name.split('-').next().unwrap_or("").to_string();
            return Some(format!("INT-{}", id));
        }
    }
    None
}
pub fn run(extra: Option<&str>) -> Result<()> {
    if is_locked() {
        bail!("Core is locked. Run 'unlock-core' first.");
    }
    let repo = GitRepo::open()?;
    let status = repo.status()?;
    if status.is_empty() {
        println!("{}", "  ✅ Nothing to commit -- tree is clean".green());
        return Ok(());
    }
    println!("{}", "🌲 fg done".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    // Show staged files
    for f in &status.files {
        println!("  {} {}", f.state.symbol().yellow(), f.path.dimmed());
    }
    println!();
    // Build commit message from active intent
    let message = if let Some(focus) = get_active_intent() {
        // focus is like "INT-245" or "245"
        let id_part = focus.trim_start_matches("INT-").split_whitespace().next().unwrap_or(&focus);
        if let Some(extra) = extra {
            format!("INT-{}: {}", id_part, extra)
        } else {
            format!("INT-{}: progress", id_part)
        }
    } else if let Some(extra) = extra {
        extra.to_string()
    } else {
        bail!("No active intent and no message provided. Use: fg done \"message\"");
    };
    // Stage all
    repo.stage_all()?;
    println!("  {} All changes staged", "✅".green());
    // Commit
    let hash = repo.commit(&message)?;
    println!(
        "  {} {} {}",
        "✅".green(),
        hash[..8.min(hash.len())].yellow().bold(),
        message.white()
    );
    // Push
    println!("  {} Pushing...", "→".cyan());
    let push = std::process::Command::new("git").arg("push").status()?;
    if push.success() {
        println!("{}", "  🚀 Pushed to origin".green().bold());
    } else {
        println!("{}", "  ❌ Push failed -- run 'git push' manually".red());
    }
    println!("{}", "━".repeat(52).dimmed());
    println!("{}", "  🌲 The forest remembers.".dimmed());
    Ok(())
}
