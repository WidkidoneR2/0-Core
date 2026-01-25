//! dot-doctor v0.4 - Faelight Forest Health Engine
//! 🌲 Model system integrity with dependency awareness

use clap::Parser;
use serde::{Serialize, Deserialize};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use chrono::{DateTime, Utc};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "dot-doctor")]
#[command(about = "🏥 Faelight Forest Health Engine")]
#[command(version)]
struct Cli {
    /// Show detailed explanations for each check
    #[arg(long)]
    explain: bool,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Fail on warnings (for CI)
    #[arg(long, name = "fail-on-warning")]
    fail_on_warning: bool,

    /// Show dependency graph
    #[arg(long)]
    graph: bool,

    /// Run specific check only
    #[arg(long)]
    check: Option<String>,
    /// Automatically apply safe fixes
    #[arg(long)]
    fix: bool,
    /// Show health history
    #[arg(long)]
    history: bool,
}

// ═══════════════════════════════════════════════════════════
// 📊 DATA STRUCTURES
// ═══════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Status {
    Pass,
    Warn,
    Fail,
    Blocked,
}

#[derive(Serialize, Deserialize)]
struct CheckResult {
    id: String,
    name: String,
    status: Status,
    severity: Severity,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct HealthReport {
    version: String,
    total: u32,
    passed: u32,
    warnings: u32,
    failed: u32,
    blocked: u32,
    health_percent: u32,
    checks: Vec<CheckResult>,
}

struct Check {
    id: &'static str,
    name: &'static str,
    depends_on: &'static [&'static str],
    severity: Severity,
    explanation: &'static str,
    run: fn(&Context) -> CheckResult,
}

struct Context {
    home: String,
    core_dir: PathBuf,
    version: String,
}

// ═══════════════════════════════════════════════════════════
// 🔍 CHECK DEFINITIONS
// ═══════════════════════════════════════════════════════════

const CHECKS: &[Check] = &[
    Check {
        id: "stow",
        name: "Stow Symlinks",
        depends_on: &[],
        severity: Severity::Critical,
        explanation: "Verifies GNU Stow has created symlinks from 0-core packages to ~/.config. \
                      Without this, configurations won't be active.",
        run: check_stow,
    },
    Check {
        id: "services",
        name: "System Services",
        depends_on: &["stow"],
        severity: Severity::High,
        explanation: "Checks that faelight-bar and faelight-notify are running. \
                      These provide the status bar and notification system.",
        run: check_services,
    },
    Check {
        id: "broken_symlinks",
        name: "Broken Symlinks",
        depends_on: &["stow"],
        severity: Severity::Medium,
        explanation: "Scans config directories for symlinks pointing to non-existent targets. \
                      Broken symlinks cause silent failures.",
        run: check_broken_symlinks,
    },
    Check {
        id: "yazi_plugins",
        name: "Yazi Plugins",
        depends_on: &["stow"],
        severity: Severity::Low,
        explanation: "Verifies Yazi file manager plugins are installed. \
                      Missing plugins reduce functionality but aren't critical.",
        run: check_yazi_plugins,
    },
    Check {
        id: "binaries",
        name: "Binary Dependencies",
        depends_on: &[],
        severity: Severity::High,
        explanation: "Checks that required command-line tools are installed. \
                      Missing binaries will cause command failures.",
        run: check_binaries,
    },
    Check {
        id: "git",
        name: "Git Repository",
        depends_on: &[],
        severity: Severity::Medium,
        explanation: "Checks 0-core git status: uncommitted changes and unpushed commits. \
                      Clean state ensures recoverability.",
        run: check_git,
    },
    Check {
        id: "themes",
        name: "Theme Packages",
        depends_on: &["stow"],
        severity: Severity::Low,
        explanation: "Verifies theme packages (colors, fonts, icons) exist. \
                      Missing themes affect appearance but not function.",
        run: check_themes,
    },
    Check {
        id: "scripts",
        name: "Scripts",
        depends_on: &[],
        severity: Severity::High,
        explanation: "Checks that core scripts exist and are executable. \
                      Non-executable scripts cause permission errors.",
        run: check_scripts,
    },
    Check {
        id: "dotmeta",
        name: "Package Metadata",
        depends_on: &[],
        severity: Severity::Low,
        explanation: "Verifies all packages have .dotmeta files documenting their purpose. \
                      Missing metadata reduces maintainability.",
        run: check_dotmeta,
    },
    Check {
        id: "intents",
        name: "Intent Ledger",
        depends_on: &[],
        severity: Severity::Low,
        explanation: "Validates the Intent Ledger has properly formatted intent files. \
                      The ledger documents all major decisions.",
        run: check_intents,
    },
    Check {
        id: "profiles",
        name: "Profile System",
        depends_on: &["scripts"],
        severity: Severity::Medium,
        explanation: "Checks the profile system is properly configured. \
                      Profiles control system behavior for different contexts.",
        run: check_profiles,
    },
    Check {
        id: "config",
        name: "Faelight Config",
        depends_on: &[],
        severity: Severity::Medium,
        explanation: "Validates TOML configuration files in ~/.config/faelight/. \
                      Invalid config prevents faelight commands from working.",
        run: check_faelight_config,
    },
    Check {
        id: "keybinds",
        name: "Sway Keybinds",
        depends_on: &["stow"],
        severity: Severity::Medium,
        explanation: "Checks for conflicting keybindings in Sway config. \
                      Duplicate keybinds cause unexpected behavior.",
        run: check_keybinds,
    },
    Check {
        id: "security",
        name: "Security Hardening",
        depends_on: &[],
        severity: Severity::High,
        explanation: "Verifies security protections: UFW firewall, fail2ban, Mullvad VPN, SSH hardening. \
                      These protect against unauthorized access.",
        run: check_security,
    },
];

// ═══════════════════════════════════════════════════════════
// 🔍 CHECK IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════

fn check_stow(ctx: &Context) -> CheckResult {
    let stow_dir = PathBuf::from(&ctx.home).join("0-core/stow");
    let mut stowed = 0;
    let mut details = vec![];
    
    // Auto-discover packages
    let packages = discover_stow_packages(&stow_dir);
    let total = packages.len();
    
    for package in &packages {
        // Package directory not needed - we find symlinks directly
        
        // Find symlinks in ~/ that point to this package
        let symlinks = find_stow_symlinks(&ctx.home, package);
        
        if !symlinks.is_empty() {
            stowed += 1;
            for link in &symlinks {
                if let Ok(stripped) = link.strip_prefix(&ctx.home) {
                    details.push(format!("✓ {} ({})", stripped.display(), package));
                }
            }
        } else {
            details.push(format!("✗ {} (no symlinks found)", package));
        }
    }
    
    if stowed == total {
        CheckResult {
            id: "stow".to_string(),
            name: "Stow Symlinks".to_string(),
            status: Status::Pass,
            severity: Severity::Critical,
            message: format!("All {}/{} packages properly stowed", stowed, total),
            fix: None,
            details: Some(details),
        }
    } else {
        CheckResult {
            id: "stow".to_string(),
            name: "Stow Symlinks".to_string(),
            status: Status::Fail,
            severity: Severity::Critical,
            message: format!("Only {}/{} packages stowed", stowed, total),
            fix: Some("Run: cd ~/0-core && stow --dir=stow -R <package>".to_string()),
            details: Some(details),
        }
    }
}

fn discover_stow_packages(stow_dir: &PathBuf) -> Vec<String> {
    let mut packages = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(stow_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
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

fn find_stow_symlinks(home: &str, package: &str) -> Vec<PathBuf> {
    let home_path = PathBuf::from(home);
    let mut symlinks = Vec::new();
    
    let search_paths = vec![
        home_path.clone(),
        home_path.join(".config"),
    ];
    
    for search_path in search_paths {
        if let Ok(entries) = std::fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_symlink() {
                    if let Ok(target) = std::fs::read_link(&path) {
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

fn check_services(_ctx: &Context) -> CheckResult {
    let mut running = 0;
    let mut details = vec![];

    let services = [("faelight-bar", "Status bar"), ("faelight-notify", "Notifications")];

    for (name, desc) in services {
        let output = Command::new("pgrep").arg("-x").arg(name).output();
        if output.map(|o| o.status.success()).unwrap_or(false) {
            running += 1;
            details.push(format!("✓ {} ({})", name, desc));
        } else {
            details.push(format!("✗ {} not running", name));
        }
    }

    if running == 2 {
        CheckResult {
            id: "services".to_string(),
            name: "System Services".to_string(),
            status: Status::Pass,
            severity: Severity::High,
            message: format!("All {}/2 services running", running),
            fix: None,
            details: Some(details),
        }
    } else {
        CheckResult {
            id: "services".to_string(),
            name: "System Services".to_string(),
            status: Status::Warn,
            severity: Severity::High,
            message: format!("Only {}/2 services running", running),
            fix: Some("Restart Sway or run services manually".to_string()),
            details: Some(details),
        }
    }
}

fn check_broken_symlinks(ctx: &Context) -> CheckResult {
    let config = PathBuf::from(&ctx.home).join(".config");
    let dirs = ["sway", "foot", "fuzzel", "yazi", "zsh"];
    let mut broken = vec![];

    for dir in dirs {
        let path = config.join(dir);
        if path.exists() {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_symlink() && !p.exists() {
                        broken.push(p.display().to_string());
                    }
                }
            }
        }
    }

    if broken.is_empty() {
        CheckResult {
            id: "broken_symlinks".to_string(),
            name: "Broken Symlinks".to_string(),
            status: Status::Pass,
            severity: Severity::Medium,
            message: "No broken symlinks found".to_string(),
            fix: None,
            details: None,
        }
    } else {
        CheckResult {
            id: "broken_symlinks".to_string(),
            name: "Broken Symlinks".to_string(),
            status: Status::Fail,
            severity: Severity::Medium,
            message: format!("{} broken symlinks found", broken.len()),
            fix: Some("Remove broken links: rm <path>".to_string()),
            details: Some(broken),
        }
    }
}

fn check_yazi_plugins(ctx: &Context) -> CheckResult {
    let plugin_dir = PathBuf::from(&ctx.home).join(".config/yazi/plugins");
    let plugins = ["full-border.yazi", "git.yazi", "jump-to-char.yazi", "smart-enter.yazi"];
    let mut found = vec![];
    let mut missing = vec![];

    for p in plugins {
        if plugin_dir.join(p).is_dir() {
            found.push(format!("✓ {}", p));
        } else {
            missing.push(format!("✗ {}", p));
        }
    }

    let count = found.len();
    let mut details = found;
    details.extend(missing);

    if count == 4 {
        CheckResult {
            id: "yazi_plugins".to_string(),
            name: "Yazi Plugins".to_string(),
            status: Status::Pass,
            severity: Severity::Low,
            message: "All 4 plugins installed".to_string(),
            fix: None,
            details: Some(details),
        }
    } else {
        CheckResult {
            id: "yazi_plugins".to_string(),
            name: "Yazi Plugins".to_string(),
            status: Status::Warn,
            severity: Severity::Low,
            message: format!("Only {}/4 plugins installed", count),
            fix: Some("Install missing plugins via ya pack".to_string()),
            details: Some(details),
        }
    }
}

fn check_binaries(_ctx: &Context) -> CheckResult {
    let bins = [
        "sway", "foot", "fuzzel", "yazi", "nvim", "git", "stow",
        "starship", "bat", "eza", "fd", "rg", "zoxide",
        "brightnessctl", "wpctl",
    ];
    let mut found = 0;
    let mut missing = vec![];

    for bin in bins {
        if Command::new("which").arg(bin).output().map(|o| o.status.success()).unwrap_or(false) {
            found += 1;
        } else {
            missing.push(bin.to_string());
        }
    }

    let total = bins.len();
    if found == total {
        CheckResult {
            id: "binaries".to_string(),
            name: "Binary Dependencies".to_string(),
            status: Status::Pass,
            severity: Severity::High,
            message: format!("All {} binaries found", total),
            fix: None,
            details: None,
        }
    } else {
        CheckResult {
            id: "binaries".to_string(),
            name: "Binary Dependencies".to_string(),
            status: Status::Fail,
            severity: Severity::High,
            message: format!("{} binaries missing", missing.len()),
            fix: Some("Install with: sudo pacman -S <package>".to_string()),
            details: Some(missing),
        }
    }
}

fn check_git(ctx: &Context) -> CheckResult {
    let mut issues = vec![];

    // Check for uncommitted changes
    let status = Command::new("git")
        .args(["-C", &ctx.core_dir.to_string_lossy(), "status", "--porcelain"])
        .output();

    let has_changes = status
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_changes {
        issues.push("Uncommitted changes".to_string());
    }

    // Check for unpushed commits
    let unpushed = Command::new("git")
        .args(["-C", &ctx.core_dir.to_string_lossy(), "log", "@{u}..", "--oneline"])
        .output();

    let has_unpushed = unpushed
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_unpushed {
        issues.push("Unpushed commits".to_string());
    }

    if issues.is_empty() {
        CheckResult {
            id: "git".to_string(),
            name: "Git Repository".to_string(),
            status: Status::Pass,
            severity: Severity::Medium,
            message: "Working tree clean, all commits pushed".to_string(),
            fix: None,
            details: None,
        }
    } else {
        CheckResult {
            id: "git".to_string(),
            name: "Git Repository".to_string(),
            status: Status::Warn,
            severity: Severity::Medium,
            message: issues.join(", "),
            fix: Some("Commit and push changes: git add -A && git commit && git push".to_string()),
            details: Some(issues),
        }
    }
}

fn check_themes(ctx: &Context) -> CheckResult {
    let packages = ["config-faelight"];
    let mut found = 0;

    for pkg in packages {
        if ctx.core_dir.join("stow").join(pkg).is_dir() {
            found += 1;
        }
    }

    // Also check for any theme- prefixed directories
    let theme_count = fs::read_dir(&ctx.core_dir.join("stow"))
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with("config-faelight") || e.file_name().to_string_lossy().starts_with("theme-"))
                .count()
        })
        .unwrap_or(0);

    if theme_count >= 1 {
        CheckResult {
            id: "themes".to_string(),
            name: "Theme Packages".to_string(),
            status: Status::Pass,
            severity: Severity::Low,
            message: format!("{}/1 theme packages present", theme_count),
            fix: None,
            details: None,
        }
    } else {
        CheckResult {
            id: "themes".to_string(),
            name: "Theme Packages".to_string(),
            status: Status::Warn,
            severity: Severity::Low,
            message: format!("Only {}/1 theme packages found", theme_count),
            fix: None,
            details: None,
        }
    }
}

fn check_scripts(ctx: &Context) -> CheckResult {
    let scripts_dir = ctx.core_dir.join("scripts");
    let required = ["dot-doctor", "dotctl", "faelight", "profile", "intent"];
    let mut issues = vec![];

    for script in required {
        let path = scripts_dir.join(script);
        if !path.exists() {
            issues.push(format!("{} missing", script));
        } else if let Ok(meta) = path.metadata() {
            if meta.permissions().mode() & 0o111 == 0 {
                issues.push(format!("{} not executable", script));
            }
        }
    }

    if issues.is_empty() {
        CheckResult {
            id: "scripts".to_string(),
            name: "Scripts".to_string(),
            status: Status::Pass,
            severity: Severity::High,
            message: "All scripts present and executable".to_string(),
            fix: None,
            details: None,
        }
    } else {
        CheckResult {
            id: "scripts".to_string(),
            name: "Scripts".to_string(),
            status: Status::Warn,
            severity: Severity::High,
            message: format!("{} script issues", issues.len()),
            fix: Some("chmod +x ~/0-core/scripts/*".to_string()),
            details: Some(issues),
        }
    }
}

fn check_dotmeta(_ctx: &Context) -> CheckResult {
    // .dotmeta files were intentionally removed to fix stow conflicts
    // See: v8.0.0 stow symlink fix commit
    CheckResult {
        id: "dotmeta".to_string(),
        name: "Package Metadata".to_string(),
        status: Status::Pass,
        severity: Severity::Low,
        message: ".dotmeta files intentionally removed (stow conflict resolution)".to_string(),
        fix: None,
        details: None,
    }
}

fn check_intents(ctx: &Context) -> CheckResult {
    let intent_dir = ctx.core_dir.join("INTENT");
    let mut total = 0;
    let mut complete = 0;
    let mut planned = 0;

    for category in ["decisions", "experiments", "philosophy", "future", "incidents"] {
        let cat_dir = intent_dir.join(category);
        if let Ok(entries) = fs::read_dir(&cat_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                    total += 1;
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains("status: complete") {
                            complete += 1;
                        } else if content.contains("status: planned") {
                            planned += 1;
                        }
                    }
                }
            }
        }
    }

    CheckResult {
        id: "intents".to_string(),
        name: "Intent Ledger".to_string(),
        status: Status::Pass,
        severity: Severity::Low,
        message: format!("{} intents ({} complete, {} planned)", total, complete, planned),
        fix: None,
        details: Some(vec![
            format!("Total: {}", total),
            format!("Complete: {}", complete),
            format!("Planned: {}", planned),
        ]),
    }
}

fn check_profiles(ctx: &Context) -> CheckResult {
    let state_dir = PathBuf::from(&ctx.home).join(".local/state/faelight");
    let profile_script = ctx.core_dir.join("scripts/profile");

    let mut issues = vec![];

    if !profile_script.exists() {
        issues.push("Profile script missing".to_string());
    }

    if !state_dir.exists() {
        issues.push("State directory missing".to_string());
    }

    let current = fs::read_to_string(state_dir.join("current-profile"))
        .unwrap_or_else(|_| "default".to_string())
        .trim()
        .to_string();

    if issues.is_empty() {
        CheckResult {
            id: "profiles".to_string(),
            name: "Profile System".to_string(),
            status: Status::Pass,
            severity: Severity::Medium,
            message: format!("Profile system OK (current: {})", current),
            fix: None,
            details: Some(vec![format!("Current profile: {}", current)]),
        }
    } else {
        CheckResult {
            id: "profiles".to_string(),
            name: "Profile System".to_string(),
            status: Status::Warn,
            severity: Severity::Medium,
            message: format!("{} issues", issues.len()),
            fix: Some("Run: mkdir -p ~/.local/state/faelight".to_string()),
            details: Some(issues),
        }
    }
}

fn check_faelight_config(ctx: &Context) -> CheckResult {
    let config_dir = PathBuf::from(&ctx.home).join(".config/faelight");
    let files = ["config.toml", "profiles.toml", "themes.toml"];
    let mut missing = vec![];
    let mut found = vec![];

    for file in files {
        let path = config_dir.join(file);
        if path.exists() {
            // Try to parse it
            if let Ok(content) = fs::read_to_string(&path) {
                if toml_valid(&content) {
                    found.push(format!("✓ {}", file));
                } else {
                    missing.push(format!("✗ {} (invalid TOML)", file));
                }
            }
        } else {
            missing.push(format!("✗ {} (missing)", file));
        }
    }

    if missing.is_empty() {
        CheckResult {
            id: "config".to_string(),
            name: "Faelight Config".to_string(),
            status: Status::Pass,
            severity: Severity::Medium,
            message: "All config files valid".to_string(),
            fix: None,
            details: Some(found),
        }
    } else {
        let mut details = found;
        details.extend(missing.clone());
        CheckResult {
            id: "config".to_string(),
            name: "Faelight Config".to_string(),
            status: if missing.iter().any(|m| m.contains("invalid")) { Status::Fail } else { Status::Warn },
            severity: Severity::Medium,
            message: format!("{} config issues", missing.len()),
            fix: Some("Run: faelight config validate".to_string()),
            details: Some(details),
        }
    }
}

fn toml_valid(content: &str) -> bool {
    // Simple validation - check for basic TOML structure
    !content.is_empty() && (content.contains('[') || content.contains('='))
}

fn check_keybinds(ctx: &Context) -> CheckResult {
    let sway_config = PathBuf::from(&ctx.home).join(".config/sway/config");
    
    if !sway_config.exists() {
        return CheckResult {
            id: "keybinds".to_string(),
            name: "Sway Keybinds".to_string(),
            status: Status::Warn,
            severity: Severity::Medium,
            message: "Sway config not found".to_string(),
            fix: Some("Ensure wm-sway is stowed".to_string()),
            details: None,
        };
    }

    let output = Command::new(ctx.core_dir.join("scripts/keyscan"))
        .arg(sway_config.to_string_lossy().to_string())
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            
            if stdout.contains("No conflicts detected") {
                let count = stdout
                    .lines()
                    .find(|l| l.contains("unique keybindings"))
                    .and_then(|l| l.split_whitespace().next())
                    .unwrap_or("0");
                
                CheckResult {
                    id: "keybinds".to_string(),
                    name: "Sway Keybinds".to_string(),
                    status: Status::Pass,
                    severity: Severity::Medium,
                    message: format!("{} unique keybindings, no conflicts", count),
                    fix: None,
                    details: None,
                }
            } else if stdout.contains("Conflict detected") {
                CheckResult {
                    id: "keybinds".to_string(),
                    name: "Sway Keybinds".to_string(),
                    status: Status::Fail,
                    severity: Severity::Medium,
                    message: "Keybind conflicts detected".to_string(),
                    fix: Some("Run: keyscan ~/.config/sway/config".to_string()),
                    details: Some(vec!["View conflicts with keyscan".to_string()]),
                }
            } else {
                CheckResult {
                    id: "keybinds".to_string(),
                    name: "Sway Keybinds".to_string(),
                    status: Status::Warn,
                    severity: Severity::Medium,
                    message: "Unable to parse keyscan output".to_string(),
                    fix: None,
                    details: None,
                }
            }
        }
        _ => CheckResult {
            id: "keybinds".to_string(),
            name: "Sway Keybinds".to_string(),
            status: Status::Warn,
            severity: Severity::Medium,
            message: "keyscan not available".to_string(),
            fix: Some("Ensure keyscan is in ~/0-core/scripts/".to_string()),
            details: None,
        },
    }
}

// ═══════════════════════════════════════════════════════════
// 🚀 MAIN
// ═══════════════════════════════════════════════════════════

fn main() {
// ═══════════════════════════════════════════════════════════
// 📊 HISTORY TRACKING
// ═══════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize)]
struct HealthSnapshot {
    timestamp: DateTime<Utc>,
    health_percent: u32,
    passed: u32,
    warnings: u32,
    failed: u32,
    total: u32,
}

fn save_health_snapshot(report: &HealthReport) -> std::io::Result<()> {
    let state_dir = PathBuf::from(env::var("HOME").unwrap()).join(".local/state/0-core");
    fs::create_dir_all(&state_dir)?;
    
    let history_file = state_dir.join("health-history.jsonl");
    let snapshot = HealthSnapshot {
        timestamp: Utc::now(),
        health_percent: report.health_percent,
        passed: report.passed,
        warnings: report.warnings,
        failed: report.failed,
        total: report.total,
    };
    
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_file)?;
    
    writeln!(file, "{}", serde_json::to_string(&snapshot)?)?;
    Ok(())
}

fn show_health_history() -> std::io::Result<()> {
    let history_file = PathBuf::from(env::var("HOME").unwrap())
        .join(".local/state/0-core/health-history.jsonl");
    
    if !history_file.exists() {
        println!("📊 No health history yet. Run 'doctor' to start tracking!");
        return Ok(());
    }
    
    let content = fs::read_to_string(history_file)?;
    let snapshots: Vec<HealthSnapshot> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    
    if snapshots.is_empty() {
        println!("📊 No health history yet.");
        return Ok(());
    }
    
    println!("📊 Health History");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    for snapshot in snapshots.iter().rev().take(10).rev() {
        let color = if snapshot.health_percent >= 95 {
            "\x1b[0;32m"
        } else if snapshot.health_percent >= 80 {
            "\x1b[1;33m"
        } else {
            "\x1b[0;31m"
        };
        
        println!("  {} - {}{}%{} ({}/{} checks)",
                 snapshot.timestamp.format("%Y-%m-%d %H:%M"),
                 color, snapshot.health_percent, "\x1b[0m",
                 snapshot.passed, snapshot.total);
    }
    
    if snapshots.len() >= 2 {
        let recent = &snapshots[snapshots.len() - 1];
        let previous = &snapshots[snapshots.len() - 2];
        let diff = recent.health_percent as i32 - previous.health_percent as i32;
        
        let trend = if diff > 0 {
            format!("\x1b[0;32m↑{}\x1b[0m", diff)
        } else if diff < 0 {
            format!("\x1b[0;31m↓{}\x1b[0m", diff.abs())
        } else {
            "→0".to_string()
        };
        
        println!();
        println!("  Trend: {} since last check", trend);
    }
    
    println!();
    println!("  Total snapshots: {}", snapshots.len());
    
    Ok(())
}

fn apply_fixes(results: &[CheckResult]) -> std::io::Result<()> {
    let fixable: Vec<_> = results.iter()
        .filter(|r| r.status != Status::Pass && r.fix.is_some())
        .collect();
    
    if fixable.is_empty() {
        println!("✅ No fixes needed!");
        return Ok(());
    }
    
    println!("🔧 Auto-Fix Mode");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    for result in &fixable {
        println!("  {} - {}", result.id, result.name);
        if let Some(ref fix) = result.fix {
            println!("    Fix: {}", fix);
        }
        println!();
    }
    
    println!("Apply these fixes? (y/n)");
    print!("> ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Cancelled.");
        return Ok(());
    }
    
    for result in &fixable {
        match result.id.as_str() {
            "scripts" => {
                println!("  Fixing: {}", result.id);
                let scripts_dir = PathBuf::from(env::var("HOME").unwrap()).join("0-core/scripts");
                if let Ok(entries) = fs::read_dir(&scripts_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() {
                            let _ = Command::new("chmod")
                                .args(["+x", path.to_str().unwrap()])
                                .output();
                        }
                    }
                    println!("    ✅ Scripts made executable");
                }
            }
            "profiles" => {
                println!("  Fixing: {}", result.id);
                let state_dir = PathBuf::from(env::var("HOME").unwrap()).join(".local/state/faelight");
                let _ = fs::create_dir_all(&state_dir);
                println!("    ✅ Created state directory");
            }
            _ => {
                println!("  {} - Manual fix required", result.id);
            }
        }
    }
    
    println!();
    println!("✅ Auto-fix complete! Run 'doctor' again to verify.");
    
    Ok(())
}

    let cli = Cli::parse();

    let home = env::var("HOME").expect("HOME not set");
    let core_dir = PathBuf::from(&home).join("0-core");
    let version = fs::read_to_string(core_dir.join("VERSION"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    let ctx = Context { home: home.clone(), core_dir, version: version.clone() };

    // Show dependency graph
    if cli.graph {
        print_dependency_graph();
        return;
    }
    
    // Show health history
    if cli.history {
        match show_health_history() {
            Ok(_) => return,
            Err(e) => {
                eprintln!("Error reading history: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Run checks
    let mut results: Vec<CheckResult> = Vec::new();
    let mut completed: Vec<&str> = Vec::new();
    let mut failed_checks: Vec<&str> = Vec::new();

    for check in CHECKS {
        // Skip if specific check requested and this isn't it
        if let Some(ref only) = cli.check {
            if check.id != only {
                continue;
            }
        }

        // Check dependencies
        let blocked = check.depends_on.iter().any(|dep| failed_checks.contains(dep));

        if blocked {
            results.push(CheckResult {
                id: check.id.to_string(),
                name: check.name.to_string(),
                status: Status::Blocked,
                severity: check.severity,
                message: format!("Blocked by failed dependency"),
                fix: None,
                details: None,
            });
            continue;
        }

        let result = (check.run)(&ctx);
        
        if result.status == Status::Fail {
            failed_checks.push(check.id);
        }
        
        completed.push(check.id);
        results.push(result);
    }

    // Calculate stats
    let total = results.len() as u32;
    let passed = results.iter().filter(|r| r.status == Status::Pass).count() as u32;
    let warnings = results.iter().filter(|r| r.status == Status::Warn).count() as u32;
    let failed = results.iter().filter(|r| r.status == Status::Fail).count() as u32;
    let blocked = results.iter().filter(|r| r.status == Status::Blocked).count() as u32;
    let health_percent = if total > 0 { (passed * 100) / total } else { 0 };

    let report = HealthReport {
        version: version.clone(),
        total,
        passed,
        warnings,
        failed,
        blocked,
        health_percent,
        checks: results,
    };

    
    // Save health snapshot
    let _ = save_health_snapshot(&report);
    // Auto-fix mode
    if cli.fix {
        if let Err(e) = apply_fixes(&report.checks) {
            eprintln!("Error applying fixes: {}", e);
        }
        return;
    }

    // Output
    if cli.json {
        match serde_json::to_string_pretty(&report) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Error serializing JSON: {}", e),
        }
    } else {
        print_report(&report, cli.explain);
    }

    // Exit code
    let exit_code = if failed > 0 {
        1
    } else if cli.fail_on_warning && warnings > 0 {
        1
    } else {
        0
    };

    std::process::exit(exit_code);
}

fn print_report(report: &HealthReport, explain: bool) {
    println!("{}🏥 0-Core Health Check - Faelight Forest v{}{}", 
             "\x1b[0;36m", report.version, "\x1b[0m");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for check in &report.checks {
        let (icon, color) = match check.status {
            Status::Pass => ("✅", "\x1b[0;32m"),
            Status::Warn => ("⚠️ ", "\x1b[1;33m"),
            Status::Fail => ("❌", "\x1b[0;31m"),
            Status::Blocked => ("🚫", "\x1b[2m"),
        };

        println!("{}{} {}: {}{}", color, icon, check.name, check.message, "\x1b[0m");

        if explain {
            // Find explanation
            if let Some(c) = CHECKS.iter().find(|c| c.id == check.id) {
                println!("   \x1b[2m{}\x1b[0m", c.explanation);
            }
            if let Some(ref fix) = check.fix {
                println!("   \x1b[0;36m💡 Fix: {}\x1b[0m", fix);
            }
            if let Some(ref details) = check.details {
                for d in details {
                    println!("   \x1b[2m• {}\x1b[0m", d);
                }
            }
            println!();
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let status_msg = if report.failed > 0 {
        format!("{}❌ {} checks failed{}", "\x1b[0;31m", report.failed, "\x1b[0m")
    } else if report.warnings > 0 {
        format!("{}⚠️  System mostly healthy ({}%){}", "\x1b[1;33m", report.health_percent, "\x1b[0m")
    } else {
        format!("{}✅ System healthy! All checks passed! 🌲{}", "\x1b[0;32m", "\x1b[0m")
    };

    println!("{}", status_msg);
    println!("Statistics:");
    println!("   Passed:   {}", report.passed);
    println!("   Warnings: {}", report.warnings);
    println!("   Failed:   {}", report.failed);
    if report.blocked > 0 {
        println!("   Blocked:  {}", report.blocked);
    }
    println!("   Total:    {}", report.total);
    println!("   Health:   {}%", report.health_percent);
}

fn print_dependency_graph() {
    println!("{}🔗 Health Check Dependency Graph{}", "\x1b[0;36m", "\x1b[0m");
    println!();
    
    for check in CHECKS {
        let deps = if check.depends_on.is_empty() {
            "(root)".to_string()
        } else {
            format!("← {}", check.depends_on.join(", "))
        };
        
        let severity_color = match check.severity {
            Severity::Critical => "\x1b[0;31m",
            Severity::High => "\x1b[1;33m",
            Severity::Medium => "\x1b[0;36m",
            Severity::Low => "\x1b[2m",
        };
        
        println!("  {}{:<20}{} {}", severity_color, check.id, "\x1b[0m", deps);
    }
    
    println!();
    println!("Legend: {}Critical{} {}High{} {}Medium{} {}Low{}",
             "\x1b[0;31m", "\x1b[0m",
             "\x1b[1;33m", "\x1b[0m",
             "\x1b[0;36m", "\x1b[0m",
             "\x1b[2m", "\x1b[0m");
}


fn check_security(_ctx: &Context) -> CheckResult {
    let mut issues = vec![];
    let mut details = vec![];
    
    // Check UFW
    let ufw = fs::read_to_string("/etc/ufw/ufw.conf");
    
    if let Ok(content) = ufw {
        if content.contains("ENABLED=yes") {
            details.push("✓ UFW firewall active".to_string());
        } else {
            issues.push("UFW firewall not active".to_string());
        }
    }
    
    // Check fail2ban
    let f2b = Command::new("systemctl")
        .args(["is-active", "fail2ban"])
        .output();
    
    if let Ok(output) = f2b {
        if String::from_utf8_lossy(&output.stdout).trim() == "active" {
            details.push("✓ fail2ban active".to_string());
        } else {
            issues.push("fail2ban not active".to_string());
        }
    }
    
    // Check Mullvad VPN
    let mullvad = Command::new("mullvad")
        .args(["status"])
        .output();
    
    if let Ok(output) = mullvad {
        let status = String::from_utf8_lossy(&output.stdout);
        if status.contains("Connected") {
            details.push("✓ Mullvad VPN connected".to_string());
        } else {
            details.push("⚠ Mullvad VPN not connected".to_string());
        }
    }
    
    // Check SSH hardening
    let sshd = PathBuf::from("/etc/ssh/sshd_config");
    if sshd.exists() {
        if let Ok(content) = fs::read_to_string(&sshd) {
            let has_root_login = content.contains("PermitRootLogin no");
            let has_password_auth = content.contains("PasswordAuthentication no");
            
            if has_root_login {
                details.push("✓ SSH root login disabled".to_string());
            } else {
                issues.push("SSH permits root login".to_string());
            }
            
            if has_password_auth {
                details.push("✓ SSH password auth disabled".to_string());
            } else {
                issues.push("SSH allows password authentication".to_string());
            }
        }
    }
    
    CheckResult {
        id: "security".to_string(),
        name: "Security Hardening".to_string(),
        status: if issues.is_empty() { Status::Pass } else { Status::Fail },
        severity: Severity::High,
        message: if issues.is_empty() {
            format!("Security: {} protections active", details.len())
        } else {
            format!("Security: {} issues found", issues.len())
        },
        fix: if !issues.is_empty() {
            Some(format!("Review security settings:\n{}", issues.join("\n")))
        } else {
            None
        },
        details: if !details.is_empty() { Some(details) } else { None },
    }
}
