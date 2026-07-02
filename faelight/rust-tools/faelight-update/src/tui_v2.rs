//! Ratatui TUI - FM/Palette aesthetic
//! Dual-pane layout with exact color scheme

use crate::UpdateCategory;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

// EXACT colors from faelight-fm
const BG_DARK: Color = Color::Rgb(17, 20, 15);
const BG_SELECTED: Color = Color::Rgb(45, 52, 38);
const ACCENT_GREEN: Color = Color::Rgb(163, 227, 107);
const ACCENT_BLUE: Color = Color::Rgb(107, 163, 227);
const TEXT_BRIGHT: Color = Color::Rgb(218, 224, 215);
const TEXT_DIM: Color = Color::Rgb(119, 127, 111);

#[derive(Clone)]
struct PackageItem {
    name: String,
    current: String,
    new: String,
    selected: bool,
    critical: bool,
}

struct UpdateTUI {
    categories: Vec<CategoryState>,
    active_category: usize,
    active_package: usize,
    total_selected: usize,
}

struct CategoryState {
    name: String,
    emoji: String,
    packages: Vec<PackageItem>,
    count: usize,
}

impl UpdateTUI {
    fn new(categories: &[UpdateCategory]) -> Self {
        let mut cat_states = Vec::new();
        let mut total_selected = 0;

        for cat in categories {
            let packages: Vec<PackageItem> = cat
                .items
                .iter()
                .map(|item| {
                    let critical = is_critical(&item.name);
                    PackageItem {
                        name: item.name.clone(),
                        current: item.current.clone(),
                        new: item.new.clone(),
                        selected: true,
                        critical,
                    }
                })
                .collect();

            total_selected += packages.len();

            cat_states.push(CategoryState {
                name: cat.name.clone(),
                emoji: cat.emoji.clone(),
                packages,
                count: cat.count,
            });
        }

        Self {
            categories: cat_states,
            active_category: 0,
            active_package: 0,
            total_selected,
        }
    }

    fn next_category(&mut self) {
        if self.active_category < self.categories.len().saturating_sub(1) {
            self.active_category += 1;
            self.active_package = 0;
        }
    }

    fn prev_category(&mut self) {
        if self.active_category > 0 {
            self.active_category -= 1;
            self.active_package = 0;
        }
    }

    fn next_package(&mut self) {
        if let Some(cat) = self.categories.get(self.active_category) {
            if self.active_package < cat.packages.len().saturating_sub(1) {
                self.active_package += 1;
            }
        }
    }

    fn prev_package(&mut self) {
        if self.active_package > 0 {
            self.active_package -= 1;
        }
    }

    fn toggle_package(&mut self) {
        if let Some(cat) = self.categories.get_mut(self.active_category) {
            if let Some(pkg) = cat.packages.get_mut(self.active_package) {
                pkg.selected = !pkg.selected;
                if pkg.selected {
                    self.total_selected += 1;
                } else {
                    self.total_selected = self.total_selected.saturating_sub(1);
                }
            }
        }
    }

    fn get_selections(&self) -> Vec<(String, Vec<String>)> {
        let mut selections = Vec::new();

        for cat in &self.categories {
            let selected: Vec<String> = cat
                .packages
                .iter()
                .filter(|p| p.selected)
                .map(|p| p.name.clone())
                .collect();

            if !selected.is_empty() {
                selections.push((cat.name.clone(), selected));
            }
        }

        selections
    }
}

fn is_critical(name: &str) -> bool {
    matches!(
        name,
        "systemd" | "systemd-libs" | "systemd-sysvcompat" | "linux" | "linux-headers"
    )
}

pub fn run_interactive_tui(
    categories: &[UpdateCategory],
) -> io::Result<Vec<(String, Vec<String>)>> {
    // Filter out empty categories
    let non_empty: Vec<UpdateCategory> =
        categories.iter().filter(|c| c.count > 0).cloned().collect();

    if non_empty.is_empty() {
        println!("No updates available");
        return Ok(Vec::new());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = UpdateTUI::new(&non_empty);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(confirmed) => {
            if confirmed {
                Ok(app.get_selections())
            } else {
                Ok(Vec::new())
            }
        }
        Err(e) => Err(e),
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut UpdateTUI,
) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                    KeyCode::Char('j') | KeyCode::Down => app.next_package(),
                    KeyCode::Char('k') | KeyCode::Up => app.prev_package(),
                    KeyCode::Tab => app.next_category(),
                    KeyCode::BackTab => app.prev_category(),
                    KeyCode::Char('h') | KeyCode::Left => app.prev_category(),
                    KeyCode::Char('l') | KeyCode::Right => app.next_category(),
                    KeyCode::Char(' ') => app.toggle_package(),
                    KeyCode::Enter => return Ok(true),
                    KeyCode::Char('a') => {
                        // Toggle all in current category
                        if let Some(cat) = app.categories.get_mut(app.active_category) {
                            let all_selected = cat.packages.iter().all(|p| p.selected);
                            for pkg in &mut cat.packages {
                                if all_selected {
                                    if pkg.selected {
                                        app.total_selected = app.total_selected.saturating_sub(1);
                                    }
                                    pkg.selected = false;
                                } else {
                                    if !pkg.selected {
                                        app.total_selected += 1;
                                    }
                                    pkg.selected = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &UpdateTUI) {
    // Dual-pane layout (like FM)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left: Categories
            Constraint::Percentage(70), // Right: Packages
        ])
        .split(f.area());

    // Left pane: Categories
    render_categories(f, app, main_chunks[0]);

    // Right pane: Package details
    render_packages(f, app, main_chunks[1]);

    // Bottom status bar
    render_status_bar(f, app, f.area());
}

fn render_categories(f: &mut Frame, app: &UpdateTUI, area: Rect) {
    let items: Vec<ListItem> = app
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let selected = i == app.active_category;
            let style = if selected {
                Style::default()
                    .fg(ACCENT_BLUE)
                    .bg(BG_SELECTED)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(TEXT_BRIGHT)
            };

            let text = if cat.count > 0 {
                format!("  {} {} ({})", cat.emoji, cat.name, cat.count)
            } else {
                format!("  {} {}", cat.emoji, cat.name)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📦 Categories (Tab/h/l to switch)")
                .style(Style::default().bg(BG_DARK)),
        )
        .style(Style::default().bg(BG_DARK));

    f.render_widget(list, area);
}

fn render_packages(f: &mut Frame, app: &UpdateTUI, area: Rect) {
    if let Some(cat) = app.categories.get(app.active_category) {
        if cat.packages.is_empty() {
            let empty = Paragraph::new("No updates available")
                .style(Style::default().fg(TEXT_DIM).bg(BG_DARK))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{} {}", cat.emoji, cat.name))
                        .style(Style::default().bg(BG_DARK)),
                );
            f.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = cat
            .packages
            .iter()
            .enumerate()
            .map(|(i, pkg)| {
                let selected = i == app.active_package;
                let bg = if selected { BG_SELECTED } else { BG_DARK };

                let checkbox = if pkg.selected { "✓" } else { " " };
                let checkbox_color = if pkg.selected { ACCENT_GREEN } else { TEXT_DIM };

                let name_color = if pkg.critical {
                    Color::Rgb(200, 100, 100) // Red for critical
                } else {
                    TEXT_BRIGHT
                };

                let critical_marker = if pkg.critical { " [CRITICAL]" } else { "" };

                let line = Line::from(vec![
                    Span::styled(
                        format!("  {} ", checkbox),
                        Style::default().fg(checkbox_color).bg(bg),
                    ),
                    Span::styled(
                        format!("{:<30}", pkg.name),
                        Style::default().fg(name_color).bg(bg),
                    ),
                    Span::styled(
                        pkg.current.to_string(),
                        Style::default().fg(Color::Rgb(200, 100, 100)).bg(bg),
                    ),
                    Span::styled(" → ", Style::default().fg(TEXT_DIM).bg(bg)),
                    Span::styled(
                        pkg.new.to_string(),
                        Style::default().fg(ACCENT_GREEN).bg(bg),
                    ),
                    Span::styled(
                        critical_marker,
                        Style::default().fg(Color::Rgb(200, 100, 100)).bg(bg),
                    ),
                ]);

                ListItem::new(line).style(Style::default().bg(bg))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "{} {} • j/k=nav • Space=toggle • a=all",
                        cat.emoji, cat.name
                    ))
                    .style(Style::default().bg(BG_DARK)),
            )
            .style(Style::default().bg(BG_DARK));

        f.render_widget(list, area);
    }
}

fn render_status_bar(f: &mut Frame, app: &UpdateTUI, area: Rect) {
    let status_area = Rect {
        x: area.x,
        y: area.y + area.height - 3,
        width: area.width,
        height: 3,
    };

    let status_text = format!(
        " {} packages selected • j/k=navigate • Space=toggle • Enter=update • q=cancel ",
        app.total_selected
    );

    let status = Paragraph::new(status_text)
        .style(
            Style::default()
                .fg(TEXT_BRIGHT)
                .bg(BG_DARK)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(status, status_area);
}
