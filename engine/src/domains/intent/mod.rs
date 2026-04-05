#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
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
    pub depends_on: Vec<String>,
    pub blocks: Vec<String>,
    pub relates: Vec<String>,
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
    let mut depends_on: Vec<String> = vec![];
    let mut blocks_field: Vec<String> = vec![];
    let mut relates: Vec<String> = vec![];

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
        } else if let Some(v) = line.strip_prefix("depends_on:") {
            let s = v.trim().trim_matches('[').trim_matches(']');
            depends_on = s
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
        } else if let Some(v) = line.strip_prefix("blocks:") {
            let s = v.trim().trim_matches('[').trim_matches(']');
            blocks_field = s
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
        } else if let Some(v) = line.strip_prefix("relates:") {
            let s = v.trim().trim_matches('[').trim_matches(']');
            relates = s
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
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
        depends_on,
        blocks: blocks_field,
        relates,
    })
}

pub fn list(ctx: &AppContext, planned: bool, active: bool, complete: bool) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;
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

// ── Phase 2 — Focus + Workflow States ─────────────────────────────────────

fn intent_state_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".local/state/0-core/intent")
}

fn focus_file() -> PathBuf {
    intent_state_dir().join("focus.toml")
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FocusState {
    id: String,
    title: String,
    started: String,
    workflow: String,
}

fn read_focus() -> Option<FocusState> {
    let content = fs::read_to_string(focus_file()).ok()?;
    toml::from_str(&content).ok()
}

fn write_focus(state: &FocusState) -> crate::errors::CoreResult<()> {
    fs::create_dir_all(intent_state_dir()).map_err(crate::errors::CoreError::Io)?;
    let content = toml::to_string_pretty(state)
        .map_err(|e| crate::errors::CoreError::Runtime(e.to_string()))?;
    fs::write(focus_file(), content).map_err(crate::errors::CoreError::Io)?;
    Ok(())
}

fn timestamp_now() -> String {
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S"])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            stdout: b"unknown".to_vec(),
            stderr: vec![],
            status: std::process::ExitStatus::default(),
        });
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn focus(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    let intents = load_all(ctx);
    let intent = intents
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| crate::errors::CoreError::Runtime(format!("Intent {} not found", id)))?;

    if intent.status == "complete" || intent.status == "cancelled" {
        println!(
            "  {} Cannot focus a {} intent",
            "✗".bright_red(),
            intent.status
        );
        return Ok(());
    }

    let state = FocusState {
        id: intent.id.clone(),
        title: intent.title.clone(),
        started: timestamp_now(),
        workflow: intent.status.clone(),
    };

    write_focus(&state)?;

    println!("{}", "🎯 Intent Focused".bold());
    println!("{}", "━".repeat(50).dimmed());
    println!("  {} {}", "ID:    ".dimmed(), intent.id.bright_white());
    println!("  {} {}", "Title: ".dimmed(), intent.title.bright_white());
    println!("  {} {}", "State: ".dimmed(), intent.status_colored());
    println!("{}", "━".repeat(50).dimmed());
    println!(
        "  {} Use {} to check drift",
        "→".dimmed(),
        "core intent drift".bright_cyan()
    );

    Ok(())
}

pub fn unfocus(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    let f = focus_file();
    if !f.exists() {
        println!("  {} No intent currently focused", "○".dimmed());
        return Ok(());
    }

    let current = read_focus();
    fs::remove_file(&f).map_err(crate::errors::CoreError::Io)?;

    if let Some(s) = current {
        println!("  {} Unfocused: {}", "○".dimmed(), s.title.bright_white());
    } else {
        println!("  {} Focus cleared", "○".dimmed());
    }

    Ok(())
}

pub fn focus_status(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;

    match read_focus() {
        None => {
            println!("  {} No intent focused", "○".dimmed());
            println!(
                "  {} Use {} to set focus",
                "→".dimmed(),
                "core intent focus <id>".bright_cyan()
            );
        }
        Some(state) => {
            println!("{}", "🎯 Current Focus".bold());
            println!("{}", "━".repeat(50).dimmed());
            println!("  {} {}", "ID:      ".dimmed(), state.id.bright_white());
            println!("  {} {}", "Title:   ".dimmed(), state.title.bright_white());
            println!(
                "  {} {}",
                "State:   ".dimmed(),
                state.workflow.bright_yellow()
            );
            println!("  {} {}", "Since:   ".dimmed(), state.started.dimmed());
            println!("{}", "━".repeat(50).dimmed());
        }
    }

    Ok(())
}

pub fn drift(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;

    let focus = match read_focus() {
        None => {
            println!(
                "  {} No intent focused — nothing to drift from",
                "○".dimmed()
            );
            println!(
                "  {} Set a focus with {}",
                "→".dimmed(),
                "core intent focus <id>".bright_cyan()
            );
            return Ok(());
        }
        Some(f) => f,
    };

    // Read recent git commits to check drift
    let output = std::process::Command::new("git")
        .args([
            "-C",
            &dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/root"))
                .join("0-core")
                .to_string_lossy()
                .to_string(),
            "log",
            "--oneline",
            "-10",
            "--format=%s",
        ])
        .output()
        .map_err(crate::errors::CoreError::Io)?;

    let commits = String::from_utf8_lossy(&output.stdout);
    let commit_lines: Vec<&str> = commits.lines().collect();

    // Simple drift detection: check if recent commits mention the focused intent
    let id_mentions = commit_lines
        .iter()
        .filter(|c| c.contains(&focus.id) || c.to_lowercase().contains("intent"))
        .count();

    let total = commit_lines.len();

    println!("{}", "🧭 Intent Drift Analysis".bold());
    println!("{}", "━".repeat(50).dimmed());
    println!("  {} {}", "Focus:   ".dimmed(), focus.title.bright_white());
    println!("  {} {}", "ID:      ".dimmed(), focus.id.bright_white());
    println!("{}", "━".repeat(50).dimmed());

    if total == 0 {
        println!("  {} No recent commits to analyze", "○".dimmed());
        return Ok(());
    }

    println!("  {} Last {} commits analyzed", "→".dimmed(), total);

    if id_mentions > 0 {
        println!(
            "  {} {} commit(s) relate to focused intent",
            "✅".green(),
            id_mentions
        );
        println!("  {} Forest is on course", "✓".bright_green());
    } else {
        println!(
            "  {} No recent commits reference focused intent",
            "⚠️ ".bright_yellow()
        );
        println!("  {} Possible drift detected — recent work:", "→".dimmed());
        for commit in commit_lines.iter().take(5) {
            println!("    {} {}", "·".dimmed(), commit.dimmed());
        }
    }

    println!("{}", "━".repeat(50).dimmed());

    Ok(())
}

pub fn start(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    let intents = load_all(ctx);
    let intent = intents
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| crate::errors::CoreError::Runtime(format!("Intent {} not found", id)))?;

    if intent.status != "planned" && intent.status != "future" {
        println!(
            "  {} Intent is already {} — use complete to finish it",
            "⚠️ ".bright_yellow(),
            intent.status
        );
        return Ok(());
    }

    // Auto-checkpoint before transition
    println!("  {} Auto-checkpointing before transition...", "→".dimmed());
    crate::domains::checkpoint::auto(ctx, &format!("intent-{}-start", id))?;

    // Update frontmatter status in file
    let base = PathBuf::from(&ctx.core_root).join("intents");
    let folders = ["future", "planned", "deferred"];
    let mut found_path: Option<PathBuf> = None;

    for folder in &folders {
        let dir = base.join(folder);
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(&format!("{:0>3}", id)) || n.starts_with(id))
                    .unwrap_or(false)
                {
                    found_path = Some(path);
                    break;
                }
            }
        }
        if found_path.is_some() {
            break;
        }
    }

    if let Some(path) = found_path {
        let content = fs::read_to_string(&path).map_err(crate::errors::CoreError::Io)?;
        let updated = content
            .replace("status: planned", "status: in-progress")
            .replace("status: future", "status: in-progress")
            .replace("status: deferred", "status: in-progress");
        fs::write(&path, updated).map_err(crate::errors::CoreError::Io)?;
    }

    // Set as focused intent
    let state = FocusState {
        id: intent.id.clone(),
        title: intent.title.clone(),
        started: timestamp_now(),
        workflow: "in-progress".to_string(),
    };
    write_focus(&state)?;

    println!("{}", "🚀 Intent Started".bold());
    println!("{}", "━".repeat(50).dimmed());
    println!("  {} {}", "ID:    ".dimmed(), intent.id.bright_white());
    println!("  {} {}", "Title: ".dimmed(), intent.title.bright_white());
    println!(
        "  {} planned → {}",
        "State: ".dimmed(),
        "in-progress".bright_yellow()
    );
    println!("  {} Intent is now focused", "🎯".green());
    println!("{}", "━".repeat(50).dimmed());

    Ok(())
}

pub fn complete_intent(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    let intents = load_all(ctx);
    let intent = intents
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| crate::errors::CoreError::Runtime(format!("Intent {} not found", id)))?;

    if intent.status == "complete" {
        println!("  {} Intent {} is already complete", "✅".green(), id);
        return Ok(());
    }

    // Auto-checkpoint before completion
    println!("  {} Auto-checkpointing before completion...", "→".dimmed());
    crate::domains::checkpoint::auto(ctx, &format!("intent-{}-complete", id))?;

    // Find and move the file to complete/
    let base = PathBuf::from(&ctx.core_root).join("intents");
    let folders = ["future", "planned", "deferred", "in-progress"];
    let mut found_path: Option<PathBuf> = None;

    for folder in &folders {
        let dir = base.join(folder);
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(&format!("{:0>3}", id)) || n.starts_with(id))
                    .unwrap_or(false)
                {
                    found_path = Some(path);
                    break;
                }
            }
        }
        if found_path.is_some() {
            break;
        }
    }

    if let Some(src_path) = found_path {
        // Update status in frontmatter
        let content = fs::read_to_string(&src_path).map_err(crate::errors::CoreError::Io)?;
        let updated = content
            .replace("status: planned", "status: complete")
            .replace("status: future", "status: complete")
            .replace("status: in-progress", "status: complete")
            .replace("status: deferred", "status: complete");

        // Move to complete/
        let complete_dir = base.join("complete");
        fs::create_dir_all(&complete_dir).map_err(crate::errors::CoreError::Io)?;
        let filename = src_path.file_name().unwrap();
        let dest_path = complete_dir.join(filename);
        fs::write(&dest_path, updated).map_err(crate::errors::CoreError::Io)?;
        fs::remove_file(&src_path).map_err(crate::errors::CoreError::Io)?;
    }

    // Clear focus if this was the focused intent
    if let Some(f) = read_focus() {
        if f.id == id {
            let _ = fs::remove_file(focus_file());
        }
    }

    println!("{}", "✅ Intent Complete".bold());
    println!("{}", "━".repeat(50).dimmed());
    println!("  {} {}", "ID:    ".dimmed(), intent.id.bright_white());
    println!("  {} {}", "Title: ".dimmed(), intent.title.bright_white());
    println!("  {} → {}", "State: ".dimmed(), "complete".bright_green());
    println!("  {} Moved to intents/complete/", "📁".dimmed());
    println!("{}", "━".repeat(50).dimmed());

    Ok(())
}

pub fn new_intent(ctx: &AppContext, template: &str, title: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    // Find next ID
    let intents = load_all(ctx);
    let max_id = intents
        .iter()
        .filter_map(|i| i.id.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    let next_id = max_id + 1;

    let today = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "2026-01-01".to_string());

    let (type_tag, tags) = match template {
        "feature" => ("feature", "feature, rust, faelight"),
        "fix" => ("fix", "fix, bugfix"),
        "arch" => ("arch", "architecture, rust, design"),
        "study" => ("study", "study, research, learning"),
        _ => ("future", "faelight"),
    };

    let slug = title.to_lowercase().replace(' ', "-");
    let filename = format!("{:0>3}-{}.md", next_id, slug);
    let filepath = PathBuf::from(&ctx.core_root)
        .join("intents/future")
        .join(&filename);

    let content = format!(
        r#"---
id: {:03}
date: {}
type: {}
title: "{}"
status: planned
tags: [{}]
version: TBD
---

## Vision

<!-- What is this intent trying to achieve? -->

## Why Now

<!-- Why is this the right time for this intent? -->

## Approach

<!-- How will this be implemented? -->

## Success Criteria

- [ ] <!-- First criterion -->
- [ ] <!-- Second criterion -->

## Gate Check
```
⬜ Not started
```

---

*"The forest grows with intention."* 🌲
"#,
        next_id, today, type_tag, title, tags
    );

    fs::write(&filepath, content).map_err(crate::errors::CoreError::Io)?;

    println!("{}", "📝 Intent Created".bold());
    println!("{}", "━".repeat(50).dimmed());
    println!("  {} {:03}", "ID:       ".dimmed(), next_id);
    println!("  {} {}", "Title:    ".dimmed(), title.bright_white());
    println!("  {} {}", "Template: ".dimmed(), template.bright_cyan());
    println!("  {} {}", "File:     ".dimmed(), filename.dimmed());
    println!("{}", "━".repeat(50).dimmed());
    println!(
        "  {} Edit: {}",
        "→".dimmed(),
        filepath.display().to_string().bright_cyan()
    );

    Ok(())
}

pub fn deps(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;

    let intents = load_all(ctx);
    let intent = intents
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| crate::errors::CoreError::Runtime(format!("Intent {} not found", id)))?;

    println!("{}", "🔗 Intent Dependencies".bold());
    println!("{}", "━".repeat(55).dimmed());
    println!(
        "  {} {} — {}",
        intent.id.bright_white(),
        intent.status_colored(),
        intent.title.bright_white()
    );
    println!("{}", "━".repeat(55).dimmed());

    if intent.depends_on.is_empty() && intent.blocks.is_empty() && intent.relates.is_empty() {
        println!("  {} No dependencies declared", "○".dimmed());
        println!(
            "  {} Add to frontmatter: depends_on: [052], blocks: [110], relates: [099]",
            "→".dimmed()
        );
        return Ok(());
    }

    if !intent.depends_on.is_empty() {
        println!("  {} Depends on:", "⬆".bright_yellow());
        for dep_id in &intent.depends_on {
            if let Some(dep) = intents.iter().find(|i| &i.id == dep_id) {
                let ready = dep.status == "complete";
                let icon = if ready {
                    "✅".to_string()
                } else {
                    "⬜".to_string()
                };
                println!(
                    "    {} {} {} — {}",
                    icon,
                    dep.id.bright_white(),
                    dep.status_colored(),
                    dep.title.dimmed()
                );
            } else {
                println!("    {} {} — not found", "?".dimmed(), dep_id.bright_red());
            }
        }
    }

    if !intent.blocks.is_empty() {
        println!("  {} Blocks:", "⬇".bright_red());
        for bid in &intent.blocks {
            if let Some(b) = intents.iter().find(|i| &i.id == bid) {
                println!(
                    "    {} {} {} — {}",
                    "🔒".dimmed(),
                    b.id.bright_white(),
                    b.status_colored(),
                    b.title.dimmed()
                );
            } else {
                println!("    {} {} — not found", "?".dimmed(), bid.bright_red());
            }
        }
    }

    if !intent.relates.is_empty() {
        println!("  {} Relates to:", "↔".bright_cyan());
        for rid in &intent.relates {
            if let Some(r) = intents.iter().find(|i| &i.id == rid) {
                println!(
                    "    {} {} {} — {}",
                    "◦".dimmed(),
                    r.id.bright_white(),
                    r.status_colored(),
                    r.title.dimmed()
                );
            } else {
                println!("    {} {} — not found", "?".dimmed(), rid.bright_red());
            }
        }
    }

    println!("{}", "━".repeat(55).dimmed());
    Ok(())
}

pub fn burndown(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;

    let intents = load_all(ctx);

    // Group completions by month
    let mut monthly: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for intent in intents.iter().filter(|i| i.status == "complete") {
        if intent.date.len() >= 7 {
            let month = intent.date[..7].to_string();
            *monthly.entry(month).or_insert(0) += 1;
        }
    }

    let total = intents.len();
    let complete = intents.iter().filter(|i| i.status == "complete").count();
    let remaining = total - complete;

    println!("{}", "📉 Intent Burndown".bold());
    println!("{}", "━".repeat(55).dimmed());
    println!(
        "  {} {}  {} {}  {} {}",
        "Total:".dimmed(),
        total.to_string().bright_white(),
        "Complete:".dimmed(),
        complete.to_string().bright_green(),
        "Remaining:".dimmed(),
        remaining.to_string().bright_yellow(),
    );
    println!("{}", "━".repeat(55).dimmed());

    if monthly.is_empty() {
        println!("  {} No dated completions found", "○".dimmed());
        return Ok(());
    }

    let max_count = *monthly.values().max().unwrap_or(&1);
    let bar_width = 30usize;

    for (month, count) in monthly
        .iter()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .iter()
        .rev()
    {
        let filled = (*count * bar_width) / max_count.max(1);
        let bar = format!(
            "{}{}",
            "█".repeat(filled).bright_green(),
            "░".repeat(bar_width - filled).dimmed()
        );
        println!(
            "  {} {} {}",
            month.bright_white(),
            bar,
            count.to_string().bright_green(),
        );
    }

    println!("{}", "━".repeat(55).dimmed());

    // Running total
    let mut running = 0usize;
    println!("  {} Completion milestones:", "→".dimmed());
    for (month, count) in &monthly {
        running += count;
        if running % 10 == 0 || running == complete {
            println!(
                "    {} {} — {} total",
                "●".bright_cyan(),
                month.dimmed(),
                running.to_string().bright_white()
            );
        }
    }

    Ok(())
}

pub fn velocity(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;

    let intents = load_all(ctx);

    let mut monthly: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for intent in intents.iter().filter(|i| i.status == "complete") {
        if intent.date.len() >= 7 {
            let month = intent.date[..7].to_string();
            *monthly.entry(month).or_insert(0) += 1;
        }
    }

    let counts: Vec<usize> = monthly.values().cloned().collect();
    let total_months = counts.len();

    println!("{}", "⚡ Intent Velocity".bold());
    println!("{}", "━".repeat(55).dimmed());

    if total_months == 0 {
        println!("  {} No completion data available", "○".dimmed());
        return Ok(());
    }

    let avg = counts.iter().sum::<usize>() as f64 / total_months as f64;
    let max = *counts.iter().max().unwrap_or(&0);
    let min = *counts.iter().min().unwrap_or(&0);

    // Last 3 months trend
    let recent: Vec<usize> = monthly.values().rev().take(3).cloned().collect();
    let recent_avg = if !recent.is_empty() {
        recent.iter().sum::<usize>() as f64 / recent.len() as f64
    } else {
        0.0
    };

    println!(
        "  {} {:.1} intents/month",
        "Average velocity:    ".dimmed(),
        avg
    );
    println!(
        "  {} {:.1} intents/month (last 3)",
        "Recent velocity:     ".dimmed(),
        recent_avg
    );
    println!(
        "  {} {}",
        "Peak month:          ".dimmed(),
        max.to_string().bright_green()
    );
    println!(
        "  {} {}",
        "Slowest month:       ".dimmed(),
        min.to_string().dimmed()
    );
    println!(
        "  {} {}",
        "Months tracked:      ".dimmed(),
        total_months.to_string().bright_white()
    );
    println!("{}", "━".repeat(55).dimmed());

    // Trend direction
    if recent.len() >= 2 {
        let trend = recent[0] as f64 - recent[recent.len() - 1] as f64;
        if trend > 0.5 {
            println!("  {} Velocity accelerating", "📈".green());
        } else if trend < -0.5 {
            println!(
                "  {} Velocity slowing — check for blockers",
                "📉".bright_yellow()
            );
        } else {
            println!("  {} Velocity stable", "→".dimmed());
        }
    }

    // By type
    println!("{}", "━".repeat(55).dimmed());
    println!("  {} Completions by type:", "→".dimmed());
    let mut by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for intent in intents.iter().filter(|i| i.status == "complete") {
        *by_type.entry(intent.intent_type.clone()).or_insert(0) += 1;
    }
    let mut by_type_vec: Vec<(String, usize)> = by_type.into_iter().collect();
    by_type_vec.sort_by(|a, b| b.1.cmp(&a.1));
    for (t, count) in &by_type_vec {
        println!(
            "    {} {:<15} {}",
            "◦".dimmed(),
            t.bright_white(),
            count.to_string().bright_green()
        );
    }

    Ok(())
}

pub fn edit(ctx: &AppContext, id: &str) -> CoreResult<()> {
    let all = load_all(ctx);
    let id_norm = id.trim_start_matches("INT-").trim_start_matches("int-");
    let intent = all.iter().find(|i| {
        i.id.trim_start_matches("INT-").trim_start_matches("int-") == id_norm
    });
    match intent {
        Some(i) => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
            let path = intents_dir(ctx).join(&i.folder).join(&i.filename);
            let status = std::process::Command::new(&editor)
                .arg(&path)
                .status()
                .map_err(|e| crate::errors::CoreError::Io(e))?;
            if status.success() {
                println!("  {} INT-{} saved", "✅".to_string(), i.id);
            }
            Ok(())
        }
        None => {
            eprintln!("  ❌ Intent {} not found", id);
            Ok(())
        }
    }
}

pub fn branch(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;

    let intents = load_all(ctx);
    let intent = intents
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| crate::errors::CoreError::Runtime(format!("Intent {} not found", id)))?;

    let slug = intent
        .title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>();

    // Collapse multiple dashes
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    let branch_name = format!("intent-{}-{}", id, &slug[..slug.len().min(40)]);

    println!("{}", "🌿 Intent Branch".bold());
    println!("{}", "━".repeat(55).dimmed());
    println!("  {} {}", "Intent: ".dimmed(), intent.title.bright_white());
    println!("  {} {}", "Branch: ".dimmed(), branch_name.bright_cyan());
    println!("{}", "━".repeat(55).dimmed());
    println!(
        "  {} git checkout -b {}",
        "→".dimmed(),
        branch_name.bright_cyan()
    );

    Ok(())
}

pub fn health(ctx: &AppContext, stale_only: bool) -> CoreResult<()> {
    use colored::*;
    let root = std::path::PathBuf::from(&ctx.core_root);
    let future_dir = root.join("intents/future");
    let now = chrono::Utc::now().timestamp();
    let mut intents: Vec<(String, String, String, f64, Vec<String>)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&future_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".md") { continue; }
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if !content.contains("status: in-progress") { continue; }
            let id = name.split('-').next().unwrap_or("?").to_string();
            let title = content.lines()
                .find(|l| l.starts_with("title:"))
                .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
                .unwrap_or_else(|| name.clone());
            let total_gates = content.matches("⬜").count() + content.matches("✅").count();
            let done_gates = content.matches("✅").count();
            let gate_pct = if total_gates > 0 { (done_gates * 100) / total_gates } else { 0 };
            let days_since: i64 = {
                let out = std::process::Command::new("git")
                    .args(["-C", &ctx.core_root, "log", "-1", "--format=%ct",
                           "--", &format!("intents/future/{}", name)])
                    .output().ok();
                let ts = out.as_ref()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<i64>().unwrap_or(0))
                    .unwrap_or(0);
                if ts > 0 { (now - ts) / 86400 } else { 0 }
            };
            let mut score = gate_pct as f64;
            let mut issues: Vec<String> = Vec::new();
            if days_since > 14 {
                score -= 20.0;
                issues.push(format!("stalled {} days", days_since));
            }
            score = score.max(0.0).min(100.0);
            if !stale_only || days_since > 14 {
                intents.push((id, title, format!("{}/{}", done_gates, total_gates), score, issues));
            }
        }
    }
    println!();
    println!("  {} Intent Health", "💊");
    println!("  {}", "─".repeat(56).dimmed());
    if intents.is_empty() {
        println!("  {} All active intents healthy", "✅");
        println!();
        return Ok(());
    }
    intents.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
    for (id, title, gates, score, issues) in &intents {
        let s = if *score >= 70.0 { format!("{:.0}%", score).bright_green().to_string() }
                else if *score >= 40.0 { format!("{:.0}%", score).yellow().to_string() }
                else { format!("{:.0}%", score).bright_red().to_string() };
        println!("  INT-{:<6} {:<36} {} gates  {}", id.bright_white(), title.dimmed(), gates, s);
        for issue in issues {
            println!("    ⚠  {}", issue.bright_yellow());
        }
    }
    println!();
    Ok(())
}

pub fn predict_completion(ctx: &AppContext, id: &str) -> CoreResult<()> {
    use colored::*;
    let root = std::path::PathBuf::from(&ctx.core_root);
    
    // Find the intent file
    let mut intent_content = None;
    let mut intent_name = String::new();
    for dir in &["intents/future", "intents/complete"] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(id) && name.ends_with(".md") {
                    intent_content = std::fs::read_to_string(entry.path()).ok();
                    intent_name = name;
                    break;
                }
            }
        }
        if intent_content.is_some() { break; }
    }
    let content = match intent_content {
        None => { println!("  {} Intent {} not found", "✗".bright_red(), id); return Ok(()); }
        Some(c) => c,
    };
    let title = content.lines()
        .find(|l| l.starts_with("title:"))
        .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
        .unwrap_or_else(|| intent_name.clone());
    let total_gates = content.matches("⬜").count() + content.matches("✅").count();
    let done_gates = content.matches("✅").count();
    let remaining = total_gates.saturating_sub(done_gates);
    // Get average gates completed per session from recent intents
    let complete_dir = root.join("intents/complete");
    let mut gates_per_session: Vec<f64> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&complete_dir) {
        for entry in entries.flatten().take(10) {
            let c = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let total = c.matches("✅").count();
            if total > 0 { gates_per_session.push(total as f64 / 2.0); } // assume ~2 sessions avg
        }
    }
    let avg_gates_per_session = if gates_per_session.is_empty() { 3.0 }
        else { gates_per_session.iter().sum::<f64>() / gates_per_session.len() as f64 };
    let sessions_needed = if avg_gates_per_session > 0.0 {
        (remaining as f64 / avg_gates_per_session).ceil() as usize
    } else { remaining };
    println!();
    println!("  {} Completion Prediction: INT-{}", "🎯".normal(), id);
    println!("  {}", "─".repeat(56).dimmed());
    println!("  {:<24} {}", "Intent:".dimmed(), title.bright_white());
    println!("  {:<24} {}/{} gates done ({}%)",
        "Gates:".dimmed(), done_gates, total_gates,
        if total_gates > 0 { (done_gates * 100) / total_gates } else { 0 });
    println!("  {:<24} {} gates remaining", "Remaining:".dimmed(), remaining.to_string().bright_yellow());
    println!("  {:<24} {:.1} gates/session (from {} recent intents)",
        "Velocity:".dimmed(), avg_gates_per_session, gates_per_session.len());
    println!("  {:<24} ~{} session(s)",
        "Estimated:".dimmed(), sessions_needed.to_string().bright_cyan());
    if sessions_needed == 0 {
        println!("  {} All gates satisfied — ready to cicomplete!", "✅".normal());
    } else if sessions_needed == 1 {
        println!("  {} One focused session should complete this intent", "→".bright_cyan());
    } else {
        println!("  {} {} sessions estimated at current velocity", "→".bright_cyan(), sessions_needed);
    }
    println!();
    Ok(())
}
pub fn auto_link(ctx: &AppContext, id: &str) -> CoreResult<()> {
    use colored::*;
    let root = std::path::PathBuf::from(&ctx.core_root);
    // Find source intent
    let mut source_content = String::new();
    let mut source_title = String::new();
    for dir in &["intents/future", "intents/complete"] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(id) && name.ends_with(".md") {
                    source_content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                    source_title = source_content.lines()
                        .find(|l| l.starts_with("title:"))
                        .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
                        .unwrap_or_default();
                    break;
                }
            }
        }
    }
    if source_title.is_empty() {
        println!("  {} Intent {} not found", "✗".bright_red(), id);
        return Ok(());
    }
    // Extract tags from source
    let source_tags: Vec<String> = source_content.lines()
        .find(|l| l.starts_with("tags:"))
        .map(|l| l.to_lowercase())
        .unwrap_or_default()
        .split(',')
        .map(|t| t.trim().trim_matches('[').trim_matches(']').trim_matches('"').to_string())
        .filter(|t| !t.is_empty())
        .collect();
    // Find related intents by tag overlap
    let mut related: Vec<(String, String, usize)> = Vec::new(); // (id, title, overlap)
    for dir in &["intents/future", "intents/complete"] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".md") || name.starts_with(id) { continue; }
                let c = std::fs::read_to_string(entry.path()).unwrap_or_default();
                let other_tags: Vec<String> = c.lines()
                    .find(|l| l.starts_with("tags:"))
                    .map(|l| l.to_lowercase())
                    .unwrap_or_default()
                    .split(',')
                    .map(|t| t.trim().trim_matches('[').trim_matches(']').trim_matches('"').to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
                let overlap = source_tags.iter().filter(|t| other_tags.contains(t)).count();
                if overlap >= 2 {
                    let other_id = name.split('-').next().unwrap_or("?").to_string();
                    let other_title = c.lines()
                        .find(|l| l.starts_with("title:"))
                        .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
                        .unwrap_or_default();
                    related.push((other_id, other_title, overlap));
                }
            }
        }
    }
    related.sort_by(|a, b| b.2.cmp(&a.2));
    println!();
    println!("  {} Auto-Link Analysis: INT-{}", "🔗".normal(), id);
    println!("  {}", "─".repeat(56).dimmed());
    println!("  Source: {}", source_title.bright_white());
    println!("  Tags:   {}", source_tags.join(", ").dimmed());
    println!();
    if related.is_empty() {
        println!("  {} No strongly related intents found", "○".dimmed());
    } else {
        println!("  {} Related intents (by tag overlap):", "▶".bright_cyan());
        for (rid, rtitle, overlap) in related.iter().take(5) {
            println!("    {} INT-{:<6} {} ({} shared tags)",
                "·".dimmed(), rid.bright_white(), rtitle.dimmed(), overlap.to_string().bright_cyan());
        }
    }
    println!();
    Ok(())
}

pub fn story(ctx: &AppContext, id: &str) -> CoreResult<()> {
    use colored::*;
    let root = std::path::PathBuf::from(&ctx.core_root);
    // Find intent in any directory
    let mut intent_content = None;
    let mut intent_path = std::path::PathBuf::new();
    for dir in &["intents/complete", "intents/future", "intents/archive"] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(id) && name.ends_with(".md") {
                    intent_path = entry.path();
                    intent_content = std::fs::read_to_string(&intent_path).ok();
                    break;
                }
            }
        }
        if intent_content.is_some() { break; }
    }
    let content = match intent_content {
        None => { println!("  {} Intent {} not found", "✗".bright_red(), id); return Ok(()); }
        Some(c) => c,
    };
    let title = content.lines()
        .find(|l| l.starts_with("title:"))
        .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
        .unwrap_or_default();
    let date = content.lines()
        .find(|l| l.starts_with("date:"))
        .map(|l| l.trim_start_matches("date:").trim().to_string())
        .unwrap_or_default();
    let status = content.lines()
        .find(|l| l.starts_with("status:"))
        .map(|l| l.trim_start_matches("status:").trim().to_string())
        .unwrap_or_default();
    let tags: Vec<String> = content.lines()
        .find(|l| l.starts_with("tags:"))
        .map(|l| l.trim_start_matches("tags:").trim()
            .trim_matches('[').trim_matches(']')
            .split(',')
            .map(|t| t.trim().trim_matches('"').to_string())
            .filter(|t| !t.is_empty())
            .collect())
        .unwrap_or_default();
    let total_gates = content.matches("⬜").count() + content.matches("✅").count();
    let done_gates = content.matches("✅").count();
    // Get git commits touching this intent
    let commit_log = std::process::Command::new("git")
        .args(["-C", &ctx.core_root, "log", "--oneline",
               "--", &format!("intents/**/*{}*", id)])
        .output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let commit_count = commit_log.lines().count();
    // Find related intents
    let mut related_ids: Vec<String> = Vec::new();
    for dir in &["intents/complete", "intents/future"] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".md") || name.starts_with(id) { continue; }
                let c = std::fs::read_to_string(entry.path()).unwrap_or_default();
                // Check if this intent references our id
                if c.contains(&format!("INT-{}", id)) {
                    let rid = name.split('-').next().unwrap_or("?").to_string();
                    related_ids.push(rid);
                }
            }
        }
    }
    println!();
    println!("  {} INT-{} — {}", "📖".normal(), id.bright_white(), title.bright_white());
    println!("  {}", "═".repeat(60).dimmed());
    println!();
    println!("  {:<16} {}", "Date:".dimmed(), date.bright_cyan());
    println!("  {:<16} {}", "Status:".dimmed(),
        if status == "complete" { "✅ complete".bright_green().to_string() }
        else { status.bright_yellow().to_string() });
    println!("  {:<16} {}/{} gates", "Gates:".dimmed(), done_gates, total_gates);
    println!("  {:<16} {}", "Tags:".dimmed(), tags.join(", ").dimmed());
    if commit_count > 0 {
        println!("  {:<16} {} commits touch this intent", "History:".dimmed(), commit_count.to_string().bright_cyan());
    }
    println!();
    println!("  {} What this intent built:", "▶".bright_cyan());
    // Extract completed gates as the story
    let completed: Vec<String> = content.lines()
        .filter(|l| l.contains("✅"))
        .map(|l| l.trim().trim_start_matches("✅").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    for item in &completed {
        println!("    {} {}", "✅".normal(), item.bright_white());
    }
    if !related_ids.is_empty() {
        println!();
        println!("  {} Referenced by:", "▶".bright_cyan());
        for rid in &related_ids {
            println!("    {} INT-{}", "·".dimmed(), rid.bright_white());
        }
    }
    // Show the phrase if it exists
    if let Some(phrase_start) = content.find("**"") {
        let phrase_end = content[phrase_start+3..].find(""**")
            .map(|i| phrase_start + 3 + i)
            .unwrap_or(content.len());
        let phrase = &content[phrase_start+3..phrase_end];
        if phrase.len() < 300 {
            println!();
            println!("  {}", "─".repeat(60).dimmed());
            for line in phrase.lines() {
                println!("  {}", line.bright_white().italic());
            }
        }
    }
    println!();
    Ok(())
}
