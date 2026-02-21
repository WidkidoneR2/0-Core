use std::collections::HashSet;

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
}

impl CapabilityContext {
    pub fn unprivileged() -> Self {
        let mut granted = HashSet::new();
        granted.insert(Capability::FilesystemReadConfig);
        granted.insert(Capability::FilesystemReadHome);
        granted.insert(Capability::FilesystemWriteRuntime);
        granted.insert(Capability::SpawnProcess);
        granted.insert(Capability::NetworkQuery);
        Self { granted }
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

    pub fn has(&self, cap: &Capability) -> bool {
        self.granted.contains(cap)
    }
}
