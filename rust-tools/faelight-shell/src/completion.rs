// faelight-shell — Schema-Aware Completion
// Phase 11: Tab completion that knows column names, commands, and pipeline ops

extern crate rusqlite;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;

use crate::schema::SchemaRegistry;

const COMMANDS: &[&str] = &[
    "core",
    "health",
    "events",
    "decisions",
    "intents",
    "tools",
    "version",
    "schema",
    "commits",
    "story",
    "advise",
    "audit",
    "forecast",
    "sandbox",
    "checkpoint",
    "cpc",
    "git",
    "search",
    "s",
    "tt",
    "tools-table",
    "et",
    "events-table",
    "at",
    "audit-table",
    "dt",
    "decisions-table",
    "ht",
    "history-table",
    "ct",
    "checkpoints-table",
    "domains",
    "histogram",
    "hist",
    "logs",
    "ps",
    "processes",
    "ports",
    "services",
    "svc",
    "files",
    "ls",
    "net",
    "network",
    "pkgs",
    "packages",
    "pwd",
    "which",
    "env",
    "clear",
    "echo",
    "cat",
    "type",
    "theme",
    "z",
    "zi",
    "ya",
    "yazi",
    "fm",
    "flow",
    "usage",
    "debug",
    "since",
    "d",
    "v",
    "c",
    "q",
    "fs",
    "ll",
    "gc",
    "git-commits",
    "gf",
    "git-files",
    "watch",
    "alias",
    "unalias",
    "plugins",
    "help",
    "h",
    "?",
    "exit",
    "quit",
];

const PIPE_OPS: &[&str] = &[
    "where", "sort", "select", "first", "last", "count", "get", "watch", "join", "group",
];

pub struct ForestHelper<'a> {
    registry: SchemaRegistry,
    db: &'a crate::db::ForestDb,
}

impl<'a> ForestHelper<'a> {
    pub fn new(db: &'a crate::db::ForestDb) -> Self {
        ForestHelper {
            registry: SchemaRegistry::build(),
            db,
        }
    }

    fn completions_for(&self, line: &str) -> (usize, Vec<String>) {
        // ── Case 1: pipe present ──────────────────────────────────────────────
        if let Some(pipe_pos) = line.rfind(" | ") {
            let after = &line[pipe_pos + 3..];
            let _trailing_space = after.ends_with(' ');
            let base_cmd = line[..pipe_pos]
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();
            let tokens: Vec<&str> = after.split_whitespace().collect();

            return match tokens.as_slice() {
                // "ps | <TAB>" — suggest pipe ops
                [] => (
                    pipe_pos + 3,
                    PIPE_OPS.iter().map(|s| s.to_string()).collect(),
                ),

                // "ps | where <TAB>" — op is complete word, suggest columns with leading space
                [op] if matches!(*op, "where" | "sort" | "select" | "get") => {
                    if let Some(s) = self.registry.get(&base_cmd) {
                        (
                            line.len(),
                            s.column_names().iter().map(|c| format!(" {}", c)).collect(),
                        )
                    } else {
                        (line.len(), vec![])
                    }
                }

                // "ps | where cp<TAB>" — filter columns
                [op, partial] if matches!(*op, "where" | "sort" | "select" | "get") => {
                    if let Some(s) = self.registry.get(&base_cmd) {
                        let start = line.len() - partial.len();
                        let cands: Vec<String> = s
                            .column_names()
                            .iter()
                            .filter(|c| c.starts_with(*partial))
                            .map(|c| c.to_string())
                            .collect();
                        (start, cands)
                    } else {
                        (line.len(), vec![])
                    }
                }

                // "ps | wh<TAB>" — complete op name
                [partial] => {
                    let start = pipe_pos + 3;
                    let cands: Vec<String> = PIPE_OPS
                        .iter()
                        .filter(|op| op.starts_with(*partial))
                        .map(|s| s.to_string())
                        .collect();
                    (start, cands)
                }

                _ => (line.len(), vec![]),
            };
        }

        // ── Case 2: schema <TAB> ──────────────────────────────────────────────
        if line == "schema" || line.starts_with("schema ") {
            let partial = if line.starts_with("schema ") {
                &line["schema ".len()..]
            } else {
                ""
            };
            let cands: Vec<String> = self
                .registry
                .names()
                .iter()
                .filter(|n| n.starts_with(partial))
                .map(|n| format!("schema {}", n))
                .collect();
            return (0, cands);
        }

        // ── Case 2b: forest-aware subcommand completion ─────────────────────
        // Same logic as Case 3 — prefix match against full "core goals" style strings
        {
            // All multi-word completions — prefix match from start=0
            const MULTI_CMDS: &[&str] = &[
                // ── core intent ──────────────────────────────────────────
                "core intent focus",
                "core intent unfocus",
                "core intent status",
                "core intent drift",
                "core intent start",
                "core intent complete",
                "core intent new",
                "core intent new --smart",
                "core intent deps",
                "core intent deps --critical-path",
                "core intent burndown",
                "core intent velocity",
                "core intent branch",
                "core intent list",
                "core intent show",
                "core intent search",
                "core intent stats",
                "core intent validate",
                "core intent predict",
                "core intent story",
                "core intent auto-link",
                "core intent health",
                "core intent edit",
                "core intent",
                // ── core partner ─────────────────────────────────────────
                "core partner propose",
                "core partner discuss",
                "core partner disagree",
                "core partner consult",
                "core partner reflect",
                "core partner pattern",
                "core partner growth",
                "core partner pushback",
                "core partner roadmap",
                "core partner roadmap-why",
                "core partner roadmap-diff",
                "core partner status",
                "core partner",
                // ── core delegate ─────────────────────────────────────────
                "core delegate simulate",
                "core delegate contracts",
                "core delegate history",
                "core delegate accuracy",
                "core delegate suspend",
                "core delegate activate",
                "core delegate",
                // ── core predict ──────────────────────────────────────────
                "core predict sessions",
                "core predict cadence",
                "core predict health",
                "core predict decline",
                "core predict intents",
                "core predict next",
                "core predict coupling",
                "core predict churn",
                "core predict accuracy",
                "core predict verify",
                "core predict cross-session",
                "core predict memory-decay",
                "core predict",
                // ── core doctor ───────────────────────────────────────────
                "core doctor run",
                "core doctor quick",
                "core doctor trend",
                "core doctor forecast",
                "core doctor history",
                "core doctor",
                // ── core react ────────────────────────────────────────────
                "core react list",
                "core react rules",
                "core react run",
                "core react history",
                "core react story",
                "core react audit",
                "core react bounds",
                "core react",
                // ── core integrity ────────────────────────────────────────
                "core integrity run",
                "core integrity apply",
                "core integrity heal",
                "core integrity heal --dry",
                "core integrity trend",
                "core integrity status",
                "core integrity log",
                "core integrity",
                // ── core strategy ─────────────────────────────────────────
                "core strategy now",
                "core strategy jarvis",
                "core strategy horizon",
                "core strategy coherence",
                "core strategy",
                // ── core autonomy ─────────────────────────────────────────
                "core autonomy status",
                "core autonomy mandate",
                "core autonomy history",
                "core autonomy",
                // ── core registry ─────────────────────────────────────────
                "core registry list",
                "core registry show",
                "core registry retire",
                "core registry unretire",
                "core registry reality-check",
                "core registry",
                // ── core db ───────────────────────────────────────────────
                "core db backup",
                "core db restore",
                "core db verify",
                "core db status",
                "core db compact",
                "core db",
                // ── core checkpoint ───────────────────────────────────────
                "core checkpoint create",
                "core checkpoint list",
                "core checkpoint restore",
                "core checkpoint",
                // ── core security ─────────────────────────────────────────
                "core security scan",
                "core security report",
                "core security history",
                "core security",
                // ── core stress ───────────────────────────────────────────
                "core stress events",
                "core stress predict",
                "core stress react",
                "core stress health",
                "core stress intents",
                "core stress report",
                "core stress health-report",
                "core stress",
                // ── core goals/plan/tradeoff ──────────────────────────────
                "core goals list",
                "core goals generate",
                "core goals accept",
                "core goals reject",
                "core goals show",
                "core goals",
                "core plan generate",
                "core plan review",
                "core plan list",
                "core plan",
                "core tradeoff analyze",
                "core tradeoff history",
                "core tradeoff balance",
                "core tradeoff",
                "core prioritize run",
                "core prioritize explain",
                "core prioritize",
                // ── core other ────────────────────────────────────────────
                "core events list",
                "core events since",
                "core events filter",
                "core events watch",
                "core events",
                "core genealogy tree",
                "core genealogy show",
                "core genealogy roots",
                "core genealogy",
                "core narrative",
                "core snapshot",
                "core anomaly",
                "core audit",
                "core deps",
                "core capabilities",
                "core version",
                "core autobiography",
                "core evolution",
                "core simulate",
                "core advise",
                "core story",
                "core lessons",
                "core hindsight",
                "core decision",
                "core decide",
                "core heuristics",
                "core why",
                "core trace",
                "core ledger",
                // ── et shortcuts ──────────────────────────────────────────
                "et today",
                "et goals",
                "et git",
                "et security",
                "et doctor",
                // ── cd shortcuts ──────────────────────────────────────────
                "cd ~/",
                "cd ~/0-core",
                "cd ~/0-core/rust-tools",
                "cd ~/0-core/engine",
                "cd ~/0-core/intents",
                "cd ~/0-core/runtime",
            ];
            let cands: Vec<String> = MULTI_CMDS
                .iter()
                .filter(|c| c.starts_with(line))
                .map(|s| s.to_string())
                .collect();
            if !cands.is_empty() {
                return (0, cands);
            }
        }
        // ── Case 2c: path completion — cd or path-like argument ─────────────────
        if line.starts_with("cd ") {
            let partial = &line["cd ".len()..];
            let cands = path_completions(partial)
                .into_iter()
                .map(|p| format!("cd {}", p))
                .collect();
            return (0, cands);
        }
        // any argument that could be a path — starts with / ~/ ./ or is a bare filename
        if line.contains(' ') {
            let last = line.split_whitespace().last().unwrap_or("");
            let is_path_like = last.starts_with('/')
                || last.starts_with("~/")
                || last.starts_with("./")
                || (!last.starts_with('-') && !last.is_empty());
            if is_path_like {
                let cands = path_completions(last);
                if !cands.is_empty() {
                    let start = line.len() - last.len();
                    return (start, cands);
                }
            }
        }

        // ── Case 2d: dynamic intent ID completion ──────────────────────────
        if line.starts_with("intent show ")
            || line.starts_with("cistart ")
            || line.starts_with("cicomplete ")
        {
            let cmd_len = line
                .find(' ')
                .map(|i| line[i + 1..].find(' ').map(|j| i + j + 2).unwrap_or(i + 1))
                .unwrap_or(line.len());
            let partial = &line[cmd_len..];
            let home = std::env::var("HOME").unwrap_or_default();
            let mut ids: Vec<String> = Vec::new();
            for dir in &["intents/future", "intents/complete"] {
                let path = format!("{}/0-core/{}", home, dir);
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for e in entries.flatten() {
                        if let Some(name) = e
                            .path()
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                        {
                            if let Some(id) = name.split('-').next() {
                                if id.chars().all(|c| c.is_ascii_digit()) {
                                    if partial.is_empty() || id.starts_with(partial) {
                                        ids.push(id.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ids.sort();
            ids.dedup();
            if !ids.is_empty() {
                let start = line.len() - partial.len();
                return (start, ids);
            }
        }

        // ── Case 2d2: git branch completion ───────────────────────────────────
        if line.starts_with("git checkout ")
            || line.starts_with("git merge ")
            || line.starts_with("git rebase ")
            || line.starts_with("git branch ")
            || line.starts_with("git diff ")
            || line.starts_with("fg checkout ")
        {
            // If line ends with space, partial is empty (show all branches)
            // Otherwise take the last token
            let partial = if line.ends_with(' ') {
                ""
            } else {
                line.split_whitespace().last().unwrap_or("")
            };
            // Skip if partial looks like a flag
            if !partial.starts_with('-') {
                let branches: Vec<String> = std::process::Command::new("git")
                    .args(["branch", "-a", "--format=%(refname:short)"])
                    .output()
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout)
                            .lines()
                            .map(|l| l.trim().to_string())
                            .filter(|b| !b.is_empty() && b.starts_with(partial))
                            .collect()
                    })
                    .unwrap_or_default();
                if !branches.is_empty() {
                    let start = line.len() - partial.len();
                    return (start, branches);
                }
            }
        }
        // ── Case 2e: alias completion from state.db ─────────────────────────
        if !line.contains(' ') && !line.is_empty() {
            let mut alias_names: Vec<String> = Vec::new();
            if let Ok(mut stmt) = self.db.conn
                .prepare("SELECT name FROM shell_aliases WHERE name LIKE ?1 ORDER BY name")
            {
                let pattern = format!("{}%", line);
                if let Ok(rows) = stmt.query_map([&pattern], |r| r.get::<_, String>(0)) {
                    for row in rows.flatten() {
                        alias_names.push(row);
                    }
                }
            }
            let mut cands: Vec<String> = COMMANDS
                .iter()
                .filter(|c| c.starts_with(line))
                .map(|s| s.to_string())
                .collect();
            cands.extend(alias_names);
            let mut bins = binary_completions(line);
            cands.append(&mut bins);
            cands.sort();
            cands.dedup();
            if !cands.is_empty() {
                return (0, cands);
            }
        }

        // ── Case 3: first word — static list + PATH binaries ─────────────────
        if !line.contains(' ') {
            let mut cands: Vec<String> = COMMANDS
                .iter()
                .filter(|c| c.starts_with(line))
                .map(|s| s.to_string())
                .collect();
            let mut bins = binary_completions(line);
            cands.append(&mut bins);
            cands.sort();
            cands.dedup();
            return (0, cands);
        }

        (line.len(), vec![])
    }
}

// ── path_completions — filesystem completion with ~/ expansion ────────────────
fn path_completions(partial: &str) -> Vec<String> {
    let expanded = if partial.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_default();
        partial.replacen("~/", &format!("{}/", home), 1)
    } else {
        partial.to_string()
    };

    let (dir, stem) = if expanded.is_empty() || expanded.ends_with('/') {
        (
            if expanded.is_empty() {
                "./".to_string()
            } else {
                expanded.clone()
            },
            String::new(),
        )
    } else {
        let p = std::path::Path::new(&expanded);
        let parent = p
            .parent()
            .map(|d| {
                let s = d.to_string_lossy().to_string();
                if s.is_empty() {
                    "./".to_string()
                } else {
                    format!("{}/", s)
                }
            })
            .unwrap_or_else(|| "./".to_string());
        let stem = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        (parent, stem)
    };

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return vec![];
    };

    let home = std::env::var("HOME").unwrap_or_default();
    let mut results: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && !stem.starts_with('.') {
                return None; // hide dotfiles unless user typed a dot
            }
            if name.starts_with(&stem) {
                let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let full = format!("{}{}{}", dir, name, if is_dir { "/" } else { "" });
                // restore ~/ prefix if original started with ~/
                let display = if partial.starts_with("~/") {
                    full.replacen(&format!("{}/", home), "~/", 1)
                } else {
                    full
                };
                Some(display)
            } else {
                None
            }
        })
        .collect();

    results.sort();
    results
}

// ── binary_completions — scan $PATH for matching executables ──────────────────
fn binary_completions(partial: &str) -> Vec<String> {
    if partial.is_empty() {
        return vec![];
    }
    let path_env = std::env::var("PATH").unwrap_or_default();
    let mut results: Vec<String> = path_env
        .split(':')
        .flat_map(|dir| std::fs::read_dir(dir).ok().into_iter().flatten().flatten())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(partial) {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    results.sort();
    results.dedup();
    results
}

impl<'a> Completer for ForestHelper<'a> {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, cands) = self.completions_for(&line[..pos]);
        Ok((
            start,
            cands
                .into_iter()
                .map(|c| Pair {
                    display: c.clone(),
                    replacement: c,
                })
                .collect(),
        ))
    }
}

impl<'a> Hinter for ForestHelper<'a> {
    type Hint = String;
}

impl<'a> Highlighter for ForestHelper<'a> {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }
    fn highlight_char(
        &self,
        _line: &str,
        _pos: usize,
        _forced: rustyline::highlight::CmdKind,
    ) -> bool {
        false
    }
}

impl<'a> Validator for ForestHelper<'a> {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        let input = ctx.input();

        // Check for heredoc patterns
        if input.contains("<<") {
            // Look for unquoted heredoc delimiter (risky)
            if let Some(heredoc_start) = input.find("<<") {
                let after_heredoc = &input[heredoc_start + 2..].trim_start();

                // Check if delimiter is unquoted (no quotes around it)
                if !after_heredoc.is_empty()
                    && !after_heredoc.starts_with('\'')
                    && !after_heredoc.starts_with('"')
                {
                    // Extract delimiter (first word)
                    let delimiter = after_heredoc.split_whitespace().next().unwrap_or("");

                    // Warn about common contamination patterns
                    if !delimiter.is_empty() && (delimiter.contains("EOF") || delimiter.len() < 10)
                    {
                        return Ok(rustyline::validate::ValidationResult::Invalid(
                            Some(format!("Unquoted heredoc delimiter '{}' - use << '{}' to prevent command substitution", delimiter, delimiter))
                        ));
                    }
                }
            }
        }

        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}
impl<'a> Helper for ForestHelper<'a> {}
