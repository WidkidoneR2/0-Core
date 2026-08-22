#![allow(clippy::all)]
//! config — load ~/.config/faelight-shell/config.fsh on startup (Phase 15)
//!
//! Supported directives:
//!   alias ll = "ls -la"       — register shell alias
//!   set prompt_style = minimal — shell setting (stored in shell_state)
//!   set history_limit = 10000  — shell setting
//!   # comment                  — ignored

use crate::db::ForestDb;

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
    // INT-134: FSH_CONFIG overrides the path so a setting can be tried without a rebuild. The
    // deployed config.fsh is a home-manager symlink into /nix/store and therefore READ-ONLY --
    // proving one line otherwise costs a full deploy, and this loop is needed once per setting.
    //
    // An env var rather than a flag: load() takes no arguments and runs before argument parsing,
    // so a flag would have to thread through every caller to serve one use.
    //
    // An empty value is IGNORED rather than treated as a path -- `FSH_CONFIG= fsh` should mean
    // "no override", not "load the current directory".
    if let Ok(p) = std::env::var("FSH_CONFIG") {
        if !p.trim().is_empty() {
            return std::path::PathBuf::from(p);
        }
    }
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
        // if command starts_with "nix-env -i" { warn "prefer declaring packages in the flake" }
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
/// What `apply` did to the shell's vocabulary, so the CALLER can decide whether to announce it.
///
/// ⚠️ `apply` used to `println!` both of these itself, which put UI inside a runtime step. That is
/// invisible in an interactive shell and wrong everywhere else: under a non-interactive invocation
/// it contaminates the program's own stdout, so `fsh -c 'echo hi' | wc -l` would have counted the
/// config banner as output. THE RULE (INT-200): any non-program output from a non-interactive
/// invocation belongs on stderr -- and deciding that is the front end's job, not this function's.
#[derive(Debug, Default, Clone, Copy)]
pub struct ApplyReport {
    pub aliases: usize,
    pub settings: usize,
    pub pruned: usize,
}

pub fn apply(cfg: &ShellConfig, db: &ForestDb) -> ApplyReport {
    if cfg.aliases.is_empty() && cfg.settings.is_empty() {
        return ApplyReport::default();
    }

    let alias_count = cfg.aliases.len();
    let setting_count = cfg.settings.len();

    // ONE TRANSACTION, NOT 285. Measured 2026-08-22 with FSH_BOOT_PROFILE: this function
    // cost 210ms of fsh's 214ms startup, and `fsh -c true` took 462ms against bash's 3ms.
    // SQLite commits per statement without an explicit transaction, so 285 aliases meant
    // 285 commits -- roughly 0.7ms each, paid on EVERY invocation including a one-shot -c
    // that uses none of them.
    //
    // ⚠️ The order rule above still holds: apply WRITES shell_aliases and the registry READS
    // it. Nothing is skipped here -- the same writes happen, they are simply committed once.
    let _ = db.conn.execute_batch("BEGIN");
    // Register aliases into shell_aliases table
    for (name, cmd) in &cfg.aliases {
        db.add_alias(name, cmd);
    }

    // INT-060 G9: config.fsh is the source of truth. After seeding config aliases,
    // remove any table alias not present in config.fsh so runtime `alias` cruft
    // cannot persist across shells. Guard: never prune when config parsed to zero
    // aliases -- a parse failure must not wipe the live set.
    let _ = db.conn.execute_batch("COMMIT");

    let mut pruned = 0usize;
    if !cfg.aliases.is_empty() {
        use std::collections::HashSet;
        let keep: HashSet<&str> = cfg.aliases.iter().map(|(n, _)| n.as_str()).collect();
        for (name, _) in db.list_aliases() {
            if !keep.contains(name.as_str()) && db.remove_alias(&name) {
                pruned += 1;
            }
        }
    }

    // Store settings in shell_state
    for (key, val) in &cfg.settings {
        let full_key = format!("config.{}", key);
        let _ = db.conn.execute(
            "INSERT OR REPLACE INTO shell_state (key, value) VALUES (?1, ?2)",
            rusqlite::params![full_key, val],
        );
    }

    ApplyReport {
        aliases: alias_count,
        settings: setting_count,
        pruned,
    }
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
