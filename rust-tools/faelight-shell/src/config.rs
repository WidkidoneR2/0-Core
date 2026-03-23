//! config — load ~/.config/faelight-shell/config.fsh on startup (Phase 15)
//!
//! Supported directives:
//!   alias ll = "ls -la"       — register shell alias
//!   set prompt_style = minimal — shell setting (stored in shell_state)
//!   set history_limit = 10000  — shell setting
//!   # comment                  — ignored

use crate::db::ForestDb;
use colored::*;

#[derive(Debug)]
pub struct ShellConfig {
    pub aliases: Vec<(String, String)>,
    pub settings: Vec<(String, String)>,
}

impl ShellConfig {
    pub fn empty() -> Self {
        Self {
            aliases: vec![],
            settings: vec![],
        }
    }
}

pub fn config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::PathBuf::from(home).join(".config/faelight-shell/config.fsh")
}

/// Parse config.fsh and return structured config.
/// Silent on missing file — config is optional.
pub fn load() -> ShellConfig {
    let path = config_path();
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return ShellConfig::empty(),
    };

    let mut aliases = vec![];
    let mut settings = vec![];

    for line in text.lines() {
        let line = line.trim();
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("alias ") {
            // alias ll = "ls -la"  or  alias ll = ls -la
            if let Some(eq_pos) = rest.find(" = ") {
                let name = rest[..eq_pos].trim().to_string();
                let val = rest[eq_pos + 3..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if !name.is_empty() && !val.is_empty() {
                    aliases.push((name, val));
                }
            }
        } else if let Some(rest) = line.strip_prefix("set ") {
            // set prompt_style = minimal
            if let Some(eq_pos) = rest.find(" = ") {
                let key = rest[..eq_pos].trim().to_string();
                let val = rest[eq_pos + 3..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if !key.is_empty() && !val.is_empty() {
                    settings.push((key, val));
                }
            }
        }
        // Future: source, export, etc.
    }

    ShellConfig { aliases, settings }
}

/// Apply config to the running shell — register aliases and settings.
pub fn apply(cfg: &ShellConfig, db: &ForestDb) {
    if cfg.aliases.is_empty() && cfg.settings.is_empty() {
        return;
    }

    let alias_count = cfg.aliases.len();
    let setting_count = cfg.settings.len();

    // Register aliases into shell_aliases table
    for (name, cmd) in &cfg.aliases {
        db.add_alias(name, cmd);
    }

    // Store settings in shell_state
    for (key, val) in &cfg.settings {
        let full_key = format!("config.{}", key);
        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES (?1, ?2)",
            rusqlite::params![full_key, val],
        );
    }

    println!(
        "  {} config.fsh — {} alias{}  {} setting{}",
        "✓".bright_green(),
        alias_count,
        if alias_count == 1 { "" } else { "es" },
        setting_count,
        if setting_count == 1 { "" } else { "s" },
    );
}

/// Create a default config.fsh if none exists.
pub fn ensure_default() {
    let path = config_path();
    if path.exists() {
        return;
    }

    // Create parent dir
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let default = r#"# faelight-shell configuration
# ~/.config/faelight-shell/config.fsh
#
# Syntax:
#   alias <name> = "<command>"
#   set <key> = <value>

# Common aliases
alias ll = "ls"
alias gs = "git status"
alias gc5 = "gc | first 5"
alias health = "health"

# Settings
set history_limit = 10000
set prompt_style = forest
"#;

    let _ = std::fs::write(&path, default);
}
