//! entropy-check v0.1 - Configuration Drift Detection
//! 🌲 Faelight Forest

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BASELINE_PATH: &str = ".config/faelight/entropy-baseline.json";

// ═══════════════════════════════════════════════════════════
// 📊 BASELINE STRUCTURE
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize)]
struct EntropyBaseline {
    version: String,
    created_at: String,
    system_version: String,
    config_checksums: HashMap<String, String>,
    service_states: HashMap<String, String>,
    package_versions: HashMap<String, String>,
    symlinks: HashMap<String, String>,
}

#[derive(Debug)]
struct DriftReport {
    config_drifts: Vec<String>,
    service_drifts: Vec<String>,
    binary_drifts: Vec<String>,
    symlink_drifts: Vec<String>,
}

impl EntropyBaseline {
    fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            system_version: get_system_version(),
            config_checksums: HashMap::new(),
            service_states: HashMap::new(),
            package_versions: HashMap::new(),
            symlinks: HashMap::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🔧 HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════

fn get_system_version() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let version_path = format!("{}/0-core/VERSION", home);
    fs::read_to_string(version_path)
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn get_baseline_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(BASELINE_PATH)
}

fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let contents = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn collect_symlinks() -> HashMap<String, String> {
    let mut symlinks = HashMap::new();
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return symlinks,
    };
    let config_dir = format!("{}/.config", home);
    let entries: Vec<_> = walkdir::WalkDir::new(&config_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path_is_symlink())
        .collect();
    for entry in entries {
        let path = entry.path();
        if let Ok(target) = std::fs::read_link(path) {
            symlinks.insert(path.display().to_string(), target.display().to_string());
        }
    }
    symlinks
}

fn find_broken_symlinks(baseline_symlinks: &HashMap<String, String>) -> Vec<String> {
    let mut broken = Vec::new();
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return broken,
    };
    let config_dir = format!("{}/.config", home);
    let entries: Vec<_> = walkdir::WalkDir::new(&config_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path_is_symlink())
        .collect();
    for entry in entries {
        let path = entry.path();
        let path_str = path.display().to_string();
        if let Ok(target) = std::fs::read_link(path) {
            if !target.exists() && !baseline_symlinks.contains_key(&path_str) {
                broken.push(format!("   {} → {} (new broken link)", path_str, target.display()));
            }
        }
    }
    broken
}

fn load_baseline() -> Result<EntropyBaseline, Box<dyn std::error::Error>> {
    let path = get_baseline_path();
    let contents = fs::read_to_string(path)?;
    let baseline = serde_json::from_str(&contents)?;
    Ok(baseline)
}

fn create_baseline() -> Result<EntropyBaseline, Box<dyn std::error::Error>> {
    println!("🔨 Creating baseline...");
    let mut baseline = EntropyBaseline::new();
    
    // 1. Hash stowed config files
    println!("   📁 Scanning config files...");
    let home = std::env::var("HOME")?;
    let configs = vec![
        format!("{}/.config/sway/config", home),
        format!("{}/.zshrc", home),
        format!("{}/.config/foot/foot.ini", home),
        format!("{}/.config/starship.toml", home),
    ];
    
    for config_path in configs {
        if let Ok(hash) = hash_file(Path::new(&config_path)) {
            baseline.config_checksums.insert(config_path, hash);
        }
    }
    
    // 2. Check service states
    println!("   ⚙️  Checking services...");
    let services = vec!["faelight-notify.service", "faelight-bar.service"];
    for service in services {
        if let Ok(output) = Command::new("systemctl")
            .args(["--user", "is-active", service])
            .output()
        {
            let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
            baseline.service_states.insert(service.to_string(), state);
        }
    }
    
    // 3. Get package versions (key packages only)
    println!("   📦 Recording package versions...");
    let packages = vec!["sway", "foot", "neovim", "rust"];
    for pkg in packages {
        if let Ok(output) = Command::new("pacman").args(["-Q", pkg]).output() {
            let version = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .nth(1)
                .unwrap_or("unknown")
                .to_string();
            baseline.package_versions.insert(pkg.to_string(), version);
        }
    }
    
    // 4. Collect symlinks
    println!("   🔗 Scanning symlinks...");
    baseline.symlinks = collect_symlinks();
    
    println!("   ✅ Baseline created");
    Ok(baseline)
}

fn save_baseline(baseline: &EntropyBaseline) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_baseline_path();
    
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let json = serde_json::to_string_pretty(baseline)?;
    fs::write(&path, json)?;
    
    println!("💾 Baseline saved to: {}", path.display());
    Ok(())
}

fn check_drift(baseline: &EntropyBaseline) -> Result<DriftReport, Box<dyn std::error::Error>> {
    let mut report = DriftReport {
        config_drifts: Vec::new(),
        service_drifts: Vec::new(),
        binary_drifts: Vec::new(),
        symlink_drifts: Vec::new(),
    };
    
    // 1. Check config file changes
    for (path, old_hash) in &baseline.config_checksums {
        if let Ok(current_hash) = hash_file(Path::new(path)) {
            if &current_hash != old_hash {
                report.config_drifts.push(format!("   {} (modified)", path));
            }
        } else {
            report.config_drifts.push(format!("   {} (missing)", path));
        }
    }
    
    // 2. Check service state changes
    for (service, old_state) in &baseline.service_states {
        if let Ok(output) = Command::new("systemctl")
            .args(["--user", "is-active", service])
            .output()
        {
            let current_state = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if &current_state != old_state {
                report.service_drifts.push(format!(
                    "   {} ({} → {})",
                    service, old_state, current_state
                ));
            }
        }
    }
    
    // 3. Check package version changes
    for (pkg, old_version) in &baseline.package_versions {
        if let Ok(output) = Command::new("pacman").args(["-Q", pkg]).output() {
            let current_version = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .nth(1)
                .unwrap_or("unknown")
                .to_string();
            if &current_version != old_version {
                report.binary_drifts.push(format!(
                    "   {} ({} → {})",
                    pkg, old_version, current_version
                ));
            }
        }
    }
    
    // 4. Check for broken symlinks
    report.symlink_drifts = find_broken_symlinks(&baseline.symlinks);
    
    Ok(report)
}

fn report_drift(baseline: &EntropyBaseline, drift: &DriftReport) {
    println!("📊 Drift Report - 0-Core v{}", get_system_version());
    println!("Baseline created: {}", baseline.created_at.split('T').next().unwrap_or("unknown"));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    // Config drift
    if drift.config_drifts.is_empty() {
        println!("✅ Config Drift (0 files)");
    } else {
        println!("⚠️  Config Drift ({} files)", drift.config_drifts.len());
        for item in &drift.config_drifts {
            println!("{}", item);
        }
    }
    println!();
    
    // Service drift
    if drift.service_drifts.is_empty() {
        println!("✅ Service Drift (0 changes)");
    } else {
        println!("⚠️  Service Drift ({} changes)", drift.service_drifts.len());
        for item in &drift.service_drifts {
            println!("{}", item);
        }
    }
    println!();
    
    // Binary drift
    if drift.binary_drifts.is_empty() {
        println!("✅ Binary Drift (0 changes)");
    } else {
        println!("⚠️  Binary Drift ({} changes)", drift.binary_drifts.len());
        for item in &drift.binary_drifts {
            println!("{}", item);
        }
    }
    println!();
    
    // Symlink drift
    if drift.symlink_drifts.is_empty() {
        println!("✅ Symlink Drift (0 new breaks)");
    } else {
        println!("⚠️  Symlink Drift ({} new breaks)", drift.symlink_drifts.len());
        for item in &drift.symlink_drifts {
            println!("{}", item);
        }
    }
    println!();
    
    // Summary
    let total_drifts = drift.config_drifts.len() + drift.service_drifts.len() + drift.binary_drifts.len() + drift.symlink_drifts.len();
    if total_drifts == 0 {
        println!("✨ No drift detected. System stable.");
    } else {
        println!("💡 Recommendation:");
        if !drift.config_drifts.is_empty() {
            println!("   Review modified configs, consider restowing");
        }
        if !drift.service_drifts.is_empty() {
            println!("   Check service states, restart if needed");
        }
        if !drift.binary_drifts.is_empty() {
            println!("   Package updates detected, run entropy-check --baseline to update");
        }
        if !drift.symlink_drifts.is_empty() {
            println!("   New broken symlinks detected, investigate and repair");
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🚀 MAIN
// ═══════════════════════════════════════════════════════════

fn main() {
    println!("🔍 entropy-check v0.1 - Configuration Drift Detection");
    println!();
    
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--baseline" {
        match create_baseline() {
            Ok(baseline) => {
                if let Err(e) = save_baseline(&baseline) {
                    eprintln!("❌ Error saving baseline: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("❌ Error creating baseline: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Load baseline and check for drift
        match load_baseline() {
            Ok(baseline) => {
                match check_drift(&baseline) {
                    Ok(drift) => report_drift(&baseline, &drift),
                    Err(e) => {
                        eprintln!("❌ Error checking drift: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(_) => {
                eprintln!("❌ No baseline found!");
                eprintln!("   Create one with: entropy-check --baseline");
                std::process::exit(1);
            }
        }
    }
}
