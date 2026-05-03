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
    let registry_path = std::path::PathBuf::from(&ctx.core_root).join("01-registry/tools.toml");
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
