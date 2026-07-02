use anyhow::Result;
use colored::Colorize;
use std::process::Command;

fn is_rustfmt_installed() -> bool {
    Command::new("rustfmt")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_rustfmt() -> Result<bool> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("git diff --cached --name-only --diff-filter=ACM | grep '\\.rs$'")
        .output()?;

    if output.stdout.is_empty() {
        println!("{}", "✅ Rustfmt: No Rust files staged".green());
        return Ok(true);
    }

    if !is_rustfmt_installed() {
        println!(
            "{}",
            "⚠️  rustfmt not found — formatting check skipped".yellow()
        );
        println!(
            "   Install with: {}",
            "rustup component add rustfmt".dimmed()
        );
        println!();
        return Ok(true);
    }

    let _rust_files = String::from_utf8_lossy(&output.stdout);

    // Use cargo fmt --check to respect workspace edition
    let check = Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .output()?;

    if !check.status.success() {
        println!("{}", "❌ Rustfmt check failed".red().bold());
        println!();
        println!("   Run: {}", "cargo fmt".cyan());
        return Ok(false);
    }

    println!(
        "{}",
        "✅ Rustfmt: All Rust files properly formatted".green()
    );
    Ok(true)
}
