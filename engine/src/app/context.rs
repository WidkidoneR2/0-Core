use crate::capabilities::CapabilityContext;
use crate::errors::CoreResult;
use crate::runtime::Runtime;

#[allow(dead_code)]
pub struct AppContext {
    pub runtime: Runtime,
    pub capabilities: CapabilityContext,
    pub home: String,
    pub core_root: String,
    pub faelight_root: String,
}

impl AppContext {
    pub fn init() -> CoreResult<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        // INT-061: core_root derives from the SINGLE path authority (paths.rs),
        // not a local format!() -- so the tree's root is defined in exactly one
        // place. Moving the tree = editing paths.rs, and the engine follows.
        let core_root = faelight_core::paths::core_root_string();
        // INT-061 v2: the faelight/ platform domain root. Dirs moved under
        // faelight/ (registry, meta, schema, runtime, intents, policy) resolve
        // from here via ctx.fpath(); root-staying dirs keep using core_root.
        let faelight_root = faelight_core::paths::faelight_dir()
            .to_string_lossy()
            .to_string();
        let runtime = Runtime::init()?;
        let capabilities = CapabilityContext::unprivileged();
        Ok(Self {
            runtime,
            capabilities,
            home,
            core_root,
            faelight_root,
        })
    }

    /// Resolve a path in the faelight/ platform domain (registry, meta, schema,
    /// runtime, intents, policy). Root-staying dirs (scripts, rust-tools, engine,
    /// target, flake) use core_root directly, NOT this.
    pub fn fpath(&self, rel: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.faelight_root).join(rel)
    }
}
