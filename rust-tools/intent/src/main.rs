use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{self, Command};

const VERSION: &str = "1.0.0";

// ANSI colors
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[0;36m";
const BLUE: &str = "\x1b[0;34m";
const GRAY: &str = "\x1b[0;90m";
const NC: &str = "\x1b[0m";

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "add" => cmd_add(),
        "list" | "ls" => {
            let filter = args.get(2).map(|s| s.as_str());
            cmd_list(filter);
        }
        "show" => {
            if args.len() < 3 {
                error("Usage: intent show <id> or intent show <category> <id>");
            }
            if args.len() >= 4 {
                cmd_show(&format!("{}/{}", args[2], args[3]));
            } else {
                cmd_show(&args[2]);
            }
        }
        "search" => {
            if args.len() < 3 {
                error("Usage: intent search <term> [--status <status>] [--tag <tag>]");
            }
            let term = &args[2];
            let status_filter = args.iter().position(|a| a == "--status")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());
            let tag_filter = args.iter().position(|a| a == "--tag")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());
            cmd_search(term, status_filter, tag_filter);
        }
        "stats" => cmd_stats(),
        "complete" => {
            if args.len() < 3 { error("Usage: intent complete <id>"); }
            cmd_status_change(&args[2], "complete", "complete");
        }
        "cancel" => {
            if args.len() < 3 { error("Usage: intent cancel <id>"); }
            cmd_status_change(&args[2], "cancelled", "cancelled");
        }
        "defer" => {
            if args.len() < 3 { error("Usage: intent defer <id>"); }
            cmd_status_change(&args[2], "deferred", "deferred");
        }
        "start" => {
            if args.len() < 3 { error("Usage: intent start <id>"); }
            cmd_status_change(&args[2], "in-progress", "future");
        }
        "version" | "--version" | "-v" => {
            println!("intent v{} - 0-Core Intent Ledger", VERSION);
        }
        "help" | "-h" | "--help" => cmd_help(),
        // Shorthand: just the number
        id if id.chars().all(|c| c.is_numeric()) => cmd_show(id),
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
    let status = prompt("Status (planned/in-progress/complete): ");

    let tags = prompt("Tags (comma-separated, optional): ");
    let tags_str = if tags.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            tags.split(',')
                .map(|t| t.trim())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    let filename = format!(
        "{}-{}.md",
        id,
        title
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
    );

    let file_path = get_intent_dir().join(category).join(&filename);

    let content = format!(
        r#"---
id: {}
date: {}
type: {}
title: "{}"
status: {}
tags: {}
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---
"#,
        id, date, category, title, status, tags_str
    );

    fs::write(&file_path, content).expect("Failed to write file");
    println!("{}✅ Created: {}{}", GREEN, filename, NC);
}

fn cmd_list(filter: Option<&str>) {
    let intent_dir = get_intent_dir();

    println!("{}📋 Intent Ledger{}", CYAN, NC);
    println!();

    let show_complete = filter == Some("--complete") || filter == Some("-c");
    let show_planned = filter == Some("--planned") || filter == Some("-p");
    let show_active = filter == Some("--active") || filter == Some("-a");

    let categories = if show_complete {
        vec!["complete"]
    } else if let Some(cat) = filter {
        if !cat.starts_with("--") && !cat.starts_with("-") {
            vec![cat]
        } else {
            vec!["decisions", "experiments", "philosophy", "future", "cancelled", "deferred", "incidents"]
        }
    } else {
        vec!["decisions", "experiments", "philosophy", "future", "cancelled", "deferred", "incidents"]
    };

    let mut total_count = 0;
    let mut complete_count = 0;
    let mut planned_count = 0;
    let mut in_progress_count = 0;

    for cat in categories {
        let cat_dir = intent_dir.join(cat);
        if !cat_dir.exists() {
            continue;
        }

        let mut files: Vec<_> = fs::read_dir(&cat_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .collect();

        files.sort_by_key(|e| e.file_name());

        let mut displayed_any = false;

        for entry in files {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let filename = entry.file_name().to_string_lossy().to_string();
                if filename == "README.md" {
                    continue;
                }
                
                if extract_frontmatter(&content, "type").map(|t| t == "index").unwrap_or(false) {
                    continue;
                }

                let id = extract_frontmatter(&content, "id").unwrap_or("?".to_string());
                let title = extract_frontmatter(&content, "title").unwrap_or("Untitled".to_string());
                let status = extract_frontmatter(&content, "status").unwrap_or("unknown".to_string());

                total_count += 1;
                
                match status.as_str() {
                    "complete" => complete_count += 1,
                    "planned" => planned_count += 1,
                    "in-progress" => in_progress_count += 1,
                    _ => {}
                }

                let should_display = if show_planned {
                    status == "planned"
                } else if show_active {
                    status == "in-progress" || status == "planned"
                } else if show_complete {
                    status == "complete"
                } else {
                    true
                };

                if !should_display {
                    continue;
                }

                if !displayed_any {
                    println!("{}{}:{}", YELLOW, cat, NC);
                    displayed_any = true;
                }

                let status_color = match status.as_str() {
                    "complete" => GREEN,
                    "planned" => BLUE,
                    "in-progress" => YELLOW,
                    _ => GRAY,
                };

                println!(
                    "  {}{:<4}{} {}[{}]{} {}",
                    GRAY, id, NC, status_color, status, NC, title
                );
            }
        }

        if displayed_any {
            println!();
        }
    }

    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", GRAY, NC);
    println!(
        "Total: {}  |  {}Complete: {}{}  |  {}Planned: {}{}  |  {}In Progress: {}{}",
        total_count,
        GREEN, complete_count, NC,
        BLUE, planned_count, NC,
        YELLOW, in_progress_count, NC
    );
    println!();
}

fn cmd_show(id: &str) {
    let intent_dir = get_intent_dir();
    let mut found_file: Option<PathBuf> = None;

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
        for cat in &[
            "decisions",
            "experiments",
            "philosophy",
            "future",
            "incidents",
            "complete",
        ] {
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

    let bat_status = Command::new("bat")
        .args([
            "--style=plain",
            "--color=always",
            "--paging=never",
            "--decorations=always",
            "--wrap=never",
            file.to_str().unwrap(),
        ])
        .status();

    if bat_status.is_err() || !bat_status.unwrap().success() {
        Command::new("cat")
            .arg(file.to_str().unwrap())
            .status()
            .expect("Failed to display file");
    }
}

fn cmd_search(term: &str, status_filter: Option<&str>, tag_filter: Option<&str>) {
    let intent_dir = get_intent_dir();
    println!("{}🔍 Searching for: {}{}", CYAN, term, NC);
    if let Some(status) = status_filter {
        println!("   Status: {}", status);
    }
    if let Some(tag) = tag_filter {
        println!("   Tag: {}", tag);
    }
    println!();

    let mut found = false;

    for cat in &[
        "decisions",
        "experiments",
        "philosophy",
        "future",
        "incidents",
        "complete",
    ] {
        let cat_dir = intent_dir.join(cat);
        if let Ok(entries) = fs::read_dir(&cat_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|x| x == "md").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let filename = entry.file_name().to_string_lossy().to_string();
                        if filename == "README.md" {
                            continue;
                        }

                        if !content.to_lowercase().contains(&term.to_lowercase()) {
                            continue;
                        }

                        let status = extract_frontmatter(&content, "status").unwrap_or_default();
                        if let Some(filter) = status_filter {
                            if status != filter {
                                continue;
                            }
                        }

                        if let Some(tag) = tag_filter {
                            let tags = extract_frontmatter(&content, "tags").unwrap_or_default();
                            if !tags.contains(tag) {
                                continue;
                            }
                        }

                        let id = extract_frontmatter(&content, "id").unwrap_or("?".to_string());
                        let title = extract_frontmatter(&content, "title")
                            .unwrap_or("Untitled".to_string());

                        let status_color = match status.as_str() {
                            "complete" => GREEN,
                            "planned" => BLUE,
                            "in-progress" => YELLOW,
                            _ => GRAY,
                        };

                        println!(
                            "{}{}/{:<4}{} {}[{}]{} {}",
                            GRAY, cat, id, NC, status_color, status, NC, title
                        );
                        found = true;
                    }
                }
            }
        }
    }

    if !found {
        println!("{}No results found{}", GRAY, NC);
    }
    println!();
}

fn cmd_stats() {
    let intent_dir = get_intent_dir();
    
    println!();
    println!("{}📊 Intent Statistics{}", CYAN, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let mut category_stats: HashMap<String, (usize, usize)> = HashMap::new();
    let mut status_counts: HashMap<String, usize> = HashMap::new();
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    let mut total = 0;

    for cat in &[
        "decisions",
        "experiments",
        "philosophy",
        "future",
        "incidents",
        "complete",
    ] {
        let cat_dir = intent_dir.join(cat);
        if let Ok(entries) = fs::read_dir(&cat_dir) {
            let mut cat_total = 0;
            let mut cat_complete = 0;

            for entry in entries.flatten() {
                if entry.path().extension().map(|x| x == "md").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let filename = entry.file_name().to_string_lossy().to_string();
                        if filename == "README.md" {
                            continue;
                        }

                        if extract_frontmatter(&content, "type").map(|t| t == "index").unwrap_or(false) {
                            continue;
                        }

                        cat_total += 1;
                        total += 1;

                        let status = extract_frontmatter(&content, "status").unwrap_or("unknown".to_string());
                        *status_counts.entry(status.clone()).or_insert(0) += 1;

                        if status == "complete" {
                            cat_complete += 1;
                        }

                        // Count tags
                        if let Some(tags) = extract_frontmatter(&content, "tags") {
                            for tag in tags.split(',') {
                                let clean_tag = tag.trim().trim_matches(|c| c == '[' || c == ']');
                                if !clean_tag.is_empty() {
                                    *tag_counts.entry(clean_tag.to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }

            if cat_total > 0 {
                category_stats.insert(cat.to_string(), (cat_total, cat_complete));
            }
        }
    }

    // Overall stats
    let complete = status_counts.get("complete").copied().unwrap_or(0);
    let planned = status_counts.get("planned").copied().unwrap_or(0);
    let in_progress = status_counts.get("in-progress").copied().unwrap_or(0);
    let success_rate = if total > 0 {
        (complete as f64 / total as f64 * 100.0) as usize
    } else {
        0
    };

    println!("{}Total Intents:{} {}", YELLOW, NC, total);
    println!("{}Success Rate:{} {}% ({} complete)", YELLOW, NC, success_rate, complete);
    println!();

    // Status breakdown
    println!("{}By Status:{}", YELLOW, NC);
    for (status, count) in status_counts.iter() {
        let color = match status.as_str() {
            "complete" => GREEN,
            "planned" => BLUE,
            "in-progress" => YELLOW,
            _ => GRAY,
        };
        println!("  {}{}: {}{}", color, status, count, NC);
    }
    println!();

    // Category breakdown
    println!("{}By Category:{}", YELLOW, NC);
    let mut cats: Vec<_> = category_stats.iter().collect();
    cats.sort_by_key(|(_, (total, _))| std::cmp::Reverse(*total));
    
    for (cat, (total, complete)) in cats {
        let rate = if *total > 0 {
            (*complete as f64 / *total as f64 * 100.0) as usize
        } else {
            0
        };
        println!("  {}: {} total ({} complete, {}%)", cat, total, complete, rate);
    }
    println!();

    // Top tags
    if !tag_counts.is_empty() {
        println!("{}Top Tags:{}", YELLOW, NC);
        let mut tags: Vec<_> = tag_counts.iter().collect();
        tags.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        
        for (tag, count) in tags.iter().take(10) {
            println!("  {}: {}", tag, count);
        }
        println!();
    }
}

fn cmd_help() {
    println!("{}intent v{} - 0-Core Intent Ledger{}", CYAN, VERSION, NC);
    println!();
    println!("USAGE:");
    println!("   intent <command> [options]");
    println!();
    println!("COMMANDS:");
    println!("   add                       Add a new intent");
    println!("   list, ls [filter]         List all intents");
    println!("     --planned, -p             Show only planned intents");
    println!("     --active, -a              Show planned + in-progress");
    println!("     --complete, -c            Show completed intents");
    println!("   show <id>                 Show specific intent");
    println!("   <id>                      Shorthand for show");
    println!("   search <term> [filters]   Search intent content");
    println!("     --status <status>         Filter by status");
    println!("     --tag <tag>               Filter by tag");
    println!("   stats                     Show intent statistics");
    println!("   version, --version, -v    Show version");
    println!("   help                      Show this help");
    println!();
    println!("EXAMPLES:");
    println!("   intent list                    # All active intents");
    println!("   intent list --planned          # Only planned");
    println!("   intent list --complete         # Completed intents");
    println!("   intent show 036                # View intent 036");
    println!("   intent 036                     # Same as above");
    println!("   intent search rust             # Search for 'rust'");
    println!("   intent search --tag v7.0       # Find v7.0 intents");
    println!("   intent stats                   # View statistics");
}

fn extract_frontmatter(content: &str, key: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_frontmatter = false;

    for line in lines {
        if line.trim() == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }

        if in_frontmatter && line.starts_with(&format!("{}: ", key)) {
            return Some(
                line.trim_start_matches(&format!("{}: ", key))
                    .trim_matches('"')
                    .to_string(),
            );
        }
    }

    None
}

// ============================================================================
// Intent 052: Auto-Move on Status Change
// ============================================================================

fn cmd_status_change(id: &str, new_status: &str, target_dir: &str) {
    let intent_dir = get_intent_dir();
    
    // Find the intent file across all directories
    let intent_file = find_intent_file(&intent_dir, id);
    if intent_file.is_none() {
        error(&format!("Intent {} not found", id));
    }
    let intent_path = intent_file.unwrap();
    
    // Read current content
    let content = fs::read_to_string(&intent_path)
        .unwrap_or_else(|_| error("Failed to read intent file"));
    
    // Get current status
    let current_status = extract_frontmatter(&content, "status")
        .unwrap_or_else(|| "unknown".to_string());
    
    // Update status in frontmatter
    let updated_content = update_frontmatter(&content, "status", new_status);
    
    // Determine target path
    let filename = intent_path.file_name().unwrap().to_str().unwrap();
    let target_path = intent_dir.join(target_dir).join(filename);
    
    // Show what we're doing
    println!("🔄 Intent Status Change");
    println!("   Intent: {}", id);
    println!("   Status: {} → {}", current_status, new_status);
    println!("   From:   {:?}", intent_path.strip_prefix(&intent_dir).unwrap());
    println!("   To:     {:?}", target_path.strip_prefix(&intent_dir).unwrap());
    println!();
    
    // Write updated content
    fs::write(&intent_path, &updated_content)
        .unwrap_or_else(|_| error("Failed to update intent file"));
    
    // Move file if directory changed
    let current_dir = intent_path.parent().unwrap();
    let target_dir_path = intent_dir.join(target_dir);
    
    if current_dir != target_dir_path {
        // Create target directory if needed
        fs::create_dir_all(&target_dir_path)
            .unwrap_or_else(|_| error("Failed to create target directory"));
        
        // Move file
        fs::rename(&intent_path, &target_path)
            .unwrap_or_else(|_| error("Failed to move intent file"));
        
        println!("✅ Intent moved to {}/", target_dir);
    } else {
        println!("✅ Intent status updated (already in correct directory)");
    }
}

fn find_intent_file(intent_dir: &PathBuf, id: &str) -> Option<PathBuf> {
    // Search in all subdirectories
    let subdirs = vec!["future", "complete", "cancelled", "deferred", 
                       "decisions", "experiments", "philosophy", "incidents"];
    
    for subdir in subdirs {
        let dir_path = intent_dir.join(subdir);
        if !dir_path.exists() {
            continue;
        }
        
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();
                
                // Match by ID at start of filename
                if filename_str.starts_with(&format!("{}-", id)) 
                    || filename_str.starts_with(&format!("0{}-", id))
                    || filename_str.starts_with(&format!("00{}-", id)) {
                    return Some(entry.path());
                }
            }
        }
    }
    None
}

fn update_frontmatter(content: &str, key: &str, value: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::new();
    let mut in_frontmatter = false;
    let mut updated = false;
    
    for line in lines {
        if line.trim() == "---" {
            in_frontmatter = !in_frontmatter;
            result.push_str(line);
            result.push('\n');
            continue;
        }
        
        if in_frontmatter && line.starts_with(&format!("{}: ", key)) {
            result.push_str(&format!("{}: {}\n", key, value));
            updated = true;
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    
    if !updated {
        eprintln!("Warning: Could not find '{}' in frontmatter", key);
    }
    
    result
}
