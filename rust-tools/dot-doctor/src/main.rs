//! dot-doctor v0.4 - Faelight Forest Health Engine
//! 🌲 Model system integrity with dependency awareness

use clap::Parser;
use colored::*;
use serde::Serialize;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

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
}

// ═══════════════════════════════════════════════════════════
// 📊 DATA STRUCTURES
// ═══════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq, Serialize)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Clone, Copy, PartialEq, Serialize)]
enum Status {
    Pass,
    Warn,
    Fail,
    Blocked,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
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
];

// ═══════════════════════════════════════════════════════════
// 🔍 CHECK IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════

fn check_stow(ctx: &Context) -> CheckResult {
    let config = PathBuf::from(&ctx.home).join(".config");
    let mut stowed = 0;
    let mut details = vec![];
    let checks = [
        ("zsh/.zshrc", "shell-zsh"),
        ("sway/config", "wm-sway"),
        ("foot/foot.ini", "term-foot"),
        ("yazi/yazi.toml", "fm-yazi"),
        ("starship.toml", "prompt-starship"),
        ("topgrade.toml", "tools-topgrade"),
        ("qutebrowser/config.py", "browser-qutebrowser"),
        ("faelight/config.toml", "config-faelight"),
        ("nvim/init.lua", "editor-nvim"),
        ("nushell/config.nu", "shell-nushell"),
    ];
    for (path, pkg) in checks {
        let target = config.join(path);
        let core_source = PathBuf::from(&ctx.home).join("0-core/stow").join(pkg).join(".config").join(path);
        
        if let (Ok(resolved_target), Ok(resolved_source)) = (std::fs::canonicalize(&target), std::fs::canonicalize(&core_source)) {
            if resolved_target == resolved_source {
                stowed += 1;
                details.push(format!("✓ {} ({})", path, pkg));
            } else {
                details.push(format!("✗ {} wrong target", path));
            }
        } else {
            details.push(format!("✗ {} missing", path));
        }
    }
    // Check .gitconfig
    if PathBuf::from(&ctx.home).join(".gitconfig").is_symlink() {
        stowed += 1;
        details.push("✓ .gitconfig (vcs-git)".to_string());
    } else {
        details.push("✗ .gitconfig missing".to_string());
    }
    let total = 11;
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
            fix: Some("Run: cd ~/0-core && stow <package-name>".to_string()),
            details: Some(details),
        }
    }
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
        if ctx.core_dir.join(pkg).is_dir() {
            found += 1;
        }
    }

    // Also check for any theme- prefixed directories
    let theme_count = fs::read_dir(&ctx.core_dir)
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

fn check_dotmeta(ctx: &Context) -> CheckResult {
    let mut missing = vec![];

    if let Ok(entries) = fs::read_dir(&ctx.core_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                // Skip non-package directories
                if name.starts_with('.') || name == "scripts" || name == "docs" 
                    || name == "INTENT" || name == "rust-tools" || name == "target" {
                    continue;
                }
                if !path.join(".dotmeta").exists() {
                    missing.push(name);
                }
            }
        }
    }

    if missing.is_empty() {
        CheckResult {
            id: "dotmeta".to_string(),
            name: "Package Metadata".to_string(),
            status: Status::Pass,
            severity: Severity::Low,
            message: "All packages have .dotmeta".to_string(),
            fix: None,
            details: None,
        }
    } else {
        CheckResult {
            id: "dotmeta".to_string(),
            name: "Package Metadata".to_string(),
            status: Status::Warn,
            severity: Severity::Low,
            message: format!("{} packages missing .dotmeta", missing.len()),
            fix: Some("Create .dotmeta files for packages".to_string()),
            details: Some(missing),
        }
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
