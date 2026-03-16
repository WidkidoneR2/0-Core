// core bootstrap — Bootstrap Intelligence
// Core v7 Phase 2 — INT-122
//
// "The forest reads its own intent ledger and registry
//  to generate a reconstruction plan. Not automation — guidance."

use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

// ── Commands ──────────────────────────────────────────────────────────────────

pub fn plan(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require("bootstrap", &[Capability::FilesystemReadHome])?;
    let core_root = &ctx.core_root;
    let home = std::env::var("HOME").unwrap_or_default();

    println!();
    println!("{}", "  ╭─ 🌱 Bootstrap Plan ────────────────────────────────".bright_cyan());
    println!("  │  What would it take to rebuild this forest?");
    println!("{}", "  ├────────────────────────────────────────────────────".dimmed());

    // 1. System requirements
    println!("  │  {}", "① System Requirements".bright_white().bold());
    println!("  │    OS:      Arch Linux (vanilla)");
    println!("  │    WM:      Niri compositor");
    println!("  │    Shell:   zsh + faelight-shell");
    println!("  │    Rust:    {}", get_rust_version());

    // 2. Repository
    println!("  │");
    println!("  │  {}", "② Repository".bright_white().bold());
    let remote = get_git_remote(core_root);
    println!("  │    git clone {}", remote.bright_yellow());
    println!("  │    cd 0-core");

    // 3. Tool inventory from registry
    println!("  │");
    println!("  │  {}", "③ Tool Inventory".bright_white().bold());
    let tools = read_registry_tools(core_root);
    println!("  │    {} tools registered", tools.len().to_string().bright_white());
    println!("  │    cargo build --release --workspace");
    println!("  │    {} tools to deploy to ~/0-core/scripts/", tools.len().to_string().bright_white());

    // 4. Stow interfaces
    println!("  │");
    println!("  │  {}", "④ Interfaces".bright_white().bold());
    let stow_pkgs = read_stow_packages(core_root);
    println!("  │    {} stow packages to deploy", stow_pkgs.len().to_string().bright_white());
    for pkg in &stow_pkgs {
        println!("  │      stow {}", pkg.dimmed());
    }

    // 5. Active intents
    println!("  │");
    println!("  │  {}", "⑤ Active Intents".bright_white().bold());
    let intents = read_active_intents(core_root);
    println!("  │    {} intents in-progress or planned", intents.len().to_string().bright_white());
    for intent in &intents {
        println!("  │      {} {}", "→".dimmed(), intent.dimmed());
    }

    // 6. Key decisions
    println!("  │");
    println!("  │  {}", "⑥ Key Decisions".bright_white().bold());
    let decisions = read_decisions(ctx);
    println!("  │    {} decisions recorded", decisions.len().to_string().bright_white());
    for d in decisions.iter().take(3) {
        println!("  │      {} {}", "→".dimmed(), d.dimmed());
    }

    println!("{}", "  ╰────────────────────────────────────────────────────".dimmed());
    println!();
    println!("  {} Full plan assumes git history intact", "💡".yellow());
    println!("  {} Run {} for state consistency check", "💡".yellow(), "core bootstrap verify".bright_cyan());
    println!();

    emit_event(ctx, "plan");
    Ok(())
}

pub fn verify(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require("bootstrap", &[Capability::FilesystemReadHome])?;
    let core_root = &ctx.core_root;

    println!();
    println!("{}", "  ╭─ 🔍 Bootstrap Verify ──────────────────────────────".bright_cyan());
    println!("  │  Is the current state consistent with history?");
    println!("{}", "  ├────────────────────────────────────────────────────".dimmed());

    let mut issues = Vec::new();
    let mut passed = 0;

    // Check 1 — registry tools deployed
    let tools = read_registry_tools(core_root);
    let scripts_dir = PathBuf::from(core_root).join("scripts");
    let mut missing = Vec::new();
    for tool in &tools {
        if !scripts_dir.join(tool).exists() {
            missing.push(tool.clone());
        }
    }
    if missing.is_empty() {
        println!("  │  {} All {} registered tools deployed", "✅".green(), tools.len());
        passed += 1;
    } else {
        println!("  │  {} {} tools not deployed: {}", "⚠".yellow(), missing.len(),
            missing[..missing.len().min(3)].join(", ").bright_red());
        issues.push(format!("{} tools not deployed", missing.len()));
    }

    // Check 2 — stow packages linked
    let stow_pkgs = read_stow_packages(core_root);
    let home = std::env::var("HOME").unwrap_or_default();
    let stow_src = PathBuf::from(core_root).join("03-interfaces/stow");
    let mut unstowed = Vec::new();
    for pkg in &stow_pkgs {
        let pkg_path = stow_src.join(pkg);
        if !pkg_path.exists() {
            unstowed.push(pkg.clone());
        }
    }
    if unstowed.is_empty() {
        println!("  │  {} All {} stow packages present", "✅".green(), stow_pkgs.len());
        passed += 1;
    } else {
        println!("  │  {} {} stow packages missing", "⚠".yellow(), unstowed.len());
        issues.push(format!("{} stow packages missing", unstowed.len()));
    }

    // Check 3 — git history intact
    let commit_count = get_commit_count(core_root);
    if commit_count > 100 {
        println!("  │  {} Git history intact ({} commits)", "✅".green(), commit_count);
        passed += 1;
    } else {
        println!("  │  {} Git history shallow ({} commits)", "⚠".yellow(), commit_count);
        issues.push("git history may be shallow".to_string());
    }

    // Check 4 — schema layer present
    let schema_dir = PathBuf::from(core_root).join("04-schema");
    if schema_dir.exists() {
        println!("  │  {} Schema layer present", "✅".green());
        passed += 1;
    } else {
        println!("  │  {} Schema layer missing (04-schema/)", "⚠".yellow());
        issues.push("schema layer missing".to_string());
    }

    // Check 5 — runtime state db
    let state_db = PathBuf::from(core_root).join("runtime/state.db");
    if state_db.exists() {
        println!("  │  {} Runtime state.db present", "✅".green());
        passed += 1;
    } else {
        println!("  │  {} Runtime state.db missing — events lost", "✗".bright_red());
        issues.push("state.db missing".to_string());
    }

    println!("{}", "  ├────────────────────────────────────────────────────".dimmed());
    if issues.is_empty() {
        println!("  │  {} All {} checks passed — forest is consistent", "✅".green(), passed);
    } else {
        println!("  │  {}/{} checks passed — {} issue(s) found",
            passed, passed + issues.len(), issues.len().to_string().yellow());
        for issue in &issues {
            println!("  │    {} {}", "→".bright_red(), issue.bright_red());
        }
    }
    println!("{}", "  ╰────────────────────────────────────────────────────".dimmed());
    println!();

    emit_event(ctx, "verify");
    Ok(())
}

pub fn diff(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require("bootstrap", &[Capability::FilesystemReadHome])?;
    let core_root = &ctx.core_root;

    println!();
    println!("{}", "  ╭─ 📊 Bootstrap Diff ────────────────────────────────".bright_cyan());
    println!("  │  What diverged from the canonical state?");
    println!("{}", "  ├────────────────────────────────────────────────────".dimmed());

    // Compare registry tools vs deployed tools
    let tools = read_registry_tools(core_root);
    let scripts_dir = PathBuf::from(core_root).join("scripts");

    // Tools in registry but not deployed
    let not_deployed: Vec<&String> = tools.iter()
        .filter(|t| !scripts_dir.join(t).exists())
        .collect();

    // Files in scripts not in registry
    let mut unregistered = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !tools.contains(&name) && !name.starts_with('.') {
                unregistered.push(name);
            }
        }
    }

    if not_deployed.is_empty() && unregistered.is_empty() {
        println!("  │  {} No drift detected — registry matches deployment", "✅".green());
    } else {
        if !not_deployed.is_empty() {
            println!("  │  {} In registry, not deployed:", "⚠".yellow());
            for t in &not_deployed {
                println!("  │    {} {}", "−".bright_red(), t.bright_red());
            }
        }
        if !unregistered.is_empty() {
            println!("  │  {} Deployed, not in registry:", "⚠".yellow());
            for t in unregistered.iter().take(5) {
                println!("  │    {} {}", "+".bright_yellow(), t.yellow());
            }
        }
    }

    // Check for intent drift — intents in future/ that are stale
    let future_dir = PathBuf::from(core_root).join("intents/future");
    let future_count = std::fs::read_dir(&future_dir)
        .map(|d| d.count()).unwrap_or(0);

    println!("  │");
    println!("  │  {} active intents in ledger", future_count.to_string().bright_white());

    println!("{}", "  ╰────────────────────────────────────────────────────".dimmed());
    println!();

    emit_event(ctx, "diff");
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn get_rust_version() -> String {
    Command::new("rustc").arg("--version")
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_git_remote(core_root: &str) -> String {
    Command::new("git")
        .args(["-C", core_root, "remote", "get-url", "origin"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "origin".to_string())
}

fn get_commit_count(core_root: &str) -> usize {
    Command::new("git")
        .args(["-C", core_root, "rev-list", "--count", "HEAD"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn read_registry_tools(core_root: &str) -> Vec<String> {
    let path = PathBuf::from(core_root).join("01-registry/tools.toml");
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    content.lines()
        .filter(|l| l.starts_with("name = "))
        .map(|l| l.split('"').nth(1).unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn read_stow_packages(core_root: &str) -> Vec<String> {
    let stow_dir = PathBuf::from(core_root).join("03-interfaces/stow");
    std::fs::read_dir(&stow_dir)
        .map(|d| d.flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect())
        .unwrap_or_default()
}

fn read_active_intents(core_root: &str) -> Vec<String> {
    let future_dir = PathBuf::from(core_root).join("intents/future");
    std::fs::read_dir(&future_dir)
        .map(|d| d.flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|s| s.ends_with(".md"))
            .collect())
        .unwrap_or_default()
}

fn read_decisions(ctx: &AppContext) -> Vec<String> {
    let mut stmt = match ctx.runtime.db.prepare(
        "SELECT description FROM decisions ORDER BY timestamp DESC LIMIT 5"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map([], |r| r.get::<_,String>(0))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
}

fn emit_event(ctx: &AppContext, action: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64).unwrap_or(0);
    let payload = format!(r#"{{"actor":"core","result":"ok","detail":{{"command":"bootstrap.{}"}}}}"#, action);
    ctx.runtime.db.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('bootstrap', ?1, ?2, ?3)",
        rusqlite::params![action, payload, ts],
    ).ok();
}
