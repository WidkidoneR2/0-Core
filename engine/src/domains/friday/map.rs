//! INT-244 v22 -- Pillar 4: System Cartographer
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
pub fn update(ctx: &AppContext) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let ts = now_ts();
    let registry_path = std::path::PathBuf::from(&ctx.core_root).join("registry/tools.toml");
    if let Ok(content) = std::fs::read_to_string(&registry_path) {
        for line in content.lines() {
            if let Some(name) = line.trim().strip_prefix("name = ") {
                let name = name.trim().trim_matches('"');
                if !name.is_empty() {
                    db.execute(
                        "INSERT INTO friday_map (entity_type, entity_name, updated_at)
                         VALUES ('tool', ?1, ?2)
                         ON CONFLICT(entity_type, entity_name) DO UPDATE SET updated_at=?2",
                        params![name, ts],
                    )?;
                }
            }
        }
    }
    println!("  {} System map updated", "✅".green());
    Ok(())
}
pub fn show(ctx: &AppContext) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let mut stmt = db.prepare(
        "SELECT entity_type, entity_name, version, status, health
         FROM friday_map ORDER BY entity_type, entity_name"
    )?;
    let rows: Vec<(String, String, String, String, f64)> = stmt.query_map(
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    )?.filter_map(|r| r.ok()).collect();
    if rows.is_empty() {
        println!("  Map is empty. Run: core friday map-update");
        return Ok(());
    }
    println!("  🗺  System Map", );
    println!("{}", "─".repeat(48).dimmed());
    let mut current_type = String::new();
    for (etype, name, version, status, health) in &rows {
        if *etype != current_type {
            println!("  {} {}", "▼".dimmed(), etype.bright_white().bold());
            current_type = etype.clone();
        }
        let health_str = if *health >= 95.0 { format!("{:.0}%", health).green().to_string() }
            else if *health >= 80.0 { format!("{:.0}%", health).yellow().to_string() }
            else { format!("{:.0}%", health).red().to_string() };
        let ver = if version.is_empty() { String::new() } else { format!(" v{}", version) };
        println!("    ▸ {}{} [{}] {}", name.bright_white(), ver.dimmed(), status.dimmed(), health_str);
    }
    Ok(())
}

pub fn impact(ctx: &AppContext, change: &str) -> CoreResult<()> {
    use colored::*;
    let db = &ctx.runtime.db;
    let change_lower = change.to_lowercase();

    // Find the entity being changed
    let matches: Vec<(String, String, String)> = db.prepare(
        "SELECT entity_type, entity_name, depends_on FROM friday_map WHERE entity_name LIKE ?1"
    ).ok().and_then(|mut s| {
        s.query_map(rusqlite::params![format!("%{}%", change_lower)], |r| {
            Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?))
        }).ok().map(|rows| rows.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    // Find what depends ON this entity
    let all_tools: Vec<(String, String)> = db.prepare(
        "SELECT entity_name, depends_on FROM friday_map WHERE depends_on != '[]'"
    ).ok().and_then(|mut s| {
        s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?)))
            .ok().map(|rows| rows.filter_map(|r| r.ok()).collect())
    }).unwrap_or_default();

    println!();
    println!("  {} Impact: {}", "🗺".normal(), change.bright_white());
    println!("  {}", "─".repeat(50).dimmed());
    println!();

    if matches.is_empty() {
        println!("  {} Not found in system map.", "·".dimmed());
        println!("  → Run: core friday map-update to refresh");
        println!();
        return Ok(());
    }

    for (etype, name, _deps) in &matches {
        println!("  {} {} [{}]", "▸".bright_cyan(), name.bright_white(), etype.dimmed());
    }
    println!();

    // Find downstream dependents
    let mut dependents: Vec<String> = Vec::new();
    for (tool_name, deps_json) in &all_tools {
        if let Ok(deps) = serde_json::from_str::<Vec<String>>(deps_json) {
            if deps.iter().any(|d| d.to_lowercase().contains(&change_lower)) {
                dependents.push(tool_name.clone());
            }
        }
    }

    if dependents.is_empty() {
        println!("  {} No downstream dependents found.", "·".dimmed());
    } else {
        println!("  {} Downstream impact ({} tool(s)):", "⚠".bright_yellow(), dependents.len());
        for dep in &dependents {
            println!("    {} {} -- redeploy after changing {}", "→".bright_red(), dep.bright_white(), change.dimmed());
        }
    }
    println!();
    Ok(())
}
