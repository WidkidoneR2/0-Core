use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;
use faelight_core::paths;

pub fn check_secrets() -> Result<bool> {
    println!("{}", "🔍 Scanning for secrets with gitleaks...".cyan());
    
    let core_dir = paths::core_dir();
    let gitleaks_config = paths::gitleaks_config();
    
    // Run gitleaks
    let output = Command::new("gitleaks")
        .args([
            "protect",
            "--staged",
            "-c",
            gitleaks_config.to_str().unwrap(),
            "--redact",
            "-v",
        ])
        .current_dir(&core_dir)
        .output()
        .context("Failed to run gitleaks - is it installed?")?;
    
    // Check exit code
    if !output.status.success() {
        println!();
        println!("{}", "❌ GITLEAKS DETECTED SECRETS!".red().bold());
        println!("{}", "⚠️  Commit blocked to protect you!".yellow());
        println!();
        println!("Review the findings above and remove the secrets.");
        println!();
        println!("{}", "To bypass (NOT RECOMMENDED):".yellow());
        println!("  git commit --no-verify");
        println!();
        return Ok(false);
    }
    
    println!("{}", "✅ No secrets detected".green());
    Ok(true)
}
