//! faelight-palette v1.0.0 - The Legendary Command Palette
//! 🎨 One interface to rule them all

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::process::Command;

// Item types in the palette
#[derive(Debug, Clone)]
enum ItemType {
    App(String),            // Application to launch
    Action(String, String), // Quick action (display, command)
    File(String),           // Recent file
    Intent(String),         // Intent ledger item
    Emoji(String, String),  // Emoji (name, char)
}

impl ItemType {
    fn display(&self) -> String {
        match self {
            ItemType::App(name) => format!("🚀 {}", name),
            ItemType::Action(name, _) => format!("⚡ {}", name),
            ItemType::File(path) => format!("📁 {}", path),
            ItemType::Intent(name) => format!("📋 {}", name),
            ItemType::Emoji(name, emoji) => format!("{} :{}", emoji, name),
        }
    }

    fn execute(&self) -> bool {
        match self {
            ItemType::App(name) => {
                let _ = Command::new("sh")
                    .arg("-c")
                    .arg(format!("{} &", name.to_lowercase()))
                    .spawn();
                false
            }
            ItemType::Action(_, cmd) => {
                // Special handling for terminal commands
                if cmd.starts_with("TERMINAL_COMMAND:") {
                    let real_cmd = cmd.strip_prefix("TERMINAL_COMMAND:").unwrap();
                    println!(
                        "
[1;36m🎨 Executing: {}[0m
",
                        real_cmd
                    );
                    let _ = Command::new("sh").arg("-c").arg(real_cmd).status();
                    return true; // Signal to wait for key
                }
                let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
                false
            }
            ItemType::File(path) => {
                let _ = Command::new("xdg-open").arg(path).spawn();
                false
            }
            ItemType::Intent(_) => {
                // TODO: Open intent in editor
                false
            }
            ItemType::Emoji(_, emoji) => {
                // Copy emoji to clipboard
                let _ = Command::new("wl-copy").arg(emoji).spawn();
                println!(
                    "
[1;32m✅ Emoji {} copied to clipboard![0m
",
                    emoji
                );
                false
            }
        }
    }
}

struct App {
    input: String,
    items: Vec<ItemType>,
    filtered_items: Vec<(ItemType, i64)>, // (item, score)
    selected: usize,
    matcher: SkimMatcherV2,
    mode: Mode,
}

#[derive(Debug, PartialEq)]
enum Mode {
    Apps,
    Actions,
    Commands,
    Files,
    Intents,
    Emoji,
    Stats,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            input: String::new(),
            items: Vec::new(),
            filtered_items: Vec::new(),
            selected: 0,
            matcher: SkimMatcherV2::default(),
            mode: Mode::Apps,
        };
        app.load_items();
        app.filter_items();
        app
    }

    fn load_items(&mut self) {
        self.items.clear();

        match self.mode {
            Mode::Apps => {
                // Parse .desktop files for rich app discovery
                let desktop_dirs = ["/usr/share/applications"];

                for dir in &desktop_dirs {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    let mut in_main_section = false;
                                    let mut name = String::new();
                                    let mut exec = String::new();

                                    for line in content.lines() {
                                        let trimmed = line.trim();
                                        if trimmed == "[Desktop Entry]" {
                                            in_main_section = true;
                                        } else if trimmed.starts_with('[') {
                                            in_main_section = false;
                                        } else if in_main_section {
                                            if trimmed.starts_with("Name=") && name.is_empty() {
                                                name = trimmed[5..].to_string();
                                            } else if trimmed.starts_with("Exec=")
                                                && exec.is_empty()
                                            {
                                                exec = trimmed[5..]
                                                    .split_whitespace()
                                                    .next()
                                                    .unwrap_or("")
                                                    .to_string();
                                            }
                                        }
                                        if !name.is_empty() && !exec.is_empty() {
                                            break;
                                        }
                                    }

                                    if !name.is_empty() && !exec.is_empty() {
                                        self.items.push(ItemType::App(name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Mode::Actions => {
                self.items.push(ItemType::Action(
                    "Lock Core".into(),
                    "TERMINAL_COMMAND:sudo /home/christian/0-core/scripts/core-protect lock && pkill faelight-bar && faelight-bar &".into(),
                ));
                self.items.push(ItemType::Action(
                    "Unlock Core".into(),
                    "TERMINAL_COMMAND:sudo /home/christian/0-core/scripts/core-protect unlock && pkill faelight-bar && faelight-bar &".into(),
                ));
                self.items.push(ItemType::Action(
                    "Health Check".into(),
                    "TERMINAL_COMMAND:/home/christian/.local/bin/doctor".into(),
                ));
                self.items.push(ItemType::Action(
                    "Dashboard".into(),
                    "foot -e faelight-dashboard &".into(),
                ));
                self.items.push(ItemType::Action(
                    "Reload Bar".into(),
                    "pkill faelight-bar && faelight-bar &".into(),
                ));
                self.items.push(ItemType::Action(
                    "Reload Sway".into(),
                    "swaymsg reload".into(),
                ));
            }
            Mode::Commands => {
                // Discover all Faelight tools from scripts directory
                let scripts_dir = "/home/christian/0-core/scripts";
                if let Ok(entries) = std::fs::read_dir(scripts_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        // Skip hidden files and backups
                        if !name.starts_with('.') && !name.ends_with('~') {
                            self.items.push(ItemType::App(name));
                        }
                    }
                }
            }
            Mode::Files => {
                // Recent files in 0-core
                self.items
                    .push(ItemType::File("~/.config/sway/config".into()));
                self.items.push(ItemType::File("~/0-core/README.md".into()));
                self.items
                    .push(ItemType::File("~/0-core/CHANGELOG.md".into()));
            }
            Mode::Intents => {
                if let Ok(output) = Command::new("sh")
                    .arg("-c")
                    .arg("int list 2>/dev/null | grep -E '^(COMPLETE|PLANNED|IN_PROGRESS)' | head -20")
                    .output()
                {
                    if let Ok(result) = String::from_utf8(output.stdout) {
                        for line in result.lines() {
                            self.items.push(ItemType::Intent(line.to_string()));
                        }
                    }
                }
            }
            Mode::Emoji => {
                // Popular emojis
                self.items
                    .push(ItemType::Emoji("rocket".into(), "🚀".into()));
                self.items.push(ItemType::Emoji("fire".into(), "🔥".into()));
                self.items.push(ItemType::Emoji("tree".into(), "🌲".into()));
                self.items
                    .push(ItemType::Emoji("check".into(), "✅".into()));
                self.items.push(ItemType::Emoji("star".into(), "⭐".into()));
                self.items
                    .push(ItemType::Emoji("heart".into(), "❤️".into()));
                self.items
                    .push(ItemType::Emoji("thumbsup".into(), "👍".into()));
                self.items
                    .push(ItemType::Emoji("party".into(), "🎉".into()));
            }
            Mode::Stats => {
                // Calculate Rust code statistics
                let rust_tools_dir = "/home/christian/0-core/rust-tools";

                // Count Rust LOC
                if let Ok(output) = Command::new("sh")
                    .arg("-c")
                    .arg(format!(
                        "find {} -name '*.rs' -exec wc -l {{}} + 2>/dev/null | tail -1",
                        rust_tools_dir
                    ))
                    .output()
                {
                    if let Ok(result) = String::from_utf8(output.stdout) {
                        if let Some(total) = result.split_whitespace().next() {
                            self.items
                                .push(ItemType::App(format!("📝 Total Rust LOC: {}", total)));
                        }
                    }
                }

                // Tool count
                self.items
                    .push(ItemType::App("🔧 Total Tools: 43".to_string()));
                self.items
                    .push(ItemType::App("🔗 Total Aliases: 299".to_string()));

                // Disk usage
                if let Ok(output) = Command::new("du")
                    .args(["-sh", "/home/christian/0-core"])
                    .output()
                {
                    if let Ok(result) = String::from_utf8(output.stdout) {
                        if let Some(size) = result.split_whitespace().next() {
                            self.items
                                .push(ItemType::App(format!("💾 0-Core Size: {}", size)));
                        }
                    }
                }

                // Intent stats
                if let Ok(output) = Command::new("sh")
                    .arg("-c")
                    .arg("intent list 2>/dev/null | tail -1")
                    .output()
                {
                    if let Ok(result) = String::from_utf8(output.stdout) {
                        let trimmed = result.trim();
                        if !trimmed.is_empty() {
                            self.items.push(ItemType::App(format!("📋 {}", trimmed)));
                        }
                    }
                }

                // Add individual tool stats - Top 10 Largest
                if let Ok(entries) = std::fs::read_dir(rust_tools_dir) {
                    let mut tool_sizes: Vec<(String, usize)> = Vec::new();

                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let src_dir = entry.path().join("src");

                        if src_dir.exists() {
                            if let Ok(output) = Command::new("sh")
                                .arg("-c")
                                .arg(format!(
                                    "find {} -name '*.rs' -exec wc -l {{}} + 2>/dev/null | tail -1",
                                    src_dir.display()
                                ))
                                .output()
                            {
                                if let Ok(result) = String::from_utf8(output.stdout) {
                                    if let Some(lines) = result.split_whitespace().next() {
                                        if let Ok(count) = lines.parse::<usize>() {
                                            tool_sizes.push((name, count));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Sort by size (largest first)
                    tool_sizes.sort_by(|a, b| b.1.cmp(&a.1));

                    // Show top 10
                    self.items
                        .push(ItemType::App("━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string()));
                    self.items
                        .push(ItemType::App("🏆 Top 10 Largest Tools:".to_string()));
                    for (i, (name, lines)) in tool_sizes.iter().take(10).enumerate() {
                        self.items.push(ItemType::App(format!(
                            "  {}. {} - {} lines",
                            i + 1,
                            name,
                            lines
                        )));
                    }
                }
            }
        }
    }

    fn filter_items(&mut self) {
        let query = if self.input.is_empty() {
            ""
        } else {
            // Remove mode prefix
            match self.input.chars().next() {
                Some('>') | Some('@') | Some('$') | Some('#') | Some(':') | Some('!') => {
                    &self.input[1..]
                }
                _ => &self.input,
            }
        };

        if query.is_empty() {
            self.filtered_items = self.items.iter().map(|item| (item.clone(), 100)).collect();
        } else {
            let mut scored: Vec<(ItemType, i64)> = self
                .items
                .iter()
                .filter_map(|item| {
                    let text = item.display();
                    self.matcher
                        .fuzzy_match(&text, query)
                        .map(|score| (item.clone(), score))
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_items = scored;
        }

        self.selected = 0;
    }

    fn update_mode(&mut self) {
        let new_mode = match self.input.chars().next() {
            Some('>') => Mode::Actions,
            Some('$') => Mode::Commands,
            Some('@') => Mode::Files,
            Some('#') => Mode::Intents,
            Some(':') => Mode::Emoji,
            Some('!') => Mode::Stats,
            _ => Mode::Apps,
        };

        if new_mode != self.mode {
            self.mode = new_mode;
            self.load_items();
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Enter => {
                    if !app.filtered_items.is_empty() {
                        // Restore terminal before executing
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                        let needs_wait = app.filtered_items[app.selected].0.execute();

                        if needs_wait {
                            println!(
                                "
[1;33mPress Enter to continue...[0m"
                            );
                            let _ = std::io::stdin().read_line(&mut String::new());
                        }
                        break;
                    }
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                    app.update_mode();
                    app.filter_items();
                }
                KeyCode::Backspace => {
                    app.input.pop();
                    app.update_mode();
                    app.filter_items();
                }
                KeyCode::Down => {
                    if app.selected < app.filtered_items.len().saturating_sub(1) {
                        app.selected += 1;
                    }
                }
                KeyCode::Up => {
                    app.selected = app.selected.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Input
            Constraint::Min(0),    // Results
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Title
    let mode_text = match app.mode {
        Mode::Apps => "🚀 LAUNCH APPS",
        Mode::Actions => "⚡ QUICK ACTIONS",
        Mode::Commands => "🔧 FAELIGHT TOOLS",
        Mode::Files => "📁 RECENT FILES",
        Mode::Intents => "📋 INTENT LEDGER",
        Mode::Emoji => "😀 EMOJI PICKER",
        Mode::Stats => "📊 STATISTICS",
    };

    let title = Paragraph::new(format!("🎨 FAELIGHT COMMAND PALETTE │ {}", mode_text))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Input
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search (Esc to quit, Enter to execute)"),
        );
    f.render_widget(input, chunks[1]);

    // Results
    let items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .enumerate()
        .map(|(i, (item, _score))| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let text = item.display();
            ListItem::new(text).style(style)
        })
        .collect();

    let results = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} results", app.filtered_items.len())),
        )
        .highlight_style(Style::default().bg(Color::Blue));

    f.render_widget(results, chunks[2]);

    // Help
    let help = Paragraph::new("Modes: Apps (default) | >Actions | @Files | #Intents | :Emoji")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[3]);
}
