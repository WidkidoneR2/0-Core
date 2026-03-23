//! faelight-pulse v1.0.0
//! 🌲 core pulse — The forest watching itself breathe.
//! Live event stream from runtime/state.db with domain filtering.

use anyhow::Result;
use chrono::{DateTime, Local};
use clap::Parser;
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
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Sparkline},
    Terminal,
};
use rusqlite::Connection;
use std::collections::VecDeque;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

// ─── THEME ──────────────────────────────────────────────────────────────────
const BG: Color = Color::Rgb(15, 20, 17);
const FG: Color = Color::Rgb(215, 224, 218);
const ACCENT: Color = Color::Rgb(163, 227, 107); // green
const DIM: Color = Color::Rgb(119, 143, 127);
const BLUE: Color = Color::Rgb(107, 163, 227);
const YELLOW: Color = Color::Rgb(227, 199, 107);
const RED: Color = Color::Rgb(227, 107, 107);
const PURPLE: Color = Color::Rgb(180, 107, 227);
const CYAN: Color = Color::Rgb(107, 227, 210);

fn domain_color(domain: &str) -> Color {
    match domain {
        "doctor" => ACCENT,
        "git" => BLUE,
        "security" => RED,
        "update" => YELLOW,
        "intent" => PURPLE,
        "checkpoint" => CYAN,
        _ => DIM,
    }
}

fn domain_icon(domain: &str) -> &'static str {
    match domain {
        "doctor" => "🏥",
        "git" => "🌿",
        "security" => "🛡️ ",
        "update" => "⬆️ ",
        "intent" => "🎯",
        "checkpoint" => "📸",
        _ => "·",
    }
}

// ─── CLI ────────────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(
    name = "faelight-pulse",
    about = "🌲 core pulse — the forest watching itself breathe",
    version = "1.0.0"
)]
struct Cli {
    /// Filter by domain (doctor, git, security, update, intent)
    #[arg(short, long)]
    domain: Option<String>,
    /// Output as plain JSON stream (no TUI)
    #[arg(long)]
    json: bool,
    /// Number of recent events to show on start
    #[arg(short, long, default_value = "50")]
    limit: usize,
}

// ─── EVENT ──────────────────────────────────────────────────────────────────
#[derive(Clone, Debug)]
struct PulseEvent {
    id: i64,
    domain: String,
    action: String,
    detail: String,
    timestamp: i64,
}

impl PulseEvent {
    fn time_str(&self) -> String {
        let dt = DateTime::from_timestamp(self.timestamp, 0)
            .map(|d| d.with_timezone(&Local))
            .map(|d| d.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "??:??:??".to_string());
        dt
    }

    fn detail_short(&self) -> String {
        // Parse payload JSON for readable summary
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.detail) {
            match self.domain.as_str() {
                "doctor" => {
                    let health = v["detail"]["health"].as_i64().unwrap_or(0);
                    let passed = v["detail"]["passed"].as_i64().unwrap_or(0);
                    let warn = v["detail"]["warnings"].as_i64().unwrap_or(0);
                    return format!(
                        "health {}%  {}/22 passed  {} warnings",
                        health, passed, warn
                    );
                }
                "security" => {
                    let crit = v["detail"]["critical"].as_i64().unwrap_or(0);
                    let high = v["detail"]["high"].as_i64().unwrap_or(0);
                    let med = v["detail"]["medium"].as_i64().unwrap_or(0);
                    return format!("critical:{}  high:{}  medium:{}", crit, high, med);
                }
                "git" => {
                    if let Some(s) = v["detail"].as_str() {
                        return s.chars().take(60).collect();
                    }
                }
                _ => {}
            }
            if let Some(s) = v["result"].as_str() {
                return s.to_string();
            }
        }
        self.detail.chars().take(60).collect()
    }
}

// ─── DB ─────────────────────────────────────────────────────────────────────
fn db_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/christian"));
    home.join("0-core/runtime/state.db")
}

fn load_events(
    conn: &Connection,
    since_id: i64,
    domain_filter: Option<&str>,
    limit: usize,
) -> Vec<PulseEvent> {
    let sql = if domain_filter.is_some() {
        "SELECT id, domain, action, COALESCE(payload,''), timestamp FROM events WHERE id > ?1 AND domain = ?2 ORDER BY id DESC LIMIT ?3"
    } else {
        "SELECT id, domain, action, COALESCE(payload,''), timestamp FROM events WHERE id > ?1 ORDER BY id DESC LIMIT ?3"
    };

    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<PulseEvent> {
        Ok(PulseEvent {
            id: row.get(0)?,
            domain: row.get(1)?,
            action: row.get(2)?,
            detail: row.get(3)?,
            timestamp: row.get(4)?,
        })
    };

    let rows = if let Some(d) = domain_filter {
        stmt.query_map(rusqlite::params![since_id, d, limit as i64], map_row)
    } else {
        stmt.query_map(rusqlite::params![since_id, limit as i64], map_row)
    };

    match rows {
        Ok(r) => r.filter_map(|e| e.ok()).collect(),
        Err(_) => vec![],
    }
}

fn load_health_sparkline(conn: &Connection) -> Vec<u64> {
    let mut stmt = match conn
        .prepare("SELECT payload FROM events WHERE domain='doctor' ORDER BY id DESC LIMIT 20")
    {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let vals: Vec<u64> = match stmt.query_map([], |row| row.get::<_, String>(0)) {
        Ok(rows) => rows
            .filter_map(|r| r.ok())
            .filter_map(|p| {
                serde_json::from_str::<serde_json::Value>(&p)
                    .ok()
                    .and_then(|v| v["detail"]["health"].as_u64())
            })
            .collect(),
        Err(_) => vec![],
    };
    let mut out = vals;
    out.reverse();
    out
}

// ─── APP STATE ───────────────────────────────────────────────────────────────
const DOMAINS: &[&str] = &[
    "all",
    "doctor",
    "git",
    "security",
    "update",
    "intent",
    "checkpoint",
];

struct App {
    events: VecDeque<PulseEvent>,
    list_state: ListState,
    domain_idx: usize,
    last_id: i64,
    last_poll: Instant,
    sparkline: Vec<u64>,
    start_time: Instant,
    paused: bool,
    limit: usize,
}

impl App {
    fn new(limit: usize) -> Self {
        Self {
            events: VecDeque::new(),
            list_state: ListState::default(),
            domain_idx: 0,
            last_id: 0,
            last_poll: Instant::now() - Duration::from_secs(10),
            sparkline: vec![],
            start_time: Instant::now(),
            paused: false,
            limit,
        }
    }

    fn active_filter(&self) -> Option<&str> {
        let d = DOMAINS[self.domain_idx];
        if d == "all" {
            None
        } else {
            Some(d)
        }
    }

    fn poll(&mut self, conn: &Connection) {
        if self.paused {
            return;
        }
        if self.last_poll.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_poll = Instant::now();

        let new = load_events(conn, self.last_id, self.active_filter(), self.limit);
        if !new.is_empty() {
            self.last_id = new.iter().map(|e| e.id).max().unwrap_or(self.last_id);
            for e in new.into_iter().rev() {
                self.events.push_front(e);
            }
            while self.events.len() > 200 {
                self.events.pop_back();
            }
        }
        self.sparkline = load_health_sparkline(conn);
    }

    fn current_health(&self) -> u64 {
        self.sparkline.last().copied().unwrap_or(0)
    }

    fn uptime(&self) -> String {
        let s = self.start_time.elapsed().as_secs();
        format!("{}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
    }
}

// ─── DRAW ────────────────────────────────────────────────────────────────────
fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        f.render_widget(Block::default().style(Style::default().bg(BG)), area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header
                Constraint::Length(4), // health gauge + sparkline
                Constraint::Length(3), // filter bar
                Constraint::Min(0),    // events
                Constraint::Length(1), // footer
            ])
            .split(area);

        // ── HEADER ──────────────────────────────────────────────────────
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(32)])
            .split(chunks[0]);

        let left = Paragraph::new(Line::from(vec![
            Span::styled(" 🌲 ", Style::default().fg(ACCENT)),
            Span::styled(
                "Core Pulse",
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(ACCENT)),
        );
        f.render_widget(left, header_chunks[0]);

        let right = Paragraph::new(Line::from(vec![Span::styled(
            format!("{}  ⏱ {}  ", now, app.uptime()),
            Style::default().fg(DIM),
        )]))
        .alignment(ratatui::layout::Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(ACCENT)),
        );
        f.render_widget(right, header_chunks[1]);

        // ── HEALTH + SPARKLINE ───────────────────────────────────────────
        let health_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(0)])
            .split(chunks[1]);

        let health = app.current_health();
        let gauge_color = if health >= 95 {
            ACCENT
        } else if health >= 80 {
            YELLOW
        } else {
            RED
        };
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(Span::styled(" System Health ", Style::default().fg(DIM)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DIM)),
            )
            .gauge_style(Style::default().fg(gauge_color).bg(BG))
            .percent(health as u16)
            .label(Span::styled(
                format!("{}%", health),
                Style::default()
                    .fg(gauge_color)
                    .add_modifier(Modifier::BOLD),
            ));
        f.render_widget(gauge, health_chunks[0]);

        let spark_data: Vec<u64> = app.sparkline.clone();
        let spark = Sparkline::default()
            .block(
                Block::default()
                    .title(Span::styled(
                        " Health Trend (last 20 checks) ",
                        Style::default().fg(DIM),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DIM)),
            )
            .data(&spark_data)
            .style(Style::default().fg(ACCENT));
        f.render_widget(spark, health_chunks[1]);

        // ── FILTER BAR ──────────────────────────────────────────────────
        let filter_spans: Vec<Span> = DOMAINS
            .iter()
            .enumerate()
            .flat_map(|(i, d)| {
                let active = i == app.domain_idx;
                let color = if *d == "all" { FG } else { domain_color(d) };
                let style = if active {
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(color)
                };
                vec![Span::raw("  "), Span::styled(format!(" {} ", d), style)]
            })
            .collect();

        let paused_span = if app.paused {
            Span::styled(
                "  ⏸ PAUSED  ",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("  ● LIVE  ", Style::default().fg(ACCENT))
        };

        let mut all_spans = filter_spans;
        all_spans.push(paused_span);
        all_spans.insert(0, Span::styled(" Filter: ", Style::default().fg(DIM)));

        let filter_bar = Paragraph::new(Line::from(all_spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM))
                .title(Span::styled(
                    " Tab: cycle domain  Space: pause  q: quit ",
                    Style::default().fg(DIM),
                )),
        );
        f.render_widget(filter_bar, chunks[2]);

        // ── EVENT LIST ───────────────────────────────────────────────────
        let items: Vec<ListItem> = app
            .events
            .iter()
            .map(|e| {
                let color = domain_color(&e.domain);
                let icon = domain_icon(&e.domain);
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {}  ", e.time_str()), Style::default().fg(DIM)),
                    Span::styled(format!("{} ", icon), Style::default()),
                    Span::styled(
                        format!("{:<10}", e.domain),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {:<12}", e.action), Style::default().fg(FG)),
                    Span::styled(format!("  {}", e.detail_short()), Style::default().fg(DIM)),
                ]))
            })
            .collect();

        let event_count = app.events.len();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(ACCENT))
                    .title(Line::from(vec![
                        Span::styled(
                            " 🌲 Event Stream ",
                            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("({} events) ", event_count),
                            Style::default().fg(DIM),
                        ),
                    ])),
            )
            .style(Style::default().bg(BG))
            .highlight_style(Style::default().bg(Color::Rgb(30, 40, 30)));
        f.render_stateful_widget(list, chunks[3], &mut app.list_state);

        // ── FOOTER ──────────────────────────────────────────────────────
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("  q", Style::default().fg(ACCENT)),
            Span::styled(" quit  ", Style::default().fg(DIM)),
            Span::styled("Tab", Style::default().fg(ACCENT)),
            Span::styled(" filter  ", Style::default().fg(DIM)),
            Span::styled("Space", Style::default().fg(ACCENT)),
            Span::styled(" pause  ", Style::default().fg(DIM)),
            Span::styled("↑↓", Style::default().fg(ACCENT)),
            Span::styled(" scroll  ", Style::default().fg(DIM)),
            Span::styled("r", Style::default().fg(ACCENT)),
            Span::styled(" reset  ", Style::default().fg(DIM)),
        ]))
        .style(Style::default().fg(DIM).bg(BG));
        f.render_widget(footer, chunks[4]);
    })?;
    Ok(())
}

// ─── MAIN ────────────────────────────────────────────────────────────────────
fn main() -> Result<()> {
    let cli = Cli::parse();

    let conn = Connection::open(db_path())?;

    // JSON mode — no TUI
    if cli.json {
        let events = load_events(&conn, 0, cli.domain.as_deref(), cli.limit);
        for e in events.iter().rev() {
            println!(
                "{}",
                serde_json::json!({
                    "time": e.time_str(),
                    "domain": e.domain,
                    "action": e.action,
                    "detail": e.detail_short(),
                })
            );
        }
        return Ok(());
    }

    // TUI mode
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(cli.limit);

    // Set initial domain filter from CLI
    if let Some(ref d) = cli.domain {
        if let Some(idx) = DOMAINS.iter().position(|x| x == d) {
            app.domain_idx = idx;
        }
    }

    // Load initial events
    let initial = load_events(&conn, 0, app.active_filter(), app.limit);
    app.last_id = initial.iter().map(|e| e.id).max().unwrap_or(0);
    for e in initial.into_iter().rev() {
        app.events.push_front(e);
    }
    app.sparkline = load_health_sparkline(&conn);

    loop {
        app.poll(&conn);
        draw(&mut terminal, &mut app)?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Tab => {
                        app.domain_idx = (app.domain_idx + 1) % DOMAINS.len();
                        // Reload for new filter
                        app.events.clear();
                        app.last_id = 0;
                        let evs = load_events(&conn, 0, app.active_filter(), app.limit);
                        app.last_id = evs.iter().map(|e| e.id).max().unwrap_or(0);
                        for e in evs.into_iter().rev() {
                            app.events.push_front(e);
                        }
                    }
                    KeyCode::Char(' ') => {
                        app.paused = !app.paused;
                    }
                    KeyCode::Char('r') => {
                        app.events.clear();
                        app.last_id = 0;
                        let evs = load_events(&conn, 0, app.active_filter(), app.limit);
                        app.last_id = evs.iter().map(|e| e.id).max().unwrap_or(0);
                        for e in evs.into_iter().rev() {
                            app.events.push_front(e);
                        }
                    }
                    KeyCode::Down => {
                        let i = app.list_state.selected().unwrap_or(0);
                        app.list_state.select(Some(
                            i.saturating_add(1).min(app.events.len().saturating_sub(1)),
                        ));
                    }
                    KeyCode::Up => {
                        let i = app.list_state.selected().unwrap_or(0);
                        app.list_state.select(Some(i.saturating_sub(1)));
                    }
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
