//! Show detailed risk breakdown

use crate::git::GitRepo;
use crate::risk::RiskScore;
use anyhow::Result;
use colored::*;

pub fn run() -> Result<()> {
    let repo = GitRepo::open()?;
    let risk = RiskScore::calculate(&repo)?;

    println!("{}", "⚠️  Git Risk Assessment".cyan().bold());
    println!("{}", "━".repeat(50).dimmed());
    println!();

    println!(
        "{}: {} {}",
        "Total Risk".bold(),
        risk.emoji(),
        format!("{}/100", risk.total).color(risk.color()).bold()
    );

    println!("{}: {:?}", "Band".dimmed(), risk.band());

    println!();
    println!("{}", "Risk Factors:".bold());

    if risk.breakdown.is_empty() {
        println!("  {} No risk factors detected", "✅".green());
    } else {
        for factor in &risk.breakdown {
            println!(
                "  {} {}: {}",
                format!("{:+3}", factor.delta).color(risk.color()),
                factor.name.bold(),
                factor.reason.dimmed()
            );
        }
    }

    println!();
    println!("{}", "━".repeat(50).dimmed());

    // Recommendations
    if risk.total > 50 {
        println!("{}", "⚠️  High Risk - Recommendations:".yellow().bold());
        println!("  • Run {} before pushing", "safe-update".cyan());
        println!("  • Create snapshot: {}", "faelight snapshot".cyan());
        println!("  • Review changes carefully");
    } else if risk.total > 20 {
        println!("{}", "💡 Moderate Risk - Suggestions:".yellow());
        println!("  • Consider creating a snapshot");
        println!("  • Verify changes with: {}", "git diff --cached".cyan());
    } else {
        println!("{}", "✅ Low Risk - Good to proceed".green());
    }

    Ok(())
}
