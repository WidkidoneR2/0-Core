use std::path::PathBuf;
use crate::error::Result;
use crate::model::{FaelightEntry, HealthStatus, Zone};
use crate::{fs, zones, intent};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Mode {
    Normal,
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
    intent_dir: PathBuf,
}

impl AppState {
    pub fn new(start_path: PathBuf) -> Result<Self> {
        let zone = zones::classify(&start_path);
        
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/home"));
        let intent_dir = home.join("0-core/INTENT");
        
        let mut app = Self {
            cwd: start_path.clone(),
            entries: Vec::new(),
            selected: 0,
            zone,
            mode: Mode::Normal,
            running: true,
            intent_dir,
        };
        
        app.reload()?;
        Ok(app)
    }
    
    pub fn reload(&mut self) -> Result<()> {
        let paths = fs::read_dir(&self.cwd)?;
        
        self.entries = paths
            .into_iter()
            .filter_map(|path| {
                let name = path.file_name()?.to_string_lossy().to_string();
                let is_dir = path.is_dir();
                let zone = zones::classify(&path);
                
                // Find intents for this path
                let intents = intent::find_intents_for_path(&self.intent_dir, &path);
                let intent_id = intents.first().map(|i| i.id.clone());
                
                Some(FaelightEntry {
                    path,
                    name,
                    is_dir,
                    zone,
                    health: HealthStatus::Ok,
                    intent_id,
                })
            })
            .collect();
        
        self.selected = 0;
        Ok(())
    }
    
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
    
    pub fn go_parent(&mut self) -> Result<()> {
        if let Some(parent) = self.cwd.parent() {
            self.cwd = parent.to_path_buf();
            self.zone = zones::classify(&self.cwd);
            self.reload()?;
        }
        Ok(())
    }
    
    pub fn jump_to_zone(&mut self, zone: Zone) -> Result<()> {
        if let Some(zone_path) = zones::zone_root(zone) {
            let expanded = shellexpand::tilde(zone_path).to_string();
            let path = PathBuf::from(&expanded);
            
            if path.exists() {
                self.cwd = path.clone();
                self.zone = zone;
                self.reload()?;
            }
        }
        Ok(())
    }
    
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
    
    pub fn select_next(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }
    
    pub fn selected_entry(&self) -> Option<&FaelightEntry> {
        self.entries.get(self.selected)
    }
    
    pub fn quit(&mut self) {
        self.running = false;
    }
}
