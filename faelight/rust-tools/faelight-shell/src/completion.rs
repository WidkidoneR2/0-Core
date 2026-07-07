#![allow(clippy::all)]
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
    // Forest core
    "deploy",
    "cistart",
    "cicomplete",
    "intent",
    "friday",
    "friday dismiss",
    // Forest vocabulary (INT-261)
    "delete",
    "del",
    "find",
    // fsh builtins
    "patch",
    "patch-multi",
    "rspatch",
    "edit",
    "run",
    "query",
    "fsearch",
    "source",
    "fg",
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
                // -- INT-040: bare domain verbs (shell-native) --
                "intent list",
                "intent show",
                "intent search",
                "intent new",
                "intent edit",
                "vm list",
                "vm start",
                "vm stop",
                "vm status",
                "vm snapshot",
                "vm restore",
                "vm snapshots",
                "project list",
                "project status",
                "project health",
                "experiment list",
                "experiment new",
                "experiment graduate",
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
                "core strategy friday-readiness",
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
        // ── Case 2d0: INT-040 domain object dynamic completion ──────────────────
        // vm start/stop/snapshot/restore <TAB> -- complete with qcow2 names
        if line.starts_with("vm start ")
            || line.starts_with("vm stop ")
            || line.starts_with("vm snapshot ")
            || line.starts_with("vm restore ")
        {
            // partial is everything after the second word (e.g. "vm start nixos" -> "nixos")
            // if line ends with space, partial is empty (show all)
            let partial = if line.ends_with(' ') {
                ""
            } else {
                line.split_whitespace().last().unwrap_or("")
            };
            let home = std::env::var("HOME").unwrap_or_default();
            let vms_dir = format!("{}/vms", home);
            if let Ok(entries) = std::fs::read_dir(&vms_dir) {
                let mut names: Vec<String> = entries
                    .flatten()
                    .filter(|e| e.path().extension().map(|x| x == "qcow2").unwrap_or(false))
                    .map(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .trim_end_matches(".qcow2")
                            .to_string()
                    })
                    .filter(|n| partial.is_empty() || n.starts_with(partial))
                    .collect();
                names.sort();
                if !names.is_empty() {
                    let start = line.len() - partial.len();
                    return (start, names);
                }
            }
        }
        // rebuild <TAB> -- complete with flake host names (INT-040 fix: leaf name, no dup)
        if let Some(rest) = line.strip_prefix("rebuild ") {
            let partial = rest.split_whitespace().last().unwrap_or("");
            let home = std::env::var("HOME").unwrap_or_default();
            let flake = format!("{}/0-core/flake.nix", home);
            let mut hosts: Vec<String> = Vec::new();
            if let Ok(src) = std::fs::read_to_string(&flake) {
                let mut in_nixos = false;
                for l in src.lines() {
                    let t = l.trim();
                    if t.contains("nixosConfigurations") && t.contains('{') {
                        in_nixos = true;
                    }
                    if in_nixos && t.contains("= nixpkgs.lib.nixosSystem") {
                        if let Some(tok) = t.split_whitespace().next() {
                            let host = tok.rsplit('.').next().unwrap_or(tok);
                            if !host.is_empty() && (partial.is_empty() || host.starts_with(partial))
                            {
                                hosts.push(host.to_string());
                            }
                        }
                    }
                    if in_nixos && t == "};" {
                        in_nixos = false;
                    }
                }
            }
            hosts.sort();
            hosts.dedup();
            if !hosts.is_empty() {
                let start = line.len() - partial.len();
                return (start, hosts);
            }
        }
        // nix develop <TAB> -- complete with devShell + package names (INT-040)
        if let Some(rest) = line.strip_prefix("nix develop ") {
            let partial = rest.split_whitespace().last().unwrap_or("");
            let home = std::env::var("HOME").unwrap_or_default();
            let flake = format!("{}/0-core/flake.nix", home);
            let mut names: Vec<String> = Vec::new();
            if let Ok(fsrc) = std::fs::read_to_string(&flake) {
                for l in fsrc.lines() {
                    let t = l.trim();
                    if let Some(after) = t.strip_prefix("devShells.${system}.") {
                        let name = after
                            .split(|c: char| c == ' ' || c == '=')
                            .next()
                            .unwrap_or("");
                        if !name.is_empty() {
                            names.push(name.to_string());
                        }
                    }
                    if t.contains("= pkgs.rustPlatform.buildRustPackage") {
                        if let Some(name) = t.split_whitespace().next() {
                            names.push(name.to_string());
                        }
                    }
                }
            }
            names.retain(|n| partial.is_empty() || n.starts_with(partial));
            names.sort();
            names.dedup();
            if !names.is_empty() {
                let start = line.len() - partial.len();
                return (start, names);
            }
        }
        // ── Case 2c: path completion — cd or path-like argument ─────────────────
        if line.starts_with("deploy ") {
            let partial = &line["deploy ".len()..];
            let home = std::env::var("HOME").unwrap_or_default();
            let scripts = format!("{}/0-core/scripts", home);
            if let Ok(entries) = std::fs::read_dir(&scripts) {
                let mut tools: Vec<String> = entries
                    .flatten()
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        let ok = e
                            .metadata()
                            .map(|m| {
                                use std::os::unix::fs::PermissionsExt;
                                m.permissions().mode() & 0o111 != 0
                            })
                            .unwrap_or(false);
                        if ok
                            && !name.contains('.')
                            && (partial.is_empty() || name.starts_with(partial))
                        {
                            Some(name)
                        } else {
                            None
                        }
                    })
                    .collect();
                tools.sort();
                if !tools.is_empty() {
                    let start = line.len() - partial.len();
                    return (start, tools);
                }
            }
        }
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
            let dirs: &[&str] = if line.starts_with("cicomplete ") {
                &["intents/future", "intents/in-progress"]
            } else if line.starts_with("cistart ") {
                &["intents/future"]
            } else {
                &["intents/future", "intents/complete", "intents/in-progress"]
            };
            for dir in dirs {
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

        // -- Case 2d1: gen-diff generation number completion (INT-044) --------
        if line.starts_with("gen-diff ") {
            let partial = if line.ends_with(' ') {
                ""
            } else {
                line.rsplit(' ').next().unwrap_or("")
            };
            if !partial.starts_with('-') {
                let mut gens: Vec<String> = Vec::new();
                if let Ok(entries) = std::fs::read_dir("/nix/var/nix/profiles") {
                    for e in entries.flatten() {
                        if let Some(name) = e
                            .path()
                            .file_name()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                        {
                            if let Some(rest) = name.strip_prefix("system-") {
                                if let Some(num) = rest.strip_suffix("-link") {
                                    if num.chars().all(|c| c.is_ascii_digit())
                                        && (partial.is_empty() || num.starts_with(partial))
                                    {
                                        gens.push(num.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                gens.sort_by(|a, b| {
                    b.parse::<u64>()
                        .unwrap_or(0)
                        .cmp(&a.parse::<u64>().unwrap_or(0))
                });
                gens.dedup();
                if !gens.is_empty() {
                    let start = line.len() - partial.len();
                    return (start, gens);
                }
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
            if let Ok(mut stmt) = self
                .db
                .conn
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

fn cmd_description(cmd: &str) -> &'static str {
    match cmd {
        "deploy" => "build + deploy a forest tool",
        "cistart" => "start an intent",
        "cicomplete" => "complete an intent",
        "intent" => "manage the intent ledger",
        "friday" => "talk to Friday AI",
        "friday dismiss" => "dismiss Friday suggestion",
        "d" => "forest health check",
        "delete" | "del" => "safely delete a file",
        "find" => "search the forest",
        "fg" => "faelight-git helper",
        "patch" => "apply a patch to a file",
        "rspatch" => "anchor-based Rust patch",
        "edit" => "edit a file",
        "fsearch" => "search forest files",
        "core" => "forest intelligence engine",
        "gc" => "git commit shorthand",
        "tt" | "tools" => "tool registry",
        "et" | "events" => "forest events",
        _ => "",
    }
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
                .map(|c| {
                    let desc = cmd_description(&c);
                    let display = if !desc.is_empty() {
                        format!("{:<28} {}", c, desc)
                    } else {
                        c.clone()
                    };
                    Pair {
                        display,
                        replacement: c,
                    }
                })
                .collect(),
        ))
    }
}

// INT-334: command classification for syntax highlighting
// Neon candy color scheme -- the forest lights up

// Electric green -- valid forest/system commands
const NEON_GREEN: &str = "[38;2;57;255;20m";
// Neon red -- unknown commands
const NEON_RED: &str = "[38;2;255;50;50m";
// Hot magenta -- dangerous commands
const NEON_MAGENTA: &str = "[38;2;255;0;128m";
// Electric cyan -- forest-native commands
const NEON_CYAN: &str = "[38;2;0;255;220m";
// Electric purple -- natural language / semantic commands
const NEON_PURPLE: &str = "[38;2;180;0;255m";
// Bright amber -- warnings
const NEON_AMBER: &str = "[38;2;255;165;0m";
const RESET: &str = "[0m";

fn is_dangerous_command(cmd: &str) -> bool {
    const DANGEROUS: &[&str] = &[
        "rm", "sudo", "dd", "kill", "pkill", "killall", "chmod", "chown", "mkfs", "fdisk",
        "parted", "shred", "wipefs", "truncate",
    ];
    DANGEROUS.contains(&cmd)
}

fn is_forest_command(cmd: &str) -> bool {
    const FOREST: &[&str] = &[
        "cistart",
        "cicomplete",
        "ds",
        "dc",
        "deploy",
        "rebuild",
        "rebuild-safe",
        "rebuild-dry",
        "rebuild-check",
        "rollback",
        "update-flake",
        "friday",
        "intent",
        "intents",
        "project",
        "experiment",
        "vm",
        "fm",
        "fmd",
        "d",
        "gc",
        "gp",
        "core",
        "fsh",
        "snapshot",
        "where",
        "fsearch",
        "patch",
        "edit",
        "run",
        "query",
    ];
    FOREST.contains(&cmd)
}

fn is_natural_language(line: &str) -> bool {
    const NL_PREFIXES: &[&str] = &[
        "what ",
        "show ",
        "how ",
        "find ",
        "where ",
        "when ",
        "why ",
        "list ",
        "tell ",
        "give ",
        "help ",
        "focus",
        "start work",
        "end work",
        "what did",
        "what was",
        "how is",
        "show me",
        "find me",
    ];
    let lower = line.to_lowercase();
    NL_PREFIXES.iter().any(|p| lower.starts_with(p))
}

fn is_known_command(cmd: &str) -> bool {
    const BUILTINS: &[&str] = &[
        // Navigation
        "cd",
        "ls",
        "ll",
        "la",
        "pwd",
        "which",
        "find",
        // Forest tools
        "cistart",
        "cicomplete",
        "dc",
        "ds",
        "deploy",
        "d",
        "rebuild",
        "rebuild-safe",
        "rebuild-dry",
        "rebuild-check",
        "rollback",
        "update-flake",
        "friday",
        "intent",
        "intents",
        "project",
        "experiment",
        "vm",
        "fm",
        "fmd",
        "faelight-fm",
        "gc",
        "gp",
        "fg",
        "core",
        "fsh",
        "snapshot",
        "where",
        "fsearch",
        "patch",
        "edit",
        "run",
        "query",
        "history",
        "rewind",
        // Git
        "git",
        "lazygit",
        "lg",
        // Build
        "cargo",
        "rustc",
        "make",
        "nix",
        // Shell
        "echo",
        "cat",
        "grep",
        "sed",
        "awk",
        "head",
        "tail",
        "sort",
        "uniq",
        "wc",
        "tr",
        "cut",
        "xargs",
        "tee",
        "export",
        "source",
        "exit",
        "clear",
        "c",
        // System
        "sudo",
        "rm",
        "mv",
        "cp",
        "mkdir",
        "touch",
        "chmod",
        "chown",
        "kill",
        "ps",
        "top",
        "htop",
        "systemctl",
        "journalctl",
        "env",
        // Network
        "ssh",
        "curl",
        "wget",
        // Files
        "tar",
        "zip",
        "unzip",
        "nvim",
        "vim",
        "hx",
        "bat",
        "less",
        "more",
        "man",
        "date",
        "uname",
        // Python
        "python3",
        "python",
        // Other
        "dev",
        "delete",
        "del",
        "diff",
        "list",
        // TUI launchers (REPL special-cases -- INT-092)
        "cheat",
        "it",
        "gt",
        "db",
        "ade",
        "rewind",
        // Aliases/commands resolved at runtime
        "reload",
        "help",
        "h",
    ];
    if BUILTINS.contains(&cmd) {
        return true;
    }
    // PATH check
    let path_env = std::env::var("PATH").unwrap_or_default();
    if path_env
        .split(':')
        .any(|dir| std::path::Path::new(&format!("{}/{}", dir, cmd)).exists())
    {
        return true;
    }
    // INT-092: alias check -- a command that is a defined alias is valid (green).
    // The 299 shell_aliases were previously all red unless coincidentally on PATH.
    is_known_alias(cmd)
}

/// INT-089: is `cmd` a forest word that `sh` CANNOT see? True for fsh aliases, and for
/// fsh builtins that are NOT on PATH (deploy, cistart, d, fg, ...). False for real PATH
/// tools (git, cargo) -- those run fine via sh. Used to explain the redirect/pipe -> sh
/// boundary clearly instead of the misleading "sh: <name>: command not found".
pub(crate) fn is_fsh_only_word(cmd: &str) -> bool {
    // aliases are always fsh-internal
    if is_known_alias(cmd) {
        return true;
    }
    // a forest builtin that is NOT resolvable on PATH is invisible to sh
    let on_path = {
        let path_env = std::env::var("PATH").unwrap_or_default();
        path_env
            .split(':')
            .any(|dir| std::path::Path::new(&format!("{}/{}", dir, cmd)).exists())
    };
    if on_path {
        return false;
    }
    // forest-only builtin names that sh cannot reach
    const FOREST_ONLY: &[&str] = &[
        "cistart",
        "cicomplete",
        "dc",
        "ds",
        "deploy",
        "d",
        "fg",
        "rebuild",
        "rebuild-safe",
        "rebuild-dry",
        "rebuild-check",
        "rollback",
        "update-flake",
        "intent",
        "intents",
        "project",
        "experiment",
        "vm",
        "fm",
        "fmd",
        "faelight-fm",
        "snapshot",
        "where",
        "fsearch",
        "patch",
        "query",
        "rewind",
        "dev",
        "cheat",
        "it",
        "gt",
        "db",
        "ade",
        "reload",
    ];
    FOREST_ONLY.contains(&cmd)
}

/// INT-092: is `cmd` a defined alias in state.db? Cached per-process to avoid
/// a db hit on every keystroke. Green = the alias exists and will run.
fn is_known_alias(cmd: &str) -> bool {
    use std::sync::OnceLock;
    static ALIASES: OnceLock<std::collections::HashSet<String>> = OnceLock::new();
    let set = ALIASES.get_or_init(|| {
        let mut s = std::collections::HashSet::new();
        if let Some(_home) = std::env::var_os("HOME") {
            let db_path = faelight_core::paths::state_db();
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                if let Ok(mut stmt) = conn.prepare("SELECT name FROM shell_aliases") {
                    if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                        for name in rows.flatten() {
                            s.insert(name);
                        }
                    }
                }
            }
        }
        s
    });
    set.contains(cmd)
}

impl<'a> Hinter for ForestHelper<'a> {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        if line.is_empty() || pos < line.len() {
            return None; // only hint when cursor is at end of line
        }
        // 1. History-based hint (primary)
        let history_hint = self.db.conn.query_row(
            "SELECT command FROM shell_history              WHERE command LIKE ?1 AND command != ?2 AND length(command) > ?3              ORDER BY timestamp DESC LIMIT 1",
            rusqlite::params![format!("{}%", line), line, line.len() as i64],
            |r| r.get::<_, String>(0),
        ).ok().map(|cmd| cmd[line.len()..].to_string());
        if history_hint.is_some() {
            return history_hint;
        }
        // 2. Friday-informed fallback (INT-334 Gate 2/11)
        // Find high-confidence Friday actions that start with the current input
        self.db.conn.query_row(
            "SELECT action FROM friday_patterns              WHERE action LIKE ?1 AND action != ?2 AND length(action) > ?3              AND confidence >= 0.7              ORDER BY confidence DESC, frequency DESC LIMIT 1",
            rusqlite::params![format!("{}%", line), line, line.len() as i64],
            |r| r.get::<_, String>(0),
        ).ok().map(|action| action[line.len()..].to_string())
    }
}

impl<'a> Highlighter for ForestHelper<'a> {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            return Cow::Borrowed(line);
        }
        let leading = line.len() - trimmed.len();

        // Natural language -- electric purple
        if is_natural_language(trimmed) {
            return Cow::Owned(format!(
                "{}{}{}{}",
                &line[..leading],
                NEON_PURPLE,
                trimmed,
                RESET
            ));
        }

        let first_word = trimmed.split_whitespace().next().unwrap_or("");
        if first_word.is_empty() {
            return Cow::Borrowed(line);
        }
        let rest = &line[leading + first_word.len()..];

        let cmd_color = if is_dangerous_command(first_word) {
            NEON_MAGENTA // hot magenta -- dangerous
        } else if is_forest_command(first_word) {
            NEON_CYAN // electric cyan -- forest-native
        } else if is_known_command(first_word) {
            NEON_GREEN // electric green -- valid
        } else {
            NEON_RED // neon red -- unknown
        };

        // Color args amber if dangerous command
        let rest_colored = if is_dangerous_command(first_word) && !rest.is_empty() {
            format!("{}{}{}", NEON_AMBER, rest, RESET)
        } else {
            rest.to_string()
        };

        Cow::Owned(format!(
            "{}{}{}{}{}",
            &line[..leading],
            cmd_color,
            first_word,
            RESET,
            rest_colored
        ))
    }
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("[38;2;0;180;180m{}[0m", hint))
    }
    fn highlight_char(
        &self,
        line: &str,
        _pos: usize,
        _forced: rustyline::highlight::CmdKind,
    ) -> bool {
        !line.is_empty()
    }
}
impl<'a> Validator for ForestHelper<'a> {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        let input = ctx.input();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(rustyline::validate::ValidationResult::Valid(None));
        }
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

impl<'a> Helper for ForestHelper<'a> {}
