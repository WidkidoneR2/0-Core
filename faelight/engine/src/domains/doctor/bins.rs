//! bin-doctor — absorbed from rust-tools/bin-doctor
use crate::errors::CoreResult;
use chrono::{DateTime, Utc};
use colored::*;
use faelight_core::paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn manifest_path() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cargo/.manifest.toml")
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

fn save_manifest(manifest: &Manifest) -> CoreResult<()> {
    let path = manifest_path();
    let content = toml::to_string_pretty(manifest)
        .map_err(|e| crate::errors::CoreError::Runtime(e.to_string()))?;
    fs::write(&path, content)?;
    Ok(())
}

fn find_tool_source(tool: &str) -> PathBuf {
    let rust_tools = paths::core_dir().join("rust-tools");
    let exact = rust_tools.join(tool);
    if exact.join("Cargo.toml").exists() {
        return exact;
    }
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

fn current_commit() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(paths::core_dir())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn run(subcmd: Option<&str>, quiet: bool) -> CoreResult<()> {
    match subcmd {
        Some("register") => {
            println!("Usage: core doctor bins register <tool-name>");
            Ok(())
        }
        Some("list") => list_binaries(),
        Some("drift") => show_drift(),
        Some("check") | None => check_binaries(quiet),
        Some(tool)
            if subcmd
                .map(|s| !["check", "list", "drift"].contains(&s))
                .unwrap_or(false) =>
        {
            register_binary(tool)
        }
        _ => check_binaries(quiet),
    }
}

pub fn register_binary(tool: &str) -> CoreResult<()> {
    println!("📦 Registering binary: {}", tool);
    let commit = current_commit();
    let rustc = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.split_whitespace().nth(1).map(|v| v.to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    let source = find_tool_source(tool);
    let version = get_tool_version(&source);
    let binary = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".cargo/bin")
        .join(tool);
    let entry = BinaryEntry {
        version: version.clone(),
        commit: commit.clone(),
        built: Utc::now(),
        rustc: rustc.clone(),
        source: source.display().to_string(),
        binary: binary.display().to_string(),
    };
    let mut manifest = load_manifest();
    manifest.binaries.insert(tool.to_string(), entry.clone());
    save_manifest(&manifest)?;
    println!("✅ Registered:");
    println!("   Version:  {}", entry.version);
    println!("   Commit:   {}", entry.commit);
    println!("   Built:    {}", entry.built.format("%Y-%m-%d %H:%M"));
    println!("   Rustc:    {}", entry.rustc);
    println!("   Source:   {}", entry.source);
    Ok(())
}

fn check_binaries(quiet: bool) -> CoreResult<()> {
    let manifest = load_manifest();
    if manifest.binaries.is_empty() {
        if !quiet {
            println!("⚠️  No binaries registered");
            println!("   Use: core doctor bins register <tool-name>");
        }
        return Ok(());
    }
    if !quiet {
        println!("🔍 Checking {} binaries...", manifest.binaries.len());
        println!();
    }
    let mut drifted = Vec::new();
    let mut ok = Vec::new();
    let src_commit = current_commit();
    for (name, entry) in &manifest.binaries {
        let source_version = get_tool_version(&PathBuf::from(&entry.source));
        if source_version != entry.version || src_commit != entry.commit {
            drifted.push((
                name.clone(),
                entry.clone(),
                source_version,
                src_commit.clone(),
            ));
        } else {
            ok.push(name.clone());
        }
    }
    if !quiet {
        for name in &ok {
            println!("✅ {}", name);
        }
        for (name, entry, src_ver, src_commit) in &drifted {
            println!("{} DRIFT DETECTED", name.red().bold());
            println!("   Binary:  v{} @ {}", entry.version, entry.commit);
            println!("   Source:  v{} @ {}", src_ver, src_commit);
            println!("   💡 Rebuild: cargo build --release -p {}", name);
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
    Ok(())
}

fn list_binaries() -> CoreResult<()> {
    let manifest = load_manifest();
    if manifest.binaries.is_empty() {
        println!("No binaries registered");
        return Ok(());
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
    Ok(())
}

fn show_drift() -> CoreResult<()> {
    let manifest = load_manifest();
    let mut found_drift = false;
    let src_commit = current_commit();
    for (name, entry) in &manifest.binaries {
        let source_version = get_tool_version(&PathBuf::from(&entry.source));
        if source_version != entry.version || src_commit != entry.commit {
            found_drift = true;
            println!("❌ {}", name);
            println!("   Binary:  v{} @ {}", entry.version, entry.commit);
            println!("   Source:  v{} @ {}", source_version, src_commit);
            println!("   💡 Rebuild: cargo build --release -p {}", name);
            println!();
        }
    }
    if !found_drift {
        println!("✅ No drift detected - all binaries in sync!");
    }
    Ok(())
}
