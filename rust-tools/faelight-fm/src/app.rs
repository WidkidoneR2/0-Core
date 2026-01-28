use std::path::PathBuf;
use faelight_fm::git::{self, GitStatus};
use faelight_fm::error::Result;
use faelight_fm::model::{FaelightEntry, HealthStatus, IntentInfo, Zone};
use faelight_fm::{fs, zones, intent};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Mode {
    Normal,
    Command,
}

pub struct AppState {
    pub cwd: PathBuf,
    pub entries: Vec<FaelightEntry>,
    pub filtered_entries: Vec<FaelightEntry>,  // NEW: filtered view
    pub selected: usize,
    pub zone: Zone,
    #[allow(dead_code)]
    pub mode: Mode,
    pub running: bool,
    pub help_visible: bool,
    pub info_visible: bool,
    pub search_mode: bool,      // NEW: search active
    pub search_query: String,   // NEW: search text
    pub preview_visible: bool,  // NEW: preview overlay
    pub preview_content: Option<Vec<String>>,  // NEW: file lines
    pub preview_path: Option<String>,  // NEW: previewed file name
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
            filtered_entries: Vec::new(),
            selected: 0,
            zone,
            mode: Mode::Normal,
            running: true,
            help_visible: false,
            info_visible: false,
            search_mode: false,
            search_query: String::new(),
            preview_visible: false,
            preview_content: None,
            preview_path: None,
            intent_dir,
        };
        
        app.reload()?;
        Ok(app)
    }
    
    pub fn reload(&mut self) -> Result<()> {
        
        // Get git status for all files in directory
        let git_statuses = git::get_status(&self.cwd);
        let paths = fs::read_dir(&self.cwd)?;
        
        self.entries = paths
            .into_iter()
            .filter_map(|path| {
                let name = path.file_name()?.to_string_lossy().to_string();
                let is_dir = path.is_dir();
                
                let is_symlink = path.symlink_metadata()
                    .map(|m| m.is_symlink())
                    .unwrap_or(false);
                
                let zone = zones::classify(&path);
                
                let intents = intent::find_intents_for_path(&self.intent_dir, &path);
                let intent_info = intents.first().map(|i| IntentInfo {
                    title: i.title.clone(),
                    id: i.id.clone(),
                    status: i.status.clone(),
                });
                
                
                // Get git status for this file
                let git_status = git_statuses.get(&name).copied().unwrap_or(GitStatus::Clean);
                Some(FaelightEntry {
                    path,
                    name,
                    is_dir,
                    is_symlink,
                    git_status,
                    zone,
                    health: HealthStatus::Ok,
                    intent_info,
                })
            })
            .collect();
        
        self.apply_filter();
        self.selected = 0;
        Ok(())
    }
    
    pub fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_entries = self.entries.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_entries = self.entries
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&query))
                .cloned()
                .collect();
        }
        
        // Keep selection valid
        if self.selected >= self.filtered_entries.len() && !self.filtered_entries.is_empty() {
            self.selected = self.filtered_entries.len() - 1;
        }
    }
    
    pub fn start_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.apply_filter();
    }
    
    pub fn exit_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.apply_filter();
    }
    
    pub fn search_add_char(&mut self, c: char) {
        self.search_query.push(c);
        self.apply_filter();
    }
    
    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.apply_filter();
    }
    
    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }
    
    pub fn toggle_info(&mut self) {
        self.info_visible = !self.info_visible;
    }
    
    pub fn enter_selected(&mut self) -> Result<()> {
        if let Some(entry) = self.filtered_entries.get(self.selected) {
            if entry.is_dir {
                self.cwd = entry.path.clone();
                self.zone = zones::classify(&self.cwd);
                self.exit_search();  // Clear search when navigating
                self.reload()?;
            }
        }
        Ok(())
    }
    
    pub fn go_parent(&mut self) -> Result<()> {
        if let Some(parent) = self.cwd.parent() {
            self.cwd = parent.to_path_buf();
            self.zone = zones::classify(&self.cwd);
            self.exit_search();  // Clear search when navigating
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
                self.exit_search();  // Clear search when navigating
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
        if self.selected + 1 < self.filtered_entries.len() {
            self.selected += 1;
        }
    }
    
    pub fn selected_entry(&self) -> Option<&FaelightEntry> {
        self.filtered_entries.get(self.selected)
    }
    
    pub fn quit(&mut self) {
        self.running = false;
    }
    
    pub fn toggle_preview(&mut self) {
        self.preview_visible = !self.preview_visible;
        if !self.preview_visible {
            self.preview_content = None;
            self.preview_path = None;
        }
    }
    pub fn load_preview(&mut self) {
        // Clone the entry data first to avoid borrow conflicts
        let entry_data = self.selected_entry().map(|e| (e.path.clone(), e.name.clone(), e.is_dir, e.is_symlink));
        
        if let Some((path, name, is_dir, is_symlink)) = entry_data {
            if !is_dir && !is_symlink {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let lines: Vec<String> = content
                            .lines()
                            .take(40)
                            .map(|l| l.to_string())
                            .collect();
                        self.preview_content = Some(lines);
                        self.preview_path = Some(name);
                    }
                    Err(_) => {
                        self.preview_content = Some(vec!["[Binary or unreadable file]".to_string()]);
                        self.preview_path = Some(name);
                    }
                }
            } else if is_dir {
                self.preview_content = Some(vec!["[Directory - no preview]".to_string()]);
                self.preview_path = Some(name);
            } else {
                self.preview_content = Some(vec!["[Symlink - no preview]".to_string()]);
                self.preview_path = Some(name);
            }
        }
    }
}
