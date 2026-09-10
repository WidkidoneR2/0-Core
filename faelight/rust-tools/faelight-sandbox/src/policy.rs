//! faelight-sandbox v3 — Policy Engine
//! Declarative TOML policies for sandbox isolation
//! INT-125 Phase 1: policy loading and enforcement

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub name: String,
    #[serde(default = "default_true")]
    pub allow_net: bool,
    #[serde(default = "default_true")]
    pub allow_fs_write: bool,
    #[serde(default)]
    pub allow_fs_read: Vec<String>,
    /// Environment variables passed THROUGH from the parent, by name.
    ///
    /// ⚠️ THIS FIELD HAD NO READER UNTIL 2026-09-06. It parsed, it printed in policy-show,
    /// and `Run` inherited the parent environment whole regardless of it -- so `untrusted`, whose
    /// description says the command is isolated, handed the child every variable this shell has.
    /// An allowlist nothing consults is not a restriction; it is a comment that deserialises.
    ///
    /// An EMPTY list now means EMPTY, not "everything". `default` declares `allow_env = []` and
    /// gets a bare environment, which is the honest reading of what it says.
    #[serde(default)]
    pub allow_env: Vec<String>,
    /// Variables the sandbox SETS for the child, applied after the allowlist so an entry here
    /// overrides an inherited value of the same name.
    ///
    /// This is what makes a SHELL sandbox possible. Passing `HOME` through (as `untrusted` does)
    /// is the opposite of isolation for a shell: nsh-test measured on 2026-09-05 that hiding the
    /// forest takes HOME **and** XDG_STATE_HOME, because state_home reads XDG independently, so a
    /// bare HOME redirect left health, focus.toml and the ledger visible. Redirecting is a
    /// different act from permitting, and only one of them isolates.
    ///
    /// `{session}` in a value is replaced with the sandbox session id.
    #[serde(default)]
    pub set_env: std::collections::HashMap<String, String>,
    /// The working directory the child starts in.
    ///
    /// EXPLICIT, NOT INFERRED. A clean room that silently relocated your cwd would be a
    /// surprise for every policy that does not want it, so nothing moves unless a policy says
    /// so. Measured 2026-09-06: with HOME redirected but cwd inherited, nsh-test pwd_returns_path
    /// compared the real cwd against the sandbox HOME and failed -- the case was right and the
    /// sandbox was half-isolated.
    ///
    /// {session} is substituted as it is in set_env. The directory is created if absent.
    #[serde(default)]
    pub set_cwd: Option<String>,
    /// Controls that MUST actually apply, or the run does not happen.
    ///
    /// THE POLICY IS THE CONTRACT, so this is where "what must be true for this to count as
    /// sandboxed" belongs -- not in the caller, who already said what they wanted by naming the
    /// policy. Recognised names: "net", "seccomp", "pid", "fs".
    ///
    /// Measured 2026-09-12, and this is why it exists. Three separate ways a control could be
    /// absent while the run continued and the report claimed otherwise:
    ///   main.rs  seccomp failing printed "continuing without" and carried on
    ///   main.rs  unshare failing FELL BACK TO NORMAL EXECUTION -- no isolation of any kind,
    ///            after the header had already printed "Network: OFF (isolated)"
    ///   main.rs  seccomp was SILENTLY SKIPPED whenever network isolation was on, which
    ///            `--isolate full` always turns on. No message at all.
    ///
    /// The escape hatch is `--allow-degraded` on the command line: the policy still declares the
    /// requirement, the operator overrides it deliberately, and the override is recorded in the
    /// session report and the emitted event rather than scrolling past on stderr.
    #[serde(default)]
    pub require: Vec<String>,
    #[serde(default = "default_cpu")]
    pub max_cpu_seconds: u64,
    #[serde(default = "default_memory")]
    pub max_memory_mb: u64,
    #[serde(default = "default_true")]
    pub emit_events: bool,
    #[serde(default)]
    pub description: String,
}

fn default_true() -> bool {
    true
}
fn default_cpu() -> u64 {
    300
}
fn default_memory() -> u64 {
    1024
}

#[derive(Debug, Deserialize)]
struct PolicyFile {
    #[serde(rename = "policy")]
    policies: Vec<SandboxPolicy>,
}

impl SandboxPolicy {
    pub fn load(name: &str) -> Result<Self> {
        let policy_path = faelight_core::paths::registry_dir().join("sandbox-policies.toml");

        if !policy_path.exists() {
            anyhow::bail!(
                "Policy file not found: {}\nRun: core sandbox policy list",
                policy_path.display()
            );
        }

        let content = std::fs::read_to_string(&policy_path)
            .with_context(|| format!("Cannot read policy file: {}", policy_path.display()))?;

        let file: PolicyFile =
            toml::from_str(&content).with_context(|| "Failed to parse sandbox-policies.toml")?;

        file.policies
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Policy '{}' not found — run: faelight-sandbox policy list",
                    name
                )
            })
    }

    pub fn list_all() -> Result<Vec<Self>> {
        let policy_path = faelight_core::paths::registry_dir().join("sandbox-policies.toml");

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
        if !self.allow_net {
            r.push("network: isolated".to_string());
        }
        if !self.allow_fs_write {
            r.push("filesystem: read-only".to_string());
        }
        if self.max_cpu_seconds < 300 {
            r.push(format!("cpu: {}s limit", self.max_cpu_seconds));
        }
        if self.max_memory_mb < 1024 {
            r.push(format!("memory: {}MB limit", self.max_memory_mb));
        }
        r
    }
}
