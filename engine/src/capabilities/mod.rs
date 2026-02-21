use chrono::Local;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Capability {
    FilesystemReadConfig,
    FilesystemReadHome,
    FilesystemWriteRuntime,
    FilesystemWriteHome,
    QueryPacman,
    ExecutePacman,
    ControlSystemdUser,
    ControlSway,
    NetworkQuery,
    SpawnProcess,
    ElevatedPrivilege,
    OrchestratorAccess,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::FilesystemReadConfig => write!(f, "filesystem.read.config"),
            Capability::FilesystemReadHome => write!(f, "filesystem.read.home"),
            Capability::FilesystemWriteRuntime => write!(f, "filesystem.write.runtime"),
            Capability::FilesystemWriteHome => write!(f, "filesystem.write.home"),
            Capability::QueryPacman => write!(f, "pacman.query"),
            Capability::ExecutePacman => write!(f, "pacman.execute"),
            Capability::ControlSystemdUser => write!(f, "systemd.user.control"),
            Capability::ControlSway => write!(f, "sway.control"),
            Capability::NetworkQuery => write!(f, "network.query"),
            Capability::SpawnProcess => write!(f, "process.spawn"),
            Capability::ElevatedPrivilege => write!(f, "privilege.elevated"),
            Capability::OrchestratorAccess => write!(f, "orchestrator.access"),
        }
    }
}

pub struct CapabilityContext {
    granted: HashSet<Capability>,
    log_path: PathBuf,
}

impl CapabilityContext {
    pub fn unprivileged() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let log_path = PathBuf::from(&home).join("0-core/runtime/logs/capabilities.jsonl");
        let mut granted = HashSet::new();
        granted.insert(Capability::FilesystemReadConfig);
        granted.insert(Capability::FilesystemReadHome);
        granted.insert(Capability::FilesystemWriteRuntime);
        granted.insert(Capability::SpawnProcess);
        granted.insert(Capability::NetworkQuery);
        granted.insert(Capability::ControlSystemdUser);
        granted.insert(Capability::ControlSway);
        granted.insert(Capability::OrchestratorAccess);
        granted.insert(Capability::FilesystemWriteHome);
        Self { granted, log_path }
    }

    pub fn require(
        &self,
        domain: &str,
        caps: &[Capability],
    ) -> Result<(), crate::errors::CoreError> {
        for cap in caps {
            let granted = self.granted.contains(cap);
            self.log_usage(domain, cap, granted);
            if !granted {
                return Err(crate::errors::CoreError::CapabilityDenied(format!(
                    "{} requires {}",
                    domain, cap
                )));
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn check(&self, required: &[Capability]) -> Result<(), String> {
        for cap in required {
            if !self.granted.contains(cap) {
                return Err(format!("Capability not granted: {}", cap));
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn has(&self, cap: &Capability) -> bool {
        self.granted.contains(cap)
    }

    fn log_usage(&self, domain: &str, cap: &Capability, granted: bool) {
        if let Some(parent) = self.log_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let entry = format!(
            "{{\"ts\":\"{}\",\"domain\":\"{}\",\"capability\":\"{}\",\"granted\":{}}}
",
            Local::now().format("%Y-%m-%dT%H:%M:%S"),
            domain,
            cap,
            granted
        );
        // Append to JSONL log
        use std::io::Write;
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            f.write_all(entry.as_bytes()).ok();
        }
    }
}
