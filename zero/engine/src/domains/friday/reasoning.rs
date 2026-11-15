//! INT-251 v23 Pillar 2 -- Reasoning Engine
//! Rule-based, no LLM. Watches event stream, produces system-wide observations.

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::collections::BTreeSet;

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// A reasoning rule: watches the event stream and produces observations
#[allow(dead_code)]
pub struct Rule {
    pub name: &'static str,
    pub description: &'static str,
    pub check: fn(&AppContext) -> Option<Observation>,
}

/// An observation produced by the reasoning engine
pub struct Observation {
    pub conclusion: String,
    pub confidence: f64,
    pub kind: ObservationKind,
}

pub enum ObservationKind {
    Normal,
    Anomaly,
    #[allow(dead_code)]
    Causal,
    SystemWide,
}

/// The starter rule set -- 8 rules covering key system patterns
pub fn starter_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "deploy_without_commit",
            description: "Deploy happened but no commit in last 5 min -- dirty deploy risk",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let deploys: i64 = db
                    .query_row(
                        "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
                        rusqlite::params![now - 300],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                let commits = crate::domains::friday_arch::room::git_commits_since(
                    std::path::Path::new(&ctx.core_root),
                    now - 300,
                );
                if deploys > 0 && commits == Some(0) {
                    Some(Observation {
                        conclusion: format!("{} deploy(s) in last 5min without a commit -- deploying uncommitted changes", deploys),
                        confidence: 0.9,
                        kind: ObservationKind::Anomaly,
                    })
                } else {
                    None
                }
            },
        },
        Rule {
            name: "high_deploy_frequency",
            description: "More than 10 deploys in an hour -- possible thrashing",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let deploys: i64 = db
                    .query_row(
                        "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
                        rusqlite::params![now - 3600],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if deploys > 10 {
                    Some(Observation {
                        conclusion: format!(
                            "{} deploys in the last hour -- consider batching changes",
                            deploys
                        ),
                        confidence: 0.8,
                        kind: ObservationKind::SystemWide,
                    })
                } else {
                    None
                }
            },
        },
        Rule {
            name: "no_intent_activity",
            description: "No cistart/cicomplete in 2+ hours during active session",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let lifecycle: i64 = db.query_row(
                    "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1 AND (command LIKE 'cistart%' OR command LIKE 'cicomplete%')",
                    rusqlite::params![now - 7200],
                    |r| r.get(0),
                ).unwrap_or(0);
                let recent_cmds: i64 = db
                    .query_row(
                        "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",
                        rusqlite::params![now - 7200],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if recent_cmds > 50 && lifecycle == 0 {
                    Some(Observation {
                        conclusion: "Active session (50+ commands) with no intent lifecycle activity in 2h -- work may be undocumented".to_string(),
                        confidence: 0.75,
                        kind: ObservationKind::Anomaly,
                    })
                } else {
                    None
                }
            },
        },
        Rule {
            name: "shell_health_signal",
            description: "Shell health events in the event stream",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let events: i64 = db
                    .query_row(
                        "SELECT COUNT(*) FROM events WHERE domain='shell' AND timestamp > ?1",
                        rusqlite::params![now - 86400],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if events > 0 {
                    Some(Observation {
                        conclusion: format!(
                            "nsh emitting {} events today -- shell intelligence active",
                            events
                        ),
                        confidence: 0.9,
                        kind: ObservationKind::Normal,
                    })
                } else {
                    None
                }
            },
        },
        Rule {
            name: "commits_without_deploy",
            description:
                "Commits in last 2h with no deploy -- changes may be uncommitted to runtime",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let two_hours_ago = now - 7200;
                let commits = crate::domains::friday_arch::room::git_commits_since(
                    std::path::Path::new(&ctx.core_root),
                    two_hours_ago,
                );
                let deploys: i64 = db
                    .query_row(
                        "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
                        rusqlite::params![two_hours_ago],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if let Some(commits) = commits.filter(|n| *n >= 3) {
                    if deploys != 0 {
                        return None;
                    }
                    Some(Observation {
                        conclusion: format!("{} commit(s) in last 2h with no deploy -- consider deploying to validate changes", commits),
                        confidence: 0.8,
                        kind: ObservationKind::Causal,
                    })
                } else {
                    None
                }
            },
        },
        Rule {
            name: "rapid_fix_cycle",
            description: "Detect rapid deploy cycles suggesting a tool change caused a regression",
            check: |ctx: &AppContext| {
                use std::collections::HashMap;
                use std::time::{SystemTime, UNIX_EPOCH};
                let db = &ctx.runtime.db;
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let one_hour_ago = now - 3600;
                let mut s = match db.prepare(
                "SELECT source_tool, timestamp FROM events WHERE action = 'deploy_completed' AND timestamp > ?1 ORDER BY source_tool, timestamp"
            ) {
                Ok(s) => s,
                Err(_) => return None,
            };
                let mapped =
                    match s.query_map(rusqlite::params![one_hour_ago], |r: &rusqlite::Row<'_>| {
                        Ok((
                            r.get::<_, String>(0).unwrap_or_default(),
                            r.get::<_, i64>(1).unwrap_or(0),
                        ))
                    }) {
                        Ok(m) => m,
                        Err(_) => return None,
                    };
                let rows: Vec<(String, i64)> = mapped.filter_map(|r| r.ok()).collect();
                let mut by_tool: HashMap<String, Vec<i64>> = HashMap::new();
                for (tool, ts) in rows {
                    by_tool.entry(tool).or_default().push(ts);
                }
                for (tool, timestamps) in &by_tool {
                    if timestamps.len() >= 3 {
                        let mut ts_sorted = timestamps.clone();
                        ts_sorted.sort();
                        for window in ts_sorted.windows(3) {
                            let span = window[2] - window[0];
                            if span <= 600 {
                                let minutes = (span / 60) + 1;
                                return Some(Observation {
                                conclusion: format!(
                                    "Rapid deploy cycle for {}: 3+ deploys in {}min -- tool change likely triggered a regression. Review last major change before the cycle.",
                                    tool, minutes
                                ),
                                confidence: 0.80,
                                kind: ObservationKind::Causal,
                            });
                            }
                        }
                    }
                }
                None
            },
        },
        Rule {
            name: "tool_retirement_regression",
            description:
                "Detect when a health check failed shortly after a deploy -- stale binary reference",
            check: |ctx: &AppContext| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let db = &ctx.runtime.db;
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let day_ago = now - 86400;
                let failed_count: i64 = db.query_row(
                "SELECT COUNT(*) FROM events WHERE action = 'health_check_failed' AND timestamp > ?1",
                rusqlite::params![day_ago],
                |r: &rusqlite::Row<'_>| r.get(0),
            ).unwrap_or(0);
                if failed_count == 0 {
                    return None;
                }
                let result: rusqlite::Result<(String, String)> = db.query_row(
                    "SELECT e1.payload, e2.source_tool FROM events e1
                 JOIN events e2 ON e2.action = 'deploy_completed'
                 AND e2.timestamp < e1.timestamp AND e2.timestamp > e1.timestamp - 120
                 WHERE e1.action = 'health_check_failed'
                 ORDER BY e1.timestamp DESC LIMIT 1",
                    [],
                    |r: &rusqlite::Row<'_>| {
                        Ok((
                            r.get::<_, String>(0).unwrap_or_default(),
                            r.get::<_, String>(1).unwrap_or_default(),
                        ))
                    },
                );
                match result {
                Ok((check_msg, tool)) => Some(Observation {
                    conclusion: format!(
                        "Health check failed within 2min of {} deploy -- stale binary reference or missing tool dependency. Check: {}",
                        tool, check_msg
                    ),
                    confidence: 0.85,
                    kind: ObservationKind::Causal,
                }),
                Err(_) => None,
            }
            },
        },
        Rule {
            name: "probe_after_pacman_syu",
            description: "A probe went from pass to warn after a pacman -Syu",
            check: |ctx| {
                cite_probe_change(&ctx.runtime.db, std::path::Path::new(PACMAN_LOG)).map(
                    |conclusion| Observation {
                        conclusion,
                        confidence: 0.85,
                        kind: ObservationKind::Causal,
                    },
                )
            },
        },
    ]
}

/// Run the reasoning engine -- produces observations from event stream
pub fn reason(ctx: &AppContext) -> CoreResult<Vec<(String, f64, String)>> {
    let rules = starter_rules();
    let mut observations = Vec::new();

    for rule in &rules {
        if let Some(obs) = (rule.check)(ctx) {
            let kind_str = match obs.kind {
                ObservationKind::Normal => "normal",
                ObservationKind::Anomaly => "anomaly",
                ObservationKind::Causal => "causal",
                ObservationKind::SystemWide => "system",
            };
            observations.push((obs.conclusion, obs.confidence, kind_str.to_string()));
        }
    }
    Ok(observations)
}

/// core friday reason -- run the reasoning engine
pub fn show(ctx: &AppContext) -> CoreResult<()> {
    println!();
    println!("  Friday -- Reasoning Engine");
    println!("  {}", "─".repeat(50).dimmed());
    println!();

    let observations = reason(ctx)?;

    if observations.is_empty() {
        println!(
            "  {} No active observations -- system within normal parameters",
            "·".dimmed()
        );
    } else {
        println!("  {} Observations ({}):", "🧠".normal(), observations.len());
        println!();
        for (conclusion, confidence, kind) in &observations {
            let kind_color = match kind.as_str() {
                "anomaly" => kind.bright_red(),
                "system" => kind.bright_cyan(),
                "causal" => kind.bright_yellow(),
                _ => kind.dimmed(),
            };
            println!(
                "  {} [{}] {}",
                "→".bright_cyan(),
                kind_color,
                conclusion.bright_white()
            );
            println!(
                "    {} confidence: {:.0}%",
                "·".dimmed(),
                confidence * 100.0
            );
            println!();
        }
    }
    Ok(())
}

/// Read-only pacman log. Missing or unreadable is no citation, never an error.
const PACMAN_LOG: &str = "/var/log/pacman.log";

// INT-316 G4. Previous and current probe status, from source='probe' rows in
// friday_observations or friday_knowledge. When one went pass to warn and the
// log records a `pacman -Syu`, cite it. The path is injectable. No root.
fn cite_probe_change(db: &rusqlite::Connection, pacman_log: &std::path::Path) -> Option<String> {
    let syu = latest_syu(pacman_log)?;
    let samples = probe_samples(db);
    pass_to_warn(&samples, syu)
}

struct ProbeSample {
    ts: i64,
    name: String,
    status: String,
}

fn latest_syu(path: &std::path::Path) -> Option<i64> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut best: Option<i64> = None;
    let mut saw = false;
    for line in text.lines() {
        if !line.contains("pacman -Syu") {
            continue;
        }
        saw = true;
        if let Some(ts) = pacman_line_ts(line) {
            best = Some(best.map_or(ts, |b| b.max(ts)));
        }
    }
    if best.is_some() {
        best
    } else if saw {
        Some(0)
    } else {
        None
    }
}

fn pacman_line_ts(line: &str) -> Option<i64> {
    let rest = line.split_once('[')?.1;
    let stamp = rest.split_once(']')?.0;
    let (date, time) = stamp.split_once('T')?;
    let mut date_parts = date.split('-');
    let year: i32 = date_parts.next()?.parse().ok()?;
    let month: u32 = date_parts.next()?.parse().ok()?;
    let day: u32 = date_parts.next()?.parse().ok()?;
    if time.len() < 8 {
        return None;
    }
    let mut clock = time[..8].split(':');
    let hour: u32 = clock.next()?.parse().ok()?;
    let min: u32 = clock.next()?.parse().ok()?;
    let sec: u32 = clock.next()?.parse().ok()?;
    let offset = tz_offset(&time[8..])?;
    let naive = chrono::NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, min, sec)?;
    Some(naive.and_utc().timestamp() - i64::from(offset))
}

fn tz_offset(tz: &str) -> Option<i32> {
    let tz = tz.trim();
    if tz.is_empty() {
        return Some(0);
    }
    let (sign, rest) = tz
        .strip_prefix('+')
        .map(|rest| (1, rest))
        .or_else(|| tz.strip_prefix('-').map(|rest| (-1, rest)))?;
    let rest = rest.replace(':', "");
    if rest.len() < 4 {
        return None;
    }
    let hours: i32 = rest[..2].parse().ok()?;
    let mins: i32 = rest[2..4].parse().ok()?;
    Some(sign * (hours * 3600 + mins * 60))
}

fn status_word(word: &str) -> Option<&str> {
    let word = word.trim_matches(|c: char| matches!(c, '.' | ',' | ';'));
    match word {
        "pass" | "warn" | "fail" | "blocked" | "unknown" => Some(word),
        _ => None,
    }
}

fn parse_probe(content: &str) -> Option<(String, String)> {
    let content = content.trim();
    if content.is_empty() {
        return None;
    }
    if let Some((name, rest)) = content.split_once(':') {
        let name = name.trim();
        let word = rest.split_whitespace().next().unwrap_or("");
        let status = status_word(word)?;
        if name.is_empty() {
            return None;
        }
        return Some((name.to_string(), status.to_string()));
    }
    let parts: Vec<&str> = content.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let status = status_word(parts[parts.len() - 1])?;
    let name = parts[..parts.len() - 1].join(" ");
    if name.is_empty() {
        None
    } else {
        Some((name, status.to_string()))
    }
}

fn push_sample(
    out: &mut Vec<ProbeSample>,
    seen: &mut BTreeSet<(i64, String, String)>,
    ts: i64,
    content: &str,
) {
    let Some((name, status)) = parse_probe(content) else {
        return;
    };
    if seen.insert((ts, name.clone(), status.clone())) {
        out.push(ProbeSample { ts, name, status });
    }
}

fn probe_samples(db: &rusqlite::Connection) -> Vec<ProbeSample> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    if let Ok(mut stmt) =
        db.prepare("SELECT timestamp, content FROM friday_observations WHERE source = 'probe'")
    {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                push_sample(&mut out, &mut seen, row.0, &row.1);
            }
        }
    }
    if let Ok(mut stmt) =
        db.prepare("SELECT updated_at, fact FROM friday_knowledge WHERE source = 'probe'")
    {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                push_sample(&mut out, &mut seen, row.0, &row.1);
            }
        }
    }
    out
}

fn pass_to_warn(samples: &[ProbeSample], syu: i64) -> Option<String> {
    use std::collections::BTreeMap;
    let mut by: BTreeMap<&str, Vec<&ProbeSample>> = BTreeMap::new();
    for sample in samples {
        by.entry(sample.name.as_str()).or_default().push(sample);
    }
    let mut strict = Vec::new();
    let mut fallback = Vec::new();
    for (name, mut group) in by {
        group.sort_by_key(|s| s.ts);
        let before = group.iter().rev().find(|s| s.ts <= syu);
        let after = group.iter().rev().find(|s| s.ts > syu);
        // A row on each side of the -Syu is the comparison. An older pass-to-warn
        // does not count once a later row already says what the probe is.
        if let (Some(before), Some(after)) = (before, after) {
            if before.status == "pass" && after.status == "warn" {
                strict.push(name);
            }
            continue;
        }
        let Some(current) = group.last() else {
            continue;
        };
        let previous = group
            .iter()
            .rev()
            .skip(1)
            .find(|s| s.status != current.status);
        if let Some(previous) = previous {
            if previous.status == "pass" && current.status == "warn" {
                fallback.push(name);
            }
        }
    }
    let names = if strict.is_empty() { fallback } else { strict };
    if names.is_empty() {
        return None;
    }
    Some(
        names
            .iter()
            .map(|name| format!("{name} went from pass to warn after a pacman -Syu"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

#[cfg(test)]
mod int315 {
    use super::{reason, starter_rules};

    fn git_repo() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "int315-reason-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            let status = std::process::Command::new("git")
                .args(args)
                .current_dir(&dir)
                .env("GIT_AUTHOR_NAME", "test")
                .env("GIT_AUTHOR_EMAIL", "test@example.com")
                .env("GIT_COMMITTER_NAME", "test")
                .env("GIT_COMMITTER_EMAIL", "test@example.com")
                .status()
                .unwrap();
            assert!(status.success(), "{args:?}");
        };
        run(&["init"]);
        std::fs::write(dir.join("f"), "x\n").unwrap();
        run(&["add", "f"]);
        run(&["commit", "-m", "today"]);
        dir
    }

    #[test]
    fn ship_after_plain_git_commit_is_not_an_anomaly() {
        let repo = git_repo();
        let ctx = crate::app::context::AppContext {
            runtime: crate::runtime::Runtime {
                root: repo.clone(),
                logs: repo.join("logs"),
                cache: repo.join("cache"),
                snapshots: repo.join("snapshots"),
                locks: repo.join("locks"),
                db: rusqlite::Connection::open_in_memory().unwrap(),
            },
            capabilities: crate::capabilities::CapabilityContext::unprivileged(),
            home: repo.to_string_lossy().to_string(),
            core_root: repo.to_string_lossy().to_string(),
            source_root: repo.join("zero").to_string_lossy().to_string(),
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        ctx.runtime
            .db
            .execute_batch(
                "CREATE TABLE deploy_patterns (timestamp INTEGER NOT NULL);
                 CREATE TABLE events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    domain TEXT NOT NULL,
                    action TEXT NOT NULL,
                    timestamp INTEGER NOT NULL
                 );",
            )
            .unwrap();
        ctx.runtime
            .db
            .execute(
                "INSERT INTO deploy_patterns (timestamp) VALUES (?1)",
                rusqlite::params![now],
            )
            .unwrap();
        let observations = reason(&ctx).unwrap();
        let hit = observations
            .iter()
            .any(|(text, _, kind)| kind == "anomaly" && text.contains("without a commit"));
        let _ = std::fs::remove_dir_all(&repo);
        assert!(
            !hit,
            "a ship after a plain git commit is not deploy_without_commit: {observations:?}"
        );
    }

    #[test]
    fn starter_rules_doc_matches_count() {
        let src = include_str!("reasoning.rs");
        let line = src
            .lines()
            .find(|line| line.contains("starter rule set"))
            .expect("starter rule doc comment");
        let claimed: usize = line
            .split_whitespace()
            .find_map(|word| word.parse().ok())
            .expect("a count in the starter rule doc comment");
        let held = starter_rules().len();
        assert_eq!(
            claimed, held,
            "doc says {claimed} rules, starter_rules holds {held}"
        );
    }
}

#[cfg(test)]
mod int316 {
    use super::cite_probe_change;

    #[test]
    fn g4_rule_cites_a_probe_change_after_pacman_syu() {
        let dir = std::env::temp_dir().join(format!(
            "int316-pacman-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let log_path = dir.join("pacman.log");
        std::fs::write(
            &log_path,
            "[2026-11-15T12:00:00+0000] [PACMAN] Running 'pacman -Syu'\n[2026-11-15T12:00:01+0000] [ALPM] transaction completed\n",
        )
        .unwrap();
        let db = rusqlite::Connection::open_in_memory().unwrap();
        db.execute_batch(
            "CREATE TABLE friday_observations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                source TEXT NOT NULL,
                kind TEXT NOT NULL,
                content TEXT NOT NULL
            );",
        )
        .unwrap();
        db.execute(
            "INSERT INTO friday_observations (timestamp, source, kind, content) VALUES (1763203100, 'probe', 'status', 'Reboot Needed pass')",
            [],
        )
        .unwrap();
        db.execute(
            "INSERT INTO friday_observations (timestamp, source, kind, content) VALUES (1763203300, 'probe', 'status', 'Reboot Needed warn')",
            [],
        )
        .unwrap();
        let said = cite_probe_change(&db, &log_path);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            said.as_deref(),
            Some("Reboot Needed went from pass to warn after a pacman -Syu"),
            "injected log {}",
            log_path.display()
        );
    }
}
