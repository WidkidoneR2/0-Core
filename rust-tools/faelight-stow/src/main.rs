//! faelight-stow v0.3.0 - Stow Verification (Auto-discovery)
//! 🌲 Faelight Forest
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
struct Issue {
    package: String,
    path: String,
    problem: String,
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
        println!("faelight-stow v0.3.0");
        return;
    }
    
    if !quiet {
        eprintln!("🔗 faelight-stow v0.3.0 - Verifying symlinks...");
    }
    
    let home = env::var("HOME").expect("HOME not set");
    let core_dir = PathBuf::from(&home).join("0-core");
    let stow_dir = core_dir.join("stow");
    
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
            .args(["--dir=stow", "--ignore=\\.dotmeta", "-R", pkg_name])
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
        let symlinks = find_package_symlinks(&home, package);
        
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
                    path: link_path.strip_prefix(&home).unwrap_or(&link_path).display().to_string(),
                    problem: "Invalid symlink".to_string(),
                });
            }
        }
    }
    
    if issues.is_empty() {
        if !quiet {
            println!("✅ All {} stow targets verified across {} packages", verified, packages.len());
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
                .args(["--dir=stow", "-R", pkg])
                .status();
            
            match status {
                Ok(s) if s.success() => println!("  ✅ Restowed {}", pkg),
                _ => println!("  ❌ Failed to restow {}", pkg),
            }
        }
    } else {
        println!("💡 To fix manually:");
        println!("   cd ~/0-core && stow --dir=stow -R <package>");
        println!();
        println!("   Or run: faelight-stow --fix");
    }
    
    if notify {
        send_notification(&issues);
    }
}

fn discover_packages(stow_dir: &PathBuf) -> Vec<String> {
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

fn find_package_symlinks(home: &str, package: &str) -> Vec<PathBuf> {
    let home_path = PathBuf::from(home);
    let mut symlinks = Vec::new();
    
    // Search common locations
    let search_paths = vec![
        home_path.clone(),
        home_path.join(".config"),
    ];
    
    for search_path in search_paths {
        if let Ok(entries) = fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Check if it's a symlink pointing to our stow package
                if path.is_symlink() {
                    if let Ok(target) = fs::read_link(&path) {
                        let target_str = target.to_string_lossy();
                        if target_str.contains(&format!("0-core/stow/{}", package)) {
                            symlinks.push(path);
                        }
                    }
                }
            }
        }
    }
    
    symlinks
}

fn verify_symlink(link_path: &PathBuf, stow_dir: &PathBuf, package: &str) -> bool {
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
    println!("faelight-stow v0.3.0 - Stow Symlink Verification");
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
    let body: Vec<String> = issues.iter().map(|i| format!("{} ({})", i.path, i.package)).collect();
    
    Command::new("notify-send")
        .args([&summary, &body.join(", ")])
        .spawn()
        .ok();
}
