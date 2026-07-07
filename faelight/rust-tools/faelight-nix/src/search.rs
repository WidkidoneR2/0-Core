//! search.rs -- the data layer for INT-076.
//! Wraps `nix search nixpkgs <query> --json`, parses the JSON object
//! (keyed by attr path) into a clean Vec<Package>. stdout only; the
//! `evaluation warning:` noise nix emits goes to stderr and is ignored.

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Package {
    pub attr: String,        // clean attr, e.g. "ripgrep" or "bat-extras.batgrep"
    pub pname: String,       // upstream package name
    pub version: String,     // version string
    pub description: String, // one-line description (may be empty)
}

/// Strip the "legacyPackages.<system>." prefix from a nix-search key,
/// leaving the human-meaningful attribute path.
fn clean_attr(key: &str) -> String {
    // key looks like: legacyPackages.x86_64-linux.ripgrep
    let parts: Vec<&str> = key.splitn(3, '.').collect();
    if parts.len() == 3 && parts[0] == "legacyPackages" {
        parts[2].to_string()
    } else {
        key.to_string()
    }
}

/// Run `nix search nixpkgs <query> --json` and parse the results.
/// Blocking -- evaluates nixpkgs, can take several seconds.
pub fn search(query: &str) -> Result<Vec<Package>> {
    let output = Command::new("nix")
        .args(["search", "nixpkgs", query, "--json"])
        .output()
        .context("failed to run `nix search` -- is nix on PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("nix search failed: {}", stderr.trim());
    }

    // stdout is the JSON object; stderr holds eval warnings (ignored).
    let stdout =
        String::from_utf8(output.stdout).context("nix search stdout was not valid UTF-8")?;

    if stdout.trim().is_empty() {
        return Ok(vec![]);
    }

    let map: BTreeMap<String, Value> =
        serde_json::from_str(&stdout).context("failed to parse nix search JSON")?;

    let mut packages: Vec<Package> = map
        .into_iter()
        .map(|(key, val)| Package {
            attr: clean_attr(&key),
            pname: val
                .get("pname")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: val
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: val
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect();

    // Sort: exact/short attrs first (closest matches), then alphabetical.
    packages.sort_by(|a, b| a.attr.len().cmp(&b.attr.len()).then(a.attr.cmp(&b.attr)));
    Ok(packages)
}
