//! verify-bootstrap v1.0.0 - Installation Verification
//! Validates that 0-Core bootstrap completed successfully

use faelight_core::paths;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("✅ Bootstrap Verification");
    println!("{}", "━".repeat(50));
    println!();
    
    let mut passed = 0;
    let mut failed = 0;
    
    // 1. System Files
    if check_system_files() {
        println!("✅ System Files: Present");
        passed += 1;
    } else {
        println!("❌ System Files: Missing");
        failed += 1;
    }
    
    // 2. Stow Packages
    let (stow_pass, stow_count) = check_stow_packages();
    if stow_pass {
        println!("✅ Stow Packages: {} packages deployed", stow_count);
        passed += 1;
    } else {
        println!("❌ Stow Packages: Missing or incomplete");
        failed += 1;
    }
    
    // 3. Scripts Directory
    if check_scripts() {
        println!("✅ Scripts: Core scripts present");
        passed += 1;
    } else {
        println!("❌ Scripts: Missing scripts");
        failed += 1;
    }
    
    // 4. Binaries
    let (bin_pass, bin_count) = check_binaries();
    if bin_pass {
        println!("✅ Binaries: {} key tools installed", bin_count);
        passed += 1;
    } else {
        println!("❌ Binaries: Missing tools");
        failed += 1;
    }
    
    // 5. PATH Configuration
    if check_path() {
        println!("✅ PATH: Correctly configured");
        passed += 1;
    } else {
        println!("❌ PATH: Missing directories");
        failed += 1;
    }
    
    // 6. Environment
    if check_environment() {
        println!("✅ Environment: Variables set");
        passed += 1;
    } else {
        println!("❌ Environment: Variables missing");
        failed += 1;
    }
    
    println!();
    println!("{}", "━".repeat(50));
    
    let total = passed + failed;
    let percent = (passed as f32 / total as f32 * 100.0) as u32;
    
    if failed == 0 {
        println!("✅ Bootstrap Complete: {}/{} checks passed ({}%)", passed, total, percent);
        std::process::exit(0);
    } else {
        println!("⚠️  Bootstrap Incomplete: {}/{} checks passed ({}%)", passed, total, percent);
        println!();
        println!("💡 Run installation steps to complete bootstrap");
        std::process::exit(1);
    }
}


fn check_stow_packages() -> (bool, usize) {
    let stow_dir = paths::core_dir().join("03-interfaces/stow");
    
    if !stow_dir.exists() {
        return (false, 0);
    }
    
    let expected = vec![
        "shell-zsh",
        "wm-sway",
        "editor-nvim",
        "terminal-foot",
        "fm-yazi",
    ];
    
    let mut found = 0;
    for pkg in &expected {
        if stow_dir.join(pkg).exists() {
            found += 1;
        }
    }
    
    (found >= 3, found)
}

fn check_scripts() -> bool {
    let scripts_dir = paths::core_dir().join("scripts");
    
    if !scripts_dir.exists() {
        return false;
    }
    
    let required = vec!["dotctl", "profile", "bump-system-version"];
    
    for script in required {
        if !scripts_dir.join(script).exists() {
            return false;
        }
    }
    true
}

fn check_binaries() -> (bool, usize) {
    let binaries = vec![
        "dot-doctor",
        "faelight-git",
        "faelight-update",
        "faelight-stow",
        "bin-doctor",
    ];
    
    let mut found = 0;
    for bin in &binaries {
        if Command::new("which").arg(bin).output().ok()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            found += 1;
        }
    }
    
    (found >= 3, found)
}

fn check_path() -> bool {
    let path = std::env::var("PATH").unwrap_or_default();
    
    let required = vec![
        ".cargo/bin",
        "0-core/scripts",
    ];
    
    for dir in required {
        if !path.contains(dir) {
            return false;
        }
    }
    true
}

fn check_environment() -> bool {
    let vars = vec!["EDITOR", "VISUAL"];
    
    let mut found = 0;
    for var in vars {
        if std::env::var(var).is_ok() {
            found += 1;
        }
    }
    
    found >= 1
}
fn check_system_files() -> bool {
    let files = [
        "/etc/sysctl.d/99-swappiness.conf",
        "/etc/security/limits.conf",
    ];
    
    // At least one system file should exist
    files.iter().any(|f| PathBuf::from(f).exists())
}
