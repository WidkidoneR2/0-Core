//! faelight v2.0.0 - Unified CLI for Faelight Forest
//! 🌲 The Core Spine - LEGENDARY EDITION
//!
//! The main entry point for all Faelight Forest operations.
//! Provides a unified interface that delegates to specialized tools.

use clap::{Parser, Subcommand};
use colored::*;
use faelight_core::paths;
use std::process::{exit, Command};

pub mod config;
use config::FaelightConfig;

#[derive(Parser)]
#[command(name = "faelight")]
#[command(about = "🌲 Faelight Forest - Unified CLI", long_about = None)]
#[command(version)]
#[command(after_help = "Examples:
  faelight health              # Run system health check
  faelight doctor              # Alias for health
  faelight profile switch dev  # Switch to dev profile
  faelight core lock           # Lock 0-core directory
  faelight --version           # Show CLI and ecosystem versions
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Dry run (show what would happen)
    #[arg(long, global = true)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// System health check (powered by dot-doctor)
    #[command(alias = "doctor")]
    Health {
        /// Show detailed explanations
        #[arg(long)]
        explain: bool,

        /// Fail on warnings (for CI)
        #[arg(long)]
        fail_on_warning: bool,

        /// Show all tool versions
        #[arg(long)]
        versions: bool,
    },

    /// Profile management
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Intent ledger
    Intent {
        #[command(subcommand)]
        action: IntentAction,
    },

    /// Core protection
    Core {
        #[command(subcommand)]
        action: CoreAction,
    },

    /// Launch applications
    Launch {
        #[command(subcommand)]
        app: LaunchApp,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ProfileAction {
    /// List available profiles
    List,
    /// Switch to a profile
    Switch { name: String },
    /// Show current profile
    Current,
}

#[derive(Subcommand)]
enum IntentAction {
    /// Create new intent
    New { title: String },
    /// List intents
    List,
    /// Show intent details
    Show { id: String },
}

#[derive(Subcommand)]
enum CoreAction {
    /// Lock 0-core directory (immutable)
    Lock,
    /// Unlock 0-core directory
    Unlock,
    /// Check core status
    Status,
}

#[derive(Subcommand)]
enum LaunchApp {
    /// File manager
    Fm,
    /// Terminal
    Term,
    /// Launcher
    Launcher,
    /// Menu
    Menu,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current config
    Show,
    /// Edit config file
    Edit,
    /// Reset to defaults
    Reset,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Health {
            explain,
            fail_on_warning,
            versions,
        } => {
            if versions {
                show_ecosystem_versions();
            } else {
                run_health_check(explain, fail_on_warning);
            }
        }
        Commands::Profile { action } => handle_profile(action),
        Commands::Intent { action } => handle_intent(action),
        Commands::Core { action } => handle_core(action),
        Commands::Launch { app } => handle_launch(app),
        Commands::Config { action } => handle_config(action),
    }
}

// ═══════════════════════════════════════════════════════════
// 🏥 HEALTH CHECK - Enhanced with better output
// ═══════════════════════════════════════════════════════════

fn run_health_check(explain: bool, fail_on_warning: bool) {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🏥 FAELIGHT FOREST HEALTH CHECK".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();

    let doctor_path = paths::scripts_dir().join("dot-doctor");

    if !check_tool_exists(&doctor_path) {
        eprintln!("{} dot-doctor not found!", "❌".red());
        eprintln!("   Expected at: {}", doctor_path.display());
        eprintln!("   Run: cargo build --release -p dot-doctor");
        exit(1);
    }

    let mut cmd = Command::new(&doctor_path);

    if explain {
        cmd.arg("--explain");
    }

    let status = cmd.status().expect("Failed to run dot-doctor");

    if !status.success() && fail_on_warning {
        exit(1);
    }
}

fn show_ecosystem_versions() {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🌲 FAELIGHT FOREST ECOSYSTEM".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();

    // Show system version
    if let Ok(version) = std::fs::read_to_string(paths::meta_dir().join("VERSION")) {
        println!("  {} {}", "System:".bold(), version.trim().green());
    }

    println!(
        "  {} {}",
        "faelight CLI:".bold(),
        env!("CARGO_PKG_VERSION").green()
    );
    println!();

    // Show key tool versions
    let tools = vec![
        ("dot-doctor", "scripts/dot-doctor"),
        ("faelight-fm", "target/release/faelight-fm"),
        ("faelight-bar", "target/release/faelight-bar"),
        ("faelight-term", "target/release/faelight-term"),
        ("faelight-hooks", "target/release/faelight-hooks"),
    ];

    println!("  {} Key Tools:", "📦".bold());
    for (name, bin_path) in tools {
        let full_path = paths::core_dir().join(bin_path);
        if full_path.exists() {
            println!("    ✅ {}", name.cyan());
        } else {
            println!("    ⚠️  {} (not built)", name.yellow());
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 📊 PROFILE MANAGEMENT
// ═══════════════════════════════════════════════════════════

fn handle_profile(action: ProfileAction) {
    let profile_cmd = find_tool("profile");

    match action {
        ProfileAction::List => {
            run_tool(&profile_cmd, &["list"]);
        }
        ProfileAction::Switch { name } => {
            run_tool(&profile_cmd, &["switch", &name]);
        }
        ProfileAction::Current => {
            run_tool(&profile_cmd, &["current"]);
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 📝 INTENT MANAGEMENT
// ═══════════════════════════════════════════════════════════

fn handle_intent(action: IntentAction) {
    let intent_cmd = find_tool("intent");

    match action {
        IntentAction::New { title } => {
            run_tool(&intent_cmd, &["new", &title]);
        }
        IntentAction::List => {
            run_tool(&intent_cmd, &["list"]);
        }
        IntentAction::Show { id } => {
            run_tool(&intent_cmd, &["show", &id]);
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🛡️ CORE PROTECTION
// ═══════════════════════════════════════════════════════════

fn handle_core(action: CoreAction) {
    let core_protect = find_tool("core-protect");

    match action {
        CoreAction::Lock => {
            run_tool(&core_protect, &["lock"]);
        }
        CoreAction::Unlock => {
            run_tool(&core_protect, &["unlock"]);
        }
        CoreAction::Status => {
            run_tool(&core_protect, &["status"]);
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🚀 LAUNCH APPLICATIONS
// ═══════════════════════════════════════════════════════════

fn handle_launch(app: LaunchApp) {
    match app {
        LaunchApp::Fm => {
            let fm = find_tool("faelight-fm");
            run_tool_bg(&fm, &[]);
        }
        LaunchApp::Term => {
            let term = find_tool("faelight-term");
            run_tool_bg(&term, &[]);
        }
        LaunchApp::Launcher => {
            let launcher = find_tool("faelight-launcher");
            run_tool_bg(&launcher, &[]);
        }
        LaunchApp::Menu => {
            let menu = find_tool("faelight-menu");
            run_tool_bg(&menu, &[]);
        }
    }
}

// ═══════════════════════════════════════════════════════════
// ⚙️ CONFIGURATION
// ═══════════════════════════════════════════════════════════

fn handle_config(action: ConfigAction) {
    match action {
        ConfigAction::Show => {
            let config = FaelightConfig::load();
            println!("{}", toml::to_string_pretty(&config).unwrap());
        }
        ConfigAction::Edit => {
            let config_path = FaelightConfig::config_path();
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

            Command::new(editor)
                .arg(&config_path)
                .status()
                .expect("Failed to open editor");
        }
        ConfigAction::Reset => {
            let config = FaelightConfig::default();
            config.save().expect("Failed to save config");
            println!("{} Config reset to defaults", "✅".green());
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🛠️ HELPER FUNCTIONS - Enhanced error handling
// ═══════════════════════════════════════════════════════════

fn find_tool(name: &str) -> String {
    // First check in target/release
    let release_path = paths::core_dir().join("target/release").join(name);
    if release_path.exists() {
        return release_path.display().to_string();
    }

    // Then check in scripts
    let scripts_path = paths::scripts_dir().join(name);
    if scripts_path.exists() {
        return scripts_path.display().to_string();
    }

    // Finally check PATH using which
    if let Ok(path) = which::which(name) {
        return path.display().to_string();
    }

    // Tool not found
    eprintln!("{} Tool '{}' not found!", "❌".red().bold(), name.yellow());
    eprintln!();
    eprintln!("Searched locations:");
    eprintln!("  • {}", release_path.display());
    eprintln!("  • {}", scripts_path.display());
    eprintln!("  • System PATH");
    eprintln!();
    eprintln!("To install:");
    eprintln!("  {}", format!("cargo build --release -p {}", name).cyan());
    exit(1);
}

fn check_tool_exists(path: &std::path::Path) -> bool {
    path.exists()
}

fn run_tool(tool: &str, args: &[&str]) {
    let status = Command::new(tool).args(args).status().unwrap_or_else(|e| {
        eprintln!("{} Failed to run {}: {}", "❌".red(), tool, e);
        exit(1);
    });

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}
#[allow(clippy::zombie_processes)] // Intentional: background launcher
fn run_tool_bg(tool: &str, args: &[&str]) {
    Command::new(tool).args(args).spawn().unwrap_or_else(|e| {
        eprintln!("{} Failed to launch {}: {}", "❌".red(), tool, e);
        exit(1);
    });

    println!(
        "{} Launched {}",
        "✅".green(),
        tool.split('/').next_back().unwrap()
    );
}
