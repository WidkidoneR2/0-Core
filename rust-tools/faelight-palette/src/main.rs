//! faelight-palette v2.0.0 - The LEGENDARY Command Palette
//! 🎨 One interface to rule them all - With EVERYTHING!

use clap::Parser;
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
use std::io::{self, BufRead};
use std::process::Command;

#[derive(Parser)]
#[command(name = "faelight-palette")]
#[command(about = "🎨 The legendary command palette - Ultimate edition")]
#[command(version = "2.0.0")]
struct Cli {
    /// Use stdin mode (dmenu compatibility)
    #[arg(long)]
    dmenu: bool,

    /// Prompt text for stdin mode
    #[arg(short, long, default_value = "Select:")]
    prompt: String,
}

// Item types in the palette
#[derive(Debug, Clone)]
enum ItemType {
    App(String, String),    // Application (display_name, executable)
    Action(String, String), // Quick action (display, command)
    File(String),           // Recent file
    Intent(String),         // Intent ledger item
    Emoji(String, String),  // Emoji (name, char)
    Stdin(String),          // Stdin item for dmenu mode
}

impl ItemType {
    fn display(&self) -> String {
        match self {
            ItemType::App(name, _) => format!("🚀 {}", name),
            ItemType::Action(name, _) => format!("⚡ {}", name),
            ItemType::File(path) => format!("📁 {}", path),
            ItemType::Intent(name) => format!("📋 {}", name),
            ItemType::Emoji(name, emoji) => format!("{} :{}", emoji, name),
            ItemType::Stdin(line) => line.clone(),
        }
    }

    fn execute(&self) -> bool {
        match self {
            ItemType::App(_, exec) => {
                if exec.is_empty() {
                    return false; // Stats items have empty exec
                }
                let cmd = format!("setsid -f {} >/dev/null 2>&1", exec);
                let _ = Command::new("sh").arg("-c").arg(&cmd).spawn();
                false
            }
            ItemType::Action(_, cmd) => {
                if cmd.starts_with("TERMINAL_COMMAND:") {
                    let real_cmd = cmd.strip_prefix("TERMINAL_COMMAND:").unwrap();

                    // Replace sudo with pkexec for graphical prompt
                    let fixed_cmd = if real_cmd.starts_with("sudo ") {
                        real_cmd.replacen("sudo ", "pkexec ", 1)
                    } else {
                        real_cmd.to_string()
                    };

                    println!("\n🎨 Executing: {}\n", fixed_cmd);
                    let _ = Command::new("sh").arg("-c").arg(&fixed_cmd).status();
                    return true;
                }
                let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
                false
            }
            ItemType::File(path) => {
                let expanded = path.replace("~", &std::env::var("HOME").unwrap_or_default());
                let _ = Command::new("xdg-open").arg(&expanded).spawn();
                false
            }
            ItemType::Intent(_) => false,
            ItemType::Emoji(_, emoji) => {
                let _ = Command::new("wl-copy").arg(emoji).spawn();
                println!("\n✅ Emoji {} copied to clipboard!\n", emoji);
                false
            }
            ItemType::Stdin(line) => {
                println!("{}", line);
                false
            }
        }
    }
}

struct App {
    input: String,
    items: Vec<ItemType>,
    filtered_items: Vec<(ItemType, i64)>,
    selected: usize,
    matcher: SkimMatcherV2,
    mode: Mode,
    prompt: String,
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
    Stdin,
}

impl App {
    fn new(prompt: String) -> Self {
        let mut app = Self {
            input: String::new(),
            items: Vec::new(),
            filtered_items: Vec::new(),
            selected: 0,
            matcher: SkimMatcherV2::default(),
            mode: Mode::Apps,
            prompt,
        };
        app.load_items();
        app.filter_items();
        app
    }

    fn new_stdin(items: Vec<String>, prompt: String) -> Self {
        let items: Vec<ItemType> = items.into_iter().map(ItemType::Stdin).collect();
        let filtered_items = items.iter().map(|i| (i.clone(), 100)).collect();
        Self {
            input: String::new(),
            items: items.clone(),
            filtered_items,
            selected: 0,
            matcher: SkimMatcherV2::default(),
            mode: Mode::Stdin,
            prompt,
        }
    }

    fn load_items(&mut self) {
        self.items.clear();
        match self.mode {
            Mode::Apps => {
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
                                        self.items.push(ItemType::App(name, exec));
                                    }
                                }
                            }
                        }
                    }
                }
                self.items.sort_by_key(|a| a.display().to_lowercase());
            }
            Mode::Actions => {
                self.items.push(ItemType::Action(
                    "Lock Core".into(),
                    "TERMINAL_COMMAND:sudo /home/christian/0-core/scripts/core-protect lock && pkill faelight-bar; setsid -f faelight-bar >/dev/null 2>&1".into(),
                ));
                self.items.push(ItemType::Action(
                    "Unlock Core".into(),
                    "TERMINAL_COMMAND:sudo /home/christian/0-core/scripts/core-protect unlock && pkill faelight-bar; setsid -f faelight-bar >/dev/null 2>&1".into(),
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
                    "pkill faelight-bar; setsid -f faelight-bar >/dev/null 2>&1".into(),
                ));
                self.items.push(ItemType::Action(
                    "Reload Sway".into(),
                    "swaymsg reload".into(),
                ));
            }
            Mode::Commands => {
                let scripts_dir = "/home/christian/0-core/scripts";
                if let Ok(entries) = std::fs::read_dir(scripts_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !name.starts_with('.') && !name.ends_with('~') && !name.ends_with(".d") {
                            let exec = format!("/home/christian/0-core/scripts/{}", name);
                            self.items.push(ItemType::App(name, exec));
                        }
                    }
                }
                self.items.sort_by_key(|a| a.display().to_lowercase());
            }
            Mode::Files => {
                self.items
                    .push(ItemType::File("~/.config/sway/config".into()));
                self.items.push(ItemType::File("~/0-core/README.md".into()));
                self.items
                    .push(ItemType::File("~/0-core/CHANGELOG-v10.0.0.md".into()));
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
                if self.items.is_empty() {
                    self.items.push(ItemType::Intent("No intents found".into()));
                }
            }
            Mode::Emoji => {
                let emojis = vec![
                    ("rocket", "🚀"),
                    ("fire", "🔥"),
                    ("tree", "🌲"),
                    ("check", "✅"),
                    ("star", "⭐"),
                    ("heart", "❤️"),
                    ("thumbsup", "👍"),
                    ("party", "🎉"),
                    ("eyes", "👀"),
                    ("100", "💯"),
                ];
                for (name, emoji) in emojis {
                    self.items
                        .push(ItemType::Emoji(name.to_string(), emoji.to_string()));
                }
            }
            Mode::Stats => {
                let rust_tools_dir = "/home/christian/0-core/rust-tools";

                // Count total Rust LOC
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
                            self.items.push(ItemType::App(
                                format!("📝 Total Rust LOC: {}", total),
                                String::new(),
                            ));
                        }
                    }
                }

                // Tool counts
                self.items.push(ItemType::App(
                    "🔧 Total Tools: 43".to_string(),
                    String::new(),
                ));
                self.items.push(ItemType::App(
                    "🔗 Total Aliases: 299".to_string(),
                    String::new(),
                ));

                // Disk usage
                if let Ok(output) = Command::new("du")
                    .args(["-sh", "/home/christian/0-core"])
                    .output()
                {
                    if let Ok(result) = String::from_utf8(output.stdout) {
                        if let Some(size) = result.split_whitespace().next() {
                            self.items.push(ItemType::App(
                                format!("💾 0-Core Size: {}", size),
                                String::new(),
                            ));
                        }
                    }
                }

                // Intent stats
                if let Ok(output) = Command::new("sh")
                    .arg("-c")
                    .arg("int list 2>/dev/null | tail -1")
                    .output()
                {
                    if let Ok(result) = String::from_utf8(output.stdout) {
                        let trimmed = result.trim();
                        if !trimmed.is_empty() {
                            self.items
                                .push(ItemType::App(format!("📋 {}", trimmed), String::new()));
                        }
                    }
                }

                // Top 10 Largest Tools
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

                    tool_sizes.sort_by(|a, b| b.1.cmp(&a.1));

                    self.items.push(ItemType::App(
                        "━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string(),
                        String::new(),
                    ));
                    self.items.push(ItemType::App(
                        "🏆 Top 10 Largest Tools:".to_string(),
                        String::new(),
                    ));

                    for (i, (name, lines)) in tool_sizes.iter().take(10).enumerate() {
                        self.items.push(ItemType::App(
                            format!("  {}. {} - {} lines", i + 1, name, lines),
                            String::new(),
                        ));
                    }
                }
            }
            Mode::Stdin => {
                // Items already loaded in new_stdin()
            }
        }
    }

    fn filter_items(&mut self) {
        let query = if self.input.is_empty() {
            ""
        } else {
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

    fn next(&mut self) {
        if self.selected < self.filtered_items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    fn prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn detect_mode_switch(&mut self) {
        if self.input.is_empty() {
            return;
        }

        let new_mode = match self.input.chars().next() {
            Some('>') => Mode::Actions,
            Some('$') => Mode::Commands,
            Some('@') => Mode::Files,
            Some('#') => Mode::Intents,
            Some(':') => Mode::Emoji,
            Some('!') => Mode::Stats,
            _ => return,
        };

        if new_mode != self.mode {
            self.mode = new_mode;
            self.input = self.input.chars().skip(1).collect();
            self.load_items();
            self.filter_items();
        }
    }
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    let stdin_has_data = !atty::is(atty::Stream::Stdin);

    if cli.dmenu || stdin_has_data {
        let stdin = io::stdin();
        let lines: Vec<String> = stdin.lock().lines().map_while(Result::ok).collect();

        if lines.is_empty() {
            eprintln!("No input provided");
            return Ok(());
        }

        run_app(App::new_stdin(lines, cli.prompt))
    } else {
        run_app(App::new(cli.prompt))
    }
}

fn run_app(mut app: App) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Enter => {
                    if !app.filtered_items.is_empty() {
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                        let needs_wait = app.filtered_items[app.selected].0.execute();

                        if needs_wait {
                            println!("\nPress Enter to continue...");
                            let _ = io::stdin().read_line(&mut String::new());
                        }
                        break;
                    }
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                    app.detect_mode_switch();
                    app.filter_items();
                }
                KeyCode::Backspace => {
                    app.input.pop();
                    app.filter_items();
                }
                KeyCode::Down => app.next(),
                KeyCode::Up => app.prev(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.area());

    let title = match app.mode {
        Mode::Apps => "🚀 LAUNCH APPS",
        Mode::Actions => "⚡ QUICK ACTIONS",
        Mode::Commands => "🔧 FAELIGHT TOOLS",
        Mode::Files => "📁 RECENT FILES",
        Mode::Intents => "📋 INTENT LEDGER",
        Mode::Emoji => "🎨 EMOJI PICKER",
        Mode::Stats => "📊 STATISTICS",
        Mode::Stdin => &app.prompt,
    };

    let input_text = if app.input.is_empty() {
        format!(
            "{} (>actions $commands @files #intents :emoji !stats)",
            title
        )
    } else {
        format!("{} > {}", title, app.input)
    };

    let input_widget = Paragraph::new(input_text)
        .style(Style::default().fg(Color::Rgb(163, 227, 107)))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(input_widget, chunks[0]);

    // Calculate scroll offset to keep selected item visible
    let list_height = (chunks[1].height.saturating_sub(2)) as usize; // Account for borders
    let scroll_offset = if app.selected < list_height / 2 {
        0
    } else {
        app.selected.saturating_sub(list_height / 2)
    };

    let items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .skip(scroll_offset)
        .take(list_height)
        .enumerate()
        .map(|(i, (item, _score))| {
            let actual_index = i + scroll_offset;
            let style = if actual_index == app.selected {
                Style::default()
                    .fg(Color::Rgb(17, 20, 15))
                    .bg(Color::Rgb(163, 227, 107))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(218, 224, 215))
            };
            ListItem::new(item.display()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().bg(Color::Rgb(17, 20, 15)));

    f.render_widget(list, chunks[1]);
}
