//! faelight-bootstrap - One-Command 0-Core Setup
//! 🌲 Faelight Forest v6.2
//!
//! Usage: curl -fsSL https://raw.githubusercontent.com/USER/0-core/main/bootstrap.sh | bash

use colored::*;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};

const VERSION: &str = "0.1.0";

fn main() {
    println!("{}", "═".repeat(60).green());
    println!("{}", "🌲 Faelight Forest Bootstrap v0.1.0".green().bold());
    println!("{}", "═".repeat(60).green());
    println!();

    // Pre-flight checks
    check_arch_linux();
    check_not_root();
    check_internet();

    // Get user info
    let home = env::var("HOME").expect("HOME not set");
    let core_path = format!("{}/0-core", home);

    // Check if already installed
    if Path::new(&core_path).exists() {
        println!("{} 0-core already exists at {}", "⚠️".yellow(), core_path);
        println!("   Run {} to update instead", "git pull".cyan());
        
        if !confirm("Reinstall anyway?") {
            println!("{}", "Aborted.".red());
            exit(0);
        }
    }

    println!();
    println!("{}", "📦 Phase 1: Installing Dependencies".cyan().bold());
    println!("{}", "─".repeat(40));
    install_dependencies();

    println!();
    println!("{}", "📥 Phase 2: Cloning 0-Core".cyan().bold());
    println!("{}", "─".repeat(40));
    clone_repo(&home, &core_path);

    println!();
    println!("{}", "🔗 Phase 3: Stowing Packages".cyan().bold());
    println!("{}", "─".repeat(40));
    stow_packages(&core_path);

    println!();
    println!("{}", "🔧 Phase 4: Building Rust Tools".cyan().bold());
    println!("{}", "─".repeat(40));
    build_rust_tools(&core_path);

    println!();
    println!("{}", "🏥 Phase 5: Health Check".cyan().bold());
    println!("{}", "─".repeat(40));
    run_health_check(&core_path);

    println!();
    println!("{}", "═".repeat(60).green());
    println!("{}", "✅ Faelight Forest Installation Complete!".green().bold());
    println!("{}", "═".repeat(60).green());
    println!();
    println!("Next steps:");
    println!("  1. {} - Reload shell configuration", "source ~/.zshrc".cyan());
    println!("  2. {} - Start Sway", "sway".cyan());
    println!("  3. {} - Check system health", "dot-doctor".cyan());
    println!();
    println!("{}", "🌲 Welcome to the Forest!".green());
}

fn check_arch_linux() {
    print!("Checking Arch Linux... ");
    
    if !Path::new("/etc/arch-release").exists() {
        println!("{}", "FAIL".red());
        eprintln!("{} This script is for Arch Linux only", "Error:".red().bold());
        exit(1);
    }
    
    println!("{}", "OK".green());
}

fn check_not_root() {
    print!("Checking not root... ");
    
    if env::var("USER").unwrap_or_default() == "root" {
        println!("{}", "FAIL".red());
        eprintln!("{} Don't run as root. Run as normal user (sudo used internally)", "Error:".red().bold());
        exit(1);
    }
    
    println!("{}", "OK".green());
}

fn check_internet() {
    print!("Checking internet... ");
    
    let status = Command::new("ping")
        .args(["-c", "1", "-W", "3", "archlinux.org"])
        .output();
    
    match status {
        Ok(output) if output.status.success() => println!("{}", "OK".green()),
        _ => {
            println!("{}", "FAIL".red());
            eprintln!("{} No internet connection", "Error:".red().bold());
            exit(1);
        }
    }
}

fn confirm(prompt: &str) -> bool {
    use std::io::{stdin, stdout, Write};
    
    print!("{} [y/N] ", prompt);
    stdout().flush().unwrap();
    
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn run_cmd(cmd: &str, args: &[&str]) -> bool {
    let status = Command::new(cmd)
        .args(args)
        .status();
    
    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

fn run_cmd_sudo(args: &[&str]) -> bool {
    let status = Command::new("sudo")
        .args(args)
        .status();
    
    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

fn install_dependencies() {
    let packages = [
        // Core
        "git", "stow", "base-devel",
        // Sway & Desktop
        "sway", "foot", "fuzzel", "mako", "grim", "slurp", "wl-clipboard",
        "swaylock", "brightnessctl", "playerctl",
        // Shell
        "zsh", "starship", "eza", "bat", "fd", "fzf", "ripgrep", "yazi",
        // Editor
        "neovim", "lazygit",
        // Fonts
        "ttf-jetbrains-mono-nerd",
        // Rust
        "rustup",
        // System
        "pipewire", "wireplumber", "xdg-desktop-portal-wlr",
    ];

    println!("Installing {} packages...", packages.len());
    
    // Update first
    if !run_cmd_sudo(&["pacman", "-Sy", "--noconfirm"]) {
        eprintln!("{} Failed to update package database", "Error:".red().bold());
        exit(1);
    }

    // Install packages
    let mut args = vec!["pacman", "-S", "--needed", "--noconfirm"];
    args.extend(packages.iter());
    
    if !run_cmd_sudo(&args) {
        eprintln!("{} Failed to install packages", "Error:".red().bold());
        exit(1);
    }

    // Setup Rust
    println!("Setting up Rust toolchain...");
    run_cmd("rustup", &["default", "stable"]);

    println!("{} Dependencies installed", "✅".green());
}

fn clone_repo(home: &str, core_path: &str) {
    // Backup existing if present
    if Path::new(core_path).exists() {
        let backup = format!("{}/0-core.backup.{}", home, chrono_simple());
        println!("Backing up existing to {}", backup);
        fs::rename(core_path, &backup).expect("Failed to backup");
    }

    println!("Cloning 0-core repository...");
    
    // Try to get git username for proper URL
    let repo_url = "https://github.com/ChristianKaworworwa/0-core.git";
    
    if !run_cmd("git", &["clone", repo_url, core_path]) {
        eprintln!("{} Failed to clone repository", "Error:".red().bold());
        eprintln!("You may need to clone manually:");
        eprintln!("  git clone YOUR_REPO_URL {}", core_path);
        exit(1);
    }

    println!("{} Repository cloned", "✅".green());
}

fn chrono_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", secs)
}

fn stow_packages(core_path: &str) {
    let packages = [
        "shell-zsh",
        "prompt-starship", 
        "editor-nvim",
        "fm-yazi",
        "vcs-git",
        "wm-sway",
        "term-foot",
        "launcher-fuzzel",
    ];

    // Change to core directory
    env::set_current_dir(core_path).expect("Failed to cd to 0-core");

    for pkg in packages {
        let pkg_path = Path::new(pkg);
        if pkg_path.exists() {
            print!("  Stowing {}... ", pkg);
            
            if run_cmd("stow", &["--ignore=\\.dotmeta", pkg]) {
                println!("{}", "OK".green());
            } else {
                println!("{}", "SKIP".yellow());
            }
        } else {
            println!("  {} {} (not found)", "⚠️".yellow(), pkg);
        }
    }

    println!("{} Packages stowed", "✅".green());
}

fn build_rust_tools(core_path: &str) {
    let tools = ["dot-doctor", "dotctl", "core-diff", "bump-system-version"];
    let scripts_dir = format!("{}/scripts", core_path);

    for tool in tools {
        let tool_path = format!("{}/rust-tools/{}", core_path, tool);
        if Path::new(&tool_path).exists() {
            print!("  Building {}... ", tool);
            
            env::set_current_dir(&tool_path).ok();
            
            if run_cmd("cargo", &["build", "--release"]) {
                // Copy to scripts
                let src = format!("{}/target/release/{}", tool_path, tool);
                let dst = format!("{}/{}", scripts_dir, tool);
                fs::copy(&src, &dst).ok();
                println!("{}", "OK".green());
            } else {
                println!("{}", "FAIL".red());
            }
        }
    }

    // Make scripts executable
    run_cmd("chmod", &["+x", &format!("{}/*", scripts_dir)]);

    println!("{} Rust tools built", "✅".green());
}

fn run_health_check(core_path: &str) {
    let doctor = format!("{}/scripts/dot-doctor", core_path);
    
    if Path::new(&doctor).exists() {
        println!("Running dot-doctor...");
        println!();
        run_cmd(&doctor, &[]);
    } else {
        println!("{} dot-doctor not found, skipping health check", "⚠️".yellow());
    }
}
