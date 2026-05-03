//! INT-244 v22 -- Pillar 5: Self-Review
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
pub fn run(ctx: &AppContext) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    println!("  🧠 Friday Self-Review");
    println!("{}", "─".repeat(48).dimmed());
    // Prediction accuracy
    let (total, correct): (i64, i64) = db.query_row(
        "SELECT COUNT(*), SUM(CASE WHEN correct=1 THEN 1 ELSE 0 END)
         FROM friday_hypotheses WHERE resolved_at IS NOT NULL",
        [],
        |r| Ok((r.get(0)?, r.get::<_, Option<i64>>(1)?.unwrap_or(0)))
    ).unwrap_or((0, 0));
    let accuracy = if total > 0 { (correct as f64 / total as f64) * 100.0 } else { 0.0 };
    println!("  🎯 Predictions: {}/{} correct ({:.0}%)", correct, total, accuracy);
    // Decisions recorded
    let decision_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_decisions", [],
        |r| r.get(0)
    ).unwrap_or(0);
    println!("  📝 Decisions recorded: {}", decision_count);
    // Knowledge base
    let knowledge_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_knowledge", [],
        |r| r.get(0)
    ).unwrap_or(0);
    println!("  📚 Knowledge entries: {}", knowledge_count);
    // High-confidence patterns
    let pattern_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM friday_patterns WHERE confidence >= 0.7", [],
        |r| r.get(0)
    ).unwrap_or(0);
    println!("  📊 High-confidence patterns: {}", pattern_count);
    // Calibration assessment
    println!();
    if total > 10 {
        if accuracy >= 80.0 {
            println!("  {} Confidence well-calibrated ({:.0}%)", "✅".green(), accuracy);
        } else {
            println!("  {} Needs recalibration ({:.0}% accuracy)", "⚠️ ".yellow(), accuracy);
        }
    } else {
        println!("  {} Not enough data to calibrate ({} resolved)", "ℹ️ ".cyan(), total);
    }
    println!("{}", "─".repeat(48).dimmed());
    println!("  💡 core friday why <topic> -- query decision record");
    println!("  💡 core friday decisions   -- list all decisions");
    Ok(())
}
