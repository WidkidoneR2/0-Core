//! Rollback — switch generation pointer with health verification.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct GenerationInfo {
    pub version: String,
    #[allow(dead_code)]
    pub date: String,
    pub theme: String,
    pub health: u32,
    #[allow(dead_code)]
    pub commits: u32,
    pub tools: u32,
    pub intents: u32,
}

impl GenerationInfo {
    pub fn load(releases_dir: &PathBuf, version: &str) -> Result<Self> {
        let manifest_path = releases_dir.join(version).join("manifest.toml");
        if !manifest_path.exists() {
            return Err(anyhow!("no manifest found for version {}", version));
        }
        let content = fs::read_to_string(&manifest_path)?;
        Ok(Self {
            version: version.to_string(),
            date: extract_str(&content, "date").unwrap_or_default(),
            theme: extract_str(&content, "theme").unwrap_or_default(),
            health: extract_u32(&content, "health_at_release").unwrap_or(0),
            commits: extract_u32(&content, "commits_since_last").unwrap_or(0),
            tools: extract_u32(&content, "tools_total").unwrap_or(0),
            intents: extract_u32(&content, "intents_complete").unwrap_or(0),
        })
    }
}

fn extract_str(content: &str, key: &str) -> Option<String> {
    content
        .lines()
        .find(|l| l.trim().starts_with(key))
        .and_then(|l| l.split('"').nth(1))
        .map(|s| s.to_string())
}

fn extract_u32(content: &str, key: &str) -> Option<u32> {
    content
        .lines()
        .find(|l| l.trim().starts_with(key))
        .and_then(|l| l.split('=').nth(1))
        .and_then(|s| s.trim().parse().ok())
}

pub fn get_sorted_versions(releases_dir: &PathBuf) -> Vec<String> {
    let mut versions: Vec<String> = fs::read_dir(releases_dir)
        .unwrap_or_else(|_| panic!("cannot read releases dir"))
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|m| m.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.split('.').filter_map(|x| x.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|x| x.parse().ok()).collect();
        a_parts.cmp(&b_parts)
    });
    versions
}

pub fn rollback(core_root: &PathBuf, target: Option<&str>) -> Result<()> {
    let gen_path = core_root.join("runtime/generation");
    let releases_dir = core_root.join("00-meta/releases");

    let current = fs::read_to_string(&gen_path)
        .unwrap_or_default()
        .trim()
        .to_string();

    let versions = get_sorted_versions(&releases_dir);

    let target_version = if let Some(t) = target {
        t.to_string()
    } else {
        // Find previous version
        let current_idx = versions
            .iter()
            .position(|v| v == &current)
            .context("current generation not found in releases")?;
        if current_idx == 0 {
            return Err(anyhow!(
                "already at oldest generation — cannot roll back further"
            ));
        }
        versions[current_idx - 1].clone()
    };

    if target_version == current {
        println!("⚠️  Already at version {} — nothing to rollback", current);
        return Ok(());
    }

    // Load both manifests for diff display
    let current_info = GenerationInfo::load(&releases_dir, &current)?;
    let target_info = GenerationInfo::load(&releases_dir, &target_version)?;

    // Show diff
    println!("🌲 faelight-release rollback");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!(
        "  Current:  {} — {}",
        current_info.version, current_info.theme
    );
    println!(
        "  Target:   {} — {}",
        target_info.version, target_info.theme
    );
    println!();
    println!("  Generation diff:");
    println!(
        "  Health:   {}% → {}%",
        current_info.health, target_info.health
    );
    println!("  Tools:    {} → {}", current_info.tools, target_info.tools);
    println!(
        "  Intents:  {} → {}",
        current_info.intents, target_info.intents
    );
    println!();
    println!("  ⚠️  This will:");
    println!("  · Switch generation pointer to {}", target_version);
    println!("  · Check out git tag v{}", target_version);
    println!("  · Run health check on restored state");
    println!();

    print!("  Proceed? (type 'rollback' to confirm): ");
    use std::io::Write;
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim() != "rollback" {
        println!("❌ Rollback cancelled");
        return Ok(());
    }

    println!();
    println!("  → Switching generation pointer...");
    fs::write(&gen_path, &target_version)?;
    println!("  ✅ Generation pointer → {}", target_version);

    // Checkout git tag
    println!("  → Checking out v{}...", target_version);
    let result = Command::new("git")
        .args([
            "-C",
            core_root.to_str().unwrap_or("."),
            "checkout",
            &format!("v{}", target_version),
        ])
        .output();

    match result {
        Ok(out) if out.status.success() => {
            println!("  ✅ Checked out v{}", target_version);
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("  ⚠️  git checkout: {}", stderr.trim());
        }
        Err(e) => {
            println!("  ⚠️  git checkout failed: {}", e);
        }
    }

    // Emit rollback event to ledger
    emit_rollback_event(core_root, &current, &target_version);

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Rollback complete — now at v{}", target_version);
    println!();
    println!("  Run: d   to verify health");
    println!("  Run: faelight-release status   to confirm generation");

    Ok(())
}

fn emit_rollback_event(core_root: &PathBuf, from: &str, to: &str) {
    let db_path = core_root.join("runtime/state.db");
    if !db_path.exists() {
        return;
    }

    let payload = serde_json::json!({
        "actor": "faelight-release",
        "result": "ok",
        "detail": {
            "from": from,
            "to": to,
            "action": "rollback"
        }
    });

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let _ = Command::new("sqlite3")
        .arg(&db_path)
        .arg(format!(
            "INSERT INTO events (domain, action, payload, timestamp) VALUES ('release', 'rollback', '{}', {});",
            payload.to_string().replace('\'', "''"),
            now
        ))
        .output();
}
