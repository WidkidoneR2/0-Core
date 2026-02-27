use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::process::Command;

pub fn update(ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    ctx.capabilities.require(
        "update",
        &[
            Capability::FilesystemReadHome,
            Capability::ExecutePacman,
            Capability::SpawnProcess,
        ],
    )?;
    let bin = format!("{}/target/release/faelight-update", ctx.core_root);
    let status = Command::new(&bin).args(args).status()?;

    // Event Ledger
    let writer = crate::runtime::EventWriter::new(&ctx.runtime.db);
    writer.write(
        "update",
        "run",
        "core update",
        if status.success() { "ok" } else { "warn" },
        None,
    );

    Ok(())
}

pub fn safe(ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    let bin = format!("{}/scripts/safe-update", ctx.core_root);
    Command::new(&bin).args(args).status()?;
    Ok(())
}

pub fn simulate(_ctx: &AppContext) -> CoreResult<()> {
    println!("{}", "🔮 core simulate update".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    println!("  {} Checking for available updates...", "→".dimmed());
    println!();

    let output = Command::new("checkupdates").output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let packages: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

            if packages.is_empty() {
                println!(
                    "  {} System is up to date — no packages to update.",
                    "✓".bright_green()
                );
            } else {
                println!(
                    "  {} packages would be updated:",
                    packages.len().to_string().bright_white()
                );
                println!();
                for pkg in &packages {
                    // Format: "pkgname oldver -> newver"
                    let parts: Vec<&str> = pkg.splitn(3, ' ').collect();
                    if parts.len() >= 3 {
                        println!(
                            "  {} {}  {}",
                            "▶".dimmed(),
                            parts[0].bright_white(),
                            parts[2].dimmed(),
                        );
                    } else {
                        println!("  {} {}", "▶".dimmed(), pkg.white());
                    }
                }
            }
        }
        Err(_) => {
            // checkupdates not available — fall back to pacman -Qu
            let fallback = Command::new("pacman").args(["-Qu"]).output();
            match fallback {
                Ok(o) if !o.stdout.is_empty() => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let count = stdout.lines().count();
                    println!(
                        "  {} packages available (run faelight-update to apply)",
                        count.to_string().bright_white()
                    );
                }
                _ => {
                    println!(
                        "  {}",
                        "checkupdates not available — install pacman-contrib".yellow()
                    );
                    println!("  {}", "paci pacman-contrib".dimmed());
                }
            }
        }
    }

    println!();
    println!("{}", "━".repeat(52).dimmed());
    println!("  {} No changes made to system.", "ℹ".cyan());

    Ok(())
}
