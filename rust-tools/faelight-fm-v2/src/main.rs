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

fn load_preview(entries: &[FileEntry], selected: Option<usize>, path: &PathBuf) -> String {
    if let Some(idx) = selected {
        if let Some(entry) = entries.get(idx) {
            let full = path.join(&entry.name);
            if entry.is_dir {
                let count = fs::read_dir(&full).map(|d| d.count()).unwrap_or(0);
                return format!("📁 Directory\n{} items\n\nPath:\n{}", count, full.display());
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
                parent_entries, parent_path, preview }, Task::none())
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

    fn view(&self) -> Element<Self::Message> {
        let dir_name = self.current_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or("/".to_string());

        let header = widget::row::with_children(vec![
            widget::button::text("h ↑").on_press(Message::GoUp).into(),
            space::horizontal().into(),
            widget::text(format!("🌲 {}", self.current_path.display())).size(13).into(),
            space::horizontal().into(),
            widget::text(format!("{} items  j/k navigate  l enter  h up", self.entries.len())).size(11).into(),
        ]).spacing(8).padding(8);

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
        let selected_name = self.entries.get(self.selected)
            .map(|e| e.name.clone())
            .unwrap_or_default();
        let status = widget::container(
            widget::text(format!("  🌲 {} / {}  |  INT-287 in progress  |  Health 100%",
                self.current_path.display(), selected_name)).size(11)
        ).width(Length::Fill).padding(4);

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
