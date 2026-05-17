// faelight-fm v2 -- INT-293 Phase 3
// Keyboard navigation: j/k up/down, l enter, h go up

use cosmic::app::{Core, Task};
use cosmic::{Application, Element, executor};
use cosmic::iced::Length;
use cosmic::iced::keyboard::{Event as KeyEvent, Key, key::Named};
use cosmic::iced::Event;
use cosmic::widget::{self, space};
use std::path::PathBuf;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum Message {
    SelectEntry(usize),
    GoUp,
    KeyPressed(Key),
    OpenFile(PathBuf),
    DeleteSelected,
    YankPath,
    ConfirmDelete,
    CancelDelete,
}

#[derive(Debug, Clone, PartialEq)]
enum DialogState {
    None,
    ConfirmDelete(String),
    Yanked(String),
}

#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    is_dir: bool,
    size: u64,
    git_status: GitStatus,
}

#[derive(Debug, Clone, PartialEq)]
enum GitStatus {
    Clean,
    Modified,
    Untracked,
}

fn get_git_status(path: &PathBuf) -> std::collections::HashMap<String, GitStatus> {
    let mut map = std::collections::HashMap::new();
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output();
    if let Ok(out) = output {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if line.len() > 3 {
                let status = &line[..2];
                let file = line[3..].to_string();
                let first = file.split('/').next().unwrap_or(&file).to_string();
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
    // Check if this directory has associated intents
    let path_str = path.to_string_lossy();
    let db = "/home/christian/0-core/runtime/state.db";
    let output = std::process::Command::new("sqlite3")
        .args([db,
            &format!("SELECT COUNT(*) FROM friday_knowledge WHERE fact LIKE '%{}%' LIMIT 1",
                path.file_name().unwrap_or_default().to_string_lossy())])
        .output();
    let friday_hits = output.ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .parse::<i32>()
        .unwrap_or(0);

    let mut context = String::new();
    if path_str.contains("intents") {
        context.push_str("📋 Intent directory\n");
    }
    if path_str.contains("rust-tools") {
        context.push_str("🦀 Rust tool source\n");
    }
    if path_str.contains("engine") {
        context.push_str("⚙️  Core engine\n");
    }
    if friday_hits > 0 {
        context.push_str(&format!("🌲 Friday: {} knowledge entries\n", friday_hits));
    }
    context
}

fn load_preview(entries: &[FileEntry], selected: Option<usize>, path: &PathBuf) -> String {
    if let Some(idx) = selected {
        if let Some(entry) = entries.get(idx) {
            let full = path.join(&entry.name);
            if entry.is_dir {
                let count = fs::read_dir(&full).map(|d| d.count()).unwrap_or(0);
                let ctx = get_intent_context(&full);
                return format!("📁 Directory\n{} items\n\nPath:\n{}\n\n{}",
                    count, full.display(), ctx);
            } else {
                if let Ok(content) = fs::read_to_string(&full) {
                    let lines: Vec<&str> = content.lines().take(40).collect();
                    return lines.join("\n");
                }
                return format!("Binary file\n{} bytes", entry.size);
            }
        }
    }
    "Select a file to preview".to_string()
}

struct FaelightFm {
    core: Core,
    current_path: PathBuf,
    entries: Vec<FileEntry>,
    selected: usize,
    parent_entries: Vec<FileEntry>,
    parent_path: Option<PathBuf>,
    preview: String,
    dialog: DialogState,
    status_msg: String,
}

impl FaelightFm {
    fn navigate_into(&mut self) {
        if let Some(entry) = self.entries.get(self.selected).cloned() {
            if entry.is_dir {
                let new_path = self.current_path.join(&entry.name);
                self.parent_path = Some(self.current_path.clone());
                self.parent_entries = self.entries.clone();
                self.current_path = new_path;
                self.entries = load_dir(&self.current_path);
                self.selected = 0;
                self.preview = load_preview(&self.entries, Some(0), &self.current_path);
            }
        }
    }

    fn navigate_up(&mut self) {
        if let Some(parent) = self.current_path.parent().map(|p| p.to_path_buf()) {
            self.entries = load_dir(&parent);
            // find current dir in parent listing
            let cur_name = self.current_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            self.selected = self.entries.iter()
                .position(|e| e.name == cur_name)
                .unwrap_or(0);
            self.parent_path = parent.parent().map(|p| p.to_path_buf());
            self.parent_entries = self.parent_path.as_ref()
                .map(|p| load_dir(p))
                .unwrap_or_default();
            self.current_path = parent;
            self.preview = load_preview(&self.entries, Some(self.selected), &self.current_path);
        }
    }
}

impl Application for FaelightFm {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.faelight.fm";

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _: Self::Flags) -> (Self, Task<Self::Message>) {
        let path = PathBuf::from("/home/christian/0-core");
        let entries = load_dir(&path);
        let parent_path = path.parent().map(|p| p.to_path_buf());
        let parent_entries = parent_path.as_ref().map(|p| load_dir(p)).unwrap_or_default();
        let preview = load_preview(&entries, Some(0), &path);
        (Self { core, current_path: path, entries, selected: 0,
                parent_entries, parent_path, preview,
                dialog: DialogState::None,
                status_msg: String::new() }, Task::none())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectEntry(idx) => {
                self.selected = idx;
                self.preview = load_preview(&self.entries, Some(idx), &self.current_path);
            }
            Message::GoUp => self.navigate_up(),
            Message::KeyPressed(key) => {
                match key {
                    Key::Character(c) => match c.as_str() {
                        "j" => {
                            if self.selected + 1 < self.entries.len() {
                                self.selected += 1;
                                self.preview = load_preview(&self.entries, Some(self.selected), &self.current_path);
                            }
                        }
                        "k" => {
                            if self.selected > 0 {
                                self.selected -= 1;
                                self.preview = load_preview(&self.entries, Some(self.selected), &self.current_path);
                            }
                        }
                        "l" => self.navigate_into(),
                        "h" => self.navigate_up(),
                        "g" => {
                            self.selected = 0;
                            self.preview = load_preview(&self.entries, Some(0), &self.current_path);
                        }
                        "G" => {
                            self.selected = self.entries.len().saturating_sub(1);
                            self.preview = load_preview(&self.entries, Some(self.selected), &self.current_path);
                        }
                        "d" => { return Task::perform(async { Message::DeleteSelected }, |m| cosmic::Action::App(m)); }
                        "y" => { return Task::perform(async { Message::YankPath }, |m| cosmic::Action::App(m)); }
                        _ => {}
                    }
                    Key::Named(Named::ArrowDown) => {
                        if self.selected + 1 < self.entries.len() {
                            self.selected += 1;
                            self.preview = load_preview(&self.entries, Some(self.selected), &self.current_path);
                        }
                    }
                    Key::Named(Named::ArrowUp) => {
                        if self.selected > 0 {
                            self.selected -= 1;
                            self.preview = load_preview(&self.entries, Some(self.selected), &self.current_path);
                        }
                    }
                    Key::Named(Named::Enter) => self.navigate_into(),
                    _ => {}
                }
            }
            Message::OpenFile(_) => {}
            Message::YankPath => {
                if let Some(entry) = self.entries.get(self.selected) {
                    let path = self.current_path.join(&entry.name);
                    let path_str = path.to_string_lossy().to_string();
                    self.dialog = DialogState::Yanked(path_str.clone());
                    self.status_msg = format!("yanked: {}", path_str);
                }
            }
            Message::DeleteSelected => {
                if let Some(entry) = self.entries.get(self.selected) {
                    let path = self.current_path.join(&entry.name);
                    let is_core = path.to_string_lossy().contains("0-core");
                    if is_core {
                        self.dialog = DialogState::ConfirmDelete(
                            format!("⚠️  FOREST SAFETY: Delete {}? (this is in 0-core)", entry.name)
                        );
                    } else {
                        self.dialog = DialogState::ConfirmDelete(
                            format!("Delete {}?", entry.name)
                        );
                    }
                }
            }
            Message::ConfirmDelete => {
                if let Some(entry) = self.entries.get(self.selected).cloned() {
                    let path = self.current_path.join(&entry.name);
                    // Move to trash instead of hard delete
                    let trash_dir = PathBuf::from("/home/christian/.local/share/Trash/files");
                    if let Err(e) = std::fs::create_dir_all(&trash_dir) {
                        self.status_msg = format!("trash error: {}", e);
                    } else {
                        let dest = trash_dir.join(&entry.name);
                        match std::fs::rename(&path, &dest) {
                            Ok(_) => {
                                self.status_msg = format!("🗑️  moved to trash: {}", entry.name);
                                self.entries = load_dir(&self.current_path);
                                if self.selected >= self.entries.len() {
                                    self.selected = self.entries.len().saturating_sub(1);
                                }
                            }
                            Err(e) => self.status_msg = format!("delete error: {}", e),
                        }
                    }
                }
                self.dialog = DialogState::None;
            }
            Message::CancelDelete => {
                self.dialog = DialogState::None;
                self.status_msg = "cancelled".to_string();
            }
        }
        Task::none()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        cosmic::iced::event::listen_with(|event, _status, _window| match event {
            Event::Keyboard(KeyEvent::KeyPressed { key, .. }) => {
                Some(Message::KeyPressed(key))
            }
            _ => None,
        })
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let dir_name = self.current_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or("/".to_string());

        let header = widget::row::with_children(vec![
            widget::button::text("↑").on_press(Message::GoUp).into(),
            space::horizontal().into(),
            widget::text("🌲 Faelight Forest Navigator").size(16).into(),
            space::horizontal().into(),
            widget::text(format!("{}", self.current_path.display())).size(12).into(),
        ]).spacing(12).padding(10);

        // Parent panel
        let mut parent_col = widget::column::with_capacity(self.parent_entries.len());
        for entry in &self.parent_entries {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let is_cur = entry.name == dir_name;
            let label = if is_cur {
                format!("▶ {} {}", icon, entry.name)
            } else {
                format!("   {} {}", icon, entry.name)
            };
            parent_col = parent_col.push(widget::text(label).size(12));
        }
        let parent_panel = widget::container(
            widget::scrollable(parent_col.spacing(1))
        ).width(Length::FillPortion(2)).height(Length::Fill).padding(6);

        // Current panel
        let mut current_col = widget::column::with_capacity(self.entries.len());
        for (idx, entry) in self.entries.iter().enumerate() {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let git = match entry.git_status {
                GitStatus::Modified  => " ●",
                GitStatus::Untracked => " ?",
                GitStatus::Clean     => "",
            };
            let label = format!("{} {}{}", icon, entry.name, git);
            let btn = if self.selected == idx {
                widget::button::text(format!("▶ {}", label))
                    .on_press(Message::SelectEntry(idx))
            } else {
                widget::button::text(format!("   {}", label))
                    .on_press(Message::SelectEntry(idx))
            };
            current_col = current_col.push(btn);
        }
        let current_panel = widget::container(
            widget::scrollable(current_col.spacing(1))
        ).width(Length::FillPortion(3)).height(Length::Fill).padding(6);

        // Preview panel
        let preview_panel = widget::container(
            widget::scrollable(widget::text(&self.preview).size(12))
        ).width(Length::FillPortion(3)).height(Length::Fill).padding(8);

        let columns = widget::row::with_children(vec![
            parent_panel.into(),
            widget::divider::vertical::default().into(),
            current_panel.into(),
            widget::divider::vertical::default().into(),
            preview_panel.into(),
        ]);

        // Status bar
        let _selected_name = self.entries.get(self.selected)
            .map(|e| e.name.clone())
            .unwrap_or_default();
        // Get active intents from filesystem
        let intent_display = {
            let output = std::process::Command::new("grep")
                .args(["-rl", "status: in-progress",
                    "/home/christian/0-core/intents/future/"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();
            let count = output.lines().count();
            if count == 0 {
                "No active intents".to_string()
            } else {
                // Get first intent title
                let first = output.lines().next().unwrap_or("");
                let title = std::process::Command::new("grep")
                    .args(["-m1", "^title:", first])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .unwrap_or_default();
                let title = title.trim().trim_start_matches("title:").trim()
                    .trim_matches('"');
                format!("▸ {} (+{} more)", &title[..title.len().min(40)], count - 1)
            }
        };
        let status_text = if !self.status_msg.is_empty() {
            format!("  {} ", self.status_msg)
        } else {
            format!("  🌲 {}  |  {}  |  j/k l h navigate  y yank  d delete",
                self.current_path.display(), intent_display)
        };
        let status = widget::container(
            widget::text(status_text).size(11)
        ).width(Length::Fill).padding(4);

        // Dialog overlay
        if let DialogState::ConfirmDelete(ref msg) = self.dialog {
            let dialog = widget::container(
                widget::column::with_children(vec![
                    widget::text(msg).size(14).into(),
                    widget::row::with_children(vec![
                        widget::button::destructive("Delete")
                            .on_press(Message::ConfirmDelete).into(),
                        widget::button::text("Cancel")
                            .on_press(Message::CancelDelete).into(),
                    ]).spacing(8).into(),
                ]).spacing(12).padding(20)
            )
            .width(Length::Fixed(500.0));

            return widget::container(
                widget::column::with_children(vec![
                    header.into(),
                    widget::divider::horizontal::default().into(),
                    columns.into(),
                    widget::divider::horizontal::default().into(),
                    status.into(),
                    dialog.into(),
                ])
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        widget::container(
            widget::column::with_children(vec![
                header.into(),
                widget::divider::horizontal::default().into(),
                columns.into(),
                widget::divider::horizontal::default().into(),
                status.into(),
            ])
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<FaelightFm>(
        cosmic::app::Settings::default()
            .size(cosmic::iced::Size::new(1400.0, 900.0)),
        ()
    )
}
