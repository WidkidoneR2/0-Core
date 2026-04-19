//! faelight-login v2.0.0
//! INT-242 -- The Forest Greets You First
//! Animated ASCII forest boot, Friday brief, forest status panel
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, terminal,
};
use greetd_ipc::{codec::SyncCodec, AuthMessageType, Request, Response};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};
const ACCENT: Color = Color::Rgb(100, 220, 100);
const ACCENT2: Color = Color::Rgb(80, 180, 80);
const DIM: Color = Color::Rgb(90, 110, 95);
const BG: Color = Color::Rgb(15, 20, 17);
const FG: Color = Color::Rgb(215, 224, 218);
const ERR: Color = Color::Rgb(227, 107, 107);
const GOLD: Color = Color::Rgb(200, 180, 80);
// ASCII forest tree -- rendered line by line during boot animation

#[derive(Clone, PartialEq)]
enum Field { Username, Password }

struct LoginState {
    username: String,
    password: String,
    focused: Field,
    error: Option<String>,
    health: String,
    commits: String,
    version: String,
    active_intent: String,
    friday_brief: String,
    boot_time: Instant,
    pulse: u8,
}
impl LoginState {
    fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            focused: Field::Username,
            error: None,
            health: read_file("/etc/faelight/HEALTH", "100%"),
            commits: read_file("/etc/faelight/COMMITS", "?"),
            version: read_file("/etc/faelight/VERSION", "11.9.0"),
            active_intent: {
                let s = read_active_intent();
                let truncated: String = s.chars().take(45).collect();
                if s.chars().count() > 45 { format!("{}...", truncated) } else { truncated }
            },
            friday_brief: read_friday_brief(),
            boot_time: Instant::now(),
            pulse: 0,
        }
    }
}
fn read_file(path: &str, fallback: &str) -> String {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| fallback.to_string())
}
fn read_active_intent() -> String {
    // Read from /etc/faelight/INTENT (exported by fsh on session start)
    read_file("/etc/faelight/INTENT", "")
}
fn read_friday_brief() -> String {
    let s = read_file("/etc/faelight/FRIDAY", "");
    // Filter out daemon noise -- only show Friday synthesis briefs
    if s.contains("contradiction") || s.contains("cross-layer") || s.contains("CONTRADICTION") || s.contains("signal(s)") {
        return String::new();
    }
    s.chars().take(65).collect()
}
// INT-242: data read from /etc/faelight/ files
fn greet(state: &mut LoginState) -> Result<bool, String> {
    let sock_path = std::env::var("GREETD_SOCK")
        .map_err(|_| "GREETD_SOCK not set".to_string())?;
    let mut stream = UnixStream::connect(&sock_path)
        .map_err(|e| format!("Socket error: {}", e))?;
    Request::CreateSession { username: state.username.clone() }
        .write_to(&mut stream).map_err(|e| e.to_string())?;
    loop {
        match Response::read_from(&mut stream).map_err(|e| e.to_string())? {
            Response::AuthMessage { auth_message_type, .. } => {
                let resp = match auth_message_type {
                    AuthMessageType::Secret => Some(state.password.clone()),
                    AuthMessageType::Visible => Some(state.username.clone()),
                    _ => None,
                };
                Request::PostAuthMessageResponse { response: resp }
                    .write_to(&mut stream).map_err(|e| e.to_string())?;
            }
            Response::Success => {
                Request::StartSession {
                    cmd: vec!["niri-session".to_string()],
                    env: vec![
                        "XDG_SESSION_TYPE=wayland".to_string(),
                        "XDG_CURRENT_DESKTOP=niri".to_string(),
                    ],
                }.write_to(&mut stream).map_err(|e| e.to_string())?;
                let _ = Response::read_from(&mut stream);
                return Ok(true);
            }
            Response::Error { description, .. } => {
                let _ = Request::CancelSession.write_to(&mut stream);
                return Err(description);
            }
        }
    }
}

fn draw_login(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &LoginState,
) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        f.render_widget(Block::default().style(Style::default().bg(BG)), area);
        // Center vertically by calculating padding
        let box_height = 22u16;
        let top_pad = area.height.saturating_sub(box_height) / 2;
        let bot_pad = area.height.saturating_sub(box_height).saturating_sub(top_pad);
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(top_pad),
                Constraint::Length(box_height),
                Constraint::Length(bot_pad),
            ])
            .split(area);
        let center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(70),
                Constraint::Fill(1),
            ])
            .split(outer[1]);
        let col = center[1];
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // title
                Constraint::Length(1), // spacer
                Constraint::Length(3), // username
                Constraint::Length(1), // spacer
                Constraint::Length(3), // password
                Constraint::Length(1), // spacer
                Constraint::Length(4), // status panel
                Constraint::Length(1), // friday brief
                Constraint::Length(1), // spacer
                Constraint::Length(1), // hint
            ])
            .split(col);
        // Title
        let pulse_color = if state.pulse % 2 == 0 { ACCENT } else { ACCENT2 };
        let version_str = format!("v{}", state.version);
        let title = Paragraph::new(Line::from(vec![
            Span::styled("  * Faelight Forest  ", Style::default().fg(pulse_color).add_modifier(Modifier::BOLD)),
            Span::styled(&version_str, Style::default().fg(DIM)),
        ])).block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(DIM)));
        f.render_widget(title, rows[0]);
        // Username
        let user_style = if state.focused == Field::Username {
            Style::default().fg(ACCENT)
        } else {
            Style::default().fg(DIM)
        };
        let user_block = Block::default()
            .borders(Borders::ALL)
            .border_style(user_style)
            .title(Span::styled(" username ", Style::default().fg(DIM)));
        let user_display = if state.focused == Field::Username {
            format!("{}_", state.username)
        } else {
            state.username.clone()
        };
        f.render_widget(Paragraph::new(user_display).style(Style::default().fg(FG)).block(user_block), rows[2]);
        // Password
        let pass_style = if state.focused == Field::Password {
            Style::default().fg(ACCENT)
        } else {
            Style::default().fg(DIM)
        };
        let pass_block = Block::default()
            .borders(Borders::ALL)
            .border_style(pass_style)
            .title(Span::styled(" password ", Style::default().fg(DIM)));
        let pass_display = if state.focused == Field::Password {
            format!("{}_", "•".repeat(state.password.len()))
        } else {
            "•".repeat(state.password.len())
        };
        f.render_widget(Paragraph::new(pass_display).style(Style::default().fg(FG)).block(pass_block), rows[4]);
        // Status panel
        let health_color = if state.health.starts_with("100") { ACCENT } else { GOLD };
        let status_lines = vec![
            Line::from(vec![
                Span::styled("  Health  ", Style::default().fg(DIM)),
                Span::styled(&state.health, Style::default().fg(health_color).add_modifier(Modifier::BOLD)),
                Span::styled("   Commits  ", Style::default().fg(DIM)),
                Span::styled(&state.commits, Style::default().fg(FG)),
            ]),
            Line::from(if !state.active_intent.is_empty() {
                vec![
                    Span::styled("  Intent  ", Style::default().fg(DIM)),
                    Span::styled(&state.active_intent, Style::default().fg(FG)),
                ]
            } else {
                vec![Span::styled("  No active intent", Style::default().fg(DIM))]
            }),
            Line::from(if let Some(ref err) = state.error {
                vec![Span::styled(format!("  ✗ {}", err), Style::default().fg(ERR))]
            } else {
                vec![Span::styled("", Style::default())]
            }),
        ];
        f.render_widget(
            Paragraph::new(status_lines)
                .block(Block::default().borders(Borders::TOP | Borders::BOTTOM)
                    .border_style(Style::default().fg(DIM))),
            rows[6]
        );
        // Friday brief
        if !state.friday_brief.is_empty() {
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("  * ", Style::default().fg(ACCENT)),
                    Span::styled(&state.friday_brief, Style::default().fg(DIM)),
                ])),
                rows[7]
            );
        }
        // Hint
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  Enter", Style::default().fg(ACCENT)),
                Span::styled(" login   ", Style::default().fg(DIM)),
                Span::styled("Tab", Style::default().fg(ACCENT)),
                Span::styled(" switch field   ", Style::default().fg(DIM)),
                Span::styled("Esc", Style::default().fg(ACCENT)),
                Span::styled(" clear  ", Style::default().fg(DIM)),
            ])).alignment(Alignment::Left),
            rows[9]
        );
    })?;
    Ok(())
}
fn read_system_version() -> String {
    std::fs::read_to_string("/etc/faelight/VERSION")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "11.9.0".to_string())
}
fn main() -> io::Result<()> {
    // Redirect stderr to /dev/null -- suppress daemon output bleeding into TUI
    unsafe {
        let devnull = libc::open(b"/dev/null ".as_ptr() as *const libc::c_char, libc::O_WRONLY);
        if devnull >= 0 {
            libc::dup2(devnull, 2); // redirect stderr
            libc::close(devnull);
        }
    }
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut state = LoginState::new();
    // Override version from /etc
    state.version = read_system_version();
    let mut last_draw = Instant::now();
    // Animation: advance one line every 60ms
    loop {
        let now = Instant::now();
        // Pulse for title color
        state.pulse = ((now.duration_since(state.boot_time).as_millis() / 800) % 2) as u8;
        // Draw
        if last_draw.elapsed() > Duration::from_millis(33) {
            draw_login(&mut terminal, &state)?;
            last_draw = Instant::now();
        }
        // Events
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press { continue; }
                match key.code {
                    KeyCode::Tab => {
                        state.focused = match state.focused {
                            Field::Username => Field::Password,
                            Field::Password => Field::Username,
                        };
                        state.error = None;
                    }
                    KeyCode::Esc => {
                        state.username.clear();
                        state.password.clear();
                        state.error = None;
                        state.focused = Field::Username;
                    }
                    KeyCode::Backspace => {
                        match state.focused {
                            Field::Username => { state.username.pop(); }
                            Field::Password => { state.password.pop(); }
                        }
                    }
                    KeyCode::Enter => {
                        match state.focused {
                            Field::Username => { state.focused = Field::Password; }
                            Field::Password => {
                                state.error = None;
                                draw_login(&mut terminal, &state)?;
                                match greet(&mut state) {
                                    Ok(true) => break,
                                    Ok(false) => {}
                                    Err(e) => {
                                        state.error = Some(e);
                                        state.password.clear();
                                        state.focused = Field::Password;
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        match state.focused {
                            Field::Username => state.username.push(c),
                            Field::Password => state.password.push(c),
                        }
                        state.error = None;
                    }
                    _ => {}
                }
            }
        }
    }
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}
