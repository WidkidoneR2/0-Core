//! Plugin registry — Phase 5
//! Static TOML manifest of ecosystem tools that integrate with core.

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn registry_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join("0-core/00-meta/plugins.toml")
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub binary: String,
    pub version: Option<String>,
    pub event_domains: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Registry {
    #[serde(default)]
    plugins: Vec<Plugin>,
}

impl Registry {
    fn load() -> Self {
        let path = registry_path();
        if !path.exists() {
            return Self::default();
        }
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self) -> CoreResult<()> {
        let path = registry_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::errors::CoreError::Io(std::io::Error::new(
                std::io::ErrorKind::Other, e.to_string()
            )))?;
        fs::write(&path, content)?;
        Ok(())
    }
}

fn binary_version(binary: &str) -> Option<String> {
    use std::process::Stdio;
    use std::time::Duration;

    // Known GUI/TUI tools that must not be invoked for version detection
    const SKIP_VERSION: &[&str] = &["faelight-bar", "faelight-fm", "faelight-term"];
    if SKIP_VERSION.contains(&binary) {
        return None;
    }

    let mut child = std::process::Command::new(binary)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    // Wait max 1 second
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() > Duration::from_secs(1) => {
                let _ = child.kill();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(_) => return None,
        }
    }

    let output = child.wait_with_output().ok()?;
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        let s2 = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if s2.is_empty() { None } else { Some(s2) }
    } else {
        Some(s)
    }
}

fn binary_installed(binary: &str) -> bool {
    which::which(binary).is_ok()
}

pub fn list(_ctx: &AppContext) -> CoreResult<()> {
    let registry = Registry::load();

    println!("{}", "🔌 Plugin Registry".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());

    if registry.plugins.is_empty() {
        println!("  {} No plugins registered", "○".dimmed());
        println!("  {} Run: core plugin add <name>", "💡".dimmed());
        println!("{}", "━".repeat(52).dimmed());
        return Ok(());
    }

    for plugin in &registry.plugins {
        let installed = binary_installed(&plugin.binary);
        let status_icon = match (plugin.enabled, installed) {
            (true, true) => "✅".to_string(),
            (true, false) => "⚠️ ".to_string(),
            (false, _) => "○ ".to_string(),
        };

        let version = if installed {
            binary_version(&plugin.binary)
                .map(|v| format!("  {}", v.dimmed()))
                .unwrap_or_default()
        } else {
            format!("  {}", "not installed".bright_red())
        };

        println!(
            "  {} {}{}",
            status_icon,
            plugin.name.bright_white(),
            version,
        );
        println!("     {}", plugin.description.dimmed());

        if !plugin.event_domains.is_empty() {
            println!(
                "     {} domains: {}",
                "→".dimmed(),
                plugin.event_domains.join(", ").cyan()
            );
        }
    }

    println!("{}", "━".repeat(52).dimmed());
    let enabled = registry.plugins.iter().filter(|p| p.enabled).count();
    let installed_count = registry.plugins.iter()
        .filter(|p| binary_installed(&p.binary))
        .count();
    println!(
        "  {} registered  {} enabled  {} installed",
        registry.plugins.len().to_string().bright_white(),
        enabled.to_string().green(),
        installed_count.to_string().cyan(),
    );
    Ok(())
}

pub fn add(_ctx: &AppContext, name: &str) -> CoreResult<()> {
    let mut registry = Registry::load();

    if registry.plugins.iter().any(|p| p.name == name) {
        println!("  {} '{}' already registered", "⚠️".yellow(), name);
        return Ok(());
    }

    // Known plugins with metadata
    let plugin = known_plugin(name).unwrap_or_else(|| Plugin {
        name: name.to_string(),
        description: format!("{} — custom plugin", name),
        binary: name.to_string(),
        version: None,
        event_domains: vec![],
        enabled: true,
    });

    let installed = binary_installed(&plugin.binary);
    registry.plugins.push(plugin.clone());
    registry.save()?;

    println!(
        "  {} Registered '{}'",
        "✅".green(),
        plugin.name.bright_white()
    );
    println!("     {}", plugin.description.dimmed());
    if installed {
        if let Some(v) = binary_version(&plugin.binary) {
            println!("     version: {}", v.cyan());
        }
    } else {
        println!("     {} Binary not found in PATH", "⚠️".yellow());
    }
    Ok(())
}

pub fn remove(_ctx: &AppContext, name: &str) -> CoreResult<()> {
    let mut registry = Registry::load();
    let before = registry.plugins.len();
    registry.plugins.retain(|p| p.name != name);

    if registry.plugins.len() == before {
        println!("  {} '{}' not found in registry", "⚠️".yellow(), name);
        return Ok(());
    }

    registry.save()?;
    println!("  {} Removed '{}'", "✅".green(), name.bright_white());
    Ok(())
}

pub fn status(_ctx: &AppContext, name: &str) -> CoreResult<()> {
    let registry = Registry::load();

    let Some(plugin) = registry.plugins.iter().find(|p| p.name == name) else {
        println!("  {} '{}' not registered — run: core plugin add {}", "⚠️".yellow(), name, name);
        return Ok(());
    };

    let installed = binary_installed(&plugin.binary);

    println!("{}", "🔌 Plugin Status".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!("  {} {}", "Name:   ".dimmed(), plugin.name.bright_white());
    println!("  {} {}", "Binary: ".dimmed(), plugin.binary.cyan());
    println!("  {} {}", "Desc:   ".dimmed(), plugin.description.dimmed());
    println!(
        "  {} {}",
        "Status: ".dimmed(),
        if installed { "installed ✅".green().to_string() } else { "not found ❌".red().to_string() }
    );
    println!(
        "  {} {}",
        "Enabled:".dimmed(),
        if plugin.enabled { "yes".green().to_string() } else { "no".dimmed().to_string() }
    );

    if installed {
        if let Some(v) = binary_version(&plugin.binary) {
            println!("  {} {}", "Version:".dimmed(), v.cyan());
        }
    }

    if !plugin.event_domains.is_empty() {
        println!("  {} {}", "Domains:".dimmed(), plugin.event_domains.join(", ").cyan());
    }

    println!("{}", "━".repeat(52).dimmed());
    Ok(())
}

/// Known first-party plugins with curated metadata
fn known_plugin(name: &str) -> Option<Plugin> {
    match name {
        "faelight-git" => Some(Plugin {
            name: "faelight-git".to_string(),
            description: "Git operations with Risk Score engine".to_string(),
            binary: "faelight-git".to_string(),
            version: None,
            event_domains: vec!["git".to_string()],
            enabled: true,
        }),
        "faelight-update" => Some(Plugin {
            name: "faelight-update".to_string(),
            description: "System update manager with rollback".to_string(),
            binary: "faelight-update".to_string(),
            version: None,
            event_domains: vec!["update".to_string()],
            enabled: true,
        }),
        "faelight-fm" => Some(Plugin {
            name: "faelight-fm".to_string(),
            description: "Terminal file manager with daemon integration".to_string(),
            binary: "faelight-fm".to_string(),
            version: None,
            event_domains: vec![],
            enabled: true,
        }),
        "faelight-bar" => Some(Plugin {
            name: "faelight-bar".to_string(),
            description: "Wayland status bar — health subscriber".to_string(),
            binary: "faelight-bar".to_string(),
            version: None,
            event_domains: vec!["doctor".to_string()],
            enabled: true,
        }),
        "faelight-fetch" => Some(Plugin {
            name: "faelight-fetch".to_string(),
            description: "System information display".to_string(),
            binary: "faelight-fetch".to_string(),
            version: None,
            event_domains: vec![],
            enabled: true,
        }),
        _ => None,
    }
}
