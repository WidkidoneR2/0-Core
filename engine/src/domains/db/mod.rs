// INT-166 — state.db Backup and Recovery Domain
// The forest protects its own memory.

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::path::PathBuf;

fn db_path(ctx: &AppContext) -> PathBuf {
    PathBuf::from(&ctx.core_root).join("runtime/state.db")
}

fn backups_dir(ctx: &AppContext) -> PathBuf {
    PathBuf::from(&ctx.core_root).join("runtime/backups")
}

fn fmt_size(bytes: u64) -> String {
    if bytes > 1_000_000 {
        format!("{:.1}MB", bytes as f64 / 1_000_000.0)
    } else if bytes > 1_000 {
        format!("{:.1}KB", bytes as f64 / 1_000.0)
    } else {
        format!("{}B", bytes)
    }
}

/// core db backup — manual snapshot to timestamped file
pub fn backup(ctx: &AppContext) -> CoreResult<()> {
    let db = db_path(ctx);
    let backups = backups_dir(ctx);
    std::fs::create_dir_all(&backups)?;

    let ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let dest = backups.join(format!("state-{}.db", ts));

    std::fs::copy(&db, &dest)?;

    // Also update the .bak rolling backup
    let bak = db.with_extension("db.bak");
    std::fs::copy(&db, &bak)?;

    // Keep only last 8 backups
    let mut entries: Vec<_> = std::fs::read_dir(&backups)?
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("state-"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    if entries.len() > 8 {
        for old in &entries[..entries.len() - 8] {
            let _ = std::fs::remove_file(old.path());
        }
    }

    let size = std::fs::metadata(&dest)?.len();

    println!();
    println!("  {}", "🌲 DB — Backup".bright_green().bold());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();
    println!("  {} Backup created:", "✅".bright_green());
    println!("    {} {}", "→".bright_green(), dest.display());
    println!("    {} Size: {}", "·".dimmed(), fmt_size(size));
    println!("    {} Rolling .bak updated", "·".dimmed());
    println!();

    Ok(())
}

/// core db restore <file> — restore from a backup snapshot
pub fn restore(ctx: &AppContext, file: &str) -> CoreResult<()> {
    let backups = backups_dir(ctx);
    let db = db_path(ctx);

    // Try as full path first, then as filename in backups dir
    let src = if std::path::Path::new(file).exists() {
        PathBuf::from(file)
    } else {
        backups.join(file)
    };

    if !src.exists() {
        println!();
        println!("  {} File not found: {}", "❌".bright_red(), file);
        println!(
            "  {} Run: core db status — to see available backups",
            "→".bright_green()
        );
        println!();
        return Ok(());
    }

    // Safety: backup current before restoring
    let ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let safety = backups.join(format!("pre-restore-{}.db", ts));
    std::fs::copy(&db, &safety)?;

    std::fs::copy(&src, &db)?;

    println!();
    println!("  {}", "🌲 DB — Restore".bright_green().bold());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();
    println!("  {} Restored from: {}", "✅".bright_green(), src.display());
    println!("  {} Safety backup: {}", "·".dimmed(), safety.display());
    println!(
        "  {} Restart core to use restored database",
        "⚠".bright_yellow()
    );
    println!();

    Ok(())
}

/// core db verify — integrity check
pub fn verify(ctx: &AppContext) -> CoreResult<()> {
    let result: String = ctx
        .runtime
        .db
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))?;

    let journal: String = ctx
        .runtime
        .db
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))?;

    println!();
    println!("  {}", "🌲 DB — Verify".bright_green().bold());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    if result == "ok" {
        println!(
            "  {} Integrity check: {}",
            "✅".bright_green(),
            "ok".bright_green()
        );
    } else {
        println!(
            "  {} Integrity check: {}",
            "❌".bright_red(),
            result.bright_red()
        );
        println!(
            "  {} Run: core db restore — to recover from backup",
            "→".bright_green()
        );
    }

    let wal_status = if journal == "wal" {
        "WAL mode ✅".bright_green().to_string()
    } else {
        format!("{} ⚠️  (should be WAL)", journal)
            .bright_yellow()
            .to_string()
    };
    println!("  {} Journal mode: {}", "·".dimmed(), wal_status);
    println!();

    Ok(())
}

/// core db status — show db size, table counts, last backup
pub fn status(ctx: &AppContext) -> CoreResult<()> {
    let db = db_path(ctx);
    let backups = backups_dir(ctx);

    let size = std::fs::metadata(&db).map(|m| m.len()).unwrap_or(0);

    // Table row counts
    let tables = [
        "events",
        "shell_history",
        "forest_predictions",
        "forest_goals",
        "reaction_log",
        "session_patterns",
    ];

    println!();
    println!("  {}", "🌲 DB — Status".bright_green().bold());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    println!(
        "  {} {}",
        "▶".bright_cyan(),
        "Database:".bright_white().bold()
    );
    println!("    {} Path: {}", "·".dimmed(), db.display());
    println!(
        "    {} Size: {}",
        "·".dimmed(),
        fmt_size(size).bright_white()
    );

    let journal: String = ctx
        .runtime
        .db
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap_or_else(|_| "unknown".to_string());
    let wal = if journal == "wal" {
        "WAL ✅".bright_green().to_string()
    } else {
        format!("{} ⚠️", journal).bright_yellow().to_string()
    };
    println!("    {} Mode: {}", "·".dimmed(), wal);
    println!();

    println!(
        "  {} {}",
        "▶".bright_cyan(),
        "Table sizes:".bright_white().bold()
    );
    for table in &tables {
        let count: i64 = ctx
            .runtime
            .db
            .query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |r| r.get(0))
            .unwrap_or(0);
        println!(
            "    {} {:<25} {:>6} rows",
            "·".dimmed(),
            table,
            count.to_string().bright_white()
        );
    }
    println!();

    println!(
        "  {} {}",
        "▶".bright_cyan(),
        "Backups:".bright_white().bold()
    );
    if let Ok(entries) = std::fs::read_dir(&backups) {
        let mut baks: Vec<_> = entries
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("state-"))
            .collect();
        baks.sort_by_key(|e| std::cmp::Reverse(e.file_name()));
        if baks.is_empty() {
            println!("    {} No backups yet — run: core db backup", "·".dimmed());
        } else {
            for bak in baks.iter().take(5) {
                let size = std::fs::metadata(bak.path()).map(|m| m.len()).unwrap_or(0);
                println!(
                    "    {} {}  {}",
                    "·".dimmed(),
                    bak.file_name().to_string_lossy().bright_white(),
                    fmt_size(size).dimmed()
                );
            }
        }
    }
    println!();

    Ok(())
}

/// core db compact — VACUUM to reclaim space
pub fn compact(ctx: &AppContext) -> CoreResult<()> {
    let before = std::fs::metadata(db_path(ctx))
        .map(|m| m.len())
        .unwrap_or(0);

    println!();
    println!("  {}", "🌲 DB — Compact".bright_green().bold());
    println!(
        "{}",
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();
    println!("  {} Running VACUUM...", "·".dimmed());

    ctx.runtime.db.execute_batch("VACUUM;")?;

    let after = std::fs::metadata(db_path(ctx))
        .map(|m| m.len())
        .unwrap_or(0);
    let saved = before.saturating_sub(after);

    println!("  {} Compact complete:", "✅".bright_green());
    println!("    {} Before: {}", "·".dimmed(), fmt_size(before));
    println!(
        "    {} After:  {}",
        "·".dimmed(),
        fmt_size(after).bright_green()
    );
    if saved > 0 {
        println!(
            "    {} Saved:  {}",
            "·".dimmed(),
            fmt_size(saved).bright_green()
        );
    }
    println!();

    Ok(())
}
