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
    let registry_path = ctx.fpath("registry/tools.toml");
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
         FROM friday_map ORDER BY entity_type, entity_name",
    )?;
    let rows: Vec<(String, String, String, String, f64)> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .filter_map(|r| r.ok())
        .collect();
    if rows.is_empty() {
        println!("  Map is empty. Run: core friday map-update");
        return Ok(());
    }
    println!("  🗺  System Map",);
    println!("{}", "─".repeat(48).dimmed());
    let mut current_type = String::new();
    for (etype, name, version, status, health) in &rows {
        if *etype != current_type {
            println!("  {} {}", "▼".dimmed(), etype.bright_white().bold());
            current_type = etype.clone();
        }
        let health_str = if *health >= 95.0 {
            format!("{:.0}%", health).green().to_string()
        } else if *health >= 80.0 {
            format!("{:.0}%", health).yellow().to_string()
        } else {
            format!("{:.0}%", health).red().to_string()
        };
        let ver = if version.is_empty() {
            String::new()
        } else {
            format!(" v{}", version)
        };
        println!(
            "    ▸ {}{} [{}] {}",
            name.bright_white(),
            ver.dimmed(),
            status.dimmed(),
            health_str
        );
    }
    Ok(())
}

pub fn impact(ctx: &AppContext, change: &str) -> CoreResult<()> {
    use colored::*;
    let db = &ctx.runtime.db;
    let change_lower = change.to_lowercase();

    // Find the entity being changed
    let matches: Vec<(String, String, String)> = db
        .prepare(
            "SELECT entity_type, entity_name, depends_on FROM friday_map WHERE entity_name LIKE ?1",
        )
        .ok()
        .and_then(|mut s| {
            s.query_map(rusqlite::params![format!("%{}%", change_lower)], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .ok()
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();

    // Find what depends ON this entity
    let all_tools: Vec<(String, String)> = db
        .prepare("SELECT entity_name, depends_on FROM friday_map WHERE depends_on != '[]'")
        .ok()
        .and_then(|mut s| {
            s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();

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
        println!(
            "  {} {} [{}]",
            "▸".bright_cyan(),
            name.bright_white(),
            etype.dimmed()
        );
    }
    println!();

    // Find downstream dependents
    let mut dependents: Vec<String> = Vec::new();
    for (tool_name, deps_json) in &all_tools {
        if let Ok(deps) = serde_json::from_str::<Vec<String>>(deps_json) {
            if deps
                .iter()
                .any(|d| d.to_lowercase().contains(&change_lower))
            {
                dependents.push(tool_name.clone());
            }
        }
    }

    if dependents.is_empty() {
        println!("  {} No downstream dependents found.", "·".dimmed());
    } else {
        println!(
            "  {} Downstream impact ({} tool(s)):",
            "⚠".bright_yellow(),
            dependents.len()
        );
        for dep in &dependents {
            println!(
                "    {} {} -- redeploy after changing {}",
                "→".bright_red(),
                dep.bright_white(),
                change.dimmed()
            );
        }
    }
    println!();
    Ok(())
}

// INT-316 G3. Intent ids from `git log --name-only --grep=INT-NNN` whose
// commits touch `file`. Red stub: returns nothing.
#[cfg(test)]
fn impact_intents(repo: &std::path::Path, file: &str) -> Vec<String> {
    let _ = (repo, file);
    Vec::new()
}

// INT-316 G3. Tool, crate, and intent tree. Tools cyan, crates green, intents
// magenta, and a spark on anything changed in the last 24 hours.
// Red stub: empty.
#[cfg(test)]
fn render_map(repo: &std::path::Path) -> String {
    let _ = repo;
    String::new()
}

#[cfg(test)]
mod int316 {
    use super::{impact_intents, render_map};

    struct TempDir(std::path::PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn git(dir: &std::path::Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .status()
            .unwrap();
        assert!(status.success(), "{args:?}");
    }

    // A temp repo whose tree names a tool and a crate, and whose history
    // mentions INT-310 and INT-314 on the same file.
    fn repo_touched_by_two_intents() -> TempDir {
        let dir = std::env::temp_dir().join(format!(
            "int316-map-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("zero/mapcrate/src/lib.rs");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::create_dir_all(dir.join("registry")).unwrap();
        std::fs::write(dir.join("registry/tools.toml"), "[[tool]]\nname = \"maptool\"\n").unwrap();
        std::fs::write(
            dir.join("zero/mapcrate/Cargo.toml"),
            "[package]\nname = \"mapcrate\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(&file, "one\n").unwrap();
        git(&dir, &["init"]);
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-m", "INT-310: one reader"]);
        std::fs::write(&file, "two\n").unwrap();
        git(&dir, &["add", "zero/mapcrate/src/lib.rs"]);
        git(&dir, &["commit", "-m", "INT-314: tells the truth"]);
        TempDir(dir)
    }

    struct ColorOverride;

    impl ColorOverride {
        fn on() -> Self {
            colored::control::set_override(true);
            Self
        }
    }

    impl Drop for ColorOverride {
        fn drop(&mut self) {
            colored::control::unset_override();
        }
    }

    fn painted(text: &str, name: &str, code: &str) -> bool {
        let mut rest = text;
        while let Some(at) = rest.find(name) {
            let prefix = &rest[..at];
            if let Some(i) = prefix.rfind('\u{1b}') {
                let seq = &prefix[i..];
                let closed = seq.contains("\u{1b}[0m");
                if seq.contains(code) && !closed {
                    return true;
                }
            }
            rest = &rest[at + name.len()..];
        }
        false
    }

    #[test]
    fn g3_impact_of_a_file_returns_int_310_and_int_314() {
        let repo = repo_touched_by_two_intents();
        let file = "zero/mapcrate/src/lib.rs";
        let mut got = impact_intents(&repo.0, file);
        got.sort();
        assert_eq!(
            got,
            vec!["INT-310".to_string(), "INT-314".to_string()],
            "impact of {file} should name both intents"
        );
    }

    #[test]
    fn g3_map_colors_tools_crates_intents_and_marks_recent() {
        let repo = repo_touched_by_two_intents();
        let _color = ColorOverride::on();
        let text = render_map(&repo.0);
        drop(_color);
        let tool = painted(&text, "maptool", "36");
        let krate = painted(&text, "mapcrate", "32");
        let intent = painted(&text, "INT-310", "35");
        let recent = text.contains('✦');
        assert!(
            tool && krate && intent && recent,
            "colored tree: tool maptool cyan={tool}, crate mapcrate green={krate}, intent INT-310 magenta={intent}, spark on the last 24h={recent}; got {text:?}"
        );
    }
}
