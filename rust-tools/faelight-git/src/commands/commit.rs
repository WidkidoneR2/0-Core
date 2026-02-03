//! Intent-aware commit verification

use crate::git::GitRepo;
use crate::risk::RiskScore;
use crate::is_locked;
use anyhow::{Result, bail};
use colored::*;
use std::io::{self, Write};

pub fn run(intent: Option<String>, no_intent: bool) -> Result<()> {
    let repo = GitRepo::open()?;
    
    // Check 1: Core must be unlocked
    if is_locked() {
        bail!("Core is locked. Run 'unlock-core' before committing.");
    }
    
    // Check 2: Working tree must have changes
    if repo.is_clean()? {
        println!("{}", "✅ Working tree is clean - nothing to commit".green());
        return Ok(());
    }
    
    // Check 3: Calculate risk
    let risk = RiskScore::calculate(&repo)?;
    let status = repo.status()?;
    
    // Show current state
    println!("{}", "🔍 Pre-commit Analysis".cyan().bold());
    println!("{}", "━".repeat(50).dimmed());
    println!("Modified files: {}", status.modified);
    println!("Untracked files: {}", status.untracked);
    println!("Risk Score: {} {}/100", risk.emoji(), risk.total);
    println!("{}", "━".repeat(50).dimmed());
    println!();
    
    // Check 4: Intent handling
    let intent_ref = if no_intent {
        println!("{}", "⚠️  Proceeding without intent (--no-intent flag)".yellow());
        None
    } else if let Some(ref i) = intent {
        println!("{} Intent: {}", "✅".green(), i);
        Some(i.clone())
    } else {
        // Interactive prompt
        print!("{} ", "Intent reference (or 'skip'):".cyan());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input == "skip" {
            println!("{}", "⚠️  Committing without intent".yellow());
            None
        } else if input.is_empty() {
            bail!("Commit cancelled - no intent provided");
        } else {
            println!("{} Intent: {}", "✅".green(), input);
            Some(input.to_string())
        }
    };
    
    println!();
    println!("{}", "✅ Pre-commit checks passed".green());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Review changes: {}", "git diff --cached".cyan());
    
    if let Some(intent) = intent_ref {
        println!("  2. Commit with intent:");
        println!("     {}", format!("git commit -m \"<message>\n\nIntent: {}\"", intent).cyan());
    } else {
        println!("  2. Commit: {}", "git commit".cyan());
    }
    
    Ok(())
}
