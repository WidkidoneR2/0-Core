use std::process::{self, Command, Stdio};

// ANSI colors
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[0;36m";
const NC: &str = "\x1b[0m";

fn main() {
    println!();
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!("{}🔄 Safe System Update{}", CYAN, NC);
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!();
    
    // Pre-update snapshot
    log_info("Creating pre-update snapshot...");
    let timestamp = get_timestamp();
    let desc = format!("Before update {}", timestamp);
    
    if run_sudo(&["snapper", "-c", "root", "create", "--description", &desc]) {
        log_success("Pre-update snapshot created");
    } else {
        log_warning("Could not create snapshot (non-critical)");
    }
    println!();
    
    // Try topgrade update
    log_info("Running topgrade...");
    println!();
    
    let update_success = if run_interactive("topgrade", &[]) {
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
                    process::exit(1);
                }
            } else {
                log_error("Failed to rebuild yay");
                process::exit(1);
            }
        } else {
            log_error("Update failed for unknown reason - check logs");
            process::exit(1);
        }
    };
    
    println!();
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!("{}📋 Post-Update Checks{}", CYAN, NC);
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!();
    
    // Check for .pacnew files
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
    println!();
    
    // Post-update snapshot
    log_info("Creating post-update snapshot...");
    let timestamp = get_timestamp();
    let desc = format!("After update {}", timestamp);
    
    if run_sudo(&["snapper", "-c", "root", "create", "--description", &desc]) {
        log_success("Post-update snapshot created");
    } else {
        log_warning("Could not create snapshot (non-critical)");
    }
    println!();
    
    // Health check
    log_info("Running system health check...");
    println!();
    
    if Command::new("which").arg("dot-doctor").output()
        .map(|o| o.status.success()).unwrap_or(false) 
    {
        run_interactive("dot-doctor", &[]);
    } else {
        log_warning("dot-doctor not found - skipping health check");
    }
    
    println!();
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    
    if update_success {
        log_success("Safe update complete! System is healthy! 🌲");
    } else {
        log_error("Update had issues - please review");
    }
    
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!();
}

fn log_info(msg: &str) {
    println!("{}ℹ {}{}",  CYAN, NC, msg);
}

fn log_success(msg: &str) {
    println!("{}✅ {}{}", GREEN, NC, msg);
}

fn log_warning(msg: &str) {
    println!("{}⚠️  {}{}", YELLOW, NC, msg);
}

fn log_error(msg: &str) {
    println!("{}❌ {}{}", RED, NC, msg);
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
