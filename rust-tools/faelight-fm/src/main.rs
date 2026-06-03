// faelight-fm v3 -- INT-004
// broot-inspired, ratatui, forest-native navigation
// Three panels: parent | current | preview
// Keybinds: j/k navigate, l enter, h go up, g top, G bottom
//           y yank path, d delete (to trash), q quit

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    fs,
    io,
    path::PathBuf,
    process::Command,
};

// ── Data structures ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum GitStatus { Clean, Modified, Untracked }

#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    is_dir: bool,
    size: u64,
    git_status: GitStatus,
}

#[derive(Debug, PartialEq)]
enum Mode {
    Normal,
    ConfirmDelete(String),
    Yanked(String),
}

struct App {
    current_path: PathBuf,
    entries: Vec<FileEntry>,
    list_state: ListState,
    parent_entries: Vec<FileEntry>,
    preview: String,
    status_msg: String,
    mode: Mode,
    active_intent: String,
}

// ── Filesystem helpers ───────────────────────────────────────────────────────

fn get_git_status(path: &PathBuf) -> std::collections::HashMap<String, GitStatus> {
    let mut map = std::collections::HashMap::new();
    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| PathBuf::from(s.trim()));
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output();
    if let Ok(out) = output {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if line.len() > 3 {
                let status = &line[..2];
                let file_path = line[3..].to_string();
                let rel = if let Some(ref root) = git_root {
                    let abs = root.join(&file_path);
                    abs.strip_prefix(path)
                        .ok()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or(file_path.clone())
                } else { file_path.clone() };
                let first = rel.split('/').next().unwrap_or(&rel).to_string();
                if first.is_empty() || first.starts_with('.') { continue; }
                let gs = if status.contains('?') { GitStatus::Untracked } else { GitStatus::Modified };
                map.entry(first).or_insert(gs);
            }
        }
    }
    map
}

fn load_dir(path: &PathBuf) -> Vec<FileEntry> {
    let git_map = get_git_status(path);
    let mut entries = vec![];
    if let Ok(dir) = fs::read_dir(path) {
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }
            let meta = entry.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let git_status = git_map.get(&name).cloned().unwrap_or(GitStatus::Clean);
            entries.push(FileEntry { name, is_dir, size, git_status });
        }
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    entries
}

fn get_intent_context(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy();
    let mut ctx = String::new();
    if path_str.contains("intents") { ctx.push_str("📋 Intent directory\n"); }
    if path_str.contains("rust-tools") { ctx.push_str("🦀 Rust tool source\n"); }
    if path_str.contains("engine") { ctx.push_str("⚙️  Core engine\n"); }
    if path_str.contains("pkgs") { ctx.push_str("📦 Nix derivations\n"); }
    if path_str.contains("modules") { ctx.push_str("🔧 NixOS module\n"); }
    ctx
}

fn load_preview(entries: &[FileEntry], selected: Option<usize>, path: &PathBuf) -> String {
    if let Some(idx) = selected {
        if let Some(entry) = entries.get(idx) {
            let full = path.join(&entry.name);
            if entry.is_dir {
                let count = fs::read_dir(&full).map(|d| d.count()).unwrap_or(0);
                let ctx = get_intent_context(&full);
                return format!("📁 {} items\n📍 {}\n\n{}", count, full.display(), ctx);
            } else {
                if let Ok(content) = fs::read_to_string(&full) {
                    return content.lines().take(50).collect::<Vec<_>>().join("\n");
                }
                return format!("Binary file\n{} bytes", entry.size);
            }
        }
    }
    "Select a file to preview".to_string()
}

fn get_active_intent() -> String {
    let intents_dir = "/home/christian/0-core/intents/in-progress";
    if let Ok(entries) = fs::read_dir(intents_dir) {
        let files: Vec<_> = entries.flatten()
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .collect();
        if files.is_empty() { return "No active intents".to_string(); }
        let first = &files[0];
        if let Ok(content) = fs::read_to_string(first.path()) {
            for line in content.lines() {
                if let Some(t) = line.strip_prefix("title:") {
                    let title = t.trim().trim_matches('"');
                    let short = if title.len() > 35 { &title[..35] } else { title };
                    let more = if files.len() > 1 { format!(" (+{})", files.len()-1) } else { String::new() };
                    return format!("▸ {}{}", short, more);
                }
            }
        }
    }
    "No active intents".to_string()
}

// ── App implementation ───────────────────────────────────────────────────────

impl App {
    fn new() -> Self {
        let path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/home/christian/0-core"));
        let entries = load_dir(&path);
        let parent_entries = path.parent()
            .map(|p| load_dir(&p.to_path_buf()))
            .unwrap_or_default();
        let preview = load_preview(&entries, Some(0), &path);
        let active_intent = get_active_intent();
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            current_path: path,
            entries,
            list_state,
            parent_entries,
            preview,
            status_msg: String::new(),
            mode: Mode::Normal,
            active_intent,
        }
    }

    fn selected(&self) -> usize { self.list_state.selected().unwrap_or(0) }

    fn navigate_into(&mut self) {
        if let Some(entry) = self.entries.get(self.selected()).cloned() {
            if entry.is_dir {
                let new_path = self.current_path.join(&entry.name);
                self.parent_entries = self.entries.clone();
                self.current_path = new_path;
                self.entries = load_dir(&self.current_path);
                self.list_state.select(Some(0));
                self.preview = load_preview(&self.entries, Some(0), &self.current_path);
            } else {
                let file_path = self.current_path.join(&entry.name);
                let _ = Command::new("helix")
                    .arg(&file_path)
                    .spawn();
            }
        }
    }

    fn navigate_up(&mut self) {
        if let Some(parent) = self.current_path.parent().map(|p| p.to_path_buf()) {
            let cur_name = self.current_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            self.entries = load_dir(&parent);
            let idx = self.entries.iter().position(|e| e.name == cur_name).unwrap_or(0);
            self.list_state.select(Some(idx));
            self.parent_entries = parent.parent()
                .map(|p| load_dir(&p.to_path_buf()))
                .unwrap_or_default();
            self.current_path = parent;
            self.preview = load_preview(&self.entries, Some(idx), &self.current_path);
        }
    }

    fn move_down(&mut self) {
        let i = (self.selected() + 1).min(self.entries.len().saturating_sub(1));
        self.list_state.select(Some(i));
        self.preview = load_preview(&self.entries, Some(i), &self.current_path);
    }

    fn move_up(&mut self) {
        let i = self.selected().saturating_sub(1);
        self.list_state.select(Some(i));
        self.preview = load_preview(&self.entries, Some(i), &self.current_path);
    }

    fn yank_path(&mut self) {
        if let Some(entry) = self.entries.get(self.selected()) {
            let path = self.current_path.join(&entry.name).to_string_lossy().to_string();
            // Copy to clipboard via wl-copy
            let _ = Command::new("wl-copy").arg(&path).spawn();
            self.status_msg = format!("yanked: {}", path);
            self.mode = Mode::Yanked(path);
        }
    }

    fn delete_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected()) {
            let path = self.current_path.join(&entry.name);
            let is_core = path.to_string_lossy().contains("0-core");
            let msg = if is_core {
                format!("⚠️  FOREST SAFETY: Delete {}? (in 0-core) [y/N]", entry.name)
            } else {
                format!("Delete {}? [y/N]", entry.name)
            };
            self.mode = Mode::ConfirmDelete(msg);
        }
    }

    fn confirm_delete(&mut self) {
        if let Some(entry) = self.entries.get(self.selected()).cloned() {
            let path = self.current_path.join(&entry.name);
            let trash = PathBuf::from("/home/christian/.local/share/Trash/files");
            let _ = fs::create_dir_all(&trash);
            let dest = trash.join(&entry.name);
            match fs::rename(&path, &dest) {
                Ok(_) => {
                    self.status_msg = format!("🗑️  moved to trash: {}", entry.name);
                    self.entries = load_dir(&self.current_path);
                    let idx = self.selected().min(self.entries.len().saturating_sub(1));
                    self.list_state.select(Some(idx));
                    self.preview = load_preview(&self.entries, Some(idx), &self.current_path);
                }
                Err(e) => self.status_msg = format!("error: {}", e),
            }
        }
        self.mode = Mode::Normal;
    }
}

// ── Rendering ────────────────────────────────────────────────────────────────

fn render(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();

        // Main layout: top bar | content | status bar
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(size);

        // Header
        let path_display = app.current_path.display().to_string();
        let short_path = if path_display.len() > 60 {
            format!("...{}", &path_display[path_display.len()-57..])
        } else { path_display };
        let header = Paragraph::new(format!("🌲 faelight-fm  {}", short_path))
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
        f.render_widget(header, main_chunks[0]);

        // Three panels
        let panel_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(40),
            ])
            .split(main_chunks[1]);

        // Parent panel
        let cur_dir_name = app.current_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let parent_items: Vec<ListItem> = app.parent_entries.iter().map(|e| {
            let icon = if e.is_dir { "📁" } else { "📄" };
            let prefix = if e.name == cur_dir_name { "▶ " } else { "  " };
            let style = if e.name == cur_dir_name {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(format!("{}{} {}", prefix, icon, e.name)).style(style)
        }).collect();
        let parent_list = List::new(parent_items)
            .block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(parent_list, panel_chunks[0]);

        // Current panel
        let current_items: Vec<ListItem> = app.entries.iter().enumerate().map(|(i, e)| {
            let icon = if e.is_dir { "📁" } else { "📄" };
            let git_badge = match e.git_status {
                GitStatus::Modified  => Span::styled(" ✎", Style::default().fg(Color::Yellow)),
                GitStatus::Untracked => Span::styled(" +", Style::default().fg(Color::Green)),
                GitStatus::Clean     => Span::raw(""),
            };
            let prefix = if app.selected() == i { "▶ " } else { "  " };
            let name_style = if app.selected() == i {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if e.is_dir {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            let short_name = if e.name.len() > 28 {
                format!("{}…", &e.name[..27])
            } else { e.name.clone() };
            ListItem::new(Line::from(vec![
                Span::raw(format!("{}{} ", prefix, icon)),
                Span::styled(short_name, name_style),
                git_badge,
            ]))
        }).collect();
        let current_list = List::new(current_items)
            .block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(Color::DarkGray)));
        f.render_stateful_widget(current_list, panel_chunks[1], &mut app.list_state);

        // Preview panel
        let preview_text = match &app.mode {
            Mode::ConfirmDelete(msg) => {
                format!("{}\n\nPress y to confirm, n to cancel", msg)
            }
            Mode::Yanked(path) => format!("📋 Yanked:\n{}", path),
            Mode::Normal => app.preview.clone(),
        };
        let preview = Paragraph::new(preview_text)
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Gray));
        f.render_widget(preview, panel_chunks[2]);

        // Status bar
        let status = if !app.status_msg.is_empty() {
            app.status_msg.clone()
        } else {
            format!("  {}  |  j/k↕  l→  h←  y yank  d delete  q quit", app.active_intent)
        };
        let status_widget = Paragraph::new(status)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(status_widget, main_chunks[2]);
    })?;
    Ok(())
}

// ── Event handling ───────────────────────────────────────────────────────────

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match &app.mode {
        Mode::ConfirmDelete(_) => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete(),
                _ => { app.mode = Mode::Normal; app.status_msg = "cancelled".to_string(); }
            }
        }
        Mode::Yanked(_) => { app.mode = Mode::Normal; app.status_msg.clear(); }
        Mode::Normal => {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Char('j') | KeyCode::Down  => app.move_down(),
                KeyCode::Char('k') | KeyCode::Up    => app.move_up(),
                KeyCode::Char('l') | KeyCode::Enter => app.navigate_into(),
                KeyCode::Char('h') | KeyCode::Backspace => app.navigate_up(),
                KeyCode::Char('g') => { app.list_state.select(Some(0)); app.preview = load_preview(&app.entries, Some(0), &app.current_path); }
                KeyCode::Char('G') => { let i = app.entries.len().saturating_sub(1); app.list_state.select(Some(i)); app.preview = load_preview(&app.entries, Some(i), &app.current_path); }
                KeyCode::Char('y') => app.yank_path(),
                KeyCode::Char('d') => app.delete_selected(),
                KeyCode::Char('.') => {
                    // Toggle hidden files -- future enhancement
                    app.status_msg = "hidden files: coming in v3.1".to_string();
                }
                _ => {}
            }
        }
    }
    false
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        render(&mut terminal, &mut app)?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_key(&mut app, key) { break; }
            }
        }
        // Clear one-shot status messages after a few renders
        if app.mode == Mode::Normal && app.status_msg.starts_with("yanked:") {
            // keep it visible
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
