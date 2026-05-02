#![allow(clippy::all)]
//! config — load ~/.config/faelight-shell/config.fsh on startup (Phase 15)
//!
//! Supported directives:
//!   alias ll = "ls -la"       — register shell alias
//!   set prompt_style = minimal — shell setting (stored in shell_state)
//!   set history_limit = 10000  — shell setting
//!   # comment                  — ignored

use crate::db::ForestDb;
use colored::*;

/// A single before_run rule — condition + action
#[derive(Debug, Clone)]
pub struct BeforeRunRule {
    pub condition: RuleCondition,
    pub action: RuleAction,
    pub message: String,
}

/// What to check before running a command
#[derive(Debug, Clone)]
pub enum RuleCondition {
    CommandEquals(String),
    CommandContains(String),
    CommandStartsWith(String),
}

/// What to do when condition matches
#[derive(Debug, Clone)]
pub enum RuleAction {
    Block,
    Warn,
    Suggest,
}

impl BeforeRunRule {
    /// Check if this rule matches the given command line
    pub fn matches(&self, raw: &str) -> bool {
        let raw_lower = raw.to_lowercase();
        match &self.condition {
            RuleCondition::CommandEquals(s) => raw_lower == s.to_lowercase(),
            RuleCondition::CommandContains(s) => raw_lower.contains(&s.to_lowercase()),
            RuleCondition::CommandStartsWith(s) => raw_lower.starts_with(&s.to_lowercase()),
        }
    }
}

#[derive(Debug)]
pub struct ShellConfig {
    pub aliases: Vec<(String, String)>,
    pub settings: Vec<(String, String)>,
    pub before_rules: Vec<BeforeRunRule>,
}

impl ShellConfig {
    pub fn empty() -> Self {
        Self {
            aliases: vec![],
            settings: vec![],
            before_rules: vec![],
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
    let mut before_rules = vec![];

    // Parse before_run { } blocks
    let mut in_before_run = false;
    for line in text.lines() {
        let line = line.trim();
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // before_run block open/close
        if line == "before_run {" {
            in_before_run = true;
            continue;
        }
        if in_before_run && line == "}" {
            in_before_run = false;
            continue;
        }
        // Parse before_run rules:
        // if command contains "rm -rf" { block "message" }
        // if command starts_with "paru" { warn "message" }
        // if command == "deploy" { suggest "message" }
        if in_before_run {
            if let Some(rest) = line.strip_prefix("if command ") {
                // Parse: <cond_type> "<cond_val>" { <action> "<message>" }
                let (cond, remainder) = if let Some(r) = rest.strip_prefix("contains ") {
                    (Some("contains"), r)
                } else if let Some(r) = rest.strip_prefix("starts_with ") {
                    (Some("starts_with"), r)
                } else if let Some(r) = rest.strip_prefix("== ") {
                    (Some("=="), r)
                } else {
                    (None, rest)
                };
                if let Some(cond_type) = cond {
                    // Extract quoted condition value
                    if let Some(q1) = remainder.find('"') {
                        if let Some(q2) = remainder[q1 + 1..].find('"') {
                            let cond_val = remainder[q1 + 1..q1 + 1 + q2].to_string();
                            let after = remainder[q1 + 1 + q2 + 1..].trim();
                            // Extract action and message: { block "msg" }
                            if let Some(inner) =
                                after.strip_prefix("{").and_then(|s| s.strip_suffix("}"))
                            {
                                let inner = inner.trim();
                                let (action, msg_rest) =
                                    if let Some(r) = inner.strip_prefix("block ") {
                                        (Some(RuleAction::Block), r)
                                    } else if let Some(r) = inner.strip_prefix("warn ") {
                                        (Some(RuleAction::Warn), r)
                                    } else if let Some(r) = inner.strip_prefix("suggest ") {
                                        (Some(RuleAction::Suggest), r)
                                    } else {
                                        (None, inner)
                                    };
                                if let Some(action) = action {
                                    let message = msg_rest.trim().trim_matches('"').to_string();
                                    let condition = match cond_type {
                                        "contains" => RuleCondition::CommandContains(cond_val),
                                        "starts_with" => RuleCondition::CommandStartsWith(cond_val),
                                        _ => RuleCondition::CommandEquals(cond_val),
                                    };
                                    before_rules.push(BeforeRunRule {
                                        condition,
                                        action,
                                        message,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("alias ") {
            // alias ll = "ls -la"  or  alias ll = ls -la
            if let Some(eq_pos) = rest.find(" = ") {
                let name = rest[..eq_pos].trim().to_string();
                let raw_val = rest[eq_pos + 3..].trim();
                // Strip inline comments:  value  # comment
                let raw_val = if let Some(idx) = raw_val.find("  #") {
                    &raw_val[..idx]
                } else {
                    raw_val
                };
                let val = raw_val
                    .trim_matches('"')
                    .trim_matches("'".chars().next().unwrap())
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

    ShellConfig {
        aliases,
        settings,
        before_rules,
    }
}
/// Validate config.fsh syntax without loading -- returns list of errors
pub fn validate() -> Vec<String> {
    let path = config_path();
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    let mut errors: Vec<String> = vec![];
    let mut in_before_run = false;
    for (lineno, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "before_run {" {
            in_before_run = true;
            continue;
        }
        if in_before_run && line == "}" {
            in_before_run = false;
            continue;
        }
        if in_before_run {
            continue;
        }
        if line.starts_with("alias ") {
            if !line.contains(" = ") {
                errors.push(format!("  line {}: invalid alias -- missing ' = '\n    got: {}\n    fix: alias name = \"command\"", lineno+1, line));
            }
        } else if line.starts_with("set ") {
            if !line.contains(" = ") {
                errors.push(format!("  line {}: invalid set -- missing ' = '\n    got: {}\n    fix: set key = value", lineno+1, line));
            }
        } else if line != "before_run {" && line != "}" {
            errors.push(format!("  line {}: unknown directive '{}'\n    fix: valid directives are: alias, set, before_run", lineno+1, &line[..line.len().min(40)]));
        }
    }
    errors
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

# Settings
set history_limit = 10000
set prompt_style = forest
"#;

    let _ = std::fs::write(&path, default);
}
