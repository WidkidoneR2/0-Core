// faelight-shell Phase 6 — .fsh scripting language
// Variables, control flow, script execution
// "Not for automating tasks. For expressing forest behavior."

use crate::commands::{execute, CommandResult};
use crate::db::ForestDb;
use crate::value::Value;
use colored::*;
use std::collections::HashMap;

/// Variable scope — let bindings
#[derive(Debug, Clone, Default)]
pub struct Scope {
    vars: HashMap<String, Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: &str, val: Value) {
        self.vars.insert(name.to_string(), val);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    /// Interpolate $vars in a string — "hello $name" → "hello christian"
    pub fn interpolate(&self, s: &str) -> String {
        let mut result = s.to_string();
        for (k, v) in &self.vars {
            result = result.replace(&format!("${}", k), &v.as_text());
        }
        result
    }
}

/// A parsed .fsh statement
#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        expr: String,
    },
    If {
        condition: String,
        body: Vec<Statement>,
    },
    When {
        event: String,
        body: Vec<Statement>,
    },
    Run {
        command: String,
    },
    Emit {
        event: String,
        payload: Option<String>,
    },
    Warn {
        message: String,
    },
    Confirm {
        message: String,
    },
}

/// Parse .fsh source into statements
pub fn parse(source: &str) -> Vec<Statement> {
    let mut stmts = vec![];
    let lines: Vec<&str> = source
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        if let Some(rest) = line.strip_prefix("let ") {
            // let name = expr
            if let Some((name, expr)) = rest.split_once('=') {
                stmts.push(Statement::Let {
                    name: name.trim().to_string(),
                    expr: expr.trim().to_string(),
                });
            }
            i += 1;
        } else if let Some(rest) = line.strip_prefix("if ") {
            // if condition {
            let condition = rest.trim_end_matches('{').trim().to_string();
            let body = collect_block(&lines, &mut i);
            stmts.push(Statement::If { condition, body });
        } else if let Some(rest) = line.strip_prefix("when ") {
            // when event {
            let event = rest.trim_end_matches('{').trim().to_string();
            let body = collect_block(&lines, &mut i);
            stmts.push(Statement::When { event, body });
        } else if let Some(rest) = line.strip_prefix("run ") {
            stmts.push(Statement::Run {
                command: rest.trim().to_string(),
            });
            i += 1;
        } else if let Some(rest) = line.strip_prefix("emit ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            stmts.push(Statement::Emit {
                event: parts[0].to_string(),
                payload: parts.get(1).map(|s| s.to_string()),
            });
            i += 1;
        } else if let Some(rest) = line.strip_prefix("warn ") {
            stmts.push(Statement::Warn {
                message: rest.trim_matches('"').to_string(),
            });
            i += 1;
        } else if let Some(rest) = line.strip_prefix("confirm ") {
            stmts.push(Statement::Confirm {
                message: rest.trim_matches('"').to_string(),
            });
            i += 1;
        } else {
            // Treat as raw command
            stmts.push(Statement::Run {
                command: line.to_string(),
            });
            i += 1;
        }
    }
    stmts
}

fn collect_block(lines: &[&str], i: &mut usize) -> Vec<Statement> {
    *i += 1; // skip opening line
    let mut block_lines = vec![];
    let mut depth = 1;
    while *i < lines.len() {
        let line = lines[*i];
        if line.ends_with('{') {
            depth += 1;
        }
        if line == "}" {
            depth -= 1;
            if depth == 0 {
                *i += 1;
                break;
            }
        }
        block_lines.push(line);
        *i += 1;
    }
    parse(&block_lines.join("\n"))
}

/// Execute a list of statements
pub fn run_stmts(stmts: &[Statement], scope: &mut Scope, db: &ForestDb, core_root: &str) -> bool {
    for stmt in stmts {
        if !run_stmt(stmt, scope, db, core_root) {
            return false;
        }
    }
    true
}


// A "literal" expression is a plain value — not a command to execute.
// Strings, numbers, booleans, and single words that are not known commands.
fn is_literal(s: &str) -> bool {
    let s = s.trim();
    // Quoted string
    if s.starts_with('"') && s.ends_with('"') {
        return true;
    }
    // Number
    if s.parse::<f64>().is_ok() {
        return true;
    }
    // Boolean
    if s == "true" || s == "false" {
        return true;
    }
    // Single word with no spaces — treat as literal value, not a command
    // This covers $1 expansions like "faelight-shell", "0.7.0", etc.
    if !s.contains(' ') && !s.contains('|') {
        return true;
    }
    false
}

fn run_stmt(stmt: &Statement, scope: &mut Scope, db: &ForestDb, core_root: &str) -> bool {
    match stmt {
        Statement::Let { name, expr } => {
            let expanded = scope.interpolate(expr);
            // If the expression is a quoted string literal — store directly
            // If it looks like a command (contains pipe, known verb, etc.) — execute
            let val = if is_literal(&expanded) {
                Value::Text(expanded.trim_matches('"').to_string())
            } else {
                match execute(&expanded, db, core_root) {
                    CommandResult::Value(v) => v,
                    CommandResult::Output(s) => Value::Text(s),
                    _ => Value::Text(expanded),
                }
            };
            scope.set(name, val);
            true
        }

        Statement::If { condition, body } => {
            let expanded = scope.interpolate(condition);
            if eval_condition(&expanded, scope, db, core_root) {
                run_stmts(body, scope, db, core_root)
            } else {
                true
            }
        }

        Statement::When { event, body } => {
            // Check if event occurred recently in state.db
            let occurred = check_event(event, db);
            if occurred {
                run_stmts(body, scope, db, core_root)
            } else {
                true
            }
        }

        Statement::Run { command } => {
            let expanded = scope.interpolate(command);
            match execute(&expanded, db, core_root) {
                CommandResult::Value(v) => {
                    println!("{}", v.render());
                    true
                }
                CommandResult::Output(o) => {
                    println!("{}", o);
                    true
                }
                CommandResult::Error(e) => {
                    eprintln!("  {} {}", "✗".bright_red(), e);
                    false
                }
                _ => true,
            }
        }

        Statement::Emit { event, payload } => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let parts: Vec<&str> = event.splitn(2, '.').collect();
            let (domain, action) = if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                ("shell", event.as_str())
            };
            db.conn.execute(
                "INSERT INTO events (domain, action, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![domain, action, payload.as_deref().unwrap_or(""), ts]
            ).ok();
            println!("  {} emit {}", "→".dimmed(), event.bright_cyan());
            true
        }

        Statement::Warn { message } => {
            let expanded = scope.interpolate(message);
            println!("  {} {}", "⚠".yellow(), expanded.yellow());
            true
        }

        Statement::Confirm { message } => {
            let expanded = scope.interpolate(message);
            print!("  {} {} [y/n]: ", "?".bright_cyan(), expanded);
            use std::io::{BufRead, Write};
            std::io::stdout().flush().ok();
            let stdin = std::io::stdin();
            let answer = stdin
                .lock()
                .lines()
                .next()
                .and_then(|l| l.ok())
                .unwrap_or_default()
                .trim()
                .to_lowercase();
            answer == "y" || answer.is_empty()
        }
    }
}

fn eval_condition(cond: &str, scope: &mut Scope, db: &ForestDb, core_root: &str) -> bool {
    // Simple conditions: "health < 90", "score > 70"
    let parts: Vec<&str> = cond.split_whitespace().collect();
    if parts.len() == 3 {
        let left = resolve_value(parts[0], scope, db, core_root);
        let op = parts[1];
        let right = parts[2].parse::<f64>().unwrap_or(0.0);
        if let Ok(left_num) = left.parse::<f64>() {
            return match op {
                "<" => left_num < right,
                ">" => left_num > right,
                "<=" => left_num <= right,
                ">=" => left_num >= right,
                "==" => (left_num - right).abs() < f64::EPSILON,
                "!=" => (left_num - right).abs() >= f64::EPSILON,
                _ => false,
            };
        }
        // String comparison
        match op {
            "==" => left == right.to_string(),
            "!=" => left != right.to_string(),
            _ => false,
        }
    } else {
        // Non-empty string = true
        !cond.is_empty() && cond != "false" && cond != "0"
    }
}

fn resolve_value(name: &str, scope: &mut Scope, db: &ForestDb, core_root: &str) -> String {
    // Check scope first
    if let Some(v) = scope.get(name) {
        return v.as_text();
    }
    // Built-in forest values
    match name {
        "health" => db.health_score().unwrap_or(0).to_string(),
        "commits" => std::process::Command::new("git")
            .args(["-C", core_root, "rev-list", "--count", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "0".to_string()),
        _ => name.to_string(),
    }
}

fn check_event(event: &str, db: &ForestDb) -> bool {
    let parts: Vec<&str> = event.splitn(2, '.').collect();
    if parts.len() == 2 {
        let since = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - 3600; // last hour
        db.conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE domain=?1 AND action=?2 AND timestamp > ?3",
                rusqlite::params![parts[0], parts[1], since],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0
    } else {
        false
    }
}

/// Execute a .fsh script file
pub fn run_file(path: &str, db: &ForestDb, core_root: &str, script_args: &[&str]) -> CommandResult {
    // Parse flags from script_args
    let trace   = script_args.contains(&"--trace");
    let dry_run = script_args.contains(&"--dry-run");
    let verbose = script_args.contains(&"--verbose");
    let clean_args: Vec<&str> = script_args.iter()
        .filter(|a| !a.starts_with("--"))
        .copied()
        .collect();

    match std::fs::read_to_string(path) {
        Ok(source) => {
            let stmts = parse(&source);
            let mut scope = Scope::new();
            for (i, arg) in clean_args.iter().enumerate() {
                scope.set(&(i + 1).to_string(), Value::Text(arg.to_string()));
            }
            scope.set("#", Value::Text(clean_args.len().to_string()));

            if dry_run {
                println!();
                println!("  {} {} (dry run)", "🌿".normal(), path.bright_white());
                println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
                for (i, stmt) in stmts.iter().enumerate() {
                    let label = match stmt {
                        Statement::Run { command } => command.clone(),
                        Statement::Let { name, expr } => format!("let {} = {}", name, expr),
                        Statement::Emit { event, .. } => format!("emit {}", event),
                        Statement::Warn { message } => format!("warn {}", message),
                        Statement::Confirm { message } => format!("confirm {}", message),
                        Statement::If { condition, .. } => format!("if {}", condition),
                        Statement::When { event, .. } => format!("when {}", event),
                    };
                    println!("  [{:>2}] {} {}", i+1, "○".dimmed(), label.bright_cyan());
                }
                println!();
                println!("  {} dry run complete — {} steps", "○".dimmed(), stmts.len());
                println!();
                return CommandResult::Empty;
            }

            println!("  {} {}", "🌿 running".dimmed(), path.bright_white());
            if trace { println!("  {} trace mode active", "→".bright_cyan()); }
            if verbose { println!("  {} verbose mode active", "→".bright_cyan()); }
            if !clean_args.is_empty() {
                println!("  {} args: {}", "→".dimmed(), clean_args.join(" ").bright_white());
            }
            println!();

            if trace {
                // Trace mode: run each statement individually with timing
                let mut all_ok = true;
                for (i, stmt) in stmts.iter().enumerate() {
                    let stmt_str = match stmt {
                        Statement::Run { command } => command.clone(),
                        Statement::Let { name, expr } => format!("let {} = {}", name, expr),
                        Statement::Emit { event, .. } => format!("emit {}", event),
                        Statement::Warn { message } => format!("warn {}", message),
                        Statement::Confirm { message } => format!("confirm {}", message),
                        Statement::If { condition, .. } => format!("if {}", condition),
                        Statement::When { event, .. } => format!("when {}", event),
                    };
                    let start = std::time::Instant::now();
                    let single: Vec<Statement> = vec![stmt.clone()];
                    let ok = run_stmts(&single, &mut scope, db, core_root);
                    let elapsed = start.elapsed();
                    let ms = elapsed.as_millis();
                    if ok {
                        println!("  [{:>2}] {} {}  {} ({}ms)",
                            i+1,
                            "✅".normal(),
                            stmt_str.bright_white(),
                            "".dimmed(),
                            ms);
                        if verbose {
                            // Show any vars set in this step
                            println!("       {} scope after step", "→".dimmed());
                        }
                    } else {
                        println!("  [{:>2}] {} {}  ({}ms)",
                            i+1,
                            "❌".normal(),
                            stmt_str.bright_white(),
                            ms);
                        all_ok = false;
                        break;
                    }
                }
                println!();
                if all_ok {
                    return CommandResult::Output(format!("  {} trace complete", "✅".normal()));
                } else {
                    return CommandResult::Error("script failed — see trace above".to_string());
                }
            }

            let ok = run_stmts(&stmts, &mut scope, db, core_root);
            if ok {
                CommandResult::Output(format!("  {} script complete", "✅".normal()))
            } else {
                CommandResult::Error("script exited with error".to_string())
            }
        }
        Err(e) => CommandResult::Error(format!("cannot read {}: {}", path, e)),
    }
}

/// Run .fsh source directly (for REPL inline blocks)
#[allow(dead_code)]
pub fn run_source(source: &str, db: &ForestDb, core_root: &str) -> CommandResult {
    let stmts = parse(source);
    let mut scope = Scope::new();
    let ok = run_stmts(&stmts, &mut scope, db, core_root);
    if ok {
        CommandResult::Empty
    } else {
        CommandResult::Error("script exited with error".to_string())
    }
}
