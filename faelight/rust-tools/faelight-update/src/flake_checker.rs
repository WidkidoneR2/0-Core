// flake_checker.rs -- INT-074 Phase 1a: legible per-input flake state.
//
// Reports each root flake input: its locked rev, how stale the lock is, and what ref it
// tracks. This is the FOUNDATION of the update manager -- the legible per-input view.
//
// Phase 1a scope (this module, now): enumerate + report. It does NOT fetch upstream to show
// "rev abc -> rev def would update", and does NOT do the closure diff or gated rebuild --
// those are Phase 1b. We surface what is locked and how old, per input.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn flake_dir() -> String {
    std::env::var("HOME").map(|h| format!("{h}/0-core")).unwrap_or_else(|_| ".".into())
}

/// Check root flake inputs: locked rev + lock age + tracked ref. One UpdateItem per input.
pub fn check_flake_updates() -> Vec<crate::UpdateItem> {
    println!("   Checking flake inputs...");
    let dir = flake_dir();
    let output = Command::new("nix")
        .args(["flake", "metadata", "--json"])
        .current_dir(&dir)
        .output();

    let stdout = match output {
        Ok(o) if o.status.success() => o.stdout,
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            eprintln!("      ⚠️  nix flake metadata failed: {}", err.lines().next().unwrap_or("").trim());
            return Vec::new();
        }
        Err(e) => {
            eprintln!("      ⚠️  nix not available: {e}");
            return Vec::new();
        }
    };

    parse_flake_metadata(&stdout)
}

fn parse_flake_metadata(stdout: &[u8]) -> Vec<crate::UpdateItem> {
    let json: serde_json::Value = match serde_json::from_slice(stdout) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("      ⚠️  could not parse flake metadata JSON: {e}");
            return Vec::new();
        }
    };

    let nodes = match json.get("locks").and_then(|l| l.get("nodes")).and_then(|n| n.as_object()) {
        Some(n) => n,
        None => return Vec::new(),
    };

    let root_inputs = match nodes
        .get("root")
        .and_then(|r| r.get("inputs"))
        .and_then(|i| i.as_object())
    {
        Some(r) => r,
        None => return Vec::new(),
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut items = Vec::new();
    let mut names: Vec<&String> = root_inputs.keys().collect();
    names.sort();

    for name in names {
        let node_key = match root_inputs.get(name) {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Array(a)) => a
                .first()
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            _ => continue,
        };

        let node = match nodes.get(&node_key) {
            Some(n) => n,
            None => continue,
        };

        let locked = node.get("locked");
        let original = node.get("original");

        let rev = locked
            .and_then(|l| l.get("rev"))
            .and_then(|r| r.as_str())
            .map(|s| s.chars().take(8).collect::<String>())
            .or_else(|| {
                locked
                    .and_then(|l| l.get("ref"))
                    .and_then(|r| r.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "?".into());

        let age = locked
            .and_then(|l| l.get("lastModified"))
            .and_then(|m| m.as_i64())
            .map(|lm| {
                let days = (now - lm) / 86_400;
                format!("{days}d ago")
            })
            .unwrap_or_else(|| "unknown age".into());

        let tracks = original
            .and_then(|o| o.get("ref"))
            .and_then(|r| r.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                original
                    .and_then(|o| o.get("type"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "--".into());

        items.push(crate::UpdateItem {
            name: name.clone(),
            current: format!("{rev} ({age})"),
            new: tracks,
            repository: None,
        });
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_metadata() {
        let fixture = br#"{
            "locks": {
                "nodes": {
                    "root": { "inputs": { "nixpkgs": "nixpkgs", "crane": "crane" } },
                    "nixpkgs": {
                        "locked": { "rev": "e8210c649915abcdef", "lastModified": 0, "type": "github" },
                        "original": { "ref": "nixos-26.05", "type": "github" }
                    },
                    "crane": {
                        "locked": { "rev": "469fd08d0bcf1234", "lastModified": 0, "type": "github" },
                        "original": { "type": "github" }
                    }
                }
            }
        }"#;
        let items = parse_flake_metadata(fixture);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "crane");
        assert_eq!(items[1].name, "nixpkgs");
        assert!(items[1].current.starts_with("e8210c64"));
        assert_eq!(items[1].new, "nixos-26.05");
    }
}
