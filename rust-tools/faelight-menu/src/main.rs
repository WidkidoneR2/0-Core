//! faelight-menu v4.0.0 - Power Menu
//! 🌲 Faelight Forest — Palette aesthetic

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{io, process::Command};

// ── Faelight Forest palette ──────────────────────────────────────────────────
const BG: Color = Color::Rgb(17, 20, 15);
const FG: Color = Color::Rgb(218, 224, 215);
const GREEN: Color = Color::Rgb(163, 227, 107);
const DIM: Color = Color::Rgb(90, 100, 80);
const ACCENT: Color = Color::Rgb(120, 190, 80);
const WARNING: Color = Color::Rgb(230, 180, 60);
const RED: Color = Color::Rgb(220, 80, 80);

#[derive(Debug, Clone, PartialEq)]
enum MenuItem {
    Lock,
    Logout,
    Suspend,
    Reboot,
    Shutdown,
}

impl MenuItem {
    fn icon(&self) -> &str {
        match self {
            MenuItem::Lock => "󰍁",
            MenuItem::Logout => "󰍃",
            MenuItem::Suspend => "󰒲",
            MenuItem::Reboot => "󰑓",
            MenuItem::Shutdown => "󰐥",
        }
    }

    fn label(&self) -> &str {
        match self {
            MenuItem::Lock => "Lock Screen",
            MenuItem::Logout => "Logout",
            MenuItem::Suspend => "Suspend",
            MenuItem::Reboot => "Reboot",
            MenuItem::Shutdown => "Shutdown",
        }
    }

    fn hint(&self) -> &str {
        match self {
            MenuItem::Lock => "faelight-lock",
            MenuItem::Logout => "niri msg action quit",
            MenuItem::Suspend => "systemctl suspend",
            MenuItem::Reboot => "systemctl reboot",
            MenuItem::Shutdown => "systemctl poweroff",
        }
    }

    fn is_dangerous(&self) -> bool {
        matches!(self, MenuItem::Reboot | MenuItem::Shutdown)
    }

    fn color(&self) -> Color {
        match self {
            MenuItem::Reboot | MenuItem::Shutdown => RED,
            MenuItem::Logout => WARNING,
            _ => FG,
        }
    }

    fn execute(&self) {
        match self {
            MenuItem::Lock => {
                let _ = Command::new("faelight-lock").spawn();
            }
            MenuItem::Logout => {
                let _ = Command::new("niri").args(["msg", "action", "quit"]).spawn();
            }
            MenuItem::Suspend => {
                let _ = Command::new("systemctl").arg("suspend").status();
            }
            MenuItem::Reboot => {
                let _ = Command::new("systemctl").arg("reboot").status();
            }
            MenuItem::Shutdown => {
                let _ = Command::new("systemctl").arg("poweroff").status();
            }
        }
    }
}

const ITEMS: &[MenuItem] = &[
    MenuItem::Lock,
    MenuItem::Logout,
    MenuItem::Suspend,
    MenuItem::Reboot,
    MenuItem::Shutdown,
];

struct App {
    selected: usize,
    confirm: Option<usize>, // index awaiting confirmation
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            selected: 0,
            confirm: None,
            should_quit: false,
        }
    }

    fn next(&mut self) {
        self.confirm = None;
        if self.selected < ITEMS.len() - 1 {
            self.selected += 1;
        }
    }

    fn previous(&mut self) {
        self.confirm = None;
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn select(&mut self) {
        let item = &ITEMS[self.selected];
        if item.is_dangerous() {
            if self.confirm == Some(self.selected) {
                item.execute();
                self.should_quit = true;
            } else {
                self.confirm = Some(self.selected);
            }
        } else {
            item.execute();
            self.should_quit = true;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Enter | KeyCode::Char(' ') => app.select(),
                KeyCode::Char('l') => {
                    MenuItem::Lock.execute();
                    app.should_quit = true;
                }
                KeyCode::Char('e') => {
                    MenuItem::Logout.execute();
                    app.should_quit = true;
                }
                KeyCode::Char('s') => {
                    MenuItem::Suspend.execute();
                    app.should_quit = true;
                }
                KeyCode::Char('r') => {
                    if app.confirm == Some(3) {
                        MenuItem::Reboot.execute();
                        app.should_quit = true;
                    } else {
                        app.confirm = Some(3);
                        app.selected = 3;
                    }
                }
                KeyCode::Char('p') => {
                    if app.confirm == Some(4) {
                        MenuItem::Shutdown.execute();
                        app.should_quit = true;
                    } else {
                        app.confirm = Some(4);
                        app.selected = 4;
                    }
                }
                _ => {
                    app.confirm = None;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    // Full background
    f.render_widget(Block::default().style(Style::default().bg(BG)), f.area());

    let area = f.area();

    // Menu box
    let outer = Block::default()
        .title(Line::from(vec![
            Span::styled(" 🌲 ", Style::default().fg(GREEN)),
            Span::styled(
                "POWER MENU",
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default()),
        ]))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(BG));

    let inner = outer.inner(area);
    f.render_widget(outer, area);

    // Layout: items + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    // Items
    let items: Vec<ListItem> = ITEMS
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.selected;
            let is_confirm = app.confirm == Some(i);

            let (icon_style, label_style, hint_style) = if is_confirm {
                (
                    Style::default()
                        .fg(BG)
                        .bg(WARNING)
                        .add_modifier(Modifier::BOLD),
                    Style::default()
                        .fg(BG)
                        .bg(WARNING)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(BG).bg(WARNING),
                )
            } else if is_selected {
                (
                    Style::default()
                        .fg(BG)
                        .bg(GREEN)
                        .add_modifier(Modifier::BOLD),
                    Style::default()
                        .fg(BG)
                        .bg(GREEN)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(BG).bg(GREEN),
                )
            } else {
                (
                    Style::default().fg(item.color()),
                    Style::default().fg(item.color()),
                    Style::default().fg(DIM),
                )
            };

            let confirm_tag = if is_confirm { " ⚠ confirm?" } else { "" };

            let line = Line::from(vec![
                Span::styled(format!("  {} ", item.icon()), icon_style),
                Span::styled(format!(" {:<14}", item.label()), label_style),
                Span::styled(format!("  {}{}", item.hint(), confirm_tag), hint_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(BG));

    f.render_widget(list, chunks[0]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("l", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("ock  ", Style::default().fg(DIM)),
        Span::styled("s", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("uspend  ", Style::default().fg(DIM)),
        Span::styled(
            "r",
            Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
        ),
        Span::styled("eboot  ", Style::default().fg(DIM)),
        Span::styled("p", Style::default().fg(RED).add_modifier(Modifier::BOLD)),
        Span::styled("ower  ", Style::default().fg(DIM)),
        Span::styled("q", Style::default().fg(DIM).add_modifier(Modifier::BOLD)),
        Span::styled("uit", Style::default().fg(DIM)),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(footer, chunks[1]);
}
