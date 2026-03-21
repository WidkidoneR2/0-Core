// faelight-shell — Schema-Aware Completion
// Phase 11: Tab completion that knows column names, commands, and pipeline ops

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;

use crate::schema::SchemaRegistry;

const COMMANDS: &[&str] = &[
    "health", "events", "decisions", "intents", "tools", "version",
    "schema", "commits", "story", "advise", "audit", "forecast",
    "sandbox", "checkpoint", "cpc", "git", "search", "s",
    "tt", "tools-table", "et", "events-table", "at", "audit-table",
    "dt", "decisions-table", "ht", "history-table", "ct", "checkpoints-table",
    "domains", "histogram", "hist", "logs",
    "ps", "processes", "ports", "services", "svc", "files", "ls",
    "net", "network", "pkgs", "packages",
    "gc", "git-commits", "gf", "git-files",
    "watch", "alias", "unalias", "plugins",
    "help", "h", "?", "exit", "quit", "q",
];

const PIPE_OPS: &[&str] = &[
    "where", "sort", "select", "first", "last", "count", "get", "watch", "join",
];

pub struct ForestHelper {
    registry: SchemaRegistry,
}

impl ForestHelper {
    pub fn new() -> Self {
        ForestHelper { registry: SchemaRegistry::build() }
    }

    fn completions_for(&self, line: &str) -> (usize, Vec<String>) {
        // ── Case 1: pipe present ──────────────────────────────────────────────
        if let Some(pipe_pos) = line.rfind(" | ") {
            let after = &line[pipe_pos + 3..];
            let _trailing_space = after.ends_with(' ');
            let base_cmd = line[..pipe_pos].trim()
                .split_whitespace().next().unwrap_or("").to_lowercase();
            let tokens: Vec<&str> = after.split_whitespace().collect();

            return match tokens.as_slice() {
                // "ps | <TAB>" — suggest pipe ops
                [] => (pipe_pos + 3, PIPE_OPS.iter().map(|s| s.to_string()).collect()),

                // "ps | where <TAB>" — op is complete word, suggest columns with leading space
                [op] if matches!(*op, "where" | "sort" | "select" | "get") =>
                {
                    if let Some(s) = self.registry.get(&base_cmd) {
                        (line.len(), s.column_names().iter().map(|c| format!(" {}", c)).collect())
                    } else {
                        (line.len(), vec![])
                    }
                }

                // "ps | where cp<TAB>" — filter columns
                [op, partial]
                    if matches!(*op, "where" | "sort" | "select" | "get") =>
                {
                    if let Some(s) = self.registry.get(&base_cmd) {
                        let start = line.len() - partial.len();
                        let cands: Vec<String> = s.column_names().iter()
                            .filter(|c| c.starts_with(*partial))
                            .map(|c| c.to_string()).collect();
                        (start, cands)
                    } else {
                        (line.len(), vec![])
                    }
                }

                // "ps | wh<TAB>" — complete op name
                [partial] => {
                    let start = pipe_pos + 3;
                    let cands: Vec<String> = PIPE_OPS.iter()
                        .filter(|op| op.starts_with(*partial))
                        .map(|s| s.to_string()).collect();
                    (start, cands)
                }

                _ => (line.len(), vec![]),
            };
        }

        // ── Case 2: schema <TAB> ──────────────────────────────────────────────
        if line == "schema" || line.starts_with("schema ") {
            let partial = if line.starts_with("schema ") { &line["schema ".len()..] } else { "" };
            let cands: Vec<String> = self.registry.names().iter()
                .filter(|n| n.starts_with(partial))
                .map(|n| format!("schema {}", n)).collect();
            return (0, cands);
        }

        // ── Case 3: first word ────────────────────────────────────────────────
        if !line.contains(' ') {
            let cands: Vec<String> = COMMANDS.iter()
                .filter(|c| c.starts_with(line))
                .map(|s| s.to_string()).collect();
            return (0, cands);
        }

        (line.len(), vec![])
    }
}

impl Completer for ForestHelper {
    type Candidate = Pair;
    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>)
        -> rustyline::Result<(usize, Vec<Pair>)>
    {
        let (start, cands) = self.completions_for(&line[..pos]);
        Ok((start, cands.into_iter().map(|c| Pair { display: c.clone(), replacement: c }).collect()))
    }
}

impl Hinter for ForestHelper { type Hint = String; }

impl Highlighter for ForestHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }
    fn highlight_char(&self, _line: &str, _pos: usize, _forced: rustyline::highlight::CmdKind) -> bool {
        false
    }
}

impl Validator for ForestHelper {}
impl Helper for ForestHelper {}
