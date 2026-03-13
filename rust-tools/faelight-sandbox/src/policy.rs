//! faelight-sandbox v3 — Policy Engine
//! Declarative TOML policies for sandbox isolation
//! INT-125 Phase 1: policy loading and enforcement

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub name: String,
    #[serde(default = "default_true")]
    pub allow_net: bool,
    #[serde(default = "default_true")]
    pub allow_fs_write: bool,
    #[serde(default)]
    pub allow_fs_read: Vec<String>,
    #[serde(default)]
    pub allow_env: Vec<String>,
    #[serde(default = "default_cpu")]
    pub max_cpu_seconds: u64,
    #[serde(default = "default_memory")]
    pub max_memory_mb: u64,
    #[serde(default = "default_true")]
    pub emit_events: bool,
    #[serde(default)]
    pub description: String,
}

fn default_true() -> bool { true }
fn default_cpu() -> u64 { 300 }
fn default_memory() -> u64 { 1024 }

#[derive(Debug, Deserialize)]
struct PolicyFile {
    #[serde(rename = "policy")]
    policies: Vec<SandboxPolicy>,
}

impl SandboxPolicy {
    pub fn load(name: &str) -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_default();
        let policy_path = PathBuf::from(&home)
            .join("0-core/01-registry/sandbox-policies.toml");

        if !policy_path.exists() {
            anyhow::bail!(
                "Policy file not found: {}\nRun: core sandbox policy list",
                policy_path.display()
            );
        }

        let content = std::fs::read_to_string(&policy_path)
            .with_context(|| format!("Cannot read policy file: {}", policy_path.display()))?;

        let file: PolicyFile = toml::from_str(&content)
            .with_context(|| "Failed to parse sandbox-policies.toml")?;

        file.policies
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!(
                "Policy '{}' not found — run: faelight-sandbox policy list", name
            ))
    }

    pub fn list_all() -> Result<Vec<Self>> {
        let home = std::env::var("HOME").unwrap_or_default();
        let policy_path = PathBuf::from(&home)
            .join("0-core/01-registry/sandbox-policies.toml");

        if !policy_path.exists() {
            return Ok(vec![]);
        }

        let content = std::fs::read_to_string(&policy_path)?;
        let file: PolicyFile = toml::from_str(&content)?;
        Ok(file.policies)
    }

    /// Describe what this policy restricts
    pub fn restrictions(&self) -> Vec<String> {
        let mut r = vec![];
        if !self.allow_net { r.push("network: isolated".to_string()); }
        if !self.allow_fs_write { r.push("filesystem: read-only".to_string()); }
        if self.max_cpu_seconds < 300 {
            r.push(format!("cpu: {}s limit", self.max_cpu_seconds));
        }
        if self.max_memory_mb < 1024 {
            r.push(format!("memory: {}MB limit", self.max_memory_mb));
        }
        r
    }
}
