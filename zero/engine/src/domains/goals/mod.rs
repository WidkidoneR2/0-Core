use crate::app::context::AppContext;
use crate::errors::CoreResult;
use rusqlite::params;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS forest_goals (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    reason      TEXT NOT NULL,
    plan        TEXT NOT NULL,
    priority    TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);";

fn ensure_schema(ctx: &AppContext) {
    let _ = ctx.runtime.db.execute_batch(SCHEMA);
}

fn next_id(ctx: &AppContext) -> String {
    let count: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM forest_goals", [], |r| r.get(0))
        .unwrap_or(0);
    format!("GOAL-{:03}", count + 1)
}

fn read_health(_ctx: &AppContext) -> u32 {
    std::fs::read_to_string(zero_core::paths::health_cache())
        .unwrap_or_else(|_| "95".to_string())
        .trim()
        .trim_end_matches('%')
        .parse()
        .unwrap_or(95)
}

pub fn list(ctx: &AppContext) -> CoreResult<()> {
    ensure_schema(ctx);
    let db = &ctx.runtime.db;
    let mut stmt = match db.prepare(
        "SELECT id, title, priority, status, reason FROM forest_goals ORDER BY \
         CASE priority WHEN 'HIGH' THEN 1 WHEN 'MEDIUM' THEN 2 ELSE 3 END, created_at",
    ) {
        Ok(s) => s,
        Err(_) => {
            println!("  No goals yet — run: core goals generate");
            return Ok(());
        }
    };
    let goals: Vec<(String, String, String, String, String)> = stmt
        .query_map(
            [],
            |r| -> rusqlite::Result<(String, String, String, String, String)> {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            },
        )
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();
    println!();
    if goals.is_empty() {
        println!("  No goals yet — run: core goals generate");
    } else {
        for (id, title, priority, status, reason) in &goals {
            let pi = match priority.as_str() {
                "HIGH" => "HIGH",
                "MEDIUM" => "MED",
                _ => "LOW",
            };
            let si = match status.as_str() {
                "accepted" => "[OK]",
                "rejected" => "[NO]",
                _ => "[..]",
            };
            println!("  {} {} {}  {}", si, pi, id, title);
            println!("     {}", reason);
        }
    }
    Ok(())
}

pub fn generate(ctx: &AppContext) -> CoreResult<()> {
    ensure_schema(ctx);
    println!();
    println!("  Analyzing evidence...");
    println!();
    let health = read_health(ctx);
    let mut proposed: Vec<(String, String, String, String)> = vec![];
    if health < 95 {
        proposed.push((
            "Restore forest health to 95%+".to_string(),
            format!("Health is {}% — below threshold", health),
            "Run: d — review warnings".to_string(),
            "HIGH".to_string(),
        ));
    }
    proposed.push((
        "Retire redundant tools absorbed by core/shell".to_string(),
        "workspace-view replaced by shell pipelines".to_string(),
        "Run: core evolution tools — review dormant tools".to_string(),
        "LOW".to_string(),
    ));
    println!("  {} goal(s) proposed from evidence:", proposed.len());
    println!();
    let now = chrono::Utc::now().timestamp();
    for (title, reason, plan, priority) in &proposed {
        let id = next_id(ctx);
        println!("  [{}] {}  {}", priority, id, title);
        println!("  Reason: {}", reason);
        println!("  Plan:   {}", plan);
        println!("  Accept: core goals accept {}", id);
        println!();
        // Dedup by title — prevent repeated generate runs inserting duplicates
        let already: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM forest_goals WHERE title=?1",
                params![title],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if already > 0 {
            println!("  [skip] already exists: {}", title);
            continue;
        }
        let _ = ctx.runtime.db.execute(
            "INSERT INTO forest_goals \
             (id,title,reason,plan,priority,status,created_at,updated_at) \
             VALUES (?1,?2,?3,?4,?5,'pending',?6,?6)",
            params![id, title, reason, plan, priority, now],
        );
    }
    println!("  Run: core goals accept <id>  to authorize a goal");
    Ok(())
}

pub fn priority(ctx: &AppContext) -> CoreResult<()> {
    let health = read_health(ctx);
    println!(
        "  Health: {}%  Gate: {}",
        health,
        if health >= 95 {
            "expansion enabled"
        } else {
            "stability goals only"
        }
    );
    list(ctx)
}

pub fn accept(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_schema(ctx);
    let now = chrono::Utc::now().timestamp();
    let updated = ctx
        .runtime
        .db
        .execute(
            "UPDATE forest_goals SET status='accepted', updated_at=?1 WHERE id=?2",
            params![now, id],
        )
        .map_err(|e| crate::errors::CoreError::Runtime(e.to_string()))?;
    if updated == 0 {
        println!("  Goal not found: {}", id);
        return Ok(());
    }
    let (title, plan): (String, String) = ctx
        .runtime
        .db
        .query_row(
            "SELECT title, plan FROM forest_goals WHERE id=?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| crate::errors::CoreError::Runtime(e.to_string()))?;
    println!("  Goal accepted: {}", id);
    println!("  Title: {}", title);
    println!("  Plan:  {}", plan);
    let _ = ctx.runtime.db.execute(
        "INSERT INTO events (domain,action,payload,timestamp) VALUES ('goals','accepted',?1,?2)",
        params![format!("goal:{}", id), now],
    );
    Ok(())
}

pub fn reject(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_schema(ctx);
    let now = chrono::Utc::now().timestamp();
    let updated = ctx
        .runtime
        .db
        .execute(
            "UPDATE forest_goals SET status='rejected', updated_at=?1 WHERE id=?2",
            params![now, id],
        )
        .map_err(|e| crate::errors::CoreError::Runtime(e.to_string()))?;
    if updated == 0 {
        println!("  Goal not found: {}", id);
    } else {
        println!("  Goal rejected: {}", id);
    }
    Ok(())
}

pub fn show(ctx: &AppContext, id: &str) -> CoreResult<()> {
    ensure_schema(ctx);
    let r = ctx.runtime.db.query_row(
        "SELECT id,title,reason,plan,priority,status,created_at FROM forest_goals WHERE id=?1",
        params![id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, i64>(6)?,
            ))
        },
    );
    match r {
        Ok((id, title, reason, plan, priority, status, created)) => {
            let date = chrono::DateTime::from_timestamp(created, 0)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            println!("  Goal:     {}", id);
            println!("  Title:    {}", title);
            println!("  Reason:   {}", reason);
            println!("  Plan:     {}", plan);
            println!("  Priority: {}", priority);
            println!("  Status:   {}", status);
            println!("  Created:  {}", date);
        }
        Err(_) => println!("  Not found: {}", id),
    }
    Ok(())
}
