use anyhow::{Context, Result};
use colored::Colorize;
use faelight_core::paths;
use std::process::Command;

fn is_gitleaks_installed() -> bool {
    Command::new("which")
        .arg("gitleaks")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_secrets() -> Result<bool> {
    println!("{}", "🔍 Scanning for secrets...".cyan());

    if !is_gitleaks_installed() {
        println!(
            "{}",
            "⚠️  gitleaks not found — secret scanning skipped".yellow()
        );
        println!(
            "   Install with: {}",
            "pacman -S gitleaks  OR  cargo install gitleaks".dimmed()
        );
        println!();
        return Ok(true); // Warning, not a hard failure
    }

    let core_dir = paths::core_dir();
    let gitleaks_config = paths::gitleaks_config();

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
        .context("Failed to run gitleaks")?;

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
