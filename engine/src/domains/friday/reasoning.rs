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
