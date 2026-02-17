#![allow(dead_code)]
use crate::UpdateCategory;
use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

/// Interactive package selection using TUI
pub fn interactive_select(categories: &[UpdateCategory]) -> Vec<(String, Vec<String>)> {
    let mut selections = Vec::new();

    println!("\n{}", "🎮 Interactive Selection Mode".cyan().bold());
    println!("{}", "─".repeat(60).cyan());
    println!(
        "{}  {} to toggle  |  {} to confirm  |  {} to cancel",
        "💡".blue(),
        "Space".bold(),
        "Enter".bold(),
        "Esc".bold()
    );
    println!("{}", "─".repeat(60).cyan());

    for category in categories {
        // Skip empty categories
        if category.count == 0 {
            continue;
        }

        println!(
            "\n{} {} ({} available)",
            category.emoji.yellow(),
            category.name.bold(),
            category.count
        );

        // Format items for display
        let items: Vec<String> = category
            .items
            .iter()
            .map(|item| {
                format!(
                    "{:<30} {} → {}",
                    item.name,
                    item.current.red(),
                    item.new.green()
                )
            })
            .collect();

        if items.is_empty() {
            continue;
        }

        // All selected by default
        let defaults = vec![true; items.len()];

        // Show multi-select dialog
        let selected_result = MultiSelect::with_theme(&ColorfulTheme::default())
            .items(&items)
            .defaults(&defaults)
            .interact_opt();

        match selected_result {
            Ok(Some(selected)) => {
                if !selected.is_empty() {
                    let selected_names: Vec<String> = selected
                        .iter()
                        .map(|&i| category.items[i].name.clone())
                        .collect();

                    println!(
                        "   {}  Selected {} packages",
                        "✓".green(),
                        selected_names.len()
                    );

                    selections.push((category.name.clone(), selected_names));
                } else {
                    println!("   {}  Skipped", "→".dimmed());
                }
            }
            Ok(None) => {
                // User cancelled with Esc
                println!("\n{}  Selection cancelled", "ℹ️".blue());
                return Vec::new();
            }
            Err(e) => {
                eprintln!("   {}  Selection error: {}", "❌".red(), e);
                continue;
            }
        }
    }

    selections
}

/// Confirm selected updates before proceeding
pub fn confirm_updates(selections: &[(String, Vec<String>)]) -> bool {
    if selections.is_empty() {
        return false;
    }

    let total: usize = selections.iter().map(|(_, items)| items.len()).sum();

    println!("\n{}", "📋 Update Summary".yellow().bold());
    println!("{}", "─".repeat(60).yellow());

    for (category, items) in selections {
        println!(
            "  {} {} ({} packages)",
            "📦".green(),
            category.bold(),
            items.len()
        );

        // Show first 10 items, then "... and N more"
        for item in items.iter().take(10) {
            println!("     • {}", item);
        }

        if items.len() > 10 {
            println!("     {} and {} more...", "...".dimmed(), items.len() - 10);
        }
    }

    println!("{}", "─".repeat(60).yellow());
    println!("  Total: {} packages", total.to_string().bold());
    println!();

    // Confirmation dialog
    let result = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Proceed with updates?")
        .default(true)
        .interact_opt();

    match result {
        Ok(Some(confirmed)) => confirmed,
        Ok(None) => {
            println!("{}  Cancelled", "ℹ️".blue());
            false
        }
        Err(e) => {
            eprintln!("{}  Confirmation error: {}", "❌".red(), e);
            false
        }
    }
}
