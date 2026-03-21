// faelight-shell — command registry
// Phase 1: 10 forest-native commands

use crate::db::ForestDb;
use colored::*;


// ── Time formatting helper ───────────────────────────────────────────────────
fn fmt_time(ts: i64, fmt: &str) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|t| t.format(fmt).to_string())
        .unwrap_or_else(|| "?".to_string())
}

pub enum CommandResult {
    Output(String),
    Value(crate::value::Value),
    Empty,
    Error(String),
    Exit,
}



// ── Security Layer — log every command ───────────────────────────────────────
fn emit_command(db: &ForestDb, cmd: &str, result: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let payload = format!(
        r#"{{"actor":"faelight-shell","result":"{}","detail":{{"command":"{}"}}}}"#,
        result,
        cmd.replace('"', "'")
    );
    db.conn.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('shell', 'command', ?1, ?2)",
        rusqlite::params![payload, ts],
    ).ok();
}

fn levenshtein(a: &str, b: &str) -> usize {
    let la = a.len();
    let lb = b.len();
    let mut dp = vec![vec![0usize; lb + 1]; la + 1];
    for i in 0..=la { dp[i][0] = i; }
    for j in 0..=lb { dp[0][j] = j; }
    for i in 1..=la {
        for j in 1..=lb {
            let cost = if a.as_bytes()[i-1] == b.as_bytes()[j-1] { 0 } else { 1 };
            dp[i][j] = (dp[i-1][j] + 1)
                .min(dp[i][j-1] + 1)
                .min(dp[i-1][j-1] + cost);
        }
    }
    dp[la][lb]
}

pub fn execute(line: &str, db: &ForestDb, core_root: &str) -> CommandResult {
    let mut parts = line.trim().splitn(3, ' ');
    let cmd = match parts.next() {
        Some(c) if !c.is_empty() => c.to_lowercase(),
        _ => return CommandResult::Empty,
    };
    let args_vec: Vec<&str> = parts.collect();
    let args = args_vec.as_slice();

    // Alias resolution — check before dispatch
    if let Some(aliased) = db.get_alias(&cmd) {
        let expanded = if args.is_empty() {
            aliased.clone()
        } else {
            format!("{} {}", aliased, args.join(" "))
        };
        // Recurse with expanded command
        return execute(&expanded, db, core_root);
    }

    // Plugin resolution — after final cmd parse
    {
        let plugins = db.load_plugins();
        if let Some((_, expand, _)) = plugins.iter().find(|(name, _, _)| name.as_str() == cmd.as_str()) {
            let expanded = if args.is_empty() {
                expand.clone()
            } else {
                format!("{} {}", expand, args.join(" "))
            };
            return execute(&expanded, db, core_root);
        }
    }

    let result = match cmd.as_str() {
        "on" => on_cmd(db, args),
        "help" | "h" => help(),
        "?" => CommandResult::Output(crate::nl::render_pattern_list()),
        "exit" | "quit" | "q" => CommandResult::Exit,
        "health" => health(db),
        "events" => events(db, args),
        "decisions" => decisions(db),
        "intents" => intents(core_root),
        "tools" => tools(db, core_root),
        "version" => version(core_root),
        "schema" => schema(args),
        "commits" => commits(core_root),
        "story" => story(db),
        "advise" => advise(db),
        "audit" => audit(db, core_root),
        "forecast" => forecast(db),
        "sandbox" => sandbox(db),
        "checkpoint" | "cpc" => checkpoint(db),
        "snapshot" => snapshot_cmd(db, args),
        "timeline" => timeline_cmd(db, args),
        "snap-diff" => snap_diff_cmd(db, args),
        "select" => sql_query_cmd(db, core_root, line),
        "git" => git_status(core_root),
        "search" | "s" => search(db, args),
        "where" => CommandResult::Error("use with pipe: tools | where score < 70".to_string()),
        "tools-table" | "tt" => tools_table(db, core_root),
        "events-table" | "et" => events_table(db, args),
        "audit-table" | "at" => audit_table(db, core_root),
        "decisions-table" | "dt" => decisions_table(db),
        "count" => CommandResult::Output("  use with pipe: tt | count".to_string()),
        "history-table" | "ht" => history_table(db),
        "hstats" => history_stats(db),
        "hpattern" => history_pattern(db),
        "checkpoints-table" | "ct" => checkpoints_table(db),
        "domains" => domains(db),
        "histogram" | "hist" => histogram(db, args),
        "logs" => sys_logs(args),
        "ps" | "processes" => sys_processes(),
        "ports" => sys_ports(),
        "services" | "svc" => sys_services(),
        "files" | "ls" => sys_files(core_root, args),
        "find" | "fd" => find_cmd(db, core_root, args),
        "net" | "network" => sys_network(),
        "pkgs" | "packages" => sys_packages(),
        "git-commits" | "gc" => git_commits(core_root, args),
        "git-files" | "gf" => git_files(core_root),
        "git-churn" | "gchurn" => git_churn(core_root, args),
        "git-branches" | "gbr" => git_branches(core_root),
        "watch" => watch_cmd(db, args),
        "alias" => alias_cmd(db, args),
        "unalias" => unalias_cmd(db, args),
        "plugins" => list_plugins(db),
        "plugin-reload" | "plr" => reload_plugins_cmd(db),
        "cd" => cd(args),
        "clear" | "c" | "cls" => { print!("\x1B[2J\x1B[1;1H"); use std::io::Write; std::io::stdout().flush().ok(); CommandResult::Output(crate::prompt::status_line(db)) }
        _ => {
            let known = ["health","events","decisions","intents","tools","audit","ps","ports","services","files","net","pkgs",
                "forecast","sandbox","story","advise","version","commits","cd",
                "clear","exit","search","checkpoint","git","tt","et","at","dt",
                "ht","ct","domains","help"];
            let suggestion = known.iter()
                .filter(|k| {
                    let k = k.to_string();
                    let c = cmd.as_str();
                    k.starts_with(&c[..c.len().min(3)]) ||
                    c.starts_with(&k[..k.len().min(3)]) ||
                    levenshtein(&k, c) <= 2
                })
                .min_by_key(|k| levenshtein(k, &cmd));

            if let Some(s) = suggestion {
                CommandResult::Error(format!(
                    "Unknown command: {}  — did you mean {}?",
                    cmd.bright_white(), s.bright_cyan()
                ))
            } else {
                CommandResult::Error(format!(
                    "Unknown command: {}  — type {} for help",
                    cmd.bright_white(), "help".bright_cyan()
                ))
            }
        }
    };

    // Security layer — log every command
    let result_str = match &result {
        CommandResult::Error(_) => "error",
        CommandResult::Exit    => "exit",
        CommandResult::Empty   => "empty",
        _                      => "ok",
    };
    emit_command(db, &cmd, result_str);
    result
}

fn forecast(db: &ForestDb) -> CommandResult {
    // Read last 10 doctor events and compute trend
    let points: Vec<i64> = {
        let mut stmt = match db.conn.prepare(
            "SELECT payload FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 10"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Error("No forecast data".to_string()),
        };
        stmt.query_map([], |r| r.get::<_,String>(0))
            .map(|rows| rows.filter_map(|r| r.ok())
                .filter_map(|p| serde_json::from_str::<serde_json::Value>(&p).ok())
                .filter_map(|v| v["detail"]["health"].as_i64())
                .collect())
            .unwrap_or_default()
    };

    if points.len() < 3 {
        return CommandResult::Output(format!(
            "  {} Not enough data for forecast yet — run {} a few times",
            "○".dimmed(), "d".bright_cyan()
        ));
    }

    let current = points[0];
    let recent_avg: f64 = points.iter().take(3).map(|h| *h as f64).sum::<f64>() / 3.0;
    let older_avg: f64 = points.iter().skip(3).map(|h| *h as f64).sum::<f64>() / (points.len() - 3) as f64;
    let trend = recent_avg - older_avg;
    let forecast_24h = (current as f64 + trend * 0.5).round() as i64;
    let forecast_7d = (current as f64 + trend * 2.0).round() as i64;
    let forecast_24h = forecast_24h.max(0).min(100);
    let forecast_7d = forecast_7d.max(0).min(100);

    let trend_icon = if trend > 1.0 { "📈" } else if trend < -1.0 { "📉" } else { "➡️ " };
    let trend_str = if trend > 0.5 { format!("+{:.1}", trend).bright_green().to_string() }
        else if trend < -0.5 { format!("{:.1}", trend).yellow().to_string() }
        else { "stable".dimmed().to_string() };

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 📈 Health Forecast ──────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Current:  {}%\n", current.to_string().bright_white().bold()));
    out.push_str(&format!("  │  24h:      {}%\n", forecast_24h.to_string().bright_green()));
    out.push_str(&format!("  │  7d:       {}%\n", forecast_7d.to_string().bright_green()));
    out.push_str(&format!("  │  Trend:    {} {}\n", trend_icon, trend_str));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn sandbox(db: &ForestDb) -> CommandResult {
    let rows: Vec<(String, String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT payload, action, timestamp FROM events WHERE domain='sandbox' ORDER BY timestamp DESC LIMIT 10"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No sandbox runs yet", "○".dimmed())),
        };
        stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No sandbox runs recorded — use {}",
            "○".dimmed(), "faelight-sandbox run".bright_cyan()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🧪 Sandbox Runs ───────────────────────────────────".bright_cyan()));
    for (payload, _, ts) in &rows {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
            let cmd = v["detail"]["command"].as_str().unwrap_or("unknown");
            let result = v["result"].as_str().unwrap_or("?");
            let dur = v["detail"]["duration_secs"].as_u64().unwrap_or(0);
            let changed = v["detail"]["files_changed"].as_u64().unwrap_or(0);
            let icon = if result == "ok" { "✅" } else { "❌" };
            let time = fmt_time(*ts, "%H:%M");
            let short_cmd = if cmd.len() > 35 { format!("{}...", &cmd[..35]) } else { cmd.to_string() };
            out.push_str(&format!("  │  {} {}  {}  {}s  {} files\n",
                icon,
                time.dimmed(),
                short_cmd.bright_white(),
                dur.to_string().dimmed(),
                changed.to_string().cyan(),
            ));
        }
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn checkpoint(db: &ForestDb) -> CommandResult {
    let rows: Vec<(String, String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT name, payload, timestamp FROM checkpoints ORDER BY timestamp DESC LIMIT 8"
        ) {
            Ok(s) => s,
            Err(_) => {
                // Try events table for checkpoint events
                let mut stmt2 = match db.conn.prepare(
                    "SELECT action, payload, timestamp FROM events WHERE domain='checkpoint' ORDER BY timestamp DESC LIMIT 8"
                ) {
                    Ok(s) => s,
                    Err(_) => return CommandResult::Output(format!("  {} No checkpoints found", "○".dimmed())),
                };
                return CommandResult::Output({
                    let rows: Vec<(String, String, i64)> = stmt2
                        .query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?)))
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    if rows.is_empty() {
                        return CommandResult::Output(format!("  {} No checkpoints yet — use {}", "○".dimmed(), "cpc <name>".bright_cyan()));
                    }
                    let mut out = String::new();
                    out.push_str(&format!("\n{}\n", "  ╭─ 📸 Checkpoints ──────────────────────────────────".bright_cyan()));
                    for (action, payload, ts) in &rows {
                        let time = fmt_time(*ts, "%m-%d %H:%M");
                        let name = serde_json::from_str::<serde_json::Value>(payload).ok()
                            .and_then(|v| v["detail"]["name"].as_str().map(|s| s.to_string()))
                            .unwrap_or_else(|| action.clone());
                        out.push_str(&format!("  │  {} {}\n", time.dimmed(), name.bright_white()));
                    }
                    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
                    out
                });
            }
        };
        stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No checkpoints yet — use {}", "○".dimmed(), "cpc <name>".bright_cyan()));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 📸 Checkpoints ──────────────────────────────────".bright_cyan()));
    for (name, _, ts) in &rows {
        let time = fmt_time(*ts, "%m-%d %H:%M");
        out.push_str(&format!("  │  {} {}\n", time.dimmed(), name.bright_white()));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn git_status(core_root: &str) -> CommandResult {
    let status = std::process::Command::new("git")
        .args(["-C", core_root, "status", "--short"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let branch = std::process::Command::new("git")
        .args(["-C", core_root, "branch", "--show-current"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let recent = std::process::Command::new("git")
        .args(["-C", core_root, "log", "--oneline", "-5"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🌿 Git Status ─────────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Branch:  {}\n", branch.bright_green()));

    if status.trim().is_empty() {
        out.push_str(&format!("  │  Status:  {}\n", "clean".bright_green()));
    } else {
        out.push_str(&format!("  │  Status:  {}\n", "uncommitted changes".yellow()));
        for line in status.lines().take(5) {
            out.push_str(&format!("  │    {}\n", line.dimmed()));
        }
    }

    out.push_str(&"  ├─────────────────────────────────────────────────────".dimmed().to_string());
    out.push_str("\n");
    out.push_str(&format!("  │  {}\n", "Recent commits:".dimmed()));
    for line in recent.lines().take(5) {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() == 2 {
            out.push_str(&format!("  │    {} {}\n",
                parts[0].bright_yellow(),
                parts[1].dimmed()
            ));
        }
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn tools_table(db: &ForestDb, core_root: &str) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let tools_dir = std::path::PathBuf::from(core_root).join("rust-tools");
    let mut rows = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&tools_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !entry.path().join("Cargo.toml").exists() { continue; }

            // Get version from Cargo.toml
            let version = std::fs::read_to_string(entry.path().join("Cargo.toml"))
                .ok()
                .and_then(|t| {
                    t.lines().find(|l| l.starts_with("version = "))
                        .map(|l| l.split('"').nth(1).unwrap_or("?").to_string())
                })
                .unwrap_or_else(|| "?".to_string());

            // Get score from audit_scores
            let score: i64 = db.conn.query_row(
                "SELECT score FROM audit_scores WHERE tool_name = ?1 ORDER BY timestamp DESC LIMIT 1",
                rusqlite::params![name],
                |r| r.get(0),
            ).unwrap_or(0);

            // Check if deployed
            let deployed = std::path::PathBuf::from(core_root)
                .join("scripts").join(&name).exists();

            let mut row = HashMap::new();
            row.insert("name".to_string(), Value::Text(name));
            row.insert("version".to_string(), Value::Text(version));
            row.insert("score".to_string(), Value::Int(score));
            row.insert("deployed".to_string(), Value::Bool(deployed));
            rows.push(row);
        }
    }

    rows.sort_by(|a, b| {
        a.get("name").map(|v| v.as_text()).unwrap_or_default()
            .cmp(&b.get("name").map(|v| v.as_text()).unwrap_or_default())
    });

    CommandResult::Value(Value::Table(rows))
}

fn events_table(db: &ForestDb, args: &[&str]) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let today_only = args.contains(&"today");
    let domain = args.first().and_then(|a| {
        if *a == "today" { None } else { Some(*a) }
    });

    let events = db.query_events(domain, today_only, 50);
    let rows = events.into_iter().map(|(domain, action, ts)| {
        let time = fmt_time(ts, "%H:%M:%S");
        let mut row = HashMap::new();
        row.insert("time".to_string(), Value::Text(time));
        row.insert("domain".to_string(), Value::Text(domain));
        row.insert("action".to_string(), Value::Text(action));
        row.insert("timestamp".to_string(), Value::Int(ts));
        row
    }).collect();

    CommandResult::Value(Value::Table(rows))
}

fn audit_table(db: &ForestDb, _core_root: &str) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let rows: Vec<HashMap<String, Value>> = {
        let mut stmt = match db.conn.prepare(
            "SELECT tool_name, score, usage_score, recency_score, doc_score, version_score, last_commit_days, timestamp
             FROM audit_scores
             WHERE timestamp = (SELECT MAX(timestamp) FROM audit_scores)
             ORDER BY score ASC"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No audit data — run: core audit scan", "○".dimmed())),
        };

        stmt.query_map([], |r| {
            Ok((
                r.get::<_,String>(0)?,
                r.get::<_,i64>(1)?,
                r.get::<_,i64>(2)?,
                r.get::<_,i64>(3)?,
                r.get::<_,i64>(4)?,
                r.get::<_,i64>(5)?,
                r.get::<_,Option<i64>>(6)?,
            ))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).map(|(name, score, usage, recency, doc, version, days)| {
            let mut row = HashMap::new();
            row.insert("name".to_string(), Value::Text(name));
            row.insert("score".to_string(), Value::Int(score));
            row.insert("usage".to_string(), Value::Int(usage));
            row.insert("recency".to_string(), Value::Int(recency));
            row.insert("doc".to_string(), Value::Int(doc));
            row.insert("version".to_string(), Value::Int(version));
            row.insert("days_ago".to_string(), Value::Int(days.unwrap_or(-1)));
            row
        }).collect())
        .unwrap_or_default()
    };

    CommandResult::Value(Value::Table(rows))
}

fn history_table(db: &ForestDb) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let mut stmt = match db.conn.prepare(
        "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 100"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
    };

    let rows: Vec<HashMap<String, Value>> = stmt
        .query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?)))
        .map(|rows| rows.filter_map(|r| r.ok()).map(|(cmd, ts)| {
            let time = fmt_time(ts, "%H:%M:%S");
            let mut row = HashMap::new();
            row.insert("time".to_string(), Value::Text(time));
            row.insert("command".to_string(), Value::Text(cmd));
            row.insert("timestamp".to_string(), Value::Int(ts));
            row
        }).collect())
        .unwrap_or_default();

    CommandResult::Value(Value::Table(rows))
}

fn checkpoints_table(db: &ForestDb) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let mut stmt = match db.conn.prepare(
        "SELECT action, payload, timestamp FROM events WHERE domain='checkpoint' ORDER BY timestamp DESC LIMIT 20"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No checkpoints yet", "○".dimmed())),
    };

    let rows: Vec<HashMap<String, Value>> = stmt
        .query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?)))
        .map(|rows| rows.filter_map(|r| r.ok()).map(|(action, payload, ts)| {
            let date = fmt_time(ts, "%m-%d %H:%M");
            let name = serde_json::from_str::<serde_json::Value>(&payload).ok()
                .and_then(|v| v["detail"]["name"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| action.clone());
            let health = serde_json::from_str::<serde_json::Value>(&payload).ok()
                .and_then(|v| v["detail"]["health"].as_i64())
                .unwrap_or(0);
            let mut row = HashMap::new();
            row.insert("date".to_string(), Value::Text(date));
            row.insert("name".to_string(), Value::Text(name));
            row.insert("health".to_string(), Value::Int(health));
            row
        }).collect())
        .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No checkpoints yet — use {}", "○".dimmed(), "cpc <name>".bright_cyan()));
    }

    CommandResult::Value(Value::Table(rows))
}

fn domains(db: &ForestDb) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let mut stmt = match db.conn.prepare(
        "SELECT domain, COUNT(*) as count, MAX(timestamp) as last FROM events GROUP BY domain ORDER BY count DESC"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No events yet", "○".dimmed())),
    };

    let rows: Vec<HashMap<String, Value>> = stmt
        .query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?, r.get::<_,i64>(2)?)))
        .map(|rows| rows.filter_map(|r| r.ok()).map(|(domain, count, last)| {
            let last_str = chrono::DateTime::from_timestamp(last, 0)
                .map(|t| t.format("%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "?".to_string());
            let mut row = HashMap::new();
            row.insert("domain".to_string(), Value::Text(domain));
            row.insert("events".to_string(), Value::Int(count));
            row.insert("last_seen".to_string(), Value::Text(last_str));
            row
        }).collect())
        .unwrap_or_default();

    CommandResult::Value(Value::Table(rows))
}


fn histogram(db: &ForestDb, args: &[&str]) -> CommandResult {
    
    

    // histogram domain  OR  et | histogram domain (pipe handles it)
    // Direct: show event count per domain as bar chart
    let field = args.first().copied().unwrap_or("domain");

    let mut stmt = match db.conn.prepare(
        &format!("SELECT {}, COUNT(*) FROM events GROUP BY {} ORDER BY COUNT(*) DESC LIMIT 20", field, field)
    ) {
        Ok(s) => s,
        Err(e) => return CommandResult::Error(format!("histogram error: {}", e)),
    };

    let rows: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?)))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No data for histogram", "○".dimmed()));
    }

    let max = rows.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let bar_width = 30;

    let mut out = String::new();
    out.push_str(&format!("
{}
", "  ╭─ 📊 Histogram ─────────────────────────────────────".bright_cyan()));

    for (label, count) in &rows {
        let filled = ((count * bar_width) / max) as usize;
        let empty = bar_width as usize - filled;
        let bar = format!("{}{}",
            "█".repeat(filled).bright_green(),
            "░".repeat(empty).dimmed()
        );
        out.push_str(&format!("  │  {:<18} {} {}
",
            label.bright_white(),
            bar,
            count.to_string().dimmed()
        ));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn git_commits(core_root: &str, args: &[&str]) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let limit = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(20);

    // Phase 15 — use faelight-git Repo directly (no subprocess)
    let repo_result = faelight_git::git::repo::GitRepo::open_at(core_root);

    match repo_result {
        Ok(repo) => {
            let log_entries: anyhow::Result<Vec<faelight_git::git::repo::CommitEntry>> = repo.log(limit);
                match log_entries {
                Ok(entries) => { // entries: Vec<CommitEntry>
                    let rows: Vec<HashMap<String, Value>> = entries.into_iter().map(|e| {
                        let mut row = HashMap::new();
                        row.insert("hash".to_string(),    Value::Text(e.hash));
                        row.insert("author".to_string(),  Value::Text(e.author));
                        row.insert("date".to_string(),    Value::Text(e.time_ago));
                        row.insert("message".to_string(), Value::Text(e.message));
                        row
                    }).collect();
                    if rows.is_empty() {
                        CommandResult::Output(format!("  {} No commits found", "○".dimmed()))
                    } else {
                        CommandResult::Value(Value::Table(rows))
                    }
                }
                Err(_) => {
                    // Fallback to git subprocess
                    git_commits_subprocess(core_root, limit)
                }
            }
        }
        Err(_) => git_commits_subprocess(core_root, limit),
    }
}

fn git_commits_subprocess(core_root: &str, limit: usize) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;
    let output = std::process::Command::new("git")
        .args(["-C", core_root, "log",
            &format!("-{}", limit),
            "--format=%H|%an|%ae|%ai|%s"
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let rows: Vec<HashMap<String, Value>> = output.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(5, '|').collect();
            if parts.len() < 5 { return None; }
            let mut row = HashMap::new();
            row.insert("hash".to_string(),    Value::Text(parts[0][..7].to_string()));
            row.insert("author".to_string(),  Value::Text(parts[1].to_string()));
            row.insert("date".to_string(),    Value::Text(parts[3][..10].to_string()));
            row.insert("message".to_string(), Value::Text(parts[4].to_string()));
            Some(row)
        })
        .collect();
    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No commits found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}
fn git_files(core_root: &str) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let output = std::process::Command::new("git")
        .args(["-C", core_root, "status", "--porcelain"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    if output.trim().is_empty() {
        return CommandResult::Output(format!("  {} Working tree clean", "✅".green()));
    }

    let rows: Vec<HashMap<String, Value>> = output.lines()
        .filter_map(|line| {
            if line.len() < 3 { return None; }
            let status = line[..2].trim().to_string();
            let file = line[3..].to_string();
            let kind = match status.as_str() {
                "M" | " M" => "modified",
                "A" | " A" => "added",
                "D" | " D" => "deleted",
                "??" => "untracked",
                "R" => "renamed",
                _ => "changed",
            };
            let mut row = HashMap::new();
            row.insert("status".to_string(), Value::Text(status));
            row.insert("kind".to_string(), Value::Text(kind.to_string()));
            row.insert("file".to_string(), Value::Text(file));
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

fn watch_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    use colored::*;

    let target = args.first().copied().unwrap_or("health");
    let interval = args.get(1).and_then(|a| a.parse::<u64>().ok()).unwrap_or(2);

    println!("{}", format!("  Watching: {} (every {}s) — press Ctrl+C to stop",
        target.bright_cyan(), interval).dimmed());

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });
    while running.load(Ordering::SeqCst) {
        // Clear screen and move to top
        print!("[2J[1;1H");

        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        println!("{}", format!("  🌲 watch {} — {} ({}s interval)",
            target.bright_cyan(),
            now.dimmed(),
            interval
        ));
        println!("{}", "━".repeat(52).dimmed());

        match target {
            "health" => {
                let health = db.health_score().unwrap_or(0);
                let version = std::fs::read_to_string(
                    std::path::PathBuf::from(db.core_root()).join("00-meta/VERSION")
                ).unwrap_or_default().trim().to_string();

                let status = if health >= 95 { "HEALTHY".bright_green().bold() }
                    else if health >= 80 { "ADVISORY".yellow().bold() }
                    else { "DEGRADED".bright_red().bold() };

                println!("  Health:   {} {}",
                    format!("{}%", health).bright_white().bold(), status);
                println!("  Version:  {}", version.dimmed());

                // Show last 5 events
                let events = db.query_events(None, false, 5);
                println!();
                println!("  {}", "Recent events:".dimmed());
                for (domain, action, ts) in &events {
                    let icon = match domain.as_str() {
                        "doctor" => "🩺", "git" => "🌿", "security" => "🔒",
                        "sandbox" => "🧪", "audit" => "🔍", _ => "○",
                    };
                    println!("    {} {} {}.{}",
                        icon,
                        fmt_time(*ts, "%H:%M:%S").dimmed(),
                        domain.bright_cyan(),
                        action.dimmed()
                    );
                }
            }
            "events" => {
                let events = db.query_events(None, false, 15);
                for (domain, action, ts) in &events {
                    let icon = match domain.as_str() {
                        "doctor" => "🩺", "git" => "🌿", "security" => "🔒",
                        "sandbox" => "🧪", "audit" => "🔍", _ => "○ ",
                    };
                    println!("  {} {} {}.{}",
                        icon,
                        fmt_time(*ts, "%H:%M:%S").dimmed(),
                        domain.bright_cyan(),
                        action.dimmed()
                    );
                }
            }
            _ => {
                println!("  {} Unknown watch target: {}", "✗".bright_red(), target);
                println!("  Available: health, events");
                running.store(false, Ordering::SeqCst);
            }
        }

        // Sleep in small increments to check running flag
        for _ in 0..(interval * 10) {
            if !running.load(Ordering::SeqCst) { break; }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    println!();
    println!("{}", "  watch stopped".dimmed());
    CommandResult::Empty
}

fn decisions_table(db: &ForestDb) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let rows: Vec<HashMap<String, Value>> = {
        let mut stmt = match db.conn.prepare(
            "SELECT dec_id, description, outcome, risk_score, domain, timestamp FROM decisions ORDER BY timestamp DESC LIMIT 50"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No decisions yet — use {}", "○".dimmed(), "core decide".bright_cyan())),
        };
        stmt.query_map([], |r| Ok((
            r.get::<_,String>(0)?,
            r.get::<_,String>(1)?,
            r.get::<_,String>(2)?,
            r.get::<_,f64>(3)?,
            r.get::<_,String>(4)?,
            r.get::<_,i64>(5)?,
        )))
        .map(|rows| rows.filter_map(|r| r.ok()).map(|(id, desc, outcome, risk, domain, ts)| {
            let date = fmt_time(ts, "%m-%d");
            let mut row = HashMap::new();
            row.insert("id".to_string(), Value::Text(id));
            row.insert("date".to_string(), Value::Text(date));
            row.insert("domain".to_string(), Value::Text(domain));
            row.insert("outcome".to_string(), Value::Text(outcome));
            row.insert("risk".to_string(), Value::Float(risk));
            row.insert("description".to_string(), Value::Text(desc));
            row
        }).collect())
        .unwrap_or_default()
    };

    CommandResult::Value(Value::Table(rows))
}


fn alias_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    // alias            — list all
    // alias h=health   — create
    // alias h "health" — create (space form)
    if args.is_empty() {
        let aliases = db.list_aliases();
        if aliases.is_empty() {
            return CommandResult::Output(format!(
                "  {} No aliases defined yet\n  Create one: {}",
                "○".dimmed(), "alias h=health".bright_cyan()
            ));
        }
        let mut out = String::new();
        out.push_str(&format!("\n{}\n",
            "  ╭─ 🔖 Aliases ──────────────────────────────────────".bright_cyan()
        ));
        for (name, cmd) in &aliases {
            out.push_str(&format!("  │  {:<15} = {}\n",
                name.bright_cyan(), cmd.dimmed()
            ));
        }
        out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
        return CommandResult::Output(out);
    }

    // Single arg without = — show existing alias
    if args.len() == 1 && !args[0].contains('=') {
        let name = args[0];
        if let Some(cmd) = db.get_alias(name) {
            return CommandResult::Output(format!("  {} = {}",
                name.bright_cyan(), cmd.dimmed()
            ));
        }
        return CommandResult::Error(format!("No alias: {}", name));
    }

    // Parse: alias name=command OR alias name command
    let full = args.join(" ");
    let (name, command) = if full.contains('=') {
        let mut parts = full.splitn(2, '=');
        let n = parts.next().unwrap_or("").trim().to_string();
        let c = parts.next().unwrap_or("").trim().trim_matches('"').to_string();
        (n, c)
    } else if args.len() >= 2 {
        (args[0].to_string(), args[1..].join(" ").trim_matches('"').to_string())
    } else {
        return CommandResult::Error("Usage: alias name=command".to_string());
    };

    if name.is_empty() || command.is_empty() {
        return CommandResult::Error("Usage: alias name=command".to_string());
    }

    if db.add_alias(&name, &command) {
        CommandResult::Output(format!("  {} alias {} = {}",
            "✅".green(), name.bright_cyan(), command.dimmed()
        ))
    } else {
        CommandResult::Error(format!("Failed to save alias: {}", name))
    }
}

fn unalias_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    let name = match args.first() {
        Some(n) => *n,
        None => return CommandResult::Error("Usage: unalias <name>".to_string()),
    };

    if db.remove_alias(name) {
        CommandResult::Output(format!("  {} removed alias: {}", "✅".green(), name.bright_cyan()))
    } else {
        CommandResult::Error(format!("Alias not found: {}", name))
    }
}


fn list_plugins(db: &ForestDb) -> CommandResult {
    let plugins = db.load_plugins();

    // Group by plugin file
    let plugin_dir = std::path::PathBuf::from(
        std::env::var("HOME").unwrap_or_default()
    ).join(".config/faelight-shell/plugins");

    let mut out = String::new();
    out.push_str(&format!("\n{}\n",
        "  ╭─ 🔌 Loaded Plugins ─────────────────────────────".bright_cyan()
    ));

    if plugins.is_empty() {
        out.push_str(&format!("  │  {} No plugins found\n", "○".dimmed()));
        out.push_str(&format!("  │  Add .fsh files to {}\n",
            plugin_dir.display().to_string().dimmed()
        ));
    } else {
        out.push_str(&format!("  │  {} commands from plugins:\n",
            plugins.len().to_string().bright_white()
        ));
        for (name, expand, desc) in &plugins {
            out.push_str(&format!("  │  {:<15} {} {}\n",
                name.bright_cyan(),
                "→".dimmed(),
                if desc.is_empty() { expand.dimmed().to_string() } else { desc.dimmed().to_string() }
            ));
        }
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn reload_plugins_cmd(db: &ForestDb) -> CommandResult {
    let plugins = db.load_plugins();
    CommandResult::Output(format!(
        "  {} Reloaded {} plugin commands",
        "✅".green(),
        plugins.len().to_string().bright_white()
    ))
}


// ── Phase 8 — System Tables ───────────────────────────────────────────────────

fn sys_processes() -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let output = std::process::Command::new("ps")
        .args(["aux", "--no-headers"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 11 { return None; }
            let mut row = HashMap::new();
            row.insert("pid".to_string(), Value::Text(parts[1].to_string()));
            row.insert("name".to_string(), Value::Text(
                parts[10..].join(" ").split('/').last()
                    .unwrap_or(parts[10]).chars().take(30).collect()
            ));
            row.insert("cpu".to_string(), Value::Text(parts[2].to_string()));
            row.insert("memory".to_string(), Value::Text(parts[3].to_string()));
            row.insert("user".to_string(), Value::Text(parts[0].to_string()));
            row.insert("status".to_string(), Value::Text(parts[7].to_string()));
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

fn sys_ports() -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;
    let output = std::process::Command::new("ss")
        .args(["-tlnp"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let rows: Vec<HashMap<String, Value>> = output.lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 { return None; }
            let addr = parts[3];
            let port = addr.rsplit(':').next().unwrap_or("?").to_string();
            let users_str = if parts.len() > 4 { parts[4..].join(" ") } else { String::new() };
            let process = users_str.split('"').nth(1).unwrap_or("?").to_string();
            let pid = users_str.split("pid=")
                .nth(1)
                .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
                .unwrap_or("?")
                .to_string();
            let mut row = HashMap::new();
            row.insert("port".to_string(),    Value::Text(port));
            row.insert("state".to_string(),   Value::Text(parts[0].to_string()));
            row.insert("address".to_string(), Value::Text(addr.to_string()));
            row.insert("process".to_string(), Value::Text(process));
            row.insert("pid".to_string(),     Value::Text(pid));
            Some(row)
        })
        .collect();
    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No listening ports found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}
fn sys_services() -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let output = std::process::Command::new("systemctl")
        .args(["list-units", "--type=service", "--no-pager",
               "--no-legend", "--all"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output.lines()
        .filter(|l| !l.trim().is_empty())
        .take(50)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 { return None; }
            let name = parts[0].trim_end_matches(".service").to_string();
            let load = parts[1].to_string();
            let active = parts[2].to_string();
            let sub = parts[3].to_string();
            let mut row = HashMap::new();
            row.insert("name".to_string(), Value::Text(name));
            row.insert("load".to_string(), Value::Text(load));
            row.insert("active".to_string(), Value::Text(active));
            row.insert("status".to_string(), Value::Text(sub));
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

fn sys_files(core_root: &str, args: &[&str]) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let dir = args.first().copied().unwrap_or(".");
    let path = if dir == "." {
        std::path::PathBuf::from(core_root)
    } else {
        std::path::PathBuf::from(dir)
    };

    let rows: Vec<HashMap<String, Value>> = std::fs::read_dir(&path)
        .ok()
        .map(|entries| {
            entries.flatten()
                .filter_map(|e| {
                    let meta = e.metadata().ok()?;
                    let name = e.file_name().to_string_lossy().to_string();
                    let size = meta.len();
                    let kind = if meta.is_dir() { "dir" } else { "file" }.to_string();
                    let modified = meta.modified().ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| {
                            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                .map(|t| t.format("%m-%d %H:%M").to_string())
                                .unwrap_or_else(|| "?".to_string())
                        })
                        .unwrap_or_else(|| "?".to_string());
                    let mut row = HashMap::new();
                    row.insert("name".to_string(), Value::Text(name));
                    row.insert("kind".to_string(), Value::Text(kind));
                    row.insert("size".to_string(), Value::Int(size as i64));
                    row.insert("modified".to_string(), Value::Text(modified));
                    Some(row)
                })
                .collect()
        })
        .unwrap_or_default();

    CommandResult::Value(Value::Table(rows))
}

fn sys_network() -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let output = std::process::Command::new("ip")
        .args(["-s", "link"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut rows = Vec::new();
    let mut current: HashMap<String, Value> = HashMap::new();
    let mut lines = output.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with(|c: char| c.is_ascii_digit()) {
            if !current.is_empty() { rows.push(current.clone()); current.clear(); }
            let name = line.split(':').nth(1).unwrap_or("?").trim().to_string();
            current.insert("interface".to_string(), Value::Text(name));
        } else if line.trim().starts_with("link/") {
            let mac = line.split_whitespace().nth(1).unwrap_or("?").to_string();
            current.insert("mac".to_string(), Value::Text(mac));
        } else if line.trim().starts_with("inet ") {
            let ip = line.split_whitespace().nth(1).unwrap_or("?").to_string();
            current.insert("ip".to_string(), Value::Text(ip));
        }
    }
    if !current.is_empty() { rows.push(current); }

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No network interfaces found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}

fn sys_packages() -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let output = std::process::Command::new("pacman")
        .args(["-Q"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output.lines()
        .take(100)
        .filter_map(|line| {
            let mut parts = line.splitn(2, ' ');
            let name = parts.next()?.to_string();
            let version = parts.next().unwrap_or("?").trim().to_string();
            let mut row = HashMap::new();
            row.insert("name".to_string(), Value::Text(name));
            row.insert("version".to_string(), Value::Text(version));
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}


fn sys_logs(args: &[&str]) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let follow = args.contains(&"--follow") || args.contains(&"-f");
    let errors_only = args.contains(&"--errors") || args.contains(&"-e");
    let lines = args.iter()
        .find(|a| a.starts_with("-n"))
        .and_then(|a| a.trim_start_matches("-n").parse::<usize>().ok())
        .unwrap_or(50);

    if follow {
        // Streaming mode — follow journalctl
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let r = running.clone();
        std::thread::spawn(move || {
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            r.store(false, std::sync::atomic::Ordering::SeqCst);
        });

        println!("  {} {} {}",
            "streaming logs".bright_cyan(),
            if errors_only { "(errors only)" } else { "" }.dimmed(),
            "— press Enter to stop".dimmed()
        );
        println!("{}", "━".repeat(52).dimmed());

        let mut cmd = std::process::Command::new("journalctl");
        cmd.args(["-f", "-n", "0", "--no-pager", "--output=short-iso"]);
        if errors_only {
            cmd.args(["-p", "err"]);
        }

        if let Ok(mut child) = cmd.stdout(std::process::Stdio::piped()).spawn() {
            use std::io::{BufRead, BufReader};
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if !running.load(std::sync::atomic::Ordering::SeqCst) { break; }
                    if let Ok(l) = line {
                        let level = if l.contains("err") || l.contains("ERROR") {
                            "ERR ".bright_red().to_string()
                        } else if l.contains("warn") || l.contains("WARN") {
                            "WARN".yellow().to_string()
                        } else {
                            "INFO".dimmed().to_string()
                        };
                        println!("  {} {}", level, l.dimmed());
                    }
                }
            }
            let _ = child.kill();
        }
        println!("
  {} log stream stopped", "○".dimmed());
        return CommandResult::Empty;
    }

    // Static mode — return as table
    let output = std::process::Command::new("journalctl")
        .args(["-n", &lines.to_string(), "--no-pager", "--output=short-iso"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output.lines()
        .filter(|l| !l.starts_with("--"))
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, ' ').collect();
            if parts.len() < 3 { return None; }
            let level = if line.contains("err") || line.contains("ERROR") { "error" }
                else if line.contains("warn") { "warn" }
                else { "info" }.to_string();
            if errors_only && level != "error" { return None; }
            let mut row = HashMap::new();
            row.insert("time".to_string(), Value::Text(parts[0].to_string()));
            row.insert("host".to_string(), Value::Text(parts[1].to_string()));
            row.insert("service".to_string(), Value::Text(parts[2].to_string()));
            row.insert("level".to_string(), Value::Text(level));
            row.insert("message".to_string(), Value::Text(
                parts.get(3).unwrap_or(&"").to_string()
            ));
            Some(row)
        })
        .collect();

    CommandResult::Value(Value::Table(rows))
}

fn search(db: &ForestDb, args: &[&str]) -> CommandResult {
    let query = args.join(" ").to_lowercase();

    let rows: Vec<(String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 200"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
        };
        stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };

    if query.is_empty() {
        // Show recent history
        let mut out = String::new();
        out.push_str(&format!("\n{}\n", "  ╭─ 📜 Command History ──────────────────────────────".bright_cyan()));
        for (cmd, ts) in rows.iter().take(15) {
            let time = fmt_time(*ts, "%H:%M");
            out.push_str(&format!("  │  {} {}\n", time.dimmed(), cmd.bright_white()));
        }
        out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
        return CommandResult::Output(out);
    }

    // Fuzzy search — score by match position and frequency
    let mut matches: Vec<(String, i64, usize)> = rows.iter()
        .filter(|(cmd, _)| cmd.to_lowercase().contains(&query))
        .map(|(cmd, ts)| {
            let score = if cmd.to_lowercase().starts_with(&query) { 0 }
                else if cmd.to_lowercase().contains(&format!(" {}", query)) { 1 }
                else { 2 };
            (cmd.clone(), *ts, score)
        })
        .collect();

    // Deduplicate keeping most recent
    matches.sort_by_key(|(cmd, _, score)| (*score, cmd.clone()));
    matches.dedup_by(|a, b| a.0 == b.0);
    matches.sort_by_key(|(_, ts, score)| (*score, -ts));

    if matches.is_empty() {
        return CommandResult::Output(format!(
            "  {} No matches for {}",
            "○".dimmed(), query.bright_white()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n",
        format!("  ╭─ 🔍 Search: {} ({} results) ──────────────────────", query, matches.len()).bright_cyan()
    ));
    for (cmd, ts, _) in matches.iter().take(10) {
        let time = fmt_time(*ts, "%H:%M");
        // Highlight the match
        let highlighted = cmd.replacen(&query, &query.bright_yellow().to_string(), 1);
        out.push_str(&format!("  │  {} {}\n", time.dimmed(), highlighted));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn cd(args: &[&str]) -> CommandResult {
    let target = args.first().copied().unwrap_or("~");
    let home = std::env::var("HOME").unwrap_or_default();
    let path = if target == "~" || target.is_empty() {
        std::path::PathBuf::from(&home)
    } else if target.starts_with("~/") {
        std::path::PathBuf::from(format!("{}/{}", home, &target[2..]))
    } else {
        std::path::PathBuf::from(target)
    };

    match std::env::set_current_dir(&path) {
        Ok(_) => CommandResult::Empty,
        Err(e) => CommandResult::Error(format!("cd: {}: {}", target, e)),
    }
}

fn help() -> CommandResult {
    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🌲 faelight-shell commands ──────────────────────────".bright_cyan()));
    let cmds = [
        ("health",    "system health and status"),
        ("events",    "recent events  [today|domain]"),
        ("decisions", "open decisions from ledger"),
        ("intents",   "active intents"),
        ("tools",     "tool deployment status"),
        ("audit",     "tool intelligence scores"),
        ("forecast",  "health trend and forecast"),
        ("sandbox",   "recent sandbox runs"),
        ("checkpoint", "recent checkpoints"),
        ("git",        "git status and recent commits"),
        ("search",     "search command history  [query]"),
        ("tt",         "tools as table — pipeable"),
        ("et",         "events as table — pipeable"),
        ("at",         "audit scores as table — pipeable"),
        ("dt",         "decisions as table — pipeable"),
        ("ht",         "shell history as table — pipeable"),
        ("ct",         "checkpoints as table — pipeable"),
        ("domains",    "event domain summary"),
        ("histogram",  "histogram of a domain  [field]"),
        ("logs",       "system logs — pipeable  [--follow] [--errors]"),
        ("ps",         "processes as table — pipeable"),
        ("ports",      "open ports as table — pipeable"),
        ("services",   "systemd services as table — pipeable"),
        ("files",      "files as table  [path]"),
        ("net",        "network interfaces as table"),
        ("pkgs",       "installed packages as table"),
        ("gc",         "git commits as table — pipeable"),
        ("gf",         "git files as table — pipeable"),
        ("watch",      "watch a command live  [health|events] — or pipe: ps | watch [interval]"),
        ("alias",      "manage aliases  [name=command]"),
        ("unalias",    "remove an alias"),
        ("plugins",    "list loaded plugins"),
        ("story",     "30-day forest narrative"),
        ("advise",    "judgment advisory"),
        ("version",   "system version"),
        ("commits",   "commit count and last commit"),
        ("cd",        "change directory"),
        ("clear",     "clear the screen"),
        ("exit",      "leave faelight-shell"),
    ];
    for (cmd, desc) in &cmds {
        out.push_str(&format!("  │  {:<12}  {}\n",
            cmd.bright_cyan(),
            desc.dimmed()
        ));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn health(db: &ForestDb) -> CommandResult {
    let health = db.health_score().unwrap_or(0);
    let status = if health >= 95 { "HEALTHY".bright_green() }
        else if health >= 80 { "ADVISORY".yellow() }
        else { "DEGRADED".bright_red() };

    let version = std::fs::read_to_string(
        std::path::PathBuf::from(db.core_root()).join("00-meta/VERSION")
    ).unwrap_or_else(|_| "unknown".into()).trim().to_string();

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🏥 Forest Health ─────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Health:  {}  {}\n", format!("{}%", health).bright_white().bold(), status));
    out.push_str(&format!("  │  Version: {}\n", version.dimmed()));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn events(db: &ForestDb, args: &[&str]) -> CommandResult {
    let today_only = args.contains(&"today");
    let domain = args.first().and_then(|a| {
        if *a == "today" { None } else { Some(*a) }
    });

    let label = if today_only { "Today's Events" } else { "Recent Events" };
    let events = db.query_events(domain, today_only, 20);
    if events.is_empty() {
        return CommandResult::Output(format!("  {} No events found", "○".dimmed()));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", format!("  ╭─ 📊 {} ─────────────────────────────────", label).bright_cyan()));
    for (domain, action, ts) in &events {
        let time = fmt_time(*ts, "%H:%M:%S");
        let icon = match domain.as_str() {
            "doctor"     => "🩺",
            "git"        => "🌿",
            "security"   => "🔒",
            "sandbox"    => "🧪",
            "audit"      => "🔍",
            "checkpoint" => "📸",
            "compositor" => "🖥",
            "idle"       => "💤",
            "decisions"  => "⚖️ ",
            "shell"      => "🐚",
            "update"     => "📦",
            _            => "○ ",
        };
        out.push_str(&format!("  │  {} {}  {}.{}\n",
            icon,
            time.dimmed(),
            domain.bright_cyan(),
            action.bright_white(),
        ));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn decisions(db: &ForestDb) -> CommandResult {
    let rows: Vec<(String, String, String)> = db.conn
        .prepare("SELECT dec_id, description, outcome FROM decisions ORDER BY timestamp DESC LIMIT 10")
        .ok()
        .map(|mut s| {
            s.query_map([], |r| {
                Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?))
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            .unwrap_or_default()
        })
        .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No decisions recorded yet — use {}",
            "○".dimmed(), "core decide".bright_cyan()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ ⚖️  Decisions ──────────────────────────────────────".bright_cyan()));
    for (id, desc, outcome) in &rows {
        let outcome_icon = match outcome.as_str() {
            "success" => "✅".to_string(),
            "failure" => "❌".to_string(),
            "partial" => "⚠️ ".to_string(),
            _ => "○ ".to_string(),
        };
        let short_desc = if desc.len() > 45 {
            format!("{}...", &desc[..45])
        } else {
            desc.clone()
        };
        out.push_str(&format!("  │  {}  {}  {}\n",
            id.bright_yellow(),
            outcome_icon,
            short_desc.dimmed()
        ));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn intents(core_root: &str) -> CommandResult {
    let future_dir = std::path::PathBuf::from(core_root).join("intents/future");
    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🎯 Active Intents ────────────────────────────────".bright_cyan()));

    if let Ok(entries) = std::fs::read_dir(&future_dir) {
        let mut found = false;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                let title = content.lines()
                    .find(|l| l.starts_with("title:"))
                    .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
                    .unwrap_or_else(|| name.clone());
                let id = name.split('-').next().unwrap_or("?");
                out.push_str(&format!("  │  {}  {}\n",
                    format!("INT-{}", id).bright_yellow(),
                    title.dimmed()
                ));
                found = true;
            }
        }
        if !found {
            out.push_str("  │  No active intents\n");
        }
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn tools(_db: &ForestDb, core_root: &str) -> CommandResult {
    let tools_dir = std::path::PathBuf::from(core_root).join("rust-tools");
    let total = std::fs::read_dir(&tools_dir)
        .map(|e| e.flatten().filter(|e| e.path().join("Cargo.toml").exists()).count())
        .unwrap_or(0);

    let deployed = std::fs::read_dir(std::path::PathBuf::from(core_root).join("scripts"))
        .map(|e| e.flatten().count())
        .unwrap_or(0);

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🛠  Tools ─────────────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Total:    {} tools\n", total.to_string().bright_white().bold()));
    out.push_str(&format!("  │  Deployed: {}/{}\n", deployed.to_string().bright_green(), total));
    out.push_str(&format!("  │  Run {} for intelligence scores\n", "audit".bright_cyan()));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn version(core_root: &str) -> CommandResult {
    let version = std::fs::read_to_string(
        std::path::PathBuf::from(core_root).join("00-meta/VERSION")
    ).unwrap_or_else(|_| "unknown".into());

    let changelog = std::fs::read_to_string(
        std::path::PathBuf::from(core_root).join("00-meta/CHANGELOG.md")
    ).unwrap_or_default();

    let release_name = changelog.lines()
        .find(|l| l.starts_with("## ["))
        .and_then(|l| l.split('—').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "The Forest Remembers".to_string());

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🌲 Version ───────────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  {}  {}\n",
        version.trim().bright_white().bold(),
        release_name.dimmed()
    ));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn commits(core_root: &str) -> CommandResult {
    let count = std::process::Command::new("git")
        .args(["-C", core_root, "rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "?".to_string());

    let last = std::process::Command::new("git")
        .args(["-C", core_root, "log", "-1", "--format=%s"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 📚 Commits ───────────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Total:  {}\n", count.bright_white().bold()));
    out.push_str(&format!("  │  Last:   {}\n", last.dimmed()));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn story(db: &ForestDb) -> CommandResult {
    // Delegate to core story via process
    let core_root = db.core_root();
    let output = std::process::Command::new(
        format!("{}/scripts/core", core_root)
    )
    .args(["story"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .unwrap_or_else(|| "core story not available".to_string());

    CommandResult::Output(output)
}

fn advise(db: &ForestDb) -> CommandResult {
    let core_root = db.core_root();
    let output = std::process::Command::new(
        format!("{}/scripts/core", core_root)
    )
    .args(["advise"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .unwrap_or_else(|| "core advise not available".to_string());

    CommandResult::Output(output)
}

fn audit(_db: &ForestDb, core_root: &str) -> CommandResult {
    let output = std::process::Command::new(
        format!("{}/scripts/core", core_root)
    )
    .args(["audit", "scan"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .unwrap_or_else(|| "core audit not available".to_string());

    CommandResult::Output(output)
}

// ── Schema — Phase 11a ────────────────────────────────────────────────────────

fn schema(args: &[&str]) -> CommandResult {
    let registry = crate::schema::SchemaRegistry::build();
    match args.first() {
        None | Some(&"") => {
            CommandResult::Output(crate::schema::render_registry(&registry))
        }
        Some(table_name) => {
            match registry.get(table_name) {
                Some(schema) => CommandResult::Output(
                    crate::schema::render_table_schema(schema)
                ),
                None => {
                    let known = registry.names().join(", ");
                    CommandResult::Error(format!(
                        "unknown table '{}' — known: {}",
                        table_name, known
                    ))
                }
            }
        }
    }
}

// ── Phase 14 — File System Index ─────────────────────────────────────────────
// Persistent index in state.db for fast recursive file queries.
// find           — query from index (or rebuild if empty)
// find reindex   — rebuild index from core_root
// find <path>    — query files under a specific path

fn find_cmd(db: &ForestDb, core_root: &str, args: &[&str]) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    // Ensure schema
    db.conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS file_index (
            path      TEXT PRIMARY KEY,
            name      TEXT NOT NULL,
            kind      TEXT NOT NULL,
            size      INTEGER NOT NULL,
            extension TEXT,
            modified  INTEGER NOT NULL
        );"
    ).ok();

    // reindex — rebuild from core_root recursively
    if args.first() == Some(&"reindex") {
        let count = index_directory(&db.conn, core_root, 0);
        return CommandResult::Output(format!(
            "  {} Indexed {} files under {}",
            "✅".green(), count.to_string().bright_green(), core_root.dimmed()
        ));
    }

    // Check if index is empty — auto-reindex on first use
    let count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM file_index", [], |r| r.get(0)
    ).unwrap_or(0);

    if count == 0 {
        println!("  {} Building file index for first time...", "⟳".bright_cyan());
        index_directory(&db.conn, core_root, 0);
    }

    // Filter by path prefix if arg given
    let path_filter = args.first().copied().unwrap_or("");
    let query = if path_filter.is_empty() {
        "SELECT path, name, kind, size, extension, modified FROM file_index ORDER BY size DESC LIMIT 500".to_string()
    } else {
        format!("SELECT path, name, kind, size, extension, modified FROM file_index WHERE path LIKE '{}%' ORDER BY size DESC LIMIT 500", path_filter)
    };

    let mut stmt = match db.conn.prepare(&query) {
        Ok(s) => s,
        Err(e) => return CommandResult::Error(e.to_string()),
    };

    let rows: Vec<HashMap<String, Value>> = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, i64>(5)?,
        ))
    }).ok()
    .map(|rows| rows.filter_map(|r| r.ok()).map(|(path, name, kind, size, ext, modified)| {
        let time_str = chrono::DateTime::from_timestamp(modified, 0)
            .map(|t| t.format("%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "?".to_string());
        let mut row = HashMap::new();
        row.insert("name".to_string(),      Value::Text(name));
        row.insert("path".to_string(),      Value::Text(path));
        row.insert("kind".to_string(),      Value::Text(kind));
        row.insert("size".to_string(),      Value::Int(size));
        row.insert("ext".to_string(),       Value::Text(ext.unwrap_or_default()));
        row.insert("modified".to_string(),  Value::Text(time_str));
        row
    }).collect())
    .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No files found. Run: find reindex", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}

fn index_directory(conn: &rusqlite::Connection, dir: &str, depth: usize) -> usize {
    if depth > 6 { return 0; } // max depth
    let mut count = 0;
    let skip_dirs = [".git", "target", "node_modules", ".cargo", "proc", "sys", "dev"];
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && depth > 0 { continue; }
            let skip = skip_dirs.iter().any(|s| name == *s);
            if skip { continue; }
            if let Ok(meta) = entry.metadata() {
                let kind = if meta.is_dir() { "dir" } else { "file" };
                let size = if meta.is_file() { meta.len() as i64 } else { 0 };
                let ext = path.extension().map(|e| e.to_string_lossy().to_string());
                let modified = meta.modified().ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                conn.execute(
                    "INSERT OR REPLACE INTO file_index (path, name, kind, size, extension, modified)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![path_str, name, kind, size, ext, modified]
                ).ok();
                count += 1;
                if meta.is_dir() {
                    count += index_directory(conn, &path_str, depth + 1);
                }
            }
        }
    }
    count
}

// ── Phase 15 — Git Data Engine ────────────────────────────────────────────────

/// git-churn — files changed most frequently across commit history
/// gchurn | sort changes desc | first 10  → hottest files
fn git_churn(core_root: &str, args: &[&str]) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let limit = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(100);

    let output = std::process::Command::new("git")
        .args(["-C", core_root, "log",
            &format!("--max-count={}", limit * 10),
            "--name-only", "--format="])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    // Count occurrences of each file
    let mut counts: HashMap<String, usize> = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        *counts.entry(line.to_string()).or_insert(0) += 1;
    }

    let mut rows: Vec<HashMap<String, Value>> = counts.into_iter()
        .map(|(file, count)| {
            let ext = std::path::Path::new(&file)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();
            let mut row = HashMap::new();
            row.insert("file".to_string(),    Value::Text(file));
            row.insert("changes".to_string(), Value::Int(count as i64));
            row.insert("ext".to_string(),     Value::Text(ext));
            row
        })
        .collect();

    // Sort by changes desc by default
    rows.sort_by(|a, b| {
        let ac = a.get("changes").and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None }).unwrap_or(0);
        let bc = b.get("changes").and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None }).unwrap_or(0);
        bc.cmp(&ac)
    });
    rows.truncate(limit);

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No git history found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}

/// git-branches — all branches as a structured table
fn git_branches(core_root: &str) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let output = std::process::Command::new("git")
        .args(["-C", core_root, "branch", "-a", "--format=%(refname:short)|%(objectname:short)|%(committerdate:relative)|%(HEAD)"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, Value>> = output.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() < 4 { return None; }
            let name = parts[0].trim().to_string();
            let hash = parts[1].trim().to_string();
            let date = parts[2].trim().to_string();
            let current = parts[3].trim() == "*";
            let remote = name.starts_with("remotes/");
            let mut row = HashMap::new();
            row.insert("name".to_string(),    Value::Text(name));
            row.insert("hash".to_string(),    Value::Text(hash));
            row.insert("date".to_string(),    Value::Text(date));
            row.insert("current".to_string(), Value::Bool(current));
            row.insert("remote".to_string(),  Value::Bool(remote));
            Some(row)
        })
        .collect();

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No branches found", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}

// ── Phase 16 — History Analytics ─────────────────────────────────────────────

/// hstats — most used commands ranked by frequency
fn history_stats(db: &ForestDb) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let mut stmt = match db.conn.prepare(
        "SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 1000"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
    };

    let commands: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    // Count first word of each command
    let mut counts: HashMap<String, usize> = HashMap::new();
    for cmd in &commands {
        let first = cmd.split_whitespace().next().unwrap_or("").to_string();
        if !first.is_empty() {
            *counts.entry(first).or_insert(0) += 1;
        }
    }

    let total = commands.len();
    let mut rows: Vec<HashMap<String, Value>> = counts.into_iter()
        .map(|(cmd, count)| {
            let pct = format!("{:.0}%", (count as f64 / total as f64) * 100.0);
            let mut row = HashMap::new();
            row.insert("command".to_string(), Value::Text(cmd));
            row.insert("count".to_string(),   Value::Int(count as i64));
            row.insert("pct".to_string(),     Value::Text(pct));
            row
        })
        .collect();

    rows.sort_by(|a, b| {
        let ac = a.get("count").and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None }).unwrap_or(0);
        let bc = b.get("count").and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None }).unwrap_or(0);
        bc.cmp(&ac)
    });
    rows.truncate(20);

    CommandResult::Value(Value::Table(rows))
}

/// hpattern — command frequency by hour of day
fn history_pattern(db: &ForestDb) -> CommandResult {
    use std::collections::HashMap;
    use crate::value::Value;

    let mut stmt = match db.conn.prepare(
        "SELECT timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 2000"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
    };

    let timestamps: Vec<i64> = stmt
        .query_map([], |r| r.get::<_, i64>(0))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    // Count by hour
    let mut by_hour: HashMap<u32, usize> = HashMap::new();
    use chrono::Timelike;
    for ts in &timestamps {
        if let Some(dt) = chrono::DateTime::from_timestamp(*ts, 0) {
            let hour = dt.hour();
            *by_hour.entry(hour).or_insert(0) += 1;
        }
    }

    let max_count = by_hour.values().copied().max().unwrap_or(1);

    let mut rows: Vec<HashMap<String, Value>> = (0u32..24).map(|hour| {
        let count = by_hour.get(&hour).copied().unwrap_or(0);
        let bar_len = (count * 20) / max_count.max(1);
        let bar = "█".repeat(bar_len);
        let label = format!("{:02}:00", hour);
        let period = match hour {
            5..=8   => "morning",
            9..=11  => "late morning",
            12..=13 => "midday",
            14..=17 => "afternoon",
            18..=21 => "evening",
            22..=23 => "night",
            _       => "late night",
        };
        let mut row = HashMap::new();
        row.insert("hour".to_string(),   Value::Text(label));
        row.insert("period".to_string(), Value::Text(period.to_string()));
        row.insert("count".to_string(),  Value::Int(count as i64));
        row.insert("bar".to_string(),    Value::Text(bar));
        row
    }).collect();

    // Only show hours with activity
    rows.retain(|r| {
        if let Some(Value::Int(c)) = r.get("count") { *c > 0 } else { false }
    });

    CommandResult::Value(Value::Table(rows))
}

// ── Phase 17 — Event System ───────────────────────────────────────────────────

fn on_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    crate::triggers::ensure_schema(db);
    // Rejoin and re-split — execute() uses splitn(3) which embeds args
    let rejoined = args.join(" ");
    let reargs: Vec<&str> = rejoined.split_whitespace().collect();
    let args = reargs.as_slice();
    match args {
        // on list
        [] | ["list"] => {
            crate::triggers::render_list(db);
            CommandResult::Empty
        }
        // on remove <id>
        ["remove", id] => {
            if let Ok(n) = id.parse::<i64>() {
                if crate::triggers::remove(db, n) {
                    CommandResult::Output(format!("  {} Trigger #{} removed.", "✅".green(), n))
                } else {
                    CommandResult::Output(format!("  {} Trigger #{} not found.", "✗".bright_red(), n))
                }
            } else {
                CommandResult::Error("Usage: on remove <id>".to_string())
            }
        }
        // on enable/disable <id>
        ["enable", id] => {
            if let Ok(n) = id.parse::<i64>() {
                crate::triggers::enable(db, n, true);
                CommandResult::Output(format!("  {} Trigger #{} enabled.", "✅".green(), n))
            } else {
                CommandResult::Error("Usage: on enable <id>".to_string())
            }
        }
        ["disable", id] => {
            if let Ok(n) = id.parse::<i64>() {
                crate::triggers::enable(db, n, false);
                CommandResult::Output(format!("  {} Trigger #{} disabled.", "○".dimmed(), n))
            } else {
                CommandResult::Error("Usage: on disable <id>".to_string())
            }
        }
        // on <trigger...> => <action...>
        args => {
            // Find => separator
            if let Some(sep) = args.iter().position(|a| *a == "=>") {
                let trigger = args[..sep].join(" ");
                let action = args[sep+1..].join(" ");
                if trigger.is_empty() || action.is_empty() {
                    return CommandResult::Error(
                        "Usage: on <trigger> => <action>  e.g. on health_drop 90 => notify \"health low\"".to_string()
                    );
                }
                match crate::triggers::add(db, &trigger, &action) {
                    Ok(_) => CommandResult::Output(format!(
                        "  {} Trigger added: {} → {}",
                        "✅".green(),
                        trigger.bright_cyan(),
                        action.bright_white()
                    )),
                    Err(e) => CommandResult::Error(e),
                }
            } else {
                CommandResult::Error(
                    "Usage: on <trigger> => <action>\nTriggers: health_drop <n>, git_commit, event <domain>, run <cmd>".to_string()
                )
            }
        }
    }
}

// ── Phase 18 — Time Travel ────────────────────────────────────────────────────
// snapshot  — capture current system state
// timeline  — show snapshots over time
// snap-diff — compare two snapshots

fn ensure_snapshots_schema(db: &ForestDb) {
    db.conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS shell_snapshots (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            name      TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            health    INTEGER,
            commits   INTEGER,
            processes INTEGER,
            load_avg  TEXT,
            top_proc  TEXT,
            note      TEXT
        );"
    ).ok();
}

fn snapshot_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    use std::time::{SystemTime, UNIX_EPOCH};

    ensure_snapshots_schema(db);

    let name = args.first().copied().unwrap_or("manual");
    let note = if args.len() > 1 { args[1..].join(" ") } else { String::new() };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

    // Capture health
    let health = db.health_score().unwrap_or(0);

    // Capture commit count
    let commits: i64 = std::process::Command::new("git")
        .args(["-C", &db.core_root(), "rev-list", "--count", "HEAD"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    // Capture process count
    let processes: i64 = std::process::Command::new("ps")
        .args(["aux", "--no-headers"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().count() as i64)
        .unwrap_or(0);

    // Load average
    let load_avg = std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".to_string());

    // Top CPU process
    let top_proc = std::process::Command::new("ps")
        .args(["aux", "--no-headers", "--sort=-pcpu"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() > 10 {
                format!("{} ({}%)", parts[10..].join(" ").chars().take(20).collect::<String>(), parts[2])
            } else { "?".to_string() }
        }))
        .unwrap_or_else(|| "?".to_string());

    db.conn.execute(
        "INSERT INTO shell_snapshots (name, timestamp, health, commits, processes, load_avg, top_proc, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![name, now, health, commits, processes, load_avg, top_proc, note]
    ).ok();

    CommandResult::Output(format!(
        "  {} Snapshot '{}' captured — health: {}%  commits: {}  procs: {}  load: {}",
        "📸".normal(),
        name.bright_white().bold(),
        health.to_string().bright_green(),
        commits.to_string().bright_white(),
        processes.to_string().dimmed(),
        load_avg.dimmed()
    ))
}

fn timeline_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    use crate::value::Value;
    use std::collections::HashMap;

    ensure_snapshots_schema(db);

    let limit = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(10);

    let mut stmt = match db.conn.prepare(
        "SELECT id, name, timestamp, health, commits, processes, load_avg FROM shell_snapshots ORDER BY timestamp DESC LIMIT ?1"
    ) {
        Ok(s) => s,
        Err(_) => return CommandResult::Output(format!("  {} No snapshots yet. Run: snapshot", "○".dimmed())),
    };

    let rows: Vec<HashMap<String, Value>> = stmt.query_map(
        rusqlite::params![limit as i64], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, String>(6)?,
        ))
    }).ok()
    .map(|rows| rows.filter_map(|r| r.ok()).map(|(id, name, ts, health, commits, procs, load)| {
        let time = fmt_time(ts, "%m-%d %H:%M");
        let mut row = HashMap::new();
        row.insert("id".to_string(),       Value::Int(id));
        row.insert("name".to_string(),     Value::Text(name));
        row.insert("time".to_string(),     Value::Text(time));
        row.insert("health".to_string(),   Value::Int(health));
        row.insert("commits".to_string(),  Value::Int(commits));
        row.insert("procs".to_string(),    Value::Int(procs));
        row.insert("load".to_string(),     Value::Text(load));
        row
    }).collect())
    .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No snapshots yet. Run: snapshot", "○".dimmed()));
    }
    CommandResult::Value(Value::Table(rows))
}

fn snap_diff_cmd(db: &ForestDb, args: &[&str]) -> CommandResult {
    ensure_snapshots_schema(db);

    let (id1, id2) = match args {
        [a, b] => {
            match (a.parse::<i64>(), b.parse::<i64>()) {
                (Ok(x), Ok(y)) => (x, y),
                _ => return CommandResult::Error("Usage: snap-diff <id1> <id2>".to_string()),
            }
        }
        _ => {
            // Default: diff last two snapshots
            let ids: Vec<i64> = db.conn.prepare(
                "SELECT id FROM shell_snapshots ORDER BY timestamp DESC LIMIT 2"
            ).ok()
            .and_then(|mut s| s.query_map([], |r| r.get::<_, i64>(0)).ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect()))
            .unwrap_or_default();
            if ids.len() < 2 {
                return CommandResult::Output(format!(
                    "  {} Need at least 2 snapshots. Run: snapshot", "○".dimmed()
                ));
            }
            (ids[1], ids[0]) // older first
        }
    };

    // Fetch both snapshots
    let fetch = |id: i64| -> Option<(String, i64, i64, i64, String)> {
        db.conn.query_row(
            "SELECT name, health, commits, processes, load_avg FROM shell_snapshots WHERE id = ?1",
            rusqlite::params![id], |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
            ))
        ).ok()
    };

    let s1 = match fetch(id1) {
        Some(s) => s,
        None => return CommandResult::Error(format!("Snapshot #{} not found", id1)),
    };
    let s2 = match fetch(id2) {
        Some(s) => s,
        None => return CommandResult::Error(format!("Snapshot #{} not found", id2)),
    };

    println!();
    println!("{}", "🔍  Snapshot Diff".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!("  {} #{} '{}'  →  #{} '{}'",
        "Comparing:".dimmed(),
        id1.to_string().bright_white(), s1.0.bright_white(),
        id2.to_string().bright_white(), s2.0.bright_white()
    );
    println!();

    // Health diff
    let health_diff = s2.1 - s1.1;
    let health_str = if health_diff > 0 {
        format!("+{}", health_diff).bright_green().to_string()
    } else if health_diff < 0 {
        format!("{}", health_diff).bright_red().to_string()
    } else { "unchanged".dimmed().to_string() };
    println!("  {}  {} → {}  ({})",
        "Health:".dimmed(), s1.1.to_string().bright_white(),
        s2.1.to_string().bright_white(), health_str
    );

    // Commits diff
    let commit_diff = s2.2 - s1.2;
    println!("  {}  {} → {}  ({} new commit{})",
        "Commits:".dimmed(), s1.2.to_string().bright_white(),
        s2.2.to_string().bright_white(),
        commit_diff.to_string().bright_green(),
        if commit_diff == 1 { "" } else { "s" }
    );

    // Process diff
    let proc_diff = s2.3 - s1.3;
    let proc_str = if proc_diff > 0 {
        format!("+{}", proc_diff).yellow().to_string()
    } else if proc_diff < 0 {
        format!("{}", proc_diff).bright_green().to_string()
    } else { "unchanged".dimmed().to_string() };
    println!("  {}  {} → {}  ({})",
        "Processes:".dimmed(), s1.3.to_string().bright_white(),
        s2.3.to_string().bright_white(), proc_str
    );

    // Load diff
    println!("  {}  {} → {}",
        "Load avg:".dimmed(),
        s1.4.dimmed(), s2.4.bright_white()
    );

    println!();
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}", "Diff complete. The forest remembers every state.".dimmed().italic());
    println!();

    CommandResult::Empty
}

// ── Phase 21 — Query Language ─────────────────────────────────────────────────
// SQL-like syntax → existing pipeline ops.
// select <cols> from <table> [where <field> <op> <val>] [order by <col>] [limit <n>]
// Adoption bridge — familiar syntax for structured queries.

fn sql_query_cmd(db: &ForestDb, core_root: &str, line: &str) -> CommandResult {
    match parse_sql_query(line) {
        Ok(q) => {
            // Build equivalent pipeline and execute
            let pipeline = q.to_pipeline();
            println!("  {} {}", "→".dimmed(), pipeline.dimmed());
            execute(&pipeline, db, core_root)
        }
        Err(e) => CommandResult::Error(format!("Query error: {}", e)),
    }
}

#[derive(Debug)]
struct SqlQuery {
    columns:  Vec<String>,   // * or specific columns
    table:    String,        // from <table>
    where_:   Option<String>, // where clause
    order_by: Option<String>, // order by <col>
    order_desc: bool,
    limit:    Option<usize>,
}

impl SqlQuery {
    fn to_pipeline(&self) -> String {
        let mut parts = vec![self.table.clone()];

        if let Some(ref w) = self.where_ {
            parts.push(format!("where {}", w));
        }
        if let Some(ref col) = self.order_by {
            if self.order_desc {
                parts.push(format!("sort {} desc", col));
            } else {
                parts.push(format!("sort {}", col));
            }
        }
        if let Some(n) = self.limit {
            parts.push(format!("first {}", n));
        }
        if !self.columns.is_empty() && self.columns != ["*"] {
            parts.push(format!("select {}", self.columns.join(" ")));
        }
        parts.join(" | ")
    }
}

fn parse_sql_query(line: &str) -> Result<SqlQuery, String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() || tokens[0].to_lowercase() != "select" {
        return Err("Expected SELECT".to_string());
    }

    // Find FROM position
    let from_pos = tokens.iter().position(|t| t.to_lowercase() == "from")
        .ok_or("Expected FROM")?;

    // Columns between SELECT and FROM
    let columns: Vec<String> = tokens[1..from_pos]
        .iter()
        .map(|s| s.trim_end_matches(',').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if from_pos + 1 >= tokens.len() {
        return Err("Expected table name after FROM".to_string());
    }
    let table = tokens[from_pos + 1].to_lowercase();

    let remaining = &tokens[from_pos + 2..];

    // Parse WHERE, ORDER BY, LIMIT
    let mut where_ = None;
    let mut order_by = None;
    let mut order_desc = false;
    let mut limit = None;

    let mut i = 0;
    while i < remaining.len() {
        match remaining[i].to_lowercase().as_str() {
            "where" => {
                // Collect until ORDER or LIMIT
                let mut clause = vec![];
                i += 1;
                while i < remaining.len() {
                    let t = remaining[i].to_lowercase();
                    if t == "order" || t == "limit" { break; }
                    clause.push(remaining[i]);
                    i += 1;
                }
                where_ = Some(clause.join(" "));
            }
            "order" => {
                i += 1;
                if i < remaining.len() && remaining[i].to_lowercase() == "by" {
                    i += 1;
                }
                if i < remaining.len() {
                    order_by = Some(remaining[i].to_string());
                    i += 1;
                    if i < remaining.len() && remaining[i].to_lowercase() == "desc" {
                        order_desc = true;
                        i += 1;
                    }
                }
            }
            "limit" => {
                i += 1;
                if i < remaining.len() {
                    limit = remaining[i].parse::<usize>().ok();
                    i += 1;
                }
            }
            _ => { i += 1; }
        }
    }

    Ok(SqlQuery { columns, table, where_, order_by, order_desc, limit })
}
