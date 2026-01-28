use std::path::PathBuf;
use faelight_fm::git::{self, GitStatus};
use faelight_fm::error::Result;
use faelight_fm::model::{FaelightEntry, HealthStatus, IntentInfo, Zone};
use faelight_fm::{fs, zones, intent};
use faelight_fm::daemon::DaemonClient;

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
    pub daemon_client: Option<DaemonClient>,  // Daemon connection
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
            daemon_client: {
                let client = DaemonClient::new();
                if client.is_available() {
                    Some(client)
                } else {
                    None
                }
            },
            intent_dir,
        };
        
        app.reload()?;
        Ok(app)
    }
    
    /// Try to load entries from daemon
    fn try_daemon_load(&mut self) -> Option<Vec<FaelightEntry>> {
        let client = self.daemon_client.as_mut()?;
        
        // Create a tokio runtime for this sync context
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;
        
        let path_str = self.cwd.to_string_lossy().to_string();
        
        // Get entries from daemon
        let response = rt.block_on(async {
            use faelight_fm::daemon::client::Command;
            client.send_command(Command::GetEntries { path: path_str }).await
        }).ok()?;
        
        // Convert daemon entries to FaelightEntry
        if let faelight_fm::daemon::client::Response::Entries { entries } = response {
            let git_statuses = git::get_status(&self.cwd);
            
            let faelight_entries: Vec<FaelightEntry> = entries.into_iter()
                .map(|daemon_entry| {
                    let path = std::path::PathBuf::from(&daemon_entry.path);
                    let zone = zones::classify(&path);
                    
                    let intents = intent::find_intents_for_path(&self.intent_dir, &path);
                    let intent_info = intents.first().map(|i| IntentInfo {
                        title: i.title.clone(),
                        id: i.id.clone(),
                        status: i.status.clone(),
                    });
                    
                    let git_status = git_statuses
                        .get(&daemon_entry.name)
                        .copied()
                        .unwrap_or(GitStatus::Clean);
                    
                    FaelightEntry {
                        path,
                        name: daemon_entry.name,
                        is_dir: daemon_entry.is_dir,
                        is_symlink: false, // TODO: daemon should track this
                        git_status,
                        zone,
                        health: HealthStatus::Ok,
                        intent_info,
                    }
                })
                .collect();
            
            return Some(faelight_entries);
        }
        
        None
    }
    pub fn reload(&mut self) -> Result<()> {
        // Try daemon first, fall back to filesystem
        self.entries = if let Some(entries) = self.try_daemon_load() {
            entries
        } else {
            
            // Get git status for all files in directory
            let git_statuses = git::get_status(&self.cwd);
            let paths = fs::read_dir(&self.cwd)?;
            
            paths
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
                .collect()
        };
        
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

    /// Edit the selected file in nvim
    /// Edit the selected file in nvim
    /// Edit the selected file in nvim
    pub fn edit_selected(&mut self) -> Result<()> {
        let entry = match self.selected_entry() {
            Some(e) => e.clone(),
            None => return Ok(()),
        };
        
        // Only edit files, not directories
        if entry.is_dir {
            return Ok(());
        }
        
        let path = entry.path.to_string_lossy().to_string();
        
        // Fully cleanup terminal before launching nvim
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        )?;
        
        // Launch nvim
        let status = std::process::Command::new("nvim")
            .arg(&path)
            .status()?;
        
        // Fully restore terminal state
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::Hide
        )?;
        crossterm::terminal::enable_raw_mode()?;
        
        // Reload in case file changed
        self.reload()?;
        
        Ok(())
    }
}
