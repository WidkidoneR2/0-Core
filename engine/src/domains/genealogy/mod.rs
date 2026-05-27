use rusqlite;
use chrono;
// INT-153 — Intent Genealogy Domain
// The forest remembers how it grew.
// Every intent has parents and children.
// Genealogy makes the lineage visible.

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
struct IntentNode {
    id: String,
    title: String,
    status: String,
    spawned_by: Option<String>,
    spawns: Vec<String>,
    depends_on: Vec<String>,
}

fn load_all_intents(core_root: &str) -> Vec<IntentNode> {
    let root = PathBuf::from(core_root);
    let mut nodes = Vec::new();
    for dir in &["complete", "future", "decisions", "incidents"] {
        if let Ok(entries) = std::fs::read_dir(root.join("intents").join(dir)) {
            for entry in entries.flatten() {
                if !entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                    continue;
                }
                let Ok(content) = std::fs::read_to_string(entry.path()) else {
                    continue;
                };
                let id = content
                    .lines()
                    .find(|l| l.starts_with("id:"))
                    .map(|l| l.trim_start_matches("id:").trim().to_string())
                    .unwrap_or_default();
                if id.is_empty() {
                    continue;
                }
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
                let status = content
                    .lines()
                    .find(|l| l.starts_with("status:"))
                    .map(|l| l.trim_start_matches("status:").trim().to_string())
                    .unwrap_or_default();
                let spawned_by = content
                    .lines()
                    .find(|l| l.starts_with("spawned_by:"))
                    .map(|l| l.trim_start_matches("spawned_by:").trim().to_string())
                    .filter(|s| s != "null" && !s.is_empty());
                let spawns = content
                    .lines()
                    .find(|l| l.starts_with("spawns:"))
                    .map(|l| {
                        l.trim_start_matches("spawns:")
                            .trim()
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                let depends_on = content
                    .lines()
                    .find(|l| l.starts_with("depends_on:"))
                    .map(|l| {
                        l.trim_start_matches("depends_on:")
                            .trim()
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                nodes.push(IntentNode {
                    id,
                    title,
                    status,
                    spawned_by,
                    spawns,
                    depends_on,
                });
            }
        }
    }
    nodes
}

fn status_icon(status: &str) -> &'static str {
    match status {
        "complete" => "✅",
        "in-progress" => "🔄",
        "planned" => "⬜",
        "cancelled" => "❌",
        _ => "·",
    }
}

/// core genealogy show <id> — show lineage of a specific intent
pub fn show(ctx: &AppContext, id: &str) -> CoreResult<()> {
    let nodes = load_all_intents(&ctx.core_root);
    let map: HashMap<String, &IntentNode> = nodes.iter().map(|n| (n.id.clone(), n)).collect();

    let target = match map.get(id) {
        Some(n) => n,
        None => {
            println!();
            println!(
                "  {} Intent {} not found",
                "❌".bright_red(),
                id.bright_white()
            );
            println!(
                "  {} Run: core genealogy tree — to see all intents",
                "·".dimmed()
            );
            println!();
            return Ok(());
        }
    };

    println!();
    println!("  {}", "🌲 Genealogy".bright_green().bold());
    println!("  {}", format!("Lineage of INT-{}", id).dimmed());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    // Ancestors — walk up the chain
    println!(
        "  {} {}",
        "▶".bright_cyan(),
        "Ancestors:".bright_white().bold()
    );
    let mut ancestors: Vec<String> = Vec::new();
    let mut current = target.spawned_by.clone();
    while let Some(ref parent_id) = current.clone() {
        if let Some(parent) = map.get(parent_id) {
            ancestors.push(parent_id.clone());
            current = parent.spawned_by.clone();
        } else {
            break;
        }
    }
    if ancestors.is_empty() {
        println!("    {} This is a root intent — no ancestors", "·".dimmed());
    } else {
        ancestors.reverse();
        for (i, anc_id) in ancestors.iter().enumerate() {
            if let Some(anc) = map.get(anc_id) {
                let indent = "  ".repeat(i);
                println!(
                    "    {}{} INT-{}: {}",
                    indent,
                    status_icon(&anc.status),
                    anc_id.bright_white(),
                    anc.title.dimmed()
                );
            }
        }
        // Show target
        let indent = "  ".repeat(ancestors.len());
        println!(
            "    {}{} INT-{}: {} ← {}",
            indent,
            status_icon(&target.status),
            id.bright_yellow().bold(),
            target.title.bright_white(),
            "YOU ARE HERE".bright_yellow()
        );
    }
    println!();

    // Descendants — walk down the chain
    println!(
        "  {} {}",
        "▶".bright_cyan(),
        "Descendants:".bright_white().bold()
    );
    fn print_descendants(node_id: &str, map: &HashMap<String, &IntentNode>, depth: usize) {
        if let Some(node) = map.get(node_id) {
            for child_id in &node.spawns {
                if let Some(child) = map.get(child_id) {
                    let indent = "  ".repeat(depth);
                    println!(
                        "    {}{} INT-{}: {}",
                        indent,
                        status_icon(&child.status),
                        child_id.bright_white(),
                        child.title.dimmed()
                    );
                    print_descendants(child_id, map, depth + 1);
                }
            }
        }
    }
    if target.spawns.is_empty() {
        println!("    {} This intent has no known children yet", "·".dimmed());
    } else {
        print_descendants(id, &map, 0);
    }
    println!();

    // Dependencies
    if !target.depends_on.is_empty() {
        println!(
            "  {} {}",
            "▶".bright_cyan(),
            "Depends on:".bright_white().bold()
        );
        for dep_id in &target.depends_on {
            if let Some(dep) = map.get(dep_id) {
                println!(
                    "    {} INT-{}: {}",
                    status_icon(&dep.status),
                    dep_id.bright_white(),
                    dep.title.dimmed()
                );
            }
        }
        println!();
    }

    Ok(())
}

/// core genealogy tree — show full intent family tree
pub fn tree(ctx: &AppContext) -> CoreResult<()> {
    let nodes = load_all_intents(&ctx.core_root);
    let map: HashMap<String, &IntentNode> = nodes.iter().map(|n| (n.id.clone(), n)).collect();

    // Find roots — intents with no spawned_by
    let mut roots: Vec<&IntentNode> = nodes
        .iter()
        .filter(|n| n.spawned_by.is_none() && !n.spawns.is_empty())
        .collect();
    roots.sort_by(|a, b| a.id.cmp(&b.id));

    println!();
    println!("  {}", "🌲 Intent Family Tree".bright_green().bold());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    fn print_tree(node_id: &str, map: &HashMap<String, &IntentNode>, depth: usize, is_last: bool) {
        if let Some(node) = map.get(node_id) {
            let indent = if depth == 0 {
                String::new()
            } else {
                let base = "    ".repeat(depth - 1);
                if is_last {
                    format!("{}  └── ", base)
                } else {
                    format!("{}  ├── ", base)
                }
            };
            println!(
                "  {}{} INT-{}: {}",
                indent,
                status_icon(&node.status),
                node_id.bright_white(),
                node.title.dimmed()
            );
            for (i, child_id) in node.spawns.iter().enumerate() {
                let last = i == node.spawns.len() - 1;
                print_tree(child_id, map, depth + 1, last);
            }
        }
    }

    if roots.is_empty() {
        println!("  {} No genealogy data found", "·".dimmed());
        println!(
            "  {} Add spawned_by/spawns fields to intent frontmatter",
            "→".bright_green()
        );
    } else {
        for root in &roots {
            print_tree(&root.id, &map, 0, true);
            println!();
        }
    }

    println!(
        "  {} Legend: ✅ complete  🔄 in-progress  ⬜ planned",
        "·".dimmed()
    );
    println!();
    Ok(())
}

/// core genealogy roots — show founding intents with no parents
pub fn roots(ctx: &AppContext) -> CoreResult<()> {
    let nodes = load_all_intents(&ctx.core_root);

    let roots: Vec<&IntentNode> = nodes.iter().filter(|n| n.spawned_by.is_none()).collect();

    println!();
    println!("  {}", "🌲 Genealogy — Roots".bright_green().bold());
    println!("  {}", "Founding intents with no ancestors.".dimmed());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    let mut roots_sorted = roots;
    roots_sorted.sort_by(|a, b| a.id.cmp(&b.id));

    for node in &roots_sorted {
        let spawns_str = if node.spawns.is_empty() {
            "no children".dimmed().to_string()
        } else {
            format!("spawns: {}", node.spawns.join(", "))
                .bright_green()
                .to_string()
        };
        println!(
            "  {} INT-{}: {}  {}",
            status_icon(&node.status),
            node.id.bright_white(),
            node.title,
            spawns_str
        );
    }
    println!();
    Ok(())
}

/// INT-312 Phase 3: show full context for a commit hash
pub fn commit_show(_ctx: &AppContext, hash: &str) -> CoreResult<()> {
    use colored::Colorize;
    let db = rusqlite::Connection::open(dirs::home_dir().unwrap_or_default().join("0-core/runtime/state.db"))?;
    let result = db.query_row(
        "SELECT commit_hash, intent_id, intent_status, phase_hint, gate_hint,
                health_at, friday_facts, friday_patterns, session_id, committed_at, message
         FROM intent_commits WHERE commit_hash LIKE ?1 LIMIT 1",
        rusqlite::params![format!("{}%", hash)],
        |r| Ok((
            r.get::<_,String>(0)?, r.get::<_,Option<i64>>(1)?,
            r.get::<_,Option<String>>(2)?, r.get::<_,Option<String>>(3)?,
            r.get::<_,Option<String>>(4)?, r.get::<_,Option<i64>>(5)?,
            r.get::<_,Option<i64>>(6)?, r.get::<_,Option<i64>>(7)?,
            r.get::<_,Option<String>>(8)?, r.get::<_,Option<i64>>(9)?,
            r.get::<_,String>(10)?
        )),
    );
    println!();
    match result {
        Err(_) => {
            println!("  {} Commit {} not found in genealogy", "❌".bright_red(), hash);
            println!("  {} Run: core genealogy search <term> to find commits", "·".dimmed());
        }
        Ok((chash, intent_id, status, phase, gate, health, facts, patterns, session, ts, msg)) => {
            println!("  {} {}", "🌲 Commit Genealogy".bright_green().bold(), chash.bright_yellow());
            println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("  {} {}", "→ Message:".bright_cyan(), msg);
            if let Some(id) = intent_id {
                println!("  {} INT-{} ({})", "→ Intent:".bright_cyan(), id,
                    status.unwrap_or_else(|| "unknown".to_string()).dimmed());
            }
            if let Some(p) = phase { println!("  {} {}", "→ Phase:".bright_cyan(), p); }
            if let Some(g) = gate { println!("  {} {}", "→ Gate:".bright_cyan(), g); }
            if let Some(h) = health { println!("  {} {}%", "→ Health:".bright_cyan(), h); }
            if let Some(f) = facts { println!("  {} {} facts, {} patterns",
                "→ Friday:".bright_cyan(), f, patterns.unwrap_or(0)); }
            if let Some(s) = session { println!("  {} {}", "→ Session:".bright_cyan(), s.dimmed()); }
            if let Some(t) = ts {
                let dt = chrono::DateTime::from_timestamp(t, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| t.to_string());
                println!("  {} {}", "→ Time:".bright_cyan(), dt.dimmed());
            }
        }
    }
    println!();
    Ok(())
}

/// INT-312 Phase 3: show all commits for an intent
pub fn commits_for_intent(_ctx: &AppContext, id: &str) -> CoreResult<()> {
    use colored::Colorize;
    let db = rusqlite::Connection::open(dirs::home_dir().unwrap_or_default().join("0-core/runtime/state.db"))?;
    let intent_id: i64 = id.parse().unwrap_or(0);
    let mut stmt = db.prepare(
        "SELECT commit_hash, phase_hint, committed_at, message FROM intent_commits
         WHERE intent_id = ?1 ORDER BY committed_at ASC"
    )?;
    let rows: Vec<_> = stmt.query_map(rusqlite::params![intent_id], |r| {
        Ok((r.get::<_,String>(0)?, r.get::<_,Option<String>>(1)?,
            r.get::<_,Option<i64>>(2)?, r.get::<_,String>(3)?))
    })?.flatten().collect();
    println!();
    println!("  {} {}", "🌲 Commits for INT-".bright_green().bold(), id.bright_yellow());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    if rows.is_empty() {
        println!("  {} No commits found for INT-{}", "·".dimmed(), id);
    } else {
        println!("  {} {} commits", "→".bright_cyan(), rows.len());
        println!();
        for (hash, phase, ts, msg) in &rows {
            let phase_str = phase.as_deref().unwrap_or("");
            let short_msg = if msg.len() > 60 { &msg[..60] } else { msg };
            println!("  {} {} {} {}",
                hash[..8.min(hash.len())].bright_yellow(),
                if phase_str.is_empty() { "      ".to_string() } else { format!("[{}]", phase_str) }.bright_cyan(),
                short_msg,
                ts.map(|t| chrono::DateTime::from_timestamp(t,0)
                    .map(|d| d.format("%m-%d").to_string())
                    .unwrap_or_default()).unwrap_or_default().dimmed()
            );
        }
    }
    println!();
    Ok(())
}

/// INT-312 Phase 3: search commit genealogy
pub fn commit_search(_ctx: &AppContext, term: &str) -> CoreResult<()> {
    use colored::Colorize;
    let db = rusqlite::Connection::open(dirs::home_dir().unwrap_or_default().join("0-core/runtime/state.db"))?;
    let pattern = format!("%{}%", term);
    let mut stmt = db.prepare(
        "SELECT commit_hash, intent_id, phase_hint, committed_at, message
         FROM intent_commits WHERE message LIKE ?1 ORDER BY committed_at DESC LIMIT 20"
    )?;
    let rows: Vec<_> = stmt.query_map(rusqlite::params![pattern], |r| {
        Ok((r.get::<_,String>(0)?, r.get::<_,Option<i64>>(1)?,
            r.get::<_,Option<String>>(2)?, r.get::<_,Option<i64>>(3)?,
            r.get::<_,String>(4)?))
    })?.flatten().collect();
    println!();
    println!("  {} {}", "🌲 Genealogy Search:".bright_green().bold(), term.bright_yellow());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    if rows.is_empty() {
        println!("  {} No commits found matching: {}", "·".dimmed(), term);
    } else {
        for (hash, intent_id, _phase, ts, msg) in &rows {
            let int_str = intent_id.map(|i| format!("INT-{}", i)).unwrap_or_default();
            let short = if msg.len() > 55 { &msg[..55] } else { msg };
            println!("  {} {} {} {}",
                hash[..8.min(hash.len())].bright_yellow(),
                int_str.bright_cyan(),
                short,
                ts.map(|t| chrono::DateTime::from_timestamp(t,0)
                    .map(|d| d.format("%m-%d").to_string())
                    .unwrap_or_default()).unwrap_or_default().dimmed()
            );
        }
    }
    println!();
    Ok(())
}

