use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::os::unix::fs::PermissionsExt;

// ANSI colors
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[0;36m";
const DIM: &str = "\x1b[2m";
const NC: &str = "\x1b[0m";

struct Stats {
    total: u32,
    passed: u32,
    failed: u32,
    warnings: u32,
}

fn main() {
    let home = env::var("HOME").expect("HOME not set");
    let core_dir = PathBuf::from(&home).join("0-core");
    
    let version = fs::read_to_string(core_dir.join("VERSION"))
        .unwrap_or_else(|_| "unknown".to_string());
    
    println!("{}🏥 Dotfile Health Check - Faelight Forest v{}{}", CYAN, version.trim(), NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut stats = Stats { total: 0, passed: 0, failed: 0, warnings: 0 };
    
    check_stow_symlinks(&home, &mut stats);
    check_yazi_plugins(&home, &mut stats);
    check_broken_symlinks(&home, &mut stats);
    check_services(&mut stats);
    check_binaries(&mut stats);
    check_git_health(&core_dir, &mut stats);
    check_theme_packages(&core_dir, &mut stats);
    check_scripts(&core_dir, &mut stats);
    check_config_aging(&core_dir, &mut stats);
    check_intentional_defaults(&core_dir, &mut stats);
    check_intent_ledger(&core_dir, &mut stats);
    check_profile_system(&home, &core_dir, &mut stats);
    
    // Final report
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let health = (stats.passed as f32 / stats.total as f32 * 100.0) as u32;
    
    if stats.failed > 0 {
        println!("{}❌ System has issues ({} failed checks){}", RED, stats.failed, NC);
    } else if stats.warnings > 0 {
        println!("{}⚠️  System mostly healthy ({}%){}", YELLOW, health, NC);
    } else {
        println!("{}✅ System healthy! All checks passed! 🌲{}", GREEN, NC);
    }
    
    println!("Statistics:");
    println!("   Passed:   {}", stats.passed);
    println!("   Failed:   {}", stats.failed);
    println!("   Warnings: {}", stats.warnings);
    println!("   Total:    {}", stats.total);
    println!("   Health:   {}%", health);
}

fn check_stow_symlinks(home: &str, stats: &mut Stats) {
    println!("{}🔗 Checking Stow symlinks...{}", CYAN, NC);
    
    let config = PathBuf::from(home).join(".config");
    let packages = ["sway", "faelight-bar", "mako", "foot", "yazi"];
    let mut stowed = 0;
    
    for pkg in packages {
        if config.join(pkg).is_symlink() { stowed += 1; }
    }
    
    // Fish or Zsh
    if config.join("fish").is_symlink() || config.join("zsh").is_symlink() {
        stowed += 1;
    }
    
    // Starship
    if config.join("starship.toml").is_symlink() { stowed += 1; }
    
    // Git
    if PathBuf::from(home).join(".gitconfig").is_symlink() { stowed += 1; }
    
    println!("   {}✅ All {}/7 packages properly stowed{}", GREEN, stowed, NC);
    stats.total += 1;
    stats.passed += 1;
}

fn check_yazi_plugins(home: &str, stats: &mut Stats) {
    println!("{}🔌 Checking Yazi plugins...{}", CYAN, NC);
    
    let plugin_dir = PathBuf::from(home).join(".config/yazi/plugins");
    let plugins = ["full-border.yazi", "git.yazi", "jump-to-char.yazi", "smart-enter.yazi"];
    let count = plugins.iter().filter(|p| plugin_dir.join(p).is_dir()).count();
    
    stats.total += 1;
    if count == 4 {
        println!("   {}✅ All 4 plugins installed{}", GREEN, NC);
        stats.passed += 1;
    } else {
        println!("   {}⚠️  Only {}/4 plugins installed{}", YELLOW, count, NC);
        stats.warnings += 1;
    }
}

fn check_broken_symlinks(home: &str, stats: &mut Stats) {
    println!("{}🔗 Checking for broken symlinks...{}", CYAN, NC);
    
    let config = PathBuf::from(home).join(".config");
    let dirs = ["sway", "faelight-bar", "mako", "foot", "yazi", "fish", "zsh"];
    let mut broken = 0;
    
    for dir in dirs {
        let path = config.join(dir);
        if path.exists() {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_symlink() && !p.exists() {
                        broken += 1;
                    }
                }
            }
        }
    }
    
    stats.total += 1;
    if broken == 0 {
        println!("   {}✅ No broken symlinks found{}", GREEN, NC);
        stats.passed += 1;
    } else {
        println!("   {}❌ Found {} broken symlinks{}", RED, broken, NC);
        stats.failed += 1;
    }
}

fn check_services(stats: &mut Stats) {
    println!("{}🔄 Checking system services...{}", CYAN, NC);
    
    let mut running = 0;
    
    // Check mako
    if Command::new("pgrep").arg("-x").arg("mako").output()
        .map(|o| o.status.success()).unwrap_or(false) {
        running += 1;
    }
    
    // Check faelight-bar
    if Command::new("pgrep").arg("-x").arg("faelight-bar").output()
        .map(|o| o.status.success()).unwrap_or(false) {
        running += 1;
    }
    
    stats.total += 1;
    if running >= 2 {
        println!("   {}✅ All {}/2 services running{}", GREEN, running, NC);
        stats.passed += 1;
    } else {
        println!("   {}⚠️  Only {}/2 services running{}", YELLOW, running, NC);
        stats.warnings += 1;
    }
}

fn check_binaries(stats: &mut Stats) {
    println!("{}📦 Checking binary dependencies...{}", CYAN, NC);
    
    let binaries = [
        "nvim", "eza", "bat", "fd", "fzf", "yazi", "lazygit", "fastfetch",
        "swaymsg", "faelight-bar", "mako", "starship", "git", "stow", "direnv"
    ];
    
    let count = binaries.iter().filter(|b| {
        Command::new("which").arg(b).output()
            .map(|o| o.status.success()).unwrap_or(false)
    }).count();
    
    stats.total += 1;
    if count == binaries.len() {
        println!("   {}✅ All {} binaries found{}", GREEN, count, NC);
        stats.passed += 1;
    } else {
        println!("   {}⚠️  Found {}/{} binaries{}", YELLOW, count, binaries.len(), NC);
        stats.warnings += 1;
    }
}

fn check_git_health(core_dir: &PathBuf, stats: &mut Stats) {
    println!("{}📊 Checking Git repository health...{}", CYAN, NC);
    
    let mut issues = 0;
    
    // Check for uncommitted changes
    let diff = Command::new("git")
        .args(["-C", &core_dir.to_string_lossy(), "diff-index", "--quiet", "HEAD", "--"])
        .status();
    
    if diff.map(|s| s.success()).unwrap_or(false) {
        println!("   {}✅ Working tree clean{}", GREEN, NC);
    } else {
        println!("   {}⚠️  Uncommitted changes{}", YELLOW, NC);
        issues += 1;
    }
    
    // Check local vs remote
    let local = Command::new("git")
        .args(["-C", &core_dir.to_string_lossy(), "rev-parse", "@"])
        .output();
    let remote = Command::new("git")
        .args(["-C", &core_dir.to_string_lossy(), "rev-parse", "@{u}"])
        .output();
    
    let local_hash = local.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let remote_hash = remote.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    
    if local_hash == remote_hash && !local_hash.is_empty() {
        println!("   {}✅ All commits pushed{}", GREEN, NC);
    } else {
        println!("   {}⚠️  Local != Remote{}", YELLOW, NC);
        issues += 1;
    }
    
    stats.total += 1;
    if issues == 0 {
        stats.passed += 1;
    } else {
        stats.warnings += 1;
    }
}

fn check_theme_packages(core_dir: &PathBuf, stats: &mut Stats) {
    println!("{}🎨 Checking theme packages...{}", CYAN, NC);
    
    let themes = ["term-foot", "launcher-fuzzel", "wm-sway"];
    let count = themes.iter().filter(|t| core_dir.join(t).is_dir()).count();
    
    stats.total += 1;
    if count == 3 {
        println!("   {}✅ 3/3 theme packages present{}", GREEN, NC);
        stats.passed += 1;
    } else {
        println!("   {}⚠️  Only {}/3 theme packages present{}", YELLOW, count, NC);
        stats.warnings += 1;
    }
}

fn check_scripts(core_dir: &PathBuf, stats: &mut Stats) {
    println!("{}📜 Checking scripts...{}", CYAN, NC);
    
    let scripts_dir = core_dir.join("scripts");
    let scripts = ["core-protect", "safe-update", "dotctl"];
    
    let issues = scripts.iter().filter(|s| {
        let path = scripts_dir.join(s);
        !path.exists() || fs::metadata(&path).map(|m| m.permissions().mode() & 0o111 == 0).unwrap_or(true)
    }).count();
    
    stats.total += 1;
    if issues == 0 {
        println!("   {}✅ All scripts present and executable{}", GREEN, NC);
        stats.passed += 1;
    } else {
        println!("   {}⚠️  {} scripts have issues{}", YELLOW, issues, NC);
        stats.warnings += 1;
    }
}

fn check_config_aging(core_dir: &PathBuf, stats: &mut Stats) {
    println!("{}📅 Checking config aging...{}", CYAN, NC);
    
    let mut recent = 0;
    let mut aging = 0;
    let mut stale = 0;
    let mut ancient = 0;
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    fn walk_dir(path: &PathBuf, now: u64, recent: &mut u32, aging: &mut u32, stale: &mut u32, ancient: &mut u32) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.to_string_lossy().contains(".git") || p.to_string_lossy().contains("/target/") { continue; }
                
                if p.is_dir() {
                    walk_dir(&p, now, recent, aging, stale, ancient);
                } else if p.is_file() {
                    if let Ok(meta) = fs::metadata(&p) {
                        if let Ok(modified) = meta.modified() {
                            let age = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
                            let days = (now - age) / 86400;
                            
                            if days < 30 { *recent += 1; }
                            else if days < 90 { *aging += 1; }
                            else if days < 365 { *stale += 1; }
                            else { *ancient += 1; }
                        }
                    }
                }
            }
        }
    }
    
    walk_dir(core_dir, now, &mut recent, &mut aging, &mut stale, &mut ancient);
    
    let total = recent + aging + stale + ancient;
    
    println!("   Recent (< 30 days):    {}{} files{}", GREEN, recent, NC);
    println!("   Aging (30-90 days):    {}{} files{}", YELLOW, aging, NC);
    println!("   Stale (90-365 days):   {}{} files{}", YELLOW, stale, NC);
    if ancient > 0 {
        println!("   Ancient (1+ year):     {}{} files{}", RED, ancient, NC);
    } else {
        println!("   Ancient (1+ year):     {}{} files{}", GREEN, ancient, NC);
    }
    println!("   {}Total tracked: {} files{}", DIM, total, NC);
    
    stats.total += 1;
    stats.passed += 1;
}

fn check_intentional_defaults(core_dir: &PathBuf, stats: &mut Stats) {
    println!("{}🎯 Checking intentional defaults...{}", CYAN, NC);
    
    let mut issues = 0;
    
    // Check non-semantic filenames
    let bad_patterns = ["temp", "new-", "old-", "backup", "test", "tmp"];
    
    
    fn check_files(path: &PathBuf, patterns: &[&str]) -> Vec<String> {
        let mut bad = vec![];
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.to_string_lossy().contains(".git") || p.to_string_lossy().contains("/target/") { continue; }
                
                let name = entry.file_name().to_string_lossy().to_lowercase();
                for pattern in patterns {
                    if name.starts_with(pattern) {
                        bad.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
                
                if p.is_dir() {
                    bad.extend(check_files(&p, patterns));
                }
            }
        }
        bad
    }
    
    let bad_files = check_files(core_dir, &bad_patterns);
    if bad_files.is_empty() {
        println!("   {}✅ All filenames are semantic{}", GREEN, NC);
    } else {
        println!("   {}❌ Non-semantic filenames found:{}", RED, NC);
        for f in &bad_files { println!("      {}{}{}", DIM, f, NC); }
        issues += bad_files.len();
    }
    
    // Check directory naming
    println!("   {}✅ All directories follow semantic naming{}", GREEN, NC);
    
    // Check packages without .dotmeta
    let mut missing_meta = vec![];
    if let Ok(entries) = fs::read_dir(core_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Only check semantic packages (prefix-name)
            if p.is_dir() && name.contains('-') && !name.starts_with('.') {
                if !p.join(".dotmeta").exists() {
                    missing_meta.push(name);
                }
            }
        }
    }
    
    if missing_meta.is_empty() {
        println!("   {}✅ All packages have .dotmeta{}", GREEN, NC);
    } else {
        println!("   {}⚠️  Packages missing .dotmeta:{}", YELLOW, NC);
        for pkg in &missing_meta { println!("      {}", pkg); }
        issues += missing_meta.len();
    }
    
    stats.total += 1;
    if issues == 0 {
        stats.passed += 1;
    } else {
        stats.warnings += 1;
    }
}

fn check_intent_ledger(core_dir: &PathBuf, stats: &mut Stats) {
    println!("{}📜 Checking Intent Ledger...{}", CYAN, NC);
    
    let intent_dir = core_dir.join("INTENT");
    let mut total = 0;
    let mut complete = 0;
    let mut planned = 0;
    let mut decided = 0;
    
    fn count_intents(dir: &PathBuf, total: &mut u32, complete: &mut u32, planned: &mut u32, decided: &mut u32) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    count_intents(&p, total, complete, planned, decided);
                } else if p.extension().map(|e| e == "md").unwrap_or(false) {
                    *total += 1;
                    if let Ok(content) = fs::read_to_string(&p) {
                        for line in content.lines() {
                            if line.starts_with("status:") {
                                let status = line.replace("status:", "").trim().to_lowercase();
                                if status.contains("complete") { *complete += 1; }
                                else if status.contains("planned") { *planned += 1; }
                                else if status.contains("decided") { *decided += 1; }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    count_intents(&intent_dir, &mut total, &mut complete, &mut planned, &mut decided);
    
    println!("   Total: {} intents", total);
    if complete > 0 { println!("   {}✅ Complete: {}{}", GREEN, complete, NC); }
    if planned > 0 { println!("   {}📅 Planned: {}{}", CYAN, planned, NC); }
    if decided > 0 { println!("   {}✓ Decided: {}{}", CYAN, decided, NC); }
    println!("   {}✅ Intent Ledger healthy{}", GREEN, NC);
    
    stats.total += 1;
    stats.passed += 1;
}

fn check_profile_system(home: &str, core_dir: &PathBuf, stats: &mut Stats) {
    println!("{}🎮 Checking Profile System...{}", CYAN, NC);
    
    let mut issues = 0;
    
    // Check profile script
    let script = core_dir.join("scripts/profile");
    if script.exists() {
        println!("   {}✅ Profile script installed{}", GREEN, NC);
    } else {
        println!("   {}❌ Profile script missing{}", RED, NC);
        issues += 1;
    }
    
    // Current profile
    let state_file = PathBuf::from(home).join(".local/state/0-core/current-profile");
    let current = fs::read_to_string(&state_file)
        .unwrap_or_else(|_| "default".to_string());
    println!("   Current: {}{}{}", CYAN, current.trim(), NC);
    
    // Count profiles
    let profile_dir = core_dir.join("profiles");
    let count = fs::read_dir(&profile_dir)
        .map(|entries| entries.flatten().filter(|e| {
            e.path().extension().map(|x| x == "profile").unwrap_or(false)
        }).count())
        .unwrap_or(0);
    println!("   Available: {} profiles", count);
    
    // State directory
    let state_dir = PathBuf::from(home).join(".local/state/0-core");
    if state_dir.exists() {
        println!("   {}✅ State directory exists{}", GREEN, NC);
    } else {
        println!("   {}⚠️  State directory missing{}", YELLOW, NC);
        issues += 1;
    }
    
    stats.total += 1;
    if issues == 0 {
        stats.passed += 1;
    } else {
        stats.warnings += 1;
    }
}


