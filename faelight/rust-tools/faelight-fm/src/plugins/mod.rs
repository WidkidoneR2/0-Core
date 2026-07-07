// faelight-fm v4.0 -- plugin system
// Static plugins compiled in, toggled at runtime
// Plugin trait: name, handles, preview, actions

pub mod git_plugin;
pub mod intent_plugin;
pub mod nix_plugin;

use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PluginAction {
    pub label: String,
    pub key: char,
    pub description: String,
}

#[allow(dead_code)]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn handles(&self, path: &Path) -> bool;
    fn preview(&self, path: &Path) -> String;
    fn actions(&self, path: &Path) -> Vec<PluginAction>;
    fn execute(&self, path: &Path, action: char) -> String;
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: vec![
                Box::new(nix_plugin::NixPlugin),
                Box::new(git_plugin::GitPlugin),
                Box::new(intent_plugin::IntentPlugin),
            ],
        }
    }

    /// Find first plugin that handles this path
    pub fn find(&self, path: &Path) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| p.handles(path))
            .map(|p| p.as_ref())
    }

    /// Get preview from matching plugin, or None
    pub fn preview(&self, path: &Path) -> Option<String> {
        self.find(path).map(|p| p.preview(path))
    }

    /// Get actions from matching plugin
    #[allow(dead_code)]
    pub fn actions(&self, path: &Path) -> Vec<PluginAction> {
        self.find(path).map(|p| p.actions(path)).unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn list(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }
}
