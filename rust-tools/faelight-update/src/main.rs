mod cargo_checker;
mod neovim_checker;
mod tui;

use clap::Parser;
use colored::*;
use std::process::Command;

#[derive(Parser)]
#[command(name = "faelight-update")]
#[command(about = "🌲 Intelligent update manager for Faelight Forest", version = "0.2.0")]
struct Cli {
    #[arg(long)]
    dry_run: bool,
    
    #[arg(long)]
    skip_health: bool,
    
    #[arg(short, long)]
    interactive: bool,
}

fn main() {
    let cli = Cli::parse();
    
    println!("{}", "🌲 Faelight Update Manager v0.3.0".green().bold());
    println!();
    
    if !cli.skip_health {
        println!("{}  Running health check...", "🏥".green());
        run_health_check();
    }
    
    println!("{}  Checking for updates...", "🔍".cyan());
    let updates = check_all_updates();
    
    show_update_summary(&updates);
    
    let total: usize = updates.iter().map(|c| c.count).sum();
    
    if total == 0 {
        println!("\n{}  All packages up to date!", "✨".green());
        return;
    }
    
    if cli.interactive {
        let selections = tui::interactive_select(&updates);
        
        if selections.is_empty() {
            println!("\n{}  No packages selected", "ℹ️".blue());
            return;
        }
        
        if !cli.dry_run && tui::confirm_updates(&selections) {
            perform_updates(&selections);
        } else {
            println!("\n{}  Cancelled", "ℹ️".blue());
        }
    } else if !cli.dry_run {
        println!("\n{}  Ready to update {} packages!", "✨".yellow(), total);
        println!("{}  Run with --interactive to select", "💡".blue());
    } else {
        println!("\n{}  Dry run complete", "ℹ️".blue());
        println!("{}  Run with --interactive to select packages", "💡".blue());
    }
}

fn perform_updates(selections: &[(String, Vec<String>)]) {
    println!("\n{}", "🚀 Starting Updates".green().bold());
    println!("{}", "─".repeat(50).green());
    
    for (category, items) in selections {
        println!("\n{}  Updating {}...", "📦".yellow(), category.bold());
        
        match category.as_str() {
            "0-Core Workspace" => {
                println!("   Running: cargo build --release");
                let status = Command::new("cargo")
                    .args(["build", "--release"])
                    .current_dir("/home/christian/0-core")
                    .status();
                
                match status {
                    Ok(s) if s.success() => println!("   {}  Workspace rebuilt", "✅".green()),
                    _ => println!("   {}  Failed", "❌".red()),
                }
            }
            _ => println!("   {}  Category not implemented yet", "⚠️".yellow()),
        }
    }
    
    println!("\n{}  Updates complete!", "✨".green());
}

fn run_health_check() {
    let output = Command::new("dot-doctor").output().expect("Failed to run doctor");
    if !output.status.success() {
        println!("{}  Health check failed!", "⚠️".yellow());
        std::process::exit(1);
    }
    println!("   {}  System healthy", "✅".green());
}

pub struct UpdateCategory {
    pub name: String,
    pub count: usize,
    pub items: Vec<UpdateItem>,
}

pub struct UpdateItem {
    pub name: String,
    pub current: String,
    pub new: String,
}

fn check_all_updates() -> Vec<UpdateCategory> {
    let mut categories = Vec::new();
    
    categories.push(check_pacman_updates());
    categories.push(check_paru_updates());
    
    let cargo_items = cargo_checker::check_cargo_updates();
    categories.push(UpdateCategory {
        name: "Cargo Tools".to_string(),
        count: cargo_items.len(),
        items: cargo_items,
    });
    
    let nvim_items = neovim_checker::check_neovim_updates();
    categories.push(UpdateCategory {
        name: "Neovim Plugins".to_string(),
        count: nvim_items.len(),
        items: nvim_items,
    });
    
    let workspace_items = cargo_checker::check_workspace_updates();
    categories.push(UpdateCategory {
        name: "0-Core Workspace".to_string(),
        count: workspace_items.len(),
        items: workspace_items,
    });
    
    categories
}

fn check_pacman_updates() -> UpdateCategory {
    println!("   Checking pacman...");
    
    let output = Command::new("checkupdates")
        .output();
    
    let items = match output {
        Ok(out) => {
            // checkupdates returns exit code 2 when no updates (not an error!)
            if out.status.code() == Some(2) {
                Vec::new()
            } else {
                parse_pacman_output(&out.stdout)
            }
        }
        Err(_) => {
            // Fallback to pacman -Qu if checkupdates not available
            eprintln!("   ⚠️  checkupdates not found, using pacman -Qu (may be outdated)");
            let fallback = Command::new("pacman")
                .args(["-Qu"])
                .output()
                .expect("Failed to check pacman");
            parse_pacman_output(&fallback.stdout)
        }
    };
    
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
    
    // Use paru -Qua which syncs AUR database and checks for updates
    let output = Command::new("paru")
        .args(["-Qua"])
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
            
            for item in category.items.iter().take(5) {
                println!(
                    "     {} {} → {}",
                    item.name.white(),
                    item.current.dimmed(),
                    item.new.green()
                );
            }
            
            if category.items.len() > 5 {
                println!("     {} {} more...", "...".dimmed(), category.items.len() - 5);
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
