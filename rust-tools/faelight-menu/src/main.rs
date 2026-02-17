//! faelight-menu v3.0.0 - Power Menu + Apps + Health
//! 🌲 Faelight Forest - Matches palette/FM aesthetic

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{io, process::Command};

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Power,
    Apps,
    Health,
}

#[derive(Debug, Clone, PartialEq)]
enum MenuItem {
    // Power items
    Lock,
    Logout,
    Suspend,
    Reboot,
    Shutdown,
    // Apps
    App(String, String), // (name, exec)
    // Health
    HealthInfo(String),
}

impl MenuItem {
    fn display(&self) -> String {
        match self {
            MenuItem::Lock => "🔒 Lock Screen".to_string(),
            MenuItem::Logout => "🚪 Logout".to_string(),
            MenuItem::Suspend => "💤 Suspend".to_string(),
            MenuItem::Reboot => "🔄 Reboot".to_string(),
            MenuItem::Shutdown => "⚠️  Shutdown".to_string(),
            MenuItem::App(name, _) => format!("🚀 {}", name),
            MenuItem::HealthInfo(info) => info.clone(),
        }
    }

    fn is_dangerous(&self) -> bool {
        matches!(self, MenuItem::Reboot | MenuItem::Shutdown)
    }

    fn execute(&self) {
        match self {
            MenuItem::Lock => {
                let _ = Command::new("swaylock").spawn();
            }
            MenuItem::Logout => {
                let _ = Command::new("swaymsg").arg("exit").spawn();
            }
            MenuItem::Suspend => {
                let _ = Command::new("systemctl").arg("suspend").spawn();
            }
            MenuItem::Reboot => {
                let _ = Command::new("systemctl").arg("reboot").spawn();
            }
            MenuItem::Shutdown => {
                let _ = Command::new("systemctl").arg("poweroff").spawn();
            }
            MenuItem::App(_, exec) => {
                let parts: Vec<&str> = exec.split_whitespace().collect();
                if let Some((cmd, args)) = parts.split_first() {
                    let _ = Command::new("setsid").arg("-f").arg(cmd).args(args).spawn();
                }
            }
            MenuItem::HealthInfo(_) => {
                // Not executable
            }
        }
    }
}

struct App {
    mode: Mode,
    items: Vec<MenuItem>,
    selected: usize,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            mode: Mode::Power,
            items: Vec::new(),
            selected: 0,
            should_quit: false,
        };
        app.load_items();
        app
    }

    fn load_items(&mut self) {
        self.items.clear();
        self.selected = 0;

        match self.mode {
            Mode::Power => {
                self.items = vec![
                    MenuItem::Lock,
                    MenuItem::Logout,
                    MenuItem::Suspend,
                    MenuItem::Reboot,
                    MenuItem::Shutdown,
                ];
            }
            Mode::Apps => {
                // Parse .desktop files
                if let Ok(entries) = std::fs::read_dir("/usr/share/applications") {
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
                                        } else if trimmed.starts_with("Exec=") && exec.is_empty() {
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
                                    self.items.push(MenuItem::App(name, exec));
                                }
                            }
                        }
                    }
                }
                // Sort alphabetically
                self.items.sort_by_key(|a| a.display().to_lowercase());
            }
            Mode::Health => {
                // Quick health check
                if let Ok(output) = Command::new("/home/christian/.local/bin/doctor").output() {
                    if let Ok(result) = String::from_utf8(output.stdout) {
                        // Parse last line for health percentage
                        for line in result.lines() {
                            if line.contains("Health:")
                                || line.starts_with('✅')
                                || line.starts_with('❌')
                                || line.starts_with('⚠')
                            {
                                self.items
                                    .push(MenuItem::HealthInfo(line.trim().to_string()));
                            }
                        }
                    }
                }
                if self.items.is_empty() {
                    self.items
                        .push(MenuItem::HealthInfo("Running health check...".to_string()));
                }
            }
        }
    }

    fn next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn execute_selected(&mut self) {
        if let Some(item) = self.items.get(self.selected) {
            item.execute();
            self.should_quit = true;
        }
    }

    fn switch_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.load_items();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.should_quit = true;
                }
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Enter => app.execute_selected(),

                // Mode switching
                KeyCode::Char('>') => app.switch_mode(Mode::Apps),
                KeyCode::Char('!') => app.switch_mode(Mode::Health),
                KeyCode::Backspace => app.switch_mode(Mode::Power),

                // Quick shortcuts (Power mode only)
                KeyCode::Char('l') if app.mode == Mode::Power => {
                    MenuItem::Lock.execute();
                    app.should_quit = true;
                }
                KeyCode::Char('e') if app.mode == Mode::Power => {
                    MenuItem::Logout.execute();
                    app.should_quit = true;
                }
                KeyCode::Char('s') if app.mode == Mode::Power => {
                    MenuItem::Suspend.execute();
                    app.should_quit = true;
                }
                KeyCode::Char('r') if app.mode == Mode::Power => {
                    MenuItem::Reboot.execute();
                    app.should_quit = true;
                }
                KeyCode::Char('p') if app.mode == Mode::Power => {
                    MenuItem::Shutdown.execute();
                    app.should_quit = true;
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    // Adjust size based on mode
    let (width, height) = match app.mode {
        Mode::Power => (98, 95),
        Mode::Apps => (40, 60),
        Mode::Health => (60, 40),
    };

    let area = centered_rect(width, height, f.area());

    // Title based on mode
    let title = match app.mode {
        Mode::Power => "⚡ POWER MENU",
        Mode::Apps => "🚀 LAUNCH APPS",
        Mode::Health => "🏥 SYSTEM HEALTH",
    };

    // Split area into list and help sections
    let _chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // List items
            Constraint::Length(2), // Help text (2 lines)
        ])
        .split(area);

    // Create menu items
    let items: Vec<ListItem> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else if item.is_dangerous() {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            ListItem::new(item.display()).style(style)
        })
        .collect();

    let menu = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    f.render_widget(menu, area);

    // Help text INSIDE at bottom
    let help_lines = match app.mode {
        Mode::Power => vec![
            "L:Lock  S:Suspend  R:Reboot  P:Power-off",
            ">:Apps  !:Health  Esc:Quit",
        ],
        Mode::Apps => vec!["Enter:Launch  Backspace:Back", ">:Apps  !:Health"],
        Mode::Health => vec!["Backspace:Back", ">:Apps  !:Health"],
    };

    // Position help at bottom INSIDE the border
    let help_area = Rect {
        x: area.x + 1,               // Inside border
        y: area.y + area.height - 3, // Bottom of area, above border
        width: area.width - 2,       // Account for borders
        height: 2,
    };

    let help_widget = Paragraph::new(help_lines.join(
        "
",
    ))
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Left);

    f.render_widget(help_widget, help_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
