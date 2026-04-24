// INT-183 — Registry Domain
// Manages tools.toml — retire, unretire, list, show
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;

fn registry_path(ctx: &AppContext) -> std::path::PathBuf {
    std::path::PathBuf::from(&ctx.core_root).join("01-registry/tools.toml")
}

fn read_registry(ctx: &AppContext) -> CoreResult<String> {
    let path = registry_path(ctx);
    Ok(std::fs::read_to_string(&path)?)
}

fn write_registry(ctx: &AppContext, content: &str) -> CoreResult<()> {
    let path = registry_path(ctx);
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn list(ctx: &AppContext) -> CoreResult<()> {
    let content = read_registry(ctx)?;
    println!();
    println!("  {} Tool Registry", "📋".normal());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    let mut current_name = String::new();
    let mut current_type = String::new();
    let mut deployable = false;
    let mut retired = false;

    let mut active = 0;
    let mut retired_count = 0;
    let mut not_deployable = 0;

    let flush = |name: &str,
                 typ: &str,
                 dep: bool,
                 ret: bool,
                 active: &mut i32,
                 retired_c: &mut i32,
                 not_dep: &mut i32| {
        if name.is_empty() {
            return;
        }
        let status = if ret {
            *retired_c += 1;
            "🪦".to_string()
        } else if !dep {
            *not_dep += 1;
            "○".dimmed().to_string()
        } else {
            *active += 1;
            "✅".to_string()
        };
        println!("  {} {:<30} [{}]", status, name, typ.dimmed());
    };

    for line in content.lines() {
        let line = line.trim();
        if line == "[[tool]]" {
            flush(
                &current_name,
                &current_type,
                deployable,
                retired,
                &mut active,
                &mut retired_count,
                &mut not_deployable,
            );
            current_name.clear();
            current_type = "rust".to_string();
            deployable = false;
            retired = false;
        } else if let Some(v) = line.strip_prefix("name = \"") {
            current_name = v.trim_end_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("type = \"") {
            current_type = v.trim_end_matches('"').to_string();
        } else if line == "deployable = true" {
            deployable = true;
        } else if line == "retired = true" {
            retired = true;
        }
    }
    flush(
        &current_name,
        &current_type,
        deployable,
        retired,
        &mut active,
        &mut retired_count,
        &mut not_deployable,
    );

    println!();
    println!(
        "  {} active   {} not deployable   {} retired",
        active.to_string().bright_green(),
        not_deployable.to_string().dimmed(),
        retired_count.to_string().yellow()
    );
    println!();
    Ok(())
}

pub fn show(ctx: &AppContext, name: &str) -> CoreResult<()> {
    let content = read_registry(ctx)?;
    let mut in_block = false;
    let mut found = false;
    let mut block = String::new();

    for line in content.lines() {
        if line.trim() == "[[tool]]" {
            if in_block && found {
                break;
            }
            in_block = true;
            block.clear();
            found = false;
        }
        if in_block {
            block.push_str(line);
            block.push('\n');
            if line.contains(&format!("name = \"{}\"", name)) {
                found = true;
            }
        }
    }

    if !found {
        println!("  {} Tool not found: {}", "✗".bright_red(), name);
        return Ok(());
    }

    println!();
    println!("  {} {}", "📦".normal(), name.bright_white().bold());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    for line in block.lines() {
        if !line.trim().is_empty() && !line.trim().starts_with("[[") {
            println!("  {}", line.dimmed());
        }
    }
    println!();
    Ok(())
}

pub fn retire(ctx: &AppContext, name: &str) -> CoreResult<()> {
    let content = read_registry(ctx)?;

    // Find the tool block and set retired = true
    let target = format!("name = \"{}\"", name);
    if !content.contains(&target) {
        println!("  {} Tool not found: {}", "✗".bright_red(), name);
        return Ok(());
    }

    // Replace retired = false with retired = true in the block containing this tool
    let new_content = retire_in_block(&content, name, true);
    write_registry(ctx, &new_content)?;

    println!();
    println!(
        "  {} {} marked as retired",
        "🪦".normal(),
        name.bright_white()
    );
    println!("  {} deploy all will skip this tool", "→".dimmed());
    println!("  {} doctor path resilience will exclude it", "→".dimmed());
    println!(
        "  {} To restore: core registry unretire {}",
        "→".dimmed(),
        name
    );
    println!();
    Ok(())
}

pub fn unretire(ctx: &AppContext, name: &str) -> CoreResult<()> {
    let content = read_registry(ctx)?;

    let target = format!("name = \"{}\"", name);
    if !content.contains(&target) {
        println!("  {} Tool not found: {}", "✗".bright_red(), name);
        return Ok(());
    }

    let new_content = retire_in_block(&content, name, false);
    write_registry(ctx, &new_content)?;

    println!();
    println!(
        "  {} {} restored to active",
        "✅".normal(),
        name.bright_white()
    );
    println!("  {} deploy all will include this tool again", "→".dimmed());
    println!();
    Ok(())
}

fn retire_in_block(content: &str, name: &str, retired: bool) -> String {
    let target = format!("name = \"{}\"", name);
    let mut result = String::new();
    let mut in_target_block = false;
    let mut changed = false;

    for line in content.lines() {
        if line.trim() == "[[tool]]" {
            in_target_block = false;
        }
        if line.contains(&target) {
            in_target_block = true;
        }
        if in_target_block && line.trim().starts_with("retired = ") && !changed {
            result.push_str(&format!("retired = {}\n", retired));
            changed = true;
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

pub fn reality_check(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;
    // Load tools.toml
    let tools_path = std::path::PathBuf::from(&ctx.core_root).join("01-registry/tools.toml");
    let tools_str = std::fs::read_to_string(&tools_path).unwrap_or_default();
    let tools_val: toml::Value =
        toml::from_str(&tools_str).unwrap_or(toml::Value::Table(toml::map::Map::new()));
    let empty = vec![];
    let tools = tools_val
        .get("tool")
        .and_then(|t| t.as_array())
        .unwrap_or(&empty);
    // Get actual usage from forest_events (last 7 days)
    let window = chrono::Utc::now().timestamp() - 604800;
    let mut usage_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    if let Ok(mut stmt) = ctx.runtime.db.prepare(
        "SELECT domain, COUNT(*) as cnt FROM forest_events WHERE timestamp > ?1 GROUP BY domain",
    ) {
        let rows: Vec<(String, i64)> = stmt
            .query_map(rusqlite::params![window], |r| Ok((r.get(0)?, r.get(1)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default();
        for (domain, cnt) in rows {
            usage_map.insert(domain, cnt);
        }
    }
    println!();
    println!(
        "  {} Registry Reality Check — actual vs expected usage (7 days)",
        "🔍".normal()
    );
    println!("  {}", "─".repeat(60).dimmed());
    println!(
        "  {:<25} {:<10} {:<10} {}",
        "tool".dimmed(),
        "expected".dimmed(),
        "actual".dimmed(),
        "status".dimmed()
    );
    println!("  {}", "─".repeat(60).dimmed());
    let mut drift_count = 0;
    for tool in tools {
        let name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let expected = tool
            .get("expected_usage")
            .and_then(|v| v.as_str())
            .unwrap_or("low");
        let actual_count = usage_map.get(name).copied().unwrap_or(0);
        let actual_label = match actual_count {
            0 => "none",
            1..=3 => "low",
            4..=10 => "medium",
            _ => "high",
        };
        let status = if expected == actual_label || actual_count == 0 {
            "✅".to_string()
        } else {
            drift_count += 1;
            "⚠️ drift".to_string()
        };
        println!(
            "  {:<25} {:<10} {:<10} {}",
            name,
            expected.dimmed(),
            format!("{} ({}x)", actual_label, actual_count).bright_white(),
            status
        );
    }
    println!("  {}", "─".repeat(60).dimmed());
    if drift_count > 0 {
        println!(
            "  {} {} tools show usage drift vs registry expectation",
            "⚠️".normal(),
            drift_count.to_string().bright_yellow()
        );
    } else {
        println!("  {} All tools within expected usage range", "✅".normal());
    }
    println!();
    Ok(())
}
