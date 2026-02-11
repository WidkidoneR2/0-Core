//! faelight-stow v2.2.0 - Stow Verification (Auto-discovery)
//! 🌲 Faelight Forest
use faelight_core::paths as core_paths;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
mod paths;

#[derive(Debug)]
struct Issue {
    package: String,
    path: String,
    problem: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Conflict {
    package: String,
    target_path: PathBuf,
    conflict_type: ConflictType,
}

#[derive(Debug)]
#[allow(dead_code)]
enum ConflictType {
    PreExistingFile,
    DuplicateTarget(String),
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let quiet = args.contains(&"--quiet".to_string());
    let fix = args.contains(&"--fix".to_string());
    let notify = args.contains(&"--notify".to_string());

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return;
    }

    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("faelight-stow v2.2.0");
        return;
    }

    if !quiet {
        eprintln!("🔗 faelight-stow v2.2.0 - Verifying symlinks...");
    }

    let core_dir = core_paths::core_dir();
    let stow_dir = PathBuf::from(crate::paths::stow_dir());

    // Handle check subcommand: faelight-stow check <packages...>
    if args.len() >= 2 && args[1] == "check" {
        let packages: Vec<String> = args[2..]
            .iter()
            .filter(|arg| !arg.starts_with("--"))
            .cloned()
            .collect();

        if packages.is_empty() {
            eprintln!("❌ No packages specified for check");
            eprintln!("   Usage: faelight-stow check <package1> [package2...]");
            std::process::exit(1);
        }

        check_collisions(&stow_dir, &packages);
        return;
    }

    // Handle stowing a new package: faelight-stow <package-name>
    if args.len() >= 2 && !args[1].starts_with("--") {
        let pkg_name = &args[1];
        println!("📦 Stowing new package: {}", pkg_name);

        let pkg_dir = stow_dir.join(pkg_name);
        if !pkg_dir.exists() {
            eprintln!("❌ Package not found: {}", pkg_name);
            std::process::exit(1);
        }

        let status = Command::new("stow")
            .current_dir(&core_dir)
            .args([
                "--dir=03-interfaces/stow",
                "--ignore=\\.dotmeta",
                "-R",
                pkg_name,
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("✅ Successfully stowed {}", pkg_name);
                std::process::exit(0);
            }
            _ => {
                eprintln!("❌ Failed to stow {}", pkg_name);
                std::process::exit(1);
            }
        }
    }

    // Auto-discover packages from stow/ directory
    let packages = discover_packages(&stow_dir);

    if packages.is_empty() {
        eprintln!("⚠️  No packages found in {}", stow_dir.display());
        return;
    }

    let mut issues: Vec<Issue> = Vec::new();
    let mut verified = 0;

    for package in &packages {
        // Find symlinks in ~/ that point to this package
        let symlinks = find_package_symlinks(&core_paths::home().to_string_lossy(), package);

        if symlinks.is_empty() {
            issues.push(Issue {
                package: package.clone(),
                path: "No symlinks found".to_string(),
                problem: "Package not stowed".to_string(),
            });
            continue;
        }

        // Verify each symlink points to correct location
        for link_path in symlinks {
            if verify_symlink(&link_path, &stow_dir, package) {
                verified += 1;
            } else {
                issues.push(Issue {
                    package: package.clone(),
                    path: link_path
                        .strip_prefix(core_paths::home())
                        .unwrap_or(&link_path)
                        .display()
                        .to_string(),
                    problem: "Invalid symlink".to_string(),
                });
            }
        }
    }

    if issues.is_empty() {
        if !quiet {
            println!(
                "✅ All {} stow targets verified across {} packages",
                verified,
                packages.len()
            );
        }
        return;
    }

    println!("⚠️  Found {} issues:", issues.len());
    println!();

    for issue in &issues {
        println!("  {} [{}]", issue.path, issue.package);
        println!("    Problem: {}", issue.problem);
    }

    println!();

    if fix {
        println!("🔧 Running stow to fix...");
        let packages: Vec<&str> = issues.iter().map(|i| i.package.as_str()).collect();
        let unique_packages: std::collections::HashSet<&str> = packages.into_iter().collect();

        for pkg in unique_packages {
            let status = Command::new("stow")
                .current_dir(&core_dir)
                .args(["--dir=03-interfaces/stow", "-R", pkg])
                .status();

            match status {
                Ok(s) if s.success() => println!("  ✅ Restowed {}", pkg),
                _ => println!("  ❌ Failed to restow {}", pkg),
            }
        }
    } else {
        println!("💡 To fix manually:");
        println!("   cd ~/0-core && stow --dir=03-interfaces/stow -R <package>");
        println!();
        println!("   Or run: faelight-stow --fix");
    }

    if notify {
        send_notification(&issues);
    }
}

fn discover_packages(stow_dir: &Path) -> Vec<String> {
    let mut packages = Vec::new();

    if let Ok(entries) = fs::read_dir(stow_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip hidden directories and .dotmeta files
                    if !name.starts_with('.') {
                        packages.push(name.to_string());
                    }
                }
            }
        }
    }

    packages.sort();
    packages
}

fn find_package_symlinks(_home: &str, package: &str) -> Vec<PathBuf> {
    let home_path = core_paths::home();
    let mut symlinks = Vec::new();

    // Search common locations
    let search_paths = vec![home_path.clone(), home_path.join(".config")];

    for search_path in search_paths {
        if let Ok(entries) = fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Check if it's a symlink pointing to our stow package
                if path.is_symlink() {
                    if let Ok(target) = fs::read_link(&path) {
                        let target_str = target.to_string_lossy();
                        if target_str.contains(&format!("0-core/03-interfaces/stow/{}", package)) {
                            symlinks.push(path);
                        }
                    }
                }
            }
        }
    }

    symlinks
}

fn verify_symlink(link_path: &PathBuf, stow_dir: &Path, package: &str) -> bool {
    // Check if symlink target contains the package path
    if let Ok(target) = fs::read_link(link_path) {
        let target_str = target.to_string_lossy();
        if target_str.contains(&format!("stow/{}", package)) {
            // Also verify the target actually exists
            if let Ok(resolved) = fs::canonicalize(link_path) {
                return resolved.starts_with(stow_dir.join(package));
            }
        }
    }
    false
}

fn print_help() {
    println!("faelight-stow v2.2.0 - Stow Symlink Verification");
    println!();
    println!("USAGE:");
    println!("    faelight-stow [OPTIONS] [PACKAGE]");
    println!();
    println!("OPTIONS:");
    println!("    --quiet      Suppress output if healthy");
    println!("    --fix        Attempt to fix issues with stow -R");
    println!("    --notify     Send notification on issues");
    println!("    -v, --version Show version");
    println!("    -h, --help   Show this help");
    println!();
    println!("EXAMPLES:");
    println!("    faelight-stow              # Verify all packages");
    println!("    faelight-stow shell-zsh    # Stow a specific package");
    println!("    faelight-stow --fix        # Auto-fix issues");
    println!();
    println!("PHILOSOPHY:");
    println!("    Auto-discover packages, verify integrity, stay in control.");
}

fn send_notification(issues: &[Issue]) {
    let summary = format!("🔗 Stow: {} issues found", issues.len());
    let body: Vec<String> = issues
        .iter()
        .map(|i| format!("{} ({})", i.path, i.package))
        .collect();

    Command::new("notify-send")
        .args([&summary, &body.join(", ")])
        .spawn()
        .ok();
}

fn check_collisions(stow_dir: &Path, packages: &[String]) {
    println!("🔍 Stow Collision Preflight Check");
    println!("{}", "━".repeat(50));
    println!();

    let mut all_conflicts = Vec::new();
    let mut target_map: HashMap<PathBuf, String> = HashMap::new();
    let home = PathBuf::from(std::env::var("HOME").unwrap());

    for package in packages {
        let pkg_dir = stow_dir.join(package);

        if !pkg_dir.exists() {
            println!("📦 {}: ❌ Package not found in stow directory", package);
            continue;
        }

        println!("📦 {}:", package);
        let mut pkg_ok = true;

        // Walk the package directory
        if let Ok(entries) = walk_dir(&pkg_dir) {
            for entry in entries {
                // Calculate where this file would be symlinked to
                if let Ok(rel_path) = entry.strip_prefix(&pkg_dir) {
                    let target = home.join(rel_path);

                    // Check 1: Pre-existing non-symlink file
                    if target.exists() && !target.is_symlink() {
                        println!("  ⚠️  CONFLICT: {}", rel_path.display());
                        println!("      Exists as regular file (not symlink)");
                        all_conflicts.push(Conflict {
                            package: package.clone(),
                            target_path: target.clone(),
                            conflict_type: ConflictType::PreExistingFile,
                        });
                        pkg_ok = false;
                    }

                    // Check 2: Duplicate target across packages
                    if let Some(other_pkg) = target_map.get(&target) {
                        println!("  ⚠️  DUPLICATE: {}", rel_path.display());
                        println!("      Also in package: {}", other_pkg);
                        all_conflicts.push(Conflict {
                            package: package.clone(),
                            target_path: target.clone(),
                            conflict_type: ConflictType::DuplicateTarget(other_pkg.clone()),
                        });
                        pkg_ok = false;
                    } else {
                        target_map.insert(target, package.clone());
                    }
                }
            }
        }

        if pkg_ok {
            println!("  ✅ No conflicts");
        }
        println!();
    }

    // Summary
    println!("{}", "━".repeat(50));
    if all_conflicts.is_empty() {
        println!("✅ All clear! Safe to stow.");
    } else {
        println!("⚠️  {} conflicts detected", all_conflicts.len());
        println!();
        println!("💡 To backup conflicting files:");
        println!("   mv <file> <file>.bak");
        std::process::exit(1);
    }
}

fn walk_dir(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    fn visit(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit(&path, files)?;
                } else {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    visit(dir, &mut files)?;
    Ok(files)
}
