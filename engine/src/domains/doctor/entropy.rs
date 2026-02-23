//! entropy-check — absorbed from rust-tools/entropy-check
use crate::errors::CoreResult;
use chrono::Utc;
use faelight_core::paths;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

#[derive(Debug, Serialize)]
struct DriftReport {
    config_drifts: Vec<String>,
    service_drifts: Vec<String>,
    binary_drifts: Vec<String>,
    symlink_drifts: Vec<String>,
    untracked_files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DriftHistory {
    version: String,
    entries: Vec<DriftEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DriftEntry {
    timestamp: String,
    system_version: String,
    total_drifts: usize,
    config_drifts: usize,
    service_drifts: usize,
    binary_drifts: usize,
    symlink_drifts: usize,
    untracked_files: usize,
}

fn get_system_version() -> String {
    fs::read_to_string(paths::version_file())
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let contents = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn collect_symlinks() -> HashMap<String, String> {
    let mut symlinks = HashMap::new();
    let config_dir = home().join(".config");
    for entry in walkdir::WalkDir::new(&config_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path_is_symlink())
    {
        if let Ok(target) = fs::read_link(entry.path()) {
            symlinks.insert(
                entry.path().display().to_string(),
                target.display().to_string(),
            );
        }
    }
    symlinks
}

fn find_broken_symlinks(baseline_symlinks: &HashMap<String, String>) -> Vec<String> {
    let mut broken = Vec::new();
    let config_dir = home().join(".config");
    for entry in walkdir::WalkDir::new(&config_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path_is_symlink())
    {
        let path_str = entry.path().display().to_string();
        if !baseline_symlinks.contains_key(&path_str) && !entry.path().exists() {
            broken.push(path_str);
        }
    }
    broken
}

fn create_baseline() -> CoreResult<EntropyBaseline> {
    println!("🔨 Creating baseline...");
    let mut baseline = EntropyBaseline {
        version: "1.0".to_string(),
        created_at: Utc::now().to_rfc3339(),
        system_version: get_system_version(),
        config_checksums: HashMap::new(),
        service_states: HashMap::new(),
        package_versions: HashMap::new(),
        symlinks: HashMap::new(),
    };
    let config_dir = home().join(".config");
    println!("   📁 Scanning config files...");
    for entry in walkdir::WalkDir::new(&config_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(hash) = hash_file(entry.path()) {
            baseline
                .config_checksums
                .insert(entry.path().display().to_string(), hash);
        }
    }
    println!("   ⚙️  Checking services...");
    if let Ok(output) = Command::new("systemctl")
        .args([
            "--user",
            "list-units",
            "--type=service",
            "--all",
            "--plain",
            "--no-legend",
        ])
        .output()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                baseline
                    .service_states
                    .insert(parts[0].to_string(), parts[3].to_string());
            }
        }
    }
    println!("   📦 Recording package versions...");
    for pkg in ["sway", "waybar", "neovim", "zsh", "kitty"] {
        if let Ok(output) = Command::new("pacman").args(["-Q", pkg]).output() {
            if output.status.success() {
                baseline.package_versions.insert(
                    pkg.to_string(),
                    String::from_utf8_lossy(&output.stdout).trim().to_string(),
                );
            }
        }
    }
    println!("   🔗 Scanning symlinks...");
    baseline.symlinks = collect_symlinks();
    println!("   ✅ Baseline created");
    Ok(baseline)
}

fn save_baseline(baseline: &EntropyBaseline) -> CoreResult<()> {
    let path = paths::entropy_baseline_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(baseline).unwrap_or_default(),
    )?;
    println!("💾 Baseline saved to: {}", path.display());
    Ok(())
}

fn load_baseline() -> Option<EntropyBaseline> {
    let path = paths::entropy_baseline_file();
    fs::read_to_string(&path)
        .ok()
        .and_then(|d| serde_json::from_str(&d).ok())
}

fn check_drift(baseline: &EntropyBaseline) -> DriftReport {
    let mut report = DriftReport {
        config_drifts: Vec::new(),
        service_drifts: Vec::new(),
        binary_drifts: Vec::new(),
        symlink_drifts: Vec::new(),
        untracked_files: Vec::new(),
    };
    let config_dir = home().join(".config");
    for entry in walkdir::WalkDir::new(&config_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path_str = entry.path().display().to_string();
        if let Some(baseline_hash) = baseline.config_checksums.get(&path_str) {
            if let Ok(current_hash) = hash_file(entry.path()) {
                if &current_hash != baseline_hash {
                    report
                        .config_drifts
                        .push(format!("{} (modified)", path_str));
                }
            }
        } else {
            report
                .untracked_files
                .push(format!("{} (new file)", path_str));
        }
    }
    if let Ok(output) = Command::new("systemctl")
        .args([
            "--user",
            "list-units",
            "--type=service",
            "--all",
            "--plain",
            "--no-legend",
        ])
        .output()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let Some(baseline_state) = baseline.service_states.get(parts[0]) {
                    if baseline_state != parts[3] {
                        report
                            .service_drifts
                            .push(format!("{} ({} → {})", parts[0], baseline_state, parts[3]));
                    }
                }
            }
        }
    }
    for (pkg, baseline_version) in &baseline.package_versions {
        if let Ok(output) = Command::new("pacman").args(["-Q", pkg]).output() {
            if output.status.success() {
                let current = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if &current != baseline_version {
                    report
                        .binary_drifts
                        .push(format!("{}: {} → {}", pkg, baseline_version, current));
                }
            }
        }
    }
    report.symlink_drifts = find_broken_symlinks(&baseline.symlinks);
    report
}

fn report_drift(baseline: &EntropyBaseline, drift: &DriftReport) {
    println!("📊 Drift Report - 0-Core v{}", get_system_version());
    println!(
        "Baseline created: {}",
        baseline.created_at.split('T').next().unwrap_or("unknown")
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if drift.config_drifts.is_empty() {
        println!("✅ Config Drift (0 files)");
    } else {
        println!("⚠️  Config Drift ({} files)", drift.config_drifts.len());
        for item in &drift.config_drifts {
            println!("   {}", item);
        }
    }
    if drift.service_drifts.is_empty() {
        println!("✅ Service Drift (0 changes)");
    } else {
        println!("⚠️  Service Drift ({} changes)", drift.service_drifts.len());
        for item in &drift.service_drifts {
            println!("   {}", item);
        }
    }
    if drift.binary_drifts.is_empty() {
        println!("✅ Binary Drift (0 changes)");
    } else {
        println!("⚠️  Binary Drift ({} changes)", drift.binary_drifts.len());
        for item in &drift.binary_drifts {
            println!("   {}", item);
        }
    }
    if drift.symlink_drifts.is_empty() {
        println!("✅ Symlink Drift (0 new breaks)");
    } else {
        println!(
            "⚠️  Symlink Drift ({} new breaks)",
            drift.symlink_drifts.len()
        );
        for item in &drift.symlink_drifts {
            println!("   {}", item);
        }
    }
    if drift.untracked_files.is_empty() {
        println!("✅ Untracked Files (0 new)");
    } else {
        println!("⚠️  Untracked Files ({} new)", drift.untracked_files.len());
        for item in drift.untracked_files.iter().take(10) {
            println!("   {}", item);
        }
        if drift.untracked_files.len() > 10 {
            println!("   ... and {} more", drift.untracked_files.len() - 10);
        }
    }
    println!();
    let total = drift.config_drifts.len()
        + drift.service_drifts.len()
        + drift.binary_drifts.len()
        + drift.symlink_drifts.len();
    if total == 0 && drift.untracked_files.is_empty() {
        println!("🎉 System stable! No drift detected.");
    } else {
        println!("💡 Review changes and update baseline if intentional.");
    }
}

fn save_history(drift: &DriftReport) {
    let path = paths::entropy_history_file();
    let mut history = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|d| serde_json::from_str::<DriftHistory>(&d).ok())
            .unwrap_or(DriftHistory {
                version: "1.0".to_string(),
                entries: Vec::new(),
            })
    } else {
        DriftHistory {
            version: "1.0".to_string(),
            entries: Vec::new(),
        }
    };
    history.entries.push(DriftEntry {
        timestamp: Utc::now().to_rfc3339(),
        system_version: get_system_version(),
        total_drifts: drift.config_drifts.len()
            + drift.service_drifts.len()
            + drift.binary_drifts.len()
            + drift.symlink_drifts.len()
            + drift.untracked_files.len(),
        config_drifts: drift.config_drifts.len(),
        service_drifts: drift.service_drifts.len(),
        binary_drifts: drift.binary_drifts.len(),
        symlink_drifts: drift.symlink_drifts.len(),
        untracked_files: drift.untracked_files.len(),
    });
    let now = Utc::now();
    history.entries.retain(|e| {
        chrono::DateTime::parse_from_rfc3339(&e.timestamp)
            .map(|ts| now.signed_duration_since(ts.with_timezone(&Utc)).num_days() <= 30)
            .unwrap_or(false)
    });
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(
        &path,
        serde_json::to_string_pretty(&history).unwrap_or_default(),
    );
}

fn show_trends() {
    let path = paths::entropy_history_file();
    let history: DriftHistory = fs::read_to_string(&path)
        .ok()
        .and_then(|d| serde_json::from_str(&d).ok())
        .unwrap_or(DriftHistory {
            version: "1.0".to_string(),
            entries: Vec::new(),
        });
    println!("📊 Drift Trends - Last 30 days");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if history.entries.is_empty() {
        println!("No drift history found. Run: core doctor entropy");
        return;
    }
    for entry in history.entries.iter().rev().take(10) {
        let date = entry.timestamp.split('T').next().unwrap_or("unknown");
        let status = if entry.total_drifts == 0 {
            "✅"
        } else if entry.total_drifts < 5 {
            "⚠️ "
        } else {
            "🔴"
        };
        println!(
            "  {} {} - {} drifts (v{})",
            status, date, entry.total_drifts, entry.system_version
        );
    }
    let avg = history
        .entries
        .iter()
        .map(|e| e.total_drifts as f32)
        .sum::<f32>()
        / history.entries.len() as f32;
    let max = history
        .entries
        .iter()
        .map(|e| e.total_drifts)
        .max()
        .unwrap_or(0);
    let clean = history
        .entries
        .iter()
        .filter(|e| e.total_drifts == 0)
        .count();
    println!("\n📈 Statistics:");
    println!("   Average drift: {:.1}", avg);
    println!("   Highest drift: {}", max);
    println!("   Clean checks: {}/{}", clean, history.entries.len());
}

pub fn run(baseline: bool, trends: bool, json: bool) -> CoreResult<()> {
    if trends {
        show_trends();
        return Ok(());
    }
    if baseline {
        let b = create_baseline()?;
        return save_baseline(&b);
    }
    match load_baseline() {
        Some(b) => {
            let drift = check_drift(&b);
            save_history(&drift);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "system_version": get_system_version(),
                        "drift": {
                            "config_drifts": drift.config_drifts,
                            "service_drifts": drift.service_drifts,
                            "binary_drifts": drift.binary_drifts,
                            "symlink_drifts": drift.symlink_drifts,
                            "untracked_files": drift.untracked_files,
                        }
                    }))
                    .unwrap_or_default()
                );
            } else {
                report_drift(&b, &drift);
            }
        }
        None => {
            println!("❌ No baseline found!");
            println!("   Create one with: core doctor entropy --baseline");
        }
    }
    Ok(())
}
