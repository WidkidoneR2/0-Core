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
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
        let core_root = format!("{}/0-core", home);
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
