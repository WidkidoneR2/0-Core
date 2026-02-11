use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn check_rustfmt() -> Result<bool> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("git diff --cached --name-only --diff-filter=ACM | grep '\\.rs$'")
        .output()?;

    if output.stdout.is_empty() {
        return Ok(true);
    }

    let rust_files = String::from_utf8_lossy(&output.stdout);

    for file in rust_files.lines() {
        let check = Command::new("rustfmt").arg("--check").arg(file).output()?;

        if !check.status.success() {
            println!("{}", "❌ Rustfmt check failed".red().bold());
            println!("   File needs formatting: {}", file.yellow());
            println!();
            println!("   Run: {}", "cargo fmt".cyan());
            return Ok(false);
        }
    }

    println!(
        "{}",
        "✅ Rustfmt: All Rust files properly formatted".green()
    );
    Ok(true)
}
