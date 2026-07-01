use crate::capabilities::CapabilityContext;
use crate::errors::CoreResult;
use crate::runtime::Runtime;

#[allow(dead_code)]
pub struct AppContext {
    pub runtime: Runtime,
    pub capabilities: CapabilityContext,
    pub home: String,
    pub core_root: String,
}

impl AppContext {
    pub fn init() -> CoreResult<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        // INT-061: core_root derives from the SINGLE path authority (paths.rs),
        // not a local format!() -- so the tree's root is defined in exactly one
        // place. Moving the tree = editing paths.rs, and the engine follows.
        let core_root = faelight_core::paths::core_root_string();
        let runtime = Runtime::init()?;
        let capabilities = CapabilityContext::unprivileged();
        Ok(Self {
            runtime,
            capabilities,
            home,
            core_root,
        })
    }
}
