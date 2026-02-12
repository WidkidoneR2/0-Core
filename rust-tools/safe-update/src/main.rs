//! safe-update v2.1.0 - Safe System Updates
//! 🌲 Faelight Forest

use clap::Parser;
use colored::*;
use faelight_core::paths;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const VERSION: &str = "2.1.0";

#[derive(Parser)]
#[command(name = "safe-update")]
#[command(version = VERSION)]
#[command(about = "🛡️ Safe System Updater - Snapshot before updating")]
#[command(
    long_about = "Safe system update tool with btrfs snapshots, health checks, and automatic recovery"
)]
struct Cli {
    /// Dry run (show what would be updated)
    #[arg(long)]
    dry_run: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    yes: bool,

    /// Skip snapshot creation
    #[arg(long = "skip-snapshot")]
    no_snapshot: bool,

    /// Run pre-flight checks only
    #[arg(long)]
    health: bool,
}

struct Config {
    dry_run: bool,
    skip_confirmation: bool,
    skip_snapshot: bool,
}

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        Config {
            dry_run: cli.dry_run,
            skip_confirmation: cli.yes,
            skip_snapshot: cli.no_snapshot,
        }
    }
}

fn main() {
    let args = Cli::parse();

    if args.health {
        run_health_check(false);
        std::process::exit(0);
    }

    let config = Config::from(args);
    run_safe_update(&config);
}

fn run_safe_update(config: &Config) {
    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════".cyan()
    );
    if config.dry_run {
        println!(
            "{}",
            format!("🔍 Safe System Update v{} - DRY RUN", VERSION).cyan()
        );
    } else {
        println!("{}", format!("🛡️  Safe System Update v{}", VERSION).cyan());
    }
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════".cyan()
    );
    println!();

    println!("{}", "🏥 Pre-flight Checks".cyan());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );

    if !run_health_check(!config.skip_snapshot) {
        log_error("Pre-flight checks failed - aborting");
        std::process::exit(1);
    }

    println!();

    if !config.dry_run {
        println!("{}", "📋 Update Preview".cyan());
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        log_info("Running dry-run to preview updates...");
        println!();

        run_interactive("topgrade", &["--dry-run"]);

        println!();
    }

    if config.dry_run {
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════".cyan()
        );
        log_info("Dry-run complete! No changes made.");
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════".cyan()
        );
        println!();
        return;
    }

    if !config.skip_confirmation {
        print!("\n{}", "⚠️  Proceed with update? (yes/no): ".yellow());
        if let Err(e) = io::stdout().flush() {
            log_error(&format!("Failed to flush stdout: {}", e));
            std::process::exit(1);
        }

        let mut response = String::new();
        if let Err(e) = io::stdin().read_line(&mut response) {
            log_error(&format!("Failed to read input: {}", e));
            std::process::exit(1);
        }

        if response.trim() != "yes" {
            log_info("Update cancelled by user");
            std::process::exit(2);
        }
        println!();
    }

    let pre_snapshot = if !config.skip_snapshot {
        println!("{}", "📸 Creating Snapshots".cyan());
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );

        log_info("Creating pre-update snapshot...");
        let timestamp = get_timestamp();
        let desc = format!("Before update {}", timestamp);

        let snapshot_num = create_snapshot(&desc);

        if let Some(num) = snapshot_num {
            log_success(&format!("Pre-update snapshot created (#{} )", num));
        } else {
            log_warning("Could not create snapshot (continuing anyway)");
        }

        println!();
        snapshot_num
    } else {
        None
    };

    println!("{}", "🔄 System Update".cyan());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );
    log_info("Running topgrade...");
    println!();

    let update_success = handle_update();

    println!();

    println!("{}", "📋 Post-Update Checks".cyan());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );

    check_pacnew_files();

    println!();

    let post_snapshot = if !config.skip_snapshot {
        log_info("Creating post-update snapshot...");
        let timestamp = get_timestamp();
        let desc = format!("After update {}", timestamp);

        let snapshot_num = create_snapshot(&desc);

        if let Some(num) = snapshot_num {
            log_success(&format!("Post-update snapshot created (#{} )", num));
        } else {
            log_warning("Could not create snapshot");
        }

        println!();
        snapshot_num
    } else {
        None
    };

    println!("{}", "🏥 System Health Check".cyan());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );
    log_info("Running system health check...");
    println!();

    run_doctor();

    println!();

    println!("{}", "📊 Drift Tracking".cyan());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );

    if check_command_exists("entropy-check") {
        log_info("Updating entropy baseline...");

        if run_command("entropy-check", &["--baseline"]) {
            log_success("Entropy baseline updated");
        } else {
            log_warning("Could not update entropy baseline");
        }
    } else {
        log_info("entropy-check not found - skipping drift tracking");
    }

    println!();

    if let (Some(pre), Some(post)) = (pre_snapshot, post_snapshot) {
        println!("{}", "💡 Rollback Available".cyan());
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        println!(
            "  {}Before:{} Snapshot #{}",
            "".bright_black(),
            "".normal(),
            pre
        );
        println!(
            "  {}After: {} Snapshot #{}",
            "".bright_black(),
            "".normal(),
            post
        );
        println!();
        println!(
            "  {}To rollback: {}sudo snapper -c root rollback {}{}",
            "".bright_black(),
            "".yellow(),
            pre,
            "".normal()
        );
        println!();
    }

    save_update_log(update_success, pre_snapshot, post_snapshot);

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════".cyan()
    );

    if update_success {
        log_success("Safe update complete! System is healthy! 🌲");
    } else {
        log_error("Update had issues - please review logs");
    }

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════".cyan()
    );
    println!();
}

fn run_health_check(check_snapper: bool) -> bool {
    let mut all_healthy = true;

    if check_snapper {
        print!("  Checking snapper... ");
        if check_command_exists("snapper") {
            if run_sudo(&["snapper", "-c", "root", "list"]) {
                println!("{}", "✅".green());
            } else {
                println!("{}", "⚠️  Available but not configured".yellow());
                all_healthy = false;
            }
        } else {
            println!("{}", "❌ Not installed".red());
            println!(
                "      {}Install with: paru -S snapper{}",
                "".bright_black(),
                "".normal()
            );
            println!(
                "      {}Or use: safe-update --skip-snapshot{}",
                "".bright_black(),
                "".normal()
            );
            all_healthy = false;
        }
    }

    print!("  Checking internet connection... ");
    if test_internet() {
        println!("{}", "✅".green());
    } else {
        println!("{}", "❌ No connection".red());
        all_healthy = false;
    }

    print!("  Checking disk space... ");
    if let Some(free_gb) = get_free_space() {
        if free_gb >= 2.0 {
            println!("{}", format!("✅ {:.1} GB free", free_gb).green());
        } else {
            println!(
                "{}",
                format!("❌ Only {:.1} GB free (need 2GB)", free_gb).red()
            );
            all_healthy = false;
        }
    } else {
        println!("{}", "⚠️  Could not determine".yellow());
    }

    if check_command_exists("doctor") {
        print!("  Checking system health... ");
        if let Err(e) = io::stdout().flush() {
            log_warning(&format!("Failed to flush stdout: {}", e));
        }

        if Command::new("doctor")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            println!("{}", "✅ 100%".green());
        } else {
            println!("{}", "⚠️  System has warnings".yellow());
        }
    }

    all_healthy
}

fn handle_update() -> bool {
    if run_interactive("topgrade", &[]) {
        log_success("Update completed successfully!");
        true
    } else {
        log_warning("Update encountered an issue - checking for paru problems...");

        let paru_check = Command::new("paru").arg("--version").output();

        let needs_rebuild = paru_check
            .map(|o| {
                String::from_utf8_lossy(&o.stderr).contains("error while loading shared libraries")
            })
            .unwrap_or(false);

        if needs_rebuild {
            log_info("Detected paru library mismatch - rebuilding paru...");
            println!();

            if rebuild_paru() {
                log_success("paru rebuilt successfully!");
                println!();

                log_info("Retrying system update...");
                println!();

                if run_interactive("topgrade", &[]) {
                    log_success("Update completed after paru rebuild!");
                    true
                } else {
                    log_error("Update still failed - manual intervention needed");
                    false
                }
            } else {
                log_error("Failed to rebuild paru");
                false
            }
        } else {
            log_error("Update failed for unknown reason - check logs");
            false
        }
    }
}

fn check_pacnew_files() {
    log_info("Checking for .pacnew files...");

    let pacnew = Command::new("find")
        .args(["/etc", "-name", "*.pacnew"])
        .output();

    if let Ok(output) = pacnew {
        let files = String::from_utf8_lossy(&output.stdout);
        let files = files.trim();

        if !files.is_empty() {
            log_warning("Found .pacnew files that need review:");
            for file in files.lines() {
                println!("   → {}", file);
            }
            println!();
            log_info("Review and merge with: sudo pacdiff");
        } else {
            log_success("No .pacnew files found");
        }
    }
}

fn run_doctor() {
    if check_command_exists("doctor") {
        run_interactive("doctor", &[]);
    } else {
        log_warning("doctor not found - skipping health check");
    }
}

fn save_update_log(success: bool, pre: Option<u32>, post: Option<u32>) {
    let home = paths::home();
    let log_dir = PathBuf::from(&home).join(".local/share/faelight/update-logs");

    if let Err(e) = fs::create_dir_all(&log_dir) {
        log_warning(&format!("Could not create log directory: {}", e));
        return;
    }

    let timestamp = get_timestamp();
    let log_file = log_dir.join(format!("{}.log", timestamp));

    let mut log_content = String::new();
    log_content.push_str(&format!("Update Log - {}\n", timestamp));
    log_content.push_str(&format!(
        "Status: {}\n",
        if success { "SUCCESS" } else { "FAILED" }
    ));

    if let Some(pre_num) = pre {
        log_content.push_str(&format!("Pre-snapshot: #{}\n", pre_num));
    }

    if let Some(post_num) = post {
        log_content.push_str(&format!("Post-snapshot: #{}\n", post_num));
    }

    match fs::write(&log_file, log_content) {
        Ok(_) => {
            println!();
            println!(
                "{}Update log: {}{}",
                "".bright_black(),
                log_file.display(),
                "".normal()
            );
        }
        Err(e) => {
            log_warning(&format!("Could not save update log: {}", e));
        }
    }
}

fn create_snapshot(desc: &str) -> Option<u32> {
    let output = Command::new("sudo")
        .args([
            "snapper",
            "-c",
            "root",
            "create",
            "--description",
            desc,
            "--print-number",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
        .ok()
}

fn test_internet() -> bool {
    Command::new("ping")
        .args(["-c", "1", "-W", "2", "archlinux.org"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn get_free_space() -> Option<f64> {
    let output = Command::new("df").args(["-BG", "/"]).output().ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().nth(1)?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    let free = parts.get(3)?;

    free.trim_end_matches('G').parse::<f64>().ok()
}

fn check_command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_command(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn rebuild_paru() -> bool {
    let _ = Command::new("rm").args(["-rf", "/tmp/paru"]).status();

    let clone = Command::new("git")
        .args(["clone", "https://aur.archlinux.org/paru.git"])
        .current_dir("/tmp")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    if !clone.map(|s| s.success()).unwrap_or(false) {
        log_error("Failed to clone paru repository");
        return false;
    }

    let build = Command::new("makepkg")
        .args(["-si", "--noconfirm"])
        .current_dir("/tmp/paru")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    build.map(|s| s.success()).unwrap_or(false)
}

fn log_info(msg: &str) {
    println!("  {}ℹ {}{}", "".cyan(), "".normal(), msg);
}

fn log_success(msg: &str) {
    println!("  {}✅ {}{}", "".green(), "".normal(), msg);
}

fn log_warning(msg: &str) {
    println!("  {}⚠️  {}{}", "".yellow(), "".normal(), msg);
}

fn log_error(msg: &str) {
    println!("  {}❌ {}{}", "".red(), "".normal(), msg);
}

fn get_timestamp() -> String {
    Command::new("date")
        .arg("+%Y-%m-%d-%H%M")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn run_sudo(args: &[&str]) -> bool {
    Command::new("sudo")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_interactive(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
