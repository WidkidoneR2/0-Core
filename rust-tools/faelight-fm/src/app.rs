use std::path::PathBuf;
use crate::model::{FaelightEntry, HealthStatus, Zone};
use crate::error::Result;
use crate::zones;
use crate::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    #[allow(dead_code)]
    Command,
}

pub struct AppState {
    pub cwd: PathBuf,
    pub entries: Vec<FaelightEntry>,
    pub selected: usize,
    pub zone: Zone,
    #[allow(dead_code)]
    pub mode: Mode,
    pub running: bool,
}

impl AppState {
    pub fn new(start_path: PathBuf) -> Result<Self> {
        let zone = zones::classify(&start_path);
        let mut app = Self {
            cwd: start_path.clone(),
            entries: Vec::new(),
            selected: 0,
            zone,
            mode: Mode::Normal,
            running: true,
        };
        
        app.reload()?;
        Ok(app)
    }
    
    /// Reload current directory
    pub fn reload(&mut self) -> Result<()> {
        let paths = fs::read_dir(&self.cwd)?;
        
        self.entries = paths
            .into_iter()
            .filter_map(|path| {
                let name = path.file_name()?.to_string_lossy().to_string();
                let is_dir = path.is_dir();
                let zone = zones::classify(&path);
                
                Some(FaelightEntry {
                    path,
                    name,
                    is_dir,
                    zone,
                    health: HealthStatus::Ok,
                })
            })
            .collect();
        
        self.selected = 0;
        Ok(())
    }
    
    /// Navigate to parent directory
    pub fn go_parent(&mut self) -> Result<()> {
        if let Some(parent) = self.cwd.parent() {
            self.cwd = parent.to_path_buf();
            self.zone = zones::classify(&self.cwd);
            self.reload()?;
        }
        Ok(())
    }
    
    /// Enter selected directory
    pub fn enter_selected(&mut self) -> Result<()> {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                self.cwd = entry.path.clone();
                self.zone = zones::classify(&self.cwd);
                self.reload()?;
            }
        }
        Ok(())
    }
    
    /// Jump to a zone's root directory
    pub fn jump_to_zone(&mut self, zone: Zone) -> Result<()> {
        if let Some(zone_path) = zones::zone_root(zone) {
            let expanded = shellexpand::tilde(zone_path).to_string();
            let path = PathBuf::from(expanded);
            
            if path.exists() {
                self.cwd = path;
                self.zone = zone;
                self.reload()?;
            }
        }
        Ok(())
    }
    
    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected < self.entries.len().saturating_sub(1) {
            self.selected += 1;
        }
    }
    
    /// Get currently selected entry
    pub fn selected_entry(&self) -> Option<&FaelightEntry> {
        self.entries.get(self.selected)
    }
    
    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}
