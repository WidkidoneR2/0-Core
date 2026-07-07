// faelight-fm v4.0 -- flake browser
// Browse flake.lock inputs as virtual directories

use std::{path::Path, process::Command};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FlakeInput {
    pub name: String,
    pub input_type: String,
    pub url: String,
    pub rev: String,
    pub store_path: Option<String>,
}

#[allow(dead_code)]
pub fn parse_flake_lock(flake_dir: &Path) -> Vec<FlakeInput> {
    let lock_path = flake_dir.join("flake.lock");
    let Ok(content) = std::fs::read_to_string(&lock_path) else {
        return vec![];
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return vec![];
    };
    let Some(nodes) = json.get("nodes").and_then(|n| n.as_object()) else {
        return vec![];
    };
    let mut inputs = vec![];
    for (name, node) in nodes {
        if name == "root" {
            continue;
        }
        let locked = node.get("locked");
        let input_type = locked
            .and_then(|l| l.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("unknown")
            .to_string();
        let rev = locked
            .and_then(|l| l.get("rev"))
            .and_then(|r| r.as_str())
            .unwrap_or("?")
            .to_string();
        let url = match input_type.as_str() {
            "github" => {
                let owner = locked
                    .and_then(|l| l.get("owner"))
                    .and_then(|o| o.as_str())
                    .unwrap_or("?");
                let repo = locked
                    .and_then(|l| l.get("repo"))
                    .and_then(|r| r.as_str())
                    .unwrap_or("?");
                format!("github:{}/{}", owner, repo)
            }
            "gitlab" => {
                let owner = locked
                    .and_then(|l| l.get("owner"))
                    .and_then(|o| o.as_str())
                    .unwrap_or("?");
                let repo = locked
                    .and_then(|l| l.get("repo"))
                    .and_then(|r| r.as_str())
                    .unwrap_or("?");
                format!("gitlab:{}/{}", owner, repo)
            }
            "path" => locked
                .and_then(|l| l.get("path"))
                .and_then(|p| p.as_str())
                .unwrap_or("?")
                .to_string(),
            _ => format!("{}:{}", input_type, rev),
        };
        inputs.push(FlakeInput {
            name: name.clone(),
            input_type,
            url,
            rev: rev[..rev.len().min(8)].to_string(),
            store_path: None,
        });
    }
    inputs.sort_by(|a, b| a.name.cmp(&b.name));
    inputs
}

#[allow(dead_code)]
pub fn find_flake_store_path(input: &FlakeInput) -> Option<String> {
    // Try to find this input in the nix store
    let out = Command::new("nix")
        .args(["flake", "metadata", "--json", &input.url])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())?;
    let json: serde_json::Value = serde_json::from_str(&out).ok()?;
    json.get("path")
        .and_then(|p| p.as_str())
        .map(|s| s.to_string())
}

#[allow(dead_code)]
pub fn flake_inputs_as_virtual_entries(flake_dir: &Path) -> Vec<VirtualEntry> {
    let inputs = parse_flake_lock(flake_dir);
    inputs.into_iter().map(|inp| VirtualEntry {
        name: inp.name.clone(),
        display: format!("{} [{}@{}]", inp.name, inp.input_type, inp.rev),
        url: inp.url.clone(),
        is_dir: true,
        preview: format!(
            "❄️  Flake Input: {}\n\nType:    {}\nURL:     {}\nRev:     {}\n\nPress i to inspect store path",
            inp.name, inp.input_type, inp.url, inp.rev
        ),
    }).collect()
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VirtualEntry {
    pub name: String,
    pub display: String,
    pub url: String,
    pub is_dir: bool,
    pub preview: String,
}

pub fn gc_roots() -> Vec<String> {
    let out = Command::new("nix-store")
        .args(["--gc", "--print-roots"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    out.lines()
        .filter(|l| !l.contains("ephemeral") && !l.contains("/proc/"))
        .filter(|l| l.contains("/nix/store/") || l.contains("/home/"))
        .map(|l| l.to_string())
        .take(20)
        .collect()
}

pub fn format_gc_roots() -> String {
    let roots = gc_roots();
    if roots.is_empty() {
        return "No GC roots found".to_string();
    }
    let mut out = format!("⚓ GC Roots ({} total)\n━━━━━━━━━━━━━━━━━━━\n", roots.len());
    for root in &roots {
        // Shorten display
        let parts: Vec<&str> = root.splitn(4, ' ').collect();
        if parts.len() >= 3 {
            let path = parts[0];
            let target = parts[2];
            let short = if target.contains("/nix/store/") {
                target
                    .strip_prefix("/nix/store/")
                    .and_then(|s| s.splitn(2, '-').nth(1))
                    .unwrap_or(target)
            } else {
                target
            };
            out.push_str(&format!(
                "  ⚓ {} → {}\n",
                path.rsplit('/').next().unwrap_or(path),
                short
            ));
        } else {
            out.push_str(&format!("  ⚓ {}\n", root));
        }
    }
    out
}
