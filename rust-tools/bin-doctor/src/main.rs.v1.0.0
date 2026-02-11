//! bin-doctor v1.0.0 - Binary Manifest System
//! Track installed binaries and detect version drift

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use faelight_core::paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "bin-doctor")]
#[command(about = "Binary manifest system - track and verify installed binaries", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a binary after cargo install
    Register {
        /// Tool name (e.g., faelight-git)
        tool: String,
    },
    /// Check all registered binaries for drift
    Check {
        /// Quiet mode (exit codes only)
        #[arg(short, long)]
        quiet: bool,
    },
    /// List all tracked binaries
    List,
    /// Show only drifted binaries
    Drift,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BinaryEntry {
    version: String,
    commit: String,
    built: DateTime<Utc>,
    rustc: String,
    source: String,
    binary: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    #[serde(flatten)]
    binaries: HashMap<String, BinaryEntry>,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Register { tool } => register_binary(&tool),
        Commands::Check { quiet } => check_binaries(quiet),
        Commands::List => list_binaries(),
        Commands::Drift => show_drift(),
    }
}

fn manifest_path() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap()).join(".cargo/.manifest.toml")
}

fn load_manifest() -> Manifest {
    let path = manifest_path();
    if !path.exists() {
        return Manifest {
            binaries: HashMap::new(),
        };
    }

    let content = fs::read_to_string(&path).unwrap_or_default();
    toml::from_str(&content).unwrap_or(Manifest {
        binaries: HashMap::new(),
    })
}

fn save_manifest(manifest: &Manifest) {
    let path = manifest_path();
    let content = toml::to_string_pretty(manifest).unwrap();
    fs::write(&path, content).expect("Failed to write manifest");
}

fn register_binary(tool: &str) {
    println!("📦 Registering binary: {}", tool);

    // Get current git commit
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(paths::core_dir())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get rustc version
    let rustc = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.split_whitespace().nth(1).map(|v| v.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    // Find the tool's source directory
    let source = find_tool_source(tool);

    // Get version from Cargo.toml
    let version = get_tool_version(&source);

    // Binary path
    let binary = PathBuf::from(std::env::var("HOME").unwrap())
        .join(".cargo/bin")
        .join(tool);

    let entry = BinaryEntry {
        version,
        commit,
        built: Utc::now(),
        rustc,
        source: source.display().to_string(),
        binary: binary.display().to_string(),
    };

    let mut manifest = load_manifest();
    manifest.binaries.insert(tool.to_string(), entry.clone());
    save_manifest(&manifest);

    println!("✅ Registered:");
    println!("   Version:  {}", entry.version);
    println!("   Commit:   {}", entry.commit);
    println!("   Built:    {}", entry.built.format("%Y-%m-%d %H:%M"));
    println!("   Rustc:    {}", entry.rustc);
    println!("   Source:   {}", entry.source);
}

fn find_tool_source(tool: &str) -> PathBuf {
    let core_dir = paths::core_dir();
    let rust_tools = core_dir.join("rust-tools");

    // Try exact match first
    let exact = rust_tools.join(tool);
    if exact.join("Cargo.toml").exists() {
        return exact;
    }

    // Try finding by binary name
    if let Ok(entries) = fs::read_dir(&rust_tools) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let cargo_toml = path.join("Cargo.toml");
                if cargo_toml.exists() {
                    if let Ok(content) = fs::read_to_string(&cargo_toml) {
                        if content.contains(&format!("name = \"{}\"", tool)) {
                            return path;
                        }
                    }
                }
            }
        }
    }

    rust_tools.join(tool)
}

fn get_tool_version(source: &Path) -> String {
    let cargo_toml = source.join("Cargo.toml");
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        for line in content.lines() {
            if line.starts_with("version") {
                if let Some(version) = line.split('=').nth(1) {
                    return version.trim().trim_matches('"').to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

fn check_binaries(quiet: bool) {
    let manifest = load_manifest();

    if manifest.binaries.is_empty() {
        if !quiet {
            println!("⚠️  No binaries registered");
            println!("   Use: bin-doctor register <tool-name>");
        }
        std::process::exit(1);
    }

    if !quiet {
        println!("🔍 Checking {} binaries...", manifest.binaries.len());
        println!();
    }

    let mut drifted = Vec::new();
    let mut ok = Vec::new();

    for (name, entry) in &manifest.binaries {
        let source_version = get_tool_version(&PathBuf::from(&entry.source));
        let source_commit = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(paths::core_dir())
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if source_version != entry.version || source_commit != entry.commit {
            drifted.push((name.clone(), entry.clone(), source_version, source_commit));
        } else {
            ok.push(name.clone());
        }
    }

    if !quiet {
        for name in &ok {
            println!("✅ {}", name);
        }

        for (name, entry, src_ver, src_commit) in &drifted {
            println!("❌ {}: DRIFT DETECTED", name);
            println!("   Binary:  v{} @ {}", entry.version, entry.commit);
            println!("   Source:  v{} @ {}", src_ver, src_commit);
            println!("   💡 Rebuild: cargo install --path {}", entry.source);
            println!();
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        if drifted.is_empty() {
            println!("✅ All {} binaries in sync!", manifest.binaries.len());
        } else {
            println!(
                "⚠️  {}/{} binaries drifted",
                drifted.len(),
                manifest.binaries.len()
            );
        }
    }

    if !drifted.is_empty() {
        std::process::exit(1);
    }
}

fn list_binaries() {
    let manifest = load_manifest();

    if manifest.binaries.is_empty() {
        println!("No binaries registered");
        return;
    }

    println!("📦 Tracked Binaries ({}):", manifest.binaries.len());
    println!();

    let mut binaries: Vec<_> = manifest.binaries.iter().collect();
    binaries.sort_by_key(|(name, _)| name.as_str());

    for (name, entry) in binaries {
        println!("{}", name);
        println!("  Version:  {}", entry.version);
        println!("  Commit:   {}", entry.commit);
        println!("  Built:    {}", entry.built.format("%Y-%m-%d %H:%M"));
        println!("  Rustc:    {}", entry.rustc);
        println!("  Source:   {}", entry.source);
        println!("  Binary:   {}", entry.binary);
        println!();
    }
}

fn show_drift() {
    let manifest = load_manifest();
    let mut found_drift = false;

    for (name, entry) in &manifest.binaries {
        let source_version = get_tool_version(&PathBuf::from(&entry.source));
        let source_commit = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(paths::core_dir())
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if source_version != entry.version || source_commit != entry.commit {
            found_drift = true;
            println!("❌ {}", name);
            println!("   Binary:  v{} @ {}", entry.version, entry.commit);
            println!("   Source:  v{} @ {}", source_version, source_commit);
            println!("   💡 Rebuild: cargo install --path {}", entry.source);
            println!();
        }
    }

    if !found_drift {
        println!("✅ No drift detected - all binaries in sync!");
    }
}
