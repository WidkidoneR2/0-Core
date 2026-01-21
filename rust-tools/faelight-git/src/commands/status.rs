//! Risk-aware git status

use crate::git::GitRepo;
use crate::risk::RiskScore;
use crate::is_core_locked;
use anyhow::Result;
use colored::*;

pub fn run() -> Result<()> {
    let repo = GitRepo::open()?;
    let risk = RiskScore::calculate(&repo)?;
    let status = repo.status()?;
    let branch = repo.current_branch()?;
    let upstream = repo.upstream()?;
    let (ahead, behind) = repo.ahead_behind()?;

    // Header
    println!("{}", "🌲 Git Status — Faelight Forest".cyan().bold());
    println!("{}", "━".repeat(50).dimmed());
    
    // Branch info
    println!("{}: {}", "Branch".dimmed(), branch.green());
    if let Some(up) = upstream {
        println!("{}: {}", "Upstream".dimmed(), up.blue());
    } else {
        println!("{}: {}", "Upstream".dimmed(), "none".yellow());
    }
    
    // Lock status
    let lock_status = if is_core_locked() {
        "🔒 LOCKED".red()
    } else {
        "🔓 UNLOCKED".green()
    };
    println!("{}: {}", "Core Lock".dimmed(), lock_status);
    
    println!();
    
    // Working tree
    println!("{}", "Working Tree:".bold());
    println!("  • Modified files: {}", if status.modified > 0 {
        status.modified.to_string().yellow()
    } else {
        status.modified.to_string().green()
    });
    println!("  • Untracked files: {}", if status.untracked > 0 {
        status.untracked.to_string().yellow()
    } else {
        status.untracked.to_string().green()
    });
    
    println!();
    
    // Commits
    println!("{}", "Commits:".bold());
    println!("  • Ahead: {}", if ahead > 0 {
        ahead.to_string().yellow()
    } else {
        ahead.to_string().green()
    });
    println!("  • Behind: {}", if behind > 0 {
        behind.to_string().yellow()
    } else {
        behind.to_string().green()
    });
    
    println!();
    
    // Risk score
    println!("{}", "━".repeat(50).dimmed());
    print!("{}: {} ", "Risk Score".bold(), risk.emoji());
    println!("{}", format!("{} / 100", risk.total)
        .color(risk.color())
        .bold());
    
    if !risk.breakdown.is_empty() {
        println!();
        for factor in &risk.breakdown {
            let sign = if factor.delta > 0 { "↑" } else { "↓" };
            println!("  {} {} {} ({})", 
                sign, 
                factor.name.dimmed(),
                format!("{:+}", factor.delta).color(risk.color()),
                factor.reason.dimmed()
            );
        }
    }
    
    println!("{}", "━".repeat(50).dimmed());
    
    Ok(())
}
