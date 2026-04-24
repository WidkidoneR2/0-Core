use anyhow::Result;
use colored::*;
use std::path::{Path, PathBuf};
fn meta_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("0-core/03-interfaces/link-meta")
}
fn stow_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("0-core/03-interfaces/stow")
}
pub struct PackageMeta {
    pub intent: String,
    pub description: String,
    pub critical: bool,
    pub _last_linked: String,
}
impl PackageMeta {
    pub fn load(package: &str) -> Option<Self> {
        let path = meta_dir().join(format!("{}.toml", package));
        let content = std::fs::read_to_string(&path).ok()?;
        let mut intent = String::new();
        let mut description = String::new();
        let mut critical = false;
        let mut last_linked = String::new();
        for line in content.lines() {
            if let Some(v) = line.strip_prefix("intent = \"") {
                intent = v.trim_end_matches('"').to_string();
            } else if let Some(v) = line.strip_prefix("description = \"") {
                description = v.trim_end_matches('"').to_string();
            } else if line == "critical = true" {
                critical = true;
            } else if let Some(v) = line.strip_prefix("last_linked = \"") {
                last_linked = v.trim_end_matches('"').to_string();
            }
        }
        Some(PackageMeta {
            intent,
            description,
            critical,
            _last_linked: last_linked,
        })
    }
}
fn count_symlinks(package: &str) -> usize {
    let pkg_dir = stow_dir().join(package);
    let home = PathBuf::from(std::env::var("HOME").unwrap_or_default());
    if !pkg_dir.exists() {
        return 0;
    }
    let mut count = 0usize;
    fn walk(dir: &Path, pkg_root: &Path, home: &Path, count: &mut usize) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path.strip_prefix(pkg_root).unwrap_or(&path);
            let target = home.join(rel);
            if target.is_symlink() {
                *count += 1;
                // Don't recurse into symlinked dirs
            } else if path.is_dir() {
                walk(&path, pkg_root, home, count);
            }
        }
    }
    walk(&pkg_dir, &pkg_dir, &home, &mut count);
    count
}
fn check_broken_symlinks(package: &str) -> Vec<String> {
    let pkg_dir = stow_dir().join(package);
    let home = PathBuf::from(std::env::var("HOME").unwrap_or_default());
    let mut broken = Vec::new();
    if !pkg_dir.exists() {
        return broken;
    }
    fn walk(dir: &Path, pkg_root: &Path, home: &Path, broken: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path.strip_prefix(pkg_root).unwrap_or(&path);
            let target = home.join(rel);
            if target.is_symlink() {
                if !target.exists() {
                    broken.push(rel.to_string_lossy().to_string());
                }
                // Don't recurse into symlinked dirs
            } else if path.is_dir() {
                walk(&path, pkg_root, home, broken);
            }
        }
    }
    walk(&pkg_dir, &pkg_dir, &home, &mut broken);
    broken
}
/// faelight-link status v3 -- per-package health with intent tracing
pub fn status_v3() -> Result<()> {
    println!();
    println!(
        "{}",
        "🔗 faelight-link v3 -- Forest-Aware Dotfile Intelligence"
            .bright_cyan()
            .bold()
    );
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let stow = stow_dir();
    let Ok(entries) = std::fs::read_dir(&stow) else {
        println!("  {} Stow directory not found", "✗".bright_red());
        return Ok(());
    };
    let mut packages: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .filter(|n| !n.starts_with('.'))
        .collect();
    packages.sort();
    let mut total_links = 0usize;
    let mut total_broken = 0usize;
    let mut critical_issues = 0usize;
    for pkg in &packages {
        let links = count_symlinks(pkg);
        let broken = check_broken_symlinks(pkg);
        let meta = PackageMeta::load(pkg);
        total_links += links;
        total_broken += broken.len();
        let status_icon = if broken.is_empty() {
            "✅".to_string()
        } else {
            "⚠️ ".to_string()
        };
        let critical_marker = if meta.as_ref().map(|m| m.critical).unwrap_or(false) {
            " [CRITICAL]".bright_red().to_string()
        } else {
            String::new()
        };
        if meta.as_ref().map(|m| m.critical).unwrap_or(false) && !broken.is_empty() {
            critical_issues += 1;
        }
        let intent_str = meta
            .as_ref()
            .map(|m| format!(" → {}", m.intent.dimmed()))
            .unwrap_or_default();
        println!(
            "  {} {:<25} {:>3} links{}{}",
            status_icon,
            pkg.bright_white(),
            links.to_string().bright_green(),
            intent_str,
            critical_marker
        );
        if !broken.is_empty() {
            for b in &broken {
                println!(
                    "    {} {} (broken symlink)",
                    "✗".bright_red(),
                    b.bright_red()
                );
            }
        }
    }
    println!();
    println!(
        "  {}",
        "─────────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  {} packages   {} symlinks   {} broken",
        packages.len().to_string().bright_cyan(),
        total_links.to_string().bright_green(),
        if total_broken > 0 {
            total_broken.to_string().bright_red()
        } else {
            total_broken.to_string().bright_green()
        }
    );
    if critical_issues > 0 {
        println!(
            "  {} {} critical package(s) have broken links!",
            "⚠️ ".yellow(),
            critical_issues
        );
    } else {
        println!("  {} All packages healthy", "✅".green());
    }
    println!();
    Ok(())
}
/// faelight-link audit v3 -- intent traceability per package
pub fn audit_v3() -> Result<()> {
    println!();
    println!(
        "{}",
        "🔍 faelight-link audit -- Intent Traceability"
            .bright_cyan()
            .bold()
    );
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let stow = stow_dir();
    let Ok(entries) = std::fs::read_dir(&stow) else {
        return Ok(());
    };
    let mut packages: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .filter(|n| !n.starts_with('.'))
        .collect();
    packages.sort();
    let mut orphaned = Vec::new();
    for pkg in &packages {
        let meta = PackageMeta::load(pkg);
        match meta {
            Some(m) => {
                let critical = if m.critical {
                    " [CRITICAL]".bright_red().to_string()
                } else {
                    String::new()
                };
                let intent = if m.intent == "INT-000" {
                    m.intent.bright_yellow().to_string()
                } else {
                    m.intent.bright_green().to_string()
                };
                println!(
                    "  {} {:<25} → {} -- {}{}",
                    "✅".normal(),
                    pkg.bright_white(),
                    intent,
                    m.description.dimmed(),
                    critical
                );
                if m.intent == "INT-000" {
                    orphaned.push(pkg.clone());
                }
            }
            None => {
                println!(
                    "  {} {:<25} → {} (no meta file)",
                    "⚠️ ".yellow(),
                    pkg.bright_white(),
                    "untraced".bright_red()
                );
                orphaned.push(pkg.clone());
            }
        }
    }
    println!();
    if !orphaned.is_empty() {
        println!(
            "  {} {} package(s) have no intent -- orphaned config is a liability:",
            "⚠️ ".yellow(),
            orphaned.len()
        );
        for o in &orphaned {
            println!("    {} {}", "→".dimmed(), o.bright_yellow());
        }
        println!(
            "  {} Update 03-interfaces/link-meta/<pkg>.toml to assign intent",
            "💡".normal()
        );
    } else {
        println!("  {} All packages traceable to an intent", "✅".green());
    }
    println!();
    Ok(())
}
/// faelight-link why <file> -- which package owns this file
pub fn why(file: &str) -> Result<()> {
    let home = PathBuf::from(std::env::var("HOME").unwrap_or_default());
    let target = if file.starts_with('/') {
        PathBuf::from(file)
    } else if file.starts_with("~/") {
        home.join(&file[2..])
    } else {
        home.join(file)
    };
    println!();
    println!(
        "  {} Looking up: {}",
        "🔍".normal(),
        target.display().to_string().bright_cyan()
    );
    // Check if it's a symlink
    if !target.is_symlink() {
        println!(
            "  {} {} is not a symlink -- not managed by faelight-link",
            "○".dimmed(),
            file
        );
        println!();
        return Ok(());
    }
    // Find which package owns it
    let stow = stow_dir();
    let rel = target.strip_prefix(&home).unwrap_or(&target);
    let Ok(entries) = std::fs::read_dir(&stow) else {
        return Ok(());
    };
    let packages: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .filter(|n| !n.starts_with('.'))
        .collect();
    for pkg in &packages {
        let source = stow.join(pkg).join(rel);
        if source.exists() {
            let meta = PackageMeta::load(pkg);
            println!("  {} Owned by: {}", "✅".green(), pkg.bright_cyan());
            if let Some(m) = meta {
                println!("  {} Intent:   {}", "→".dimmed(), m.intent.bright_green());
                println!("  {} Purpose:  {}", "→".dimmed(), m.description.dimmed());
                println!(
                    "  {} Critical: {}",
                    "→".dimmed(),
                    if m.critical {
                        "yes".bright_red()
                    } else {
                        "no".dimmed()
                    }
                );
            }
            println!(
                "  {} Source:   {}",
                "→".dimmed(),
                source.display().to_string().dimmed()
            );
            println!();
            return Ok(());
        }
    }
    println!(
        "  {} No package found owning {}",
        "○".dimmed(),
        file.bright_yellow()
    );
    println!();
    Ok(())
}
/// faelight-link verify -- deep validation of all symlinks
pub fn verify() -> Result<()> {
    println!();
    println!(
        "{}",
        "🔎 faelight-link verify -- Deep Validation"
            .bright_cyan()
            .bold()
    );
    println!("{}", "━".repeat(60).dimmed());
    println!();
    let stow = stow_dir();
    let home = PathBuf::from(std::env::var("HOME").unwrap_or_default());
    let Ok(entries) = std::fs::read_dir(&stow) else {
        return Ok(());
    };
    let mut packages: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .filter(|n| !n.starts_with('.'))
        .collect();
    packages.sort();
    let mut total_valid = 0usize;
    let mut total_broken = 0usize;
    let mut total_missing = 0usize;
    for pkg in &packages {
        let pkg_dir = stow.join(pkg);
        let broken = check_broken_symlinks(pkg);
        let links = count_symlinks(pkg);
        let meta = PackageMeta::load(pkg);
        let critical = meta.map(|m| m.critical).unwrap_or(false);
        if broken.is_empty() {
            total_valid += links;
            println!(
                "  {} {:<25} {}/{} valid",
                "✅".normal(),
                pkg.bright_white(),
                links,
                links
            );
        } else {
            total_broken += broken.len();
            total_valid += links - broken.len();
            let crit = if critical {
                " [CRITICAL]".bright_red().to_string()
            } else {
                String::new()
            };
            println!(
                "  {} {:<25} {}/{} valid -- {} broken{}",
                "⚠️ ".yellow(),
                pkg.bright_white(),
                (links - broken.len()),
                links,
                broken.len().to_string().bright_red(),
                crit
            );
        }
        // Check for files in package that have no symlink
        fn count_missing(dir: &Path, pkg_root: &Path, home: &Path) -> usize {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return 0;
            };
            let mut count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    count += count_missing(&path, pkg_root, home);
                } else {
                    let rel = path.strip_prefix(pkg_root).unwrap_or(&path);
                    let target = home.join(rel);
                    if !target.is_symlink() && !target.exists() {
                        count += 1;
                    }
                }
            }
            count
        }
        let missing = count_missing(&pkg_dir, &pkg_dir, &home);
        total_missing += missing;
        if missing > 0 {
            println!(
                "    {} {} file(s) not linked -- run: faelight-link stow {}",
                "💡".normal(),
                missing,
                pkg
            );
        }
    }
    println!();
    println!(
        "  {}",
        "─────────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  {} valid   {} broken   {} unlinked",
        total_valid.to_string().bright_green(),
        if total_broken > 0 {
            total_broken.to_string().bright_red()
        } else {
            "0".bright_green()
        },
        if total_missing > 0 {
            total_missing.to_string().bright_yellow()
        } else {
            "0".bright_green()
        }
    );
    println!();
    Ok(())
}
