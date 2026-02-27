//! Intent-aware commit — stages, verifies, and commits

use crate::git::GitRepo;
use crate::is_locked;
use crate::risk::RiskScore;
use anyhow::{bail, Result};
use colored::*;
use std::io::{self, Write};

pub fn run(intent: Option<String>, no_intent: bool) -> Result<()> {
    let repo = GitRepo::open()?;

    // ── Guard: core must be unlocked ──────────────────────────
    if is_locked() {
        bail!("Core is locked. Run 'unlock-core' before committing.");
    }

    // ── Guard: must have changes ───────────────────────────────
    let status = repo.status()?;
    if status.is_empty() {
        println!("{}", "✅ Working tree clean — nothing to commit".green());
        return Ok(());
    }

    // ── Show current state ────────────────────────────────────
    println!("{}", "🌲 faelight-git commit".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    let staged = status.staged_files();
    let unstaged = status.unstaged_files();
    let untracked = status.untracked_files();

    // Show staged files
    if !staged.is_empty() {
        println!("{}", "  Staged".green().bold());
        for f in &staged {
            println!("  {} {}", "●".green(), f.path.green());
        }
        println!();
    }

    // Show unstaged files
    if !unstaged.is_empty() {
        println!("{}", "  Unstaged".yellow().bold());
        for f in &unstaged {
            println!("  {} {}", "○".yellow(), f.path.yellow());
        }
        println!();
    }

    // Show untracked
    if !untracked.is_empty() {
        println!("{}", "  Untracked".dimmed().bold());
        for f in &untracked {
            println!("  {} {}", "?".dimmed(), f.path.dimmed());
        }
        println!();
    }

    // ── Risk check ────────────────────────────────────────────
    let risk = RiskScore::calculate(&repo)?;
    println!("  {} {} {}/100", "risk".dimmed(), risk.emoji(), risk.total);
    println!("{}", "━".repeat(52).dimmed());
    println!();

    // ── Stage unstaged files? ─────────────────────────────────
    if staged.is_empty() && (!unstaged.is_empty() || !untracked.is_empty()) {
        print!("  Stage all changes? (y/n): ");
        io::stdout().flush()?;
        let mut ans = String::new();
        io::stdin().read_line(&mut ans)?;
        if ans.trim().to_lowercase() != "y" {
            println!("{}", "  ⚠️  Commit cancelled — nothing staged".yellow());
            return Ok(());
        }
        repo.stage_all()?;
        println!("{}", "  ✅ All changes staged".green());
        println!();
    } else if !unstaged.is_empty() {
        print!("  Also stage {} unstaged file(s)? (y/n): ", unstaged.len());
        io::stdout().flush()?;
        let mut ans = String::new();
        io::stdin().read_line(&mut ans)?;
        if ans.trim().to_lowercase() == "y" {
            repo.stage_all()?;
            println!("{}", "  ✅ All changes staged".green());
            println!();
        }
    }

    // ── Intent ────────────────────────────────────────────────
    let intent_ref = if no_intent {
        println!(
            "{}",
            "  ⚠️  Proceeding without intent (--no-intent)".yellow()
        );
        None
    } else if let Some(ref i) = intent {
        println!("  {} linked to intent {}", "✅".green(), i.cyan());
        Some(i.clone())
    } else {
        print!("  Intent reference (INT-0XX or 'skip'): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input == "skip" || input.is_empty() {
            println!("{}", "  ⚠️  Committing without intent".yellow());
            None
        } else {
            println!("  {} linked to intent {}", "✅".green(), input.cyan());
            Some(input)
        }
    };

    // ── Commit message ────────────────────────────────────────
    println!();
    print!("  Commit message: ");
    io::stdout().flush()?;
    let mut message = String::new();
    io::stdin().read_line(&mut message)?;
    let message = message.trim().to_string();

    if message.is_empty() {
        bail!("Commit cancelled — empty message");
    }

    // Build full message with intent footer if provided
    let full_message = match intent_ref {
        Some(ref i) => format!("{}\n\nIntent: {}", message, i),
        None => message.clone(),
    };

    // ── Preview ───────────────────────────────────────────────
    println!();
    println!("{}", "  Preview".dimmed());
    println!("{}", "  ─".repeat(26).dimmed());
    println!("  {}", full_message.replace('\n', "\n  ").white().bold());
    println!("{}", "  ─".repeat(26).dimmed());
    println!();

    print!("  Confirm commit? (y/n): ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() != "y" {
        println!("{}", "  ⚠️  Commit cancelled".yellow());
        return Ok(());
    }

    // ── Commit ────────────────────────────────────────────────
    let hash = repo.commit(&full_message)?;
    println!();
    println!(
        "  {} commit {} {}",
        "✅".green(),
        hash.yellow().bold(),
        message.white()
    );

    // ── Push? ─────────────────────────────────────────────────
    println!();
    print!("  Push to origin now? (y/n): ");
    io::stdout().flush()?;
    let mut push_ans = String::new();
    io::stdin().read_line(&mut push_ans)?;

    if push_ans.trim().to_lowercase() == "y" {
        println!("  {} Pushing...", "→".cyan());
        let push = std::process::Command::new("git").arg("push").status()?;
        if push.success() {
            println!("{}", "  🚀 Pushed to origin".green().bold());
        } else {
            println!("{}", "  ❌ Push failed — run 'git push' manually".red());
        }
    } else {
        println!("{}", "  ℹ️  Committed locally — push when ready".dimmed());
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    println!("{}", "  🌲 The forest remembers.".cyan().dimmed());

    Ok(())
}
