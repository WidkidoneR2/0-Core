use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{self, Command};

// ANSI colors
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[0;36m";
const BLUE: &str = "\x1b[0;34m";
const NC: &str = "\x1b[0m";

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    
    match command {
        "add" => cmd_add(),
        "list" | "ls" => cmd_list(args.get(2).map(|s| s.as_str())),
        "show" => {
            if args.len() < 3 {
                error("Usage: intent show <id>");
            }
            cmd_show(&args[2]);
        }
        "search" => {
            if args.len() < 3 {
                error("Usage: intent search <term>");
            }
            cmd_search(&args[2]);
        }
        "help" | "-h" | "--help" => cmd_help(),
        _ => {
            eprintln!("{}❌ Error: Unknown command: {}{}", RED, command, NC);
            cmd_help();
            process::exit(1);
        }
    }
}

fn get_intent_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("0-core/INTENT")
}

fn error(msg: &str) -> ! {
    eprintln!("{}❌ Error: {}{}", RED, msg, NC);
    process::exit(1);
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn get_next_id(category: &str) -> String {
    let dir = get_intent_dir().join(category);
    if !dir.exists() {
        return "001".to_string();
    }
    
    let mut max_id = 0;
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(id_str) = name.split('-').next() {
                if let Ok(id) = id_str.parse::<u32>() {
                    if id > max_id {
                        max_id = id;
                    }
                }
            }
        }
    }
    format!("{:03}", max_id + 1)
}

fn cmd_add() {
    println!("{}📝 Adding New Intent{}", CYAN, NC);
    println!();
    
    // Select category
    println!("Select category:");
    println!("  1) decisions");
    println!("  2) experiments");
    println!("  3) philosophy");
    println!("  4) future");
    println!("  5) incidents");
    
    let category = match prompt("Choice (1-5): ").as_str() {
        "1" => "decisions",
        "2" => "experiments",
        "3" => "philosophy",
        "4" => "future",
        "5" => "incidents",
        _ => error("Invalid choice"),
    };
    
    let id = get_next_id(category);
    let title = prompt("Title: ");
    let tags_input = prompt("Tags (comma-separated): ");
    
    println!("Status:");
    println!("  1) planned");
    println!("  2) in-progress");
    println!("  3) complete");
    println!("  4) abandoned");
    
    let status = match prompt("Choice (1-4): ").as_str() {
        "1" => "planned",
        "2" => "in-progress",
        "3" => "complete",
        "4" => "abandoned",
        _ => error("Invalid choice"),
    };
    
    // Create filename
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-");
    
    let filename = get_intent_dir()
        .join(category)
        .join(format!("{}-{}.md", id, slug));
    
    let today = get_today();
    
    let content = format!(r#"---
id: {}
date: {}
type: {}
title: "{}"
status: {}
tags: [{}]
---

## Why

[Explain the reasoning behind this decision/experiment/vision]

## What

[Describe what we're doing or planning to do]

## How (if applicable)

[Technical details, implementation notes]

## Alternatives Considered (if applicable)

[What else we thought about and why we didn't choose it]

## Impact

[Expected outcomes, benefits, risks]

## Status Updates

### {}

[Initial creation]

---
*Part of the 0-Core Intent Ledger* 🌲
"#, id, today, category, title, status, tags_input, today);
    
    // Ensure directory exists
    if let Some(parent) = filename.parent() {
        fs::create_dir_all(parent).ok();
    }
    
    fs::write(&filename, content).expect("Failed to write intent file");
    
    println!("{}✅ Intent created: {}{}", GREEN, filename.display(), NC);
    println!("{}ℹ️  Opening in editor...{}", BLUE, NC);
    
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
    Command::new(&editor)
        .arg(&filename)
        .status()
        .ok();
}

fn cmd_list(category: Option<&str>) {
    let intent_dir = get_intent_dir();
    
    println!("{}📋 Intent Ledger{}", CYAN, NC);
    println!();
    
    let categories = if let Some(cat) = category {
        vec![cat]
    } else {
        vec!["decisions", "experiments", "philosophy", "future", "incidents"]
    };
    
    for cat in categories {
        println!("{}{}:{}", YELLOW, cat, NC);
        
        let cat_dir = intent_dir.join(cat);
        if !cat_dir.exists() {
            println!("  {}(empty){}", BLUE, NC);
            println!();
            continue;
        }
        
        let mut files: Vec<_> = fs::read_dir(&cat_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .collect();
        
        files.sort_by_key(|e| e.file_name());
        
        if files.is_empty() {
            println!("  {}(empty){}", BLUE, NC);
        } else {
            for entry in files {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let id = extract_frontmatter(&content, "id").unwrap_or("?".to_string());
                    let title = extract_frontmatter(&content, "title").unwrap_or("Untitled".to_string());
                    let status = extract_frontmatter(&content, "status").unwrap_or("unknown".to_string());
                    
                    let icon = match status.as_str() {
                        "planned" => "📅",
                        "in-progress" => "🚧",
                        "complete" => "✅",
                        "decided" => "✓",
                        "resolved" => "🔧",
                        "abandoned" => "❌",
                        _ => "❓",
                    };
                    
                    println!("  {} [{}] {}", icon, id, title);
                }
            }
        }
        println!();
    }
}

fn cmd_show(id: &str) {
    let intent_dir = get_intent_dir();
    let mut found_file: Option<PathBuf> = None;
    
    // Check if id contains category (e.g., "future/002")
    if id.contains('/') {
        let parts: Vec<&str> = id.split('/').collect();
        if parts.len() == 2 {
            let category = parts[0];
            let num = parts[1];
            let cat_dir = intent_dir.join(category);
            if let Ok(entries) = fs::read_dir(&cat_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(&format!("{}-", num)) {
                        found_file = Some(entry.path());
                        break;
                    }
                }
            }
        }
    } else {
        // Search all categories for the ID
        for cat in &["decisions", "experiments", "philosophy", "future", "incidents"] {
            let cat_dir = intent_dir.join(cat);
            if let Ok(entries) = fs::read_dir(&cat_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(&format!("{}-", id)) {
                        found_file = Some(entry.path());
                        break;
                    }
                }
            }
            if found_file.is_some() {
                break;
            }
        }
    }
    
    let file = found_file.unwrap_or_else(|| error(&format!("Intent {} not found", id)));
    
    // Try bat first, fallback to cat
    let bat_status = Command::new("bat")
        .args(["--style=header,grid", "--theme=Monokai Extended", "--paging=never"])
        .arg(&file)
        .status();
    
    if bat_status.is_err() || !bat_status.unwrap().success() {
        println!("{}─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────{}", GREEN, NC);
        println!("{}File: {}{}", GREEN, file.display(), NC);
        println!("{}─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────{}", GREEN, NC);
        if let Ok(content) = fs::read_to_string(&file) {
            println!("{}", content);
        }
        println!("{}─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────{}", GREEN, NC);
    }
}

fn cmd_search(term: &str) {
    let intent_dir = get_intent_dir();
    
    println!("{}🔍 Searching for: {}{}", CYAN, term, NC);
    println!();
    
    for cat in &["decisions", "experiments", "philosophy", "future", "incidents"] {
        let cat_dir = intent_dir.join(cat);
        if let Ok(entries) = fs::read_dir(&cat_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.to_lowercase().contains(&term.to_lowercase()) {
                        let id = extract_frontmatter(&content, "id").unwrap_or("?".to_string());
                        let title = extract_frontmatter(&content, "title").unwrap_or("Untitled".to_string());
                        println!("{}[{}]{} {} {}({}){}", GREEN, id, NC, title, BLUE, cat, NC);
                    }
                }
            }
        }
    }
}

fn cmd_help() {
    println!("{}intent{} - Intent Ledger management", CYAN, NC);
    println!();
    println!("{}Usage:{}", YELLOW, NC);
    println!("  intent add                  Add new intent (interactive)");
    println!("  intent list [category]      List all intents (or specific category)");
    println!("  intent show <id>            Show specific intent");
    println!("  intent search <term>        Search intents by keyword/tag");
    println!();
    println!("{}Categories:{}", YELLOW, NC);
    println!("  decisions     - Major architectural choices");
    println!("  experiments   - Things we tried (success or failure)");
    println!("  philosophy    - Core beliefs and principles");
    println!("  future        - Planned features and vision");
    println!("  incidents     - System failures and lessons learned");
    println!();
    println!("{}Examples:{}", YELLOW, NC);
    println!("  intent add                  # Interactive intent creation");
    println!("  intent list decisions       # List all decisions");
    println!("  intent show 001             # Show intent 001");
    println!("  intent search rust          # Find all intents mentioning rust");
    println!();
    println!("{}The forest remembers.{} 🌲", BLUE, NC);
}

fn extract_frontmatter(content: &str, field: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with(&format!("{}: ", field)) {
            let value = line.split(": ").skip(1).collect::<Vec<&str>>().join(": ");
            // Remove quotes if present
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}

fn get_today() -> String {
    let output = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .expect("Failed to get date");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
