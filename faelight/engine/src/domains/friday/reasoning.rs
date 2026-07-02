//! INT-251 v23 Pillar 2 -- Reasoning Engine
//! Rule-based, no LLM. Watches event stream, produces system-wide observations.

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
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

/// The starter rule set -- 10 rules covering key system patterns
pub fn starter_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "deploy_without_commit",
            description: "Deploy happened but no commit in last 5 min -- dirty deploy risk",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let deploys: i64 = db.query_row(
                    "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
                    rusqlite::params![now - 300],
                    |r| r.get(0),
                ).unwrap_or(0);
                let commits: i64 = db.query_row(
                    "SELECT COUNT(*) FROM events WHERE domain='git' AND action='commit' AND timestamp > ?1",
                    rusqlite::params![now - 300],
                    |r| r.get(0),
                ).unwrap_or(0);
                if deploys > 0 && commits == 0 {
                    Some(Observation {
                        conclusion: format!("{} deploy(s) in last 5min without a commit -- deploying uncommitted changes", deploys),
                        confidence: 0.9,
                        kind: ObservationKind::Anomaly,
                    })
                } else { None }
            },
        },
        Rule {
            name: "high_deploy_frequency",
            description: "More than 10 deploys in an hour -- possible thrashing",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let deploys: i64 = db.query_row(
                    "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
                    rusqlite::params![now - 3600],
                    |r| r.get(0),
                ).unwrap_or(0);
                if deploys > 10 {
                    Some(Observation {
                        conclusion: format!("{} deploys in the last hour -- consider batching changes", deploys),
                        confidence: 0.8,
                        kind: ObservationKind::SystemWide,
                    })
                } else { None }
            },
        },
        Rule {
            name: "compositor_active",
            description: "faelight-compositor is receiving window events",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let events: i64 = db.query_row(
                    "SELECT COUNT(*) FROM events WHERE domain='compositor' AND timestamp > ?1",
                    rusqlite::params![now - 3600],
                    |r| r.get(0),
                ).unwrap_or(0);
                if events > 0 {
                    Some(Observation {
                        conclusion: format!("faelight-compositor active -- {} window events in last hour", events),
                        confidence: 0.95,
                        kind: ObservationKind::SystemWide,
                    })
                } else { None }
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
                let recent_cmds: i64 = db.query_row(
                    "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",
                    rusqlite::params![now - 7200],
                    |r| r.get(0),
                ).unwrap_or(0);
                if recent_cmds > 50 && lifecycle == 0 {
                    Some(Observation {
                        conclusion: "Active session (50+ commands) with no intent lifecycle activity in 2h -- work may be undocumented".to_string(),
                        confidence: 0.75,
                        kind: ObservationKind::Anomaly,
                    })
                } else { None }
            },
        },
        Rule {
            name: "shell_health_signal",
            description: "Shell health events in the event stream",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let events: i64 = db.query_row(
                    "SELECT COUNT(*) FROM events WHERE domain='shell' AND timestamp > ?1",
                    rusqlite::params![now - 86400],
                    |r| r.get(0),
                ).unwrap_or(0);
                if events > 0 {
                    Some(Observation {
                        conclusion: format!("fsh emitting {} events today -- shell intelligence active", events),
                        confidence: 0.9,
                        kind: ObservationKind::Normal,
                    })
                } else { None }
            },
        },
        Rule {
            name: "commits_without_deploy",
            description: "Commits in last 2h with no deploy -- changes may be uncommitted to runtime",
            check: |ctx| {
                let db = &ctx.runtime.db;
                let now = now_ts();
                let two_hours_ago = now - 7200;
                let commits: i64 = db.query_row(
                    "SELECT COUNT(*) FROM git_operations WHERE operation='commit' AND timestamp > ?1",
                    rusqlite::params![two_hours_ago],
                    |r| r.get(0),
                ).unwrap_or(0);
                let deploys: i64 = db.query_row(
                    "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
                    rusqlite::params![two_hours_ago],
                    |r| r.get(0),
                ).unwrap_or(0);
                if commits >= 3 && deploys == 0 {
                    Some(Observation {
                        conclusion: format!("{} commit(s) in last 2h with no deploy -- consider deploying to validate changes", commits),
                        confidence: 0.8,
                        kind: ObservationKind::Causal,
                    })
                } else { None }
            },
        },
    Rule {
        name: "rapid_fix_cycle",
        description: "Detect rapid deploy cycles suggesting a tool change caused a regression",
        check: |ctx: &AppContext| {
            use std::collections::HashMap;
            use std::time::{SystemTime, UNIX_EPOCH};
            let db = &ctx.runtime.db;
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
            let one_hour_ago = now - 3600;
            let mut s = match db.prepare(
                "SELECT source_tool, timestamp FROM events WHERE action = 'deploy_completed' AND timestamp > ?1 ORDER BY source_tool, timestamp"
            ) {
                Ok(s) => s,
                Err(_) => return None,
            };
            let mapped = match s.query_map(rusqlite::params![one_hour_ago], |r: &rusqlite::Row<'_>| {
                Ok((r.get::<_, String>(0).unwrap_or_default(), r.get::<_, i64>(1).unwrap_or(0)))
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
        description: "Detect when a health check failed shortly after a deploy -- stale binary reference",
        check: |ctx: &AppContext| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let db = &ctx.runtime.db;
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
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
                |r: &rusqlite::Row<'_>| Ok((r.get::<_, String>(0).unwrap_or_default(), r.get::<_, String>(1).unwrap_or_default())),
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
    println!("  {} Friday -- Reasoning Engine", "🌲".normal());
    println!("  {}", "─".repeat(50).dimmed());
    println!();

    let observations = reason(ctx)?;

    if observations.is_empty() {
        println!("  {} No active observations -- system within normal parameters", "·".dimmed());
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
            println!("  {} [{}] {}", "→".bright_cyan(), kind_color, conclusion.bright_white());
            println!("    {} confidence: {:.0}%", "·".dimmed(), confidence * 100.0);
            println!();
        }
    }
    Ok(())
}
