//! tradeoffs domain — surface competing values in every decision (Core v9 Phase 3)
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS forest_tradeoffs (
    id          TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    axes        TEXT NOT NULL,
    scores      TEXT NOT NULL,
    recommendation TEXT NOT NULL,
    confidence  TEXT NOT NULL DEFAULT 'medium',
    linked_goal TEXT,
    created_at  INTEGER NOT NULL
);";

fn ensure_schema(ctx: &AppContext) {
    let _ = ctx.runtime.db.execute_batch(SCHEMA);
}

fn next_id(ctx: &AppContext) -> String {
    let count: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM forest_tradeoffs", [], |r| r.get(0))
        .unwrap_or(0);
    format!("TRADEOFF-{:03}", count + 1)
}

fn read_health(ctx: &AppContext) -> u32 {
    let root = &ctx.core_root;
    std::fs::read_to_string(std::path::PathBuf::from(&root).join("runtime/cache/health.txt"))
        .unwrap_or_else(|_| "95".to_string())
        .trim()
        .trim_end_matches('%')
        .parse()
        .unwrap_or(95)
}

fn confidence_colored(c: &str) -> colored::ColoredString {
    match c {
        "high" => "HIGH".bright_green(),
        "low" => "LOW".bright_red(),
        _ => "MEDIUM".yellow(),
    }
}

fn score_bar(score: f64) -> String {
    let filled = (score * 10.0).round() as usize;
    let empty = 10usize.saturating_sub(filled);
    format!(
        "[{}{}] {:.0}%",
        "█".repeat(filled).bright_green().to_string(),
        "░".repeat(empty).dimmed().to_string(),
        score * 100.0
    )
}

struct AxisAnalysis {
    name: &'static str,
    left: &'static str,
    right: &'static str,
    left_score: f64,
    right_score: f64,
    tension: String,
    lean: &'static str,
}

fn analyze_axes(ctx: &AppContext, description: &str) -> Vec<AxisAnalysis> {
    let desc = description.to_lowercase();
    let health = read_health(ctx);

    // Read decision history for context
    let recent_failures: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='failure' \
             AND timestamp > strftime('%s','now','-30 days')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let evolution_proposals: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM evolution_proposals WHERE status='pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let active_goals: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM forest_goals WHERE status='accepted'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let security_findings: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM audit_scores WHERE score < 0.7",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Axis 1: Stability <-> Evolution
    let stability_score = (health as f64 / 100.0).min(1.0);
    let evolution_score = if evolution_proposals > 0 { 0.7 } else { 0.3 };
    let stab_tension =
        if desc.contains("refactor") || desc.contains("rewrite") || desc.contains("migrate") {
            "High: structural change directly challenges stability".to_string()
        } else if desc.contains("add") || desc.contains("new") || desc.contains("implement") {
            "Medium: new capability adds complexity load".to_string()
        } else {
            "Low: change is contained".to_string()
        };
    let stab_lean = if stability_score > evolution_score {
        "stability"
    } else {
        "evolution"
    };

    // Axis 2: Complexity <-> Capability
    let tool_count: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM audit_scores", [], |r| r.get(0))
        .unwrap_or(0);
    let complexity_score = ((tool_count as f64 / 60.0) + (active_goals as f64 / 10.0)) / 2.0;
    let complexity_score = complexity_score.min(1.0);
    let capability_score =
        if desc.contains("feature") || desc.contains("add") || desc.contains("new") {
            0.8
        } else {
            0.4
        };
    let comp_tension = if complexity_score > 0.6 {
        "High: system already complex — new capability adds load".to_string()
    } else {
        "Low: system has capacity for new capability".to_string()
    };
    let comp_lean = if complexity_score > capability_score {
        "complexity concern"
    } else {
        "capability gain"
    };

    // Axis 3: Privacy <-> Utility
    let privacy_score = if security_findings > 3 { 0.8 } else { 0.5 };
    let utility_score =
        if desc.contains("vault") || desc.contains("secret") || desc.contains("credential") {
            0.9
        } else if desc.contains("network") || desc.contains("fetch") || desc.contains("remote") {
            0.7
        } else {
            0.5
        };
    let priv_tension = if privacy_score > 0.6 {
        format!("Medium: {} open security findings noted", security_findings)
    } else {
        "Low: no significant privacy concerns identified".to_string()
    };
    let priv_lean = if privacy_score > utility_score {
        "privacy caution"
    } else {
        "utility gain"
    };

    // Axis 4: Performance <-> Power
    let perf_score = if desc.contains("poll") || desc.contains("watch") || desc.contains("daemon") {
        0.7
    } else {
        0.3
    };
    let power_score = 1.0 - perf_score;
    let perf_tension = if perf_score > 0.5 {
        "Medium: polling/daemon work has power cost".to_string()
    } else {
        "Low: no significant performance/power tradeoff".to_string()
    };
    let perf_lean = if perf_score > power_score {
        "performance concern"
    } else {
        "power efficient"
    };

    // Overall recommendation
    let _ = recent_failures;

    vec![
        AxisAnalysis {
            name: "Stability ↔ Evolution",
            left: "Stability",
            right: "Evolution",
            left_score: stability_score,
            right_score: evolution_score,
            tension: stab_tension,
            lean: stab_lean,
        },
        AxisAnalysis {
            name: "Complexity ↔ Capability",
            left: "Complexity",
            right: "Capability",
            left_score: complexity_score,
            right_score: capability_score,
            tension: comp_tension,
            lean: comp_lean,
        },
        AxisAnalysis {
            name: "Privacy ↔ Utility",
            left: "Privacy",
            right: "Utility",
            left_score: privacy_score,
            right_score: utility_score,
            tension: priv_tension,
            lean: priv_lean,
        },
        AxisAnalysis {
            name: "Performance ↔ Power",
            left: "Performance",
            right: "Power",
            left_score: perf_score,
            right_score: power_score,
            tension: perf_tension,
            lean: perf_lean,
        },
    ]
}

fn overall_recommendation(axes: &[AxisAnalysis]) -> (String, &'static str) {
    let high_tension = axes
        .iter()
        .filter(|a| a.tension.starts_with("High"))
        .count();
    let medium_tension = axes
        .iter()
        .filter(|a| a.tension.starts_with("Medium"))
        .count();

    if high_tension == 0 && medium_tension <= 1 {
        (
            "Conditions favorable — proceed with confidence".to_string(),
            "high",
        )
    } else if high_tension >= 2 {
        (
            "Multiple high-tension axes — consider deferring or scoping down".to_string(),
            "low",
        )
    } else {
        (
            "Proceed with care — review tension axes before committing".to_string(),
            "medium",
        )
    }
}

pub fn analyze(ctx: &AppContext, description: &str) -> CoreResult<()> {
    ensure_schema(ctx);

    // Find linked goal if any
    let linked_goal: Option<String> = ctx
        .runtime
        .db
        .query_row(
            "SELECT id FROM forest_goals WHERE status='accepted' \
             ORDER BY created_at DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();

    let axes = analyze_axes(ctx, description);
    let (recommendation, confidence) = overall_recommendation(&axes);

    println!();
    println!(
        "  {} {}",
        "⚖  Tradeoff Analysis:".bright_cyan().bold(),
        description.bright_white().bold()
    );
    println!("{}", "━".repeat(56).dimmed());

    if let Some(ref goal) = linked_goal {
        println!("  Goal context: {}", goal.yellow());
        println!();
    }

    println!("  {}", "Values in tension:".bright_white().bold());
    println!();

    // Serialize for storage
    let mut axes_names = vec![];
    let mut scores_data = vec![];

    for axis in &axes {
        let left_label = if axis.left_score > axis.right_score {
            format!("+ {}", axis.left).bright_green().to_string()
        } else {
            format!("- {}", axis.left).dimmed().to_string()
        };
        let right_label = if axis.right_score > axis.left_score {
            format!("+ {}", axis.right).bright_green().to_string()
        } else {
            format!("- {}", axis.right).dimmed().to_string()
        };

        println!(
            "  {} {}",
            "◆".bright_cyan(),
            axis.name.bright_white().bold()
        );
        println!("    {}  vs  {}", left_label, right_label);
        println!("    Tension:  {}", axis.tension.dimmed());
        println!("    Lean:     {}", axis.lean.yellow());
        println!();

        axes_names.push(axis.name);
        scores_data.push(format!("{:.2}/{:.2}", axis.left_score, axis.right_score));
    }

    println!("{}", "─".repeat(56).dimmed());
    println!(
        "  {} {}",
        "Recommendation:".bright_white().bold(),
        recommendation.bright_white()
    );
    println!(
        "  {} {}",
        "Confidence:".dimmed(),
        confidence_colored(confidence)
    );
    println!();
    println!("  {}", "The forest weighs. You decide.".dimmed().italic());
    println!("{}", "━".repeat(56).dimmed());
    println!();

    // Store analysis
    let id = next_id(ctx);
    let now = chrono::Utc::now().timestamp();
    let axes_json = serde_json::to_string(&axes_names).unwrap_or_default();
    let scores_json = serde_json::to_string(&scores_data).unwrap_or_default();

    let _ = ctx.runtime.db.execute(
        "INSERT INTO forest_tradeoffs \
         (id,description,axes,scores,recommendation,confidence,linked_goal,created_at) \
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            id,
            description,
            axes_json,
            scores_json,
            recommendation,
            confidence,
            linked_goal.unwrap_or_default(),
            now
        ],
    );

    let _ = ctx.runtime.db.execute(
        "INSERT INTO events (domain,action,payload,timestamp) \
         VALUES ('tradeoffs','analyzed',?1,?2)",
        params![format!("tradeoff:{}", id), now],
    );

    Ok(())
}

pub fn history(ctx: &AppContext) -> CoreResult<()> {
    ensure_schema(ctx);

    let mut stmt = match ctx.runtime.db.prepare(
        "SELECT id, description, confidence, linked_goal, created_at \
         FROM forest_tradeoffs ORDER BY created_at DESC LIMIT 20",
    ) {
        Ok(s) => s,
        Err(_) => {
            println!("  No tradeoff analyses yet — run: core tradeoff analyze <decision>");
            return Ok(());
        }
    };

    let rows: Vec<(String, String, String, String, i64)> = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get::<_, String>(3).unwrap_or_default(),
                r.get(4)?,
            ))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    println!();
    if rows.is_empty() {
        println!("  No tradeoff analyses yet — run: core tradeoff analyze <decision>");
    } else {
        println!("  {}", "Tradeoff History".bright_white().bold());
        println!("{}", "━".repeat(56).dimmed());
        for (id, desc, confidence, goal, ts) in &rows {
            let date = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            println!(
                "  {}  {}  {}",
                id.bright_cyan(),
                date.dimmed(),
                confidence_colored(confidence)
            );
            println!("     {}", desc);
            if !goal.is_empty() {
                println!("     Goal: {}", goal.yellow());
            }
            println!();
        }
        println!("{}", "━".repeat(56).dimmed());
    }
    println!();
    Ok(())
}

pub fn balance(ctx: &AppContext) -> CoreResult<()> {
    ensure_schema(ctx);

    let health = read_health(ctx);
    let evolution_proposals: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM evolution_proposals WHERE status='pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let active_goals: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM forest_goals WHERE status='accepted'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let security_findings: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM audit_scores WHERE score < 0.7",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let total_tradeoffs: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM forest_tradeoffs", [], |r| r.get(0))
        .unwrap_or(0);

    let stability = health as f64 / 100.0;
    let evolution = if evolution_proposals > 0 { 0.65 } else { 0.35 };
    let complexity = (active_goals as f64 / 10.0).min(1.0);
    let capability = 0.7_f64;
    let privacy = if security_findings > 3 { 0.75 } else { 0.50 };
    let utility = 0.65_f64;

    println!();
    println!("  {}", "⚖  System Balance State".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  Tradeoff analyses recorded: {}",
        total_tradeoffs.to_string().yellow()
    );
    println!();

    println!("  {} {}", "Stability ↔ Evolution".bright_white().bold(), "");
    println!("    Stability  {}", score_bar(stability));
    println!("    Evolution  {}", score_bar(evolution));
    let se_state = if stability > 0.85 && evolution < 0.5 {
        "Forest is stable — capacity for evolution exists".yellow()
    } else if stability < 0.9 {
        "Stability focus recommended before expanding".bright_red()
    } else {
        "Balanced — proceed with intention".bright_green()
    };
    println!("    State: {}", se_state);
    println!();

    println!(
        "  {} {}",
        "Complexity ↔ Capability".bright_white().bold(),
        ""
    );
    println!("    Complexity {}", score_bar(complexity));
    println!("    Capability {}", score_bar(capability));
    let cc_state = if complexity > 0.7 {
        "High complexity — consider retiring tools before adding".yellow()
    } else {
        "Healthy — capacity for new capability".bright_green()
    };
    println!("    State: {}", cc_state);
    println!();

    println!("  {} {}", "Privacy ↔ Utility".bright_white().bold(), "");
    println!("    Privacy    {}", score_bar(privacy));
    println!("    Utility    {}", score_bar(utility));
    let pu_state = if security_findings > 5 {
        "Privacy pressure elevated — review findings".bright_red()
    } else {
        "Balanced — no critical privacy concerns".bright_green()
    };
    println!("    State: {}", pu_state);
    println!();

    println!("{}", "─".repeat(56).dimmed());
    println!(
        "  {} Run: core tradeoff analyze <decision>  before major changes",
        "→".dimmed()
    );
    println!("{}", "━".repeat(56).dimmed());
    println!();

    Ok(())
}
