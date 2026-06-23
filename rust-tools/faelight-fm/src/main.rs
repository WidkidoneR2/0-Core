// faelight-fm v3.1 -- forest-native file manager
// broot-style tree navigation, single and dual panel
// INT-015

mod types;
mod fs;
mod git;
mod nix;
mod search;
mod ui;
mod input;
mod plugins;
mod flake;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::{io, path::PathBuf, process::Command};
use types::{FlatNode, Mode, Panel, TreeNode};

struct PanelData {
    needs_redraw: bool,  // INT-069: force full clear after returning from Helix
    root: PathBuf,
    tree: Vec<TreeNode>,
    flat: Vec<FlatNode>,
    filtered: Vec<usize>,
    list_state: ListState,
    mode: Mode,
    show_hidden: bool,
    preview: String,
}

impl PanelData {
    fn new(path: PathBuf) -> Self {
        let tree = fs::load_tree(&path, 0, false);
        let flat = fs::flatten(&tree);
        let filtered = (0..flat.len()).collect();
        let preview = Self::make_preview(&flat, 0, &path);
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            root: path,
            tree,
            flat,
            filtered,
            list_state,
            mode: Mode::Normal,
            show_hidden: false,
            preview,
            needs_redraw: false,
        }
    }

    fn make_preview(flat: &[FlatNode], display_idx: usize, _root: &PathBuf) -> String {
        // find real index
        if flat.is_empty() { return String::new(); }
        let node = &flat[display_idx.min(flat.len()-1)];
        if node.is_unlisted_marker {
            return format!("{} more items not shown\nPress . to show all", node.unlisted);
        }
        fs::load_preview(&node.node_path, node.is_dir)
    }

    fn refresh_flat(&mut self) {
        self.flat = fs::flatten(&self.tree);
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        let query = match &self.mode {
            Mode::Filter(q) => q.clone(),
            _ => String::new(),
        };
        if query.is_empty() {
            self.filtered = (0..self.flat.len()).collect();
        } else {
            self.filtered = fs::filter_flat(&self.flat, &query);
        }
        let sel = self.list_state.selected().unwrap_or(0)
            .min(self.filtered.len().saturating_sub(1));
        self.list_state.select(Some(sel));
        self.refresh_preview();
    }

    fn refresh_preview(&mut self) {
        let idx = self.list_state.selected().unwrap_or(0);
        let real_idx = self.filtered.get(idx).copied().unwrap_or(0);
        if self.flat.is_empty() { return; }
        let node = &self.flat[real_idx.min(self.flat.len()-1)];
        if node.is_unlisted_marker {
            self.preview = format!("{} more items\nNavigate into to see all", node.unlisted);
            return;
        }
        self.preview = fs::load_preview(&node.node_path, node.is_dir);
    }

    fn selected_real(&self) -> Option<usize> {
        let d = self.list_state.selected().unwrap_or(0);
        self.filtered.get(d).copied()
    }

    fn selected_node(&self) -> Option<&FlatNode> {
        self.selected_real().and_then(|i| self.flat.get(i))
    }

    fn navigate_to(&mut self, path: PathBuf) {
        self.root = path.clone();
        self.tree = fs::load_tree(&path, 0, self.show_hidden);
        self.refresh_flat();
        self.list_state.select(Some(0));
        self.mode = Mode::Normal;
    }

    fn toggle_expand(&mut self) {
        let Some(real_idx) = self.selected_real() else { return; };
        let node_path = self.flat[real_idx].node_path.clone();
        let is_dir = self.flat[real_idx].is_dir;
        let is_unlisted = self.flat[real_idx].is_unlisted_marker;
        if is_unlisted {
            // Navigate into the parent dir
            if let Some(parent) = node_path.parent() {
                self.navigate_to(parent.to_path_buf());
            }
            return;
        }
        if !is_dir {
            // Open file in helix -- suspend TUI first
            let _ = crossterm::terminal::disable_raw_mode();
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::cursor::Show,
            );
            let _ = Command::new("hx").arg(&node_path).status();
            let _ = crossterm::terminal::enable_raw_mode();
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::EnterAlternateScreen,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                crossterm::cursor::Hide,
            );
            self.needs_redraw = true; // INT-069: force a full clear+redraw next loop
            return;
        }
        // Find and toggle in tree
        let display_idx = self.list_state.selected().unwrap_or(0);
        let real_idx = self.filtered[display_idx];
        let depth = self.flat[real_idx].depth;
        toggle_node_in_tree(&mut self.tree, &node_path, depth, self.show_hidden);
        self.refresh_flat();
    }

    fn navigate_up(&mut self) {
        if let Some(parent) = self.root.parent() {
            let parent = parent.to_path_buf();
            let cur_name = self.root.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            self.navigate_to(parent);
            // Select the dir we came from
            if let Some(idx) = self.filtered.iter().position(|&i| {
                self.flat.get(i).map(|n| n.name == cur_name).unwrap_or(false)
            }) {
                self.list_state.select(Some(idx));
                self.refresh_preview();
            }
        }
    }

    fn move_down(&mut self) {
        let max = self.filtered.len().saturating_sub(1);
        let i = (self.list_state.selected().unwrap_or(0) + 1).min(max);
        self.list_state.select(Some(i));
        self.refresh_preview();
    }

    fn move_up_sel(&mut self) {
        let i = self.list_state.selected().unwrap_or(0).saturating_sub(1);
        self.list_state.select(Some(i));
        self.refresh_preview();
    }

    fn yank_path(&mut self) -> String {
        if let Some(node) = self.selected_node() {
            let path = node.node_path.to_string_lossy().to_string();
            let _ = Command::new("wl-copy").arg(&path).spawn();
            return format!("yanked: {}", path);
        }
        String::new()
    }

    fn stage_unstage(&mut self) -> String {
        if let Some(node) = self.selected_node() {
            let filename = node.name.clone();
            let result = match node.git_status {
                types::GitStatus::Staged => git::unstage_file(&self.root, &filename),
                _ => git::stage_file(&self.root, &filename),
            };
            match result {
                Ok(_) => {
                    self.tree = fs::load_tree(&self.root, 0, self.show_hidden);
                    self.refresh_flat();
                    return format!("staged: {}", filename);
                }
                Err(e) => return format!("git error: {}", e),
            }
        }
        String::new()
    }

    fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.tree = fs::load_tree(&self.root, 0, self.show_hidden);
        self.refresh_flat();
    }

    fn nix_info(&self, plugins: &plugins::PluginRegistry) -> String {
        if let Some(node) = self.selected_node() {
            // Try plugin first
            if let Some(preview) = plugins.preview(&node.node_path) {
                return preview;
            }
            if let Some(ref target) = node.symlink_target {
                if target.starts_with("/nix/store/") {
                    return nix::describe_store_path(target);
                }
            }
            return nix::describe_store_path(&node.node_path.to_string_lossy());
        }
        String::new()
    }

    #[allow(dead_code)]
    fn plugin_preview(&self, plugins: &plugins::PluginRegistry) -> Option<String> {
        let node = self.selected_node()?;
        plugins.preview(&node.node_path)
    }
}

fn toggle_node_in_tree(
    nodes: &mut Vec<TreeNode>,
    target: &PathBuf,
    _depth: usize,
    show_hidden: bool,
) -> bool {
    for node in nodes.iter_mut() {
        if &node.path == target {
            if node.expanded {
                fs::collapse_node(node);
            } else {
                fs::expand_node(node, show_hidden);
            }
            return true;
        }
        if node.expanded && toggle_node_in_tree(&mut node.children, target, _depth, show_hidden) {
            return true;
        }
    }
    false
}

struct App {
    left: PanelData,
    right: PanelData,
    active_panel: Panel,
    dual_mode: bool,
    active_intent: String,
    status_msg: String,
    status_timer: u8,
    nix_overlay: Option<String>,
    plugins: plugins::PluginRegistry,
}

impl App {
    fn new(start_path: PathBuf, dual_mode: bool) -> Self {
        let right_path = start_path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| dirs_next::home_dir().unwrap_or_default());
        Self {
            left: PanelData::new(start_path),
            right: PanelData::new(right_path),
            active_panel: Panel::Left,
            dual_mode,
            active_intent: search::get_active_intent(),
            status_msg: String::new(),
            status_timer: 0,
            nix_overlay: None,
            plugins: plugins::PluginRegistry::new(),
        }
    }

    fn active_mut(&mut self) -> &mut PanelData {
        match self.active_panel {
            Panel::Left  => &mut self.left,
            Panel::Right => &mut self.right,
        }
    }

    fn active(&self) -> &PanelData {
        match self.active_panel {
            Panel::Left  => &self.left,
            Panel::Right => &self.right,
        }
    }

    fn set_status(&mut self, msg: String) {
        if !msg.is_empty() {
            self.status_msg = msg;
            self.status_timer = 25;
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        // Nix overlay dismissal
        if self.nix_overlay.is_some() {
            self.nix_overlay = None;
            return false;
        }

        let mode = self.active().mode.clone();

        match mode {
            Mode::Filter(_) => self.handle_filter_key(key),
            Mode::Command(_) => self.handle_command_key(key),
            Mode::ConfirmDelete(ref msg) => {
                let msg = msg.clone();
                self.handle_delete_key(key, &msg);
            },
            Mode::Normal => self.handle_normal_key(key),
        }
        false
    }

    fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => { return; },
            KeyCode::Char('j') | KeyCode::Down  => self.active_mut().move_down(),
            KeyCode::Char('k') | KeyCode::Up    => self.active_mut().move_up_sel(),
            KeyCode::Char('g') => {
                self.active_mut().list_state.select(Some(0));
                self.active_mut().refresh_preview();
            },
            KeyCode::Char('G') => {
                let max = self.active().filtered.len().saturating_sub(1);
                self.active_mut().list_state.select(Some(max));
                self.active_mut().refresh_preview();
            },
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => self.active_mut().toggle_expand(),
            KeyCode::Char('h') | KeyCode::Backspace | KeyCode::Left => self.active_mut().navigate_up(),
            KeyCode::Char('y') => {
                let msg = self.active_mut().yank_path();
                self.set_status(msg);
            },
            KeyCode::Char('d') => {
                if let Some(node) = self.active().selected_node() {
                    let name = node.name.clone();
                    let in_core = node.node_path.to_string_lossy().contains("0-core");
                    let msg = if in_core {
                        format!("⚠️  FOREST SAFETY: delete {}?", name)
                    } else {
                        format!("delete {}?", name)
                    };
                    self.active_mut().mode = Mode::ConfirmDelete(msg);
                }
            },
            KeyCode::Char('s') => {
                let msg = self.active_mut().stage_unstage();
                self.set_status(msg);
            },
            KeyCode::Char('.') => {
                self.active_mut().toggle_hidden();
                let hidden = self.active().show_hidden;
                self.set_status(format!("hidden: {}", if hidden { "shown" } else { "hidden" }));
            },
            KeyCode::Char('n') => {
                let info = self.active().nix_info(&self.plugins);
                self.nix_overlay = Some(info);
            },
            KeyCode::Char('r') => {
                self.nix_overlay = Some(flake::format_gc_roots());
            },
            KeyCode::Char('/') => {
                self.active_mut().mode = Mode::Filter(String::new());
            },
            KeyCode::Char(':') => {
                self.active_mut().mode = Mode::Command(String::new());
            },
            KeyCode::Tab => {
                if self.dual_mode {
                    self.active_panel = match self.active_panel {
                        Panel::Left  => Panel::Right,
                        Panel::Right => Panel::Left,
                    };
                }
            },
            _ => {}
        }
    }

    fn handle_filter_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.active_mut().mode = Mode::Normal;
                self.active_mut().filtered = (0..self.active().flat.len()).collect();
                self.active_mut().refresh_preview();
            },
            KeyCode::Enter => {
                self.active_mut().mode = Mode::Normal;
            },
            KeyCode::Backspace => {
                if let Mode::Filter(ref mut q) = self.active_mut().mode {
                    q.pop();
                    if q.is_empty() {
                        self.active_mut().mode = Mode::Normal;
                        self.active_mut().filtered = (0..self.active().flat.len()).collect();
                        return;
                    }
                }
                self.active_mut().apply_filter();
            },
            KeyCode::Down => self.active_mut().move_down(),
            KeyCode::Up   => self.active_mut().move_up_sel(),
            KeyCode::Char('j') => self.active_mut().move_down(),
            KeyCode::Char('k') => self.active_mut().move_up_sel(),
            KeyCode::Char(c) => {
                if let Mode::Filter(ref mut q) = self.active_mut().mode {
                    q.push(c);
                }
                self.active_mut().apply_filter();
            },
            _ => {}
        }
    }

    fn handle_command_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => { self.active_mut().mode = Mode::Normal; },
            KeyCode::Enter => {
                if let Mode::Command(ref cmd) = self.active().mode.clone() {
                    let msg = self.execute_command(cmd);
                    self.set_status(msg);
                }
                self.active_mut().mode = Mode::Normal;
            },
            KeyCode::Backspace => {
                if let Mode::Command(ref mut c) = self.active_mut().mode {
                    c.pop();
                }
            },
            KeyCode::Char(c) => {
                if let Mode::Command(ref mut cmd) = self.active_mut().mode {
                    cmd.push(c);
                }
            },
            _ => {}
        }
    }

    fn handle_delete_key(&mut self, key: crossterm::event::KeyEvent, _msg: &str) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(node) = self.active().selected_node() {
                    let path = node.node_path.clone();
                    let name = node.name.clone();
                    let trash = dirs_next::home_dir().unwrap_or_default()
                        .join(".local/share/Trash/files");
                    let _ = std::fs::create_dir_all(&trash);
                    match std::fs::rename(&path, trash.join(&name)) {
                        Ok(_) => {
                            let msg = format!("🗑️  moved to trash: {}", name);
                            self.active_mut().mode = Mode::Normal;
                            let root = self.active().root.clone();
                            let hidden = self.active().show_hidden;
                            self.active_mut().tree = fs::load_tree(&root, 0, hidden);
                            self.active_mut().refresh_flat();
                            self.set_status(msg);
                            return;
                        },
                        Err(e) => self.set_status(format!("error: {}", e)),
                    }
                }
            },
            _ => self.set_status("cancelled".to_string()),
        }
        self.active_mut().mode = Mode::Normal;
    }

    fn execute_command(&mut self, cmd: &str) -> String {
        let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
        match parts[0] {
            "cd" => {
                let path = if parts.len() > 1 {
                    PathBuf::from(parts[1])
                } else {
                    dirs_next::home_dir().unwrap_or_default()
                };
                if path.is_dir() {
                    self.active_mut().navigate_to(path);
                    return String::new();
                }
                format!("not a directory: {}", parts.get(1).unwrap_or(&""))
            },
            "cp" if self.dual_mode => {
                if let Some(src) = self.active().selected_node() {
                    let src_path = src.node_path.clone();
                    let dst = match self.active_panel {
                        Panel::Left  => self.right.root.join(&src.name),
                        Panel::Right => self.left.root.join(&src.name),
                    };
                    match std::fs::copy(&src_path, &dst) {
                        Ok(_) => {
                            let other_root = dst.parent().unwrap_or(&dst).to_path_buf();
                            let hidden = match self.active_panel {
                                Panel::Left  => self.right.show_hidden,
                                Panel::Right => self.left.show_hidden,
                            };
                            match self.active_panel {
                                Panel::Left  => { self.right.tree = fs::load_tree(&other_root, 0, hidden); self.right.refresh_flat(); },
                                Panel::Right => { self.left.tree = fs::load_tree(&other_root, 0, hidden); self.left.refresh_flat(); },
                            }
                            return format!("copied to {}", dst.display());
                        },
                        Err(e) => return format!("copy failed: {}", e),
                    }
                }
                "nothing selected".to_string()
            },
            "mv" if self.dual_mode => {
                if let Some(src) = self.active().selected_node() {
                    let src_path = src.node_path.clone();
                    let name = src.name.clone();
                    let dst = match self.active_panel {
                        Panel::Left  => self.right.root.join(&name),
                        Panel::Right => self.left.root.join(&name),
                    };
                    match std::fs::rename(&src_path, &dst) {
                        Ok(_) => {
                            let root = self.active().root.clone();
                            let hidden = self.active().show_hidden;
                            self.active_mut().tree = fs::load_tree(&root, 0, hidden);
                            self.active_mut().refresh_flat();
                            let other_root = dst.parent().unwrap_or(&dst).to_path_buf();
                            let other_hidden = match self.active_panel {
                                Panel::Left  => self.right.show_hidden,
                                Panel::Right => self.left.show_hidden,
                            };
                            match self.active_panel {
                                Panel::Left  => { self.right.tree = fs::load_tree(&other_root, 0, other_hidden); self.right.refresh_flat(); },
                                Panel::Right => { self.left.tree = fs::load_tree(&other_root, 0, other_hidden); self.left.refresh_flat(); },
                            }
                            return format!("moved to {}", dst.display());
                        },
                        Err(e) => return format!("move failed: {}", e),
                    }
                }
                "nothing selected".to_string()
            },
            _ => format!("unknown command: {}", parts[0]),
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let dual_mode = args.iter().any(|a| a == "--dual" || a == "-d");
    // INT-069: only accept a DIRECTORY arg as the start path. fsh passes a cwd temp file
    // (/tmp/fsh-cwd.tmp) which is NOT a directory -- ignore it and use the real cwd.
    let start_path = args.iter()
        .find(|a| !a.starts_with('-') && *a != &args[0])
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Panic hook -- restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show,
        );
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(start_path, dual_mode);

    loop {
        // INT-069: after returning from Helix the screen is stale; clear forces a full repaint.
        if app.active().needs_redraw {
            terminal.clear()?;
            app.active_mut().needs_redraw = false;
        }
        // Render
        if let Some(ref overlay) = app.nix_overlay.clone() {
            // Show nix info as preview overlay
            let left_state = ui::PanelState {
                root: &app.left.root,
                flat: &app.left.flat,
                filtered: &app.left.filtered,
                list_state: &mut app.left.list_state,
                mode: &app.left.mode,
                active_intent: &app.active_intent,
            };
            ui::render_single(&mut terminal, left_state, overlay, "press any key to dismiss")?;
        } else if app.dual_mode {
            let active_panel = &app.active_panel;
            let status = app.status_msg.clone();
            let intent = app.active_intent.clone();
            let (left_flat, left_filtered, left_root, left_mode) = (
                &app.left.flat, &app.left.filtered, &app.left.root, &app.left.mode,
            );
            let (right_flat, right_filtered, right_root, right_mode) = (
                &app.right.flat, &app.right.filtered, &app.right.root, &app.right.mode,
            );
            let left_state = ui::PanelState {
                root: left_root, flat: left_flat, filtered: left_filtered,
                list_state: &mut app.left.list_state, mode: left_mode,
                active_intent: &intent,
            };
            let right_state = ui::PanelState {
                root: right_root, flat: right_flat, filtered: right_filtered,
                list_state: &mut app.right.list_state, mode: right_mode,
                active_intent: &intent,
            };
            ui::render_dual(&mut terminal, left_state, right_state, active_panel, &status)?;
        } else {
            let preview = app.left.preview.clone();
            let status = app.status_msg.clone();
            let intent = app.active_intent.clone();
            let left_state = ui::PanelState {
                root: &app.left.root,
                flat: &app.left.flat,
                filtered: &app.left.filtered,
                list_state: &mut app.left.list_state,
                mode: &app.left.mode,
                active_intent: &intent,
            };
            ui::render_single(&mut terminal, left_state, &preview, &status)?;
        }

        // Events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT {
                    if let Mode::Normal = app.active().mode {
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                            break;
                        }
                    }
                    app.handle_key(key);
                }
            }
        }

        // Status timer
        if app.status_timer > 0 {
            app.status_timer -= 1;
            if app.status_timer == 0 {
                app.status_msg.clear();
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::event::DisableMouseCapture,
        crossterm::cursor::Show,
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;
    Ok(())
}
