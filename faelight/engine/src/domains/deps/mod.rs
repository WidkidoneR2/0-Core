// core deps — Dependency Intelligence
// Core v7 Phase 4 — INT-122
//
// "The forest understands its own dependency tree
//  and can reason about the risk of changes."

use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn graph(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("deps", &[Capability::FilesystemReadHome])?;
    let core_root = &ctx.core_root;

    println!();
    println!(
        "{}",
        "  ╭─ 🌐 Dependency Graph ───────────────────────────────".bright_cyan()
    );
    println!("  │  Visual dependency map of all forest tools");
    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );

    let deps = gather_deps(core_root);
    let mut domain_map: HashMap<String, Vec<String>> = HashMap::new();

    for tool in deps.keys() {
        let domain = categorize_tool(tool);
        domain_map.entry(domain).or_default().push(tool.clone());
    }

    let mut domains: Vec<String> = domain_map.keys().cloned().collect();
    domains.sort();

    for domain in &domains {
        let tools = &domain_map[domain];
        println!(
            "  │  {} {}",
            domain_icon(domain),
            domain.bright_white().bold()
        );
        for tool in tools {
            let tool_deps = deps.get(tool).map(|d| d.len()).unwrap_or(0);
            println!(
                "  │    {} {}  {} deps",
                "├─".dimmed(),
                tool.bright_cyan(),
                tool_deps.to_string().dimmed()
            );
        }
        println!("  │");
    }

    // Summary
    let total_tools = deps.len();
    let total_deps: usize = deps.values().map(|d| d.len()).sum();
    let avg_deps = if total_tools > 0 {
        total_deps / total_tools
    } else {
        0
    };

    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  │  {} tools  |  {} total deps  |  {} avg per tool",
        total_tools.to_string().bright_white(),
        total_deps.to_string().bright_white(),
        avg_deps.to_string().bright_white()
    );
    println!(
        "{}",
        "  ╰────────────────────────────────────────────────────".dimmed()
    );
    println!();

    emit_event(ctx, "graph");
    Ok(())
}

pub fn risk(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("deps", &[Capability::FilesystemReadHome])?;
    let core_root = &ctx.core_root;

    println!();
    println!(
        "{}",
        "  ╭─ ⚠️  Dependency Risk ────────────────────────────────".bright_cyan()
    );
    println!("  │  Which dependencies carry the most risk?");
    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );

    // Parse Cargo.lock for all deps and their usage count
    let lock_path = PathBuf::from(core_root).join("Cargo.lock");
    let mut dep_usage: HashMap<String, usize> = HashMap::new();
    let mut dep_versions: HashMap<String, String> = HashMap::new();

    if let Ok(content) = std::fs::read_to_string(&lock_path) {
        let mut current_name = String::new();
        let mut current_version = String::new();
        let _ = &current_version;
        for line in content.lines() {
            if line.starts_with("name = ") {
                current_name = line
                    .trim_start_matches("name = ")
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("version = ") {
                let ver = line
                    .trim_start_matches("version = ")
                    .trim_matches('"')
                    .to_string();
                if !current_name.is_empty() {
                    current_version = ver.clone();
                    *dep_usage.entry(current_name.clone()).or_insert(0) += 1;
                    dep_versions.insert(current_name.clone(), current_version.clone());
                }
            }
        }
    }

    // High-risk = used by many tools (high coupling)
    let mut risk_list: Vec<(String, usize, String)> = dep_usage
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(name, count)| {
            let version = dep_versions.get(name).cloned().unwrap_or_default();
            (name.clone(), *count, version)
        })
        .collect();

    risk_list.sort_by(|a, b| b.1.cmp(&a.1));

    println!(
        "  │  {} High-coupling dependencies (used by 2+ tools):",
        "⚠".yellow()
    );
    println!("  │");
    for (name, count, version) in risk_list.iter().take(15) {
        let risk_level = if *count > 10 {
            "HIGH".bright_red()
        } else if *count > 5 {
            "MED".yellow()
        } else {
            "LOW".dimmed()
        };
        println!(
            "  │  {:<30} v{:<12} {} tools  {}",
            name.bright_white(),
            version.dimmed(),
            count.to_string().bright_yellow(),
            risk_level
        );
    }

    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );
    println!("  │  💡 High-coupling deps are upgrade risks");
    println!(
        "  │  Run {} before upgrading",
        "core security simulate <dep>".bright_cyan()
    );
    println!(
        "{}",
        "  ╰────────────────────────────────────────────────────".dimmed()
    );
    println!();

    emit_event(ctx, "risk");
    Ok(())
}

pub fn audit(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("deps", &[Capability::FilesystemReadHome])?;
    let _core_root = &ctx.core_root;

    println!();
    println!(
        "{}",
        "  ╭─ 🔍 Dependency Audit ──────────────────────────────".bright_cyan()
    );
    println!("  │  Cross-reference deps with decision history");
    println!(
        "{}",
        "  ├────────────────────────────────────────────────────".dimmed()
    );

    // Get decisions from ledger
    let decisions: Vec<String> = {
        let mut stmt = ctx
            .runtime
            .db
            .prepare("SELECT description FROM decisions ORDER BY timestamp DESC")
            .ok();
        if let Some(ref mut s) = stmt {
            s.query_map([], |r| r.get::<_, String>(0))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default()
        } else {
            vec![]
        }
    };

    // Key deps to audit
    let key_deps = [
        "openssl", "rusqlite", "tokio", "ratatui", "smithay", "wayland", "rand", "serde", "clap",
        "colored",
    ];

    println!("  │  🔍 Key dependency audit:");
    println!("  │");

    for dep in &key_deps {
        // Check if this dep has any decision records
        let has_decision = decisions.iter().any(|d| d.to_lowercase().contains(dep));

        let icon = if has_decision {
            "✅".to_string()
        } else {
            "○ ".to_string()
        };
        let note = if has_decision {
            "decision recorded".green().to_string()
        } else {
            "no decision record".dimmed().to_string()
        };

        println!("  │  {} {:<20} {}", icon, dep.bright_white(), note);
    }

    println!("  │");
    println!(
        "  │  {} Record dependency decisions with:",
        "💡".to_string().dimmed()
    );
    println!("  │  {}", "core decide".bright_cyan());
    println!(
        "{}",
        "  ╰────────────────────────────────────────────────────".dimmed()
    );
    println!();

    emit_event(ctx, "audit");
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn gather_deps(_core_root: &str) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();
    let tools_dir = faelight_core::paths::rust_tools_dir();

    if let Ok(entries) = std::fs::read_dir(&tools_dir) {
        for entry in entries.flatten() {
            let cargo_toml = entry.path().join("Cargo.toml");
            if !cargo_toml.exists() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(".archived") {
                continue;
            }

            let deps = std::fs::read_to_string(&cargo_toml)
                .map(|t| {
                    let mut in_deps = false;
                    t.lines()
                        .filter_map(|l| {
                            if l.starts_with("[dependencies]") {
                                in_deps = true;
                                return None;
                            }
                            if l.starts_with('[') && l != "[dependencies]" {
                                in_deps = false;
                            }
                            if in_deps && l.contains('=') {
                                Some(l.split('=').next().unwrap_or("").trim().to_string())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            result.insert(name, deps);
        }
    }
    result
}

fn categorize_tool(name: &str) -> String {
    if name.starts_with("faelight-") {
        let suffix = name.trim_start_matches("faelight-");
        match suffix {
            "shell" | "term" | "browser" => "Shell & Terminal".to_string(),
            "bar" | "menu" | "palette" | "wallpaper" => "UI & Display".to_string(),
            "git" | "snapshot" | "release" => "Version Control".to_string(),
            "sandbox" | "gen" | "vault" => "Security".to_string(),
            "compositor" | "idle" | "notify" | "lock" => "Compositor".to_string(),
            "fm" | "fetch" | "link" | "zone" => "System Tools".to_string(),
            _ => "Zero Core".to_string(),
        }
    } else if name == "core" || name == "engine" {
        "Core Engine".to_string()
    } else {
        "Utilities".to_string()
    }
}

fn domain_icon(domain: &str) -> &'static str {
    match domain {
        "Shell & Terminal" => "🐚",
        "UI & Display" => "🖥",
        "Version Control" => "🌿",
        "Security" => "🔒",
        "Compositor" => "🪟",
        "System Tools" => "🛠",
        "Zero Core" => "◉",
        "Core Engine" => "⚙",
        _ => "○",
    }
}

fn emit_event(ctx: &AppContext, action: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let payload = format!(
        r#"{{"actor":"core","result":"ok","detail":{{"command":"deps.{}"}}}}"#,
        action
    );
    ctx.runtime
        .db
        .execute(
            "INSERT INTO events (domain, action, payload, timestamp) VALUES ('deps', ?1, ?2, ?3)",
            rusqlite::params![action, payload, ts],
        )
        .ok();
    crate::runtime::write_event_log("deps", action, &payload, ts);
}

// ─── Coordinator -- Registry-based Tool Dependency Planning (INT-251 Pillar 4) ──

/// Parse registry/tools.toml and return a map of tool -> depends_on list.
fn read_tool_registry(_core_root: &str) -> HashMap<String, Vec<String>> {
    let path = faelight_core::paths::tools_registry();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    let mut result = HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_deps: Vec<String> = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line == "[[tool]]" {
            if let Some(name) = current_name.take() {
                result.insert(name, current_deps.clone());
                current_deps.clear();
            }
        } else if let Some(rest) = line.strip_prefix("name = ") {
            current_name = Some(rest.trim().trim_matches('"').to_string());
        } else if let Some(rest) = line.strip_prefix("depends_on = ") {
            current_deps = rest
                .trim()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    if let Some(name) = current_name {
        result.insert(name, current_deps);
    }
    result
}

/// Iterative post-order DFS: produces topological deployment order for a tool.
fn collect_ordered_deps(tool: &str, registry: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut stack: Vec<(String, bool)> = vec![(tool.to_string(), false)];
    while let Some((t, processed)) = stack.pop() {
        if processed {
            if !order.contains(&t) {
                order.push(t);
            }
            continue;
        }
        if visited.contains(&t) {
            continue;
        }
        visited.insert(t.clone());
        stack.push((t.clone(), true));
        if let Some(deps) = registry.get(&t) {
            for dep in deps.iter().rev() {
                if !visited.contains(dep) {
                    stack.push((dep.clone(), false));
                }
            }
        }
    }
    order
}

/// `core deps plan <tool>` -- show ordered deployment plan respecting depends_on.
/// INT-251 v23 Pillar 4: coordinator schedules cooperative work in correct order.
pub fn plan(ctx: &AppContext, tool: &str) -> CoreResult<()> {
    use colored::*;
    let registry = read_tool_registry(&ctx.core_root);
    let order = collect_ordered_deps(tool, &registry);
    println!();
    println!("  {} Coordinator -- Deployment Plan", "🌲".normal());
    println!("  {}", "━".repeat(50).dimmed());
    println!();
    if order.len() == 1 {
        println!(
            "  {} {} has no declared dependencies -- deploy directly.",
            "✅".normal(),
            tool.bright_white()
        );
    } else {
        println!("  {} Target: {}", "→".dimmed(), tool.bright_white());
        println!();
        for (i, step) in order.iter().enumerate() {
            let step_num = i + 1;
            if step == tool {
                println!(
                    "  {} Step {}: {} {}",
                    "→".bright_green(),
                    step_num,
                    step.bright_white(),
                    "(target)".dimmed()
                );
            } else {
                println!(
                    "  {} Step {}: {} {}",
                    "·".bright_cyan(),
                    step_num,
                    step.bright_cyan(),
                    "(dependency)".dimmed()
                );
            }
        }
    }
    println!();
    Ok(())
}

/// core deps blocked -- show tools with unresolved depends_on (not deployed).
/// INT-251 v23 Pillar 4: pre-action surface of blocked work.
pub fn blocked(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;
    let registry = read_tool_registry(&ctx.core_root);
    let scripts = PathBuf::from(&ctx.core_root).join("scripts");
    let home = std::env::var("HOME").unwrap_or_default();
    let cargo_bin = PathBuf::from(&home).join(".cargo/bin");
    println!();
    println!("  {} Coordinator -- Blocked Tools", "🌲".normal());
    println!("  {}", "━".repeat(50).dimmed());
    println!();
    let mut any_blocked = false;
    let mut tools_with_deps: Vec<(&String, &Vec<String>)> = registry
        .iter()
        .filter(|(_, deps)| !deps.is_empty())
        .collect();
    tools_with_deps.sort_by_key(|(name, _)| name.as_str());
    for (tool, deps) in &tools_with_deps {
        let unresolved: Vec<&String> = deps
            .iter()
            .filter(|dep| {
                !scripts.join(dep.as_str()).exists() && !cargo_bin.join(dep.as_str()).exists()
            })
            .collect();
        if !unresolved.is_empty() {
            any_blocked = true;
            println!(
                "  {} {} is waiting for prerequisites:",
                "⚠️ ".normal(),
                tool.bright_white()
            );
            for dep in &unresolved {
                println!(
                    "    {} deploy {} is waiting for {} to deploy first",
                    "→".bright_yellow(),
                    tool.bright_yellow(),
                    dep.bright_yellow()
                );
            }
        } else {
            println!(
                "  {} {} -- all {} dep(s) resolved",
                "✅".normal(),
                tool.bright_white(),
                deps.len()
            );
        }
    }
    if !any_blocked {
        println!(
            "  {} No blocked tools -- all declared dependencies are resolved.",
            "✅".normal()
        );
    }
    if tools_with_deps.is_empty() {
        println!(
            "  {} No tools declare dependencies in the registry.",
            "💡".dimmed()
        );
    }
    println!();
    Ok(())
}
