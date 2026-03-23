//! faelight-intent v1.0.0
//! 🌲 Intent dashboard — the forest knows where it is going.

use anyhow::Result;
use chrono::Local;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, terminal,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::path::PathBuf;

const BG: Color = Color::Rgb(15, 20, 17);
const FG: Color = Color::Rgb(215, 224, 218);
const ACCENT: Color = Color::Rgb(163, 227, 107);
const DIM: Color = Color::Rgb(119, 143, 127);
const YELLOW: Color = Color::Rgb(227, 199, 107);
const BLUE: Color = Color::Rgb(107, 163, 227);
const CYAN: Color = Color::Rgb(107, 227, 210);

fn core_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home/christian"))
        .join("0-core")
}

// ─── DATA ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Intent {
    id: String,
    title: String,
    status: String,
    #[allow(dead_code)]
    tags: Vec<String>,
}

fn load_intents(root: &PathBuf, subdir: &str) -> Vec<Intent> {
    let dir = root.join("intents").join(subdir);
    if !dir.exists() {
        return vec![];
    }

    let mut intents = vec![];
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("cannot read"))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let id = entry
            .file_name()
            .to_string_lossy()
            .split('-')
            .next()
            .unwrap_or("?")
            .to_string();
        let title = content
            .lines()
            .find(|l| l.starts_with("title:"))
            .and_then(|l| l.split('"').nth(1))
            .unwrap_or("Untitled")
            .to_string();
        let status = content
            .lines()
            .find(|l| l.starts_with("status:"))
            .map(|l| l.replace("status:", "").trim().to_string())
            .unwrap_or_default();
        let tags = content
            .lines()
            .find(|l| l.starts_with("tags:"))
            .map(|l| {
                l.replace("tags:", "")
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .collect()
            })
            .unwrap_or_default();
        intents.push(Intent {
            id,
            title,
            status,
            tags,
        });
    }
    intents
}

fn load_focus(root: &PathBuf) -> Option<(String, String)> {
    let focus_path = root.join("runtime/focus.toml");
    if !focus_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&focus_path).ok()?;
    let id = content
        .lines()
        .find(|l| l.starts_with("intent_id"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())?;
    let since = content
        .lines()
        .find(|l| l.starts_with("started_at"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
        .unwrap_or_default();
    Some((id, since))
}

fn load_checkpoints(root: &PathBuf) -> Vec<(String, String)> {
    let dir = root.join("runtime/checkpoints");
    if !dir.exists() {
        return vec![];
    }
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("cannot read checkpoints"))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    entries
        .iter()
        .rev()
        .take(5)
        .map(|e| {
            let name = e
                .file_name()
                .to_string_lossy()
                .trim_end_matches(".toml")
                .to_string();
            let content = std::fs::read_to_string(e.path()).unwrap_or_default();
            let health = content
                .lines()
                .find(|l| l.contains("health"))
                .and_then(|l| l.split('=').nth(1))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "?".to_string());
            (name, health)
        })
        .collect()
}

fn status_color(status: &str) -> Color {
    match status {
        s if s.contains("in-progress") => ACCENT,
        s if s.contains("complete") => BLUE,
        s if s.contains("planned") => YELLOW,
        _ => DIM,
    }
}

fn status_icon(status: &str) -> &'static str {
    match status {
        s if s.contains("in-progress") => "🟡",
        s if s.contains("complete") => "✅",
        s if s.contains("planned") => "📋",
        _ => "·",
    }
}

// ─── DRAW ────────────────────────────────────────────────────────────────────

struct App {
    root: PathBuf,
    active: Vec<Intent>,
    planned: Vec<Intent>,
    focus: Option<(String, String)>,
    checkpoints: Vec<(String, String)>,
    #[allow(dead_code)]
    selected: usize,
}

impl App {
    fn new(root: PathBuf) -> Self {
        let active = load_intents(&root, "future");
        let planned = active
            .iter()
            .filter(|i| i.status.contains("planned"))
            .cloned()
            .collect();
        let active: Vec<Intent> = active
            .into_iter()
            .filter(|i| i.status.contains("in-progress"))
            .collect();
        let focus = load_focus(&root);
        let checkpoints = load_checkpoints(&root);
        Self {
            root,
            active,
            planned,
            focus,
            checkpoints,
            selected: 0,
        }
    }

    fn refresh(&mut self) {
        let all = load_intents(&self.root, "future");
        self.planned = all
            .iter()
            .filter(|i| i.status.contains("planned"))
            .cloned()
            .collect();
        self.active = all
            .into_iter()
            .filter(|i| i.status.contains("in-progress"))
            .collect();
        self.focus = load_focus(&self.root);
        self.checkpoints = load_checkpoints(&self.root);
    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        f.render_widget(Block::default().style(Style::default().bg(BG)), area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header
                Constraint::Length(5), // focus
                Constraint::Min(0),    // main content
                Constraint::Length(1), // footer
            ])
            .split(area);

        // ── HEADER ──────────────────────────────────────────────────────
        let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(22)])
            .split(chunks[0]);

        let left = Paragraph::new(Line::from(vec![
            Span::styled(" 🌲 ", Style::default().fg(ACCENT)),
            Span::styled(
                "Intent Dashboard",
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  — the forest knows where it is going",
                Style::default().fg(DIM),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(ACCENT)),
        );
        f.render_widget(left, header_chunks[0]);

        let right = Paragraph::new(Line::from(vec![Span::styled(
            format!("{}  ", now),
            Style::default().fg(DIM),
        )]))
        .alignment(ratatui::layout::Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(ACCENT)),
        );
        f.render_widget(right, header_chunks[1]);

        // ── FOCUS ────────────────────────────────────────────────────────
        let focus_content = if let Some((ref id, ref since)) = app.focus {
            let focused_intent = app
                .active
                .iter()
                .find(|i| i.id == *id)
                .map(|i| i.title.clone())
                .unwrap_or_else(|| format!("INT-{}", id));
            vec![
                Line::from(vec![
                    Span::styled("  🎯 ", Style::default().fg(ACCENT)),
                    Span::styled(format!("INT-{}  ", id), Style::default().fg(DIM)),
                    Span::styled(
                        focused_intent.clone(),
                        Style::default().fg(FG).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("     Since: ", Style::default().fg(DIM)),
                    Span::styled(since, Style::default().fg(CYAN)),
                ]),
            ]
        } else {
            vec![
                Line::from(vec![Span::styled(
                    "  No active focus",
                    Style::default().fg(DIM),
                )]),
                Line::from(vec![Span::styled(
                    "  Use: cistart <id> to begin an intent",
                    Style::default().fg(DIM),
                )]),
            ]
        };
        let focus_block = Paragraph::new(focus_content)
            .block(
                Block::default()
                    .title(Span::styled(
                        " 🎯 Current Focus ",
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(ACCENT)),
            )
            .style(Style::default().bg(BG));
        f.render_widget(focus_block, chunks[1]);

        // ── MAIN CONTENT ─────────────────────────────────────────────────
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(chunks[2]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(main_chunks[0]);

        // Active intents
        let active_items: Vec<ListItem> = if app.active.is_empty() {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "  No in-progress intents",
                Style::default().fg(DIM),
            )]))]
        } else {
            app.active
                .iter()
                .map(|i| {
                    ListItem::new(Line::from(vec![
                        Span::styled("  🟡 ", Style::default().fg(YELLOW)),
                        Span::styled(format!("INT-{}  ", i.id), Style::default().fg(DIM)),
                        Span::styled(
                            i.title.chars().take(40).collect::<String>(),
                            Style::default().fg(FG),
                        ),
                    ]))
                })
                .collect()
        };
        let active_list = List::new(active_items)
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" 🔥 In Progress ({}) ", app.active.len()),
                        Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(YELLOW)),
            )
            .style(Style::default().bg(BG));
        f.render_widget(active_list, left_chunks[0]);

        // Checkpoints
        let cp_items: Vec<ListItem> = if app.checkpoints.is_empty() {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "  No checkpoints yet",
                Style::default().fg(DIM),
            )]))]
        } else {
            app.checkpoints
                .iter()
                .map(|(name, health)| {
                    let short: String = name.chars().take(35).collect();
                    ListItem::new(Line::from(vec![
                        Span::styled("  📸 ", Style::default().fg(CYAN)),
                        Span::styled(short, Style::default().fg(FG)),
                        Span::styled(format!("  {}%", health), Style::default().fg(ACCENT)),
                    ]))
                })
                .collect()
        };
        let cp_list = List::new(cp_items)
            .block(
                Block::default()
                    .title(Span::styled(
                        " 📸 Recent Checkpoints ",
                        Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(CYAN)),
            )
            .style(Style::default().bg(BG));
        f.render_widget(cp_list, left_chunks[1]);

        // Planned intents
        let planned_items: Vec<ListItem> = app
            .planned
            .iter()
            .map(|i| {
                let color = status_color(&i.status);
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} ", status_icon(&i.status)), Style::default()),
                    Span::styled(format!("INT-{}  ", i.id), Style::default().fg(DIM)),
                    Span::styled(
                        i.title.chars().take(45).collect::<String>(),
                        Style::default().fg(color),
                    ),
                ]))
            })
            .collect();

        let planned_list = List::new(planned_items)
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" 📋 Planned ({}) ", app.planned.len()),
                        Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(BLUE)),
            )
            .style(Style::default().bg(BG));
        f.render_widget(planned_list, main_chunks[1]);

        // ── FOOTER ──────────────────────────────────────────────────────
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("  q", Style::default().fg(ACCENT)),
            Span::styled(" quit  ", Style::default().fg(DIM)),
            Span::styled("r", Style::default().fg(ACCENT)),
            Span::styled(" refresh  ", Style::default().fg(DIM)),
            Span::styled("cistart <id>", Style::default().fg(ACCENT)),
            Span::styled(" begin intent  ", Style::default().fg(DIM)),
            Span::styled("cpc <name>", Style::default().fg(ACCENT)),
            Span::styled(" checkpoint", Style::default().fg(DIM)),
        ]))
        .style(Style::default().fg(DIM).bg(BG));
        f.render_widget(footer, chunks[3]);
    })?;
    Ok(())
}

fn main() -> Result<()> {
    let root = core_root();
    let mut app = App::new(root);

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        draw(&mut terminal, &app)?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => app.refresh(),
                    _ => {}
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        cursor::Show
    )?;
    Ok(())
}
