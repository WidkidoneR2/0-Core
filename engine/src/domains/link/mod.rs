use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

// ── public domain functions ──────────────────────────────────────────────────

pub fn status(ctx: &AppContext, json: bool) -> CoreResult<()> {
    ctx.capabilities.require(
        "link",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;
    let stow_dir = stow_dir(ctx);
    if !stow_dir.exists() {
        eprintln!(
            "{} Stow directory not found: {}",
            "✗".bright_red(),
            stow_dir.display()
        );
        return Ok(());
    }
    let packages = discover_packages(&stow_dir);
    let home = PathBuf::from(&ctx.home);
    let mut total_links = 0;
    let mut results: Vec<(String, usize)> = vec![];
    for pkg in &packages {
        let links = count_links_recursive(&stow_dir.join(pkg), &home, pkg);
        total_links += links;
        results.push((pkg.clone(), links));
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "packages": results.iter().map(|(p, l)| serde_json::json!({"package": p, "links": l})).collect::<Vec<_>>(),
            "total": total_links
        })).unwrap_or_default());
    } else {
        println!("{}", "━".repeat(41).dimmed());
        println!("{} {}", "🔗 core link status".bold(), "v3.0.0".dimmed());
        println!("{}", "━".repeat(41).dimmed());
        for (pkg, links) in &results {
            if *links > 0 {
                println!(
                    "  {} {} ({} links)",
                    "✓".bright_green(),
                    pkg.bright_white(),
                    links
                );
            } else {
                println!("  {} {} (not deployed)", "✗".bright_red(), pkg);
            }
        }
        println!("{}", "━".repeat(41).dimmed());
        println!("  Total links: {}", total_links.to_string().bright_white());
    }
    Ok(())
}

pub fn list(ctx: &AppContext) -> CoreResult<()> {
    let stow_dir = stow_dir(ctx);
    let packages = discover_packages(&stow_dir);
    println!("{}", "📦 Packages:".bold());
    for pkg in &packages {
        println!("  {} {}", "▶".dimmed(), pkg);
    }
    println!("  Total: {}", packages.len());
    Ok(())
}

pub fn audit(ctx: &AppContext) -> CoreResult<()> {
    let stow_dir = stow_dir(ctx);
    let home = PathBuf::from(&ctx.home);
    let packages = discover_packages(&stow_dir);
    let mut broken: Vec<(String, String)> = vec![];
    let mut total = 0;
    for pkg in &packages {
        for link in find_links_recursive(&stow_dir.join(pkg), &home) {
            total += 1;
            if !link.target.exists() {
                broken.push((pkg.clone(), link.path.display().to_string()));
            }
        }
    }
    println!("{}", "━".repeat(41).dimmed());
    println!("{}", "🔍 core link audit".bold());
    println!("{}", "━".repeat(41).dimmed());
    println!("  Scanned: {} links", total);
    if broken.is_empty() {
        println!("  {} No broken links", "✅".green());
    } else {
        println!("  {} {} broken links:", "✗".bright_red(), broken.len());
        for (pkg, path) in &broken {
            println!("    {} [{}] {}", "▶".dimmed(), pkg, path);
        }
    }
    Ok(())
}

pub fn plan(ctx: &AppContext, package: Option<&str>) -> CoreResult<()> {
    let stow_dir = stow_dir(ctx);
    let home = PathBuf::from(&ctx.home);
    let packages = resolve_packages(&stow_dir, package);
    println!("{}", "━".repeat(41).dimmed());
    println!("{}", "📋 Deploy Plan (dry-run)".bold());
    println!("{}", "━".repeat(41).dimmed());
    let mut total_create = 0;
    let mut total_conflict = 0;
    for pkg in &packages {
        let ops = build_plan(&stow_dir.join(pkg), &home);
        println!("  {} {}", "📦".dimmed(), pkg.bright_white());
        for op in &ops {
            match op {
                Op::Create { link, .. } => {
                    println!("    {} {}", "create".bright_green(), link.display());
                    total_create += 1;
                }
                Op::Conflict { link, reason } => {
                    println!(
                        "    {} {} — {}",
                        "conflict".bright_red(),
                        link.display(),
                        reason
                    );
                    total_conflict += 1;
                }
                Op::Skip { link, reason } => {
                    println!("    {} {} — {}", "skip".yellow(), link.display(), reason);
                }
            }
        }
    }
    println!("{}", "━".repeat(41).dimmed());
    println!(
        "  {} to create, {} conflicts",
        total_create.to_string().green(),
        if total_conflict > 0 {
            total_conflict.to_string().bright_red()
        } else {
            "0".normal()
        }
    );
    if total_conflict > 0 {
        println!("  {} Resolve conflicts before deploying", "⚠️".yellow());
    }
    Ok(())
}

pub fn adopt(ctx: &AppContext, package: Option<&str>) -> CoreResult<()> {
    let stow_dir = stow_dir(ctx);
    let home = PathBuf::from(&ctx.home);
    let packages = resolve_packages(&stow_dir, package);
    println!("{}", "━".repeat(41).dimmed());
    println!(
        "{}",
        "📎 Adopting (converting real files to symlinks)".bold()
    );
    println!("{}", "━".repeat(41).dimmed());
    let mut created = 0;
    let mut skipped = 0;
    let mut failed = 0;
    for pkg in &packages {
        let pkg_path = stow_dir.join(pkg);
        println!("  {} {}", "📦".dimmed(), pkg.bright_white());
        for entry in walkdir::WalkDir::new(&pkg_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && !e.path_is_symlink())
        {
            let stow_file = entry.path().to_path_buf();
            let rel = stow_file.strip_prefix(&pkg_path).unwrap_or(&stow_file);
            let real = home.join(rel);
            // Skip if real path resolves into the stow directory (directory folding)
            let real_canonical = std::fs::canonicalize(&real).unwrap_or(real.clone());
            if real_canonical.starts_with(&stow_dir) {
                skipped += 1;
                continue;
            }
            if real.is_symlink() {
                if let Ok(target) = fs::read_link(&real) {
                    let resolved = if target.is_absolute() {
                        target.clone()
                    } else if let Some(parent) = real.parent() {
                        std::fs::canonicalize(parent.join(&target))
                            .unwrap_or_else(|_| parent.join(&target))
                    } else {
                        target.clone()
                    };
                    eprintln!(
                        "DBG real={} resolved={} stow={} match={}",
                        real.display(),
                        resolved.display(),
                        stow_file.display(),
                        resolved == stow_file
                    );
                    if resolved == stow_file {
                        skipped += 1;
                        continue;
                    }
                }
                let _ = fs::remove_file(&real);
            } else if real.is_file() {
                let _ = fs::remove_file(&real);
            } else {
                continue;
            }
            if let Some(parent) = real.parent() {
                let _ = fs::create_dir_all(parent);
            }
            match std::os::unix::fs::symlink(&stow_file, &real) {
                Ok(_) => {
                    println!("    {} {}", "✓".bright_green(), real.display());
                    created += 1;
                }
                Err(e) => {
                    println!("    {} {} — {}", "✗".bright_red(), real.display(), e);
                    failed += 1;
                }
            }
        }
    }
    println!("{}", "━".repeat(41).dimmed());
    println!(
        "  {} adopted, {} skipped, {} failed",
        created.to_string().green(),
        skipped,
        if failed > 0 {
            failed.to_string().bright_red()
        } else {
            "0".normal()
        }
    );
    Ok(())
}

pub fn deploy(
    ctx: &AppContext,
    package: Option<&str>,
    no_snapshot: bool,
    adopt: bool,
) -> CoreResult<()> {
    let stow_dir = stow_dir(ctx);
    let home = PathBuf::from(&ctx.home);
    let packages = resolve_packages(&stow_dir, package);
    println!("{}", "━".repeat(41).dimmed());
    println!("{}", "🚀 Deploying".bold());
    println!("{}", "━".repeat(41).dimmed());

    // Snapshot before deploy
    if !no_snapshot {
        print!("  📸 Creating snapshot... ");
        let snap = std::process::Command::new("faelight-snapshot")
            .args([
                "create",
                &format!(
                    "pre-deploy-{}",
                    chrono::Local::now().format("%Y%m%d_%H%M%S")
                ),
            ])
            .output();
        match snap {
            Ok(s) if s.status.success() => println!("{}", "✅".green()),
            _ => println!("{}", "⚠️  snapshot failed, continuing".yellow()),
        }
    }

    // Validate — abort if any conflicts
    let mut all_ops: Vec<(String, Vec<Op>)> = vec![];
    let mut has_conflicts = false;
    for pkg in &packages {
        let ops = build_plan(&stow_dir.join(pkg), &home);
        if ops.iter().any(|o| matches!(o, Op::Conflict { .. })) {
            has_conflicts = true;
        }
        all_ops.push((pkg.clone(), ops));
    }
    if has_conflicts && !adopt {
        println!(
            "  {} Conflicts detected — run {} to see details",
            "✗".bright_red(),
            "core link plan".cyan()
        );
        return Ok(());
    }

    // Execute atomically
    let mut created = 0;
    let mut failed = 0;
    for (pkg, ops) in &all_ops {
        println!("  {} {}", "📦".dimmed(), pkg.bright_white());
        for op in ops {
            match op {
                Op::Create { link, target } => {
                    if let Some(parent) = link.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    match std::os::unix::fs::symlink(target, link) {
                        Ok(_) => {
                            println!("    {} {}", "✓".bright_green(), link.display());
                            created += 1;
                        }
                        Err(e) => {
                            println!("    {} {} — {}", "✗".bright_red(), link.display(), e);
                            failed += 1;
                        }
                    }
                }
                Op::Conflict { link, .. } if adopt => {
                    // Adopt: only replace real files in ~/.config with symlinks.
                    // Never touch stow package files.
                    // The target is the stow package file for this pkg/rel.
                    let rel = link.strip_prefix(&home).unwrap_or(link.as_path());
                    let target = stow_dir.join(pkg).join(rel);
                    // Only proceed if stow target is a real file (not symlink)
                    if !target.exists() || target.is_symlink() {
                        println!(
                            "    {} {} — stow target not a real file, skipping",
                            "⚠️".yellow(),
                            link.display()
                        );
                        continue;
                    }
                    // Only replace real files — skip if already a symlink
                    if link.is_symlink() {
                        continue;
                    }
                    if link.is_file() {
                        let _ = fs::remove_file(link);
                    }
                    if let Some(parent) = link.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    match std::os::unix::fs::symlink(&target, link) {
                        Ok(_) => {
                            println!("    {} {}", "✓".bright_green(), link.display());
                            created += 1;
                        }
                        Err(e) => {
                            println!("    {} {} — {}", "✗".bright_red(), link.display(), e);
                            failed += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    println!("{}", "━".repeat(41).dimmed());
    println!(
        "  {} created, {} failed",
        created.to_string().green(),
        if failed > 0 {
            failed.to_string().bright_red()
        } else {
            "0".normal()
        }
    );
    Ok(())
}

pub fn undeploy(ctx: &AppContext, package: &str) -> CoreResult<()> {
    let stow_dir = stow_dir(ctx);
    let home = PathBuf::from(&ctx.home);
    let pkg_path = stow_dir.join(package);
    if !pkg_path.exists() {
        println!("  {} Package not found: {}", "✗".bright_red(), package);
        return Ok(());
    }
    println!("{}", "━".repeat(41).dimmed());
    println!("  {} Undeploying {}", "🗑️".dimmed(), package.bright_white());
    println!("{}", "━".repeat(41).dimmed());
    let mut removed = 0;
    let mut skipped = 0;
    for entry in walkdir::WalkDir::new(&pkg_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !e.path_is_symlink())
    {
        let rel = entry.path().strip_prefix(&pkg_path).unwrap_or(entry.path());
        let link = home.join(rel);
        if link.symlink_metadata().is_ok() {
            if let Ok(target) = fs::read_link(&link) {
                if target == entry.path() {
                    let _ = fs::remove_file(&link);
                    println!("    {} {}", "✓".bright_green(), link.display());
                    removed += 1;
                    continue;
                }
            }
        }
        println!("    {} {} (not our link)", "skip".yellow(), link.display());
        skipped += 1;
    }
    println!("{}", "━".repeat(41).dimmed());
    println!(
        "  {} removed, {} skipped",
        removed.to_string().green(),
        skipped
    );
    Ok(())
}

pub fn redeploy(ctx: &AppContext, package: Option<&str>) -> CoreResult<()> {
    let stow_dir = stow_dir(ctx);
    let packages = resolve_packages(&stow_dir, package);
    println!("{}", "━".repeat(41).dimmed());
    println!("{}", "🔄 Redeploying".bold());
    println!("{}", "━".repeat(41).dimmed());
    for pkg in &packages {
        undeploy(ctx, pkg)?;
        deploy(ctx, Some(pkg), false, false)?;
    }
    Ok(())
}

pub fn sync(ctx: &AppContext, package: Option<&str>) -> CoreResult<()> {
    ctx.capabilities.require(
        "link",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;

    let stow_dir = stow_dir(ctx);
    let home = PathBuf::from(&ctx.home);
    let packages = resolve_packages(&stow_dir, package);

    println!("{}", "━".repeat(52).dimmed());
    println!("{}", "🔗 core link sync".bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();

    let mut total_created = 0usize;
    let mut total_skipped = 0usize;
    let mut total_conflicts: Vec<(String, PathBuf, String)> = vec![];

    for pkg in &packages {
        let ops = build_plan(&stow_dir.join(pkg), &home);

        let creates: Vec<_> = ops
            .iter()
            .filter(|o| matches!(o, Op::Create { .. }))
            .count()
            .to_string()
            .into_bytes();
        let conflicts: Vec<_> = ops
            .iter()
            .filter(|o| matches!(o, Op::Conflict { .. }))
            .collect();
        let skips = ops.iter().filter(|o| matches!(o, Op::Skip { .. })).count();
        let creates_count = ops
            .iter()
            .filter(|o| matches!(o, Op::Create { .. }))
            .count();
        let _ = creates;

        if creates_count == 0 && conflicts.is_empty() {
            // Nothing to do for this package
            println!(
                "  {} {} — already in sync ({} links)",
                "✓".bright_green(),
                pkg.bright_white(),
                skips
            );
            total_skipped += skips;
            continue;
        }

        println!("  {} {}", "📦".dimmed(), pkg.bright_white());

        // Deploy clean ops
        for op in &ops {
            match op {
                Op::Create { link, target } => {
                    if let Some(parent) = link.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    match std::os::unix::fs::symlink(target, link) {
                        Ok(_) => {
                            println!(
                                "    {} {}",
                                "linked".bright_green(),
                                link.strip_prefix(&home).unwrap_or(link.as_path()).display()
                            );
                            total_created += 1;
                        }
                        Err(e) => {
                            println!("    {} {} — {}", "✗".bright_red(), link.display(), e);
                        }
                    }
                }
                Op::Skip { .. } => {
                    total_skipped += 1;
                }
                Op::Conflict { link, reason } => {
                    let rel = link.strip_prefix(&home).unwrap_or(link.as_path());
                    println!(
                        "    {} {} — {}",
                        "conflict".bright_red(),
                        rel.display(),
                        reason.dimmed()
                    );
                    total_conflicts.push((pkg.clone(), link.clone(), reason.clone()));
                }
            }
        }
        println!();
    }

    println!("{}", "━".repeat(52).dimmed());
    println!(
        "  {} linked   {} already in sync   {} conflicts",
        total_created.to_string().bright_green(),
        total_skipped.to_string().dimmed(),
        if total_conflicts.is_empty() {
            "0".normal()
        } else {
            total_conflicts.len().to_string().bright_red()
        }
    );

    if !total_conflicts.is_empty() {
        println!();
        println!("  {}", "Resolve conflicts:".yellow().bold());
        for (pkg, link, _) in &total_conflicts {
            let rel = link.strip_prefix(&home).unwrap_or(link.as_path());
            println!(
                "  {} backup original:  mv {} {}.bak",
                "→".dimmed(),
                link.display(),
                link.display()
            );
            println!(
                "  {} then re-run:      core link sync {}",
                "→".dimmed(),
                pkg
            );
            let _ = rel;
        }
    }

    Ok(())
}

// ── internal types ───────────────────────────────────────────────────────────

enum Op {
    Create { link: PathBuf, target: PathBuf },
    Conflict { link: PathBuf, reason: String },
    Skip { link: PathBuf, reason: String },
}

struct LinkInfo {
    path: PathBuf,
    target: PathBuf,
}

// ── internal helpers ─────────────────────────────────────────────────────────

fn stow_dir(ctx: &AppContext) -> PathBuf {
    PathBuf::from(&ctx.core_root).join("03-interfaces/stow")
}

fn resolve_packages(stow_dir: &Path, package: Option<&str>) -> Vec<String> {
    match package {
        Some("all") | None => discover_packages(stow_dir),
        Some(pkg) => vec![pkg.to_string()],
    }
}

fn build_plan(pkg_path: &Path, home: &Path) -> Vec<Op> {
    let mut ops = vec![];
    for entry in walkdir::WalkDir::new(pkg_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !e.path_is_symlink())
    {
        let rel = entry.path().strip_prefix(pkg_path).unwrap_or(entry.path());
        let link = home.join(rel);
        let target = entry.path().to_path_buf();
        if link.symlink_metadata().is_ok() {
            // Check if the file is already reachable via a directory-level symlink
            if let Ok(canonical) = std::fs::canonicalize(&link) {
                if canonical == target {
                    ops.push(Op::Skip {
                        link,
                        reason: "already linked (dir)".to_string(),
                    });
                    continue;
                }
            }
            if let Ok(existing) = fs::read_link(&link) {
                // Resolve relative symlinks to absolute for comparison
                let resolved = if existing.is_absolute() {
                    existing.clone()
                } else if let Some(parent) = link.parent() {
                    std::fs::canonicalize(parent.join(&existing))
                        .unwrap_or_else(|_| parent.join(&existing))
                } else {
                    existing.clone()
                };
                if resolved == target {
                    ops.push(Op::Skip {
                        link,
                        reason: "already linked".to_string(),
                    });
                } else {
                    ops.push(Op::Conflict {
                        link,
                        reason: format!("points to {}", existing.display()),
                    });
                }
            } else {
                ops.push(Op::Conflict {
                    link,
                    reason: "real file exists".to_string(),
                });
            }
        } else {
            ops.push(Op::Create { link, target });
        }
    }
    ops
}

fn discover_packages(stow_dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(stow_dir) else {
        return vec![];
    };
    let mut packages: Vec<String> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .collect();
    packages.retain(|pkg| {
        walkdir::WalkDir::new(stow_dir.join(pkg))
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| e.file_type().is_file())
    });
    packages.sort();
    packages
}

fn find_links_recursive(pkg_path: &Path, home: &Path) -> Vec<LinkInfo> {
    let mut links = vec![];
    for entry in walkdir::WalkDir::new(pkg_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !e.path_is_symlink())
    {
        let rel = entry.path().strip_prefix(pkg_path).unwrap_or(entry.path());
        let link_path = home.join(rel);
        if link_path.exists() || link_path.symlink_metadata().is_ok() {
            links.push(LinkInfo {
                path: link_path.clone(),
                target: link_path,
            });
        }
    }
    links
}

fn count_links_recursive(pkg_path: &Path, home: &Path, _pkg: &str) -> usize {
    find_links_recursive(pkg_path, home).len()
}
