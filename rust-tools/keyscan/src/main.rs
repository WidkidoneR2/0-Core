//! keyscan v1.0.0 - Sway Keybind Analysis
//! 🌲 Faelight Forest

use colored::*;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

// ============================================================================
// TYPES
// ============================================================================

#[derive(Debug, Clone)]
struct Keybind {
    line: usize,
    binding: String,
    modifiers: Vec<String>,
    key: String,
    command: String,
}

#[derive(Debug)]
struct KeybindStats {
    total_binds: usize,
    unique_binds: usize,
    conflicts: usize,
    modifier_usage: HashMap<String, usize>,
    key_usage: HashMap<String, usize>,
    categories: HashMap<String, usize>,
}

// ============================================================================
// PARSING
// ============================================================================

fn get_sway_config_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".config/sway/config")
}

fn parse_sway_config(path: &PathBuf) -> Result<Vec<Keybind>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    
    let mut keybinds = Vec::new();
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        
        // Match bindsym or bindcode
        if line.starts_with("bindsym") || line.starts_with("bindcode") {
            if let Some(keybind) = parse_keybind_line(line, line_num + 1) {
                keybinds.push(keybind);
            }
        }
    }
    
    Ok(keybinds)
}

fn parse_keybind_line(line: &str, line_num: usize) -> Option<Keybind> {
    // Remove "bindsym" or "bindcode" prefix
    let line = line.trim_start_matches("bindsym").trim_start_matches("bindcode").trim();
    
    // Remove flags
    let line = line
        .trim_start_matches("--no-repeat")
        .trim_start_matches("--release")
        .trim_start_matches("--locked")
        .trim_start_matches("--to-code")
        .trim();
    
    // Split on first whitespace
    let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
    
    if parts.len() >= 2 {
        let binding = normalize_binding(parts[0]);
        let (modifiers, key) = parse_modifiers_and_key(&binding);
        
        Some(Keybind {
            line: line_num,
            binding: binding.clone(),
            modifiers,
            key,
            command: parts[1].trim().to_string(),
        })
    } else {
        None
    }
}

fn normalize_binding(binding: &str) -> String {
    binding
        .replace("Mod4", "Super")
        .replace("$mod", "Super")
        .to_lowercase()
}

fn parse_modifiers_and_key(binding: &str) -> (Vec<String>, String) {
    let parts: Vec<&str> = binding.split('+').collect();
    
    if parts.is_empty() {
        return (vec![], String::new());
    }
    
    let key = parts.last().unwrap().to_string();
    let modifiers: Vec<String> = parts[..parts.len()-1]
        .iter()
        .map(|s| s.to_string())
        .collect();
    
    (modifiers, key)
}

// ============================================================================
// CATEGORIZATION
// ============================================================================

fn categorize_keybind(kb: &Keybind) -> String {
    let cmd = kb.command.to_lowercase();
    
    if cmd.contains("workspace") {
        "Workspaces".to_string()
    } else if cmd.contains("exec") {
        if cmd.contains("firefox") || cmd.contains("browser") || cmd.contains("brave") {
            "Browser".to_string()
        } else if cmd.contains("terminal") || cmd.contains("foot") || cmd.contains("kitty") {
            "Terminal".to_string()
        } else if cmd.contains("launcher") || cmd.contains("fuzzel") {
            "Launcher".to_string()
        } else {
            "Applications".to_string()
        }
    } else if cmd.contains("move") || cmd.contains("resize") || cmd.contains("focus") {
        "Window Management".to_string()
    } else if cmd.contains("mode") {
        "Modes".to_string()
    } else if cmd.contains("reload") || cmd.contains("exit") || cmd.contains("kill") {
        "System".to_string()
    } else if cmd.contains("layout") || cmd.contains("split") || cmd.contains("fullscreen") || cmd.contains("floating") {
        "Layout".to_string()
    } else if cmd.contains("brightness") || cmd.contains("volume") {
        "Media".to_string()
    } else {
        "Other".to_string()
    }
}

// ============================================================================
// ANALYSIS
// ============================================================================

fn find_conflicts(keybinds: &[Keybind]) -> HashMap<String, Vec<Keybind>> {
    let mut map: HashMap<String, Vec<Keybind>> = HashMap::new();
    
    for keybind in keybinds {
        map.entry(keybind.binding.clone())
            .or_insert_with(Vec::new)
            .push(keybind.clone());
    }
    
    map.into_iter()
        .filter(|(_, binds)| binds.len() > 1)
        .collect()
}

fn calculate_stats(keybinds: &[Keybind]) -> KeybindStats {
    let mut modifier_usage = HashMap::new();
    let mut key_usage = HashMap::new();
    let mut categories = HashMap::new();
    
    let unique_bindings: HashSet<String> = keybinds.iter()
        .map(|kb| kb.binding.clone())
        .collect();
    
    for kb in keybinds {
        // Count modifiers
        for modifier in &kb.modifiers {
            *modifier_usage.entry(modifier.clone()).or_insert(0) += 1;
        }
        
        // Count keys
        *key_usage.entry(kb.key.clone()).or_insert(0) += 1;
        
        // Count categories
        let category = categorize_keybind(kb);
        *categories.entry(category).or_insert(0) += 1;
    }
    
    let conflicts = find_conflicts(keybinds);
    
    KeybindStats {
        total_binds: keybinds.len(),
        unique_binds: unique_bindings.len(),
        conflicts: conflicts.len(),
        modifier_usage,
        key_usage,
        categories,
    }
}

fn suggest_unused_combos(keybinds: &[Keybind]) -> Vec<String> {
    let used_bindings: HashSet<String> = keybinds.iter()
        .map(|kb| kb.binding.clone())
        .collect();
    
    let modifiers = vec!["super", "super+shift", "super+ctrl", "super+alt"];
    let keys = vec!["f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12"];
    
    let mut suggestions = Vec::new();
    
    for modifier in &modifiers {
        for key in &keys {
            let combo = format!("{}+{}", modifier, key);
            if !used_bindings.contains(&combo) {
                suggestions.push(combo);
                if suggestions.len() >= 10 {
                    return suggestions;
                }
            }
        }
    }
    
    suggestions
}

// ============================================================================
// OUTPUT COMMANDS
// ============================================================================

fn cmd_list(keybinds: &[Keybind]) {
    println!("{}", "🔍 All Keybindings".cyan().bold());
    println!("{}", "━".repeat(80).dimmed());
    println!();
    
    let mut sorted = keybinds.to_vec();
    sorted.sort_by(|a, b| a.binding.cmp(&b.binding));
    
    for kb in sorted {
        println!("{} → {}", 
                 kb.binding.green(),
                 kb.command.dimmed());
    }
    
    println!();
    println!("{} {} keybindings", "Total:".bold(), keybinds.len());
}

fn cmd_stats(keybinds: &[Keybind]) {
    let stats = calculate_stats(keybinds);
    
    println!("{}", "📊 Keybind Statistics".cyan().bold());
    println!("{}", "━".repeat(80).dimmed());
    println!();
    
    println!("{}", "Overview:".bold());
    println!("  Total keybindings: {}", stats.total_binds);
    println!("  Unique bindings:   {}", stats.unique_binds);
    println!("  Conflicts:         {}", if stats.conflicts > 0 {
        stats.conflicts.to_string().red()
    } else {
        stats.conflicts.to_string().green()
    });
    println!();
    
    println!("{}", "Modifier Usage:".bold());
    let mut mod_sorted: Vec<_> = stats.modifier_usage.iter().collect();
    mod_sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (modifier, count) in mod_sorted {
        println!("  {:<15} {}", modifier, count);
    }
    println!();
    
    println!("{}", "Most Used Keys:".bold());
    let mut key_sorted: Vec<_> = stats.key_usage.iter().collect();
    key_sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (key, count) in key_sorted.iter().take(10) {
        println!("  {:<15} {}", key, count);
    }
    println!();
    
    println!("{}", "Categories:".bold());
    let mut cat_sorted: Vec<_> = stats.categories.iter().collect();
    cat_sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (category, count) in cat_sorted {
        println!("  {:<20} {}", category, count);
    }
}

fn cmd_find(keybinds: &[Keybind], pattern: &str) {
    let pattern_lower = pattern.to_lowercase();
    
    let matches: Vec<&Keybind> = keybinds.iter()
        .filter(|kb| {
            kb.binding.contains(&pattern_lower) || 
            kb.command.to_lowercase().contains(&pattern_lower)
        })
        .collect();
    
    println!("{}", format!("🔎 Search: '{}'", pattern).cyan().bold());
    println!("{}", "━".repeat(80).dimmed());
    println!();
    
    if matches.is_empty() {
        println!("{}", "No matches found".yellow());
        return;
    }
    
    for kb in &matches {
        println!("{} → {}", 
                 kb.binding.green(),
                 kb.command.dimmed());
    }
    
    println!();
    println!("{} {} matches", "Total:".bold(), matches.len());
}

fn cmd_category(keybinds: &[Keybind]) {
    let mut categories: HashMap<String, Vec<&Keybind>> = HashMap::new();
    
    for kb in keybinds {
        let cat = categorize_keybind(kb);
        categories.entry(cat).or_insert_with(Vec::new).push(kb);
    }
    
    println!("{}", "📂 Keybindings by Category".cyan().bold());
    println!("{}", "━".repeat(80).dimmed());
    println!();
    
    let mut cat_sorted: Vec<_> = categories.iter().collect();
    cat_sorted.sort_by(|a, b| a.0.cmp(b.0));
    
    for (category, binds) in cat_sorted {
        println!("{} {}:", "▸".blue(), category.bold());
        
        let mut sorted_binds = binds.clone();
        sorted_binds.sort_by(|a, b| a.binding.cmp(&b.binding));
        
        for kb in sorted_binds {
            println!("  {} → {}", 
                     kb.binding.green(),
                     kb.command.dimmed());
        }
        println!();
    }
}

fn cmd_cheatsheet(keybinds: &[Keybind]) {
    println!("# Sway Keybind Cheatsheet");
    println!();
    
    let mut categories: HashMap<String, Vec<&Keybind>> = HashMap::new();
    
    for kb in keybinds {
        let cat = categorize_keybind(kb);
        categories.entry(cat).or_insert_with(Vec::new).push(kb);
    }
    
    let mut cat_sorted: Vec<_> = categories.iter().collect();
    cat_sorted.sort_by(|a, b| a.0.cmp(b.0));
    
    for (category, binds) in cat_sorted {
        println!("## {}", category);
        println!();
        println!("| Keybind | Command |");
        println!("|---------|---------|");
        
        let mut sorted_binds = binds.clone();
        sorted_binds.sort_by(|a, b| a.binding.cmp(&b.binding));
        
        for kb in sorted_binds {
            println!("| `{}` | {} |", kb.binding, kb.command);
        }
        println!();
    }
    
    println!("---");
    println!();
    println!("*Generated by keyscan v1.0.0*");
}

fn cmd_suggest(keybinds: &[Keybind]) {
    let suggestions = suggest_unused_combos(keybinds);
    
    println!("{}", "💡 Suggested Unused Keybindings".cyan().bold());
    println!("{}", "━".repeat(80).dimmed());
    println!();
    
    if suggestions.is_empty() {
        println!("{}", "All common keybinds are used!".yellow());
        return;
    }
    
    println!("Available combinations:");
    for suggestion in suggestions {
        println!("  {}", suggestion.green());
    }
}

fn cmd_health() -> i32 {
    let config_path = get_sway_config_path();
    
    match parse_sway_config(&config_path) {
        Ok(keybinds) => {
            let conflicts = find_conflicts(&keybinds);
            
            if conflicts.is_empty() {
                println!("✅ Keybinds: {} unique, no conflicts", keybinds.len());
                0
            } else {
                println!("⚠️  Keybinds: {} conflicts detected", conflicts.len());
                1
            }
        }
        Err(e) => {
            println!("❌ Keybinds: {}", e);
            1
        }
    }
}

fn cmd_conflicts(keybinds: &[Keybind]) -> i32 {
    let conflicts = find_conflicts(keybinds);
    
    println!("{}", "🔍 Keybind Conflict Detection".cyan().bold());
    println!("{}", "━".repeat(80).dimmed());
    println!();
    
    if conflicts.is_empty() {
        println!("✅ No conflicts detected");
        println!("   {} unique keybindings", keybinds.len());
        return 0;
    }
    
    println!("⚠️  {} conflict(s) detected", conflicts.len());
    println!("   {} total keybinds", keybinds.len());
    println!();
    
    for (binding, binds) in &conflicts {
        println!("{} {}", "Conflict:".red().bold(), binding.yellow());
        for bind in binds {
            println!("  {} {}", format!("Line {}:", bind.line).dimmed(), bind.command);
        }
        println!();
    }
    
    println!("💡 Fix conflicts by removing or modifying duplicate bindings");
    1
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("{}", "keyscan v1.0.0 - Sway Keybind Analyzer".bold());
        println!();
        println!("Usage: keyscan [COMMAND] [OPTIONS]");
        println!();
        println!("Commands:");
        println!("  (none)           Check for conflicts (default)");
        println!("  --list           List all keybindings");
        println!("  --stats          Show keybind statistics");
        println!("  --find PATTERN   Search keybindings");
        println!("  --category       Group keybindings by category");
        println!("  --cheatsheet     Generate markdown cheatsheet");
        println!("  --suggest        Suggest unused key combinations");
        println!("  --health         Health check for dot-doctor");
        println!();
        println!("Options:");
        println!("  --config PATH    Use specific config file");
        println!("  -h, --help       Show this help");
        println!("  -v, --version    Show version");
        return;
    }
    
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("keyscan v1.0.0");
        return;
    }
    
    // Health check (for dot-doctor)
    if args.contains(&"--health".to_string()) {
        process::exit(cmd_health());
    }
    
    // Get config path
    let config_path = if let Some(pos) = args.iter().position(|a| a == "--config") {
        if pos + 1 < args.len() {
            PathBuf::from(&args[pos + 1])
        } else {
            eprintln!("Error: --config requires a path");
            process::exit(1);
        }
    } else {
        get_sway_config_path()
    };
    
    // Parse config
    let keybinds = match parse_sway_config(&config_path) {
        Ok(kb) => kb,
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            process::exit(1);
        }
    };
    
    // Execute command
    if args.contains(&"--list".to_string()) {
        cmd_list(&keybinds);
    } else if args.contains(&"--stats".to_string()) {
        cmd_stats(&keybinds);
    } else if let Some(pos) = args.iter().position(|a| a == "--find") {
        if pos + 1 < args.len() {
            cmd_find(&keybinds, &args[pos + 1]);
        } else {
            eprintln!("Error: --find requires a pattern");
            process::exit(1);
        }
    } else if args.contains(&"--category".to_string()) {
        cmd_category(&keybinds);
    } else if args.contains(&"--cheatsheet".to_string()) {
        cmd_cheatsheet(&keybinds);
    } else if args.contains(&"--suggest".to_string()) {
        cmd_suggest(&keybinds);
    } else {
        // Default: check conflicts
        process::exit(cmd_conflicts(&keybinds));
    }
}
