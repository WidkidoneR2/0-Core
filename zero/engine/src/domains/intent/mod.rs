#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// INT-266: findings -- one record per file under intents/findings/.
pub mod findings;
pub mod trace;

#[derive(Debug, Clone)]
pub struct Intent {
    pub id: String,
    pub title: String,
    pub status: String,
    pub date: String,
    pub tags: Vec<String>,
    pub intent_type: String,
    /// INT-213 G10: declared in frontmatter, lowercased on read. Empty means undeclared,
    /// which is different from low -- an undeclared intent has simply never been ranked.
    pub priority: String,
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
    ctx.fpath("intents")
}

/// INT-213 G4 -- the ONE place that decides what satisfies a dependency.
///
/// Contract recorded in the intent before this code existed:
///   complete   -> satisfies
///   cancelled  -> satisfies, but the dependent is flagged as questionable
///   planned / in-progress / deferred -> does not satisfy
///   missing id -> an error, never a permanent block
///
/// Cancellation removes the blocking condition without retroactively making
/// the dependency assumption true. Proven by INT-223 depending on INT-175.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DepState {
    Satisfied,
    Questionable,
    Blocked,
    Missing,
}

impl DepState {
    /// Does this dependency stop the dependent from starting?
    pub fn clears(self) -> bool {
        matches!(self, DepState::Satisfied | DepState::Questionable)
    }
}

/// Shown against any dependent whose dependency was cancelled.
pub const CANCELLED_DEP_NOTE: &str =
    "depends on a cancelled intent -- the assumption behind this edge may no longer hold";

/// Resolve one dependency id against the loaded intent set.
pub fn dep_state(intents: &[Intent], dep_id: &str) -> DepState {
    match intents.iter().find(|i| i.id == dep_id) {
        None => DepState::Missing,
        Some(dep) => match dep.status.as_str() {
            "complete" => DepState::Satisfied,
            "cancelled" => DepState::Questionable,
            _ => DepState::Blocked,
        },
    }
}

/// INT-279 G10: the intents in progress, as INT-NNN, asked of the one loader that core intent
/// list reads. The doctor used to scan intents/future/ for the status line itself, and cistart
/// moves an intent to intents/in-progress/, so that scan never found one.
pub(crate) fn in_progress_ids(ctx: &AppContext) -> Vec<String> {
    load_all(ctx)
        .into_iter()
        .filter(|i| i.status == "in-progress")
        .map(|i| format!("INT-{}", i.id.trim_start_matches("INT-")))
        .collect()
}

fn load_all(ctx: &AppContext) -> Vec<Intent> {
    let base = intents_dir(ctx);
    let folders = [
        "complete",
        "future",
        "in-progress",
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
    // INT-213 G10: priority was written into 72 intents and read by nothing.
    let mut priority = String::new();
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
        } else if let Some(v) = line.strip_prefix("priority:") {
            priority = v.trim().to_lowercase();
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
        priority,
        depends_on,
        blocks: blocks_field,
        relates,
    })
}

pub fn list(
    ctx: &AppContext,
    planned: bool,
    active: bool,
    complete: bool,
    all: bool,
    cancelled: bool,
) -> CoreResult<()> {
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
                return i.status == "in-progress";
            }
            if cancelled {
                return i.status == "cancelled";
            }
            if all {
                return true;
            }
            // default: only actionable intents
            // INT-332: in-progress/ folder must always show
            (i.folder == "future" || i.folder == "deferred" || i.folder == "in-progress")
                && (i.status == "planned" || i.status == "in-progress")
        })
        .collect();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "📋 Intent Ledger".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    // INT-332: ACTIVE first, then PLANNED -- no mixing by folder
    let mut active_items: Vec<&Intent> = filtered
        .iter()
        .filter(|i| i.status == "in-progress")
        .copied()
        .collect();
    let mut planned_items: Vec<&Intent> = filtered
        .iter()
        .filter(|i| i.status != "in-progress" && i.status != "complete" && i.status != "cancelled")
        .copied()
        .collect();
    let mut cancelled_items: Vec<&Intent> = filtered
        .iter()
        .filter(|i| i.status == "cancelled")
        .copied()
        .collect();
    let complete_items: Vec<&Intent> = filtered
        .iter()
        .filter(|i| i.status == "complete")
        .copied()
        .collect();
    active_items.sort_by_key(|i| i.id.parse::<i32>().unwrap_or(0));
    planned_items.sort_by_key(|i| i.id.parse::<i32>().unwrap_or(0));
    cancelled_items.sort_by_key(|i| i.id.parse::<i32>().unwrap_or(0));

    if !active_items.is_empty() {
        println!("{}:", "ACTIVE (in-progress)".bright_green().bold());
        for i in &active_items {
            let id_display = format!("{:>3}", i.id).bright_cyan();
            println!("  {}  {} {}", id_display, i.status_colored(), i.title);
        }
    }
    if !planned_items.is_empty() {
        println!("{}:", "PLANNED (next up)".bright_yellow().bold());
        for i in &planned_items {
            let id_display = format!("{:>3}", i.id).bright_cyan();
            println!("  {}  {} {}", id_display, i.status_colored(), i.title);
        }
    }
    if !cancelled_items.is_empty() {
        println!("{}:", "CANCELLED".red().bold());
        for i in &cancelled_items {
            let id_display = format!("{:>3}", i.id).bright_cyan();
            println!("  {}  {} {}", id_display, i.status_colored(), i.title);
        }
    }
    if !complete_items.is_empty() && all {
        println!("{}:", "COMPLETE".dimmed().bold());
        for i in &complete_items {
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

/// The ONE validator. Context-free: reads `zero_core::paths::intents_dir()`, the same
/// source doctor uses. Returns (intents_seen, issues). Callers decide how to present it.
///
/// decisions/137: intent lifecycle dirs share ONE id namespace (a file MOVES between them).
/// Each record dir owns its own sequence.
pub fn validate_issues() -> (usize, Vec<String>) {
    const LIFECYCLE: &[&str] = &["future", "in-progress", "complete", "cancelled"];
    const VALID_STATUS: &[&str] = &[
        "planned",
        "in-progress",
        "complete",
        "cancelled",
        "deferred",
        "resolved",
        "decided",
    ];
    const FOLDERS: &[&str] = &[
        "complete",
        "future",
        "in-progress",
        "cancelled",
        "deferred",
        "decisions",
        "experiments",
        "incidents",
        "philosophy",
    ];

    let base = zero_core::paths::intents_dir();
    let mut issues: Vec<String> = Vec::new();
    // ⚠️ AN ABSENT LEDGER IS NOT AN EMPTY ONE, and until 2026-08-23 this could not tell them apart.
    // Every folder below is read with `let Ok(entries) = read_dir(..) else { continue }`, so on a
    // machine without 0-Core all nine simply skip: count stays 0, issues stays empty, and the
    // doctor prints `✅ Intent Ledger -- 0 intents, all valid` about a directory that does not
    // exist. FOUND IN THE VM: a guest with the tools deployed but no ledger reported a green PASS.
    //
    // ⭐ Same defect as a services table that comes back empty because the service manager is
    // missing. The failure is indistinguishable from a legitimate empty result, so it is reported
    // as an issue -- which is what the existing type already means, and what both callers already
    // render correctly.
    if !base.exists() {
        issues.push(format!(
            "intent ledger not found at {} -- this machine has no 0-Core checkout",
            base.display()
        ));
        return (0, issues);
    }
    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    // INT-213 G2/G3: (id, depends_on, where) -- resolved after every file is known.
    let mut edges: Vec<(String, Vec<String>, String)> = Vec::new();
    let mut count = 0usize;

    for folder in FOLDERS {
        let dir = base.join(folder);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "README.md" {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            // Only the FRONTMATTER decides. A `content.contains("type: index")` here skipped
            // any charter that merely QUOTED the string -- including INT-135's own, which
            // documents this exemption. The validator could not see the intent that built it.
            let is_index = content
                .split("---")
                .nth(1)
                .map(|fm| fm.lines().any(|l| l.trim() == "type: index"))
                .unwrap_or(false);
            if is_index {
                continue;
            }

            // Frontmatter MUST begin at byte 0. parse_intent() returns Option and would
            // silently drop this file -- which is exactly why complete/105 ('i---') hid
            // for months, and cancelled/058 ('\n---') beside it.
            if !content.starts_with("---") {
                issues.push(format!("Malformed frontmatter: {}/{}", folder, name));
                continue;
            }

            count += 1;
            let fm = content.split("---").nth(1).unwrap_or("");
            let field = |k: &str| -> Option<String> {
                fm.lines()
                    .find(|l| l.trim_start().starts_with(&format!("{}:", k)))
                    .map(|l| {
                        l.splitn(2, ':')
                            .nth(1)
                            .unwrap_or("")
                            .trim()
                            .trim_matches('"')
                            .to_string()
                    })
                    .filter(|v| !v.is_empty())
            };

            let id = field("id");
            for k in ["id", "title", "status", "date"] {
                if field(k).is_none() {
                    issues.push(format!("Missing '{}': {}/{}", k, folder, name));
                }
            }

            if let Some(st) = field("status") {
                if !VALID_STATUS.contains(&st.as_str()) {
                    issues.push(format!("Invalid status '{}': {}/{}", st, folder, name));
                }
                // Record dirs legitimately carry several statuses (decisions/121 is
                // 'complete', decisions/136 is 'decided'). Only lifecycle dirs map 1:1.
                if LIFECYCLE.contains(folder) {
                    let want = if *folder == "future" {
                        "planned"
                    } else {
                        folder
                    };
                    if st != want {
                        issues.push(format!("status '{}' != directory: {}/{}", st, folder, name));
                    }
                }
            }

            if let Some(id) = id {
                if let Some(prefix) = name.split('-').next() {
                    if prefix != id {
                        issues.push(format!(
                            "id '{}' != filename prefix '{}': {}/{}",
                            id, prefix, folder, name
                        ));
                    }
                }
                let ns = if LIFECYCLE.contains(folder) {
                    "lifecycle"
                } else {
                    folder
                };
                // INT-213 G2/G3: collect lifecycle edges for the second pass. An edge is only
                // meaningful inside one namespace -- decision 144 and intent 144 both exist.
                if LIFECYCLE.contains(folder) {
                    if let Some(raw) = field("depends_on") {
                        let deps: Vec<String> = raw
                            .trim_matches('[')
                            .trim_matches(']')
                            .split(',')
                            .map(|t| t.trim().to_string())
                            .filter(|t| !t.is_empty())
                            .collect();
                        if !deps.is_empty() {
                            edges.push((id.clone(), deps, format!("{}/{}", folder, name)));
                        }
                    }
                }
                *seen.entry((ns.to_string(), id)).or_insert(0) += 1;
            }
        }
    }

    for ((ns, id), n) in &seen {
        if *n > 1 {
            issues.push(format!("Duplicate id {} within namespace '{}'", id, ns));
        }
    }

    // INT-213 G2: a depends_on naming an id that does not exist blocks its dependent
    // forever and renders as "not found". Nothing validated it until now.
    let known: std::collections::HashSet<&str> = seen
        .keys()
        .filter(|(ns, _)| ns == "lifecycle")
        .map(|(_, id)| id.as_str())
        .collect();
    for (id, deps, where_) in &edges {
        for dep in deps {
            if !known.contains(dep.as_str()) {
                issues.push(format!(
                    "INT-{} depends_on INT-{}, which does not exist: {}",
                    id, dep, where_
                ));
            }
        }
    }
    // INT-213 G3: a cycle does not hang -- deps_critical_path already guards that with a
    // visited set. It makes every intent in the cycle vanish from recommendations silently.
    let adjacency: HashMap<&str, &Vec<String>> = edges
        .iter()
        .map(|(id, deps, _)| (id.as_str(), deps))
        .collect();
    for (start, _, where_) in &edges {
        let mut stack = vec![start.as_str()];
        let mut visited: std::collections::HashSet<&str> = std::collections::HashSet::new();
        while let Some(node) = stack.pop() {
            if !visited.insert(node) {
                continue;
            }
            if let Some(deps) = adjacency.get(node) {
                for dep in deps.iter() {
                    if dep == start {
                        issues.push(format!(
                            "Dependency cycle reaching INT-{}: {}",
                            start, where_
                        ));
                    }
                    stack.push(dep.as_str());
                }
            }
        }
    }
    (count, issues)
}

pub fn validate(_ctx: &AppContext) -> CoreResult<()> {
    let (count, issues) = validate_issues();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔍 Intent Ledger Validation".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if issues.is_empty() {
        println!("  {} All {} intents valid", "✅".green(), count);
    } else {
        for msg in &issues {
            println!("  {} {}", "✗".bright_red(), msg);
        }
        println!("  {} {} issues found", "✗".bright_red(), issues.len());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    Ok(())
}

// ── Phase 2 — Focus + Workflow States ─────────────────────────────────────

// INT-250: THE WRITER delegates to paths.rs, so writer and five readers resolve identically.
//
// These two built the path themselves, in the file that DEFINES the focus format. Five readers
// then built it again, each with its own line-parser, each carrying a copy of the INT-071 note
// explaining that focus.toml is the source of truth. The knowledge propagated; the path did not.
fn intent_state_dir() -> PathBuf {
    zero_core::paths::zero_state_dir().join("intent")
}

fn focus_file() -> PathBuf {
    zero_core::paths::focus_file()
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
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/root"))
                .join("0-core")
                .to_string_lossy()
                .as_ref(),
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
        println!("  {} Project 0 is on course", "✓".bright_green());
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

    // INT-247 Phase 3: dependency check before starting
    {
        let intents_all = load_all(ctx);
        // INT-213 G4: only a genuinely blocking dependency refuses the start.
        // A cancelled dependency no longer stops the work -- it warns, because
        // cancellation removes the blocking condition without making the
        // assumption behind the edge true.
        let mut unmet: Vec<String> = Vec::new();
        let mut notes: Vec<String> = Vec::new();
        for dep in &intent.depends_on {
            match dep_state(&intents_all, dep) {
                DepState::Satisfied => {}
                DepState::Blocked => {
                    let text = intents_all
                        .iter()
                        .find(|i| &i.id == dep)
                        .map(|i| format!("INT-{} -- {} [{}]", i.id, i.title, i.status))
                        .unwrap_or_else(|| format!("INT-{}", dep));
                    unmet.push(text);
                }
                DepState::Questionable => {
                    notes.push(format!("INT-{} (cancelled) -- {}", dep, CANCELLED_DEP_NOTE));
                }
                DepState::Missing => {
                    notes.push(format!(
                        "INT-{} names no intent -- the reference is invalid",
                        dep
                    ));
                }
            }
        }
        if !notes.is_empty() {
            println!("  {} Dependency notes:", "!".bright_yellow());
            for note in &notes {
                println!("    {} {}", "~".dimmed(), note.bright_yellow());
            }
            println!();
        }
        if !unmet.is_empty() {
            println!("{}", "⚠️  Dependency Check Failed".bold().bright_yellow());
            println!("{}", "━".repeat(55).dimmed());
            println!("  {} INT-{} has unmet dependencies:", "🔒".dimmed(), id);
            for dep in &unmet {
                println!("    {} {}", "⬆".bright_yellow(), dep.bright_red());
            }
            println!();
            println!(
                "  {} Complete dependencies first, or override:",
                "→".dimmed()
            );
            println!("  {} cistart {} --override", "  ".dimmed(), id);
            println!("{}", "━".repeat(55).dimmed());
            return Ok(());
        }
    }
    // Auto-checkpoint before transition
    println!("  {} Auto-checkpointing before transition...", "→".dimmed());
    crate::domains::checkpoint::auto(ctx, &format!("intent-{}-start", id))?;
    // INT-251 v23: emit to unified event bus
    let _ = crate::domains::friday::events::emit(
        ctx,
        "intent",
        "intent_started",
        &format!(r#"{{\"id\":\"{}\"}}"#, id),
        "core",
        None,
    );

    // Update frontmatter status in file
    let base = ctx.fpath("intents");
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
        // cistart fix: move file from future/ to in-progress/ if needed
        if path.to_string_lossy().contains("/intents/future/") {
            if let (Some(filename), Some(parent)) =
                (path.file_name(), path.parent().and_then(|p| p.parent()))
            {
                let new_path = parent.join("in-progress").join(filename);
                let _ = fs::rename(&path, &new_path);
            }
        }
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

/// INT-271: how the close settles versions. Empty: ask in a terminal, refuse anywhere else.
#[derive(Debug, Default, Clone)]
pub struct BumpFlags {
    /// One `tool=patch|minor|major|skip` per --bump.
    pub bumps: Vec<String>,
    /// Every touched crate that no --bump names is recorded as skipped.
    pub skip_rest: bool,
}

/// A crate this intent's commits touched: the id bump-versions accepts, and its Cargo.toml
/// relative to the 0-core root.
struct TouchedCrate {
    tool: &'static str,
    rel: String,
}

/// One settled crate. `level` None means skipped.
struct VersionPlan {
    tool: &'static str,
    rel: String,
    level: Option<zero_core::version::Level>,
}

/// INT-326 found the touched crates this way. INT-271 keeps the detection and names each crate
/// by the id bump-versions accepts: `engine`, never the label `core (engine)`.
fn touched_crates(ctx: &AppContext, id: &str) -> Vec<TouchedCrate> {
    let touched = std::process::Command::new("git")
        .args([
            "-C",
            &ctx.core_root,
            "log",
            "--name-only",
            "--format=",
            &format!("--grep=INT-{}", id),
            "--since=60 days ago",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    // INT-273: one rule for which crate a path is in, shared with the evidence at the question.
    let paths: Vec<&str> = touched.lines().collect();
    TRACKED_CRATES
        .iter()
        .filter(|tool| paths.iter().any(|p| path_in_crate(p, tool)))
        .filter_map(|tool| {
            zero_core::paths::crate_rel_dir(tool).map(|d| TouchedCrate {
                tool: *tool,
                rel: format!("{}/Cargo.toml", d),
            })
        })
        .collect()
}

/// Check every --bump before anything is asked or written: the `tool=level` shape, a crate this
/// intent touched, a known level, no crate named twice. `core` is accepted for `engine`.
fn parse_bump_flags(
    raw: &[String],
    touched: &[&'static str],
) -> Result<Vec<(&'static str, Option<zero_core::version::Level>)>, String> {
    let mut out: Vec<(&'static str, Option<zero_core::version::Level>)> = Vec::new();
    for r in raw {
        let (name, word) = r
            .split_once('=')
            .ok_or_else(|| format!("--bump {} is not tool=level", r))?;
        let name = if name == "core" { "engine" } else { name };
        let tool = touched
            .iter()
            .copied()
            .find(|t| *t == name)
            .ok_or_else(|| {
                let seen = if touched.is_empty() {
                    "none".to_string()
                } else {
                    touched.join(", ")
                };
                format!(
                    "--bump {}: this intent's commits did not touch {} (touched: {})",
                    r, name, seen
                )
            })?;
        let level = if word == "skip" {
            None
        } else {
            Some(zero_core::version::Level::parse(word).ok_or_else(|| {
                format!("--bump {}: {} is not patch, minor, major or skip", r, word)
            })?)
        };
        if out.iter().any(|(t, _)| *t == tool) {
            return Err(format!("--bump names {} twice", tool));
        }
        out.push((tool, level));
    }
    Ok(out)
}

/// Ask on stderr until the answer is a level or skip. No default: the digit is a judgement about
/// the contract (INT-102 D4 and D5), so an empty or mistyped answer asks again.
fn ask_level(tool: &str, current: &str) -> Result<Option<zero_core::version::Level>, String> {
    use std::io::{BufRead, Write};
    loop {
        eprint!("    bump {} {}? [patch/minor/major/skip] > ", tool, current);
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        let n = std::io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|e| format!("reading the answer for {}: {}", tool, e))?;
        if n == 0 {
            return Err(format!("no answer for {}: input closed", tool));
        }
        let word = line.trim();
        if matches!(word, "skip" | "s" | "n") {
            return Ok(None);
        }
        match zero_core::version::Level::parse(word) {
            Some(level) => return Ok(Some(level)),
            None => eprintln!("      '{}' is not patch, minor, major or skip", word),
        }
    }
}

/// INT-273: the crates the close tracks, in the order it asks about them.
const TRACKED_CRATES: [&str; 5] = [
    "novashell",
    "engine",
    "zero-git",
    "friday-chat",
    "db-browse",
];

/// INT-273: whether a repo path is inside a tracked crate. The one rule: touched_crates uses it to
/// decide what to ask about and the evidence uses it to decide what to show, so the two cannot
/// disagree. novashell is matched under every parent it has lived in, engine by its source and
/// manifest, the rest by name, as INT-326 did.
fn path_in_crate(path: &str, tool: &str) -> bool {
    match tool {
        "novashell" => zero_core::paths::CRATE_PARENTS_HISTORY
            .iter()
            .any(|p| path.contains(&format!("{}/novashell", p))),
        "engine" => path.contains("engine/src") || path.contains("engine/Cargo"),
        _ => path.contains(tool),
    }
}

/// INT-273: one file a commit changed. `added` and `removed` are None for a binary file; `from` is
/// the old path when git detected a rename, and `path` is always the destination.
#[derive(Debug, Clone, PartialEq)]
struct EvFile {
    added: Option<u64>,
    removed: Option<u64>,
    path: String,
    from: Option<String>,
}

/// INT-273: one commit of this intent, as git log -z --numstat reports it.
#[derive(Debug, Clone, PartialEq)]
struct EvCommit {
    hash: String,
    subject: String,
    files: Vec<EvFile>,
}

/// INT-273: this intent's commits with their numstat, read from the same commit set as
/// touched_crates (the same --grep and --since), so the evidence describes exactly the work the
/// question is about. Renames stay detected: a move counts its changed lines at its destination,
/// never the whole file twice.
fn intent_commits(core_root: &str, id: &str) -> Result<Vec<EvCommit>, String> {
    let out = std::process::Command::new("git")
        .args([
            "-C",
            core_root,
            "log",
            "-z",
            "--numstat",
            "--format=%x01%h%x09%s",
            &format!("--grep=INT-{}", id),
            "--since=60 days ago",
        ])
        .output()
        .map_err(|e| format!("git log could not run: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "git log exited {}: {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(parse_numstat_z(&String::from_utf8_lossy(&out.stdout)))
}

/// INT-273: parse git log -z --numstat --format=%x01%h%x09%s. A header is \x01, the hash, a TAB
/// and the subject. A file is added TAB removed TAB path; a rename has an empty path and is
/// followed by the old and the new path as their own NUL-separated fields.
fn parse_numstat_z(raw: &str) -> Vec<EvCommit> {
    let fields: Vec<&str> = raw.split('\0').collect();
    let mut commits: Vec<EvCommit> = Vec::new();
    let mut i = 0;
    while i < fields.len() {
        let f = fields[i].trim_start_matches('\n');
        i += 1;
        if f.is_empty() {
            continue;
        }
        if let Some(head) = f.strip_prefix('\u{1}') {
            let (hash, subject) = head.split_once('\t').unwrap_or((head, ""));
            commits.push(EvCommit {
                hash: hash.to_string(),
                subject: subject.to_string(),
                files: Vec::new(),
            });
            continue;
        }
        let mut parts = f.splitn(3, '\t');
        let added = parts.next().and_then(|n| n.parse().ok());
        let removed = parts.next().and_then(|n| n.parse().ok());
        let path = parts.next().unwrap_or("");
        let file = if path.is_empty() {
            let from = fields.get(i).map(|s| s.to_string());
            let to = fields.get(i + 1).copied().unwrap_or("");
            i += 2;
            EvFile {
                added,
                removed,
                path: to.to_string(),
                from,
            }
        } else {
            EvFile {
                added,
                removed,
                path: path.to_string(),
                from: None,
            }
        };
        if let Some(c) = commits.last_mut() {
            c.files.push(file);
        }
    }
    commits
}

/// INT-273: what is printed above the question for one crate: this intent's commits that touched
/// it, the files and lines they changed under it, the tier legend and the join-key note. It
/// informs the digit and never chooses it (INT-102 D4 and D5).
fn evidence_lines(commits: &[EvCommit], tool: &str, id: &str) -> Vec<String> {
    struct Total {
        path: String,
        from: Option<String>,
        added: u64,
        removed: u64,
        binary: bool,
    }
    const SHOWN: usize = 8;
    let mine: Vec<&EvCommit> = commits
        .iter()
        .filter(|c| c.files.iter().any(|f| path_in_crate(&f.path, tool)))
        .collect();
    let mut out = vec![format!(
        "    {}: {} commit(s) naming INT-{}",
        tool,
        mine.len(),
        id
    )];
    for c in mine.iter().take(SHOWN) {
        out.push(format!("      {} {}", c.hash, c.subject));
    }
    if mine.len() > SHOWN {
        out.push(format!("      ... and {} more", mine.len() - SHOWN));
    }
    let mut totals: Vec<Total> = Vec::new();
    for c in &mine {
        for f in c.files.iter().filter(|f| path_in_crate(&f.path, tool)) {
            let at = match totals.iter().position(|t| t.path == f.path) {
                Some(n) => n,
                None => {
                    totals.push(Total {
                        path: f.path.clone(),
                        from: None,
                        added: 0,
                        removed: 0,
                        binary: false,
                    });
                    totals.len() - 1
                }
            };
            let t = &mut totals[at];
            if t.from.is_none() {
                t.from = f.from.clone();
            }
            match (f.added, f.removed) {
                (Some(a), Some(r)) => {
                    t.added += a;
                    t.removed += r;
                }
                _ => t.binary = true,
            }
        }
    }
    totals.sort_by(|a, b| a.path.cmp(&b.path));
    let added: u64 = totals.iter().map(|t| t.added).sum();
    let removed: u64 = totals.iter().map(|t| t.removed).sum();
    out.push(format!(
        "    {} file(s) under {}, +{} -{}",
        totals.len(),
        tool,
        added,
        removed
    ));
    for t in totals.iter().take(SHOWN) {
        let size = if t.binary {
            "binary".to_string()
        } else {
            format!("+{} -{}", t.added, t.removed)
        };
        let moved = match &t.from {
            Some(f) => format!(" (renamed from {})", f),
            None => String::new(),
        };
        out.push(format!("      {:<10} {}{}", size, t.path, moved));
    }
    if totals.len() > SHOWN {
        out.push(format!("      ... and {} more files", totals.len() - SHOWN));
    }
    out.push(
        "    tiers: major breaks a promise, minor adds one, patch fixes one or touches nothing promised"
            .to_string(),
    );
    out.push(format!(
        "    counted: only commits from the last 60 days whose message names INT-{}",
        id
    ));
    out
}

#[cfg(test)]
mod int273_tests {
    use super::{evidence_lines, parse_numstat_z, path_in_crate};
    use zero_core::paths::{CRATE_PARENTS_HISTORY, SHELL_PARENT, TOOLS_PARENT};

    // The bytes git log -z --numstat --format=%x01%h%x09%s wrote for the INT-273 scratch tree.
    // Crate directories come from the owner (INT-267 L2), never from a literal.
    fn raw() -> String {
        let a = format!("{}/novashell/src/a.rs", SHELL_PARENT);
        let b = format!("{}/novashell/src/b.rs", SHELL_PARENT);
        [
            "\u{1}a13cd9e\tINT-999: novashell renames a.rs to b.rs with one edit".to_string(),
            "\n1\t1\t".to_string(),
            a.clone(),
            b,
            "\u{1}31ebd31\tINT-999: novashell edits a.rs, engine gains e.rs".to_string(),
            "\n5\t0\tzero/engine/src/e.rs".to_string(),
            format!("3\t2\t{}", a),
            "\u{1}f064548\tINT-999: novashell gains a.rs".to_string(),
            format!("\n10\t0\t{}", a),
        ]
        .join("\u{0}")
            + "\u{0}"
    }

    #[test]
    fn numstat_z_reads_a_rename_at_its_destination() {
        let commits = parse_numstat_z(&raw());
        assert_eq!(commits.len(), 3);
        let moved = &commits[0].files[0];
        assert_eq!(moved.path, format!("{}/novashell/src/b.rs", SHELL_PARENT));
        let from = format!("{}/novashell/src/a.rs", SHELL_PARENT);
        assert_eq!(moved.from.as_deref(), Some(from.as_str()));
        assert_eq!((moved.added, moved.removed), (Some(1), Some(1)));
        assert_eq!(commits[1].files.len(), 2);
        assert_eq!(commits[2].subject, "INT-999: novashell gains a.rs");
    }

    #[test]
    fn every_number_matches_the_scratch_tree() {
        let commits = parse_numstat_z(&raw());
        let ns = evidence_lines(&commits, "novashell", "999").join("\n");
        let renamed = format!("(renamed from {}/novashell/src/a.rs)", SHELL_PARENT);
        assert!(
            ns.contains("novashell: 3 commit(s) naming INT-999"),
            "{}",
            ns
        );
        assert!(ns.contains("2 file(s) under novashell, +14 -3"), "{}", ns);
        assert!(ns.contains("+13 -2"), "{}", ns);
        assert!(ns.contains(&renamed), "{}", ns);
        assert!(ns.contains("tiers: major breaks a promise"), "{}", ns);
        assert!(ns.contains("whose message names INT-999"), "{}", ns);
        let en = evidence_lines(&commits, "engine", "999").join("\n");
        assert!(en.contains("engine: 1 commit(s) naming INT-999"), "{}", en);
        assert!(en.contains("1 file(s) under engine, +5 -0"), "{}", en);
        assert!(!en.contains("novashell/src"), "{}", en);
    }

    #[test]
    fn one_rule_places_a_path_in_a_crate() {
        for parent in CRATE_PARENTS_HISTORY {
            let p = format!("{}/novashell/src/main.rs", parent);
            assert!(path_in_crate(&p, "novashell"), "{}", p);
        }
        // A historical parent under an older root still matches, as touched_crates always did.
        let nested = format!("old/{}/novashell/Cargo.toml", CRATE_PARENTS_HISTORY[0]);
        assert!(path_in_crate(&nested, "novashell"), "{}", nested);
        assert!(path_in_crate("zero/engine/src/main.rs", "engine"));
        assert!(path_in_crate("zero/engine/Cargo.toml", "engine"));
        assert!(!path_in_crate("zero/engine/README.md", "engine"));
        let git = format!("{}/zero-git/src/lib.rs", TOOLS_PARENT);
        assert!(path_in_crate(&git, "zero-git"), "{}", git);
        assert!(!path_in_crate(
            "zero/intents/complete/272-x.md",
            "novashell"
        ));
    }
}

/// INT-271: every version decision, made and checked before the first write. A --bump decides
/// first, --skip-bumps fills the rest, a terminal asks for what is left, and anywhere else an
/// undecided crate refuses the close. Each Cargo.toml passes plan_bump here, so a manifest the
/// writer would refuse stops the close before anything changes.
fn decide_versions(
    ctx: &AppContext,
    id: &str,
    flags: &BumpFlags,
) -> Result<Vec<VersionPlan>, String> {
    use std::io::IsTerminal;
    let touched = touched_crates(ctx, id);
    let ids: Vec<&'static str> = touched.iter().map(|t| t.tool).collect();
    let explicit = parse_bump_flags(&flags.bumps, &ids)?;
    let interactive = std::io::stdin().is_terminal();
    // INT-273: the work this intent did, read once and shown above each question that is asked.
    let evidence = if interactive {
        Some(intent_commits(&ctx.core_root, id))
    } else {
        None
    };
    let mut plans = Vec::new();
    let mut undecided: Vec<&'static str> = Vec::new();
    for t in &touched {
        let full = std::path::Path::new(&ctx.core_root).join(&t.rel);
        let content = std::fs::read_to_string(&full)
            .map_err(|e| format!("cannot read {}: {}", full.display(), e))?;
        let current = zero_core::version::plan_bump(&content, zero_core::version::Level::Patch)
            .map_err(|e| format!("{}: {}", full.display(), e))?
            .old;
        let level = if let Some((_, l)) = explicit.iter().find(|(tool, _)| *tool == t.tool) {
            *l
        } else if flags.skip_rest {
            None
        } else if interactive {
            match &evidence {
                Some(Ok(commits)) => {
                    for line in evidence_lines(commits, t.tool, id) {
                        eprintln!("{}", line);
                    }
                }
                Some(Err(e)) => eprintln!("    evidence unavailable: {}", e),
                None => {}
            }
            ask_level(t.tool, &current)?
        } else {
            undecided.push(t.tool);
            continue;
        };
        if let Some(l) = level {
            zero_core::version::plan_bump(&content, l)
                .map_err(|e| format!("{}: {}", full.display(), e))?;
        }
        plans.push(VersionPlan {
            tool: t.tool,
            rel: t.rel.clone(),
            level,
        });
    }
    if !undecided.is_empty() {
        let decide: Vec<String> = undecided
            .iter()
            .map(|t| format!("--bump {}=<patch|minor|major|skip>", t))
            .collect();
        return Err(format!(
            "no terminal to ask and no decision for {}. Decide with: cicomplete {} {} (or --skip-bumps)",
            undecided.join(", "),
            id,
            decide.join(" ")
        ));
    }
    Ok(plans)
}

/// INT-271: write every planned bump through the one owner, then sync Cargo.lock for it. Returns
/// the Versions receipt for the intent. A write that fails stops the close: the caller does not
/// move the intent, and the message names what was written and how to finish.
fn write_versions(ctx: &AppContext, id: &str, plans: &[VersionPlan]) -> CoreResult<String> {
    if plans.is_empty() {
        return Ok(String::new());
    }
    let mut receipt = String::from("\n## Versions\n");
    let mut written: Vec<String> = Vec::new();
    println!();
    println!("  {} Versions", "\u{1f4e6}".normal());
    for (i, p) in plans.iter().enumerate() {
        let Some(level) = p.level else {
            println!("    {} {} skipped", "\u{25e6}".dimmed(), p.tool);
            receipt.push_str(&format!("- {} skipped\n", p.tool));
            continue;
        };
        let full = std::path::Path::new(&ctx.core_root).join(&p.rel);
        match zero_core::version::apply_bump(&full, level) {
            Ok((old, new)) => {
                println!(
                    "    {} {} {} -> {} ({})",
                    "\u{2713}".green(),
                    p.tool,
                    old,
                    new,
                    level.as_str()
                );
                receipt.push_str(&format!(
                    "- {} {} -> {} ({})\n",
                    p.tool,
                    old,
                    new,
                    level.as_str()
                ));
                written.push(format!("{} {}", p.tool, new));
                sync_lock(ctx, &full, &new);
            }
            Err(e) => {
                let rest: Vec<String> = plans[i..]
                    .iter()
                    .filter_map(|q| {
                        q.level
                            .map(|l| format!("bump-versions {} {}", q.tool, l.as_str()))
                    })
                    .collect();
                let before = if written.is_empty() {
                    "nothing".to_string()
                } else {
                    written.join(", ")
                };
                eprintln!("  bump failed for {}: {}", p.tool, e);
                eprintln!("  written before it: {}", before);
                eprintln!(
                    "  the intent was not moved. Finish with: {}; then rerun cicomplete {} and answer skip for crates already bumped",
                    rest.join("; "),
                    id
                );
                return Err(crate::errors::CoreError::Reported(format!(
                    "bump failed for {}",
                    p.tool
                )));
            }
        }
    }
    Ok(receipt)
}

/// INT-125: keep Cargo.lock in step with a bump. The cargo package name comes from the crate's
/// own Cargo.toml, not the tool id.
fn sync_lock(ctx: &AppContext, toml: &std::path::Path, new: &str) {
    let pkg = std::fs::read_to_string(toml).ok().and_then(|c| {
        c.lines()
            .find(|l| l.trim_start().starts_with("name = "))
            .map(|l| {
                l.trim()
                    .trim_start_matches("name = ")
                    .trim_matches('"')
                    .to_string()
            })
    });
    let Some(pkg) = pkg else {
        return;
    };
    let lock = std::process::Command::new("cargo")
        .args(["update", "-p", &pkg, "--precise", new])
        .current_dir(&ctx.core_root)
        .output();
    match lock {
        Ok(o) if o.status.success() => println!(
            "      {} Cargo.lock synced ({} {})",
            "\u{2713}".green(),
            pkg.dimmed(),
            new.dimmed()
        ),
        Ok(o) => println!(
            "      {} Cargo.lock sync skipped: {}",
            "\u{26a0}".yellow(),
            String::from_utf8_lossy(&o.stderr).trim().dimmed()
        ),
        Err(e) => println!(
            "      {} cargo not available for lock sync: {}",
            "\u{26a0}".yellow(),
            e.to_string().dimmed()
        ),
    }
}

#[cfg(test)]
mod int271_close_tests {
    use super::parse_bump_flags;
    use zero_core::version::Level;

    const TOUCHED: &[&str] = &["novashell", "engine"];

    fn flags(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_bump_names_a_touched_crate_and_a_level() {
        let got = parse_bump_flags(&flags(&["novashell=minor", "core=skip"]), TOUCHED).unwrap();
        assert_eq!(
            got,
            vec![("novashell", Some(Level::Minor)), ("engine", None)]
        );
    }

    #[test]
    fn every_bad_bump_is_refused_before_anything_is_asked() {
        for bad in [
            "novashell",
            "novashell=ptach",
            "frob=patch",
            "db-browse=patch",
            "novashell=",
        ] {
            assert!(
                parse_bump_flags(&flags(&[bad]), TOUCHED).is_err(),
                "{} should be refused",
                bad
            );
        }
        assert!(
            parse_bump_flags(&flags(&["novashell=patch", "novashell=minor"]), TOUCHED).is_err()
        );
    }
}

pub fn complete_intent(ctx: &AppContext, id: &str, flags: &BumpFlags) -> CoreResult<()> {
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

    // INT-332: block cicomplete if open gates exist without formal deferral
    if let Some(ref path) = {
        let base = ctx.fpath("intents");
        let folders = ["future", "planned", "deferred", "in-progress"];
        let mut found: Option<std::path::PathBuf> = None;
        for folder in &folders {
            let dir = base.join(folder);
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with(&format!("{:0>3}", id)) || n.starts_with(id))
                        .unwrap_or(false)
                    {
                        found = Some(p);
                        break;
                    }
                }
            }
            if found.is_some() {
                break;
            }
        }
        found
    } {
        let file_content = std::fs::read_to_string(path).unwrap_or_default();
        // Check for malformed deferrals (⏸ without approval signature)
        let bad_deferrals: Vec<&str> = file_content
            .lines()
            .filter(|l| {
                let trimmed = l.trim();
                trimmed.starts_with('⏸') &&
                !l.contains("[date]") &&   // skip example lines
                !l.contains("[reason]") && // skip example lines
                (!l.contains("approved by: christian") || !l.contains("20"))
            })
            .collect();
        if !bad_deferrals.is_empty() {
            println!();
            println!(
                "  {} Deferral format error -- missing approval signature:",
                "🚫".normal()
            );
            for d in &bad_deferrals {
                println!("  {} {}", "⏸".normal(), d.trim());
            }
            println!();
            println!("  {} Required format:", "💡".normal());
            println!("    ⏸ gate -- deferred: [reason] -- approved by: christian 2026-05-25");
            println!();
            return Ok(());
        }
        let open_gates: Vec<&str> = file_content
            .lines()
            .filter(|l| {
                let trimmed = l.trim();
                // INT-130: detect markdown gates (the real format), not just the emoji.
                // OPEN = unchecked `- [ ]` or partial `- [~]`; legacy ⬜ still counts.
                // Only `- [x]` passes. A ⏸ deferral line is exempt.
                (trimmed.starts_with("- [ ]")
                    || trimmed.starts_with("- [~]")
                    || trimmed.starts_with('⬜'))
                    && !l.contains('⏸')
            })
            .collect();
        if !open_gates.is_empty() {
            println!();
            println!("  {} cicomplete BLOCKED -- {} open gate(s) require demonstration or formal deferral:", "🚫".normal(), open_gates.len());
            println!();
            for gate in &open_gates {
                println!("  {} {}", "⬜".normal(), gate.trim());
            }
            println!();
            println!(
                "  {} To defer a gate, edit the intent file and use:",
                "💡".normal()
            );
            println!(
                "    ⏸ gate description -- deferred: [reason] -- approved by: christian {}",
                chrono::Utc::now().format("%Y-%m-%d")
            );
            println!();
            println!(
                "  {} cicomplete blocked. Demonstrate gates or formally defer them.",
                "→".bright_red()
            );
            return Ok(());
        }
    }

    // INT-271: every version decision is made and checked here, before anything is written.
    let planned = match decide_versions(ctx, id, flags) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("  refused: {}", msg);
            eprintln!("  nothing was written and the intent was not moved");
            return Err(crate::errors::CoreError::Reported(msg));
        }
    };

    // Auto-checkpoint before completion
    println!("  {} Auto-checkpointing before completion...", "→".dimmed());
    crate::domains::checkpoint::auto(ctx, &format!("intent-{}-complete", id))?;
    // INT-271: write every planned bump. A failure stops here and the intent is not moved.
    let receipt = write_versions(ctx, id, &planned)?;

    // Find and move the file to complete/
    let base = ctx.fpath("intents");
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
        // INT-271: the Versions receipt rides in the same write as the move.
        let updated = updated + &receipt;

        // Move to complete/
        let complete_dir = base.join("complete");
        fs::create_dir_all(&complete_dir).map_err(crate::errors::CoreError::Io)?;
        let filename = src_path.file_name().unwrap();
        let dest_path = complete_dir.join(filename);
        fs::write(&dest_path, updated).map_err(crate::errors::CoreError::Io)?;
        fs::remove_file(&src_path).map_err(crate::errors::CoreError::Io)?;
    }

    // INT-251 v23: emit to unified event bus. INT-271: after the move, so a close that stopped
    // on a failed bump never announces a completion.
    let _ = crate::domains::friday::events::emit(
        ctx,
        "intent",
        "intent_completed",
        &format!(r#"{{\"id\":\"{}\"}}"#, id),
        "core",
        None,
    );

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

    // INT-247 Phase 4: show newly unblocked intents + simple retrospective
    {
        let all = load_all(ctx);
        let newly_unblocked: Vec<String> = all
            .iter()
            .filter(|i| i.status == "planned")
            .filter(|i| {
                i.depends_on.contains(&intent.id)
                    && i.depends_on.iter().all(|dep| {
                        dep == id || all.iter().any(|x| &x.id == dep && x.status == "complete")
                    })
            })
            .map(|i| format!("INT-{} -- {}", i.id, i.title))
            .collect();
        if !newly_unblocked.is_empty() {
            println!();
            println!("  {} Newly unblocked:", "🔓".green());
            for u in &newly_unblocked {
                println!("    {} {} is now ready to start", "◦".bright_green(), u);
            }
        }
        let commits = std::process::Command::new("git")
            .args([
                "-C",
                &ctx.core_root,
                "log",
                "--oneline",
                &format!("--grep=INT-{}", id),
                "--since=30 days ago",
            ])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        println!();
        println!("  {} Retrospective:", "📋".dimmed());
        if commits > 0 {
            println!(
                "    {} ~{} commits referencing INT-{}",
                "◦".dimmed(),
                commits,
                id
            );
        }
        println!("    {} Run: core intent next", "◦".dimmed());
    }
    // INT-217 -- Friday speaks on cicomplete
    let _ = crate::domains::friday::speak_on_complete(ctx, &intent.title);
    crate::domains::notify::desktop(&format!("Intent {} complete", id), &intent.title, false);
    Ok(())
}
/// A category IS a directory. Nothing outside this list is creatable.
/// `banana` was accepted for months because `new_intent` matched templates, not categories,
/// and fell through `_ =>` to "future".
// Order matters: this IS the wizard menu. Matches rust-tools/intent's numbering so
// muscle memory survives the retirement. 1=decisions, 2=experiments, 3=philosophy,
// 4=future, 5=incidents.
pub const CATEGORIES: &[&str] = &[
    "decisions",
    "experiments",
    "philosophy",
    "future",
    "incidents",
];

/// (template, type_tag, default_tags, default_status). `type` is SINGULAR -- what the
/// document IS. The wizard used to write `type: decisions` (the plural folder name) while
/// hand-written records said `type: decision`.
pub const TEMPLATES: &[(&str, &str, &str, &str)] = &[
    // No default names the retired project (INT-247 Layer 0); template_tests enforces it. An
    // empty default is fine: the wizard drops empty tags and offers tags from active work.
    ("feature", "feature", "feature, rust", "planned"),
    ("fix", "fix", "fix, bugfix", "planned"),
    ("arch", "arch", "architecture, rust, design", "planned"),
    ("study", "study", "study, research, learning", "planned"),
    ("future", "future", "", "planned"),
    ("decision", "decision", "decision", "decided"),
    ("incident", "incident", "incident", "resolved"),
    ("experiment", "experiment", "experiment", "complete"),
    ("philosophy", "philosophy", "philosophy", "complete"),
];

/// The ONE numberer. Same namespace rule `validate_issues` enforces (decisions/137):
/// the lifecycle dirs share a counter because a file MOVES between them; each record dir
/// owns its own sequence. Three copies of this logic existed before.
fn next_id(category: &str) -> u32 {
    const LIFECYCLE: &[&str] = &["future", "in-progress", "complete", "cancelled"];
    let base = zero_core::paths::intents_dir();
    let scan: Vec<&str> = if LIFECYCLE.contains(&category) {
        LIFECYCLE.to_vec()
    } else {
        vec![category]
    };
    scan.iter()
        .flat_map(|d| fs::read_dir(base.join(d)).ok().into_iter().flatten())
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            let p = n.split('-').next()?;
            if p.len() == 3 && p.bytes().all(|b| b.is_ascii_digit()) {
                p.parse::<u32>().ok()
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0)
        + 1
}

/// Chronological history. Ported from rust-tools/intent's cmd_timeline, which walked eight
/// folders and omitted `in-progress` -- it never once showed an active intent. load_all()
/// walks all nine.
pub fn timeline(ctx: &AppContext) -> CoreResult<()> {
    let mut intents: Vec<Intent> = load_all(ctx)
        .into_iter()
        .filter(|i| i.intent_type != "index" && !i.title.is_empty())
        .collect();
    intents.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));

    println!();
    println!("{}", "📅 Intent Timeline".bright_cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!();
    for i in &intents {
        println!(
            "{} - {} {} {}",
            i.date.dimmed(),
            format!("{:<4}", i.id).dimmed(),
            i.status_colored(),
            i.title
        );
    }
    println!();
    println!("  {} {}", "Total:".dimmed(), intents.len());
    Ok(())
}

/// `core intent add` -- the interactive wizard. Ported from rust-tools/intent's cmd_add,
/// which wrote `type: {plural folder name}`. This writes the SINGULAR type from TEMPLATES.
pub fn add(ctx: &AppContext, smart: bool) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    println!("{}", "📝 Adding New Intent".bright_cyan().bold());
    println!();
    println!("Select category:");
    for (n, c) in CATEGORIES.iter().enumerate() {
        println!("  {}) {}", n + 1, c);
    }

    let Some(choice) = prompt("Choice (1-5): ") else {
        return Err(crate::errors::CoreError::Runtime(
            "core intent add requires a terminal -- use `core intent new <category> <template> <title>`".into(),
        ));
    };
    let idx: usize = choice.parse().unwrap_or(0);
    if idx == 0 || idx > CATEGORIES.len() {
        return Err(crate::errors::CoreError::Runtime(format!(
            "invalid choice '{}'",
            choice
        )));
    }
    let category = CATEGORIES[idx - 1];

    // Default type + status for this category, from TEMPLATES.
    // Explicit map. Do NOT chop the trailing 's': philosophy -> philosoph.
    let want = match category {
        "decisions" => "decision",
        "experiments" => "experiment",
        "philosophy" => "philosophy",
        "incidents" => "incident",
        _ => "future",
    };
    let (_, type_tag, def_tags, def_status) = TEMPLATES
        .iter()
        .find(|(t, ..)| *t == want)
        // INT-247 Layer 0: the fallback IS the future row, never a copy of it. The copy that
        // lived here defaulted to the retired name, out of template_tests' reach.
        .or_else(|| TEMPLATES.iter().find(|(t, ..)| *t == "future"))
        .copied()
        .expect("TEMPLATES has a future row -- template_tests::future_row_exists pins it");

    let title = prompt("Title: ").unwrap_or_default();
    if title.is_empty() {
        return Err(crate::errors::CoreError::Runtime(
            "title is required".into(),
        ));
    }

    let status = prompt(&format!("Status [{}]: ", def_status))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| def_status.to_string());

    if smart {
        let intents = load_all(ctx);
        let mut freq: HashMap<String, usize> = HashMap::new();
        for i in intents.iter().filter(|i| i.status == "in-progress") {
            for t in &i.tags {
                *freq.entry(t.trim().to_string()).or_insert(0) += 1;
            }
        }
        let mut sug: Vec<String> = freq.into_keys().collect();
        sug.sort();
        sug.truncate(5);
        if !sug.is_empty() {
            println!(
                "  {} from active work: {}",
                "💡".normal(),
                sug.join(", ").dimmed()
            );
        }
    }

    let tags_in = prompt(&format!("Tags [{}]: ", def_tags)).unwrap_or_default();
    let tags: Vec<String> = if tags_in.is_empty() {
        def_tags
    } else {
        &tags_in
    }
    .split(',')
    .map(|t| t.trim().to_string())
    .filter(|t| !t.is_empty())
    .collect();

    let path = create(category, type_tag, &title, &status, &tags)?;
    println!();
    println!("  {} {}", "✅".normal(), path.display());
    Ok(())
}

/// The ONE creator. Nothing else writes an intent file.
pub fn create(
    category: &str,
    type_tag: &str,
    title: &str,
    status: &str,
    tags: &[String],
) -> CoreResult<PathBuf> {
    if !CATEGORIES.contains(&category) {
        return Err(crate::errors::CoreError::Runtime(format!(
            "unknown category '{}' -- expected one of {:?}",
            category, CATEGORIES
        )));
    }

    // A stray backspace (0x7f) from read_line landed inside a title during Gate 6 testing,
    // and the validator called it valid. Strip control characters before anything sees them.
    let title: String = title.chars().filter(|c| !c.is_control()).collect();
    let title = title.trim();
    if title.is_empty() {
        return Err(crate::errors::CoreError::Runtime(
            "title is empty after sanitizing".into(),
        ));
    }
    let id = next_id(category);
    let date = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "1970-01-01".to_string());

    // One slug rule. `new_intent` only replaced spaces, which is why 136's filename kept a
    // colon and a comma. cmd_add filtered to alphanumeric+dash. This does the latter.
    let slug: String = title
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect();

    let filename = format!("{:03}-{}.md", id, slug);
    let path = zero_core::paths::intents_dir()
        .join(category)
        .join(&filename);
    if path.exists() {
        return Err(crate::errors::CoreError::Runtime(format!(
            "refusing to overwrite {}",
            path.display()
        )));
    }

    let tags_str = if tags.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", tags.join(", "))
    };

    let content = format!(
        "---\nid: {:03}\ndate: {}\ntype: {}\ntitle: \"{}\"\nstatus: {}\ntags: {}\n---\n\n\
## Vision\n[Describe the goal and desired outcome]\n\n\
## The Problem\n[What problem does this solve?]\n\n\
## The Solution\n[High-level approach]\n\n\
## Success Criteria\n- [ ] ...\n\n\
<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.\n\
     When you tick a gate, put the proof in an HTML comment on the line after it: a commit\n\
     hash, a file:line, a log or artifact path, or \"demonstrated: what + how\". Prose counts.\n\
     FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).\n\
     SOFT (a discipline, not gate-police; nothing enforces this).\n\
     LIGHT (trivial self-evident gates need no artifact).\n\
     Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.\n\
     See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->\n",
        id, date, type_tag, title, status, tags_str
    );

    fs::create_dir_all(path.parent().unwrap()).map_err(crate::errors::CoreError::Io)?;
    fs::write(&path, content).map_err(crate::errors::CoreError::Io)?;
    Ok(path)
}

/// Prompt on a TTY. Returns None when stdin is not a terminal -- never block a script.
fn prompt(label: &str) -> Option<String> {
    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return None;
    }
    print!("{}", label);
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).ok()?;
    Some(line.trim().to_string())
}

pub fn new_intent(ctx: &AppContext, category: &str, template: &str, title: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "intent",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    // `banana` used to work: the old match fell through to a catch-all arm,
    // silently filing an unknown template as a future intent. Now it is an error.
    let (_, type_tag, tags, status) = TEMPLATES
        .iter()
        .find(|(t, ..)| *t == template)
        .copied()
        .ok_or_else(|| {
            crate::errors::CoreError::Runtime(format!(
                "unknown template '{}' -- expected one of: {}",
                template,
                TEMPLATES
                    .iter()
                    .map(|(t, ..)| *t)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;

    let tags: Vec<String> = tags.split(',').map(|t| t.trim().to_string()).collect();
    let path = create(category, type_tag, title, status, &tags)?;
    println!("  {} {}", "✅".normal(), path.display());
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

    // INT-213 G9 / law 3: blocks is the inverse of depends_on, so it is DERIVED, never stored.
    // Two copies of one fact can disagree, and only one of them can be authoritative. The
    // renderer below is unchanged -- only what it reads has changed.
    let derived_blocks: Vec<String> = intents
        .iter()
        .filter(|i| i.depends_on.contains(&intent.id))
        .map(|i| i.id.clone())
        .collect();
    let owned = Intent {
        blocks: derived_blocks,
        ..intent.clone()
    };
    let intent = &owned;
    if intent.depends_on.is_empty() && intent.blocks.is_empty() && intent.relates.is_empty() {
        println!("  {} No dependencies declared", "○".dimmed());
        println!(
            "  {} Add to frontmatter: depends_on: [052], relates: [099] -- blocks is derived",
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
    let cancelled = intents.iter().filter(|i| i.status == "cancelled").count();
    let remaining = total - complete - cancelled;

    println!("{}", "📉 Intent Burndown".bold());
    println!("{}", "━".repeat(55).dimmed());
    println!(
        "  {} {}  {} {}  {} {}  {} {}",
        "Total:".dimmed(),
        total.to_string().bright_white(),
        "Complete:".dimmed(),
        complete.to_string().bright_green(),
        "Cancelled:".dimmed(),
        cancelled.to_string().dimmed(),
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
        if running.is_multiple_of(10) || running == complete {
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
    let intent = all
        .iter()
        .find(|i| i.id.trim_start_matches("INT-").trim_start_matches("int-") == id_norm);
    match intent {
        Some(i) => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
            let path = intents_dir(ctx).join(&i.folder).join(&i.filename);
            let status = std::process::Command::new(&editor)
                .arg(&path)
                .status()
                .map_err(crate::errors::CoreError::Io)?;
            if status.success() {
                println!("  ✅ INT-{} saved", i.id);
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
    let _root = std::path::PathBuf::from(&ctx.core_root);
    let future_dir = zero_core::paths::intents_dir().join("future");
    let now = chrono::Utc::now().timestamp();
    let mut intents: Vec<(String, String, String, f64, Vec<String>)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&future_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".md") {
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if !content.contains("status: in-progress") {
                continue;
            }
            let id = name.split('-').next().unwrap_or("?").to_string();
            let title = content
                .lines()
                .find(|l| l.starts_with("title:"))
                .map(|l| {
                    l.trim_start_matches("title:")
                        .trim()
                        .trim_matches('"')
                        .to_string()
                })
                .unwrap_or_else(|| name.clone());
            let total_gates = content.matches("⬜").count() + content.matches("✅").count();
            let done_gates = content.matches("✅").count();
            let gate_pct = if total_gates > 0 {
                (done_gates * 100) / total_gates
            } else {
                0
            };
            let days_since: i64 = {
                let out = std::process::Command::new("git")
                    .args([
                        "-C",
                        &ctx.core_root,
                        "log",
                        "-1",
                        "--format=%ct",
                        "--",
                        &format!("intents/future/{}", name),
                    ])
                    .output()
                    .ok();
                let ts = out
                    .as_ref()
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout)
                            .trim()
                            .parse::<i64>()
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                if ts > 0 {
                    (now - ts) / 86400
                } else {
                    0
                }
            };
            let mut score = gate_pct as f64;
            let mut issues: Vec<String> = Vec::new();
            if days_since > 14 {
                score -= 20.0;
                issues.push(format!("stalled {} days", days_since));
            }
            score = score.clamp(0.0, 100.0);
            if !stale_only || days_since > 14 {
                intents.push((
                    id,
                    title,
                    format!("{}/{}", done_gates, total_gates),
                    score,
                    issues,
                ));
            }
        }
    }
    println!();
    println!("  💊 Intent Health");
    println!("  {}", "─".repeat(56).dimmed());
    if intents.is_empty() {
        println!("  ✅ All active intents healthy");
        println!();
        return Ok(());
    }
    intents.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
    for (id, title, gates, score, issues) in &intents {
        let s = if *score >= 70.0 {
            format!("{:.0}%", score).bright_green().to_string()
        } else if *score >= 40.0 {
            format!("{:.0}%", score).yellow().to_string()
        } else {
            format!("{:.0}%", score).bright_red().to_string()
        };
        println!(
            "  INT-{:<6} {:<36} {} gates  {}",
            id.bright_white(),
            title.dimmed(),
            gates,
            s
        );
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
        if intent_content.is_some() {
            break;
        }
    }
    let content = match intent_content {
        None => {
            println!("  {} Intent {} not found", "✗".bright_red(), id);
            return Ok(());
        }
        Some(c) => c,
    };
    let title = content
        .lines()
        .find(|l| l.starts_with("title:"))
        .map(|l| {
            l.trim_start_matches("title:")
                .trim()
                .trim_matches('"')
                .to_string()
        })
        .unwrap_or_else(|| intent_name.clone());
    let total_gates = content.matches("⬜").count() + content.matches("✅").count();
    let done_gates = content.matches("✅").count();
    let remaining = total_gates.saturating_sub(done_gates);
    // Get average gates completed per session from recent intents
    let complete_dir = zero_core::paths::intents_dir().join("complete");
    let mut gates_per_session: Vec<f64> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&complete_dir) {
        for entry in entries.flatten().take(10) {
            let c = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let total = c.matches("✅").count();
            if total > 0 {
                gates_per_session.push(total as f64 / 2.0);
            } // assume ~2 sessions avg
        }
    }
    let avg_gates_per_session = if gates_per_session.is_empty() {
        3.0
    } else {
        gates_per_session.iter().sum::<f64>() / gates_per_session.len() as f64
    };
    let sessions_needed = if avg_gates_per_session > 0.0 {
        (remaining as f64 / avg_gates_per_session).ceil() as usize
    } else {
        remaining
    };
    println!();
    println!("  {} Completion Prediction: INT-{}", "🎯".normal(), id);
    println!("  {}", "─".repeat(56).dimmed());
    println!("  {:<24} {}", "Intent:".dimmed(), title.bright_white());
    println!(
        "  {:<24} {}/{} gates done ({}%)",
        "Gates:".dimmed(),
        done_gates,
        total_gates,
        if total_gates > 0 {
            (done_gates * 100) / total_gates
        } else {
            0
        }
    );
    println!(
        "  {:<24} {} gates remaining",
        "Remaining:".dimmed(),
        remaining.to_string().bright_yellow()
    );
    println!(
        "  {:<24} {:.1} gates/session (from {} recent intents)",
        "Velocity:".dimmed(),
        avg_gates_per_session,
        gates_per_session.len()
    );
    println!(
        "  {:<24} ~{} session(s)",
        "Estimated:".dimmed(),
        sessions_needed.to_string().bright_cyan()
    );
    if sessions_needed == 0 {
        println!(
            "  {} All gates satisfied — ready to cicomplete!",
            "✅".normal()
        );
    } else if sessions_needed == 1 {
        println!(
            "  {} One focused session should complete this intent",
            "→".bright_cyan()
        );
    } else {
        println!(
            "  {} {} sessions estimated at current velocity",
            "→".bright_cyan(),
            sessions_needed
        );
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
                    source_title = source_content
                        .lines()
                        .find(|l| l.starts_with("title:"))
                        .map(|l| {
                            l.trim_start_matches("title:")
                                .trim()
                                .trim_matches('"')
                                .to_string()
                        })
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
    let source_tags: Vec<String> = source_content
        .lines()
        .find(|l| l.starts_with("tags:"))
        .map(|l| l.to_lowercase())
        .unwrap_or_default()
        .split(',')
        .map(|t| {
            t.trim()
                .trim_matches('[')
                .trim_matches(']')
                .trim_matches('"')
                .to_string()
        })
        .filter(|t| !t.is_empty())
        .collect();
    // Find related intents by tag overlap
    let mut related: Vec<(String, String, usize)> = Vec::new(); // (id, title, overlap)
    for dir in &["intents/future", "intents/complete"] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".md") || name.starts_with(id) {
                    continue;
                }
                let c = std::fs::read_to_string(entry.path()).unwrap_or_default();
                let other_tags: Vec<String> = c
                    .lines()
                    .find(|l| l.starts_with("tags:"))
                    .map(|l| l.to_lowercase())
                    .unwrap_or_default()
                    .split(',')
                    .map(|t| {
                        t.trim()
                            .trim_matches('[')
                            .trim_matches(']')
                            .trim_matches('"')
                            .to_string()
                    })
                    .filter(|t| !t.is_empty())
                    .collect();
                let overlap = source_tags
                    .iter()
                    .filter(|t| other_tags.contains(t))
                    .count();
                if overlap >= 2 {
                    let other_id = name.split('-').next().unwrap_or("?").to_string();
                    let other_title = c
                        .lines()
                        .find(|l| l.starts_with("title:"))
                        .map(|l| {
                            l.trim_start_matches("title:")
                                .trim()
                                .trim_matches('"')
                                .to_string()
                        })
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
            println!(
                "    {} INT-{:<6} {} ({} shared tags)",
                "·".dimmed(),
                rid.bright_white(),
                rtitle.dimmed(),
                overlap.to_string().bright_cyan()
            );
        }
    }
    println!();
    Ok(())
}

#[allow(unused_assignments)]
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
        if intent_content.is_some() {
            break;
        }
    }
    let content = match intent_content {
        None => {
            println!("  {} Intent {} not found", "✗".bright_red(), id);
            return Ok(());
        }
        Some(c) => c,
    };
    let title = content
        .lines()
        .find(|l| l.starts_with("title:"))
        .map(|l| {
            l.trim_start_matches("title:")
                .trim()
                .trim_matches('"')
                .to_string()
        })
        .unwrap_or_default();
    let date = content
        .lines()
        .find(|l| l.starts_with("date:"))
        .map(|l| l.trim_start_matches("date:").trim().to_string())
        .unwrap_or_default();
    let status = content
        .lines()
        .find(|l| l.starts_with("status:"))
        .map(|l| l.trim_start_matches("status:").trim().to_string())
        .unwrap_or_default();
    let tags: Vec<String> = content
        .lines()
        .find(|l| l.starts_with("tags:"))
        .map(|l| {
            l.trim_start_matches("tags:")
                .trim()
                .trim_matches('[')
                .trim_matches(']')
                .split(',')
                .map(|t| t.trim().trim_matches('"').to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let total_gates = content.matches("⬜").count() + content.matches("✅").count();
    let done_gates = content.matches("✅").count();
    // Get git commits touching this intent
    let commit_log = std::process::Command::new("git")
        .args([
            "-C",
            &ctx.core_root,
            "log",
            "--oneline",
            "--",
            &format!("intents/**/*{}*", id),
        ])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let commit_count = commit_log.lines().count();
    // Find related intents
    let mut related_ids: Vec<String> = Vec::new();
    for dir in &["intents/complete", "intents/future"] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".md") || name.starts_with(id) {
                    continue;
                }
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
    println!(
        "  {} INT-{} — {}",
        "📖".normal(),
        id.bright_white(),
        title.bright_white()
    );
    println!("  {}", "═".repeat(60).dimmed());
    println!();
    println!("  {:<16} {}", "Date:".dimmed(), date.bright_cyan());
    println!(
        "  {:<16} {}",
        "Status:".dimmed(),
        if status == "complete" {
            "✅ complete".bright_green().to_string()
        } else {
            status.bright_yellow().to_string()
        }
    );
    println!(
        "  {:<16} {}/{} gates",
        "Gates:".dimmed(),
        done_gates,
        total_gates
    );
    println!("  {:<16} {}", "Tags:".dimmed(), tags.join(", ").dimmed());
    if commit_count > 0 {
        println!(
            "  {:<16} {} commits touch this intent",
            "History:".dimmed(),
            commit_count.to_string().bright_cyan()
        );
    }
    println!();
    println!("  {} What this intent built:", "▶".bright_cyan());
    // Extract completed gates as the story
    let completed: Vec<String> = content
        .lines()
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
    if let Some(phrase_start) = content.find("**\"") {
        let phrase_end = content[phrase_start + 3..]
            .find("\"**")
            .map(|i| phrase_start + 3 + i)
            .unwrap_or(content.len());
        let phrase = &content[phrase_start + 3..phrase_end];
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

// ── Smart Intent Creation (INT-198) ──────────────────────────────────────────
// ── Critical Path Analysis (INT-198) ─────────────────────────────────────────
pub fn deps_critical_path(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;
    let intents = load_all(ctx);
    println!();
    println!("{}", "🔗 Critical Path Analysis".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!();
    // Find in-progress and planned intents
    let active: Vec<&Intent> = intents
        .iter()
        .filter(|i| i.status == "in-progress" || i.status == "planned")
        .collect();
    if active.is_empty() {
        println!("  {} No active or planned intents found", "○".dimmed());
        return Ok(());
    }
    // Build dependency chains
    // For each intent, compute chain length by following depends_on
    let mut chains: Vec<Vec<String>> = Vec::new();
    for intent in &active {
        let mut chain = vec![intent.id.clone()];
        let mut current_deps = intent.depends_on.clone();
        let mut visited = std::collections::HashSet::new();
        visited.insert(intent.id.clone());
        loop {
            let unmet: Vec<String> = current_deps
                .iter()
                .filter(|dep_id| {
                    !visited.contains(*dep_id)
                        && intents
                            .iter()
                            .find(|i| &i.id == *dep_id)
                            .map(|i| i.status != "complete")
                            .unwrap_or(false)
                })
                .cloned()
                .collect();
            if unmet.is_empty() {
                break;
            }
            let next = unmet[0].clone();
            visited.insert(next.clone());
            chain.push(next.clone());
            current_deps = intents
                .iter()
                .find(|i| i.id == next)
                .map(|i| i.depends_on.clone())
                .unwrap_or_default();
        }
        if chain.len() > 1 {
            chains.push(chain);
        }
    }
    // Find the longest chain — that's the critical path
    chains.sort_by_key(|b| std::cmp::Reverse(b.len()));
    if chains.is_empty() {
        println!(
            "  {} No dependency chains found in active intents",
            "○".dimmed()
        );
        println!(
            "  {} Add depends_on: [INT-NNN] to intent frontmatter to build the graph",
            "→".dimmed()
        );
    } else {
        println!(
            "  {} Critical path ({} steps):",
            "🔴".normal(),
            chains[0].len()
        );
        for (i, id) in chains[0].iter().enumerate() {
            if let Some(intent) = intents.iter().find(|x| &x.id == id) {
                let arrow = if i == 0 {
                    "▶".bright_red().to_string()
                } else if i == chains[0].len() - 1 {
                    "🏁".to_string()
                } else {
                    "→".bright_yellow().to_string()
                };
                println!(
                    "    {} INT-{} — {} [{}]",
                    arrow,
                    intent.id.bright_white(),
                    intent.title.normal(),
                    intent.status_colored()
                );
            }
        }
        println!();
        if chains.len() > 1 {
            println!("  {} Other dependency chains:", "▶".bright_cyan());
            for chain in chains.iter().skip(1).take(3) {
                let chain_str: Vec<String> = chain.iter().map(|id| format!("INT-{}", id)).collect();
                println!(
                    "    {} {} ({} steps)",
                    "·".dimmed(),
                    chain_str.join(" → ").dimmed(),
                    chain.len()
                );
            }
            println!();
        }
    }
    // Show all active intents with their blocker status
    println!("  {} Active intent status:", "▶".bright_cyan());
    for intent in &active {
        let unmet_deps: Vec<String> = intent
            .depends_on
            .iter()
            .filter(|dep_id| {
                intents
                    .iter()
                    .find(|i| &i.id == *dep_id)
                    .map(|i| i.status != "complete")
                    .unwrap_or(false)
            })
            .map(|id| format!("INT-{}", id))
            .collect();
        let status_icon = if unmet_deps.is_empty() {
            "✅".to_string()
        } else {
            "⏳".to_string()
        };
        println!(
            "    {} INT-{} — {}",
            status_icon,
            intent.id.bright_white(),
            intent.title.dimmed()
        );
        if !unmet_deps.is_empty() {
            println!(
                "       {} waiting on: {}",
                "⬆".bright_yellow(),
                unmet_deps.join(", ").bright_red()
            );
        }
    }
    println!();
    println!("{}", "━".repeat(56).dimmed());
    Ok(())
}

// ── INT-247: Intent Ledger v2 -- New commands ─────────────────────────────
/// Show all blocked intents and exactly what is blocking them
pub fn blocked(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;
    let intents = load_all(ctx);
    // INT-213 G4: dep_state is the single owner of what satisfies a dependency.
    let mut blocked_list: Vec<(&Intent, Vec<String>)> = Vec::new();
    let mut flagged_list: Vec<(&Intent, Vec<String>)> = Vec::new();
    for intent in intents
        .iter()
        .filter(|i| i.status == "planned" || i.status == "in-progress")
    {
        let mut unmet: Vec<String> = Vec::new();
        let mut flags: Vec<String> = Vec::new();
        for dep in &intent.depends_on {
            match dep_state(&intents, dep) {
                DepState::Satisfied => {}
                DepState::Blocked => {
                    let status = intents
                        .iter()
                        .find(|i| &i.id == dep)
                        .map(|i| i.status.clone())
                        .unwrap_or_default();
                    unmet.push(format!("INT-{} ({})", dep, status));
                }
                DepState::Questionable => {
                    flags.push(format!("INT-{} (cancelled) -- {}", dep, CANCELLED_DEP_NOTE));
                }
                DepState::Missing => {
                    flags.push(format!(
                        "INT-{} names no intent -- the reference is invalid",
                        dep
                    ));
                }
            }
        }
        if !unmet.is_empty() {
            blocked_list.push((intent, unmet));
        }
        if !flags.is_empty() {
            flagged_list.push((intent, flags));
        }
    }
    println!("{}", "🔒 Blocked Intents".bold());
    println!("{}", "━".repeat(60).dimmed());
    if !flagged_list.is_empty() {
        println!(
            "  {} {} intent(s) have edges needing attention",
            "!".bright_yellow(),
            flagged_list.len()
        );
        for (intent, flags) in &flagged_list {
            println!(
                "  {} INT-{} -- {}",
                "?".dimmed(),
                intent.id.bright_white(),
                intent.title.bright_white()
            );
            for flag in flags {
                println!("       {} {}", "~".bright_yellow(), flag.bright_yellow());
            }
        }
        println!();
    }
    if blocked_list.is_empty() {
        println!(
            "  {} No blocked intents -- all dependencies satisfied",
            "✅".green()
        );
        return Ok(());
    }
    println!(
        "  {} {} intents blocked by incomplete dependencies",
        "⚠".bright_yellow(),
        blocked_list.len()
    );
    println!();
    for (intent, blockers) in &blocked_list {
        println!(
            "  {} INT-{} -- {}",
            "🔒".dimmed(),
            intent.id.bright_white(),
            intent.title.bright_white()
        );
        for blocker in blockers {
            println!(
                "       {} waiting on: {}",
                "⬆".bright_yellow(),
                blocker.bright_red()
            );
        }
    }
    println!("{}", "━".repeat(60).dimmed());
    Ok(())
}
/// Recommend the next intent with explanation
pub fn next_intent(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;
    let intents = load_all(ctx);
    // INT-213 G4/G5: dep_state owns satisfaction, and a skipped intent is never silent.
    let mut scored: Vec<(&Intent, u32, Vec<String>)> = Vec::new();
    let mut skipped: Vec<(&Intent, Vec<String>)> = Vec::new();
    for intent in intents.iter().filter(|i| i.status == "planned") {
        let mut unmet: Vec<String> = Vec::new();
        for dep in &intent.depends_on {
            match dep_state(&intents, dep) {
                DepState::Satisfied | DepState::Questionable => {}
                DepState::Blocked => unmet.push(format!("INT-{}", dep)),
                DepState::Missing => {
                    unmet.push(format!("INT-{} (names no intent -- invalid edge)", dep))
                }
            }
        }
        if !unmet.is_empty() {
            skipped.push((intent, unmet));
            continue;
        }
        let mut score = 0u32;
        let mut reasons = Vec::new();
        // How many intents does this unblock?
        let unblocks: usize = intents
            .iter()
            .filter(|other| other.status == "planned" && other.depends_on.contains(&intent.id))
            .count();
        if unblocks > 0 {
            score += (unblocks as u32 * 15).min(30);
            reasons.push(format!("unblocks {} other intent(s)", unblocks));
        }
        // INT-213 G10 / decision 142: declared priority, not tag string-matching.
        // The tag bonuses are gone deliberately -- a security tag added 20 on top of a
        // declared priority, counting the same intent twice, and no reader ever declared
        // that a tag should rank anything. An empty priority scores nothing: undeclared
        // is not low, it means the intent has never been ranked.
        match intent.priority.as_str() {
            "high" => {
                score += 40;
                reasons.push("declared priority: high".to_string());
            }
            "medium" => {
                score += 20;
                reasons.push("declared priority: medium".to_string());
            }
            "low" => {
                score += 5;
                reasons.push("declared priority: low".to_string());
            }
            "" => {}
            other => {
                reasons.push(format!("unrecognised priority '{}' -- not scored", other));
            }
        }
        // Base score for being unblocked
        score += 10;
        reasons.push("no blocking dependencies".to_string());
        scored.push((intent, score, reasons));
    }
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    println!("{}", "Next Intent Recommendation".bold());
    println!("{}", "━".repeat(60).dimmed());
    if scored.is_empty() {
        println!("  {} No unblocked planned intents found", "○".dimmed());
    }
    if let Some((top, score, reasons)) = scored.first() {
        println!(
            "  {} Recommended: INT-{}",
            "→".bright_green(),
            top.id.bright_white()
        );
        println!("  {} {}", "  ".dimmed(), top.title.bright_white());
        println!(
            "  {} Priority score: {}/100",
            "  ".dimmed(),
            score.to_string().bright_green()
        );
        println!();
        println!("  {} Why this intent:", "→".dimmed());
        for reason in reasons {
            println!("    {} {}", "◦".dimmed(), reason.bright_white());
        }
        if scored.len() > 1 {
            println!();
            println!("{}", "  Other ready intents:".dimmed());
            for (intent, score, _) in scored.iter().skip(1).take(3) {
                println!(
                    "    {} INT-{} -- {} (score: {})",
                    "◦".dimmed(),
                    intent.id.dimmed(),
                    intent.title.dimmed(),
                    score.to_string().dimmed()
                );
            }
        }
        println!("{}", "━".repeat(60).dimmed());
    }
    if !skipped.is_empty() {
        println!();
        println!("  {} Not recommended, and why:", "!".dimmed());
        for (intent, blockers) in &skipped {
            println!(
                "    {} INT-{} -- waiting on {}",
                "~".dimmed(),
                intent.id.dimmed(),
                blockers.join(", ").bright_yellow()
            );
        }
        println!("{}", "━".repeat(60).dimmed());
    }
    Ok(())
}
/// Session brief -- what to work on, context from last session
pub fn brief(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;
    let intents = load_all(ctx);
    let active: Vec<&Intent> = intents
        .iter()
        .filter(|i| i.status == "in-progress")
        .collect();
    let planned_count = intents.iter().filter(|i| i.status == "planned").count();
    // A count of finished intents, not a dependency question -- not dep_state's job.
    let complete_count = intents.iter().filter(|i| i.status == "complete").count();
    println!("{}", "Session Brief".bold());
    println!("{}", "━".repeat(60).dimmed());
    // Active intents
    if active.is_empty() {
        println!("  {} No active intents", "○".dimmed());
    } else {
        println!("  {} Active:", "▸".bright_cyan());
        for intent in &active {
            println!(
                "    {} INT-{} -- {}",
                "→".bright_green(),
                intent.id.bright_white(),
                intent.title.bright_white()
            );
        }
    }
    println!();
    // State
    println!("  {} State:", "→".dimmed());
    println!(
        "    {} {} complete  {} planned",
        "◦".dimmed(),
        complete_count.to_string().bright_green(),
        planned_count.to_string().bright_white()
    );
    // What's ready to start
    let ready: Vec<&Intent> = intents
        .iter()
        .filter(|i| i.status == "planned")
        .filter(|i| {
            i.depends_on
                .iter()
                .all(|dep| dep_state(&intents, dep).clears())
        })
        .collect();
    println!();
    println!("  {} Ready to start ({}):", "✅".green(), ready.len());
    for intent in ready.iter().take(5) {
        println!(
            "    {} INT-{} -- {}",
            "◦".dimmed(),
            intent.id.bright_white(),
            intent.title.dimmed()
        );
    }
    if ready.len() > 5 {
        println!("    {} ...and {} more", "◦".dimmed(), ready.len() - 5);
    }
    println!();
    println!(
        "  {} Run: core intent next -- for priority recommendation",
        "→".dimmed()
    );
    println!("{}", "━".repeat(60).dimmed());
    Ok(())
}
/// ASCII dependency graph of planned intents
pub fn graph(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemReadHome])?;
    let intents = load_all(ctx);
    let planned: Vec<&Intent> = intents
        .iter()
        .filter(|i| i.status == "planned" || i.status == "in-progress")
        .collect();
    println!("{}", "Intent Dependency Graph".bold());
    println!("{}", "━".repeat(60).dimmed());
    println!(
        "  {} complete  {} in-progress  {} planned  {} blocked",
        "✅".green(),
        "▸".bright_cyan(),
        "○".bright_white(),
        "🔒".dimmed()
    );
    println!("{}", "━".repeat(60).dimmed());
    for intent in &planned {
        let status_icon = match intent.status.as_str() {
            "in-progress" => "▸".bright_cyan().to_string(),
            "complete" => "✅".to_string(),
            _ => {
                let blocked = intent
                    .depends_on
                    .iter()
                    .any(|dep| !dep_state(&intents, dep).clears());
                if blocked {
                    "🔒".to_string()
                } else {
                    "○".bright_white().to_string()
                }
            }
        };
        println!(
            "  {} INT-{} -- {}",
            status_icon,
            intent.id.bright_white(),
            intent.title.dimmed()
        );
        for dep in &intent.depends_on {
            if !dep_state(&intents, dep).clears() {
                if let Some(d) = intents.iter().find(|i| &i.id == dep) {
                    println!(
                        "       {} needs: INT-{} [{}]",
                        "⬆".bright_yellow(),
                        d.id.bright_red(),
                        d.status.dimmed()
                    );
                }
            }
        }
    }
    println!("{}", "━".repeat(60).dimmed());
    Ok(())
}

pub fn cancel_intent(ctx: &AppContext, id: &str, reason: &str) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemWriteHome])?;
    if reason.trim().is_empty() {
        return Err(crate::errors::CoreError::Domain {
            domain: "intent".to_string(),
            message: "cancel requires a non-empty --reason: a cancelled intent must record why"
                .to_string(),
        });
    }
    let base = intents_dir(ctx);
    // Active folders only -- you cancel work in flight, not something already done.
    let folders = ["in-progress", "future", "planned", "deferred"];
    let mut found_path: Option<std::path::PathBuf> = None;
    for folder in &folders {
        let dir = base.join(folder);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("{}-", id)) && name.ends_with(".md") {
                    found_path = Some(entry.path());
                    break;
                }
            }
        }
        if found_path.is_some() {
            break;
        }
    }
    let path = found_path.ok_or_else(|| crate::errors::CoreError::Domain {
        domain: "intent".to_string(),
        message: format!(
            "Active intent {} not found (already complete or cancelled?)",
            id
        ),
    })?;
    let content = std::fs::read_to_string(&path)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let updated = content
        .replace("status: in-progress", "status: cancelled")
        .replace("status: planned", "status: cancelled")
        .replace("status: future", "status: cancelled")
        .replace("status: deferred", "status: cancelled");
    let stamp = format!(
        "🚫 {} -- cancelled: {} -- approved by: christian {}",
        id, reason, today
    );
    let stamped = if updated.contains("## Gate Check") {
        updated.replace("## Gate Check", &format!("## Gate Check\n{}", stamp))
    } else {
        format!("{}\n\n## Gate Check\n{}\n", updated.trim_end(), stamp)
    };
    let cancelled_dir = base.join("cancelled");
    std::fs::create_dir_all(&cancelled_dir)?;
    let filename = path
        .file_name()
        .ok_or_else(|| crate::errors::CoreError::Domain {
            domain: "intent".to_string(),
            message: "intent file has no name".to_string(),
        })?;
    let dest = cancelled_dir.join(filename);
    std::fs::write(&dest, stamped)?;
    if dest != path {
        let _ = std::fs::remove_file(&path);
    }
    // Clear focus if the cancelled intent was the focused one.
    if let Some(state) = read_focus() {
        if state.id == id {
            let _ = std::fs::remove_file(focus_file());
        }
    }
    println!("{}", "🚫 Intent cancelled".red().bold());
    println!("  Intent:  INT-{}", id);
    println!("  Reason:  {}", reason);
    println!("  Date:    {}", today);
    println!("  Moved:   intents/cancelled/");
    // INT-066 criterion 8: warn about active intents that still depend on this one.
    let dependents: Vec<String> = load_all(ctx)
        .iter()
        .filter(|i| i.status != "cancelled" && i.depends_on.iter().any(|d| d.as_str() == id))
        .map(|i| format!("INT-{} -- {} [{}]", i.id, i.title, i.status))
        .collect();
    if !dependents.is_empty() {
        println!();
        println!(
            "  {} {} intent(s) still depend on INT-{}:",
            "⚠".yellow().bold(),
            dependents.len(),
            id
        );
        for d in &dependents {
            println!("    {} {}", "→".yellow(), d);
        }
        println!(
            "  {} Review these -- their dependency now points at a cancelled intent.",
            "→".dimmed()
        );
    }
    Ok(())
}

pub fn defer_intent(ctx: &AppContext, id: &str, reason: &str) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemWriteHome])?;
    let base = intents_dir(ctx);
    let folders = ["in-progress", "future", "complete"];
    let mut found_path: Option<std::path::PathBuf> = None;
    for folder in &folders {
        let dir = base.join(folder);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("{}-", id)) && name.ends_with(".md") {
                    found_path = Some(entry.path());
                    break;
                }
            }
        }
        if found_path.is_some() {
            break;
        }
    }
    let path = found_path.ok_or_else(|| crate::errors::CoreError::Domain {
        domain: "intent".to_string(),
        message: format!("Intent {} not found", id),
    })?;
    let content = std::fs::read_to_string(&path)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let deferral = format!(
        "\n⏸ {} -- deferred: {} -- approved by: christian {}\n",
        id, reason, today
    );
    let new_content = if content.contains("## Gate Check") {
        content.replace(
            "## Gate Check",
            &format!("## Gate Check\n{}", deferral.trim()),
        )
    } else {
        format!("{}\n## Gate Check\n{}", content.trim(), deferral.trim())
    };
    std::fs::write(&path, new_content)?;
    println!("{}", "⏸ Gate deferred".yellow().bold());
    println!("  Intent:  INT-{}", id);
    println!("  Reason:  {}", reason);
    println!("  Date:    {}", today);
    Ok(())
}

pub fn override_intent(ctx: &AppContext, id: &str, reason: &str) -> CoreResult<()> {
    ctx.capabilities
        .require("intent", &[Capability::FilesystemWriteHome])?;
    let base = intents_dir(ctx);
    let folders = ["in-progress", "future", "complete"];
    let mut found_path: Option<std::path::PathBuf> = None;
    for folder in &folders {
        let dir = base.join(folder);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("{}-", id)) && name.ends_with(".md") {
                    found_path = Some(entry.path());
                    break;
                }
            }
        }
        if found_path.is_some() {
            break;
        }
    }
    let path = found_path.ok_or_else(|| crate::errors::CoreError::Domain {
        domain: "intent".to_string(),
        message: format!("Intent {} not found", id),
    })?;
    let content = std::fs::read_to_string(&path)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    // Log override to state.db
    if let Ok(conn) = zero_core::state_db::open_state_db() {
        let _ = conn.execute(
            "INSERT INTO integrity_log (category, check_name, severity, description, weight, fixed, detected_at)
             VALUES ('intent', 'gate_override', 'propose', ?1, 1, 1, strftime('%s','now'))",
            rusqlite::params![format!("INT-{} gate overridden: {}", id, reason)],
        );
    }
    let override_note = format!(
        "\n✅ OVERRIDE: {} -- approved by: christian {} -- reason: {}\n",
        id, today, reason
    );
    let new_content = if content.contains("## Gate Check") {
        content.replace(
            "## Gate Check",
            &format!("## Gate Check\n{}", override_note.trim()),
        )
    } else {
        format!(
            "{}\n## Gate Check\n{}",
            content.trim(),
            override_note.trim()
        )
    };
    std::fs::write(&path, new_content)?;
    println!("{}", "✅ Gate overridden".green().bold());
    println!("  Intent:  INT-{}", id);
    println!("  Reason:  {}", reason);
    println!("  Date:    {}", today);
    println!(
        "{}",
        "  ⚠️  Override logged to integrity audit trail".yellow()
    );
    Ok(())
}

#[cfg(test)]
mod absent_ledger_tests {
    /// ⚠️ FOUND IN THE VM, 2026-08-23. A guest with the tools deployed but no 0-Core checkout ran
    /// the doctor and got `✅ Intent Ledger -- 0 intents, all valid` about a directory that does
    /// not exist. Nine folders, each read with `let Ok(entries) = read_dir(..) else { continue }`,
    /// so an absent ledger and an empty one produced identical output.
    ///
    /// ⭐ THE TEST DRIVES THE REAL FUNCTION THROUGH `HOME`, because that is what
    /// `paths::intents_dir()` reads -- the same technique that proved the platform degrades in
    /// INT-227, and it needs no second machine.
    #[test]
    fn an_absent_ledger_is_an_issue_not_a_clean_bill() {
        let dir = std::env::temp_dir().join("core_absent_ledger_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp home");
        // Shares the lock with the doctor's registry tests -- see its comment. Both mutate HOME,
        // and cargo runs them in parallel.
        let _guard = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("HOME").ok();
        // SAFETY: single-threaded test, restored below.
        unsafe { std::env::set_var("HOME", &dir) };
        let (count, issues) = super::validate_issues();
        match prev {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(count, 0, "nothing was counted, which is true");
        assert!(
            !issues.is_empty(),
            "but silence about it is the defect: an absent ledger must not read as a clean bill"
        );
        assert!(
            issues[0].contains("not found"),
            "and it must name the real problem: {}",
            issues[0]
        );
    }
}

#[cfg(test)]
mod template_tests {
    /// INT-247 LAYER 0: NOTHING NEW TAKES THE RETIRED NAME. The wizard broke that rule on every use:
    /// `inta` offered the retired name as its default tag, so pressing Enter filed it into the
    /// ledger that is being renamed. Found 2026-09-24.
    ///
    /// Checks EVERY template, not the one that was noticed -- `feature` carried it too. And it
    /// matches any tag CONTAINING the name, so a hyphenated variant cannot slip past.
    #[test]
    fn no_template_defaults_to_the_retired_name() {
        let offenders: Vec<&str> = super::TEMPLATES
            .iter()
            .filter(|(_, _, tags, _)| {
                tags.split(',')
                    .any(|t| t.trim().to_lowercase().contains(concat!("fae", "light")))
            })
            .map(|(name, _, _, _)| *name)
            .collect();
        assert!(
            offenders.is_empty(),
            "templates still default to the retired name: {:?}",
            offenders
        );
    }

    /// The wizard's fallback is TEMPLATES' future row (INT-247 Layer 0), so the row must
    /// exist. Without it inta would panic instead of offering a default.
    #[test]
    fn future_row_exists() {
        assert!(super::TEMPLATES.iter().any(|(t, ..)| *t == "future"));
    }
}
