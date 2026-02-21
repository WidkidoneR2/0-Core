#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Intent {
    pub id: String,
    pub title: String,
    pub status: String,
    pub date: String,
    pub tags: Vec<String>,
    pub intent_type: String,
    pub folder: String,
    pub filename: String,
}

impl Intent {
    pub fn status_colored(&self) -> String {
        match self.status.as_str() {
            "complete" => format!("[{}]", "complete".bright_green()),
            "in-progress" => format!("[{}]", "in-progress".bright_yellow()),
            "planned" => format!("[{}]", "planned".bright_blue()),
            "cancelled" => format!("[{}]", "cancelled".dimmed()),
            "deferred" => format!("[{}]", "deferred".dimmed()),
            other => format!("[{}]", other.white()),
        }
    }
}

fn intents_dir(ctx: &AppContext) -> PathBuf {
    PathBuf::from(&ctx.core_root).join("intents")
}

fn load_all(ctx: &AppContext) -> Vec<Intent> {
    let base = intents_dir(ctx);
    let folders = [
        "complete",
        "future",
        "cancelled",
        "deferred",
        "decisions",
        "experiments",
        "incidents",
        "philosophy",
    ];
    let mut intents = vec![];

    for folder in &folders {
        let dir = base.join(folder);
        if !dir.exists() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Some(intent) = parse_intent(&path, folder) {
                intents.push(intent);
            }
        }
    }

    intents.sort_by(|a, b| a.id.cmp(&b.id));
    intents
}

fn parse_intent(path: &Path, folder: &str) -> Option<Intent> {
    let content = fs::read_to_string(path).ok()?;
    let filename = path.file_name()?.to_string_lossy().to_string();

    // Parse YAML frontmatter between --- delimiters
    let _stripped = content
        .trim_start_matches("---\n")
        .trim_start_matches("---\r\n");
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 2 {
        return None;
    }
    let frontmatter = parts[1];

    let mut id = String::new();
    let mut title = String::new();
    let mut status = String::new();
    let mut date = String::new();
    let mut tags: Vec<String> = vec![];
    let mut intent_type = folder.to_string();

    for line in frontmatter.lines() {
        if let Some(v) = line.strip_prefix("id:") {
            id = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("title:") {
            title = v.trim().trim_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("status:") {
            status = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("date:") {
            date = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("type:") {
            intent_type = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("tags:") {
            let tag_str = v.trim().trim_matches('[').trim_matches(']');
            tags = tag_str.split(',').map(|t| t.trim().to_string()).collect();
        }
    }

    if id.is_empty() || title.is_empty() {
        return None;
    }
    if status.is_empty() {
        status = folder.to_string();
    }

    Some(Intent {
        id,
        title,
        status,
        date,
        tags,
        intent_type,
        folder: folder.to_string(),
        filename,
    })
}

pub fn list(ctx: &AppContext, planned: bool, active: bool, complete: bool) -> CoreResult<()> {
    let intents = load_all(ctx);

    let filtered: Vec<&Intent> = intents
        .iter()
        .filter(|i| {
            if complete {
                return i.status == "complete";
            }
            if planned {
                return i.status == "planned";
            }
            if active {
                return i.status == "planned" || i.status == "in-progress";
            }
            i.status != "complete" && i.status != "cancelled"
        })
        .collect();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "📋 Intent Ledger".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let mut by_folder: HashMap<String, Vec<&Intent>> = HashMap::new();
    for i in &filtered {
        by_folder.entry(i.folder.clone()).or_default().push(i);
    }

    let folder_order = [
        "future",
        "decisions",
        "experiments",
        "incidents",
        "philosophy",
        "deferred",
        "cancelled",
        "complete",
    ];
    for folder in &folder_order {
        let Some(items) = by_folder.get(*folder) else {
            continue;
        };
        println!("{}:", folder.bright_white().bold());
        for i in items {
            let id_display = format!("{:>3}", i.id).bright_cyan();
            println!("  {}  {} {}", id_display, i.status_colored(), i.title);
        }
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  Total: {}", filtered.len());
    Ok(())
}

pub fn stats(ctx: &AppContext) -> CoreResult<()> {
    let intents = load_all(ctx);
    let total = intents.len();
    let complete = intents.iter().filter(|i| i.status == "complete").count();
    let planned = intents.iter().filter(|i| i.status == "planned").count();
    let in_progress = intents.iter().filter(|i| i.status == "in-progress").count();
    let cancelled = intents.iter().filter(|i| i.status == "cancelled").count();
    let deferred = intents.iter().filter(|i| i.status == "deferred").count();

    let pct = if total > 0 {
        (complete * 100) / total
    } else {
        0
    };
    let bar_filled = (pct / 5) as usize;
    let bar = format!(
        "{}{}",
        "█".repeat(bar_filled).bright_green(),
        "░".repeat(20 - bar_filled).dimmed()
    );

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "📊 Intent Statistics".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  Total:       {}", total.to_string().bright_white());
    println!(
        "  Complete:    {} ({}%)",
        complete.to_string().bright_green(),
        pct
    );
    println!("  In-progress: {}", in_progress.to_string().bright_yellow());
    println!("  Planned:     {}", planned.to_string().bright_blue());
    println!("  Deferred:    {}", deferred.to_string().dimmed());
    println!("  Cancelled:   {}", cancelled.to_string().dimmed());
    println!("  Progress: [{}] {}%", bar, pct);
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    Ok(())
}

pub fn show(ctx: &AppContext, id: &str) -> CoreResult<()> {
    let intents = load_all(ctx);
    let found: Vec<&Intent> = intents.iter().filter(|i| i.id == id).collect();

    if found.is_empty() {
        println!("  {} Intent {} not found", "✗".bright_red(), id);
        return Ok(());
    }

    let intent = found[0];
    let base = intents_dir(ctx);
    let path = base.join(&intent.folder).join(&intent.filename);
    let content = fs::read_to_string(&path).unwrap_or_default();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!(
        "{} {} — {}",
        "📋".bold(),
        intent.id.bright_cyan(),
        intent.title.bright_white().bold()
    );
    println!(
        "  Status: {}  Date: {}",
        intent.status_colored(),
        intent.date.dimmed()
    );
    if !intent.tags.is_empty() {
        println!("  Tags: {}", intent.tags.join(", ").dimmed());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    // Print body (skip frontmatter)
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() >= 3 {
        println!("{}", parts[2].trim());
    }
    Ok(())
}

pub fn search(ctx: &AppContext, term: &str) -> CoreResult<()> {
    let intents = load_all(ctx);
    let term_lower = term.to_lowercase();

    let matches: Vec<&Intent> = intents
        .iter()
        .filter(|i| {
            i.title.to_lowercase().contains(&term_lower)
                || i.tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&term_lower))
        })
        .collect();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("🔍 Search: {}", term.bright_white().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if matches.is_empty() {
        println!("  {} No results for '{}'", "○".dimmed(), term);
    } else {
        for i in &matches {
            println!(
                "  {} {} {}",
                i.id.bright_cyan(),
                i.status_colored(),
                i.title
            );
        }
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        println!("  {} results", matches.len());
    }
    Ok(())
}

pub fn validate(ctx: &AppContext) -> CoreResult<()> {
    let intents = load_all(ctx);
    let mut issues = 0;

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔍 Intent Ledger Validation".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    // Check for duplicate IDs
    let mut seen_ids: HashMap<String, usize> = HashMap::new();
    for i in &intents {
        *seen_ids.entry(i.id.clone()).or_insert(0) += 1;
    }
    for (id, count) in &seen_ids {
        if *count > 1 {
            println!("  {} Duplicate ID: {}", "✗".bright_red(), id);
            issues += 1;
        }
    }

    if issues == 0 {
        println!("  {} All {} intents valid", "✅".green(), intents.len());
    } else {
        println!("  {} {} issues found", "✗".bright_red(), issues);
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    Ok(())
}
