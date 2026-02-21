use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

pub fn status(ctx: &AppContext, json: bool) -> CoreResult<()> {
    let stow_dir = PathBuf::from(&ctx.core_root).join("03-interfaces/stow");

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
        let pkg_path = stow_dir.join(pkg);
        let links = count_links_recursive(&pkg_path, &home, pkg);
        total_links += links;
        results.push((pkg.clone(), links));
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "packages": results.iter().map(|(p, l)| serde_json::json!({"package": p, "links": l})).collect::<Vec<_>>(),
            "total": total_links
        })).unwrap_or_default());
    } else {
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        println!("{} {}", "🔗 core link status".bold(), "v2.0.0".dimmed());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        for (pkg, links) in &results {
            if *links > 0 {
                println!(
                    "  {} {} ({} links)",
                    "✓".bright_green(),
                    pkg.bright_white(),
                    links
                );
            } else {
                println!("  {} {} (not stowed)", "✗".bright_red(), pkg);
            }
        }
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        println!("  Total links: {}", total_links.to_string().bright_white());
    }

    Ok(())
}

pub fn list(ctx: &AppContext) -> CoreResult<()> {
    let stow_dir = PathBuf::from(&ctx.core_root).join("03-interfaces/stow");
    let packages = discover_packages(&stow_dir);
    println!("{}", "📦 Stow packages:".bold());
    for pkg in &packages {
        println!("  {} {}", "▶".dimmed(), pkg);
    }
    println!("  Total: {}", packages.len());
    Ok(())
}

pub fn audit(ctx: &AppContext) -> CoreResult<()> {
    let stow_dir = PathBuf::from(&ctx.core_root).join("03-interfaces/stow");
    let home = PathBuf::from(&ctx.home);
    let packages = discover_packages(&stow_dir);
    let mut broken: Vec<(String, String)> = vec![];
    let mut total = 0;

    for pkg in &packages {
        let pkg_path = stow_dir.join(pkg);
        let links = find_links_recursive(&pkg_path, &home, pkg);
        for link in links {
            total += 1;
            if !link.target.exists() {
                broken.push((pkg.clone(), link.path.display().to_string()));
            }
        }
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔍 core link audit".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
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

// --- Internal helpers ---

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

    // Skip empty packages
    packages.retain(|pkg| {
        walkdir::WalkDir::new(stow_dir.join(pkg))
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| e.file_type().is_file())
    });

    packages.sort();
    packages
}

struct LinkInfo {
    path: PathBuf,
    target: PathBuf,
}

fn find_links_recursive(pkg_path: &Path, home: &Path, _pkg: &str) -> Vec<LinkInfo> {
    let mut links = vec![];
    let walker = walkdir::WalkDir::new(pkg_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());

    for entry in walker {
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

fn count_links_recursive(pkg_path: &Path, home: &Path, pkg: &str) -> usize {
    find_links_recursive(pkg_path, home, pkg).len()
}
