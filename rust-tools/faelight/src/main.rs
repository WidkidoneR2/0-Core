//! faelight - Unified CLI for Faelight Forest
//! 🌲 The Core Spine
//!
//! Usage: faelight <command> [args]

use clap::{Parser, Subcommand};
use colored::*;
mod config;
use std::process::{Command, exit};

#[derive(Parser)]
#[command(name = "faelight")]
#[command(about = "🌲 Faelight Forest - Unified CLI", long_about = None)]
#[command(version)]
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
    /// System health check
    Health {
        /// Show detailed explanations
        #[arg(long)]
        explain: bool,
        
        /// Fail on warnings (for CI)
        #[arg(long)]
        fail_on_warning: bool,
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
    
    /// Sway window manager
    Sway {
        #[command(subcommand)]
        action: SwayAction,
    },
    
    /// Launch applications
    Launch {
        #[command(subcommand)]
        app: LaunchApp,
    },
    
    /// Git governance
    Git {
        #[command(subcommand)]
        action: GitAction,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Show system info
    Status,
    
    /// Explain a concept
    Explain {
        /// Topic to explain
        topic: String,
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
    /// List all intents
    List {
        /// Filter by category
        category: Option<String>,
    },
    /// Show specific intent
    Show { id: String },
    /// Search intents
    Search { term: String },
}

#[derive(Subcommand)]
enum CoreAction {
    /// Lock core (protect from changes)
    Lock,
    /// Unlock core
    Unlock,
    /// Show lock status
    Status,
}

#[derive(Subcommand)]
enum SwayAction {
    /// Reload sway config
    Reload,
    /// Show sway status
    Status,
}

#[derive(Subcommand)]
enum LaunchApp {
    /// App launcher
    Launcher,
    /// Power menu
    Menu,
    /// Lock screen
    Lock,
    /// Notifications test
    Notify { message: String },
}

#[derive(Subcommand)]
enum GitAction {
    /// Install git hooks
    InstallHooks,
    /// Remove git hooks
    RemoveHooks,
    /// Verify commit readiness
    Verify,
    Status,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Validate all config files
    Validate,
    /// Show current configuration
    Show,
    /// Show config file path
    Path,
    /// Edit config in editor
    Edit,
}

fn main() {
    let cli = Cli::parse();
    
    let exit_code = match cli.command {
        Commands::Health { explain, fail_on_warning } => {
            cmd_health(explain, fail_on_warning, cli.json)
        }
        Commands::Profile { action } => cmd_profile(action, cli.dry_run),
        Commands::Intent { action } => cmd_intent(action),
        Commands::Core { action } => cmd_core(action, cli.dry_run),
        Commands::Sway { action } => cmd_sway(action, cli.dry_run),
        Commands::Launch { app } => cmd_launch(app),
        Commands::Git { action } => cmd_git(action),
        Commands::Config { action } => cmd_config(action),
        Commands::Status => cmd_status(cli.json),
        Commands::Explain { topic } => cmd_explain(&topic),
    };
    
    exit(exit_code);
}

// ═══════════════════════════════════════════════════════════
// 🏥 HEALTH
// ═══════════════════════════════════════════════════════════
fn cmd_health(explain: bool, _fail_on_warning: bool, json: bool) -> i32 {
    let mut args = vec![];
    if explain { args.push("--explain"); }
    if json { args.push("--json"); }
    
    // Delegate to dot-doctor for now
    let status = Command::new("dot-doctor")
        .args(&args)
        .status();
    
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(_) => {
            eprintln!("{} dot-doctor not found", "Error:".red());
            1
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🎭 PROFILE
// ═══════════════════════════════════════════════════════════
fn cmd_profile(action: ProfileAction, dry_run: bool) -> i32 {
    match action {
        ProfileAction::List => {
            exec_script("profile", &["list"])
        }
        ProfileAction::Switch { name } => {
            if dry_run {
                println!("{} Would switch to profile: {}", "DRY-RUN:".yellow(), name);
                return 0;
            }
            exec_script("profile", &[&name])
        }
        ProfileAction::Current => {
            exec_script("profile", &["current"])
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 📜 INTENT
// ═══════════════════════════════════════════════════════════
fn cmd_intent(action: IntentAction) -> i32 {
    match action {
        IntentAction::List { category } => {
            match category {
                Some(cat) => exec_script("intent", &["list", &cat]),
                None => exec_script("intent", &["list"]),
            }
        }
        IntentAction::Show { id } => {
            exec_script("intent", &["show", &id])
        }
        IntentAction::Search { term } => {
            exec_script("intent", &["search", &term])
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🔒 CORE
// ═══════════════════════════════════════════════════════════
fn cmd_core(action: CoreAction, dry_run: bool) -> i32 {
    match action {
        CoreAction::Lock => {
            if dry_run {
                println!("{} Would lock core", "DRY-RUN:".yellow());
                return 0;
            }
            exec_script("lock-core", &[])
        }
        CoreAction::Unlock => {
            if dry_run {
                println!("{} Would unlock core", "DRY-RUN:".yellow());
                return 0;
            }
            exec_script("unlock-core", &[])
        }
        CoreAction::Status => {
            // Check if locked
            let home = std::env::var("HOME").unwrap_or_default();
            let lock_file = format!("{}/.0-core-locked", home);
            if std::path::Path::new(&lock_file).exists() {
                println!("🔒 Core is {}", "LOCKED".red());
            } else {
                println!("🔓 Core is {}", "UNLOCKED".green());
            }
            0
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🪟 SWAY
// ═══════════════════════════════════════════════════════════
fn cmd_sway(action: SwayAction, dry_run: bool) -> i32 {
    match action {
        SwayAction::Reload => {
            if dry_run {
                println!("{} Would reload sway", "DRY-RUN:".yellow());
                return 0;
            }
            let status = Command::new("swaymsg").arg("reload").status();
            match status {
                Ok(s) if s.success() => {
                    println!("✅ Sway configuration reloaded");
                    0
                }
                _ => {
                    eprintln!("{} Failed to reload sway", "Error:".red());
                    1
                }
            }
        }
        SwayAction::Status => {
            let _ = Command::new("swaymsg").args(["-t", "get_version"]).status();
            0
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🚀 LAUNCH
// ═══════════════════════════════════════════════════════════
fn cmd_launch(app: LaunchApp) -> i32 {
    match app {
        LaunchApp::Launcher => exec_faelight_tool("faelight-launcher"),
        LaunchApp::Menu => exec_faelight_tool("faelight-menu"),
        LaunchApp::Lock => exec_faelight_tool("faelight-lock"),
        LaunchApp::Notify { message } => {
            let status = Command::new("notify-send")
                .args(["Faelight", &message])
                .status();
            match status {
                Ok(s) => s.code().unwrap_or(1),
                Err(_) => 1,
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 📊 STATUS
// ═══════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════
// ⚙️ CONFIG
// ═══════════════════════════════════════════════════════════
// 🔧 GIT
// ═══════════════════════════════════════════════════════════
fn cmd_git(action: GitAction) -> i32 {
    let cmd = match action {
        GitAction::InstallHooks => "install-hooks",
        GitAction::RemoveHooks => "remove-hooks",
        GitAction::Verify => "verify",
        GitAction::Status => "status",
    };
    exec_script("faelight-git", &[cmd])
}

// ═══════════════════════════════════════════════════════════
fn cmd_config(action: ConfigAction) -> i32 {
    use colored::*;
    match action {
        ConfigAction::Validate => {
            println!("{}", "⚙️ Validating configuration files...".cyan());
            println!();
            let results = config::validate_all();
            let mut all_ok = true;
            for (file, result) in results {
                match result {
                    Ok(()) => println!("  {} {}", "✅".green(), file),
                    Err(e) => {
                        println!("  {} {} - {}", "❌".red(), file, e);
                        all_ok = false;
                    }
                }
            }
            println!();
            if all_ok {
                println!("{}", "All configuration files valid! 🌲".green());
                0
            } else {
                println!("{}", "Some configuration files have errors.".red());
                1
            }
        }
        ConfigAction::Show => {
            match config::load_config() {
                Ok(cfg) => {
                    println!("{}", "⚙️ Current Configuration".cyan().bold());
                    println!();
                    println!("  Theme:    {}", cfg.system.theme);
                    println!("  Profile:  {}", cfg.system.default_profile);
                    println!("  Version:  {}", cfg.system.version);
                    println!();
                    println!("  Bar refresh:    {}ms", cfg.bar.refresh_ms);
                    println!("  Notify timeout: {}ms", cfg.notifications.timeout_ms);
                    println!("  Lock timeout:   {}min", cfg.lock.timeout_minutes);
                    0
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    1
                }
            }
        }
        ConfigAction::Path => {
            println!("{}", config::config_dir().display());
            0
        }
        ConfigAction::Edit => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
            let path = config::config_dir().join("config.toml");
            std::process::Command::new(editor).arg(path).status().ok();
            0
        }
    }
}

fn cmd_status(json: bool) -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let version = std::fs::read_to_string(format!("{}/0-core/VERSION", home))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    let locked = std::path::Path::new(&format!("{}/.0-core-locked", home)).exists();
    
    let profile = std::fs::read_to_string(format!("{}/.local/state/faelight/current-profile", home))
        .unwrap_or_else(|_| "default".to_string())
        .trim()
        .to_string();
    
    if json {
        println!(r#"{{"version":"{}","locked":{},"profile":"{}"}}"#, version, locked, profile);
    } else {
        println!("{}", "═".repeat(50).green());
        println!("{}", "🌲 Faelight Forest Status".green().bold());
        println!("{}", "═".repeat(50).green());
        println!("  Version:  {}", format!("v{}", version).cyan());
        println!("  Profile:  {}", profile.cyan());
        println!("  Core:     {}", if locked { "🔒 Locked".red().to_string() } else { "🔓 Unlocked".green().to_string() });
        println!("{}", "═".repeat(50).green());
    }
    0
}

// ═══════════════════════════════════════════════════════════
// 📖 EXPLAIN
// ═══════════════════════════════════════════════════════════
fn cmd_explain(topic: &str) -> i32 {
    match topic.to_lowercase().as_str() {
        "profile" | "profiles" => {
            println!("{}", "🎭 Profiles".green().bold());
            println!("Profiles control system behavior for different contexts.");
            println!();
            println!("Available profiles:");
            println!("  {} - Normal daily use", "default".cyan());
            println!("  {} - VPN on, focused notifications", "work".cyan());
            println!("  {} - VPN off, performance mode", "gaming".cyan());
            println!("  {} - Minimal distractions", "focus".cyan());
            println!();
            println!("Usage: {} switch <profile>", "faelight profile".yellow());
        }
        "intent" | "intents" => {
            println!("{}", "📜 Intent Ledger".green().bold());
            println!("Every major decision is documented with rationale.");
            println!();
            println!("Categories:");
            println!("  {} - Major architectural choices", "decisions".cyan());
            println!("  {} - Things we tried", "experiments".cyan());
            println!("  {} - Core beliefs", "philosophy".cyan());
            println!("  {} - Planned features", "future".cyan());
            println!();
            println!("Usage: {} list [category]", "faelight intent".yellow());
        }
        "health" => {
            println!("{}", "🏥 Health System".green().bold());
            println!("Validates system integrity across multiple dimensions.");
            println!();
            println!("Checks:");
            println!("  - Stow symlinks");
            println!("  - Running services");
            println!("  - Git repository state");
            println!("  - Theme packages");
            println!("  - Intent ledger");
            println!();
            println!("Usage: {} [--explain]", "faelight health".yellow());
        }
        "core" => {
            println!("{}", "🔒 Core Protection".green().bold());
            println!("Prevents accidental changes to 0-core repository.");
            println!();
            println!("When locked:");
            println!("  - Git commits blocked");
            println!("  - Changes warned");
            println!();
            println!("Usage:");
            println!("  {} lock", "faelight core".yellow());
            println!("  {} unlock", "faelight core".yellow());
        }
        _ => {
            println!("{} Unknown topic: {}", "Error:".red(), topic);
            println!();
            println!("Available topics:");
            println!("  profile, intent, health, core");
            return 1;
        }
    }
    0
}

// ═══════════════════════════════════════════════════════════
// 🔧 HELPERS
// ═══════════════════════════════════════════════════════════
fn exec_script(name: &str, args: &[&str]) -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let script_path = format!("{}/0-core/scripts/{}", home, name);
    
    let status = Command::new(&script_path).args(args).status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(_) => {
            eprintln!("{} Script not found: {}", "Error:".red(), name);
            1
        }
    }
}

fn exec_faelight_tool(name: &str) -> i32 {
    let home = std::env::var("HOME").unwrap_or_default();
    let tool_path = format!("{}/0-core/scripts/{}", home, name);
    
    let status = Command::new(&tool_path).spawn();
    match status {
        Ok(_) => 0,
        Err(_) => {
            eprintln!("{} Tool not found: {}", "Error:".red(), name);
            1
        }
    }
}
