// INT-342: core db browse -- Forest-Native state.db TUI Browser
// Phase 1: table list, row counts, basic navigation
// Phase 2: forest icons, jump keys, color palette
// Phase 3: / filter, : SQL query mode
// Phase 4: y/Y yank, x export, schema view, preview panel
// Phase 5: core db browse <table> direct jump, db alias

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState,
        Paragraph, Row, Table, TableState, Wrap,
    },
    Terminal,
};
use rusqlite::Connection;
use std::io;

// Forest palette (design-system.md)
const BG:      Color = Color::Rgb(10, 15, 10);
const FG:      Color = Color::Rgb(168, 197, 176);
const GREEN:   Color = Color::Rgb(42, 255, 213);
const ACCENT:  Color = Color::Rgb(0, 191, 255);
const AMBER:   Color = Color::Rgb(255, 212, 59);
const DIM:     Color = Color::Rgb(74, 107, 82);
const ROW_ALT: Color = Color::Rgb(15, 22, 15);
const SELECT:  Color = Color::Rgb(0, 60, 40);

// Table categories and icons
fn table_icon(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if n.contains("intent") { return "🌲"; }
    if n.contains("friday") || n.contains("synthesis") { return "🔮"; }
    if n.contains("event") || n.contains("audit") { return "📋"; }
    if n.contains("deploy") { return "🚀"; }
    if n.contains("failure") || n.contains("integrity") || n.contains("security") { return "🔒"; }
    if n.contains("history") || n.contains("snapshot") || n.contains("checkpoint") { return "📊"; }
    if n.contains("shell") || n.contains("alias") { return "⚙️ "; }
    "  "
}

// Jump key for a table
fn jump_key(name: &str) -> Option<char> {
    match name {
        "intents" => Some('i'),
        "friday_patterns" => Some('f'),
        "events" => Some('e'),
        "friday_knowledge" => Some('k'),
        "shell_history" => Some('h'),
        "friday_decisions" => Some('d'),
        "friday_attention" => Some('a'),
        "intent_commits" => Some('c'),
        _ => None,
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Panel { Tables, Data }

#[derive(PartialEq, Clone, Copy)]
#[allow(dead_code)]
enum Mode { Normal, Filter, Query, Schema, Preview }

struct App {
    db: Connection,
    tables: Vec<(String, i64)>,    // (name, row_count)
    table_state: ListState,
    data_headers: Vec<String>,
    data_rows: Vec<Vec<String>>,
    filtered_rows: Vec<Vec<String>>,
    data_state: TableState,
    col_offset: usize,
    active_panel: Panel,
    mode: Mode,
    filter_input: String,
    query_input: String,
    query_history: Vec<String>,
    query_history_idx: usize,
    status_msg: String,
    preview_text: String,
    schema_text: String,
    col_widths: Vec<u16>,
}

impl App {
    fn new(initial_table: Option<&str>) -> anyhow::Result<Self> {
        let db = Connection::open(faelight_core::paths::state_db())?;
        let tables = get_tables(&db);
        let mut app = App {
            db,
            tables: tables.clone(),
            table_state: ListState::default(),
            data_headers: vec![],
            data_rows: vec![],
            filtered_rows: vec![],
            data_state: TableState::default(),
            col_offset: 0,
            active_panel: Panel::Tables,
            mode: Mode::Normal,
            filter_input: String::new(),
            query_input: String::new(),
            query_history: vec![],
            query_history_idx: 0,
            status_msg: String::from("hjkl navigate · Tab switch panel · / filter · : query · s schema · y yank · q quit"),
            preview_text: String::new(),
            schema_text: String::new(),
            col_widths: vec![],
        };
        if !tables.is_empty() {
            // Jump to initial table if specified
            let idx = if let Some(t) = initial_table {
                tables.iter().position(|(n, _)| n == t).unwrap_or(0)
            } else { 0 };
            app.table_state.select(Some(idx));
            app.load_table(idx);
        }
        Ok(app)
    }

    fn load_table(&mut self, idx: usize) {
        if idx >= self.tables.len() { return; }
        let name = self.tables[idx].0.clone();
        let (headers, rows) = query_table(&self.db, &name, 500);
        self.col_widths = calc_col_widths(&headers, &rows);
        self.data_headers = headers;
        self.data_rows = rows.clone();
        self.filtered_rows = rows;
        self.data_state = TableState::default();
        if !self.filtered_rows.is_empty() {
            self.data_state.select(Some(0));
        }
        self.col_offset = 0;
        self.filter_input.clear();
        self.schema_text = get_schema(&self.db, &name);
        self.update_preview();
        self.status_msg = format!("Table: {} ({} rows)", name, self.filtered_rows.len());
    }

    fn apply_filter(&mut self) {
        let f = self.filter_input.to_lowercase();
        if f.is_empty() {
            self.filtered_rows = self.data_rows.clone();
        } else {
            self.filtered_rows = self.data_rows.iter()
                .filter(|row| row.iter().any(|cell| cell.to_lowercase().contains(&f)))
                .cloned().collect();
        }
        if self.filtered_rows.is_empty() {
            self.data_state.select(None);
        } else {
            self.data_state.select(Some(0));
        }
        let name = self.current_table_name();
        self.status_msg = if f.is_empty() {
            format!("Table: {} ({} rows)", name, self.filtered_rows.len())
        } else {
            format!("{}/{} rows (filter: {})", self.filtered_rows.len(), self.data_rows.len(), f)
        };
    }

    fn run_query(&mut self) {
        let q = self.query_input.trim().to_string();
        if q.is_empty() { return; }
        self.query_history.push(q.clone());
        self.query_history_idx = self.query_history.len();
        let (headers, rows) = run_sql(&self.db, &q);
        self.col_widths = calc_col_widths(&headers, &rows);
        self.data_headers = headers;
        self.filtered_rows = rows.clone();
        self.data_rows = rows;
        self.data_state = TableState::default();
        if !self.filtered_rows.is_empty() {
            self.data_state.select(Some(0));
        }
        self.col_offset = 0;
        self.status_msg = format!("Query: {} ({} rows)", &q[..q.len().min(40)], self.filtered_rows.len());
        self.query_input.clear();
        self.mode = Mode::Normal;
        self.update_preview();
    }

    fn current_table_name(&self) -> String {
        self.table_state.selected()
            .and_then(|i| self.tables.get(i))
            .map(|(n, _)| n.clone())
            .unwrap_or_default()
    }

    fn update_preview(&mut self) {
        if let Some(row_idx) = self.data_state.selected() {
            if let Some(row) = self.filtered_rows.get(row_idx) {
                self.preview_text = self.data_headers.iter().zip(row.iter())
                    .map(|(h, v)| format!("{}: {}", h, v))
                    .collect::<Vec<_>>()
                    .join("\n");
            }
        }
    }

    fn yank_cell(&self) -> Option<String> {
        let row_idx = self.data_state.selected()?;
        let row = self.filtered_rows.get(row_idx)?;
        // Yank focused column (col_offset or first visible)
        row.get(self.col_offset).cloned()
    }

    fn yank_row(&self) -> Option<String> {
        let row_idx = self.data_state.selected()?;
        let row = self.filtered_rows.get(row_idx)?;
        Some(row.join(","))
    }

    fn export_csv(&self) -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create("/tmp/forest-export.csv")?;
        writeln!(f, "{}", self.data_headers.join(","))?;
        for row in &self.filtered_rows {
            writeln!(f, "{}", row.join(","))?;
        }
        Ok(())
    }
}

fn get_tables(db: &Connection) -> Vec<(String, i64)> {
    let mut stmt = db.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    ).unwrap();
    let names: Vec<String> = stmt.query_map([], |r| r.get(0))
        .unwrap().flatten().collect();
    names.into_iter().map(|name| {
        let count: i64 = db.query_row(
            &format!("SELECT COUNT(*) FROM \"{}\"", name), [], |r| r.get(0)
        ).unwrap_or(0);
        (name, count)
    }).collect()
}

fn query_table(db: &Connection, name: &str, limit: usize) -> (Vec<String>, Vec<Vec<String>>) {
    let sql = format!("SELECT * FROM \"{}\" LIMIT {}", name, limit);
    run_sql(db, &sql)
}

fn run_sql(db: &Connection, sql: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let mut stmt = match db.prepare(sql) {
        Ok(s) => s,
        Err(e) => return (vec!["error".to_string()], vec![vec![e.to_string()]]),
    };
    let headers: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let col_count = headers.len();
    let rows: Vec<Vec<String>> = stmt.query_map([], |r| {
        let mut row = Vec::new();
        for i in 0..col_count {
            let val = match r.get_ref(i) {
                Ok(v) => match v {
                    rusqlite::types::ValueRef::Null => "NULL".to_string(),
                    rusqlite::types::ValueRef::Integer(n) => n.to_string(),
                    rusqlite::types::ValueRef::Real(f) => format!("{:.3}", f),
                    rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
                    rusqlite::types::ValueRef::Blob(b) => format!("<blob {} bytes>", b.len()),
                },
                Err(_) => "?".to_string(),
            };
            row.push(val);
        }
        Ok(row)
    }).unwrap().flatten().collect();
    (headers, rows)
}

fn get_schema(db: &Connection, name: &str) -> String {
    db.query_row(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name=?1",
        rusqlite::params![name], |r| r.get::<_, String>(0)
    ).unwrap_or_else(|_| format!("No schema for {}", name))
}

fn calc_col_widths(headers: &[String], rows: &[Vec<String>]) -> Vec<u16> {
    headers.iter().enumerate().map(|(i, h)| {
        let max_data = rows.iter()
            .filter_map(|r| r.get(i))
            .map(|v| v.len().min(40))
            .max()
            .unwrap_or(0);
        (h.len().max(max_data) + 2).min(42) as u16
    }).collect()
}

fn draw(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    f.render_widget(ratatui::widgets::Block::default().style(Style::default().bg(BG)), area);

    let show_preview = !app.preview_text.is_empty() && app.mode == Mode::Normal;
    let show_schema  = app.mode == Mode::Schema;

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(if show_preview || show_schema { 5 } else { 0 }),
            Constraint::Length(3),
        ])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(1)])
        .split(main_chunks[0]);

    // LEFT: table list
    draw_table_list(f, app, top_chunks[0]);
    // RIGHT: data panel
    draw_data_panel(f, app, top_chunks[1]);
    // BOTTOM PANEL: preview or schema
    if show_schema {
        draw_schema(f, app, main_chunks[1]);
    } else if show_preview {
        draw_preview(f, app, main_chunks[1]);
    }
    // STATUS BAR
    draw_status(f, app, main_chunks[2]);
}

fn draw_table_list(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let border_color = if app.active_panel == Panel::Tables { GREEN } else { DIM };
    let items: Vec<ListItem> = app.tables.iter().enumerate().map(|(i, (name, count))| {
        let icon = table_icon(name);
        let key = jump_key(name).map(|k| format!("[{}]", k)).unwrap_or_default();
        let selected = app.table_state.selected() == Some(i);
        let style = if selected {
            Style::default().fg(BG).bg(GREEN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(FG)
        };
        ListItem::new(Line::from(vec![
            Span::raw(format!("{} ", icon)),
            Span::styled(format!("{:<18}", &name[..name.len().min(18)]), style),
            Span::styled(format!(" {:>4}", count), Style::default().fg(AMBER)),
            if !key.is_empty() {
                Span::styled(format!(" {}", key), Style::default().fg(DIM))
            } else {
                Span::raw("")
            },
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(" Tables ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(BG)));
    f.render_stateful_widget(list, area, &mut app.table_state);
}

fn draw_data_panel(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let border_color = if app.active_panel == Panel::Data { GREEN } else { DIM };
    let name = app.current_table_name();
    let title = match app.mode {
        Mode::Filter => format!(" {} │ filter: {} ", name, app.filter_input),
        Mode::Query  => format!(" SQL: {} ", app.query_input),
        _            => format!(" {} ", name),
    };

    if app.data_headers.is_empty() {
        let p = Paragraph::new("No data")
            .block(Block::default().borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(BG)));
        f.render_widget(p, area);
        return;
    }

    // Visible columns based on col_offset
    let available_width = area.width.saturating_sub(4);
    let mut visible_cols: Vec<usize> = vec![];
    let mut used = 0u16;
    for i in app.col_offset..app.data_headers.len() {
        let w = app.col_widths.get(i).copied().unwrap_or(10);
        if used + w > available_width && !visible_cols.is_empty() { break; }
        visible_cols.push(i);
        used += w;
    }

    let header_cells: Vec<Cell> = visible_cols.iter().map(|&i| {
        Cell::from(app.data_headers[i].clone())
            .style(Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
    }).collect();
    let header = Row::new(header_cells).style(Style::default().bg(BG)).height(1);

    let rows: Vec<Row> = app.filtered_rows.iter().enumerate().map(|(ri, row)| {
        let cells: Vec<Cell> = visible_cols.iter().map(|&ci| {
            let val = row.get(ci).map(|s| {
                if s.len() > 40 { format!("{}…", &s[..39]) } else { s.clone() }
            }).unwrap_or_default();
            Cell::from(val).style(Style::default().fg(FG))
        }).collect();
        let bg = if ri % 2 == 0 { BG } else { ROW_ALT };
        let selected_bg = SELECT;
        let is_selected = app.data_state.selected() == Some(ri);
        Row::new(cells).style(Style::default().bg(if is_selected { selected_bg } else { bg }))
    }).collect();

    let widths: Vec<Constraint> = visible_cols.iter()
        .map(|&i| Constraint::Length(app.col_widths.get(i).copied().unwrap_or(10)))
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(BG)))
        .highlight_style(Style::default().bg(SELECT));
    f.render_stateful_widget(table, area, &mut app.data_state);
}

fn draw_preview(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let p = Paragraph::new(app.preview_text.clone())
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(DIM))
            .title(Span::styled(" Preview ", Style::default().fg(DIM)))
            .style(Style::default().bg(BG)))
        .style(Style::default().fg(FG))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_schema(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let p = Paragraph::new(app.schema_text.clone())
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(AMBER))
            .title(Span::styled(" Schema ", Style::default().fg(AMBER)))
            .style(Style::default().bg(BG)))
        .style(Style::default().fg(FG))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_status(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let msg = match app.mode {
        Mode::Filter => format!("FILTER: {}  (Esc clear · Enter apply)", app.filter_input),
        Mode::Query  => format!("SQL: {}  (Enter run · Esc cancel)", app.query_input),
        Mode::Schema => "SCHEMA  (s or Esc to close)".to_string(),
        Mode::Preview => "PREVIEW  (p or Esc to close)".to_string(),
        Mode::Normal => app.status_msg.clone(),
    };
    let p = Paragraph::new(Line::from(vec![
        Span::styled(" 🌲 ", Style::default().fg(GREEN)),
        Span::styled(msg, Style::default().fg(FG)),
    ])).block(Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .style(Style::default().bg(BG)));
    f.render_widget(p, area);
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let initial_table = args.get(1).map(|s| s.as_str());

    let mut app = App::new(initial_table)?;

    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture);
        panic_hook(info);
    }));

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Filter => match key.code {
                    KeyCode::Esc => {
                        app.filter_input.clear();
                        app.apply_filter();
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Enter => {
                        app.apply_filter();
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        app.filter_input.pop();
                        app.apply_filter();
                    }
                    KeyCode::Char(c) => {
                        app.filter_input.push(c);
                        app.apply_filter();
                    }
                    _ => {}
                },
                Mode::Query => match key.code {
                    KeyCode::Esc => { app.query_input.clear(); app.mode = Mode::Normal; }
                    KeyCode::Enter => { app.run_query(); }
                    KeyCode::Backspace => { app.query_input.pop(); }
                    KeyCode::Up => {
                        if !app.query_history.is_empty() && app.query_history_idx > 0 {
                            app.query_history_idx -= 1;
                            app.query_input = app.query_history[app.query_history_idx].clone();
                        }
                    }
                    KeyCode::Down => {
                        if app.query_history_idx + 1 < app.query_history.len() {
                            app.query_history_idx += 1;
                            app.query_input = app.query_history[app.query_history_idx].clone();
                        } else {
                            app.query_input.clear();
                        }
                    }
                    KeyCode::Char(c) => { app.query_input.push(c); }
                    _ => {}
                },
                Mode::Schema | Mode::Preview => match key.code {
                    KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('p') | KeyCode::Char('q') => {
                        app.mode = Mode::Normal;
                    }
                    _ => {}
                },
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Esc => break,
                    KeyCode::Tab => {
                        app.active_panel = match app.active_panel {
                            Panel::Tables => Panel::Data,
                            Panel::Data => Panel::Tables,
                        };
                    }
                    // Navigation
                    KeyCode::Char('j') | KeyCode::Down => {
                        match app.active_panel {
                            Panel::Tables => {
                                let next = app.table_state.selected()
                                    .map(|i| (i + 1).min(app.tables.len().saturating_sub(1)))
                                    .unwrap_or(0);
                                app.table_state.select(Some(next));
                                app.load_table(next);
                            }
                            Panel::Data => {
                                let next = app.data_state.selected()
                                    .map(|i| (i + 1).min(app.filtered_rows.len().saturating_sub(1)))
                                    .unwrap_or(0);
                                app.data_state.select(Some(next));
                                app.update_preview();
                            }
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        match app.active_panel {
                            Panel::Tables => {
                                let prev = app.table_state.selected()
                                    .map(|i| i.saturating_sub(1)).unwrap_or(0);
                                app.table_state.select(Some(prev));
                                app.load_table(prev);
                            }
                            Panel::Data => {
                                let prev = app.data_state.selected()
                                    .map(|i| i.saturating_sub(1)).unwrap_or(0);
                                app.data_state.select(Some(prev));
                                app.update_preview();
                            }
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if app.active_panel == Panel::Data {
                            app.col_offset = (app.col_offset + 1).min(app.data_headers.len().saturating_sub(1));
                        } else {
                            app.active_panel = Panel::Data;
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        match app.active_panel {
                            Panel::Data => { app.col_offset = app.col_offset.saturating_sub(1); }
                            Panel::Tables => {
                                if let Some(idx) = app.tables.iter().position(|(n, _)| n == "shell_history") {
                                    app.table_state.select(Some(idx));
                                    app.load_table(idx);
                                }
                            }
                        }
                    }
                    // Page scroll
                    KeyCode::Char('u') | KeyCode::PageUp => {
                        let cur = app.data_state.selected().unwrap_or(0);
                        let prev = cur.saturating_sub(20);
                        app.data_state.select(Some(prev));
                        app.update_preview();
                    }
                    KeyCode::Char('d') | KeyCode::PageDown => {
                        let cur = app.data_state.selected().unwrap_or(0);
                        let next = (cur + 20).min(app.filtered_rows.len().saturating_sub(1));
                        app.data_state.select(Some(next));
                        app.update_preview();
                    }
                    KeyCode::Char('g') => {
                        app.data_state.select(Some(0));
                        app.update_preview();
                    }
                    KeyCode::Char('G') => {
                        let last = app.filtered_rows.len().saturating_sub(1);
                        app.data_state.select(Some(last));
                        app.update_preview();
                    }
                    // Jump keys
                    KeyCode::Char('i') => {
                        if let Some(idx) = app.tables.iter().position(|(n,_)| n=="intents") {
                            app.table_state.select(Some(idx)); app.load_table(idx);
                            app.active_panel = Panel::Data;
                        }
                    }
                    KeyCode::Char('f') => {
                        if let Some(idx) = app.tables.iter().position(|(n,_)| n=="friday_patterns") {
                            app.table_state.select(Some(idx)); app.load_table(idx);
                            app.active_panel = Panel::Data;
                        }
                    }
                    KeyCode::Char('e') => {
                        if let Some(idx) = app.tables.iter().position(|(n,_)| n=="events") {
                            app.table_state.select(Some(idx)); app.load_table(idx);
                            app.active_panel = Panel::Data;
                        }
                    }
                    KeyCode::Char('a') => {
                        if let Some(idx) = app.tables.iter().position(|(n,_)| n=="friday_attention") {
                            app.table_state.select(Some(idx)); app.load_table(idx);
                            app.active_panel = Panel::Data;
                        }
                    }
                    KeyCode::Char('c') => {
                        if let Some(idx) = app.tables.iter().position(|(n,_)| n=="intent_commits") {
                            app.table_state.select(Some(idx)); app.load_table(idx);
                            app.active_panel = Panel::Data;
                        }
                    }
                    // Modes
                    KeyCode::Char('/') => { app.mode = Mode::Filter; }
                    KeyCode::Char(':') => { app.mode = Mode::Query; app.query_input.clear(); }
                    KeyCode::Char('s') => {
                        app.mode = if app.mode == Mode::Schema { Mode::Normal } else { Mode::Schema };
                    }
                    // Yank / export
                    KeyCode::Char('y') => {
                        if let Some(val) = app.yank_cell() {
                            app.status_msg = format!("Yanked: {}", &val[..val.len().min(60)]);
                        }
                    }
                    KeyCode::Char('Y') => {
                        if let Some(val) = app.yank_row() {
                            app.status_msg = format!("Yanked row: {}", &val[..val.len().min(60)]);
                        }
                    }
                    KeyCode::Char('x') => {
                        match app.export_csv() {
                            Ok(_) => app.status_msg = format!("Exported {} rows to /tmp/forest-export.csv", app.filtered_rows.len()),
                            Err(e) => app.status_msg = format!("Export failed: {}", e),
                        }
                    }
                    KeyCode::Char('r') => {
                        // Reload current table
                        if let Some(idx) = app.table_state.selected() {
                            app.tables = get_tables(&app.db);
                            app.load_table(idx);
                            app.status_msg = "Reloaded".to_string();
                        }
                    }
                    KeyCode::Char('?') => {
                        app.status_msg = "j/k navigate · l/h cols · Tab panel · / filter · : SQL · s schema · i/f/e/a/c jump · y yank · Y row · x export · r reload · q quit".to_string();
                    }
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
