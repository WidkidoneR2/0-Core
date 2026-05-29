// INT-346: faelight-ade v1 -- Forest ADE
// ratatui layout + portable-pty (fsh) + friday-chat (state.db)
// Left pane: real fsh PTY | Right pane: Friday Chat TUI

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Terminal,
};
use rusqlite::Connection;
use std::{
    io::{self, Read, Write},
    sync::{Arc, Mutex},
};

// Forest palette
const BG:     Color = Color::Rgb(10, 15, 10);
const FG:     Color = Color::Rgb(168, 197, 176);
const GREEN:  Color = Color::Rgb(42, 255, 213);
const ACCENT: Color = Color::Rgb(0, 191, 255);
const AMBER:  Color = Color::Rgb(255, 212, 59);
const DIM:    Color = Color::Rgb(74, 107, 82);

#[derive(PartialEq, Clone, Copy)]
enum ActivePane { Terminal, Friday }

#[derive(Clone)]
struct FridayMessage {
    from_friday: bool,
    text: String,
}

struct App {
    // PTY state
    pty_output: Arc<Mutex<Vec<u8>>>,
    pty_writer: Box<dyn Write + Send>,
    terminal_lines: Vec<Vec<(String, Style)>>,
    // Friday state
    friday_messages: Vec<FridayMessage>,
    friday_input: String,
    db: Connection,
    intent_hint: String,
    health_hint: String,
    // UI state
    active_pane: ActivePane,
    terminal_scroll: usize,
    friday_scroll: usize,
    status: String,
}

impl App {
    fn new() -> anyhow::Result<Self> {
        // Set ADE environment
        std::env::set_var("FAELIGHT_ADE", "1");

        // Setup PTY
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Launch fsh in PTY
        let mut cmd = CommandBuilder::new("fsh");
        cmd.env("FAELIGHT_ADE", "1");
        cmd.env("TERM", "xterm-256color");
        let _child = pair.slave.spawn_command(cmd)?;

        // PTY reader thread
        let pty_output = Arc::new(Mutex::new(Vec::<u8>::new()));
        let pty_output_clone = pty_output.clone();
        let mut reader = pair.master.try_clone_reader()?;
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if let Ok(mut out) = pty_output_clone.lock() {
                            out.extend_from_slice(&buf[..n]);
                        }
                    }
                }
            }
        });

        let pty_writer = pair.master.take_writer()?;

        // Friday state.db
        let home = dirs::home_dir().unwrap_or_default();
        let db = Connection::open(home.join("0-core/runtime/state.db"))?;
        let intent_hint = get_intent(&db);
        let health_hint = get_health(&db);

        let welcome = format!(
            "Forest ADE active\nINT-{} | Health: {}\n/help for commands",
            intent_hint, health_hint
        );

        Ok(App {
            pty_output,
            pty_writer,
            terminal_lines: vec![vec![("🌲 fsh starting...".to_string(), Style::default().fg(GREEN))]],

            friday_messages: vec![FridayMessage { from_friday: true, text: welcome }],
            friday_input: String::new(),
            db,
            intent_hint,
            health_hint,
            active_pane: ActivePane::Terminal,
            terminal_scroll: 0,
            friday_scroll: 0,
            status: "Alt+Tab switch panes · Ctrl+c exit".to_string(),
        })
    }

    fn poll_pty(&mut self) {
        if let Ok(mut out) = self.pty_output.lock() {
            if !out.is_empty() {
                let text = String::from_utf8_lossy(&out).to_string();
                let parsed = parse_ansi(&text);
                for line in parsed {
                    if !line.is_empty() {
                        self.terminal_lines.push(line);
                    }
                }
                // Keep last 1000 lines
                if self.terminal_lines.len() > 1000 {
                    self.terminal_lines.drain(0..self.terminal_lines.len() - 1000);
                }
                out.clear();
                // Auto-scroll to bottom
                self.terminal_scroll = self.terminal_lines.len().saturating_sub(1);

                // INT-346 Phase 5: Friday sees PTY output
                let last: String = self.terminal_lines.last()
                    .map(|spans| spans.iter().map(|(s,_)| s.as_str()).collect::<String>())
                    .unwrap_or_default();
                if last.contains("error") || last.contains("Error") || last.contains("warning") {
                    let msg = format!("Detected: {}", &last[..last.len().min(80)]);
                    self.friday_messages.push(FridayMessage { from_friday: true, text: msg });
                }
            }
        }
    }

    fn send_to_pty(&mut self, input: &str) {
        let _ = self.pty_writer.write_all(input.as_bytes());
    }

    fn send_friday(&mut self) {
        let input = self.friday_input.trim().to_string();
        if input.is_empty() { return; }
        self.friday_input.clear();
        self.friday_messages.push(FridayMessage { from_friday: false, text: input.clone() });
        let response = friday_respond(&self.db, &input);
        self.friday_messages.push(FridayMessage { from_friday: true, text: response });
        self.friday_scroll = self.friday_messages.len().saturating_sub(1);
    }
}

fn get_intent(db: &Connection) -> String {
    db.query_row(
        "SELECT value FROM shell_state WHERE key = 'focus_intent'",
        [], |r| r.get::<_, String>(0),
    ).unwrap_or_else(|_| "none".to_string())
}

fn get_health(db: &Connection) -> String {
    db.query_row(
        "SELECT value FROM shell_state WHERE key = 'last_health'",
        [], |r| r.get::<_, String>(0),
    ).unwrap_or_else(|_| "100%".to_string())
}

/// Parse ANSI escape sequences into ratatui styled spans
fn parse_ansi(s: &str) -> Vec<Vec<(String, Style)>> {
    let mut lines: Vec<Vec<(String, Style)>> = vec![vec![]];
    let mut current_style = Style::default().fg(FG);
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next(); // consume [
                let mut params = String::new();
                while let Some(&p) = chars.peek() {
                    if p.is_ascii_digit() || p == ';' {
                        params.push(p);
                        chars.next();
                    } else {
                        break;
                    }
                }
                chars.next(); // consume final letter (m, A, B, etc.)
                // Parse SGR color codes
                current_style = parse_sgr(&params, current_style);
            } else {
                // skip other escape sequences
                while let Some(&n) = chars.peek() {
                    chars.next();
                    if n.is_ascii_alphabetic() { break; }
                }
            }
        } else if c == '\n' {
            lines.push(vec![]);
        } else if c == '\r' {
            // ignore CR
        } else if !c.is_control() {
            if let Some(last_line) = lines.last_mut() {
                if let Some(last_span) = last_line.last_mut() {
                    if last_span.1 == current_style {
                        last_span.0.push(c);
                        continue;
                    }
                }
                last_line.push((c.to_string(), current_style));
            }
        }
    }
    let total = lines.len();
    lines.retain(|l| !l.is_empty() || total == 1);
    lines
}

fn parse_sgr(params: &str, current: Style) -> Style {
    if params.is_empty() || params == "0" {
        return Style::default().fg(FG);
    }
    let mut style = current;
    let codes: Vec<u8> = params.split(';')
        .filter_map(|p| p.parse().ok())
        .collect();
    let mut i = 0;
    while i < codes.len() {
        match codes[i] {
            0  => style = Style::default().fg(FG),
            1  => style = style.add_modifier(Modifier::BOLD),
            2  => style = style.add_modifier(Modifier::DIM),
            3  => style = style.add_modifier(Modifier::ITALIC),
            4  => style = style.add_modifier(Modifier::UNDERLINED),
            22 => style = style.remove_modifier(Modifier::BOLD),
            // Standard foreground colors
            30 => style = style.fg(Color::Black),
            31 => style = style.fg(Color::Red),
            32 => style = style.fg(Color::Green),
            33 => style = style.fg(Color::Yellow),
            34 => style = style.fg(Color::Blue),
            35 => style = style.fg(Color::Magenta),
            36 => style = style.fg(Color::Cyan),
            37 => style = style.fg(Color::White),
            39 => style = style.fg(FG),
            // Bright foreground colors
            90 => style = style.fg(Color::DarkGray),
            91 => style = style.fg(Color::LightRed),
            92 => style = style.fg(Color::LightGreen),
            93 => style = style.fg(Color::LightYellow),
            94 => style = style.fg(Color::LightBlue),
            95 => style = style.fg(Color::LightMagenta),
            96 => style = style.fg(Color::LightCyan),
            97 => style = style.fg(Color::White),
            // 256 color and RGB
            38 if i + 2 < codes.len() && codes[i+1] == 5 => {
                style = style.fg(ansi256_to_color(codes[i+2]));
                i += 2;
            }
            38 if i + 4 < codes.len() && codes[i+1] == 2 => {
                style = style.fg(Color::Rgb(codes[i+2], codes[i+3], codes[i+4]));
                i += 4;
            }
            _ => {}
        }
        i += 1;
    }
    style
}

fn ansi256_to_color(n: u8) -> Color {
    match n {
        0  => Color::Black,
        1  => Color::Red,
        2  => Color::Green,
        3  => Color::Yellow,
        4  => Color::Blue,
        5  => Color::Magenta,
        6  => Color::Cyan,
        7  => Color::White,
        8  => Color::DarkGray,
        9  => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::White,
        n  => {
            // 6x6x6 color cube
            if n >= 16 && n <= 231 {
                let n = n - 16;
                let b = (n % 6) * 51;
                let g = ((n / 6) % 6) * 51;
                let r = (n / 36) * 51;
                Color::Rgb(r, g, b)
            } else {
                // Grayscale
                let v = (n - 232) * 10 + 8;
                Color::Rgb(v, v, v)
            }
        }
    }
}

fn friday_respond(db: &Connection, input: &str) -> String {
    let lower = input.to_lowercase();
    if lower == "/help" || lower == "help" {
        return "/status /intent /patterns /facts /why /recall /trace /where /show".to_string();
    }
    if lower.starts_with("/status") || lower == "status" {
        let facts: i64 = db.query_row("SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0)).unwrap_or(0);
        let patterns: i64 = db.query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)).unwrap_or(0);
        return format!("Health: {} | INT-{} | Facts: {} | Patterns: {}",
            get_health(db), get_intent(db), facts, patterns);
    }
    if lower.starts_with("/patterns") || lower == "patterns" {
        let mut stmt = db.prepare(
            "SELECT trigger, action, confidence FROM friday_patterns ORDER BY confidence DESC LIMIT 5"
        ).unwrap();
        let rows: Vec<String> = stmt.query_map([], |r| {
            Ok(format!("{:.0}% {} → {}", r.get::<_,f64>(2)?*100.0, r.get::<_,String>(0)?, r.get::<_,String>(1)?))
        }).unwrap().flatten().collect();
        return if rows.is_empty() { "No patterns yet".to_string() } else { rows.join("\n") };
    }
    if lower.starts_with("/why") || lower.starts_with("why") {
        let term = input.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        if term.is_empty() { return "Usage: why [event]".to_string(); }
        let pattern = format!("%{}%", term);
        let mut stmt = db.prepare(
            "SELECT domain, action, payload FROM events WHERE payload LIKE ?1 ORDER BY timestamp DESC LIMIT 3"
        ).unwrap();
        let rows: Vec<String> = stmt.query_map(rusqlite::params![pattern], |r| {
            Ok(format!("[{}:{}] {}", r.get::<_,String>(0)?, r.get::<_,String>(1)?,
                r.get::<_,String>(2).unwrap_or_default()))
        }).unwrap().flatten().collect();
        return if rows.is_empty() { format!("No events matching: {}", term) }
               else { rows.join("\n") };
    }
    // Natural language fallback
    let pattern = format!("%{}%", input.split_whitespace().next().unwrap_or(""));
    let fact: Option<String> = db.query_row(
        "SELECT fact FROM friday_knowledge WHERE fact LIKE ?1 LIMIT 1",
        rusqlite::params![pattern], |r| r.get(0)
    ).ok();
    fact.unwrap_or_else(|| format!("No knowledge about '{}'. Try /why or /patterns", input))
}

fn draw(f: &mut ratatui::Frame, app: &App) {
    let area = f.area();
    f.render_widget(ratatui::widgets::Block::default().style(Style::default().bg(BG)), area);

    // Header
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" 🌲 Forest ADE ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("│ ", Style::default().fg(DIM)),
        Span::styled(format!("INT-{}", app.intent_hint), Style::default().fg(ACCENT)),
        Span::styled(" │ ", Style::default().fg(DIM)),
        Span::styled(format!("Health: {}", app.health_hint), Style::default().fg(GREEN)),
        Span::styled("  Alt+Tab: switch panes  Ctrl+c: exit", Style::default().fg(DIM)),
    ])).style(Style::default().bg(BG));
    f.render_widget(header, chunks[0]);

    // Main split
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // Terminal pane
    let term_border = if app.active_pane == ActivePane::Terminal { GREEN } else { DIM };
    let term_lines: Vec<Line> = app.terminal_lines.iter()
        .skip(app.terminal_scroll.saturating_sub(panes[0].height as usize))
        .take(panes[0].height as usize)
        .map(|spans| {
            Line::from(spans.iter()
                .map(|(text, style)| Span::styled(text.clone(), *style))
                .collect::<Vec<_>>())
        })
        .collect();
    let term_widget = Paragraph::new(term_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(term_border))
            .title(Span::styled(" fsh ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(BG)));
    f.render_widget(term_widget, panes[0]);

    // Friday pane -- split into messages + input
    let friday_border = if app.active_pane == ActivePane::Friday { GREEN } else { DIM };
    let friday_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(panes[1]);

    let friday_lines: Vec<Line> = app.friday_messages.iter()
        .flat_map(|m| {
            let prefix = if m.from_friday {
                Span::styled("🌲 ", Style::default().fg(GREEN))
            } else {
                Span::styled("  ", Style::default())
            };
            m.text.lines().enumerate().map(move |(i, line)| {
                if i == 0 {
                    Line::from(vec![prefix.clone(),
                        Span::styled(line.to_string(), Style::default().fg(FG))])
                } else {
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(line.to_string(), Style::default().fg(FG))])
                }
            }).collect::<Vec<_>>()
        }).collect();

    let friday_widget = Paragraph::new(friday_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(friday_border))
            .title(Span::styled(" Friday ", Style::default().fg(AMBER).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(BG)))
        .wrap(Wrap { trim: true });
    f.render_widget(friday_widget, friday_chunks[0]);

    // Friday input
    let input_widget = Paragraph::new(format!("> {}", app.friday_input))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(DIM))
            .title(Span::styled(" Ask Friday ", Style::default().fg(DIM)))
            .style(Style::default().bg(BG)))
        .style(Style::default().fg(FG));
    f.render_widget(input_widget, friday_chunks[1]);

    // Status bar
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(&app.status, Style::default().fg(DIM)),
    ])).style(Style::default().bg(BG));
    f.render_widget(status, chunks[2]);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::new()?;

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
        app.poll_pty();
        terminal.draw(|f| draw(f, &app))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match app.active_pane {
                    ActivePane::Terminal => match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Tab if key.modifiers.contains(KeyModifiers::ALT) => {
                            app.active_pane = ActivePane::Friday;
                        }
                        KeyCode::Char(c) => app.send_to_pty(&c.to_string()),
                        KeyCode::Enter  => app.send_to_pty("\n"),
                        KeyCode::Backspace => app.send_to_pty("\x7f"),
                        KeyCode::Tab    => app.send_to_pty("\t"),
                        KeyCode::Up     => app.send_to_pty("\x1b[A"),
                        KeyCode::Down   => app.send_to_pty("\x1b[B"),
                        KeyCode::Left   => app.send_to_pty("\x1b[D"),
                        KeyCode::Right  => app.send_to_pty("\x1b[C"),
                        KeyCode::Esc    => app.send_to_pty("\x1b"),
                        _ => {}
                    },
                    ActivePane::Friday => match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Tab if key.modifiers.contains(KeyModifiers::ALT) => {
                            app.active_pane = ActivePane::Terminal;
                        }
                        KeyCode::Enter => app.send_friday(),
                        KeyCode::Backspace => { app.friday_input.pop(); }
                        KeyCode::Char(c) => app.friday_input.push(c),
                        KeyCode::Esc => app.active_pane = ActivePane::Terminal,
                        _ => {}
                    },
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
