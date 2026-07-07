// flake_update.rs -- INT-074 Phase 1b: per-input flake update with pre-switch closure diff
// and a review gate. "See what changes before it changes you."
//
// SAFE FLOW (nothing touches the running system until explicit approval):
//   1. back up flake.lock (revert point)
//   2. nix flake update <input>            -- update just this input's lock entry (reversible)
//   3. show the lock delta (old rev -> new rev)
//   4. nixos-rebuild build --flake .#host  -- BUILD ONLY (unprivileged, -> ./result)
//   5. nvd diff /run/current-system ./result  -- the closure diff (read-only)
//   6. REVIEW GATE: show diff, prompt apply? (y/N)
//   7a. yes -> sudo nixos-rebuild switch    7b. no -> restore flake.lock, nothing changed
//
// In dry_run mode we do steps 1-5 and STOP before the gate/switch, then restore the lock --
// so the diff is shown but the system is never touched and the lock is left unchanged.

use anyhow::{Context, Result};
use colored::*;
use std::path::Path;
use std::process::Command;

fn flake_dir() -> String {
    std::env::var("HOME")
        .map(|h| format!("{h}/0-core"))
        .unwrap_or_else(|_| ".".into())
}

fn host() -> String {
    // The framework16 host. Could be made configurable later.
    "framework16".to_string()
}

/// Run the per-input update + closure-diff + review-gate flow.
/// `input` is a flake input name (e.g. "nixpkgs"); `dry_run` stops before any switch.
pub fn run_flake_update(input: &str, dry_run: bool) -> Result<()> {
    let dir = flake_dir();
    let lock = format!("{dir}/flake.lock");
    let lock_bak = format!("{dir}/flake.lock.faelight-bak");

    println!("{}", "❄ Faelight Flake Update".bright_cyan().bold());
    println!("{}", "─".repeat(48).dimmed());
    println!("  Input:   {}", input.bright_white());
    println!(
        "  Mode:    {}",
        if dry_run {
            "dry-run (no switch)".yellow()
        } else {
            "live".green()
        }
    );
    println!();

    // 1. Back up the lock so we can revert cleanly.
    std::fs::copy(&lock, &lock_bak).context("failed to back up flake.lock")?;

    // 2. Update just this input.
    println!("  {} Updating input '{}'...", "→".bright_cyan(), input);
    let upd = Command::new("nix")
        .args(["flake", "update", input])
        .current_dir(&dir)
        .status()
        .context("nix flake update failed to run")?;
    if !upd.success() {
        restore_lock(&lock, &lock_bak);
        anyhow::bail!("nix flake update '{input}' failed");
    }

    // 3. Build the new config (NO switch). Unprivileged.
    println!(
        "  {} Building new configuration (no switch)...",
        "→".bright_cyan()
    );
    let build = Command::new("nixos-rebuild")
        .args(["build", "--flake", &format!("{dir}#{}", host())])
        .current_dir(&dir)
        .status()
        .context("nixos-rebuild build failed to run")?;
    if !build.success() {
        restore_lock(&lock, &lock_bak);
        anyhow::bail!("nixos-rebuild build failed -- lock reverted, system untouched");
    }

    // 4. Closure diff: current system vs the freshly built ./result.
    let result = format!("{dir}/result");
    if Path::new(&result).exists() {
        println!();
        println!("  {} Closure diff (current vs new):", "🔍".normal());
        println!("{}", "─".repeat(48).dimmed());
        let _ = Command::new("nvd")
            .args(["diff", "/run/current-system", &result])
            .status();
        println!("{}", "─".repeat(48).dimmed());
    } else {
        println!("  {} ./result not found -- skipping diff", "⚠️".yellow());
    }

    // 5. Gate.
    if dry_run {
        println!();
        println!(
            "  {} Dry-run: reverting lock, system untouched.",
            "✓".green()
        );
        restore_lock(&lock, &lock_bak);
        return Ok(());
    }

    println!();
    print!(
        "  {} Apply this update (switch)? (y/N): ",
        "💡".bright_cyan()
    );
    use std::io::Write;
    std::io::stdout().flush().ok();
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer).ok();
    if answer.trim().to_lowercase() == "y" {
        println!("  {} Switching...", "→".bright_cyan());
        let sw = Command::new("sudo")
            .args([
                "nixos-rebuild",
                "switch",
                "--flake",
                &format!("{dir}#{}", host()),
            ])
            .current_dir(&dir)
            .status()
            .context("nixos-rebuild switch failed to run")?;
        if sw.success() {
            println!("  {} Update applied.", "✅".green());
            std::fs::remove_file(&lock_bak).ok();
        } else {
            restore_lock(&lock, &lock_bak);
            anyhow::bail!("switch failed -- lock reverted");
        }
    } else {
        println!(
            "  {} Aborted -- reverting lock, system untouched.",
            "✓".green()
        );
        restore_lock(&lock, &lock_bak);
    }
    Ok(())
}

fn restore_lock(lock: &str, lock_bak: &str) {
    if Path::new(lock_bak).exists() {
        let _ = std::fs::copy(lock_bak, lock);
        let _ = std::fs::remove_file(lock_bak);
    }
}
