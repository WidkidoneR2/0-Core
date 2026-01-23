use clap::Parser;
use colored::*;
use std::process::Command;

#[derive(Parser)]
#[command(name = "faelight-update")]
#[command(about = "🌲 Intelligent update manager for Faelight Forest")]
struct Cli {
    /// Show what would update without updating
    #[arg(long)]
    dry_run: bool,
    
    /// Skip health check
    #[arg(long)]
    skip_health: bool,
}

fn main() {
    let cli = Cli::parse();
    
    println!("{}", "🌲 Faelight Update Manager v0.1.0".green().bold());
    println!();
    
    // Phase 1: Health check
    if !cli.skip_health {
        println!("{}  Running health check...", "🏥".green());
        run_health_check();
    }
    
    // Phase 2: Check for updates
    println!("{}  Checking for updates...", "🔍".cyan());
    let updates = check_all_updates();
    
    // Phase 3: Show summary
    show_update_summary(&updates);
    
    // Phase 4: Update (if not dry run)
    if !cli.dry_run {
        println!("\n{}  Ready to update!", "✨".yellow());
        // TODO: Interactive selection
    } else {
        println!("\n{}  Dry run complete - no changes made", "ℹ️".blue());
    }
}

fn run_health_check() {
    let output = Command::new("dot-doctor")
        .output()
        .expect("Failed to run doctor");
    
    if !output.status.success() {
        println!("{}  Health check failed! Fix issues before updating.", "⚠️".yellow());
        std::process::exit(1);
    }
    println!("   {}  System healthy", "✅".green());
}

struct UpdateCategory {
    name: String,
    count: usize,
    items: Vec<UpdateItem>,
}

struct UpdateItem {
    name: String,
    current: String,
    new: String,
}

fn check_all_updates() -> Vec<UpdateCategory> {
    let mut categories = Vec::new();
    
    // System packages (pacman)
    categories.push(check_pacman_updates());
    
    // AUR packages (paru)
    categories.push(check_paru_updates());
    
    // Cargo tools
    categories.push(check_cargo_updates());
    
    // Neovim plugins
    categories.push(check_neovim_updates());
    
    // 0-Core workspace
    categories.push(check_workspace_updates());
    
    categories
}

fn check_pacman_updates() -> UpdateCategory {
    println!("   Checking pacman...");
    
    let output = Command::new("checkupdates")
        .output()
        .unwrap_or_else(|_| {
            // checkupdates might not exist, use pacman -Qu
            Command::new("pacman")
                .args(["-Qu"])
                .output()
                .expect("Failed to check pacman updates")
        });
    
    let items = parse_pacman_output(&output.stdout);
    
    UpdateCategory {
        name: "System Packages".to_string(),
        count: items.len(),
        items,
    }
}

fn parse_pacman_output(output: &[u8]) -> Vec<UpdateItem> {
    let text = String::from_utf8_lossy(output);
    text.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                Some(UpdateItem {
                    name: parts[0].to_string(),
                    current: parts[1].to_string(),
                    new: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn check_paru_updates() -> UpdateCategory {
    println!("   Checking AUR (paru)...");
    
    let output = Command::new("paru")
        .args(["-Qu", "--aur"])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });
    
    let items = parse_pacman_output(&output.stdout);
    
    UpdateCategory {
        name: "AUR Packages".to_string(),
        count: items.len(),
        items,
    }
}

fn check_cargo_updates() -> UpdateCategory {
    println!("   Checking cargo tools...");
    
    // TODO: Use cargo-update crate
    UpdateCategory {
        name: "Cargo Tools".to_string(),
        count: 0,
        items: Vec::new(),
    }
}

fn check_neovim_updates() -> UpdateCategory {
    println!("   Checking neovim plugins...");
    
    // TODO: Check lazy.nvim updates
    UpdateCategory {
        name: "Neovim Plugins".to_string(),
        count: 0,
        items: Vec::new(),
    }
}

fn check_workspace_updates() -> UpdateCategory {
    println!("   Checking 0-Core workspace...");
    
    // TODO: Check if workspace needs rebuild
    UpdateCategory {
        name: "0-Core Tools".to_string(),
        count: 0,
        items: Vec::new(),
    }
}

fn show_update_summary(categories: &[UpdateCategory]) {
    println!();
    println!("{}", "📊 Update Summary".cyan().bold());
    println!("{}", "─".repeat(50).cyan());
    
    let total: usize = categories.iter().map(|c| c.count).sum();
    
    for category in categories {
        if category.count > 0 {
            println!(
                "  {} {} ({})",
                "📦".yellow(),
                category.name.bold(),
                format!("{} available", category.count).yellow()
            );
            
            for item in &category.items {
                println!(
                    "     {} {} → {}",
                    item.name.white(),
                    item.current.dimmed(),
                    item.new.green()
                );
            }
        } else {
            println!(
                "  {} {} {}",
                "✅".green(),
                category.name,
                "(up to date)".dimmed()
            );
        }
    }
    
    println!("{}", "─".repeat(50).cyan());
    println!("  {} updates available", total.to_string().yellow().bold());
}
