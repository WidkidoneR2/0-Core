// faelight-fm v4.0 -- Nix plugin
// Handles: flake.nix, flake.lock, .drv files, /nix/store paths

use super::{Plugin, PluginAction};
use std::{path::Path, process::Command};

pub struct NixPlugin;

impl Plugin for NixPlugin {
    fn name(&self) -> &str {
        "nix"
    }

    fn handles(&self, path: &Path) -> bool {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let path_str = path.to_string_lossy();
        name == "flake.nix"
            || name == "flake.lock"
            || name.ends_with(".drv")
            || name.ends_with(".nix")
            || path_str.contains("/nix/store/")
    }

    fn preview(&self, path: &Path) -> String {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let path_str = path.to_string_lossy();

        if name == "flake.lock" {
            return preview_flake_lock(path);
        }
        if name == "flake.nix" {
            return preview_flake_nix(path);
        }
        if name.ends_with(".drv") {
            return preview_derivation(&path_str);
        }
        if path_str.contains("/nix/store/") {
            return preview_store_path(&path_str);
        }
        if name.ends_with(".nix") {
            return preview_nix_file(path);
        }
        String::new()
    }

    fn actions(&self, path: &Path) -> Vec<PluginAction> {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut actions = vec![PluginAction {
            label: "nix-store info".to_string(),
            key: 'i',
            description: "Show store path info".to_string(),
        }];
        if name == "flake.nix" || name == "flake.lock" {
            actions.push(PluginAction {
                label: "browse inputs".to_string(),
                key: 'f',
                description: "Browse flake inputs as virtual dirs".to_string(),
            });
        }
        if name.ends_with(".drv") {
            actions.push(PluginAction {
                label: "show deps".to_string(),
                key: 'd',
                description: "Show derivation dependencies".to_string(),
            });
        }
        actions
    }

    fn execute(&self, path: &Path, action: char) -> String {
        match action {
            'i' => preview_store_path(&path.to_string_lossy()),
            'f' => browse_flake_inputs(path),
            'd' => show_drv_deps(&path.to_string_lossy()),
            _ => String::new(),
        }
    }
}

fn preview_flake_lock(path: &Path) -> String {
    let Ok(content) = std::fs::read_to_string(path) else {
        return "Could not read flake.lock".to_string();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return "Could not parse flake.lock".to_string();
    };
    let mut out = String::from("❄️  Flake Inputs\n━━━━━━━━━━━━━━━━\n");
    if let Some(nodes) = json.get("nodes").and_then(|n| n.as_object()) {
        let mut inputs: Vec<String> = nodes.keys().filter(|k| *k != "root").cloned().collect();
        inputs.sort();
        for input in &inputs {
            if let Some(node) = nodes.get(input) {
                let rev = node
                    .get("locked")
                    .and_then(|l| l.get("rev"))
                    .and_then(|r| r.as_str())
                    .map(|r| &r[..8])
                    .unwrap_or("?");
                let typ = node
                    .get("locked")
                    .and_then(|l| l.get("type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("?");
                out.push_str(&format!("  {} {} [{}@{}]\n", "❄", input, typ, rev));
            }
        }
        out.push_str(&format!("\n{} inputs total\n", inputs.len()));
    }
    out
}

fn preview_flake_nix(path: &Path) -> String {
    let Ok(content) = std::fs::read_to_string(path) else {
        return "Could not read flake.nix".to_string();
    };
    let mut out = String::from("❄️  flake.nix\n━━━━━━━━━━━━━\n");
    // Extract inputs
    let mut in_inputs = false;
    for line in content.lines().take(60) {
        if line.contains("inputs") && line.contains("{") {
            in_inputs = true;
        }
        if in_inputs {
            out.push_str(line);
            out.push('\n');
            if line.contains('}') && !line.contains('{') {
                in_inputs = false;
            }
        }
    }
    if out.len() < 30 {
        out.push_str(&content.lines().take(40).collect::<Vec<_>>().join("\n"));
    }
    out
}

fn preview_derivation(store_path: &str) -> String {
    let out = Command::new("nix")
        .args(["show-derivation", store_path])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if out.is_empty() {
        return format!(
            "⚙️  Derivation: {}\n\nRun: nix show-derivation {}",
            store_path, store_path
        );
    }
    // Pretty print key fields
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&out) else {
        return out.lines().take(40).collect::<Vec<_>>().join("\n");
    };
    let mut result = String::from("⚙️  Derivation\n━━━━━━━━━━━━━\n");
    if let Some(obj) = json.as_object() {
        for (_, drv) in obj.iter().take(1) {
            if let Some(name) = drv.get("name").and_then(|n| n.as_str()) {
                result.push_str(&format!("Name:    {}\n", name));
            }
            if let Some(system) = drv.get("system").and_then(|s| s.as_str()) {
                result.push_str(&format!("System:  {}\n", system));
            }
            if let Some(inputs) = drv.get("inputDrvs").and_then(|i| i.as_object()) {
                result.push_str(&format!("\nInputs: {} derivations\n", inputs.len()));
                for (k, _) in inputs.iter().take(5) {
                    let name = k
                        .strip_prefix("/nix/store/")
                        .and_then(|s| s.splitn(2, '-').nth(1))
                        .unwrap_or(k);
                    result.push_str(&format!("  ← {}\n", name));
                }
            }
        }
    }
    result
}

fn preview_store_path(store_path: &str) -> String {
    let mut info = format!("❄️  Nix Store\n{}\n\n", store_path);
    if let Some(pkg) = store_path
        .strip_prefix("/nix/store/")
        .and_then(|s| s.splitn(2, '-').nth(1))
    {
        info.push_str(&format!("Package: {}\n", pkg));
    }
    // Referrers
    let refs = Command::new("nix-store")
        .args(["--query", "--referrers", store_path])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if !refs.is_empty() {
        let count = refs.lines().count();
        info.push_str(&format!("\nReferrers: {}\n", count));
        for line in refs.lines().take(3) {
            if let Some(n) = line
                .strip_prefix("/nix/store/")
                .and_then(|s| s.splitn(2, '-').nth(1))
            {
                info.push_str(&format!("  ← {}\n", n));
            }
        }
    }
    // GC roots
    let roots = Command::new("nix-store")
        .args(["--query", "--roots", store_path])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if !roots.is_empty() {
        info.push_str("\nGC Roots:\n");
        for line in roots.lines().take(3) {
            info.push_str(&format!("  ⚓ {}\n", line.trim()));
        }
    }
    info
}

fn preview_nix_file(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .take(60)
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
fn browse_flake_inputs(path: &Path) -> String {
    let lock = path.parent().unwrap_or(path).join("flake.lock");
    preview_flake_lock(&lock)
}

#[allow(dead_code)]
fn show_drv_deps(store_path: &str) -> String {
    let out = Command::new("nix-store")
        .args(["--query", "--references", store_path])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if out.is_empty() {
        return format!("No dependencies found for {}", store_path);
    }
    let mut result = String::from("Dependencies:\n");
    for line in out.lines() {
        if let Some(n) = line
            .strip_prefix("/nix/store/")
            .and_then(|s| s.splitn(2, '-').nth(1))
        {
            result.push_str(&format!("  → {}\n", n));
        }
    }
    result
}
