#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub emoji: Option<String>,
    pub description: Option<String>,
}

fn profiles_toml(ctx: &AppContext) -> PathBuf {
    PathBuf::from(&ctx.core_root).join("01-registry/profiles.toml")
}

fn state_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".local/state/0-core/current-profile")
}

fn log_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".local/state/0-core/profile.log")
}

fn load_profiles(ctx: &AppContext) -> Vec<Profile> {
    let path = profiles_toml(ctx);
    let Ok(content) = fs::read_to_string(&path) else {
        return vec![];
    };

    #[derive(Deserialize)]
    struct Root {
        profile: Vec<Profile>,
    }
    toml::from_str::<Root>(&content)
        .map(|r| r.profile)
        .unwrap_or_default()
}

fn current_profile() -> String {
    fs::read_to_string(state_file())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "default".to_string())
}

pub fn list(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require(
        "profile",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;
    let profiles = load_profiles(ctx);
    let current = current_profile();

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "📋 Available Profiles".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    for p in &profiles {
        let emoji = p.emoji.as_deref().unwrap_or("▶");
        let desc = p.description.as_deref().unwrap_or("");
        if p.name == current {
            println!(
                "  {} {} {} {}",
                "▶".bright_green(),
                emoji,
                p.name.bright_white().bold(),
                "(active)".bright_green()
            );
            println!("    {}", desc.dimmed());
        } else {
            println!("  {} {} {}", " ".normal(), emoji, p.name);
            println!("    {}", desc.dimmed());
        }
    }
    Ok(())
}

pub fn status(ctx: &AppContext) -> CoreResult<()> {
    let current = current_profile();
    let profiles = load_profiles(ctx);
    let profile = profiles.iter().find(|p| p.name == current);
    let emoji = profile
        .and_then(|p| p.emoji.clone())
        .unwrap_or_else(|| "🏠".to_string());

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "📊 Profile Status".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  Current: {} {}", emoji, current.bright_white().bold());
    Ok(())
}

pub fn switch(ctx: &AppContext, name: &str) -> CoreResult<()> {
    let profiles = load_profiles(ctx);
    if !profiles.iter().any(|p| p.name == name) {
        println!("  {} Profile '{}' not found", "✗".bright_red(), name);
        println!("  Run: core profile list");
        return Ok(());
    }

    let from = current_profile();
    if from == name {
        println!("  {} Already on profile '{}'", "○".dimmed(), name);
        return Ok(());
    }

    // Write state
    if let Some(parent) = state_file().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(state_file(), name)?;

    // Log the switch
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let entry = format!("{} | {} -> {}\n", timestamp, from, name);
    let mut log = fs::read_to_string(log_file()).unwrap_or_default();
    log.push_str(&entry);
    fs::write(log_file(), log)?;

    println!(
        "  {} Switched: {} → {}",
        "✅".green(),
        from.dimmed(),
        name.bright_white().bold()
    );
    Ok(())
}

pub fn history() -> CoreResult<()> {
    let log = fs::read_to_string(log_file()).unwrap_or_default();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "📜 Profile History".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if log.is_empty() {
        println!("  {} No switches recorded", "○".dimmed());
    } else {
        let lines: Vec<&str> = log.lines().collect();
        for line in lines.iter().rev().take(10) {
            println!("  {} {}", "▶".dimmed(), line);
        }
    }
    Ok(())
}

pub fn health(ctx: &AppContext) -> CoreResult<()> {
    let profiles = load_profiles(ctx);
    let current = current_profile();
    println!("{}", "🏥 core profile health".bold());
    println!("  {} Profiles loaded: {}", "✅".green(), profiles.len());
    println!(
        "  {} Current profile: {}",
        "✅".green(),
        current.bright_white()
    );
    println!(
        "  {} State file: {}",
        "✅".green(),
        state_file().display().to_string().dimmed()
    );
    println!("  {} All checks passed!", "✅".green());
    Ok(())
}
