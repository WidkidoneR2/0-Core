use crate::{UpdateCategory};
use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

pub fn interactive_select(categories: &[UpdateCategory]) -> Vec<(String, Vec<String>)> {
    let mut selections = Vec::new();
    
    println!("\n{}", "🎮 Interactive Selection Mode".cyan().bold());
    println!("{}", "─".repeat(50).cyan());
    println!("{}  {} to toggle  |  {} to confirm", "💡".blue(), "Space".bold(), "Enter".bold());
    println!("{}", "─".repeat(50).cyan());
    
    for category in categories {
        if category.count == 0 {
            continue;
        }
        
        println!("\n{} {} ({} available)", "📦".yellow(), category.name.bold(), category.count);
        
        let items: Vec<String> = category.items.iter()
            .map(|item| format!("{} {} → {}", 
                item.name, 
                item.current, 
                item.new))
            .collect();
        
        if items.is_empty() {
            continue;
        }
        
        let defaults = vec![true; items.len()];
        
        let selected = MultiSelect::with_theme(&ColorfulTheme::default())
            .items(&items)
            .defaults(&defaults)
            .interact()
            .unwrap();
        
        if !selected.is_empty() {
            let selected_names: Vec<String> = selected.iter()
                .map(|&i| category.items[i].name.clone())
                .collect();
            selections.push((category.name.clone(), selected_names));
        }
    }
    
    selections
}

pub fn confirm_updates(selections: &[(String, Vec<String>)]) -> bool {
    let total: usize = selections.iter().map(|(_, items)| items.len()).sum();
    
    println!("\n{}", "📋 Update Summary".yellow().bold());
    println!("{}", "─".repeat(50).yellow());
    
    for (category, items) in selections {
        println!("  {} {} ({} packages)", "📦".green(), category.bold(), items.len());
        for item in items {
            println!("     • {}", item);
        }
    }
    
    println!("{}", "─".repeat(50).yellow());
    println!("  Total: {} packages", total.to_string().bold());
    println!();
    
    dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Proceed with updates?")
        .default(true)
        .interact()
        .unwrap()
}
