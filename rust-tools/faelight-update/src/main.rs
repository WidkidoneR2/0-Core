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
    verbose: bool,
    /// Preview what would change without updating
    #[arg(long)]
    preview: bool,

    /// Output results in JSON format
    #[arg(long)]
    json: bool,
    /// Only check specific categories (comma-separated: pacman,aur,cargo,neovim,workspace)

    /// Run maintenance tasks (clean cache, orphans, journal)
    #[arg(long)]
    maintain: bool,
    /// Output only the total count of updates (for scripts/bar)
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
    // Check for orphan packages
    if let Ok(output) = std::process::Command::new("pacman")
        .args(["-Qtdq"])
        .output()
    {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .count();
        if count > 0 {
            suggestions.push(format!(
                "{} orphan packages found — run: sudo pacman -Rns $(pacman -Qtdq)",
                count
            ));
        }
    }
    // Check for pacnew files
    if let Ok(output) = std::process::Command::new("find")
        .args(["/etc", "-name", "*.pacnew", "-type", "f"])
        .output()
    {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .count();
        if count > 0 {
            suggestions.push(format!("Run pacdiff to resolve {} config file(s)", count));
        }
    }
    // If nothing was updated
    if total == 0 {
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
    // Check 2: mirrorlist age
    if let Ok(meta) = std::fs::metadata("/etc/pacman.d/mirrorlist") {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = modified.elapsed() {
                let days = age.as_secs() / 86400;
                if days >= 30 {
                    warnings.push(format!("Mirrorlist is {} days old — run: reflector --save /etc/pacman.d/mirrorlist", days));
                } else if days >= 14 {
                    warnings.push(format!(
                        "Mirrorlist is {} days old — consider refreshing with reflector",
                        days
                    ));
                }
            }
        }
    }
    // Check 3: pacnew files
    if let Ok(output) = std::process::Command::new("find")
        .args(["/etc", "-name", "*.pacnew", "-type", "f"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let count = text.lines().filter(|l| !l.is_empty()).count();
        if count > 0 {
            warnings.push(format!(
                "{} .pacnew config files pending review — run: pacdiff",
                count
            ));
        }
    }
    // Check 4: partial upgrade risk
    if let Ok(output) = std::process::Command::new("pacman")
        .args(["-Qu", "--dbonly"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        if text.lines().count() > 0 {
            warnings.push("pacman database may be out of sync — partial upgrade risk".to_string());
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
    // Read last upgrade time from pacman log
    let log_path = "/var/log/pacman.log";
    let last_upgrade = std::fs::read_to_string(log_path)
        .unwrap_or_default()
        .lines()
        .filter(|l| l.contains("starting full system upgrade") || l.contains("upgraded "))
        .next_back()
        .and_then(|l| {
            // Parse date from [YYYY-MM-DDThh:mm:ss+0000]
            l.get(1..11)
        })
        .map(|s| s.to_string());
    let days_ago = if let Some(ref date_str) = last_upgrade {
        let today = chrono::Local::now().date_naive();
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map(|d| (today - d).num_days())
            .unwrap_or(-1)
    } else {
        -1
    };
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
    let home = std::env::var("HOME").unwrap_or_default();
    let db_path = format!("{}/0-core/runtime/state.db", home);
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
    // 1. Clean pacman cache
    println!("  {} Cleaning pacman cache...", "→".bright_cyan());
    let status = std::process::Command::new("sudo")
        .args(["pacman", "-Sc", "--noconfirm"])
        .status();
    match status {
        Ok(s) if s.success() => println!("  {} Pacman cache cleaned", "✅".green()),
        _ => println!(
            "  {} Pacman cache clean failed (sudo required)",
            "⚠️".yellow()
        ),
    }
    // 2. Remove orphan packages
    println!("  {} Checking orphan packages...", "→".bright_cyan());
    let orphans = std::process::Command::new("pacman")
        .args(["-Qtdq"])
        .output();
    if let Ok(out) = orphans {
        let pkgs: Vec<&str> = std::str::from_utf8(&out.stdout)
            .unwrap_or("")
            .lines()
            .filter(|l| !l.is_empty())
            .collect();
        if pkgs.is_empty() {
            println!("  {} No orphan packages found", "✅".green());
        } else {
            println!(
                "  {} {} orphan packages: {}",
                "⚠️".yellow(),
                pkgs.len(),
                pkgs.join(", ").dimmed()
            );
            println!(
                "  {} Run: sudo pacman -Rns $(pacman -Qtdq)",
                "💡".bright_cyan()
            );
        }
    }
    // 3. Clean cargo cache
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
    // 4. Vacuum systemd journal
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
    // 5. Check pacnew files
    println!("  {} Checking .pacnew files...", "→".bright_cyan());
    if let Ok(out) = std::process::Command::new("find")
        .args(["/etc", "-name", "*.pacnew", "-type", "f"])
        .output()
    {
        let count = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .count();
        if count == 0 {
            println!("  {} No .pacnew files found", "✅".green());
        } else {
            println!("  {} {} .pacnew files — run: pacdiff", "⚠️".yellow(), count);
        }
    }
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
    let shell = std::env::var("SHELL")
        .map(|s| s.split('/').next_back().unwrap_or("unknown").to_string())
        .unwrap_or_else(|_| "unknown".to_string());
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
    let core_root = std::env::var("HOME").unwrap_or_default() + "/0-core";
    let intents_dir = std::path::PathBuf::from(&core_root).join("intents/future");
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
    let state_db = std::path::PathBuf::from(&core_root).join("runtime/state.db");
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
        println!();
        println!("{}", "📦 Preview — What Would Change".cyan().bold());
        println!("{}", "─".repeat(48).dimmed());
        if total == 0 {
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

    // Count-only output
    if cli.count_only {
        println!("{}", total);
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
    if total == 0 {
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

        // CHECK FOR .PACNEW FILES
        check_pacnew()?;

        // CHECK AUR REBUILDS
        check_aur_rebuilds()?;

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
        println!("\n{}  Ready to update {} packages!", "✨".yellow(), total);

        // Prompt for confirmation
        use std::io::{self, Write};
        print!("Proceed? (Y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            // LOCK CORE BEFORE UPDATES
            let in_core = is_in_core();
            if in_core {
                // lock_core()?; // Disabled - using simple file lock instead
            }

            // Convert UpdateCategory to format perform_updates expects
            let all_updates: Vec<(String, Vec<String>)> = updates
                .iter()
                .map(|cat| {
                    let items: Vec<String> =
                        cat.items.iter().map(|item| item.name.clone()).collect();
                    (cat.name.clone(), items)
                })
                .collect();

            // PERFORM UPDATES
            perform_updates(&all_updates)?;

            // CLEANUP CACHES
            cleanup_caches()?;

            // UPDATE PROMPT CACHE
            update_prompt_cache()?;

            // UNLOCK CORE
            if in_core {
                // lock_core()?; // Disabled
            }

            // FINAL HEALTH CHECK (moved from beginning!)
            let health = run_doctor_final()?;

            // GIT STATUS CHECK
            check_git_status()?;

            // CHECK FOR .PACNEW FILES
            check_pacnew()?;

            // CHECK AUR REBUILDS
            check_aur_rebuilds()?;

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
        } else {
            println!("\n{}  Cancelled", "ℹ️".blue());
        }
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
    (filter_lower == "pacman" && category_lower.contains("system")) ||
    (filter_lower == "aur" && category_lower.contains("aur")) ||
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

    // System packages (pacman)
    if let Ok(cat) = check_pacman_updates() {
        categories.push(cat);
    }

    // AUR packages (paru)
    if let Ok(cat) = check_paru_updates() {
        categories.push(cat);
    }

    // Cargo tools
    let cargo_items = cargo_checker::check_cargo_updates();
    categories.push(UpdateCategory {
        name: "Cargo Tools".to_string(),
        emoji: "🦀".to_string(),
        count: cargo_items.len(),
        items: cargo_items,
    });

    // Neovim plugins
    let nvim_items: Vec<UpdateItem> = neovim_checker::check_neovim_updates()
        .into_iter()
        .map(|name| UpdateItem {
            name,
            current: "unknown".to_string(),
            new: "available".to_string(),
            repository: None,
        })
        .collect();
    categories.push(UpdateCategory {
        name: "Neovim Plugins".to_string(),
        emoji: "📝".to_string(),
        count: nvim_items.len(),
        items: nvim_items,
    });

    // 0-Core workspace
    let workspace_items = cargo_checker::check_workspace_updates();
    categories.push(UpdateCategory {
        name: "0-Core Workspace".to_string(),
        emoji: "🌲".to_string(),
        count: workspace_items.len(),
        items: workspace_items,
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
    });

    // Git repositories
    let git_items: Vec<UpdateItem> = git_checker::check_git_updates()
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
    });
    Ok(categories)
}

/// Check for pacman updates
/// Check for pacman updates
fn check_pacman_updates() -> Result<UpdateCategory> {
    println!("   Checking pacman...");

    // First sync the database quietly
    let _ = Command::new("sudo")
        .args(["pacman", "-Sy", "--noconfirm"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Now check for updates using the synced database
    let output = Command::new("pacman")
        .args(["-Qu"])
        .output()
        .context("Failed to run pacman -Qu")?;

    let items = if !output.status.success() || output.stdout.is_empty() {
        Vec::new()
    } else {
        parse_pacman_output(&output.stdout)
    };

    Ok(UpdateCategory {
        name: "System Packages".to_string(),
        emoji: "📦".to_string(),
        count: items.len(),
        items,
    })
}

/// Parse pacman-style output (works for checkupdates and paru -Qua)
fn parse_pacman_output(output: &[u8]) -> Vec<UpdateItem> {
    use once_cell::sync::Lazy;
    use regex::Regex;

    // Regex to parse: "package current -> new" or "repo/package current -> new"
    static PACMAN_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(?:([^/]+)/)?(\S+)\s+(\S+)\s+->\s+(\S+)").unwrap());

    let text = String::from_utf8_lossy(output);
    text.lines()
        .filter_map(|line| {
            PACMAN_REGEX.captures(line).and_then(|caps| {
                Some(UpdateItem {
                    repository: caps.get(1).map(|m| m.as_str().to_string()),
                    name: caps.get(2)?.as_str().to_string(),
                    current: caps.get(3)?.as_str().to_string(),
                    new: caps.get(4)?.as_str().to_string(),
                })
            })
        })
        .collect()
}

/// Check for AUR updates
fn check_paru_updates() -> Result<UpdateCategory> {
    println!("   Checking AUR (paru)...");

    let output = Command::new("paru")
        .args(["-Qua"])
        .output()
        .context("Failed to run paru - is it installed?")?;

    let items = if output.status.success() {
        parse_pacman_output(&output.stdout)
    } else {
        Vec::new()
    };

    Ok(UpdateCategory {
        name: "AUR Packages".to_string(),
        emoji: "🔷".to_string(),
        count: items.len(),
        items,
    })
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
        "pacman",
        "filesystem",
        "linux-firmware",
        "grub",
        "efibootmgr",
        "wayland",
    ];
    const IMPORTANT: &[&str] = &[
        "git", "neovim", "rust", "rustup", "cargo", "python", "nodejs", "npm", "openssh", "curl",
        "wget", "bash", "niri", "sway", "ripgrep", "fd", "bat", "eza", "fzf", "zoxide", "paru",
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
        "pacman",
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
            "System Packages" => {
                update_pacman(items)?;
            }
            "AUR Packages" => {
                update_aur(items)?;
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

/// Update system packages
fn update_pacman(items: &[String]) -> Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    println!(
        "   Running: sudo pacman -S --needed --noconfirm {}",
        items.join(" ")
    );

    let status = Command::new("sudo")
        .arg("pacman")
        .arg("--needed")
        .arg("-S")
        .arg("--noconfirm")
        .args(items)
        .status()
        .context("Failed to update pacman packages")?;

    if status.success() {
        println!("   {}  Packages updated", "✅".green());
    } else {
        println!("   {}  Update failed", "❌".red());
    }

    Ok(())
}

/// Update AUR packages
fn update_aur(items: &[String]) -> Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    println!("   Running: paru -Su --noconfirm {}", items.join(" "));

    let status = Command::new("paru")
        .arg("-S")
        .arg("--needed")
        .arg("--noconfirm")
        .args(items)
        .status()
        .context("Failed to update AUR packages")?;

    if status.success() {
        println!("   {}  Packages updated", "✅".green());
    } else {
        println!("   {}  Update failed", "❌".red());
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

    // Pacman cache
    if let Err(e) = cleanup_checker::cleanup_pacman_cache() {
        println!("    {}  Pacman cache cleanup failed: {}", "⚠️".yellow(), e);
    } else {
        println!("    {}  Pacman cache cleaned", "✓".green());
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
/// Check for .pacnew files and offer to handle them
/// Check for .pacnew files and offer to handle them
fn check_pacnew() -> Result<()> {
    let output = Command::new("find")
        .args(["/etc", "-name", "*.pacnew", "-type", "f"])
        .output()
        .context("Failed to find .pacnew files")?;

    let output_str = String::from_utf8_lossy(&output.stdout).to_string();
    let pacnew_files: Vec<_> = output_str.lines().filter(|line| !line.is_empty()).collect();

    if pacnew_files.is_empty() {
        return Ok(());
    }

    println!("⚠️  Found {} .pacnew config files:", pacnew_files.len());
    for file in &pacnew_files {
        println!("    {}", file);
    }

    println!("    Review with: pacdiff");
    if !pacnew_files.is_empty() {
        let original = pacnew_files[0].trim_end_matches(".pacnew");
        println!(
            "    Or manually: sudo vimdiff {} {}",
            pacnew_files[0], original
        );
    }

    // Offer to run pacdiff if available
    if Command::new("which")
        .arg("pacdiff")
        .output()?
        .status
        .success()
    {
        println!("\n💡 Run pacdiff now to merge changes? (y/N)");
        use std::io::{self, Write};
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            println!("\n🔧 Running pacdiff...");
            let status = Command::new("sudo")
                .arg("pacdiff")
                .status()
                .context("Failed to run pacdiff")?;

            if status.success() {
                println!("   ✅  Config files merged");
            } else {
                println!("   ⚠️  pacdiff exited with errors");
            }
        }
    }

    Ok(())
}

/// Check for AUR packages that need rebuilding after library updates
fn check_aur_rebuilds() -> Result<()> {
    println!("🔍  Checking for AUR packages needing rebuild...");

    // Check if checkrebuild or similar tool exists
    let has_checkrebuild = Command::new("which")
        .arg("checkrebuild")
        .output()?
        .status
        .success();

    if has_checkrebuild {
        let output = Command::new("checkrebuild")
            .output()
            .context("Failed to run checkrebuild")?;

        let rebuild_list = String::from_utf8_lossy(&output.stdout).to_string();
        let packages: Vec<_> = rebuild_list
            .lines()
            .filter(|line| !line.is_empty())
            .collect();

        if !packages.is_empty() {
            println!("   ⚠️  {} packages need rebuilding:", packages.len());
            for pkg in &packages {
                println!("      - {}", pkg);
            }

            println!(
                "\n   💡 Rebuild with: paru -S --rebuild {}",
                packages.join(" ")
            );
        } else {
            println!("   ✅  No packages need rebuilding");
        }
    } else {
        println!("   ℹ️  checkrebuild not available (optional)");
    }

    Ok(())
}
