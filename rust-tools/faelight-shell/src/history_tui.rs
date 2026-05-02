#![allow(clippy::all)]
// INT-250: native Ctrl+R history search TUI for faelight-shell.
// ratatui + crossterm based. Searches across full state.db shell_history.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use rusqlite::Connection;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct HistoryEntry {
    command: String,
    timestamp: i64,
    cwd: Option<String>,
    exit_code: Option<i32>,
    duration_ms: Option<i64>,
}

pub fn run_history_search(initial_query: &str) -> Option<String> {
    let db_path = match resolve_db_path() {
        Some(p) => p,
        None => return None,
    };
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    enable_raw_mode().ok()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).ok()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).ok()?;

    let result = run_loop(&mut terminal, &conn, initial_query);

    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = disable_raw_mode();

    result
}

fn resolve_db_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join("0-core/runtime/state.db"))
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    conn: &Connection,
    initial_query: &str,
) -> Option<String> {
    let mut query = String::from(initial_query);
    let mut entries = search(conn, &query);
    let mut list_state = ListState::default();
    if !entries.is_empty() {
        list_state.select(Some(0));
    }

    loop {
        let _ = terminal.draw(|f| draw_ui(f, &query, &entries, &mut list_state));

        if let Ok(event) = event::read() {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event
            {
                match (code, modifiers) {
                    (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return None,
                    (KeyCode::Enter, _) => {
                        if let Some(idx) = list_state.selected() {
                            return entries.get(idx).map(|e| e.command.clone());
                        }
                        return None;
                    }
                    (KeyCode::Up, _) => {
                        let i = list_state.selected().unwrap_or(0);
                        if i > 0 {
                            list_state.select(Some(i - 1));
                        }
                    }
                    (KeyCode::Down, _) => {
                        let i = list_state.selected().unwrap_or(0);
                        if i + 1 < entries.len() {
                            list_state.select(Some(i + 1));
                        }
                    }
                    (KeyCode::Backspace, _) => {
                        query.pop();
                        entries = search(conn, &query);
                        list_state.select(if entries.is_empty() { None } else { Some(0) });
                    }
                    (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => {
                        query.push(c);
                        entries = search(conn, &query);
                        list_state.select(if entries.is_empty() { None } else { Some(0) });
                    }
                    _ => {}
                }
            }
        }
    }
}

fn search(conn: &Connection, query: &str) -> Vec<HistoryEntry> {
    let sql = if query.is_empty() {
        "SELECT command, timestamp, cwd, exit_code, duration_ms
         FROM shell_history
         ORDER BY timestamp DESC
         LIMIT 200"
            .to_string()
    } else {
        "SELECT command, timestamp, cwd, exit_code, duration_ms
         FROM shell_history
         WHERE command LIKE ?1
         ORDER BY timestamp DESC
         LIMIT 200"
            .to_string()
    };
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let pattern = format!("%{}%", query);
    let row_mapper = |r: &rusqlite::Row| {
        Ok(HistoryEntry {
            command: r.get::<_, String>(0)?,
            timestamp: r.get::<_, i64>(1)?,
            cwd: r.get::<_, Option<String>>(2)?,
            exit_code: r.get::<_, Option<i32>>(3)?,
            duration_ms: r.get::<_, Option<i64>>(4)?,
        })
    };
    let rows = if query.is_empty() {
        stmt.query_map([], row_mapper)
    } else {
        stmt.query_map([&pattern], row_mapper)
    };
    rows.map(|r| r.filter_map(|x| x.ok()).collect())
        .unwrap_or_default()
}

fn draw_ui(
    f: &mut ratatui::Frame,
    query: &str,
    entries: &[HistoryEntry],
    list_state: &mut ListState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let search_box = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "search: ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{}_", query), Style::default().fg(Color::White)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(
                " 🌲 history ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    f.render_widget(search_box, chunks[0]);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let items: Vec<ListItem> = entries
        .iter()
        .map(|e| {
            let age = format_age(now - e.timestamp);
            let exit_marker = match e.exit_code {
                Some(0) => Span::styled("✓", Style::default().fg(Color::Green)),
                Some(_) => Span::styled("✗", Style::default().fg(Color::Red)),
                None => Span::raw(" "),
            };
            let dur = match e.duration_ms {
                Some(d) if d < 1000 => format!("{}ms", d),
                Some(d) => format!("{}.{}s", d / 1000, (d % 1000) / 100),
                None => String::new(),
            };
            let cwd_short = e.cwd.as_ref().map(|c| shorten_cwd(c)).unwrap_or_default();
            let line = Line::from(vec![
                Span::raw(format!("{:50}  ", truncate(&e.command, 50))),
                Span::styled(
                    format!("{:>10}  ", age),
                    Style::default().fg(Color::DarkGray),
                ),
                exit_marker,
                Span::raw(format!(" {:>8}  ", dur)),
                Span::styled(cwd_short, Style::default().fg(Color::Blue)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(Span::styled(
                    " results ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 80, 40))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▶ ");
    f.render_stateful_widget(list, chunks[1], list_state);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": run  "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": cancel  "),
        Span::styled(
            "↑↓",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": navigate"),
    ]))
    .style(Style::default().fg(Color::DarkGray))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, chunks[2]);
}

fn format_age(seconds_ago: i64) -> String {
    if seconds_ago < 60 {
        format!("{}s ago", seconds_ago)
    } else if seconds_ago < 3600 {
        format!("{}m ago", seconds_ago / 60)
    } else if seconds_ago < 86400 {
        format!("{}h ago", seconds_ago / 3600)
    } else {
        format!("{}d ago", seconds_ago / 86400)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max - 1).collect();
        format!("{}…", truncated)
    }
}

fn shorten_cwd(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    if path.starts_with(&home) {
        format!("~{}", &path[home.len()..])
    } else {
        path.to_string()
    }
}
