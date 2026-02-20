use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(default = "default_true")]
    pub secrets_enabled: bool,

    #[serde(default = "default_true")]
    pub conflicts_enabled: bool,

    #[serde(default = "default_true")]
    pub rustfmt_enabled: bool,

    #[serde(default = "default_true")]
    pub clippy_enabled: bool,

    #[serde(default = "default_true")]
    pub branch_enabled: bool,

    #[serde(default = "default_true")]
    pub filesize_enabled: bool,

    #[serde(default = "default_filesize_mb")]
    pub filesize_limit_mb: u64,
}

fn default_true() -> bool {
    true
}
fn default_filesize_mb() -> u64 {
    50
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            secrets_enabled: true,
            conflicts_enabled: true,
            rustfmt_enabled: true,
            clippy_enabled: true,
            branch_enabled: true,
            filesize_enabled: true,
            filesize_limit_mb: 50,
        }
    }
}

pub fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("faelight")
        .join("hooks.toml")
}

pub fn load_config() -> HooksConfig {
    let path = config_path();
    if !path.exists() {
        return HooksConfig::default();
    }
    match fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => HooksConfig::default(),
    }
}

pub fn show_config() -> Result<()> {
    let path = config_path();
    let cfg = load_config();

    println!("{}", "⚙️  Hook Configuration".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    println!("{}", "Pre-commit checks:".bold());
    println!("  branch      {}", status_label(cfg.branch_enabled));
    println!(
        "  filesize    {} (limit: {} MB)",
        status_label(cfg.filesize_enabled),
        cfg.filesize_limit_mb
    );
    println!("  secrets     {}", status_label(cfg.secrets_enabled));
    println!("  conflicts   {}", status_label(cfg.conflicts_enabled));
    println!("  rustfmt     {}", status_label(cfg.rustfmt_enabled));
    println!("  clippy      {}", status_label(cfg.clippy_enabled));
    println!();
    println!("{}", "Config file:".bold());
    if path.exists() {
        println!("  {} (active)", path.display().to_string().green());
    } else {
        println!("  {} — using defaults", path.display().to_string().dimmed());
        println!();
        println!("{}", "💡 To create a config file:".yellow());
        println!("   {}", "faelight-hooks config --init".cyan());
    }
    println!();
    Ok(())
}

pub fn init_config() -> Result<()> {
    let path = config_path();

    if path.exists() {
        println!("{}", "⚠️  Config file already exists:".yellow());
        println!("   {}", path.display());
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }
    // Write as flat TOML matching our struct
    let cfg = HooksConfig::default();
    let contents = toml::to_string_pretty(&cfg).context("Failed to serialize config")?;
    fs::write(&path, contents).context("Failed to write config file")?;

    println!("{}", "✅ Config file created:".green().bold());
    println!("   {}", path.display());
    println!();
    println!("Edit it to enable/disable individual checks.");
    Ok(())
}

fn status_label(enabled: bool) -> colored::ColoredString {
    if enabled {
        "enabled".green()
    } else {
        "disabled".red()
    }
}
