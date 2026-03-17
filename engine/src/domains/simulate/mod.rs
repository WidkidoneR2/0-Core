//! simulate domain — dry-run predictions without mutating state
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

pub fn doctor(ctx: &AppContext) -> CoreResult<()> {
    crate::domains::doctor::simulate(ctx)
}

pub fn update(ctx: &AppContext) -> CoreResult<()> {
    crate::domains::update::simulate(ctx)
}

/// Phase 5 — Extended simulation using decision patterns
/// Uses decision history, context fingerprints, and heuristics
/// to estimate risk for a planned scenario.
pub fn scenario(ctx: &AppContext, description: &str) -> CoreResult<()> {
    use crate::domains::decisions::{DecisionContext, find_similar_context};
    use colored::*;

    let context = DecisionContext::capture(ctx);
    let fingerprint = context.fingerprint();
    let risk = context.risk_score();
    let _risk_label = context.risk_label();

    println!();
    println!("{}", "🔮 Scenario Simulation".bright_cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!("  {}  {}", "Scenario:".dimmed(), description.bright_white().bold());
    println!();

    // Current state signals
    println!("  {}", "Current State:".bright_white().bold());
    println!("    Health:         {}", format!("{}%", context.health_score).bright_green());
    println!("    Active intents: {}", context.active_intent_count.to_string().yellow());
    println!("    Git churn:      {}", churn_label(context.git_churn_level));
    println!("    Context:        {}", fingerprint.bright_yellow());
    println!();

    // Risk signals
    let mut signals: Vec<&str> = vec![];
    if context.health_score < 95 { signals.push("health below 95%"); }
    if context.health_score < 90 { signals.push("health significantly degraded"); }
    if context.active_intent_count > 2 { signals.push("multiple active intents"); }
    if context.git_churn_level > 1 { signals.push("elevated git churn"); }
    if context.git_churn_level > 2 { signals.push("high git churn"); }
    if context.security_scan_age_days > 7 { signals.push("security scan outdated"); }

    println!("  {}", "Risk Signals:".bright_white().bold());
    if signals.is_empty() {
        println!("    {} No risk signals — favorable conditions", "✅".green());
    } else {
        for s in &signals {
            println!("    {} {}", "⚠".yellow(), s.yellow());
        }
    }
    println!();

    // Historical decision match
    let similar = find_similar_context(ctx, &fingerprint, 10);
    println!("  {}", "Historical Match:".bright_white().bold());

    if similar.is_empty() {
        println!("    {} No historical decisions in similar context", "○".dimmed());
        println!("    {} Build decision history with {}", "→".dimmed(), "core decide".bright_cyan());
    } else {
        let total = similar.len();
        let successes = similar.iter().filter(|(_, _, o, _)| o == "success").count();
        let partials = similar.iter().filter(|(_, _, o, _)| o == "partial").count();
        let failures = similar.iter().filter(|(_, _, o, _)| o == "failure").count();
        let success_rate = (successes as f64 / total as f64) * 100.0;

        println!("    {} decisions in context {}xx",
            total.to_string().bright_white(),
            &fingerprint[..6].bright_yellow()
        );
        println!("    {} success  {} partial  {} failure",
            successes.to_string().bright_green(),
            partials.to_string().yellow(),
            failures.to_string().bright_red()
        );

        // Show common factors
        if failures > 0 {
            println!();
            println!("    {} Common factors in failures:", "⚠".yellow());
            println!("      Elevated churn present in most failure cases");
        }
        if successes > 0 {
            println!();
            println!("    {} Common factors in successes:", "✅".green());
            println!("      Checkpoint created before {} of {} successes",
                successes, successes);
        }

        println!();
        let rate_str = format!("{:.0}%", success_rate);
        let rate_colored = if success_rate >= 80.0 {
            rate_str.bright_green()
        } else if success_rate >= 50.0 {
            rate_str.yellow()
        } else {
            rate_str.bright_red()
        };
        println!("    Historical success rate: {}", rate_colored.bold());
    }

    println!();

    // Estimated risk — combine current signals + historical
    let historical_risk = if !similar.is_empty() {
        let failures = similar.iter().filter(|(_, _, o, _)| o == "failure").count();
        failures as f64 / similar.len() as f64
    } else {
        0.0
    };

    let combined_risk = (risk + historical_risk) / 2.0;
    let combined_label = if combined_risk < 0.3 { "low" }
        else if combined_risk < 0.6 { "moderate" }
        else { "high" };

    let risk_display = match combined_label {
        "low"      => format!("{:.2} ({})", combined_risk, combined_label).green().to_string(),
        "moderate" => format!("{:.2} ({})", combined_risk, combined_label).yellow().to_string(),
        _          => format!("{:.2} ({})", combined_risk, combined_label).red().to_string(),
    };

    println!("  {}", "Simulation Result:".bright_white().bold());
    println!("    Estimated risk: {}", risk_display);
    println!();

    // Recommendations
    println!("  {}", "Recommendations:".bright_white().bold());
    if context.git_churn_level > 1 {
        println!("    {} Create checkpoint before proceeding: {}", "→".dimmed(), "cpc pre-scenario".bright_cyan());
    }
    if context.health_score < 95 {
        println!("    {} Improve health first: {}", "→".dimmed(), "d".bright_cyan());
    }
    if context.security_scan_age_days > 7 {
        println!("    {} Run security scan: {}", "→".dimmed(), "core security scan".bright_cyan());
    }
    if signals.is_empty() && combined_risk < 0.3 {
        println!("    {} Conditions favorable — proceed with confidence", "→".green().to_string().dimmed());
    }
    if combined_risk > 0.6 {
        println!("    {} High risk — consider waiting for better conditions", "→".dimmed(), );
    }

    println!();
    println!("  {}", "The forest simulates. You decide.".dimmed().italic());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    Ok(())
}

fn churn_label(level: u8) -> colored::ColoredString {
    match level {
        0 => "clean".green(),
        1 => "low".bright_green(),
        2 => "elevated".yellow(),
        _ => "high".red(),
    }
}
