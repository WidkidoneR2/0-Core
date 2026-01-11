//! faelight-stow v0.1 - Stow Verification (Philosophy: verify, don't auto-fix)
//! 🌲 Faelight Forest

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Check specific files - handles both dir-level and file-level symlinks
const STOW_FILES: &[(&str, &str)] = &[
    ("shell-zsh", ".config/zsh/.zshrc"),
    ("wm-sway", ".config/sway/config"),
    ("term-foot", ".config/foot/foot.ini"),
    ("prompt-starship", ".config/starship.toml"),
    ("fm-yazi", ".config/yazi/init.lua"),
    ("vcs-git", ".gitconfig"),
];

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
    
    if !quiet {
        eprintln!("🔗 faelight-stow v0.1 - Verifying symlinks...");
    }
    
    let home = env::var("HOME").expect("HOME not set");
    let core_dir = PathBuf::from(&home).join("0-core");
    let home_path = PathBuf::from(&home);
    
    let mut issues: Vec<Issue> = Vec::new();
    let mut verified = 0;
    
    for (package, target) in STOW_FILES {
        let expected_source = core_dir.join(package).join(target);
        let link_path = home_path.join(target);
        
        if !expected_source.exists() {
            continue;
        }
        
        if !link_path.exists() {
            issues.push(Issue {
                package: package.to_string(),
                path: target.to_string(),
                problem: "Missing".to_string(),
            });
            continue;
        }
        
        // Check if the file resolves to the correct 0-core location
        // This handles both:
        // 1. File is a direct symlink to 0-core
        // 2. Parent directory is a symlink to 0-core
        if let (Ok(resolved), Ok(expected_resolved)) = 
            (fs::canonicalize(&link_path), fs::canonicalize(&expected_source)) 
        {
            if resolved == expected_resolved {
                verified += 1;
                continue;
            } else {
                issues.push(Issue {
                    package: package.to_string(),
                    path: target.to_string(),
                    problem: format!("Wrong location: {:?}", resolved),
                });
                continue;
            }
        }
        
        // If canonicalize fails, something is wrong
        issues.push(Issue {
            package: package.to_string(),
            path: target.to_string(),
            problem: "Cannot resolve path".to_string(),
        });
    }
    
    if issues.is_empty() {
        if !quiet {
            println!("✅ All {} stow targets verified", verified);
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
                .args(["-R", pkg])
                .status();
            
            match status {
                Ok(s) if s.success() => println!("  ✅ Restowed {}", pkg),
                _ => println!("  ❌ Failed to restow {}", pkg),
            }
        }
    } else {
        println!("💡 To fix manually:");
        println!("   cd ~/0-core && stow -R <package>");
        println!();
        println!("   Or run: faelight-stow --fix");
    }
    
    if notify {
        send_notification(&issues);
    }
}

fn print_help() {
    println!("faelight-stow v0.1 - Stow Symlink Verification");
    println!();
    println!("USAGE:");
    println!("    faelight-stow [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --quiet     Suppress output if healthy");
    println!("    --fix       Attempt to fix issues with stow -R");
    println!("    --notify    Send notification on issues");
    println!("    -h, --help  Show this help");
    println!();
    println!("PHILOSOPHY:");
    println!("    Verify, don't auto-fix. You stay in control.");
}

fn send_notification(issues: &[Issue]) {
    let summary = format!("🔗 Stow: {} issues found", issues.len());
    let body: Vec<String> = issues.iter().map(|i| format!("{} ({})", i.path, i.package)).collect();
    
    Command::new("notify-send")
        .args([&summary, &body.join(", ")])
        .spawn()
        .ok();
}
