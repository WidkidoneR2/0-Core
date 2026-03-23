//! faelight-login v1.0.0
//! 🌲 Faelight Forest — The forest greets you first.
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, terminal,
};
use greetd_ipc::{codec::SyncCodec, AuthMessageType, Request, Response};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

const ACCENT: Color = Color::Rgb(100, 220, 100); // true green
const DIM: Color = Color::Rgb(119, 143, 127); // #778f7f
const BG: Color = Color::Rgb(15, 20, 17); // #0f1411
const FG: Color = Color::Rgb(215, 224, 218); // #d7e0da
const ERR: Color = Color::Rgb(227, 107, 107); // #e36b6b

#[derive(Clone, PartialEq)]
enum Field {
    Username,
    Password,
}

#[derive(Clone, PartialEq)]
enum SessionChoice {
    Niri,
    Sway,
}

impl SessionChoice {
    fn cmd(&self) -> Vec<String> {
        match self {
            SessionChoice::Niri => vec!["niri-session".to_string()],
            SessionChoice::Sway => vec!["sway".to_string()],
        }
    }
    fn label(&self) -> &str {
        match self {
            SessionChoice::Niri => "Niri",
            SessionChoice::Sway => "Sway",
        }
    }
}

struct LoginState {
    username: String,
    password: String,
    focused: Field,
    session: SessionChoice,
    error: Option<String>,
    status: String,
    health: String,
    commits: String,
    #[allow(dead_code)]
    authenticating: bool,
    boot_time: Instant,
}

impl LoginState {
    fn new() -> Self {
        let health = read_health();
        let commits = read_commits();
        Self {
            username: String::new(),
            password: String::new(),
            focused: Field::Username,
            session: SessionChoice::Niri,
            error: None,
            status: "Welcome to Faelight Forest".to_string(),
            health,
            commits,
            authenticating: false,
            boot_time: Instant::now(),
        }
    }
}

fn read_health() -> String {
    std::fs::read_to_string("/etc/faelight/HEALTH")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "??%".to_string())
}

fn read_commits() -> String {
    std::fs::read_to_string("/etc/faelight/COMMITS")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "?".to_string())
}

fn greet(state: &mut LoginState) -> Result<bool, String> {
    let sock_path = std::env::var("GREETD_SOCK")
        .map_err(|_| "GREETD_SOCK not set — run via greetd".to_string())?;

    let mut stream = UnixStream::connect(&sock_path).map_err(|e| format!("Socket error: {}", e))?;

    // Create session
    let req = Request::CreateSession {
        username: state.username.clone(),
    };
    req.write_to(&mut stream).map_err(|e| e.to_string())?;

    loop {
        let response = Response::read_from(&mut stream).map_err(|e| e.to_string())?;
        match response {
            Response::AuthMessage {
                auth_message_type,
                auth_message: _,
            } => match auth_message_type {
                AuthMessageType::Secret => {
                    let resp = Request::PostAuthMessageResponse {
                        response: Some(state.password.clone()),
                    };
                    resp.write_to(&mut stream).map_err(|e| e.to_string())?;
                }
                AuthMessageType::Visible => {
                    let resp = Request::PostAuthMessageResponse {
                        response: Some(state.username.clone()),
                    };
                    resp.write_to(&mut stream).map_err(|e| e.to_string())?;
                }
                AuthMessageType::Info | AuthMessageType::Error => {
                    Request::PostAuthMessageResponse { response: None }
                        .write_to(&mut stream)
                        .map_err(|e| e.to_string())?;
                }
            },
            Response::Success => {
                let start = Request::StartSession {
                    cmd: state.session.cmd(),
                    env: vec![
                        "XDG_SESSION_TYPE=wayland".to_string(),
                        format!("XDG_CURRENT_DESKTOP={}", state.session.label()),
                    ],
                };
                start.write_to(&mut stream).map_err(|e| e.to_string())?;
                let _ = Response::read_from(&mut stream);
                return Ok(true);
            }
            Response::Error {
                error_type: _,
                description,
            } => {
                let _ = Request::CancelSession.write_to(&mut stream);
                return Err(description);
            }
        }
    }
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &LoginState,
) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        f.render_widget(Block::default().style(Style::default().bg(BG)), area);

        let pulse = ((state.boot_time.elapsed().as_millis() % 2000) as f64 / 2000.0
            * std::f64::consts::TAU)
            .sin();
        let pulse_color = if pulse > 0.5 { ACCENT } else { DIM };

        // Center layout
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(18),
                Constraint::Fill(1),
            ])
            .split(area);

        let horiz = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(70),
                Constraint::Fill(1),
            ])
            .split(vert[1]);

        let panel = horiz[1];

        // Outer border
        let border = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .title(Line::from(vec![
                Span::styled(" 🌲 ", Style::default().fg(ACCENT)),
                Span::styled(
                    "Faelight Forest",
                    Style::default().fg(FG).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" v{} ", read_system_version()),
                    Style::default().fg(DIM),
                ),
            ]))
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(BG));
        f.render_widget(border, panel);

        let inner = panel.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // spacer
                Constraint::Length(1), // status
                Constraint::Length(1), // spacer
                Constraint::Length(3), // username
                Constraint::Length(1), // spacer
                Constraint::Length(3), // password
                Constraint::Length(1), // spacer
                Constraint::Length(1), // session
                Constraint::Length(1), // spacer
                Constraint::Length(1), // health + commits
                Constraint::Length(1), // spacer
                Constraint::Length(1), // hint
            ])
            .split(inner);

        // Status / error
        let status_text = if let Some(ref err) = state.error {
            Line::from(vec![Span::styled(
                format!("⚠  {}", err),
                Style::default().fg(ERR),
            )])
        } else {
            Line::from(vec![Span::styled(&state.status, Style::default().fg(DIM))])
        };
        f.render_widget(
            Paragraph::new(status_text).alignment(Alignment::Center),
            rows[1],
        );

        // Username field
        let user_style = if state.focused == Field::Username {
            Style::default().fg(ACCENT)
        } else {
            Style::default().fg(DIM)
        };
        let user_block = Block::default()
            .borders(Borders::ALL)
            .border_style(user_style)
            .title(Span::styled(" username ", Style::default().fg(DIM)));
        let user_text = Paragraph::new(state.username.as_str())
            .style(Style::default().fg(FG))
            .block(user_block);
        f.render_widget(user_text, rows[3]);

        // Password field
        let pass_style = if state.focused == Field::Password {
            Style::default().fg(ACCENT)
        } else {
            Style::default().fg(DIM)
        };
        let pass_block = Block::default()
            .borders(Borders::ALL)
            .border_style(pass_style)
            .title(Span::styled(" password ", Style::default().fg(DIM)));
        let pass_text = Paragraph::new("•".repeat(state.password.len()))
            .style(Style::default().fg(FG))
            .block(pass_block);
        f.render_widget(pass_text, rows[5]);

        // Session selector
        let niri_style = if state.session == SessionChoice::Niri {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        };
        let sway_style = if state.session == SessionChoice::Sway {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        };
        let session_line = Line::from(vec![
            Span::styled("Session: ", Style::default().fg(DIM)),
            Span::styled(
                if state.session == SessionChoice::Niri {
                    "[Niri] "
                } else {
                    " Niri  "
                },
                niri_style,
            ),
            Span::styled(
                if state.session == SessionChoice::Sway {
                    "[Sway]"
                } else {
                    " Sway "
                },
                sway_style,
            ),
            Span::styled("  (Tab)", Style::default().fg(DIM)),
        ]);
        f.render_widget(Paragraph::new(session_line), rows[7]);

        // Health + commits
        let info_line = Line::from(vec![
            Span::styled("Health: ", Style::default().fg(DIM)),
            Span::styled(&state.health, Style::default().fg(ACCENT)),
            Span::styled("  ·  Commits: ", Style::default().fg(DIM)),
            Span::styled(&state.commits, Style::default().fg(pulse_color)),
        ]);
        f.render_widget(
            Paragraph::new(info_line).alignment(Alignment::Center),
            rows[9],
        );

        // Hint
        let hint = Line::from(vec![
            Span::styled("Enter", Style::default().fg(ACCENT)),
            Span::styled(" login  ", Style::default().fg(DIM)),
            Span::styled("Tab", Style::default().fg(ACCENT)),
            Span::styled(" session  ", Style::default().fg(DIM)),
            Span::styled("Esc", Style::default().fg(ACCENT)),
            Span::styled(" clear", Style::default().fg(DIM)),
        ]);
        f.render_widget(Paragraph::new(hint).alignment(Alignment::Center), rows[11]);
    })?;
    Ok(())
}

fn read_system_version() -> String {
    std::fs::read_to_string("/etc/faelight/VERSION")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "??".to_string())
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = LoginState::new();
    let mut last_draw = Instant::now();

    loop {
        if last_draw.elapsed() > Duration::from_millis(50) {
            draw(&mut terminal, &state)?;
            last_draw = Instant::now();
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Tab => {
                        state.session = match state.session {
                            SessionChoice::Niri => SessionChoice::Sway,
                            SessionChoice::Sway => SessionChoice::Niri,
                        };
                    }
                    KeyCode::Enter => {
                        match state.focused {
                            Field::Username => {
                                state.focused = Field::Password;
                            }
                            Field::Password => {
                                state.status = "Authenticating...".to_string();
                                state.error = None;
                                draw(&mut terminal, &state)?;
                                match greet(&mut state) {
                                    Ok(true) => {
                                        // Session started — clean exit
                                        terminal::disable_raw_mode()?;
                                        execute!(
                                            terminal.backend_mut(),
                                            terminal::LeaveAlternateScreen,
                                            cursor::Show
                                        )?;
                                        return Ok(());
                                    }
                                    Ok(false) => {
                                        state.error = Some("Login failed".to_string());
                                        state.password.clear();
                                    }
                                    Err(e) => {
                                        state.error = Some(e);
                                        state.password.clear();
                                        state.focused = Field::Username;
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        state.username.clear();
                        state.password.clear();
                        state.focused = Field::Username;
                        state.error = None;
                        state.status = "Welcome to Faelight Forest".to_string();
                    }
                    KeyCode::Backspace => match state.focused {
                        Field::Username => {
                            state.username.pop();
                        }
                        Field::Password => {
                            state.password.pop();
                        }
                    },
                    KeyCode::Char(c) => match state.focused {
                        Field::Username => state.username.push(c),
                        Field::Password => state.password.push(c),
                    },
                    _ => {}
                }
            }
        }
    }
}
