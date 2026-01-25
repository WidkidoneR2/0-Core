mod cargo_checker;
mod neovim_checker;
mod tui;

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
    
    /// Show detailed version information for each update
    #[arg(short, long)]
    verbose: bool,
    
    /// Output results in JSON format
    #[arg(long)]
    json: bool,
    
    /// Only check specific categories (comma-separated: pacman,aur,cargo,neovim,workspace)
    #[arg(long, value_delimiter = ',')]
    only: Option<Vec<String>>,
    
    /// Skip specific categories (comma-separated)
    #[arg(long, value_delimiter = ',')]
    skip: Option<Vec<String>>,
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
    if !cli.json {
        println!(
            "{} v{}", 
            "🌲 Faelight Update Manager".green().bold(),
            env!("CARGO_PKG_VERSION").cyan()
        );
        println!();
    }
    
    // Health check
    if !cli.skip_health && !cli.json {
        println!("{}  Running health check...", "🏥".green());
        run_health_check()?;
    }
    
    // Create pre-update snapshot if requested
    if cli.snapshot && !cli.dry_run && !cli.json {
        create_snapshot()?;
    }
    
    // Check for updates
    if !cli.json {
        println!("{}  Checking for updates...", "🔍".cyan());
    }
    
    let mut updates = check_all_updates()?;
    
    // Filter categories based on --only and --skip
    if let Some(ref only) = cli.only {
        updates.retain(|cat| only.iter().any(|o| category_matches(o, &cat.name)));
    }
    
    if let Some(ref skip) = cli.skip {
        updates.retain(|cat| !skip.iter().any(|s| category_matches(s, &cat.name)));
    }
    
    let total: usize = updates.iter().map(|c| c.count).sum();
    
    // JSON output
    if cli.json {
        output_json(&updates, total)?;
        return Ok(());
    }
    
    // Show summary
    show_update_summary(&updates, cli.verbose);
    
    if total == 0 {
        println!("\n{}  All packages up to date!", "✨".green());
        return Ok(());
    }
    
    // Show impact analysis
    let impact = analyze_impact(&updates);
    if impact.has_impact() {
        println!("{}", impact);
    }
    
    // Interactive mode
    if cli.interactive {
        let selections = tui::interactive_select(&updates);
        
        if selections.is_empty() {
            println!("\n{}  No packages selected", "ℹ️".blue());
            return Ok(());
        }
        
        if !cli.dry_run && tui::confirm_updates(&selections) {
            perform_updates(&selections)?;
        } else {
            println!("\n{}  Cancelled", "ℹ️".blue());
        }
    } else if !cli.dry_run {
        println!("\n{}  Ready to update {} packages!", "✨".yellow(), total);
        println!("{}  Run with --interactive to select packages", "💡".blue());
        println!("{}  Run with --dry-run to preview changes", "💡".blue());
    } else {
        println!("\n{}  Dry run complete - {} updates available", "ℹ️".blue(), total);
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
        anyhow::bail!("Snapshot failed: {}", stderr);
    }
    
    Ok(())
}

/// Run health check
fn run_health_check() -> Result<()> {
    let output = Command::new("dot-doctor")
        .output()
        .context("Failed to run dot-doctor - is it installed?")?;
    
    if !output.status.success() {
        println!("{}  Health check failed!", "⚠️".yellow());
        anyhow::bail!("System health check did not pass");
    }
    
    println!("   {}  System healthy", "✅".green());
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
    let nvim_items = neovim_checker::check_neovim_updates();
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
    
    Ok(categories)
}

/// Check for pacman updates
fn check_pacman_updates() -> Result<UpdateCategory> {
    println!("   Checking pacman...");
    
    let output = Command::new("checkupdates")
        .output()
        .context("Failed to run checkupdates")?;
    
    let items = if output.status.code() == Some(2) {
        // Exit code 2 means no updates (not an error)
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
    static PACMAN_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(?:([^/]+)/)?(\S+)\s+(\S+)\s+->\s+(\S+)").unwrap()
    });
    
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
fn show_update_summary(categories: &[UpdateCategory], verbose: bool) {
    println!();
    println!("{}", "📊 Update Summary".cyan().bold());
    println!("{}", "─".repeat(50).cyan());
    
    let total: usize = categories.iter().map(|c| c.count).sum();
    
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
            writeln!(f, "  {} Kernel update - reboot required after update", "⚠️".yellow())?;
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
        "systemd", "glibc", "gcc", "binutils", "filesystem",
        "pacman", "linux-firmware", "mesa"
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
        .current_dir("/home/christian/0-core")
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
    println!("   Running: sudo pacman -S --noconfirm {}", items.join(" "));
    
    let status = Command::new("sudo")
        .arg("pacman")
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
    println!("   Running: paru -S --noconfirm {}", items.join(" "));
    
    let status = Command::new("paru")
        .arg("-S")
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
    println!("   Running: cargo install-update {}", items.join(" "));
    
    let status = Command::new("cargo")
        .arg("install-update")
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
