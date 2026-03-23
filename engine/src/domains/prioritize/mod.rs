//! prioritize domain — dynamic goal reranking based on live conditions (Core v9 Phase 4)
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

struct GoalScore {
    id:          String,
    title:       String,
    base:        &'static str,   // original priority
    score:       f64,            // computed score 0.0–1.0
    factors:     Vec<(String, f64, String)>, // (factor, delta, reason)
    has_plan:    bool,
    plan_risk:   String,
    has_tradeoff: bool,
}

fn read_health(ctx: &AppContext) -> u32 {
    let root = &ctx.core_root;
    std::fs::read_to_string(
        std::path::PathBuf::from(&root).join("runtime/cache/health.txt")
    ).unwrap_or_else(|_| "95".to_string())
    .trim().trim_end_matches('%').parse().unwrap_or(95)
}

fn read_forecast_trend(ctx: &AppContext) -> f64 {
    // Read last two health scores from events to derive trend
    let vals: Vec<f64> = ctx.runtime.db.prepare(
        "SELECT payload FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 5"
    ).ok().map(|mut s| {
        s.query_map([], |r| r.get::<_,String>(0))
         .map(|rows| rows.filter_map(|r| r.ok())
              .filter_map(|p| p.parse::<f64>().ok())
              .collect())
         .unwrap_or_default()
    }).unwrap_or_default();

    if vals.len() >= 2 {
        vals[0] - vals[1]
    } else {
        0.0
    }
}

fn base_score(priority: &str) -> f64 {
    match priority {
        "HIGH"   => 0.8,
        "MEDIUM" => 0.5,
        _        => 0.3,
    }
}

fn score_goals(ctx: &AppContext) -> Vec<GoalScore> {
    let health       = read_health(ctx);
    let trend        = read_forecast_trend(ctx);
    let security_findings: i64 = ctx.runtime.db
        .query_row("SELECT COUNT(*) FROM audit_scores WHERE score < 0.7",
            [], |r| r.get(0)).unwrap_or(0);
    let active_intents: i64 = ctx.runtime.db
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE outcome='pending'",
            [], |r| r.get(0)).unwrap_or(0);

    // Load all accepted goals
    let mut stmt = match ctx.runtime.db.prepare(
        "SELECT id, title, priority FROM forest_goals WHERE status='accepted' ORDER BY created_at"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let goals: Vec<(String,String,String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    goals.into_iter().map(|(id, title, priority)| {
        let mut score = base_score(&priority);
        let mut factors: Vec<(String, f64, String)> = vec![];

        // Factor 1 — Health gate
        // Below 95%: stability goals get +0.3, expansion goals get -0.2
        let title_lower = title.to_lowercase();
        let is_stability = title_lower.contains("health") || title_lower.contains("stability")
            || title_lower.contains("fix") || title_lower.contains("restore");
        if health < 95 {
            if is_stability {
                score += 0.3;
                factors.push(("Health gate".into(), 0.3,
                    format!("Health {}% < 95% — stability goals elevated", health)));
            } else {
                score -= 0.2;
                factors.push(("Health gate".into(), -0.2,
                    format!("Health {}% < 95% — expansion goals deprioritized", health)));
            }
        } else {
            factors.push(("Health gate".into(), 0.0,
                format!("Health {}% ≥ 95% — no gate applied", health)));
        }

        // Factor 2 — Forecast trend
        if trend < -1.0 {
            if is_stability {
                score += 0.2;
                factors.push(("Forecast trend".into(), 0.2,
                    format!("Trend {:.1} declining — stability elevated", trend)));
            } else {
                score -= 0.1;
                factors.push(("Forecast trend".into(), -0.1,
                    format!("Trend {:.1} declining — expansion reduced", trend)));
            }
        } else if trend > 0.5 {
            score += 0.05;
            factors.push(("Forecast trend".into(), 0.05,
                format!("Trend {:.1} positive — slight boost", trend)));
        } else {
            factors.push(("Forecast trend".into(), 0.0,
                format!("Trend {:.1} stable", trend)));
        }

        // Factor 3 — Security posture
        if security_findings > 5 {
            let is_security = title_lower.contains("security") || title_lower.contains("audit")
                || title_lower.contains("retire");
            if is_security {
                score += 0.2;
                factors.push(("Security posture".into(), 0.2,
                    format!("{} findings open — security goals elevated", security_findings)));
            } else {
                factors.push(("Security posture".into(), 0.0,
                    format!("{} findings open — not security-related", security_findings)));
            }
        } else {
            factors.push(("Security posture".into(), 0.0,
                "Security posture acceptable".into()));
        }

        // Factor 4 — Intent backlog
        if active_intents > 5 {
            score -= 0.1;
            factors.push(("Intent backlog".into(), -0.1,
                format!("{} pending decisions — reduce scope", active_intents)));
        } else {
            factors.push(("Intent backlog".into(), 0.0,
                format!("{} pending decisions — backlog manageable", active_intents)));
        }

        // Factor 5 — Plan and tradeoff readiness
        let has_plan: bool = ctx.runtime.db
            .query_row(
                "SELECT COUNT(*) FROM forest_plans WHERE goal_id=?1",
                rusqlite::params![id], |r| r.get::<_,i64>(0))
            .unwrap_or(0) > 0;

        let plan_risk: String = ctx.runtime.db
            .query_row(
                "SELECT risk FROM forest_plans WHERE goal_id=?1 LIMIT 1",
                rusqlite::params![id], |r| r.get(0))
            .unwrap_or_else(|_| "UNKNOWN".into());

        let has_tradeoff: bool = ctx.runtime.db
            .query_row(
                "SELECT COUNT(*) FROM forest_tradeoffs WHERE linked_goal=?1",
                rusqlite::params![id], |r| r.get::<_,i64>(0))
            .unwrap_or(0) > 0;

        if has_plan {
            score += 0.1;
            factors.push(("Plan readiness".into(), 0.1,
                format!("Plan exists (risk: {}) — actionable", plan_risk)));
        } else {
            score -= 0.05;
            factors.push(("Plan readiness".into(), -0.05,
                "No plan yet — run: core plan generate".into()));
        }

        if has_tradeoff {
            score += 0.05;
            factors.push(("Tradeoff analysis".into(), 0.05,
                "Tradeoff analyzed — values understood".into()));
        } else {
            factors.push(("Tradeoff analysis".into(), 0.0,
                "No tradeoff analysis yet".into()));
        }

        GoalScore {
            id, title, base: match priority.as_str() {
                "HIGH" => "HIGH", "MEDIUM" => "MEDIUM", _ => "LOW"
            },
            score: score.clamp(0.0, 1.0),
            factors,
            has_plan,
            plan_risk,
            has_tradeoff,
        }
    }).collect()
}

fn priority_label(score: f64) -> colored::ColoredString {
    if score >= 0.7      { "HIGH  ".bright_red() }
    else if score >= 0.4 { "MEDIUM".yellow() }
    else                 { "LOW   ".dimmed() }
}

pub fn prioritize(ctx: &AppContext) -> CoreResult<()> {
    let mut goals = score_goals(ctx);
    if goals.is_empty() {
        println!();
        println!("  No accepted goals — run: core goals generate");
        println!("  Then: core goals accept <id>");
        println!();
        return Ok(());
    }

    // Sort by score descending
    goals.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let health = read_health(ctx);
    println!();
    println!("  {} {}", "⚖  Dynamic Prioritization".bright_cyan().bold(),
        format!("(health: {}%)", health).dimmed());
    println!("{}", "━".repeat(56).dimmed());
    println!();

    for (rank, goal) in goals.iter().enumerate() {
        let rank_str = format!("#{}", rank + 1);
        println!("  {} {} {}  {}",
            rank_str.bright_cyan().bold(),
            priority_label(goal.score),
            goal.id.yellow(),
            goal.title.bright_white()
        );
        println!("    Base: {}  Score: {:.2}  Plan: {}  Tradeoff: {}",
            goal.base.dimmed(),
            goal.score,
            if goal.has_plan {
                format!("✓ ({})", goal.plan_risk).bright_green().to_string()
            } else {
                "✗".bright_red().to_string()
            },
            if goal.has_tradeoff { "✓".bright_green().to_string() }
            else { "✗".dimmed().to_string() }
        );
        println!();
    }

    println!("{}", "─".repeat(56).dimmed());
    println!("  {} run {} for factor breakdown",
        "→".dimmed(), "core prioritize explain".bright_cyan());
    println!("{}", "━".repeat(56).dimmed());
    println!();

    // Emit event
    let now = chrono::Utc::now().timestamp();
    let _ = ctx.runtime.db.execute(
        "INSERT INTO events (domain,action,payload,timestamp) VALUES ('prioritize','ranked',?1,?2)",
        rusqlite::params![format!("{} goals ranked", goals.len()), now],
    );

    Ok(())
}

pub fn explain(ctx: &AppContext) -> CoreResult<()> {
    let mut goals = score_goals(ctx);
    if goals.is_empty() {
        println!();
        println!("  No accepted goals — run: core goals generate");
        println!();
        return Ok(());
    }

    goals.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let health = read_health(ctx);
    println!();
    println!("  {} {}", "⚖  Prioritization Explained".bright_cyan().bold(),
        format!("(health: {}%)", health).dimmed());

    for (rank, goal) in goals.iter().enumerate() {
        println!();
        println!("{}", "━".repeat(56).dimmed());
        println!("  #{} {} — {}  (score: {:.2})",
            rank + 1,
            goal.id.yellow(),
            goal.title.bright_white(),
            goal.score
        );
        println!("  {}", "Factor breakdown:".bright_white().bold());
        for (factor, delta, reason) in &goal.factors {
            let delta_str = if *delta > 0.0 {
                format!("+{:.2}", delta).bright_green().to_string()
            } else if *delta < 0.0 {
                format!("{:.2}", delta).bright_red().to_string()
            } else {
                " 0.00".dimmed().to_string()
            };
            println!("    {} {}  {}",
                delta_str,
                factor.bright_white(),
                reason.dimmed()
            );
        }
    }

    println!();
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}", "The forest ranks. You decide.".dimmed().italic());
    println!();
    Ok(())
}
