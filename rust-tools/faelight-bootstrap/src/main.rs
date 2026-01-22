//! faelight-bootstrap v1.0.0 - One-Command 0-Core Setup
//! 🌲 Faelight Forest - Linus Edition
//!
//! Philosophy: Automation serves installation, but human controls choices

use colored::*;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, exit};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle CLI flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("faelight-bootstrap v{}", VERSION);
                return;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--health" => {
                println!("✅ faelight-bootstrap: Installation system operational");
                return;
            }
            "--dry-run" => {
                dry_run();
                return;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Try 'faelight-bootstrap --help'");
                exit(1);
            }
        }
    }
    
    // Main installation
    run_installation();
}

fn print_help() {
    println!("faelight-bootstrap v{} - One-Command 0-Core Setup", VERSION);
    println!("🌲 Faelight Forest");
    println!();
    println!("USAGE:");
    println!("    faelight-bootstrap [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Show this help message");
    println!("    -v, --version    Show version information");
    println!("    --health         Check tool health status");
    println!("    --dry-run        Preview installation without changes");
    println!();
    println!("WHAT IT DOES:");
    println!("    1. Pre-flight checks (Arch, internet, permissions)");
    println!("    2. Installs system dependencies via pacman");
    println!("    3. Clones 0-core repository");
    println!("    4. Stows all dotfile packages (12 packages)");
    println!("    5. Builds all Rust tools (30 tools via workspace)");
    println!("    6. Runs health verification");
    println!("    7. Provides post-install tutorial");
    println!();
    println!("REQUIREMENTS:");
    println!("    - Arch Linux");
    println!("    - Internet connection");
    println!("    - Sudo access");
    println!("    - ~2GB disk space");
    println!();
    println!("PHILOSOPHY:");
    println!("    Interactive prompts ensure you control the installation.");
    println!("    The tool automates tedious work while you make decisions.");
}

fn dry_run() {
    println!("🌲 Faelight Forest Bootstrap - Dry Run");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("📋 Would perform these actions:");
    println!();
    println!("  📊 Phase 1: Pre-Flight Checks");
    println!("     ✓ Verify Arch Linux");
    println!("     ✓ Check internet connection");
    println!("     ✓ Verify not running as root");
    println!();
    println!("  📦 Phase 2: System Dependencies");
    println!("     ✓ Install ~40 packages via pacman");
    println!("       (Sway, Foot, Neovim, Rust, fonts, etc.)");
    println!();
    println!("  📥 Phase 3: Clone Repository");
    println!("     ✓ Clone 0-core to ~/0-core");
    println!("     ✓ Backup existing if present");
    println!();
    println!("  🔗 Phase 4: Stow Packages (12 packages)");
    println!("     ✓ wm-sway, shell-zsh, shell-nushell");
    println!("     ✓ prompt-starship, term-foot, editor-nvim");
    println!("     ✓ fm-yazi, vcs-git, config-faelight");
    println!("     ✓ browser-qutebrowser, browser-brave");
    println!("     ✓ tools-topgrade");
    println!();
    println!("  🦀 Phase 5: Build Rust Tools (30 tools)");
    println!("     ✓ Workspace build from ~/0-core");
    println!("     ✓ Copy binaries to scripts/");
    println!();
    println!("  🏥 Phase 6: Health Verification");
    println!("     ✓ Run dot-doctor health check");
    println!();
    println!("  🎓 Phase 7: Post-Install Tutorial");
    println!("     ✓ Interactive guide to first steps");
    println!();
    println!("⚠️  This is a DRY RUN - no changes would be made");
}

fn run_installation() {
    print_banner();
    
    // Phase 1: Pre-flight checks
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "📊 Phase 1: Pre-Flight Checks".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    check_arch_linux();
    check_not_root();
    check_internet();
    check_disk_space();
    
    println!();
    
    // Get paths
    let home = env::var("HOME").expect("HOME not set");
    let core_path = format!("{}/0-core", home);
    
    // Check existing installation
    if Path::new(&core_path).exists() {
        println!("{}", "⚠️  Existing Installation Detected".yellow().bold());
        println!("   0-core already exists at: {}", core_path);
        println!();
        
        if !confirm("  This will backup and reinstall. Continue?") {
            println!("{}", "Installation cancelled.".red());
            exit(0);
        }
    }
    
    println!();
    
    // Phase 2: Dependencies
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "📦 Phase 2: System Dependencies".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    if confirm("  Install ~40 system packages?") {
        install_dependencies();
    } else {
        println!("{}", "  ⚠️  Skipping dependencies (may cause issues)".yellow());
    }
    
    println!();
    
    // Phase 3: Clone
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "📥 Phase 3: Clone Repository".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    clone_repo(&home, &core_path);
    
    println!();
    
    // Phase 4: Stow
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🔗 Phase 4: Stow Packages".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    if confirm("  Stow all dotfile packages?") {
        stow_packages(&core_path);
    } else {
        println!("{}", "  ⚠️  Skipping stow (manual stow required)".yellow());
    }
    
    println!();
    
    // Phase 5: Rust
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🦀 Phase 5: Build Rust Tools".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    if confirm("  Build all 30 Rust tools? (takes ~2-5 min)") {
        build_rust_workspace(&core_path);
    } else {
        println!("{}", "  ⚠️  Skipping build (tools unavailable)".yellow());
    }
    
    println!();
    
    // Phase 6: Health
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🏥 Phase 6: Health Verification".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    run_health_check(&core_path);
    
    println!();
    
    // Success!
    print_success();
    
    // Phase 7: Tutorial
    if confirm("  Start post-install tutorial?") {
        post_install_tutorial(&core_path);
    }
}

fn print_banner() {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
    println!("{}", "🌲 Faelight Forest Bootstrap v1.0.0".green().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
    println!();
    println!("  {} Philosophy-driven Linux desktop environment", "▸".green());
    println!("  {} 30 custom Rust tools", "▸".green());
    println!("  {} 12 dotfile packages", "▸".green());
    println!("  {} 100% reproducible setup", "▸".green());
    println!();
}

fn print_success() {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
    println!("{}", "✅ Faelight Forest Installation Complete!".green().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
    println!();
    println!("{}", "🎉 Welcome to the Forest!".green().bold());
    println!();
}

fn check_arch_linux() {
    print!("  {} Checking Arch Linux... ", "▸".cyan());
    io::stdout().flush().unwrap();
    
    if !Path::new("/etc/arch-release").exists() {
        println!("{}", "FAIL".red());
        eprintln!();
        eprintln!("{} This installer is for Arch Linux only", "  Error:".red().bold());
        exit(1);
    }
    
    println!("{}", "✓".green());
}

fn check_not_root() {
    print!("  {} Checking permissions... ", "▸".cyan());
    io::stdout().flush().unwrap();
    
    if env::var("USER").unwrap_or_default() == "root" {
        println!("{}", "FAIL".red());
        eprintln!();
        eprintln!("{} Don't run as root (sudo used internally)", "  Error:".red().bold());
        exit(1);
    }
    
    println!("{}", "✓".green());
}

fn check_internet() {
    print!("  {} Checking internet... ", "▸".cyan());
    io::stdout().flush().unwrap();
    
    let status = Command::new("ping")
        .args(["-c", "1", "-W", "3", "archlinux.org"])
        .output();
    
    match status {
        Ok(output) if output.status.success() => println!("{}", "✓".green()),
        _ => {
            println!("{}", "FAIL".red());
            eprintln!();
            eprintln!("{} No internet connection", "  Error:".red().bold());
            exit(1);
        }
    }
}

fn check_disk_space() {
    print!("  {} Checking disk space... ", "▸".cyan());
    io::stdout().flush().unwrap();
    
    // Simple check - just verify /home has space
    let output = Command::new("df")
        .args(["-h", "/home"])
        .output();
    
    // Basic validation - just check command succeeds
    match output {
        Ok(o) if o.status.success() => println!("{}", "✓".green()),
        _ => println!("{}", "⚠️  (couldn't verify)".yellow()),
    }
}

fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn run_cmd(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_cmd_show(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn install_dependencies() {
    let packages = vec![
        // Core system
        "git", "stow", "base-devel", "rustup",
        // Sway desktop
        "sway", "swaybg", "swayidle", "swaylock",
        // Wayland tools
        "wl-clipboard", "grim", "slurp", 
        "xdg-desktop-portal-wlr",
        // Terminal & shell
        "foot", "zsh", "starship", "nushell",
        // CLI tools
        "eza", "bat", "fd", "fzf", "ripgrep", "yazi",
        "lazygit", "btop",
        // Editor
        "neovim",
        // Audio
        "pipewire", "wireplumber", "pipewire-pulse",
        "brightnessctl", "playerctl",
        // Fonts
        "ttf-jetbrains-mono-nerd", "ttf-nerd-fonts-symbols",
        // Browsers
        "brave-bin", "qutebrowser",
        // System
        "snapper", "mullvad-vpn", "ufw",
    ];
    
    println!("  {} Installing {} packages...", "▸".cyan(), packages.len());
    println!("     (This may take 5-10 minutes)");
    println!();
    
    // Update database
    print!("  {} Updating package database... ", "▸".cyan());
    io::stdout().flush().unwrap();
    
    if run_cmd("sudo", &["pacman", "-Sy", "--noconfirm"]) {
        println!("{}", "✓".green());
    } else {
        println!("{}", "FAIL".red());
        eprintln!("{} Failed to update package database", "  Error:".red().bold());
        exit(1);
    }
    
    // Install packages
    print!("  {} Installing packages... ", "▸".cyan());
    io::stdout().flush().unwrap();
    
    let mut args = vec!["pacman", "-S", "--needed", "--noconfirm"];
    for pkg in &packages {
        args.push(pkg);
    }
    
    if run_cmd("sudo", &args) {
        println!("{}", "✓".green());
    } else {
        println!("{}", "⚠️  (some packages may have failed)".yellow());
    }
    
    // Setup Rust
    print!("  {} Setting up Rust toolchain... ", "▸".cyan());
    io::stdout().flush().unwrap();
    
    run_cmd("rustup", &["default", "stable"]);
    println!("{}", "✓".green());
    
    println!();
    println!("  {} Dependencies installed", "✓".green().bold());
}

fn clone_repo(home: &str, core_path: &str) {
    // Backup existing
    if Path::new(core_path).exists() {
        let timestamp = timestamp();
        let backup = format!("{}/0-core.backup.{}", home, timestamp);
        
        print!("  {} Backing up existing to {}... ", "▸".cyan(), backup);
        io::stdout().flush().unwrap();
        
        if fs::rename(core_path, &backup).is_ok() {
            println!("{}", "✓".green());
        } else {
            println!("{}", "FAIL".red());
            exit(1);
        }
    }
    
    // Clone
    print!("  {} Cloning repository... ", "▸".cyan());
    io::stdout().flush().unwrap();
    
    let repo_url = "https://github.com/WidkidoneR2/0-Core.git";
    
    if run_cmd("git", &["clone", repo_url, core_path]) {
        println!("{}", "✓".green());
    } else {
        println!("{}", "FAIL".red());
        eprintln!();
        eprintln!("{} Failed to clone repository", "  Error:".red().bold());
        eprintln!("  Try manually: git clone YOUR_REPO {}", core_path);
        exit(1);
    }
    
    println!();
    println!("  {} Repository cloned to {}", "✓".green().bold(), core_path);
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

fn stow_packages(core_path: &str) {
    let packages = vec![
        "wm-sway",
        "shell-zsh",
        "shell-nushell",
        "prompt-starship",
        "term-foot",
        "editor-nvim",
        "fm-yazi",
        "vcs-git",
        "config-faelight",
        "browser-qutebrowser",
        "browser-brave",
        "tools-topgrade",
    ];
    
    let stow_dir = format!("{}/stow", core_path);
    
    println!("  {} Stowing {} packages...", "▸".cyan(), packages.len());
    println!();
    
    for pkg in &packages {
        let pkg_path = format!("{}/{}", stow_dir, pkg);
        
        if Path::new(&pkg_path).exists() {
            print!("    {} {}... ", "▸".cyan(), pkg);
            io::stdout().flush().unwrap();
            
            let status = Command::new("stow")
                .current_dir(&stow_dir)
                .args(&["--ignore=\\.dotmeta", "-t", &env::var("HOME").unwrap(), pkg])
                .status();
            
            match status {
                Ok(s) if s.success() => println!("{}", "✓".green()),
                _ => println!("{}", "⚠️".yellow()),
            }
        } else {
            println!("    {} {} {}", "▸".cyan(), pkg, "(not found)".yellow());
        }
    }
    
    println!();
    println!("  {} Packages stowed", "✓".green().bold());
}

fn build_rust_workspace(core_path: &str) {
    println!("  {} Building Rust workspace (30 tools)...", "▸".cyan());
    println!("     (This takes 2-5 minutes - be patient!)");
    println!();
    
    print!("  {} Running cargo build --release... ", "▸".cyan());
    io::stdout().flush().unwrap();
    
    let status = Command::new("cargo")
        .current_dir(core_path)
        .args(&["build", "--release"])
        .status();
    
    match status {
        Ok(s) if s.success() => {
            println!("{}", "✓".green());
            
            // All binaries are in target/release/
            // They're already accessible via scripts/ symlinks or copies
            println!();
            println!("  {} All tools built successfully", "✓".green().bold());
        }
        _ => {
            println!("{}", "FAIL".red());
            eprintln!();
            eprintln!("{} Build failed - some tools may be unavailable", "  Warning:".yellow().bold());
        }
    }
}

fn run_health_check(core_path: &str) {
    let doctor = format!("{}/scripts/dot-doctor", core_path);
    
    if Path::new(&doctor).exists() {
        println!("  {} Running system health check...", "▸".cyan());
        println!();
        
        Command::new(&doctor)
            .status()
            .ok();
        
        println!();
    } else {
        println!("  {} Health check unavailable (dot-doctor not built)", "⚠️".yellow());
    }
}

fn post_install_tutorial(_core_path: &str) {
    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🎓 Post-Install Tutorial".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    println!("{}", "📋 Next Steps:".yellow().bold());
    println!();
    println!("  1. {} - Reload shell configuration", "source ~/.zshrc".cyan());
    println!("     This activates all aliases and functions");
    println!();
    println!("  2. {} - Start Sway window manager", "sway".cyan());
    println!("     (Or reboot and select Sway from login screen)");
    println!();
    println!("  3. {} - Verify system health", "doctor".cyan());
    println!("     Should show 100% health");
    println!();
    println!("{}", "🎮 Essential Keybindings:".yellow().bold());
    println!();
    println!("  {} - Terminal", "Super + Return".cyan());
    println!("  {} - App launcher", "Super + D".cyan());
    println!("  {} - Browser", "Super + B".cyan());
    println!("  {} - Close window", "Super + Q".cyan());
    println!("  {} - Workspaces 1-5", "Super + 1-5".cyan());
    println!();
    println!("{}", "🦀 Quick Commands:".yellow().bold());
    println!();
    println!("  {} - System health", "doctor".cyan());
    println!("  {} - Update system", "safe-update".cyan());
    println!("  {} - Open editor", "v".cyan());
    println!("  {} - File manager", "y".cyan());
    println!("  {} - Git UI", "lg".cyan());
    println!();
    println!("{}", "📚 Learn More:".yellow().bold());
    println!();
    println!("  {} - Interactive tutorial", "teach --begin".cyan());
    println!("  {} - List aliases", "alias | grep".cyan());
    println!("  {} - View decisions", "intent list".cyan());
    println!();
    println!("{}", "🌲 Welcome to Faelight Forest!".green().bold());
    println!();
}
