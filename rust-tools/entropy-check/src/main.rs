//! entropy-check v1.0.0 - Configuration Drift Detection
//! 🌲 Faelight Forest

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION: &str = "1.0.0";
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

#[derive(Debug, Serialize)]
struct DriftReport {
    config_drifts: Vec<String>,
    service_drifts: Vec<String>,
    binary_drifts: Vec<String>,
    symlink_drifts: Vec<String>,
    untracked_files: Vec<String>,
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
        
        // Skip if it was already broken in baseline
        if baseline_symlinks.contains_key(&path_str) {
            continue;
        }
        
        // Check if it's broken now
        if !path.exists() {
            broken.push(path_str);
        }
    }
    
    broken
}

// ═══════════════════════════════════════════════════════════
// 📸 BASELINE MANAGEMENT
// ═══════════════════════════════════════════════════════════

fn create_baseline() -> Result<EntropyBaseline, Box<dyn std::error::Error>> {
    println!("🔨 Creating baseline...");
    let mut baseline = EntropyBaseline::new();
    
    // Collect config checksums
    println!("   📁 Scanning config files...");
    let home = std::env::var("HOME")?;
    let config_dir = format!("{}/.config", home);
    
    for entry in walkdir::WalkDir::new(&config_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(hash) = hash_file(entry.path()) {
            baseline.config_checksums.insert(
                entry.path().display().to_string(),
                hash,
            );
        }
    }
    
    // Collect service states
    println!("   ⚙️  Checking services...");
    if let Ok(output) = Command::new("systemctl")
        .args(["--user", "list-units", "--type=service", "--all", "--plain", "--no-legend"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let service = parts[0].to_string();
                let state = parts[3].to_string();
                baseline.service_states.insert(service, state);
            }
        }
    }
    
    // Collect package versions (key packages only)
    println!("   📦 Recording package versions...");
    let packages = vec!["sway", "waybar", "neovim", "zsh", "kitty"];
    for pkg in packages {
        if let Ok(output) = Command::new("pacman").args(["-Q", pkg]).output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                baseline.package_versions.insert(pkg.to_string(), version);
            }
        }
    }
    
    // Collect symlinks
    println!("   🔗 Scanning symlinks...");
    baseline.symlinks = collect_symlinks();
    
    println!("   ✅ Baseline created");
    Ok(baseline)
}

fn save_baseline(baseline: &EntropyBaseline) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_baseline_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let json = serde_json::to_string_pretty(baseline)?;
    fs::write(&path, json)?;
    
    println!("💾 Baseline saved to: {}", path.display());
    Ok(())
}

fn load_baseline() -> Result<EntropyBaseline, Box<dyn std::error::Error>> {
    let path = get_baseline_path();
    let data = fs::read_to_string(path)?;
    let baseline = serde_json::from_str(&data)?;
    Ok(baseline)
}

// ═══════════════════════════════════════════════════════════
// 🔍 DRIFT DETECTION
// ═══════════════════════════════════════════════════════════

fn check_drift(baseline: &EntropyBaseline) -> Result<DriftReport, Box<dyn std::error::Error>> {
    let mut report = DriftReport {
        config_drifts: Vec::new(),
        service_drifts: Vec::new(),
        binary_drifts: Vec::new(),
        symlink_drifts: Vec::new(),
        untracked_files: Vec::new(),
    };
    
    // Check config file changes
    let home = std::env::var("HOME")?;
    let config_dir = format!("{}/.config", home);
    
    for entry in walkdir::WalkDir::new(&config_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path_str = entry.path().display().to_string();
        
        if let Some(baseline_hash) = baseline.config_checksums.get(&path_str) {
            // File existed in baseline - check if it changed
            if let Ok(current_hash) = hash_file(entry.path()) {
                if &current_hash != baseline_hash {
                    report.config_drifts.push(format!("{} (modified)", path_str));
                }
            }
        } else {
            // New file
            report.untracked_files.push(format!("{} (new file)", path_str));
        }
    }
    
    // Check service state changes
    if let Ok(output) = Command::new("systemctl")
        .args(["--user", "list-units", "--type=service", "--all", "--plain", "--no-legend"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let service = parts[0];
                let current_state = parts[3];
                
                if let Some(baseline_state) = baseline.service_states.get(service) {
                    if baseline_state != current_state {
                        report.service_drifts.push(
                            format!("{} ({} → {})", service, baseline_state, current_state)
                        );
                    }
                }
            }
        }
    }
    
    // Check package version changes
    for (pkg, baseline_version) in &baseline.package_versions {
        if let Ok(output) = Command::new("pacman").args(["-Q", pkg]).output() {
            if output.status.success() {
                let current_version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                if &current_version != baseline_version {
                    report.binary_drifts.push(
                        format!("{}: {} → {}", pkg, baseline_version, current_version)
                    );
                }
            }
        }
    }
    
    // Check for new broken symlinks
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
            println!("   {}", item);
        }
    }
    
    // Service drift
    if drift.service_drifts.is_empty() {
        println!("✅ Service Drift (0 changes)");
    } else {
        println!("⚠️  Service Drift ({} changes)", drift.service_drifts.len());
        for item in &drift.service_drifts {
            println!("   {}", item);
        }
    }
    
    // Binary drift
    if drift.binary_drifts.is_empty() {
        println!("✅ Binary Drift (0 changes)");
    } else {
        println!("⚠️  Binary Drift ({} changes)", drift.binary_drifts.len());
        for item in &drift.binary_drifts {
            println!("   {}", item);
        }
    }
    
    // Symlink drift
    if drift.symlink_drifts.is_empty() {
        println!("✅ Symlink Drift (0 new breaks)");
    } else {
        println!("⚠️  Symlink Drift ({} new breaks)", drift.symlink_drifts.len());
        for item in &drift.symlink_drifts {
            println!("   {}", item);
        }
    }
    
    // Untracked files
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
    
    // Recommendation
    let total_issues = drift.config_drifts.len() 
                     + drift.service_drifts.len() 
                     + drift.binary_drifts.len() 
                     + drift.symlink_drifts.len();
    
    if total_issues == 0 && drift.untracked_files.is_empty() {
        println!("🎉 System stable! No drift detected.");
    } else if total_issues > 0 {
        println!("💡 Recommendation:");
        if !drift.config_drifts.is_empty() {
            println!("   Config files changed - review and update baseline if intentional");
        }
        if !drift.service_drifts.is_empty() {
            println!("   Services changed state - check if this is expected");
        }
        if !drift.binary_drifts.is_empty() {
            println!("   Packages updated - update baseline after verifying system health");
        }
        if !drift.symlink_drifts.is_empty() {
            println!("   Broken symlinks detected - investigate and repair");
        }
    } else if !drift.untracked_files.is_empty() {
        println!("💡 Recommendation:");
        println!("   New untracked files detected, review and add to baseline if intended");
    }
}

// ═══════════════════════════════════════════════════════════
// 🎯 MAIN & FLAGS
// ═══════════════════════════════════════════════════════════

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    // Handle flags
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("entropy-check v{}", VERSION);
        return;
    }
    
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        show_help();
        return;
    }
    
    if args.contains(&"--health".to_string()) {
        show_health();
        return;
    }
    
    if args.contains(&"--trends".to_string()) {
        show_trends();
        return;
    }
    
    let json_output = args.contains(&"--json".to_string());
    
    if !json_output {
        println!("🔍 entropy-check v{} - Configuration Drift Detection", VERSION);
        println!();
    }
    
    if args.contains(&"--baseline".to_string()) {
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
                    Ok(drift) => {
                        // Save to history
                        let mut history = DriftHistory::load();
                        history.add_entry(&drift, &get_system_version());
                        if let Err(e) = history.save() {
                            eprintln!("⚠️  Warning: Could not save drift history: {}", e);
                        }
                        
                        // Display results
                        if json_output {
                            output_json(&baseline, &drift);
                        } else {
                            report_drift(&baseline, &drift);
                        }
                    }
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

fn show_help() {
    println!("🔍 entropy-check v{} - Configuration Drift Detection", VERSION);
    println!();
    println!("USAGE:");
    println!("  entropy-check [FLAGS]");
    println!();
    println!("FLAGS:");
    println!("  --baseline          Create/update baseline snapshot");
    println!("  --trends            Show drift history (last 30 days)");
    println!("  --json              Output in JSON format");
    println!("  --health            Run health check");
    println!("  --version, -v       Show version");
    println!("  --help, -h          Show this help");
    println!();
    println!("EXAMPLES:");
    println!("  entropy-check --baseline    # Create baseline");
    println!("  entropy-check               # Check for drift");
    println!("  entropy-check --trends      # Show history");
    println!("  entropy-check --json        # JSON output");
    println!();
    println!("FILES:");
    println!("  ~/.config/faelight/entropy-baseline.json    Baseline snapshot");
    println!("  ~/.config/faelight/entropy-history.json     Drift history");
    println!();
    println!("PHILOSOPHY:");
    println!("  Systems drift. Humans change. Contexts evolve.");
    println!("  entropy-check watches for the silent changes.");
}

fn show_health() {
    println!();
    println!("🏥 entropy-check v{} - Health Check", VERSION);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut healthy = true;
    
    // Check baseline exists
    print!("  Checking baseline... ");
    let baseline_path = get_baseline_path();
    if baseline_path.exists() {
        if let Ok(baseline) = load_baseline() {
            println!("✅ Found (created {})", baseline.created_at.split('T').next().unwrap_or("unknown"));
        } else {
            println!("⚠️  Exists but corrupted");
            healthy = false;
        }
    } else {
        println!("❌ No baseline found");
        healthy = false;
    }
    
    // Check history
    print!("  Checking history... ");
    let history_path = get_history_path();
    if history_path.exists() {
        let history = DriftHistory::load();
        println!("✅ {} entries", history.entries.len());
    } else {
        println!("⚠️  No history yet");
    }
    
    // Check systemctl available
    print!("  Checking systemctl... ");
    if Command::new("which").arg("systemctl").output().is_ok() {
        println!("✅");
    } else {
        println!("❌ Not found");
        healthy = false;
    }
    
    // Check pacman available
    print!("  Checking pacman... ");
    if Command::new("which").arg("pacman").output().is_ok() {
        println!("✅");
    } else {
        println!("❌ Not found");
        healthy = false;
    }
    
    // Check ~/.config exists
    print!("  Checking ~/.config... ");
    let home = std::env::var("HOME").unwrap_or_default();
    let config_dir = format!("{}/.config", home);
    if PathBuf::from(&config_dir).exists() {
        println!("✅");
    } else {
        println!("❌ Not found");
        healthy = false;
    }
    
    println!();
    if healthy {
        println!("✅ All systems operational");
        std::process::exit(0);
    } else {
        println!("❌ System unhealthy");
        std::process::exit(1);
    }
}

fn output_json(baseline: &EntropyBaseline, drift: &DriftReport) {
    let output = serde_json::json!({
        "version": "1.0",
        "tool_version": VERSION,
        "system_version": get_system_version(),
        "baseline_created": baseline.created_at,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "drift": {
            "config_drifts": drift.config_drifts,
            "service_drifts": drift.service_drifts,
            "binary_drifts": drift.binary_drifts,
            "symlink_drifts": drift.symlink_drifts,
            "untracked_files": drift.untracked_files,
        },
        "summary": {
            "total_drifts": drift.config_drifts.len() + drift.service_drifts.len() + drift.binary_drifts.len() + drift.symlink_drifts.len() + drift.untracked_files.len(),
            "has_drift": !(drift.config_drifts.is_empty() && drift.service_drifts.is_empty() && drift.binary_drifts.is_empty() && drift.symlink_drifts.is_empty() && drift.untracked_files.is_empty()),
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

// ═══════════════════════════════════════════════════════════
// 📈 HISTORY TRACKING
// ═══════════════════════════════════════════════════════════

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

impl DriftHistory {
    fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            entries: Vec::new(),
        }
    }
    
    fn load() -> Self {
        let path = get_history_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(history) = serde_json::from_str(&data) {
                    return history;
                }
            }
        }
        Self::new()
    }
    
    fn save(&self) -> std::io::Result<()> {
        let path = get_history_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }
    
    fn add_entry(&mut self, report: &DriftReport, system_version: &str) {
        let entry = DriftEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            system_version: system_version.to_string(),
            total_drifts: report.config_drifts.len() 
                        + report.service_drifts.len() 
                        + report.binary_drifts.len() 
                        + report.symlink_drifts.len() 
                        + report.untracked_files.len(),
            config_drifts: report.config_drifts.len(),
            service_drifts: report.service_drifts.len(),
            binary_drifts: report.binary_drifts.len(),
            symlink_drifts: report.symlink_drifts.len(),
            untracked_files: report.untracked_files.len(),
        };
        
        self.entries.push(entry);
        
        // Keep only last 30 days
        let now = chrono::Utc::now();
        self.entries.retain(|e| {
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&e.timestamp) {
                let age = now.signed_duration_since(ts.with_timezone(&chrono::Utc));
                age.num_days() <= 30
            } else {
                false
            }
        });
    }
}

fn get_history_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".config/faelight/entropy-history.json")
}

fn show_trends() {
    println!("📊 Drift Trends - Last 30 days");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    let history = DriftHistory::load();
    
    if history.entries.is_empty() {
        println!("No drift history found");
        println!("Run entropy-check a few times to build history");
        return;
    }
    
    println!("Total checks: {}", history.entries.len());
    println!();
    println!("Recent Drift History:");
    
    for entry in history.entries.iter().rev().take(10) {
        let date = entry.timestamp.split('T').next().unwrap_or("unknown");
        let status = if entry.total_drifts == 0 {
            "✅"
        } else if entry.total_drifts < 5 {
            "⚠️ "
        } else {
            "🔴"
        };
        
        println!("  {} {} - {} total drifts (v{})", 
                 status, date, entry.total_drifts, entry.system_version);
        
        if entry.total_drifts > 0 {
            print!("      ");
            if entry.config_drifts > 0 { print!("config:{} ", entry.config_drifts); }
            if entry.service_drifts > 0 { print!("services:{} ", entry.service_drifts); }
            if entry.binary_drifts > 0 { print!("binaries:{} ", entry.binary_drifts); }
            if entry.symlink_drifts > 0 { print!("symlinks:{} ", entry.symlink_drifts); }
            if entry.untracked_files > 0 { print!("untracked:{} ", entry.untracked_files); }
            println!();
        }
    }
    
    // Calculate statistics
    let avg_drift: f32 = history.entries.iter()
        .map(|e| e.total_drifts as f32)
        .sum::<f32>() / history.entries.len() as f32;
    
    let max_drift = history.entries.iter()
        .map(|e| e.total_drifts)
        .max()
        .unwrap_or(0);
    
    let clean_checks = history.entries.iter()
        .filter(|e| e.total_drifts == 0)
        .count();
    
    println!();
    println!("📈 Statistics:");
    println!("   Average drift per check: {:.1}", avg_drift);
    println!("   Highest drift: {}", max_drift);
    println!("   Clean checks: {}/{}", clean_checks, history.entries.len());
}
