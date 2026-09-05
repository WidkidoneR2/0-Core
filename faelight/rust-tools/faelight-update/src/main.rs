mod cargo_checker;
mod cleanup_checker;
mod config;
mod firmware_checker;
mod flatpak_checker;
mod git_checker;
mod neovim_checker;
mod npm_checker;
mod pip_checker;
mod rustup_checker;
mod system_checker;
mod tui_v2;
mod yazi_checker;

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use std::process::Command;

/// CLI Arguments
#[derive(Parser)]
#[command(
    name = "faelight-update",
    about = "🌲 Intelligent update manager for Faelight Forest",
    version  // Automatically uses CARGO_PKG_VERSION from Cargo.toml
)]
struct Cli {
    /// Check for updates without applying them
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Skip health check before updates
    #[arg(long)]
    skip_health: bool,

    /// Interactive mode to select packages
    #[arg(short, long)]
    interactive: bool,

    /// Create pre-update snapshot (requires faelight-snapshot)
    #[arg(long)]
    snapshot: bool,

    /// Skip interactive TUI and update immediately
    #[arg(short = 'y', long)]
    yes: bool,

    /// Show detailed version information for each update
    #[arg(short, long)]
    verbose: bool,
    /// Preview what would change without updating
    #[arg(long)]
    preview: bool,

    /// Output results in JSON format
    #[arg(long)]
    json: bool,
    /// Only check specific categories (comma-separated: flake,cargo,neovim,workspace)

    /// Run maintenance tasks (clean cache, orphans, journal)
    #[arg(long)]
    maintain: bool,
    /// Output only the total count of updates (for scripts/bar)
    #[arg(long)]
    count_only: bool,
    #[arg(long, value_delimiter = ',')]
    only: Option<Vec<String>>,

    /// Skip specific categories (comma-separated)
    #[arg(long, value_delimiter = ',')]
    skip: Option<Vec<String>>,
}

/// Generate post-run suggestions
fn print_suggestions(categories: &[UpdateCategory], total: usize) {
    let mut suggestions: Vec<String> = Vec::new();
    // Check for kernel update — reboot needed
    for cat in categories {
        for item in &cat.items {
            if item.name.starts_with("linux") && !item.name.contains("headers") {
                suggestions.push("Reboot recommended — kernel was updated".to_string());
            }
        }
    }
    // If nothing was updated
    // INT-192: a total containing an unknown is itself unknown.
    if total == 0 && !categories.iter().any(|c| c.skipped.is_some()) {
        suggestions.push("System is fully up to date — nothing to do".to_string());
    }
    if !suggestions.is_empty() {
        println!();
        println!("  {} Suggestions", "💡".normal());
        println!("  {}", "─".repeat(46).dimmed());
        for s in &suggestions {
            println!("  {} {}", "•".bright_cyan(), s.bright_white());
        }
        println!();
    }
}
/// Run pre-flight checks before updating
fn run_preflight_checks() {
    let mut warnings: Vec<String> = Vec::new();
    // Check 1: disk space
    if let Ok(output) = std::process::Command::new("df").args(["-h", "/"]).output() {
        let text = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = text.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let pct_str = parts[4].trim_end_matches('%');
                if let Ok(pct) = pct_str.parse::<u32>() {
                    if pct >= 90 {
                        warnings.push(format!(
                            "Disk space critical: {}% used — clean before updating",
                            pct
                        ));
                    } else if pct >= 80 {
                        warnings.push(format!(
                            "Disk space: {}% used — consider cleaning first",
                            pct
                        ));
                    }
                }
            }
        }
    }
    if !warnings.is_empty() {
        println!("  {} Pre-Update Warnings", "⚠️ ".yellow().bold());
        println!("  {}", "─".repeat(46).dimmed());
        for w in &warnings {
            println!("  {} {}", "•".bright_yellow(), w.bright_white());
        }
        println!("  {}", "─".repeat(46).dimmed());
        println!();
    }
}
/// Get system drift score based on last upgrade time
fn get_drift_score() -> (String, String) {
    // NixOS drift (INT-074 de-Arch): days since the flake.lock was last updated -- the
    // NixOS analog of "days since last system upgrade". Older lock = more drift.
    let lock_path = std::env::var("HOME")
        .map(|h| format!("{h}/0-core/flake.lock"))
        .unwrap_or_else(|_| "flake.lock".to_string());
    let days_ago = std::fs::metadata(&lock_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| (d.as_secs() / 86_400) as i64)
        .unwrap_or(-1);
    let label = if days_ago < 0 {
        ("unknown".to_string(), "?".to_string())
    } else if days_ago == 0 {
        ("today".to_string(), "FRESH".to_string())
    } else if days_ago == 1 {
        ("1 day ago".to_string(), "FRESH".to_string())
    } else if days_ago <= 3 {
        (format!("{} days ago", days_ago), "LOW".to_string())
    } else if days_ago <= 7 {
        (format!("{} days ago", days_ago), "MEDIUM".to_string())
    } else if days_ago <= 14 {
        (format!("{} days ago", days_ago), "HIGH".to_string())
    } else {
        (format!("{} days ago", days_ago), "CRITICAL".to_string())
    };
    label
}

/// Log update run to state.db
fn log_update_run(total: usize, duration_ms: u128, outcome: &str, health_after: i64, drift: &str) {
    let db_path = faelight_core::paths::state_db();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS update_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                total_updates INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                outcome TEXT NOT NULL,
                health_after INTEGER,
                drift_label TEXT
            );",
        );
        let now = chrono::Utc::now().timestamp();
        let _ = conn.execute(
            "INSERT INTO update_history (timestamp, total_updates, duration_ms, outcome, health_after, drift_label)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![now, total as i64, duration_ms as i64, outcome, health_after, drift],
        );
        let payload = format!(
            "{{\"total_updates\":{},\"outcome\":\"{}\",\"health_after\":{},\"drift\":\"{}\"}}",
            total, outcome, health_after, drift
        );
        let weight = if outcome == "success" {
            health_after as f64 / 100.0
        } else {
            0.3
        };
        let _ = conn.execute(
            "INSERT INTO engine_signals (source, signal_type, payload, weight, created_at) VALUES ('faelight-update', 'update', ?1, ?2, ?3)",
            rusqlite::params![payload, weight, now],
        );
    }
}
/// Run system maintenance tasks
fn run_maintenance() -> Result<()> {
    println!("{}", "🧹 Faelight Maintenance Mode".green().bold());
    println!("{}", "─".repeat(48).dimmed());
    println!();

    // 1. Clean cargo cache (cross-platform).
    println!("  {} Cleaning cargo cache...", "→".bright_cyan());
    let cargo_clean = std::process::Command::new("cargo")
        .args(["cache", "--autoclean"])
        .status();
    match cargo_clean {
        Ok(s) if s.success() => println!("  {} Cargo cache cleaned", "✅".green()),
        _ => println!(
            "  {} Cargo cache: run cargo cache --autoclean manually",
            "⚠️".yellow()
        ),
    }

    // 2. Vacuum systemd journal (keep 2 weeks). systemd is cross-platform.
    println!(
        "  {} Vacuuming systemd journal (keep 2 weeks)...",
        "→".bright_cyan()
    );
    let journal = std::process::Command::new("sudo")
        .args(["journalctl", "--vacuum-time=2weeks"])
        .status();
    match journal {
        Ok(s) if s.success() => println!("  {} Journal vacuumed", "✅".green()),
        _ => println!("  {} Journal vacuum failed (sudo required)", "⚠️".yellow()),
    }

    // 3. Nix store cleanup -- delegate to the forest's dedicated tool rather than duplicate it.
    //    (INT-074 de-Arch: replaces the old `sudo pacman -Sc` + orphan-removal steps.)
    println!("  {} Nix store cleanup", "→".bright_cyan());
    println!(
        "    {} Run `nhclean` (nh clean all --keep-since 7d --ask) to reclaim store space",
        "💡".bright_cyan()
    );

    println!();
    println!("{}", "─".repeat(48).dimmed());
    println!("  {} Maintenance complete", "✅".green().bold());
    println!();
    Ok(())
}
/// Print system identity header
fn print_system_identity() {
    use std::process::Command;
    let hostname = std::fs::read_to_string("/etc/hostname")
        .unwrap_or_default()
        .trim()
        .to_string();
    let kernel = Command::new("uname")
        .arg("-r")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let uptime = Command::new("uptime")
        .arg("-p")
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .trim_start_matches("up ")
                .to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string());
    let wm = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("WAYLAND_DISPLAY").map(|_| "Wayland".to_string()))
        .unwrap_or_else(|_| "unknown".to_string());
    // $SHELL IS THE LOGIN SHELL, NOT THE RUNNING ONE, and here they are deliberately
    // different: bash holds the passwd seat per INT-190 and starts nsh from .bashrc.
    // So this said bash -- right for the question asked, wrong for the one a reader has.
    // INT-129 recorded it 2026-07-07 when the answer was fsh. Still bash 2026-09-04.
    let shell = match std::env::var("NSH_VERSION") {
        Ok(v) if !v.is_empty() => format!("nsh {}", v),
        _ => std::env::var("SHELL")
            .map(|s| s.split('/').next_back().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
    };
    // Get health from cache
    let health = std::fs::read_to_string(
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".cache/faelight/health-status"),
    )
    .unwrap_or_else(|_| "?".to_string())
    .trim()
    .to_string();
    println!("{}", "🧬 System Profile".cyan().bold());
    println!("{}", "─".repeat(40).dimmed());
    println!("  {:<12} {}", "Host:".dimmed(), hostname.bright_white());
    println!("  {:<12} {}", "Kernel:".dimmed(), kernel.bright_white());
    println!("  {:<12} {}", "Shell:".dimmed(), shell.bright_white());
    println!("  {:<12} {}", "WM:".dimmed(), wm.bright_white());
    println!("  {:<12} {}", "Uptime:".dimmed(), uptime.bright_white());
    println!(
        "  {:<12} {}%",
        "Health:".dimmed(),
        if health == "100" {
            health.bright_green()
        } else if health.parse::<u32>().unwrap_or(0) >= 80 {
            health.bright_yellow()
        } else {
            health.bright_red()
        }
    );
    // Drift score
    let (last_upgrade, drift_label) = get_drift_score();
    let drift_colored = match drift_label.as_str() {
        "FRESH" | "LOW" => drift_label.bright_green(),
        "MEDIUM" => drift_label.bright_yellow(),
        "HIGH" | "CRITICAL" => drift_label.bright_red(),
        _ => drift_label.normal(),
    };
    println!(
        "  {:<12} {} ({})",
        "Last update:".dimmed(),
        last_upgrade.bright_white(),
        drift_colored
    );
    println!("  {:<12} {}", "Drift:".dimmed(), drift_colored);
    // INT-207 L1 — Show active intents
    let _core_root = std::env::var("HOME").unwrap_or_default() + "/0-core";
    let intents_dir = faelight_core::paths::intents_dir().join("future");
    let active_intents: Vec<String> = std::fs::read_dir(&intents_dir)
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter(|e| {
                    if let Ok(c) = std::fs::read_to_string(e.path()) {
                        c.contains("status: in-progress") || c.contains("type: in-progress")
                    } else {
                        false
                    }
                })
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let num = name.split('-').next().unwrap_or("").to_string();
                    if !num.is_empty() {
                        Some(format!("INT-{}", num))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    if !active_intents.is_empty() {
        println!(
            "  {:<12} {}",
            "Active:".dimmed(),
            active_intents.join(", ").bright_cyan()
        );
    }
    // INT-207 L1 — Show alignment score
    let state_db = faelight_core::paths::state_db();
    if let Ok(conn) = rusqlite::Connection::open(&state_db) {
        let align: Option<f64> = conn.query_row(
            "SELECT AVG(score) FROM alignment_checks WHERE checked_at > (strftime('%s','now') - 604800)",
            [], |r| r.get(0)
        ).ok().flatten();
        if let Some(score) = align {
            let pct = (score * 100.0) as i64;
            let colored = if pct >= 80 {
                format!("{}%", pct).bright_green()
            } else if pct >= 60 {
                format!("{}%", pct).bright_yellow()
            } else {
                format!("{}%", pct).bright_red()
            };
            println!("  {:<12} {}", "Alignment:".dimmed(), colored);
        }
    }
    // Warn if critical update during active development
    if !active_intents.is_empty() {
        println!("{}", "─".repeat(40).dimmed());
        println!(
            "  {} {} intent(s) in progress — update may interrupt development",
            "💡".normal(),
            active_intents.len().to_string().bright_yellow()
        );
    } else {
        println!("{}", "─".repeat(40).dimmed());
    }
    println!();
}
fn main() {
    faelight_core::restore_sigpipe();
    if let Err(e) = run() {
        eprintln!("{} {}", "❌".red(), format!("Error: {:#}", e).red());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Print banner with version from Cargo.toml
    if !cli.json && !cli.count_only {
        println!(
            "{} v{}",
            "🌲 Faelight Update Manager".green().bold(),
            env!("CARGO_PKG_VERSION").cyan()
        );
        println!();
    }

    // Preview mode — show exactly what would change
    if cli.preview {
        if !cli.json && !cli.count_only {
            print_system_identity();
        }
        println!("{}  Checking for updates...", "🔍".cyan());
        let check_start = std::time::Instant::now();
        let updates = check_all_updates()?;
        let total_check_ms = check_start.elapsed().as_millis();
        let total: usize = updates.iter().map(|c| c.count).sum();
        let unknown = updates.iter().any(|c| c.skipped.is_some());
        println!();
        println!("{}", "📦 Preview — What Would Change".cyan().bold());
        println!("{}", "─".repeat(48).dimmed());
        if total == 0 && !unknown {
            println!("  {} Nothing to update — system is current", "✅".green());
        } else {
            for cat in &updates {
                if cat.count > 0 {
                    println!(
                        "  {} {} ({} available)",
                        cat.emoji.yellow(),
                        cat.name.bold(),
                        cat.count.to_string().bright_yellow()
                    );
                    for item in &cat.items {
                        println!(
                            "    {} {} {} → {}",
                            "+".bright_green(),
                            item.name.bright_white(),
                            item.current.dimmed(),
                            item.new.bright_green()
                        );
                    }
                    println!();
                }
            }
        }
        println!("{}", "─".repeat(48).dimmed());
        println!(
            "  {} {} updates available  ({:.1}s)",
            "→".dimmed(),
            total.to_string().bright_yellow(),
            total_check_ms as f64 / 1000.0
        );
        println!("  {} Run without --preview to apply", "💡".bright_cyan());
        println!();
        return Ok(());
    }
    // Maintenance mode
    if cli.maintain {
        return run_maintenance();
    }
    // System Identity header
    if !cli.json && !cli.count_only {
        print_system_identity();
    }
    // Health check
    if !cli.skip_health && !cli.json && !cli.count_only {
        println!("{}  Running health check...", "🏥".green());
        run_health_check()?;
    }

    // Create pre-update snapshot if requested
    if cli.snapshot && !cli.dry_run && !cli.json {
        create_snapshot()?;
    }

    // Pre-flight warnings
    if !cli.json && !cli.count_only && !cli.skip_health {
        run_preflight_checks();
    }
    // Check for updates
    if !cli.json && !cli.count_only {
        println!("{}  Checking for updates...", "🔍".cyan());
    }
    let check_start = std::time::Instant::now();
    let mut updates = check_all_updates()?;
    let total_check_ms = check_start.elapsed().as_millis();

    // Filter categories based on --only and --skip
    if let Some(ref only) = cli.only {
        updates.retain(|cat| only.iter().any(|o| category_matches(o, &cat.name)));
    }

    if let Some(ref skip) = cli.skip {
        updates.retain(|cat| !skip.iter().any(|s| category_matches(s, &cat.name)));
    }

    let total: usize = updates.iter().map(|c| c.count).sum();
    // INT-192: a category that could not be checked makes the total unknown.
    let unknown = updates.iter().any(|c| c.skipped.is_some());

    // Count-only output
    if cli.count_only {
        // INT-192: a bar reading this must not see 0 for could-not-check.
        if unknown {
            println!("?");
        } else {
            println!("{}", total);
        }
        return Ok(());
    }
    // JSON output
    if cli.json {
        output_json(&updates, total)?;
        return Ok(());
    }

    // Show summary
    show_update_summary(&updates, cli.verbose);

    // Print suggestions
    print_suggestions(&updates, total);
    if total == 0 && !unknown {
        println!("\n{}  All packages up to date!", "✨".green());
        return Ok(());
    }

    // Performance breakdown
    if !cli.json && !cli.count_only {
        println!(
            "  {} Check completed in {:.1}s",
            "⏱️ ".normal(),
            total_check_ms as f64 / 1000.0
        );
        println!();
    }
    // Show impact analysis
    let impact = analyze_impact(&updates);
    if impact.has_impact() {
        println!("{}", impact);
    }

    // Interactive TUI mode (default unless --yes or --dry-run)
    if !cli.dry_run && !cli.yes && updates.iter().any(|c| c.count > 0) {
        let selections = match tui_v2::run_interactive_tui(&updates) {
            Ok(sel) => sel,
            Err(e) => {
                eprintln!("{}  TUI error: {}", "❌".red(), e);
                return Ok(());
            }
        };

        if selections.is_empty() {
            println!("\n{}  No packages selected", "ℹ️".blue());
            return Ok(());
        }

        // LOCK CORE
        let in_core = is_in_core();

        // PERFORM UPDATES
        perform_updates(&selections)?;

        // CLEANUP CACHES
        cleanup_caches()?;

        // UPDATE PROMPT CACHE
        update_prompt_cache()?;

        // UNLOCK CORE
        if in_core {
            // unlock handled automatically
        }

        // FINAL HEALTH CHECK
        let health = run_doctor_final()?;

        // GIT STATUS CHECK
        check_git_status()?;

        // (Arch .pacnew + AUR-rebuild checks removed -- INT-074 de-Arch.)

        // FINAL SUMMARY
        println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        if health == 100 {
            println!("{}  Update complete! System: {}%", "✅".green(), health);
        } else {
            println!(
                "{}  Update complete! System: {}% (check warnings)",
                "⚠️".yellow(),
                health
            );
        }
        // Log to state.db
        let (_, drift_after) = get_drift_score();
        log_update_run(selections.len(), 0, "success", health as i64, &drift_after);
        return Ok(());
    } else if cli.dry_run {
        // THIS BRANCH APPLIED THE UPDATES. --dry-run is documented as check for updates
        // WITHOUT applying them. It printed Ready to update N packages, prompted Proceed?
        // (Y/n), treated an EMPTY INPUT as yes, and called perform_updates -- so pressing
        // Enter, the obvious thing to do at a prompt, installed everything.
        //
        // Found 2026-09-04 closing INT-129. A dry run reports and stops. The interactive
        // path at ~554 is where perform_updates belongs and it still lives there.
        println!();
        println!(
            "  {} {} update(s) found -- nothing applied (--dry-run)",
            "✓".green(),
            total
        );
        if total > 0 {
            println!("     run without --dry-run to apply");
        }
        return Ok(());
    } else {
        println!(
            "\n{}  Dry run complete - {} updates available",
            "ℹ️".blue(),
            total
        );
        println!("{}  Run with --interactive to select packages", "💡".blue());
    }

    Ok(())
}

/// Check if a filter matches a category name (case-insensitive partial match)
fn category_matches(filter: &str, category: &str) -> bool {
    let filter_lower = filter.to_lowercase();
    let category_lower = category.to_lowercase();

    // Exact match or contains
    category_lower.contains(&filter_lower) ||
    // Common aliases
    (filter_lower == "flake" && category_lower.contains("flake")) ||
    (filter_lower == "cargo" && category_lower.contains("cargo")) ||
    (filter_lower == "neovim" && category_lower.contains("neovim")) ||
    (filter_lower == "workspace" && category_lower.contains("workspace"))
}

/// Create pre-update snapshot
fn create_snapshot() -> Result<()> {
    println!("{}  Creating pre-update snapshot...", "📸".yellow());

    let output = Command::new("faelight-snapshot")
        .args(["create", "--tag", "pre-update"])
        .output()
        .context("Failed to create snapshot - is faelight-snapshot installed?")?;

    if output.status.success() {
        println!("   {}  Snapshot created", "✅".green());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Snapshot failed: {}\n💡 Install faelight-snapshot or run without --snapshot flag",
            stderr
        );
    }

    Ok(())
}

/// Run health check
fn run_health_check() -> Result<()> {
    let output = Command::new("core")
        .args(["doctor", "run"])
        .output()
        .context("Failed to run core doctor run")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse health percentage from "Health:   90%" line
    let health_pct = stdout
        .lines()
        .find(|l| l.trim().starts_with("Health:"))
        .and_then(|l| l.split_whitespace().last())
        .and_then(|s| s.trim_end_matches('%').parse::<u32>().ok())
        .unwrap_or(100);

    // Parse failed count from "Failed:   0" line
    let failed = stdout
        .lines()
        .find(|l| l.trim().starts_with("Failed:"))
        .and_then(|l| l.split_whitespace().last())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    if failed > 0 {
        // Hard failures — warn but do not block (manual control over automation)
        println!(
            "   {}  Health {}% — {} checks failed (use --skip-health to suppress)",
            "⚠️".yellow(),
            health_pct,
            failed
        );
    } else {
        println!("   {}  System healthy ({}%)", "✅".green(), health_pct);
    }

    Ok(())
}

/// Update category structure
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateCategory {
    pub name: String,
    pub count: usize,
    pub items: Vec<UpdateItem>,
    /// INT-192: Some(reason) when the checker could not run. A skipped category is
    /// UNKNOWN, not zero updates, and every reader of `count` must treat it so.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped: Option<String>,
    #[serde(skip)]
    pub emoji: String,
}

/// Individual update item
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateItem {
    pub name: String,
    pub current: String,
    pub new: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

/// Check all update sources
fn check_all_updates() -> Result<Vec<UpdateCategory>> {
    let mut categories = Vec::new();

    // ❄ FLAKE INPUTS REMOVED 2026-09-04. This category, the --flake-update flag and both
    // flake modules (324 lines) asked nix what the system depends on. There is no flake
    // and no nix: NixOS was wiped for Omarchy on 2026-08-28. INT-129 measured them as
    // genuinely dead rather than repointable -- and generation.rs turned out the same way
    // once measured. Omarchy has snapper, but limine-snapper-sync already puts snapshots in
    // the BOOT MENU, which works when the system will not boot and a TUI cannot. The
    // timeline is sudo snapper list, already a formatted table. Closure diff has no snapper
    // equivalent at all. 476 lines to wrap a command you can run directly.

    // (Arch pacman/AUR checkers were removed by INT-074 for NixOS. The machine is back
    // on Arch, and the System category below restores the capability by a safer route:
    // report what is pending, never apply it.)

    // Cargo tools
    // INT-192: a checker that could not run is UNKNOWN, not zero updates. Same
    // shape as the System category below.
    let (cargo_items, cargo_note) = match cargo_checker::check_cargo_updates() {
        Ok(v) => (v, None),
        Err(s) => (Vec::new(), Some(s.to_string())),
    };
    if let Some(note) = &cargo_note {
        eprintln!("  [??] cargo tools: {}", note);
    }
    categories.push(UpdateCategory {
        name: "Cargo Tools".to_string(),
        emoji: "🦀".to_string(),
        count: cargo_items.len(),
        items: cargo_items,
        skipped: cargo_note,
    });

    // Neovim plugins
    let (nvim_lines, nvim_note) = match neovim_checker::check_neovim_updates() {
        Ok(v) => (v, None),
        Err(s) => (Vec::new(), Some(s.to_string())),
    };
    if let Some(note) = &nvim_note {
        eprintln!("  [??] neovim plugins: {}", note);
    }
    let nvim_items: Vec<UpdateItem> = nvim_lines
        .into_iter()
        // checkupdates prints: name old_version -> new_version. Parsing it fills the
        // columns the TUI already renders; a bare name would leave them blank.
        .map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next().unwrap_or_default().to_string();
            let current = parts.next().unwrap_or_default().to_string();
            let _arrow = parts.next();
            let new = parts.next().unwrap_or_default().to_string();
            UpdateItem {
                name,
                current,
                new,
                // checkupdates does not report the repository, and inventing an empty
                // string would claim it did.
                repository: None,
            }
        })
        .collect();
    categories.push(UpdateCategory {
        name: "Neovim Plugins".to_string(),
        emoji: "📝".to_string(),
        count: nvim_items.len(),
        items: nvim_items,
        skipped: nvim_note,
    });

    // 0-Core workspace
    // INT-192: same shape as the other Checked categories.
    let (workspace_items, workspace_note) = match cargo_checker::check_workspace_updates() {
        Ok(v) => (v, None),
        Err(s) => (Vec::new(), Some(s.to_string())),
    };
    if let Some(note) = &workspace_note {
        eprintln!("  [??] 0-core workspace: {}", note);
    }
    categories.push(UpdateCategory {
        name: "0-Core Workspace".to_string(),
        emoji: "🌲".to_string(),
        count: workspace_items.len(),
        items: workspace_items,
        skipped: workspace_note,
    });

    // Yazi/FM packages (will rename to FM later)

    // Rustup toolchain
    let rustup_items: Vec<UpdateItem> = rustup_checker::check_rustup_updates()
        .into_iter()
        .map(|name| UpdateItem {
            name,
            current: "unknown".to_string(),
            new: "available".to_string(),
            repository: None,
        })
        .collect();
    categories.push(UpdateCategory {
        name: "Rust Toolchain".to_string(),
        emoji: "🦀".to_string(),
        count: rustup_items.len(),
        items: rustup_items,
        skipped: None,
    });

    // NPM global packages
    let npm_items: Vec<UpdateItem> = npm_checker::check_npm_updates()
        .into_iter()
        .map(|name| UpdateItem {
            name,
            current: "unknown".to_string(),
            new: "available".to_string(),
            repository: None,
        })
        .collect();
    categories.push(UpdateCategory {
        name: "NPM Packages".to_string(),
        emoji: "📦".to_string(),
        count: npm_items.len(),
        items: npm_items,
        skipped: None,
    });

    // Pip/pipx packages
    let pip_items: Vec<UpdateItem> = pip_checker::check_pip_updates()
        .into_iter()
        .map(|name| UpdateItem {
            name,
            current: "unknown".to_string(),
            new: "available".to_string(),
            repository: None,
        })
        .collect();
    categories.push(UpdateCategory {
        name: "Python Packages".to_string(),
        emoji: "🐍".to_string(),
        count: pip_items.len(),
        items: pip_items,
        skipped: None,
    });

    let yazi_items: Vec<UpdateItem> = yazi_checker::check_yazi_packages()
        .into_iter()
        .map(|name| UpdateItem {
            name,
            current: "unknown".to_string(),
            new: "available".to_string(),
            repository: None,
        })
        .collect();
    categories.push(UpdateCategory {
        name: "Yazi Packages".to_string(), // TODO: Change to "FM Packages" when ready
        emoji: "📁".to_string(),
        count: yazi_items.len(),
        items: yazi_items,
        skipped: None,
    });

    // Git repositories
    // INT-192: same shape as the other Checked categories.
    let (git_lines, git_note) = match git_checker::check_git_updates() {
        Ok(v) => (v, None),
        Err(s) => (Vec::new(), Some(s.to_string())),
    };
    if let Some(note) = &git_note {
        eprintln!("  [??] git repositories: {}", note);
    }
    let git_items: Vec<UpdateItem> = git_lines
        .into_iter()
        .map(|name| UpdateItem {
            name,
            current: "local".to_string(),
            new: "needs pull".to_string(),
            repository: None,
        })
        .collect();
    categories.push(UpdateCategory {
        name: "Git Repositories".to_string(),
        emoji: "🔄".to_string(),
        count: git_items.len(),
        items: git_items,
        skipped: git_note,
    });

    // Firmware updates
    let firmware_items: Vec<UpdateItem> = firmware_checker::check_firmware_updates()
        .into_iter()
        .map(|name| UpdateItem {
            name,
            current: "unknown".to_string(),
            new: "available".to_string(),
            repository: None,
        })
        .collect();
    categories.push(UpdateCategory {
        name: "Firmware".to_string(),
        emoji: "⚡".to_string(),
        count: firmware_items.len(),
        items: firmware_items,
        skipped: None,
    });

    // Flatpak packages
    let flatpak_items: Vec<UpdateItem> = flatpak_checker::check_flatpak_updates()
        .into_iter()
        .map(|name| UpdateItem {
            name,
            current: "unknown".to_string(),
            new: "available".to_string(),
            repository: None,
        })
        .collect();
    categories.push(UpdateCategory {
        name: "Flatpak".to_string(),
        emoji: "📦".to_string(),
        count: flatpak_items.len(),
        items: flatpak_items,
        skipped: None,
    });

    // System packages -- REPORTED, NEVER APPLIED. INT-129: the distribution owns these and
    // reimplementing that is how you end up fighting it. checkupdates syncs to a temporary
    // database and touches nothing; omarchy-update is what applies them, and it cannot be
    // scripted anyway -- it opens with a box saying you cannot stop the update once you
    // start, and waits for a keypress.
    //
    // INT-192: an absent checkupdates is UNKNOWN, not zero pending, and the item says so
    // rather than leaving an empty category that reads as up to date.
    let (sys_items, sys_note) = match system_checker::check_system_updates() {
        Ok(v) => (
            v.into_iter()
                // checkupdates prints: name old_version -> new_version. Parsing it fills
                // the columns the TUI already renders.
                .map(|line| {
                    let mut parts = line.split_whitespace();
                    let name = parts.next().unwrap_or_default().to_string();
                    let current = parts.next().unwrap_or_default().to_string();
                    let _arrow = parts.next();
                    let new = parts.next().unwrap_or_default().to_string();
                    UpdateItem {
                        name,
                        current,
                        new,
                        // checkupdates does not report the repository, and an empty string
                        // would claim it did.
                        repository: None,
                    }
                })
                .collect::<Vec<_>>(),
            None,
        ),
        Err(s) => (Vec::new(), Some(s.to_string())),
    };
    if let Some(note) = &sys_note {
        eprintln!("  [??] system packages: {}", note);
    }
    categories.push(UpdateCategory {
        name: "System (run omarchy-update)".to_string(),
        emoji: "🧱".to_string(),
        count: sys_items.len(),
        items: sys_items,
        skipped: sys_note,
    });
    Ok(categories)
}

/// Show update summary
/// Classify update risk level by package name
fn classify_risk(name: &str) -> &'static str {
    const CRITICAL: &[&str] = &[
        "linux",
        "linux-lts",
        "linux-zen",
        "linux-hardened",
        "systemd",
        "systemd-libs",
        "glibc",
        "gcc",
        "gcc-libs",
        "binutils",
        "mesa",
        "vulkan-radeon",
        "vulkan-intel",
        "openssl",
        "nss",
        "filesystem",
        "linux-firmware",
        "grub",
        "efibootmgr",
        "wayland",
    ];
    const IMPORTANT: &[&str] = &[
        "git", "neovim", "rust", "rustup", "cargo", "python", "nodejs", "npm", "openssh", "curl",
        "wget", "bash", "sway", "ripgrep", "fd", "bat", "eza", "skim", "zoxide",
    ];
    let n = name.to_lowercase();
    if CRITICAL
        .iter()
        .any(|c| n == *c || n.starts_with(&format!("{}-", c)))
    {
        "critical"
    } else if IMPORTANT.iter().any(|i| n.contains(i)) {
        "important"
    } else {
        "optional"
    }
}
fn show_update_summary(categories: &[UpdateCategory], verbose: bool) {
    println!();
    println!("{}", "📊 Update Summary".cyan().bold());
    println!("{}", "─".repeat(50).cyan());

    let total: usize = categories.iter().map(|c| c.count).sum();
    // Risk summary block (INT-204)
    if total > 0 {
        let mut critical = 0usize;
        let mut important = 0usize;
        let mut optional = 0usize;
        let mut critical_names: Vec<String> = Vec::new();
        let mut important_names: Vec<String> = Vec::new();
        for category in categories {
            for item in &category.items {
                match classify_risk(&item.name) {
                    "critical" => {
                        critical += 1;
                        if critical_names.len() < 3 {
                            critical_names.push(item.name.clone());
                        }
                    }
                    "important" => {
                        important += 1;
                        if important_names.len() < 3 {
                            important_names.push(item.name.clone());
                        }
                    }
                    _ => {
                        optional += 1;
                    }
                }
            }
        }
        if critical > 0 {
            println!(
                "  🔴 {:<10} {} ({})",
                "Critical:".bright_red().bold(),
                critical.to_string().bright_red(),
                critical_names.join(", ").dimmed()
            );
        }
        if important > 0 {
            println!(
                "  🟡 {:<10} {} ({})",
                "Important:".bright_yellow().bold(),
                important.to_string().bright_yellow(),
                important_names.join(", ").dimmed()
            );
        }
        if optional > 0 {
            println!(
                "  🔵 {:<10} {}",
                "Optional:".bright_blue().bold(),
                optional.to_string().bright_blue()
            );
        }
        println!("{}", "─".repeat(50).dimmed());
        println!();
    }
    for category in categories {
        if category.count > 0 {
            println!(
                "  {} {} ({})",
                category.emoji.yellow(),
                category.name.bold(),
                format!("{} available", category.count).yellow()
            );

            let display_count = if verbose { category.items.len() } else { 5 };

            for item in category.items.iter().take(display_count) {
                if verbose {
                    println!(
                        "     {} {} {} → {}",
                        "•".cyan(),
                        item.name.white(),
                        item.current.red(),
                        item.new.green()
                    );
                } else {
                    println!("     {} {}", "•".cyan(), item.name.white());
                }
            }

            if category.items.len() > display_count {
                println!(
                    "     {} {} more...",
                    "...".dimmed(),
                    category.items.len() - display_count
                );
            }
        } else if let Some(reason) = &category.skipped {
            // INT-192: could not check is not up to date.
            println!(
                "  {} {} {}",
                category.emoji.yellow(),
                category.name,
                format!("({})", reason).yellow()
            );
        } else {
            println!(
                "  {} {} {}",
                category.emoji.green(),
                category.name,
                "(up to date)".dimmed()
            );
        }
    }

    println!("{}", "─".repeat(50).cyan());
    println!("  {} updates available", total.to_string().yellow().bold());
}

/// Update impact analysis
#[derive(Default)]
struct UpdateImpact {
    requires_reboot: bool,
    kernel_update: bool,
    critical_count: usize,
    major_updates: Vec<String>,
}

impl UpdateImpact {
    fn has_impact(&self) -> bool {
        self.requires_reboot || self.critical_count > 0 || !self.major_updates.is_empty()
    }
}

impl std::fmt::Display for UpdateImpact {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "\n{}", "📊 Impact Analysis".yellow().bold())?;
        writeln!(f, "{}", "─".repeat(50).yellow())?;

        if self.kernel_update {
            writeln!(
                f,
                "  {} Kernel update - reboot required after update",
                "⚠️".yellow()
            )?;
        }

        if self.critical_count > 0 {
            writeln!(
                f,
                "  {} {} critical system packages",
                "🔴".red(),
                self.critical_count
            )?;
        }

        if !self.major_updates.is_empty() {
            writeln!(f, "  {} Major version updates:", "📈".blue())?;
            for pkg in &self.major_updates {
                writeln!(f, "     • {}", pkg)?;
            }
        }

        Ok(())
    }
}

/// Analyze update impact
fn analyze_impact(categories: &[UpdateCategory]) -> UpdateImpact {
    let mut impact = UpdateImpact::default();

    // Critical packages that should be noted
    const CRITICAL: &[&str] = &[
        "systemd",
        "glibc",
        "gcc",
        "binutils",
        "filesystem",
        "linux-firmware",
        "mesa",
    ];

    for category in categories {
        for item in &category.items {
            // Check for kernel updates
            if item.name.starts_with("linux") && !item.name.contains("headers") {
                impact.kernel_update = true;
                impact.requires_reboot = true;
            }

            // Check for critical packages
            if CRITICAL.contains(&item.name.as_str()) {
                impact.critical_count += 1;
            }

            // Check for major version bumps
            if is_major_version_bump(&item.current, &item.new) {
                impact.major_updates.push(item.name.clone());
            }
        }
    }

    impact
}

/// Check if version bump is major (x.y.z -> (x+1).y.z)
fn is_major_version_bump(current: &str, new: &str) -> bool {
    let current_parts: Vec<&str> = current.split('.').collect();
    let new_parts: Vec<&str> = new.split('.').collect();

    if current_parts.is_empty() || new_parts.is_empty() {
        return false;
    }

    // Extract major version numbers
    let current_major: Option<u32> = current_parts[0].parse().ok();
    let new_major: Option<u32> = new_parts[0].parse().ok();

    match (current_major, new_major) {
        (Some(c), Some(n)) => n > c,
        _ => false,
    }
}

/// Perform selected updates
fn perform_updates(selections: &[(String, Vec<String>)]) -> Result<()> {
    println!("\n{}", "🚀 Starting Updates".green().bold());
    println!("{}", "─".repeat(50).green());

    for (category, items) in selections {
        println!("\n{}  Updating {}...", "📦".yellow(), category.bold());

        match category.as_str() {
            "0-Core Workspace" => {
                update_workspace()?;
            }
            "Cargo Tools" => {
                update_cargo(items)?;
            }
            "Rust Toolchain" => {
                rustup_checker::update_rustup()?;
                println!("   ✅  Rust toolchain updated");
            }
            "NPM Packages" => {
                npm_checker::update_npm()?;
                println!("   ✅  NPM packages updated");
            }
            "Python Packages" => {
                pip_checker::update_pip()?;
                println!("   ✅  Python packages updated");
            }
            "Neovim Plugins" => {
                neovim_checker::update_neovim()?;
            }
            "Yazi Packages" => {
                yazi_checker::update_yazi()?;
            }
            "Git Repositories" => {
                git_checker::update_git_repos()?;
            }
            "Firmware" => {
                firmware_checker::update_firmware()?;
            }
            "Flatpak" => {
                flatpak_checker::update_flatpak()?;
            }
            _ => {
                println!("   {}  Category not implemented yet", "⚠️".yellow());
            }
        }
    }

    println!("\n{}  Updates complete!", "✨".green());
    Ok(())
}

/// Update 0-Core workspace
fn update_workspace() -> Result<()> {
    println!("   Running: cargo build --release");

    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(faelight_core::paths::core_dir())
        .status()
        .context("Failed to build workspace")?;

    if status.success() {
        println!("   {}  Workspace rebuilt", "✅".green());
    } else {
        println!("   {}  Build failed", "❌".red());
    }

    Ok(())
}

/// Update cargo tools
fn update_cargo(items: &[String]) -> Result<()> {
    println!("   Running: cargo install-update -a");

    let status = Command::new("cargo")
        .arg("install-update")
        .arg("-a")
        .args(items)
        .status()
        .context("Failed to update cargo tools - is cargo-update installed?")?;

    if status.success() {
        println!("   {}  Tools updated", "✅".green());
    } else {
        println!("   {}  Update failed", "❌".red());
    }

    Ok(())
}

/// Output results as JSON
fn output_json(categories: &[UpdateCategory], total: usize) -> Result<()> {
    use chrono::Utc;

    #[derive(serde::Serialize)]
    struct JsonOutput {
        version: String,
        timestamp: String,
        total_updates: usize,
        categories: Vec<UpdateCategory>,
    }

    let output = JsonOutput {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now().to_rfc3339(),
        total_updates: total,
        categories: categories.to_vec(),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Lock 0-core before updates
/// Lock 0-core before updates
/// Unlock 0-core after updates
/// Check if we're in 0-core directory
fn is_in_core() -> bool {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.contains("0-core")))
        .unwrap_or(false)
}

/// Run doctor check and return health percentage
fn run_doctor_final() -> Result<u32> {
    println!("\n{}  Running final health check...", "🏥".green());
    let output = Command::new("core")
        .args(["doctor", "run"])
        .output()
        .context("Failed to run doctor")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse health percentage
    for line in stdout.lines() {
        if line.contains("Health:") {
            if let Some(percent) = line
                .split_whitespace()
                .find(|s| s.ends_with('%'))
                .and_then(|s| s.trim_end_matches('%').parse::<u32>().ok())
            {
                return Ok(percent);
            }
        }
    }

    Ok(100) // Default to 100 if not found
}

/// Check git status
fn check_git_status() -> Result<()> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "-b"])
        .current_dir(faelight_core::paths::core_dir())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("[ahead") {
        println!("{}  Git: Commits need to be pushed", "⚠️".yellow());
        println!("    Run: git push");
    } else if stdout.lines().nth(1).is_some() {
        println!("{}  Git: Uncommitted changes", "⚠️".yellow());
    } else {
        println!("{}  Git: Clean and synced", "✅".green());
    }

    Ok(())
}

/// Check for .pacnew files
/// Cleanup caches after updates
fn cleanup_caches() -> Result<()> {
    // Cargo cache
    if let Err(e) = cleanup_checker::cleanup_cargo_cache() {
        println!("    {}  Cargo cache cleanup failed: {}", "⚠️".yellow(), e);
    } else {
        println!("    {}  Cargo cache cleaned", "✓".green());
    }

    Ok(())
}

/// Update prompt cache after successful update
fn update_prompt_cache() -> Result<()> {
    println!("🔄  Updating prompt cache...");

    // Run the prompt-update-count script to refresh the cache
    let home = std::env::var("HOME")?;
    let script = format!("{}/0-core/scripts/prompt-update-count", home);

    if std::path::Path::new(&script).exists() {
        let output = std::process::Command::new(&script)
            .output()
            .context("Failed to update prompt cache")?;

        if output.status.success() {
            println!("   ✅  Prompt cache updated");
        } else {
            println!("   ⚠️  Prompt cache update failed (non-critical)");
        }
    } else {
        println!("   ℹ️  Prompt script not found (skipping)");
    }

    Ok(())
}
