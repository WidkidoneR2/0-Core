//! autobiography domain — the forest narrates its own goal history (Core v9 Phase 5)
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

fn read_version(ctx: &AppContext) -> String {
    let root = &ctx.core_root;
    std::fs::read_to_string(
        std::path::PathBuf::from(&root).join("00-meta/VERSION")
    ).unwrap_or_else(|_| "unknown".into()).trim().to_string()
}

fn read_theme(ctx: &AppContext) -> String {
    let root = &ctx.core_root;
    let version = read_version(ctx);
    let changelog = std::fs::read_to_string(
        std::path::PathBuf::from(&root).join("00-meta/CHANGELOG.md")
    ).unwrap_or_default();
    changelog.lines()
        .find(|l| l.starts_with(&format!("## [{}]", version)))
        .and_then(|l| l.split(" — ").nth(1))
        .and_then(|s| s.split('(').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "The Living Forest".to_string())
}

fn format_ts(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn status_narrative(status: &str) -> &'static str {
    match status {
        "accepted" => "authorized and in motion",
        "rejected" => "considered and set aside",
        _          => "proposed, awaiting decision",
    }
}

pub fn narrate(ctx: &AppContext, version_filter: Option<&str>) -> CoreResult<()> {
    let version = read_version(ctx);
    let theme   = read_theme(ctx);

    // Load all goals — filter by version if requested
    let mut stmt = match ctx.runtime.db.prepare(
        "SELECT id, title, reason, plan, priority, status, created_at, updated_at \
         FROM forest_goals ORDER BY created_at ASC"
    ) {
        Ok(s) => s,
        Err(_) => {
            println!();
            println!("  No goals recorded yet — run: core goals generate");
            println!();
            return Ok(());
        }
    };

    let goals: Vec<(String,String,String,String,String,String,i64,i64)> = stmt
        .query_map([], |r| Ok((
            r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
            r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?,
        )))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    if goals.is_empty() {
        println!();
        println!("  The forest has not yet set its intentions.");
        println!("  Run: core goals generate — let the forest speak.");
        println!();
        return Ok(());
    }

    // Count totals
    let total        = goals.len();
    let accepted     = goals.iter().filter(|g| g.5 == "accepted").count();
    let rejected     = goals.iter().filter(|g| g.5 == "rejected").count();
    let pending      = total - accepted - rejected;
    let total_commits: String = std::fs::read_to_string("/etc/faelight/COMMITS")
        .unwrap_or_default().trim().to_string();

    let display_version = version_filter.unwrap_or(&version);

    println!();
    println!("  {} {}", "📖  Forest Autobiography".bright_cyan().bold(),
        format!("v{}", display_version).dimmed());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    // Opening narrative
    println!("  {} — {}", display_version.bright_white().bold(), theme.bright_green());
    println!();
    println!("  {}", "The forest set its intentions.".dimmed().italic());
    println!("  {} goals proposed  ·  {} authorized  ·  {} set aside  ·  {} pending",
        total.to_string().bright_white(),
        accepted.to_string().bright_green(),
        rejected.to_string().dimmed(),
        pending.to_string().yellow()
    );
    if !total_commits.is_empty() {
        println!("  {} commits in the forest's memory", total_commits.bright_white());
    }
    println!();

    // Narrate each goal
    for (idx, (id, title, reason, plan, priority, status, created, updated)) in goals.iter().enumerate() {
        let date_created = format_ts(*created);
        let date_updated = format_ts(*updated);
        let narrative    = status_narrative(status);

        // Load plan if exists
        let plan_steps: Option<Vec<String>> = ctx.runtime.db
            .query_row(
                "SELECT steps FROM forest_plans WHERE goal_id=?1 LIMIT 1",
                rusqlite::params![id],
                |r| r.get::<_,String>(0),
            )
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());

        // Load tradeoff if exists
        let tradeoff_rec: Option<String> = ctx.runtime.db
            .query_row(
                "SELECT recommendation FROM forest_tradeoffs WHERE linked_goal=?1 LIMIT 1",
                rusqlite::params![id],
                |r| r.get(0),
            ).ok();

        // Priority color
        let pri_colored = match priority.as_str() {
            "HIGH"   => priority.bright_red(),
            "MEDIUM" => priority.yellow(),
            _        => priority.dimmed(),
        };

        // Status color
        let status_colored = match status.as_str() {
            "accepted" => "[authorized]".bright_green(),
            "rejected" => "[set aside] ".dimmed(),
            _          => "[pending]   ".yellow(),
        };

        println!("{}", "─".repeat(60).dimmed());
        println!("  {} {}  {} {}",
            format!("Goal {}", idx + 1).bright_cyan().bold(),
            id.yellow(),
            status_colored,
            pri_colored
        );
        println!("  {}", title.bright_white().bold());
        println!();
        println!("  {} {}", "Why the forest wanted this:".dimmed(), reason.italic());
        println!("  {} {}", "Originally planned:".dimmed(), plan.dimmed());
        println!("  {} {}  → {}",
            "Timeline:".dimmed(),
            date_created.bright_white(),
            if date_updated != date_created { date_updated.clone() } else { "same day".to_string() }.dimmed()
        );
        println!("  {} {}", "Status:".dimmed(), narrative.bright_white());
        println!();

        // Plan steps if available
        if let Some(steps) = &plan_steps {
            println!("  {} ({} steps)",
                "Concrete path:".bright_white().bold(), steps.len());
            for (i, step) in steps.iter().take(3).enumerate() {
                println!("    {}. {}", i + 1, step.dimmed());
            }
            if steps.len() > 3 {
                println!("    {} {} more steps...", "+".dimmed(),
                    (steps.len() - 3).to_string().dimmed());
            }
            println!();
        }

        // Tradeoff if available
        if let Some(rec) = &tradeoff_rec {
            println!("  {} {}", "Tradeoff weighed:".bright_white().bold(), rec.dimmed().italic());
            println!();
        }
    }

    println!("{}", "━".repeat(60).dimmed());
    println!();

    // Closing narrative
    let closing = if accepted == total {
        "Every intention was authorized. The forest moved with purpose."
    } else if accepted == 0 {
        "No intentions authorized yet. The forest waits for direction."
    } else {
        "Some intentions authorized. The forest grows selectively."
    };
    println!("  {}", closing.bright_white().italic());
    println!();
    println!("  {} {}",
        "Next:".dimmed(),
        "core goals generate  — let the forest propose new intentions".bright_cyan()
    );
    println!();

    // Emit event
    let now = chrono::Utc::now().timestamp();
    let _ = ctx.runtime.db.execute(
        "INSERT INTO events (domain,action,payload,timestamp) \
         VALUES ('autobiography','narrated',?1,?2)",
        rusqlite::params![format!("version:{} goals:{}", display_version, total), now],
    );

    Ok(())
}
