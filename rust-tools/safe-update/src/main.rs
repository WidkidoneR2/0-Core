//! safe-update v1.0.0 - Safe System Updates
//! 🌲 Faelight Forest

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const VERSION: &str = "1.0.0";

// ANSI colors
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[0;36m";
const GRAY: &str = "\x1b[0;90m";
const NC: &str = "\x1b[0m";

struct Config {
    dry_run: bool,
    skip_confirmation: bool,
    skip_snapshot: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Parse flags
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("safe-update v{}", VERSION);
        return;
    }
    
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        show_help();
        return;
    }
    
    if args.contains(&"--health".to_string()) {
        run_health_check(false);
        std::process::exit(0);
    }
    
    let config = Config {
        dry_run: args.contains(&"--dry-run".to_string()),
        skip_confirmation: args.contains(&"--yes".to_string()) || args.contains(&"-y".to_string()),
        skip_snapshot: args.contains(&"--skip-snapshot".to_string()),
    };
    
    // Start update process
    run_safe_update(&config);
}

fn run_safe_update(config: &Config) {
    println!();
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    if config.dry_run {
        println!("{}🔍 Safe System Update v{} - DRY RUN{}", CYAN, VERSION, NC);
    } else {
        println!("{}🛡️  Safe System Update v{}{}", CYAN, VERSION, NC);
    }
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!();
    
    // Pre-flight checks
    println!("{}🏥 Pre-flight Checks{}", CYAN, NC);
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", CYAN, NC);
    
    if !run_health_check(!config.skip_snapshot) {
        log_error("Pre-flight checks failed - aborting");
        std::process::exit(1);
    }
    
    println!();
    
    // Preview updates
    if !config.dry_run {
        println!("{}📋 Update Preview{}", CYAN, NC);
        println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", CYAN, NC);
        log_info("Running dry-run to preview updates...");
        println!();
        
        run_interactive("topgrade", &["--dry-run"]);
        
        println!();
    }
    
    // Confirmation
    if config.dry_run {
        println!();
        println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
        log_info("Dry-run complete! No changes made.");
        println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
        println!();
        return;
    }
    
    if !config.skip_confirmation {
        print!("\n{}⚠️  Proceed with update? (yes/no): {}", YELLOW, NC);
        io::stdout().flush().unwrap();
        
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        
        if response.trim() != "yes" {
            log_info("Update cancelled by user");
            std::process::exit(2);
        }
        println!();
    }
    
    // Create pre-update snapshot
    let pre_snapshot = if !config.skip_snapshot {
        println!("{}📸 Creating Snapshots{}", CYAN, NC);
        println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", CYAN, NC);
        
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
    
    // Run update
    println!("{}🔄 System Update{}", CYAN, NC);
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", CYAN, NC);
    log_info("Running topgrade...");
    println!();
    
    let update_success = handle_update();
    
    println!();
    
    // Post-update checks
    println!("{}📋 Post-Update Checks{}", CYAN, NC);
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", CYAN, NC);
    
    // Check for .pacnew files
    check_pacnew_files();
    
    println!();
    
    // Create post-update snapshot
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
    
    // Health check
    println!("{}🏥 System Health Check{}", CYAN, NC);
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", CYAN, NC);
    log_info("Running system health check...");
    println!();
    
    run_doctor();
    
    println!();
    
    // Update entropy baseline
    println!("{}📊 Drift Tracking{}", CYAN, NC);
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", CYAN, NC);
    
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
    
    // Show rollback instructions
    if let (Some(pre), Some(post)) = (pre_snapshot, post_snapshot) {
        println!("{}💡 Rollback Available{}", CYAN, NC);
        println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", CYAN, NC);
        println!("  {}Before:{} Snapshot #{}", GRAY, NC, pre);
        println!("  {}After: {} Snapshot #{}", GRAY, NC, post);
        println!();
        println!("  {}To rollback: {}sudo snapper -c root rollback {}{}", 
                 GRAY, YELLOW, pre, NC);
        println!();
    }
    
    // Save update log
    save_update_log(update_success, pre_snapshot, post_snapshot);
    
    // Final result
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    
    if update_success {
        log_success("Safe update complete! System is healthy! 🌲");
    } else {
        log_error("Update had issues - please review logs");
    }
    
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!();
}

fn run_health_check(check_snapper: bool) -> bool {
    let mut all_healthy = true;
    
    // Check snapper (if needed)
    if check_snapper {
        print!("  Checking snapper... ");
        if check_command_exists("snapper") {
            // Try to access snapper
            if run_sudo(&["snapper", "-c", "root", "list"]) {
                println!("{}✅{}", GREEN, NC);
            } else {
                println!("{}⚠️  Available but not configured{}", YELLOW, NC);
                all_healthy = false;
            }
        } else {
            println!("{}❌ Not installed{}", RED, NC);
            println!("      {}Install with: yay -S snapper{}", GRAY, NC);
            println!("      {}Or use: safe-update --skip-snapshot{}", GRAY, NC);
            all_healthy = false;
        }
    }
    
    // Check internet
    print!("  Checking internet connection... ");
    if test_internet() {
        println!("{}✅{}", GREEN, NC);
    } else {
        println!("{}❌ No connection{}", RED, NC);
        all_healthy = false;
    }
    
    // Check disk space
    print!("  Checking disk space... ");
    if let Some(free_gb) = get_free_space() {
        if free_gb >= 2.0 {
            println!("{}✅ {:.1} GB free{}", GREEN, free_gb, NC);
        } else {
            println!("{}❌ Only {:.1} GB free (need 2GB){}", RED, free_gb, NC);
            all_healthy = false;
        }
    } else {
        println!("{}⚠️  Could not determine{}", YELLOW, NC);
    }
    
    // Check system health (if doctor exists)
    if check_command_exists("doctor") {
        print!("  Checking system health... ");
        io::stdout().flush().unwrap();
        
        if Command::new("doctor").output()
            .map(|o| o.status.success())
            .unwrap_or(false) 
        {
            println!("{}✅ 100%{}", GREEN, NC);
        } else {
            println!("{}⚠️  System has warnings{}", YELLOW, NC);
        }
    }
    
    all_healthy
}

fn handle_update() -> bool {
    if run_interactive("topgrade", &[]) {
        log_success("Update completed successfully!");
        true
    } else {
        log_warning("Update encountered an issue - checking for yay problems...");
        
        // Check if it's the yay library issue
        let yay_check = Command::new("yay")
            .arg("--version")
            .output();
        
        let needs_rebuild = yay_check
            .map(|o| String::from_utf8_lossy(&o.stderr).contains("error while loading shared libraries"))
            .unwrap_or(false);
        
        if needs_rebuild {
            log_info("Detected yay library mismatch - rebuilding yay...");
            println!();
            
            if rebuild_yay() {
                log_success("yay rebuilt successfully!");
                println!();
                
                log_info("Retrying system update...");
                println!();
                
                if run_interactive("topgrade", &[]) {
                    log_success("Update completed after yay rebuild!");
                    true
                } else {
                    log_error("Update still failed - manual intervention needed");
                    false
                }
            } else {
                log_error("Failed to rebuild yay");
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
    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    
    let log_dir = PathBuf::from(&home).join(".local/share/faelight/update-logs");
    
    if let Err(_) = fs::create_dir_all(&log_dir) {
        return;
    }
    
    let timestamp = get_timestamp();
    let log_file = log_dir.join(format!("{}.log", timestamp));
    
    let mut log_content = String::new();
    log_content.push_str(&format!("Update Log - {}\n", timestamp));
    log_content.push_str(&format!("Status: {}\n", if success { "SUCCESS" } else { "FAILED" }));
    
    if let Some(pre_num) = pre {
        log_content.push_str(&format!("Pre-snapshot: #{}\n", pre_num));
    }
    
    if let Some(post_num) = post {
        log_content.push_str(&format!("Post-snapshot: #{}\n", post_num));
    }
    
    if fs::write(&log_file, log_content).is_ok() {
        println!();
        println!("{}Update log: {}{}", GRAY, log_file.display(), NC);
    }
}

fn create_snapshot(desc: &str) -> Option<u32> {
    let output = Command::new("sudo")
        .args(["snapper", "-c", "root", "create", "--description", desc, "--print-number"])
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
    let output = Command::new("df")
        .args(["-BG", "/"])
        .output()
        .ok()?;
    
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

fn rebuild_yay() -> bool {
    // Clean up any existing yay directory
    Command::new("rm")
        .args(["-rf", "/tmp/yay"])
        .status()
        .ok();
    
    // Clone yay
    let clone = Command::new("git")
        .args(["clone", "https://aur.archlinux.org/yay.git"])
        .current_dir("/tmp")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    
    if !clone.map(|s| s.success()).unwrap_or(false) {
        log_error("Failed to clone yay repository");
        return false;
    }
    
    // Build yay
    let build = Command::new("makepkg")
        .args(["-si", "--noconfirm"])
        .current_dir("/tmp/yay")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    
    build.map(|s| s.success()).unwrap_or(false)
}

fn log_info(msg: &str) {
    println!("  {}ℹ {}{}",  CYAN, NC, msg);
}

fn log_success(msg: &str) {
    println!("  {}✅ {}{}", GREEN, NC, msg);
}

fn log_warning(msg: &str) {
    println!("  {}⚠️  {}{}", YELLOW, NC, msg);
}

fn log_error(msg: &str) {
    println!("  {}❌ {}{}", RED, NC, msg);
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

fn show_help() {
    println!("{}🛡️  safe-update v{}{} - Safe System Updates", CYAN, VERSION, NC);
    println!();
    println!("USAGE:");
    println!("   safe-update [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("   --dry-run            Preview updates without applying");
    println!("   --yes, -y            Skip confirmation prompt");
    println!("   --skip-snapshot      Don't create snapshots");
    println!("   --health             Run pre-flight checks only");
    println!("   --version, -v        Show version");
    println!("   --help, -h           Show this help");
    println!();
    println!("EXAMPLES:");
    println!("   safe-update                      # Standard update with all safety");
    println!("   safe-update --dry-run            # Preview what would update");
    println!("   safe-update --yes                # Skip confirmation");
    println!("   safe-update --skip-snapshot      # Fast update without snapshots");
    println!("   safe-update --health             # Check if system ready to update");
    println!();
    println!("SAFETY FEATURES:");
    println!("   ✅ Pre-flight health checks");
    println!("   ✅ Btrfs snapshots (before/after)");
    println!("   ✅ Automatic yay recovery");
    println!("   ✅ .pacnew file detection");
    println!("   ✅ Post-update verification");
    println!("   ✅ Rollback instructions");
    println!("   ✅ Drift baseline updates");
    println!("   ✅ Update logging");
    println!();
    println!("PHILOSOPHY:");
    println!("   'Manual control over automation. Reliability over convenience.'");
    println!("   Updates are never automatic - you control when they happen.");
}
