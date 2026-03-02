use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreError;
use crate::errors::CoreResult;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn checkpoints_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join("0-core/runtime/checkpoints")
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckpointManifest {
    name: String,
    created: String,
    version: String,
    health: u32,
    git_head: String,
    tool_versions: HashMap<String, String>,
    config_hashes: HashMap<String, String>,
    notes: Option<String>,
}

fn timestamp() -> String {
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

fn read_version() -> String {
    let version_file = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join("0-core/00-meta/VERSION");
    fs::read_to_string(version_file)
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn read_git_head() -> String {
    let output = std::process::Command::new("git")
        .args(["-C", &dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/root"))
            .join("0-core")
            .to_string_lossy()
            .to_string(),
            "rev-parse", "--short", "HEAD"])
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "unknown".to_string(),
    }
}

fn read_tool_versions() -> HashMap<String, String> {
    let mut versions = HashMap::new();
    let tools_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join("0-core/rust-tools");
    if let Ok(entries) = fs::read_dir(&tools_dir) {
        for entry in entries.flatten() {
            let cargo_toml = entry.path().join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(content) = fs::read_to_string(&cargo_toml) {
                    let mut name = None;
                    let mut version = None;
                    for line in content.lines() {
                        if line.starts_with("name = \"") && name.is_none() {
                            name = Some(line.split('"').nth(1).unwrap_or("").to_string());
                        }
                        if line.starts_with("version = \"") && version.is_none() {
                            version = Some(line.split('"').nth(1).unwrap_or("").to_string());
                        }
                        if name.is_some() && version.is_some() {
                            break;
                        }
                    }
                    if let (Some(n), Some(v)) = (name, version) {
                        if !n.is_empty() && !v.is_empty() {
                            versions.insert(n, v);
                        }
                    }
                }
            }
        }
    }
    versions
}

fn hash_file(path: &PathBuf) -> String {
    let output = std::process::Command::new("sha256sum")
        .arg(path)
        .output();
    match output {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            s.split_whitespace().next().unwrap_or("unknown").to_string()
        }
        Err(_) => "unknown".to_string(),
    }
}

fn read_config_hashes() -> HashMap<String, String> {
    let mut hashes = HashMap::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
    let configs = vec![
        ("sway", home.join(".config/sway/config")),
        ("zshrc", home.join(".zshrc")),
        ("foot", home.join(".config/foot/foot.ini")),
        ("aliases", home.join(".config/zsh/aliases.zsh")),
    ];
    for (name, path) in configs {
        if path.exists() {
            hashes.insert(name.to_string(), hash_file(&path));
        }
    }
    hashes
}

fn read_last_health() -> u32 {
    let state_db = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join("0-core/runtime/state.db");
    if !state_db.exists() {
        return 0;
    }
    let output = std::process::Command::new("sqlite3")
        .arg(&state_db)
        .arg("SELECT payload FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 1;")
        .output();
    if let Ok(o) = output {
        let s = String::from_utf8_lossy(&o.stdout);
        let s = s.trim();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
            if let Some(h) = v.get("detail").and_then(|d| d.get("health")).and_then(|h| h.as_u64()) {
                return h as u32;
            }
        }
    }
    0
}

pub fn create(ctx: &AppContext, name: &str, notes: Option<&str>) -> CoreResult<()> {
    ctx.capabilities.require(
        "checkpoint",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    let dir = checkpoints_dir();
    fs::create_dir_all(&dir).map_err(|e| CoreError::Io(e))?;

    println!("{}", "📸 Creating checkpoint...".bold());
    println!("  {} Collecting system state...", "→".dimmed());

    let manifest = CheckpointManifest {
        name: name.to_string(),
        created: timestamp(),
        version: read_version(),
        health: read_last_health(),
        git_head: read_git_head(),
        tool_versions: read_tool_versions(),
        config_hashes: read_config_hashes(),
        notes: notes.map(|s| s.to_string()),
    };

    let filename = format!("{}-{}.toml", manifest.created.replace(':', "-"), name);
    let filepath = dir.join(&filename);

    let toml_content = toml::to_string_pretty(&manifest)
        .map_err(|e| CoreError::Runtime(e.to_string()))?;

    fs::write(&filepath, toml_content).map_err(|e| CoreError::Io(e))?;

    println!("{}", "━".repeat(50).dimmed());
    println!("  {} Checkpoint created", "✅".green());
    println!("  {} {}", "Name:   ".dimmed(), name.bright_white());
    println!("  {} {}", "Version:".dimmed(), manifest.version.bright_white());
    println!("  {} {}%", "Health: ".dimmed(), manifest.health.to_string().bright_green());
    println!("  {} {}", "Commit: ".dimmed(), manifest.git_head.bright_white());
    println!("  {} {}", "Tools:  ".dimmed(), manifest.tool_versions.len().to_string().bright_white());
    println!("  {} {}", "File:   ".dimmed(), filename.dimmed());
    if let Some(n) = &manifest.notes {
        println!("  {} {}", "Notes:  ".dimmed(), n.bright_white());
    }
    println!("{}", "━".repeat(50).dimmed());

    Ok(())
}

pub fn list(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "checkpoint",
        &[Capability::FilesystemReadHome],
    )?;

    let dir = checkpoints_dir();
    if !dir.exists() {
        println!("  {} No checkpoints found", "○".dimmed());
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| CoreError::Io(e))?
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x == "toml")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());
    entries.reverse();

    if entries.is_empty() {
        println!("  {} No checkpoints found", "○".dimmed());
        return Ok(());
    }

    println!("{}", "📸 Checkpoints".bold());
    println!("{}", "━".repeat(50).dimmed());

    for entry in &entries {
        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        if let Ok(manifest) = toml::from_str::<CheckpointManifest>(&content) {
            let health_color = if manifest.health >= 95 {
                manifest.health.to_string().bright_green()
            } else if manifest.health >= 80 {
                manifest.health.to_string().bright_yellow()
            } else {
                manifest.health.to_string().bright_red()
            };
            println!(
                "  {} {}  {}  {}%  {}",
                "●".bright_cyan(),
                manifest.name.bright_white(),
                manifest.created.dimmed(),
                health_color,
                manifest.version.dimmed(),
            );
            if let Some(n) = &manifest.notes {
                println!("    {} {}", "↳".dimmed(), n.dimmed());
            }
        }
    }

    println!("{}", "━".repeat(50).dimmed());
    println!("  {} checkpoint(s)", entries.len().to_string().bright_white());

    Ok(())
}

pub fn diff(ctx: &AppContext, name: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "checkpoint",
        &[Capability::FilesystemReadHome],
    )?;

    let dir = checkpoints_dir();
    if !dir.exists() {
        println!("  {} No checkpoints found", "○".dimmed());
        return Ok(());
    }

    // Find checkpoint by name (most recent match)
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| CoreError::Io(e))?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .contains(name)
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());
    entries.reverse();

    let entry = entries.first().ok_or_else(|| {
        CoreError::Runtime(format!("Checkpoint '{}' not found", name))
    })?;

    let content = fs::read_to_string(entry.path()).map_err(|e| CoreError::Io(e))?;
    let manifest = toml::from_str::<CheckpointManifest>(&content)
        .map_err(|e| CoreError::Runtime(e.to_string()))?;

    println!("{}", "📸 Checkpoint Diff".bold());
    println!("  {} {} ({})", "→".dimmed(), manifest.name.bright_white(), manifest.created.dimmed());
    println!("{}", "━".repeat(50).dimmed());

    // Version diff
    let current_version = read_version();
    if current_version != manifest.version {
        println!("  {} Version", "~".bright_yellow());
        println!("    {} {}", "was:".dimmed(), manifest.version.bright_red());
        println!("    {} {}", "now:".dimmed(), current_version.bright_green());
    } else {
        println!("  {} Version: {} (unchanged)", "✓".bright_green(), current_version.dimmed());
    }

    // Git HEAD diff
    let current_head = read_git_head();
    if current_head != manifest.git_head {
        println!("  {} Git HEAD", "~".bright_yellow());
        println!("    {} {}", "was:".dimmed(), manifest.git_head.bright_red());
        println!("    {} {}", "now:".dimmed(), current_head.bright_green());
    } else {
        println!("  {} Git HEAD: {} (unchanged)", "✓".bright_green(), current_head.dimmed());
    }

    // Health diff
    let current_health = read_last_health();
    if current_health != manifest.health {
        let symbol = if current_health > manifest.health { "↑".bright_green() } else { "↓".bright_red() };
        println!("  {} Health: {}% → {}% {}", "~".bright_yellow(), manifest.health, current_health, symbol);
    } else {
        println!("  {} Health: {}% (unchanged)", "✓".bright_green(), current_health);
    }

    // Tool version diffs
    let current_tools = read_tool_versions();
    let mut tool_changes = 0;
    for (tool, old_ver) in &manifest.tool_versions {
        if let Some(new_ver) = current_tools.get(tool) {
            if new_ver != old_ver {
                if tool_changes == 0 {
                    println!("  {} Tool versions changed", "~".bright_yellow());
                }
                println!("    {} {} {} → {}", "↳".dimmed(), tool.bright_white(), old_ver.bright_red(), new_ver.bright_green());
                tool_changes += 1;
            }
        }
    }
    if tool_changes == 0 {
        println!("  {} Tool versions: all unchanged", "✓".bright_green());
    }

    // Config hash diffs
    let current_hashes = read_config_hashes();
    let mut config_changes = 0;
    for (config, old_hash) in &manifest.config_hashes {
        if let Some(new_hash) = current_hashes.get(config) {
            if new_hash != old_hash {
                if config_changes == 0 {
                    println!("  {} Config files changed", "~".bright_yellow());
                }
                println!("    {} {} modified", "↳".dimmed(), config.bright_white());
                config_changes += 1;
            }
        }
    }
    if config_changes == 0 {
        println!("  {} Config files: all unchanged", "✓".bright_green());
    }

    println!("{}", "━".repeat(50).dimmed());

    Ok(())
}

pub fn auto(ctx: &AppContext, reason: &str) -> CoreResult<()> {
    ctx.capabilities.require(
        "checkpoint",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    let name = format!("auto-{}", reason.replace(' ', "-"));
    create(ctx, &name, Some(&format!("Auto-checkpoint before {}", reason)))?;

    Ok(())
}
