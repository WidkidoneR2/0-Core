//! faelight-palette v3.0.0
//! Split-view command palette — fast + rich

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io::{self, BufRead};
use std::process::Command;

// ── colours ──────────────────────────────────────────────
const BG: Color = Color::Rgb(17, 20, 15);
const FG: Color = Color::Rgb(218, 224, 215);
const GREEN: Color = Color::Rgb(163, 227, 107);
const DIM: Color = Color::Rgb(90, 100, 80);
const ACCENT: Color = Color::Rgb(120, 190, 80);
const WARNING: Color = Color::Rgb(230, 180, 60);
const RED: Color = Color::Rgb(220, 80, 80);

// ── item types ────────────────────────────────────────────
#[derive(Debug, Clone)]
enum Item {
    App {
        name: String,
        exec: String,
    },
    Action {
        label: String,
        cmd: String,
        terminal: bool,
    },
    Script {
        name: String,
        path: String,
    },
    File {
        path: String,
    },
    Intent {
        text: String,
    },
    Stdin {
        text: String,
    },
}

impl Item {
    fn display(&self) -> String {
        match self {
            Item::App { name, .. } => name.clone(),
            Item::Action { label, .. } => format!("⚡ {}", label),
            Item::Script { name, .. } => format!("$ {}", name),
            Item::File { path } => format!("  {}", path),
            Item::Intent { text } => format!("  {}", text),
            Item::Stdin { text } => text.clone(),
        }
    }

    fn execute(&self) -> bool {
        match self {
            Item::App { exec, .. } | Item::Script { path: exec, .. } => {
                let _ = Command::new("sh")
                    .arg("-c")
                    .arg(format!("setsid -f {} >/dev/null 2>&1", exec))
                    .spawn();
                false
            }
            Item::Action { cmd, terminal, .. } => {
                if *terminal {
                    let _ = Command::new("sh").arg("-c").arg(cmd).status();
                    true
                } else {
                    let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
                    false
                }
            }
            Item::File { path } => {
                let p = path.replace('~', &std::env::var("HOME").unwrap_or_default());
                let _ = Command::new("xdg-open").arg(p).spawn();
                false
            }
            Item::Intent { .. } | Item::Stdin { text: _ } => false,
        }
    }
}

// ── modes ─────────────────────────────────────────────────
#[derive(Debug, PartialEq, Clone)]
enum Mode {
    Apps,
    Actions,
    Scripts,
    Files,
    Intents,
    Stdin,
}

// ── stats panel ───────────────────────────────────────────
#[derive(Default)]
struct Stats {
    health: String,
    health_pct: u8,
    loc: String,
    tools: String,
    commits: String,
    git: String,
    core: String,
    top_tools: Vec<(String, usize)>,
}

impl Stats {
    fn load() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let core = format!("{}/0-core", home);

        // health % — read from shared cache (same source as bar)
        let health_pct = std::fs::read_to_string(
            std::path::PathBuf::from(&home).join(".cache/faelight/health-status"),
        )
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0);

        // LOC
        let loc = Command::new("sh")
            .arg("-c")
            .arg(format!("find {}/rust-tools {}/engine -name '*.rs' -exec wc -l {{}} + 2>/dev/null | tail -1", core, core))
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.split_whitespace().next().unwrap_or("?").to_string())
            .unwrap_or("?".into());

        // tools count
        let tools = Command::new("sh")
            .arg("-c")
            .arg(format!("ls {}/rust-tools | wc -l", core))
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or("?".into());

        // commits
        let commits = Command::new("sh")
            .arg("-c")
            .arg(format!("git -C {} rev-list --count HEAD 2>/dev/null", core))
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or("?".into());

        // git status
        let git = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "git -C {} status --porcelain 2>/dev/null | wc -l",
                core
            ))
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                if s.trim() == "0" {
                    "✅ clean".into()
                } else {
                    "⚠  dirty".into()
                }
            })
            .unwrap_or("?".into());

        // core lock status
        let core_lock = Command::new("sh")
            .arg("-c")
            .arg(format!("lsattr {}/engine/src/main.rs 2>/dev/null", core))
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                let attrs = s.split_whitespace().next().unwrap_or("");
                if attrs.contains('i') {
                    "🔒 locked".into()
                } else {
                    "🔓 unlocked".into()
                }
            })
            .unwrap_or("?".into());

        // top tools by LOC
        let mut top_tools: Vec<(String, usize)> = Vec::new();
        let rt = format!("{}/rust-tools", core);
        if let Ok(entries) = std::fs::read_dir(&rt) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let src = entry.path().join("src");
                if src.exists() {
                    if let Ok(o) = Command::new("sh")
                        .arg("-c")
                        .arg(format!(
                            "find {} -name '*.rs' -exec wc -l {{}} + 2>/dev/null | tail -1",
                            src.display()
                        ))
                        .output()
                    {
                        if let Ok(s) = String::from_utf8(o.stdout) {
                            if let Some(n) =
                                s.split_whitespace().next().and_then(|x| x.parse().ok())
                            {
                                top_tools.push((name, n));
                            }
                        }
                    }
                }
            }
        }
        top_tools.sort_by(|a, b| b.1.cmp(&a.1));
        top_tools.truncate(8);

        Stats {
            health: format!("{}%", health_pct),
            health_pct,
            loc,
            tools,
            commits,
            git,
            core: core_lock,
            top_tools,
        }
    }
}

// ── app state ─────────────────────────────────────────────
struct App {
    input: String,
    items: Vec<Item>,
    filtered: Vec<(Item, i64)>,
    selected: usize,
    matcher: SkimMatcherV2,
    mode: Mode,
    stats: Stats,
    #[allow(dead_code)]
    prompt: String,
}

impl App {
    fn new() -> Self {
        let stats = Stats::load();
        let mut app = Self {
            input: String::new(),
            items: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            matcher: SkimMatcherV2::default(),
            mode: Mode::Apps,
            stats,
            prompt: String::new(),
        };
        app.load_items();
        app.filter();
        app
    }

    fn new_stdin(lines: Vec<String>, prompt: String) -> Self {
        let items: Vec<Item> = lines.into_iter().map(|t| Item::Stdin { text: t }).collect();
        let filtered = items.iter().map(|i| (i.clone(), 100)).collect();
        let mut app = Self {
            input: String::new(),
            items: items.clone(),
            filtered,
            selected: 0,
            matcher: SkimMatcherV2::default(),
            mode: Mode::Stdin,
            stats: Stats::default(),
            prompt,
        };
        app.filter();
        app
    }

    fn load_items(&mut self) {
        self.items.clear();
        match self.mode {
            Mode::Apps => {
                for dir in &["/usr/share/applications", "/usr/local/share/applications"] {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                                continue;
                            }
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let mut in_main = false;
                                let mut name = String::new();
                                let mut exec = String::new();
                                let mut no_display = false;
                                for line in content.lines() {
                                    let t = line.trim();
                                    if t == "[Desktop Entry]" {
                                        in_main = true;
                                    } else if t.starts_with('[') {
                                        in_main = false;
                                    } else if in_main {
                                        if t.starts_with("Name=") && name.is_empty() {
                                            name = t[5..].into();
                                        } else if t.starts_with("Exec=") && exec.is_empty() {
                                            exec = t[5..]
                                                .split_whitespace()
                                                .next()
                                                .unwrap_or("")
                                                .into();
                                        } else if t == "NoDisplay=true" {
                                            no_display = true;
                                        }
                                    }
                                }
                                if !name.is_empty() && !exec.is_empty() && !no_display {
                                    self.items.push(Item::App { name, exec });
                                }
                            }
                        }
                    }
                }
                // Add faelight tools from scripts dir
                let scripts = format!(
                    "{}/0-core/scripts",
                    std::env::var("HOME").unwrap_or_default()
                );
                if let Ok(entries) = std::fs::read_dir(&scripts) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("faelight")
                            || name == "doctor"
                            || name == "snap-now"
                            || name == "fg"
                        {
                            let path = entry.path().to_string_lossy().to_string();
                            self.items.push(Item::App { name, exec: path });
                        }
                    }
                }
                self.items.sort_by_key(|a| a.display().to_lowercase());
            }
            Mode::Actions => {
                let core = format!("{}/0-core", std::env::var("HOME").unwrap_or_default());
                let actions = vec![
                    ("Lock Core",    format!("sudo {}/scripts/core-protect lock && pkill faelight-bar; setsid -f faelight-bar >/dev/null 2>&1", core), true),
                    ("Unlock Core",  format!("sudo {}/scripts/core-protect unlock && pkill faelight-bar; setsid -f faelight-bar >/dev/null 2>&1", core), true),
                    ("Health Check", format!("{}/scripts/doctor", core), true),
                    ("Reload Bar",   "pkill faelight-bar; setsid -f faelight-bar >/dev/null 2>&1".into(), false),
                    ("Reload Niri",  "niri msg action reload-config".into(), false),
                    ("Snapshot Now", "snap-now 'manual'".into(), true),
                    ("Git Sync",     format!("{}/scripts/fg sync", core), true),
                ];
                for (label, cmd, terminal) in actions {
                    self.items.push(Item::Action {
                        label: label.into(),
                        cmd,
                        terminal,
                    });
                }
            }
            Mode::Scripts => {
                let scripts = format!(
                    "{}/0-core/scripts",
                    std::env::var("HOME").unwrap_or_default()
                );
                if let Ok(entries) = std::fs::read_dir(&scripts) {
                    let mut names: Vec<_> = entries
                        .flatten()
                        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                        .collect();
                    names.sort_by_key(|e| e.file_name());
                    for entry in names {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let path = entry.path().to_string_lossy().to_string();
                        self.items.push(Item::Script { name, path });
                    }
                }
            }
            Mode::Files => {
                let home = std::env::var("HOME").unwrap_or_default();
                let files = vec![
                    "~/.config/niri/config.kdl",
                    "~/0-core/README.md",
                    "~/0-core/CHANGELOG-v10.0.0.md",
                    "~/.config/zsh/aliases.zsh",
                    "~/.config/nvim/init.lua",
                ];
                for f in files {
                    self.items.push(Item::File {
                        path: f.replace('~', &home),
                    });
                }
            }
            Mode::Intents => {
                if let Ok(o) = Command::new("sh")
                    .arg("-c")
                    .arg("int list 2>/dev/null | grep -E '^(COMPLETE|PLANNED|IN_PROGRESS)' | head -30")
                    .output()
                {
                    if let Ok(s) = String::from_utf8(o.stdout) {
                        for line in s.lines() {
                            self.items.push(Item::Intent { text: line.into() });
                        }
                    }
                }
                if self.items.is_empty() {
                    self.items.push(Item::Intent {
                        text: "No intents found".into(),
                    });
                }
            }
            Mode::Stdin => {}
        }
    }

    fn filter(&mut self) {
        let raw = self.input.trim_start_matches(|c| ">$@#!".contains(c));
        if raw.is_empty() {
            self.filtered = self.items.iter().map(|i| (i.clone(), 100)).collect();
        } else {
            let mut scored: Vec<_> = self
                .items
                .iter()
                .filter_map(|i| {
                    self.matcher
                        .fuzzy_match(&i.display(), raw)
                        .map(|s| (i.clone(), s))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered = scored;
        }
        self.selected = 0;
    }

    fn switch_mode(&mut self) {
        let new = match self.input.chars().next() {
            Some('>') => Mode::Actions,
            Some('$') => Mode::Scripts,
            Some('@') => Mode::Files,
            Some('#') => Mode::Intents,
            _ => return,
        };
        if new != self.mode {
            self.mode = new;
            self.input = self.input[1..].into();
            self.load_items();
            self.filter();
        }
    }

    fn next(&mut self) {
        if self.selected < self.filtered.len().saturating_sub(1) {
            self.selected += 1;
        }
    }
    fn prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn lock_core(&self) {
        let core = format!("{}/0-core", std::env::var("HOME").unwrap_or_default());
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("sudo {}/scripts/core-protect lock && pkill faelight-bar; setsid -f faelight-bar >/dev/null 2>&1", core))
            .spawn();
    }

    fn unlock_core(&self) {
        let core = format!("{}/0-core", std::env::var("HOME").unwrap_or_default());
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("sudo {}/scripts/core-protect unlock && pkill faelight-bar; setsid -f faelight-bar >/dev/null 2>&1", core))
            .spawn();
    }
}

// ── main ──────────────────────────────────────────────────
fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let dmenu = args.contains(&"--dmenu".to_string());
    let stdin_data = !atty::is(atty::Stream::Stdin);

    if dmenu || stdin_data {
        let lines: Vec<String> = io::stdin().lock().lines().map_while(Result::ok).collect();
        if lines.is_empty() {
            eprintln!("No input");
            return Ok(());
        }
        let prompt = args
            .windows(2)
            .find(|w| w[0] == "-p")
            .map(|w| w[1].clone())
            .unwrap_or("Select:".into());
        run(App::new_stdin(lines, prompt))
    } else {
        run(App::new())
    }
}

fn run(mut app: App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| draw(f, &app))?;
        if let Event::Key(key) = event::read()? {
            match (key.modifiers, key.code) {
                (_, KeyCode::Esc) => break,
                (_, KeyCode::F(1)) => {
                    app.lock_core();
                    break;
                }
                (_, KeyCode::F(2)) => {
                    app.unlock_core();
                    break;
                }
                (_, KeyCode::Enter) => {
                    if !app.filtered.is_empty() {
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        let wait = app.filtered[app.selected].0.execute();
                        if wait {
                            println!("\nPress Enter to continue...");
                            let _ = io::stdin().read_line(&mut String::new());
                        }
                        return Ok(());
                    }
                }
                (_, KeyCode::Down) => app.next(),
                (_, KeyCode::Up) => app.prev(),
                (KeyModifiers::NONE, KeyCode::Char(c)) => {
                    app.input.push(c);
                    app.switch_mode();
                    app.filter();
                }
                (_, KeyCode::Backspace) => {
                    app.input.pop();
                    app.filter();
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

// ── drawing ───────────────────────────────────────────────
fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    // outer vertical split: header + body
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // body horizontal split: list | stats
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(rows[1]);

    draw_header(f, app, rows[0]);
    draw_list(f, app, cols[0]);
    draw_stats(f, app, cols[1]);
}

fn draw_header(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mode_label = match app.mode {
        Mode::Apps => "  APPS",
        Mode::Actions => "⚡ ACTIONS",
        Mode::Scripts => "$  SCRIPTS",
        Mode::Files => "   FILES",
        Mode::Intents => "   INTENTS",
        Mode::Stdin => "   SELECT",
    };
    let hint = "  >actions  $scripts  @files  #intents  F1=lock  F2=unlock  Esc=close";
    let text = if app.input.is_empty() {
        format!("{}{}", mode_label, hint)
    } else {
        format!("{}  › {}", mode_label, app.input)
    };
    let p = Paragraph::new(text)
        .style(Style::default().fg(GREEN))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM))
                .title(Span::styled(
                    " 🌲 Faelight Palette ",
                    Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
                )),
        );
    f.render_widget(p, area);
}

fn draw_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let height = area.height.saturating_sub(2) as usize;
    let offset = if app.selected < height / 2 {
        0
    } else {
        app.selected.saturating_sub(height / 2)
    };

    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .skip(offset)
        .take(height)
        .enumerate()
        .map(|(i, (item, _))| {
            let idx = i + offset;
            let style = if idx == app.selected {
                Style::default()
                    .fg(BG)
                    .bg(GREEN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(FG)
            };
            ListItem::new(item.display()).style(style)
        })
        .collect();

    let count = app.filtered.len();
    let title = format!(" {} items ", count);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM))
                .title(Span::styled(title, Style::default().fg(DIM))),
        )
        .style(Style::default().bg(BG));
    f.render_widget(list, area);
}

fn draw_stats(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let s = &app.stats;

    // health bar
    let filled = (s.health_pct as usize * 10 / 100).min(10);
    let bar: String = "█".repeat(filled) + &"░".repeat(10 - filled);
    let health_color = if s.health_pct >= 90 {
        GREEN
    } else if s.health_pct >= 70 {
        WARNING
    } else {
        RED
    };
    let core_color = if s.core.contains("locked") {
        GREEN
    } else {
        WARNING
    };

    let mut lines: Vec<Line> = vec![
        Line::from(vec![Span::styled(
            " 0-CORE",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Health  ", Style::default().fg(DIM)),
            Span::styled(&bar, Style::default().fg(health_color)),
            Span::styled(
                format!("  {}", s.health),
                Style::default()
                    .fg(health_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  LOC     ", Style::default().fg(DIM)),
            Span::styled(&s.loc, Style::default().fg(FG)),
        ]),
        Line::from(vec![
            Span::styled("  Tools   ", Style::default().fg(DIM)),
            Span::styled(&s.tools, Style::default().fg(FG)),
        ]),
        Line::from(vec![
            Span::styled("  Commits ", Style::default().fg(DIM)),
            Span::styled(&s.commits, Style::default().fg(FG)),
        ]),
        Line::from(vec![
            Span::styled("  Git     ", Style::default().fg(DIM)),
            Span::styled(&s.git, Style::default().fg(FG)),
        ]),
        Line::from(vec![
            Span::styled("  Core    ", Style::default().fg(DIM)),
            Span::styled(&s.core, Style::default().fg(core_color)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  TOP TOOLS",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )]),
    ];

    for (name, loc) in &s.top_tools {
        let short = if name.len() > 18 { &name[..18] } else { name };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<18}", short), Style::default().fg(FG)),
            Span::styled(format!("{:>5}", loc), Style::default().fg(DIM)),
        ]));
    }

    let p = Paragraph::new(lines).style(Style::default().bg(BG)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(DIM))
            .title(Span::styled(" stats ", Style::default().fg(DIM))),
    );
    f.render_widget(p, area);
}
