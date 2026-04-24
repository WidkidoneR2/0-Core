//! weight_engine domain — Core v17: Pattern Weight Engine
//!
//! Every pattern earns its weight through:
//! frequency, recency, consequence, trend, volatility, confidence
//!
//! Design constraints (architectural review):
//! - volatility = modifier, NOT a weight dimension
//! - trend asymmetry: worsening amplifies harder than improving dampens
//! - frequency = rate (occurrences / window_days), not raw count
//! - identity alignment clamped to [0.9, 1.1]
//! - WeightBreakdown required on every computed weight
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;
// ── Core Structs ──────────────────────────────────────────────────────────────
pub struct PatternMetrics {
    /// How often does this occur? Rate = occurrences / window_days (0.0 → 1.0)
    pub frequency: f64,
    /// How recent? Decay-adjusted (0.0 → 1.0)
    pub recency: f64,
    /// Severity of outcomes (0.0 → 1.0)
    pub consequence: f64,
    /// Direction: -1.0 (improving) → 1.0 (worsening)
    pub trend: f64,
    /// Consistency: 0.0 (stable) → 1.0 (chaotic)
    pub volatility: f64,
    /// Data reliability (0.0 → 1.0)
    pub confidence: f64,
}
/// Weights for each dimension — context-sensitive
/// NOTE: volatility is NOT here — it's a modifier applied separately
pub struct ContextWeights {
    pub frequency: f64,
    pub recency: f64,
    pub consequence: f64,
    pub trend: f64,
}
/// Full breakdown for explainability — every stage traceable
pub struct WeightBreakdown {
    pub base: f64,
    pub confidence_adjusted: f64,
    pub volatility_adjusted: f64,
    pub decay_adjusted: f64,
    pub identity_adjusted: f64,
    pub final_weight: f64,
}
pub enum WeightClass {
    Ignore,   // < 0.25
    Weak,     // 0.25–0.45
    Moderate, // 0.45–0.65
    Strong,   // 0.65–0.80
    Critical, // > 0.80
}
impl WeightClass {
    pub fn label(&self) -> &'static str {
        match self {
            WeightClass::Ignore => "IGNORE",
            WeightClass::Weak => "WEAK",
            WeightClass::Moderate => "MODERATE",
            WeightClass::Strong => "STRONG",
            WeightClass::Critical => "CRITICAL",
        }
    }
    pub fn behavior(&self) -> &'static str {
        match self {
            WeightClass::Ignore => "silent — not worth surfacing",
            WeightClass::Weak => "mention only if directly asked",
            WeightClass::Moderate => "suggest during relevant context",
            WeightClass::Strong => "recommend proactively",
            WeightClass::Critical => "challenge / interrupt current action",
        }
    }
    pub fn color(&self) -> colored::ColoredString {
        match self {
            WeightClass::Ignore => self.label().dimmed(),
            WeightClass::Weak => self.label().bright_cyan(),
            WeightClass::Moderate => self.label().bright_yellow(),
            WeightClass::Strong => self.label().bright_green(),
            WeightClass::Critical => self.label().bright_red().bold(),
        }
    }
}
pub fn classify_weight(weight: f64) -> WeightClass {
    match weight {
        w if w < 0.25 => WeightClass::Ignore,
        w if w < 0.45 => WeightClass::Weak,
        w if w < 0.65 => WeightClass::Moderate,
        w if w < 0.80 => WeightClass::Strong,
        _ => WeightClass::Critical,
    }
}
// ── Context Presets ───────────────────────────────────────────────────────────
pub fn weights_for_context(context: &str) -> ContextWeights {
    match context {
        "deployment" => ContextWeights {
            consequence: 0.40,
            recency: 0.25,
            frequency: 0.15,
            trend: 0.20,
        },
        "work_rhythm" => ContextWeights {
            recency: 0.40,
            frequency: 0.25,
            trend: 0.20,
            consequence: 0.15,
        },
        "prediction" => ContextWeights {
            frequency: 0.35,
            recency: 0.25,
            trend: 0.25,
            consequence: 0.15,
        },
        "health" => ContextWeights {
            consequence: 0.35,
            trend: 0.30,
            recency: 0.20,
            frequency: 0.15,
        },
        _ => ContextWeights {
            frequency: 0.25,
            recency: 0.25,
            consequence: 0.25,
            trend: 0.25,
        },
    }
}
// ── Core Weight Functions ─────────────────────────────────────────────────────
/// Trend factor: worsening amplifies, improving dampens
/// Design constraint: 0.4 coefficient for improvement (not 0.2)
pub fn trend_factor(trend: f64) -> f64 {
    match trend {
        t if t > 0.0 => 0.5 + (t * 0.5), // worsening: up to 1.0
        t => 0.5 + (t * 0.4),            // improving: down to 0.1
    }
}
/// Stability factor: volatile patterns are less trustworthy
/// Volatility is a MODIFIER — applied to base, not a weighted dimension
pub fn stability_factor(volatility: f64) -> f64 {
    1.0 - (volatility * 0.7)
}
/// Decay: patterns lose weight over time without reinforcement
pub fn apply_decay(weight: f64, age_hours: f64) -> f64 {
    let decay_rate = 0.015;
    weight * (-decay_rate * age_hours).exp()
}
/// Identity alignment — clamped to [0.9, 1.1]
/// Values inform but do NOT override reality
pub fn apply_identity_alignment(weight: f64, alignment: f64) -> f64 {
    let clamped = alignment.clamp(0.9, 1.1);
    weight * clamped
}
/// Compute weight with full breakdown for explainability
pub fn compute_weight_with_breakdown(
    m: &PatternMetrics,
    w: &ContextWeights,
    age_hours: f64,
    alignment: f64,
) -> WeightBreakdown {
    // Stage 1: base from weighted dimensions
    let base = (m.frequency * w.frequency)
        + (m.recency * w.recency)
        + (m.consequence * w.consequence)
        + (trend_factor(m.trend) * w.trend);
    // Stage 2: confidence — data reliability scales everything
    let confidence_adjusted = base * m.confidence;
    // Stage 3: volatility as modifier (NOT a weight dimension)
    let volatility_adjusted = confidence_adjusted * stability_factor(m.volatility);
    // Stage 4: temporal decay
    let decay_adjusted = apply_decay(volatility_adjusted, age_hours);
    // Stage 5: identity alignment [0.9, 1.1] clamp
    let identity_adjusted = apply_identity_alignment(decay_adjusted, alignment);
    WeightBreakdown {
        base,
        confidence_adjusted,
        volatility_adjusted,
        decay_adjusted,
        identity_adjusted,
        final_weight: identity_adjusted,
    }
}
// ── Database ──────────────────────────────────────────────────────────────────
pub fn ensure_tables(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS pattern_weights (
            id              TEXT PRIMARY KEY,
            description     TEXT NOT NULL,
            context         TEXT NOT NULL,
            frequency       REAL NOT NULL DEFAULT 0.0,
            recency         REAL NOT NULL DEFAULT 0.0,
            consequence     REAL NOT NULL DEFAULT 0.0,
            trend           REAL NOT NULL DEFAULT 0.0,
            volatility      REAL NOT NULL DEFAULT 0.0,
            confidence      REAL NOT NULL DEFAULT 0.5,
            final_weight    REAL NOT NULL DEFAULT 0.0,
            weight_class    TEXT NOT NULL DEFAULT 'Weak',
            base_weight     REAL NOT NULL DEFAULT 0.0,
            confidence_adj  REAL NOT NULL DEFAULT 0.0,
            volatility_adj  REAL NOT NULL DEFAULT 0.0,
            decay_adj       REAL NOT NULL DEFAULT 0.0,
            identity_adj    REAL NOT NULL DEFAULT 0.0,
            last_updated    INTEGER NOT NULL,
            occurrence_count INTEGER NOT NULL DEFAULT 0,
            window_days     INTEGER NOT NULL DEFAULT 30,
            is_positive     INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS weight_calibrations (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            pattern_id      TEXT NOT NULL,
            context         TEXT NOT NULL,
            predicted_importance REAL NOT NULL,
            actual_outcome  TEXT NOT NULL,
            was_correct     INTEGER NOT NULL,
            contribution_breakdown TEXT,
            calibrated_at   INTEGER NOT NULL
        );
    ",
    )?;
    Ok(())
}
/// Compute and store weight for a pattern
pub fn update_pattern_weight(
    ctx: &AppContext,
    id: &str,
    description: &str,
    context: &str,
    metrics: &PatternMetrics,
    age_hours: f64,
    is_positive: bool,
) -> CoreResult<WeightBreakdown> {
    ensure_tables(ctx)?;
    // Get alignment score from DB
    let alignment: f64 = ctx.runtime.db.query_row(
        "SELECT AVG(score) FROM alignment_checks WHERE checked_at > (strftime('%s','now') - 604800)",
        [], |r| r.get::<_, Option<f64>>(0)
    ).unwrap_or(None).unwrap_or(1.0);
    let weights = weights_for_context(context);
    let breakdown = compute_weight_with_breakdown(metrics, &weights, age_hours, alignment);
    let class = classify_weight(breakdown.final_weight);
    let now = now_ts();
    ctx.runtime.db.execute(
        "INSERT OR REPLACE INTO pattern_weights
         (id, description, context, frequency, recency, consequence, trend, volatility,
          confidence, final_weight, weight_class, base_weight, confidence_adj,
          volatility_adj, decay_adj, identity_adj, last_updated, occurrence_count,
          window_days, is_positive)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
        params![
            id,
            description,
            context,
            metrics.frequency,
            metrics.recency,
            metrics.consequence,
            metrics.trend,
            metrics.volatility,
            metrics.confidence,
            breakdown.final_weight,
            class.label(),
            breakdown.base,
            breakdown.confidence_adjusted,
            breakdown.volatility_adjusted,
            breakdown.decay_adjusted,
            breakdown.identity_adjusted,
            now,
            (metrics.frequency * 30.0) as i64,
            30i64,
            is_positive as i64
        ],
    )?;
    Ok(breakdown)
}
// ── Commands ──────────────────────────────────────────────────────────────────
/// core weight list — show all patterns ranked by weight
pub fn list(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!(
        "{}",
        "⚖️  Pattern Weights — Ranked by Importance".cyan().bold()
    );
    println!("{}", "━".repeat(70).dimmed());
    println!();
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, description, context, final_weight, weight_class,
                frequency, recency, consequence, trend, confidence, is_positive
         FROM pattern_weights
         ORDER BY final_weight DESC LIMIT 20",
    )?;
    let rows: Vec<(
        String,
        String,
        String,
        f64,
        String,
        f64,
        f64,
        f64,
        f64,
        f64,
        i64,
    )> = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
                r.get(10)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    if rows.is_empty() {
        println!(
            "  {} No patterns yet — run: core weight compute",
            "○".dimmed()
        );
        println!(
            "  {} Weights are built from forest events over time",
            "→".dimmed()
        );
        println!();
        return Ok(());
    }
    println!(
        "  {:<8} {:<14} {:<20} {:<8} {}",
        "Weight".dimmed(),
        "Class".dimmed(),
        "Context".dimmed(),
        "Conf".dimmed(),
        "Description".dimmed()
    );
    println!("  {}", "─".repeat(65).dimmed());
    for (id, desc, ctx_name, weight, class, _freq, _rec, _cons, _trend, conf, positive) in &rows {
        let class_enum = match class.as_str() {
            "IGNORE" => WeightClass::Ignore,
            "WEAK" => WeightClass::Weak,
            "MODERATE" => WeightClass::Moderate,
            "STRONG" => WeightClass::Strong,
            _ => WeightClass::Critical,
        };
        let pos_marker = if *positive == 1 { "✨" } else { "⚠️ " };
        println!(
            "  {:<8} {:<14} {:<20} {:<8} {} {}",
            format!("{:.3}", weight).bright_white(),
            class_enum.color(),
            ctx_name.bright_cyan(),
            format!("{:.0}%", conf * 100.0).dimmed(),
            pos_marker,
            desc.bright_white()
        );
        let _ = id;
    }
    println!();
    println!(
        "  {} Run: core weight explain <id> for full breakdown",
        "→".bright_cyan()
    );
    println!();
    Ok(())
}
/// core weight explain <id> — full decomposed explanation
pub fn explain(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let row: Option<(
        String,
        String,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
    )> = ctx
        .runtime
        .db
        .query_row(
            "SELECT description, context, final_weight, frequency, recency, consequence,
                    trend, volatility, confidence, base_weight, confidence_adj,
                    volatility_adj, decay_adj, identity_adj
             FROM pattern_weights WHERE id = ?1",
            params![id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                    r.get(10)?,
                    r.get(11)?,
                    r.get(12)?,
                    r.get(13)?,
                ))
            },
        )
        .ok();
    match row {
        None => {
            println!("  {} Pattern '{}' not found", "⚠️ ".yellow(), id);
            println!(
                "  {} Run: core weight list to see available patterns",
                "→".dimmed()
            );
        }
        Some((
            desc,
            ctx_name,
            final_w,
            freq,
            rec,
            cons,
            trend,
            vol,
            conf,
            base,
            conf_adj,
            vol_adj,
            decay_adj,
            id_adj,
        )) => {
            let class = classify_weight(final_w);
            println!();
            println!("{} Pattern: {}", "⚖️ ".normal(), desc.bright_white().bold());
            println!("{}", "━".repeat(60).dimmed());
            println!(
                "  {} {}  {} {}",
                "Context:".dimmed(),
                ctx_name.bright_cyan(),
                "Class:".dimmed(),
                class.color()
            );
            println!();
            println!("  {}", "Input Metrics:".bright_white().bold());
            println!(
                "  {:<20} {:.3}  (rate = occurrences/window_days)",
                "frequency:".dimmed(),
                freq
            );
            println!("  {:<20} {:.3}  (decay-adjusted)", "recency:".dimmed(), rec);
            println!(
                "  {:<20} {:.3}  (outcome severity)",
                "consequence:".dimmed(),
                cons
            );
            println!(
                "  {:<20} {:.3}  (-1=improving → 1=worsening)",
                "trend:".dimmed(),
                trend
            );
            println!(
                "  {:<20} {:.3}  (0=stable → 1=chaotic)",
                "volatility:".dimmed(),
                vol
            );
            println!(
                "  {:<20} {:.3}  (data reliability)",
                "confidence:".dimmed(),
                conf
            );
            println!();
            println!(
                "  {}",
                "Weight Breakdown (each stage):".bright_white().bold()
            );
            println!(
                "  {:<28} {:.4}  (weighted dimensions sum)",
                "base:".dimmed(),
                base
            );
            println!(
                "  {:<28} {:.4}  (× confidence)",
                "confidence_adjusted:".dimmed(),
                conf_adj
            );
            println!(
                "  {:<28} {:.4}  (× stability_factor(volatility))",
                "volatility_adjusted:".dimmed(),
                vol_adj
            );
            println!(
                "  {:<28} {:.4}  (× temporal decay)",
                "decay_adjusted:".dimmed(),
                decay_adj
            );
            println!(
                "  {:<28} {:.4}  (× alignment [0.9,1.1])",
                "identity_adjusted:".dimmed(),
                id_adj
            );
            println!();
            let fw: f64 = final_w;
            let weight_colored = if fw >= 0.80 {
                format!("{:.4}", fw).bright_red().bold()
            } else if fw >= 0.65 {
                format!("{:.4}", fw).bright_green()
            } else if fw >= 0.45 {
                format!("{:.4}", fw).bright_yellow()
            } else {
                format!("{:.4}", fw).dimmed()
            };
            println!(
                "  {:<28} {} → {}",
                "FINAL WEIGHT:".bright_white().bold(),
                weight_colored,
                class.color()
            );
            println!();
            println!("  {}", "Friday Behavior:".bright_white().bold());
            println!("  {} {}", "→".bright_cyan(), class.behavior());
            println!();
        }
    }
    Ok(())
}
/// core weight compute — scan events and compute weights for known patterns
pub fn compute(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("{}", "🔄 Computing Pattern Weights...".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    let now = now_ts();
    let window_secs = 30 * 86400i64; // 30 days
    let window_days = 30.0f64;
    // Pattern 1: commit frequency
    let commit_count: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM events WHERE domain='git' AND action='commit' AND timestamp > ?1",
            params![now - window_secs],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let commit_rate = (commit_count as f64 / window_days).min(1.0);
    let commit_recency: f64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT MAX(timestamp) FROM events WHERE domain='git' AND action='commit'",
            [],
            |r| r.get::<_, Option<i64>>(0),
        )
        .unwrap_or(None)
        .map(|ts| {
            let age_hours = (now - ts) as f64 / 3600.0;
            (1.0 - (age_hours / 168.0)).max(0.0) // 0 after 7 days
        })
        .unwrap_or(0.0);
    let commit_metrics = PatternMetrics {
        frequency: commit_rate.min(1.0),
        recency: commit_recency,
        consequence: 0.6,
        trend: if commit_rate > 0.5 { -0.3 } else { 0.2 },
        volatility: 0.2,
        confidence: 0.85,
    };
    let commit_breakdown = update_pattern_weight(
        ctx,
        "commit-velocity",
        "Commit velocity — shipping cadence",
        "work_rhythm",
        &commit_metrics,
        (now - ctx
            .runtime
            .db
            .query_row(
                "SELECT MAX(timestamp) FROM events WHERE domain='git' AND action='commit'",
                [],
                |r| r.get::<_, Option<i64>>(0),
            )
            .unwrap_or(None)
            .unwrap_or(now)) as f64
            / 3600.0,
        true,
    )?;
    println!(
        "  {} commit-velocity → {:.3} ({})",
        "✓".bright_green(),
        commit_breakdown.final_weight,
        classify_weight(commit_breakdown.final_weight).label()
    );
    // Pattern 2: health stability
    let health_drops: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM doctor_history WHERE health < 95 AND checked_at > ?1",
            params![now - window_secs],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let health_checks: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM doctor_history WHERE checked_at > ?1",
            params![now - window_secs],
            |r| r.get(0),
        )
        .unwrap_or(1);
    let drop_rate = health_drops as f64 / health_checks.max(1) as f64;
    let health_metrics = PatternMetrics {
        frequency: drop_rate,
        recency: if health_drops > 0 { 0.6 } else { 0.1 },
        consequence: 0.8,
        trend: if drop_rate > 0.1 { 0.4 } else { -0.2 },
        volatility: drop_rate * 0.5,
        confidence: 0.9,
    };
    let health_breakdown = update_pattern_weight(
        ctx,
        "health-drops",
        "Health drops below 95%",
        "health",
        &health_metrics,
        0.0,
        false,
    )?;
    println!(
        "  {} health-drops → {:.3} ({})",
        "✓".bright_green(),
        health_breakdown.final_weight,
        classify_weight(health_breakdown.final_weight).label()
    );
    // Pattern 3: active intent load
    let active_intents: i64 =
        std::fs::read_dir(std::path::PathBuf::from(&ctx.core_root).join("intents/future"))
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .filter(|e| {
                        std::fs::read_to_string(e.path())
                            .map(|c| c.contains("status: in-progress"))
                            .unwrap_or(false)
                    })
                    .count() as i64
            })
            .unwrap_or(0);
    let intent_rate = (active_intents as f64 / 8.0).min(1.0);
    let intent_metrics = PatternMetrics {
        frequency: intent_rate,
        recency: 1.0,
        consequence: 0.5,
        trend: if active_intents > 4 { 0.3 } else { -0.1 },
        volatility: 0.3,
        confidence: 0.95,
    };
    let intent_breakdown = update_pattern_weight(
        ctx,
        "intent-load",
        "Active intent load (focus risk)",
        "work_rhythm",
        &intent_metrics,
        0.0,
        active_intents <= 3,
    )?;
    println!(
        "  {} intent-load → {:.3} ({})",
        "✓".bright_green(),
        intent_breakdown.final_weight,
        classify_weight(intent_breakdown.final_weight).label()
    );
    // Pattern 4: prediction accuracy
    let total_preds: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM forest_predictions WHERE created_at > ?1",
            params![now - window_secs],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let correct_preds: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM prediction_outcomes WHERE correct = 1 AND verified_at > ?1",
            params![now - window_secs],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let accuracy = if total_preds > 0 {
        correct_preds as f64 / total_preds as f64
    } else {
        0.5
    };
    let pred_metrics = PatternMetrics {
        frequency: (total_preds as f64 / (window_days * 2.0)).min(1.0),
        recency: 0.7,
        consequence: 0.6,
        trend: if accuracy > 0.7 { -0.3 } else { 0.3 },
        volatility: 0.2,
        confidence: accuracy,
    };
    let pred_breakdown = update_pattern_weight(
        ctx,
        "prediction-accuracy",
        "Prediction engine accuracy",
        "prediction",
        &pred_metrics,
        0.0,
        accuracy > 0.7,
    )?;
    println!(
        "  {} prediction-accuracy → {:.3} ({})",
        "✓".bright_green(),
        pred_breakdown.final_weight,
        classify_weight(pred_breakdown.final_weight).label()
    );
    println!();
    println!(
        "  {} Weights computed. Run: core weight list to see rankings",
        "✅".green()
    );
    println!(
        "  {} Run: core weight explain <id> for full breakdown",
        "→".bright_cyan()
    );
    println!();
    Ok(())
}
/// core weight top — show top Critical and Strong patterns
pub fn top(ctx: &AppContext) -> CoreResult<()> {
    ensure_tables(ctx)?;
    println!();
    println!("{}", "🎯 High-Weight Patterns".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, description, context, final_weight, weight_class, is_positive
         FROM pattern_weights
         WHERE weight_class IN ('CRITICAL', 'STRONG')
         ORDER BY final_weight DESC",
    )?;
    let rows: Vec<(String, String, String, f64, String, i64)> = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    if rows.is_empty() {
        println!("  {} No high-weight patterns yet", "○".dimmed());
        println!(
            "  {} Run: core weight compute to build pattern weights",
            "→".dimmed()
        );
    } else {
        for (id, desc, ctx_name, weight, class, positive) in &rows {
            let marker = if *positive == 1 {
                "✨".to_string()
            } else {
                "⚠️ ".to_string()
            };
            let class_colored = match class.as_str() {
                "CRITICAL" => class.bright_red().bold(),
                _ => class.bright_green(),
            };
            println!(
                "  {} {} [{} | {}] {:.3}",
                marker,
                desc.bright_white(),
                class_colored,
                ctx_name.bright_cyan(),
                weight
            );
            println!("    {} core weight explain {}", "→".dimmed(), id.dimmed());
            println!();
        }
    }
    Ok(())
}
/// core weight calibrate <id> <outcome> — record outcome for calibration
pub fn calibrate(ctx: &AppContext, id: &str, outcome: &str) -> CoreResult<()> {
    ensure_tables(ctx)?;
    let pattern: Option<(f64, String)> = ctx
        .runtime
        .db
        .query_row(
            "SELECT final_weight, context FROM pattern_weights WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();
    match pattern {
        None => println!("  {} Pattern '{}' not found", "⚠️ ".yellow(), id),
        Some((weight, ctx_name)) => {
            let success = matches!(
                outcome.to_lowercase().as_str(),
                "success" | "correct" | "yes"
            );
            let now = now_ts();
            ctx.runtime.db.execute(
                "INSERT INTO weight_calibrations
                 (pattern_id, context, predicted_importance, actual_outcome, was_correct, calibrated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, ctx_name, weight, outcome, success as i64, now],
            )?;
            println!(
                "  {} Calibration recorded for '{}': outcome={}, was_correct={}",
                "✅".green(),
                id,
                outcome,
                success
            );
            println!("  {} The forest learns from this outcome", "→".dimmed());
        }
    }
    Ok(())
}
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
