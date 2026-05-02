// INT-258: Ctrl+D health TUI for faelight-shell
// Same pattern as history_tui (INT-250): ratatui + crossterm, ConditionalEventHandler
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Healthy,
    Info,
    Warning,
    Critical,
}
pub struct Check {
    #[allow(dead_code)]
    pub name: String,
    pub detail: String,
    pub severity: Severity,
}
pub struct Section {
    #[allow(dead_code)]
    pub name: String,
    pub icon: String,
    pub checks: Vec<Check>,
}
impl Section {
    pub fn worst(&self) -> &Severity {
        if self.checks.iter().any(|c| c.severity == Severity::Critical) {
            &Severity::Critical
        } else if self.checks.iter().any(|c| c.severity == Severity::Warning) {
            &Severity::Warning
        } else if self.checks.iter().any(|c| c.severity == Severity::Info) {
            &Severity::Info
        } else {
            &Severity::Healthy
        }
    }
    pub fn summary_strip(&self) -> String {
        self.checks.iter().map(|c| match c.severity {
            Severity::Healthy => "✅",
            Severity::Info => "🔵",
            Severity::Warning => "⚠️ ",
            Severity::Critical => "🔴",
        }).collect::<Vec<_>>().join("")
    }
}
fn severity_color(s: &Severity) -> Color {
    match s {
        Severity::Healthy => Color::Rgb(107, 227, 163),  // forest green
        Severity::Info    => Color::Rgb(92, 200, 255),   // blue
        Severity::Warning => Color::Rgb(245, 193, 119),  // amber
        Severity::Critical => Color::Rgb(230, 126, 128), // red
    }
}
fn parse_doctor_output(raw: &str) -> (Vec<Section>, String, String, u8) {
    let mut sections: Vec<Section> = Vec::new();
    let mut current_section: Option<Section> = None;
    let mut friday_line = String::new();
    let mut forecast_line = String::new();
    let mut health_pct: u8 = 100;
    for line in raw.lines() {
        let stripped = strip_ansi(line);
        let s = stripped.trim();
        // Health percentage
        if s.contains("Health:") && s.contains('%') {
            if let Some(pct) = extract_number(s) {
                health_pct = pct.min(100) as u8;
            }
        }
        // Friday line
        if s.contains("Friday:") {
            friday_line = s.trim_start_matches("🌲").trim().trim_start_matches("Friday:").trim().to_string();
            continue;
        }
        // Forecast line
        if s.contains("Forecast") {
            forecast_line = s.trim_start_matches("➡️").trim_start_matches("📈").trim()
                .trim_start_matches("Forecast").trim().to_string();
            continue;
        }
        // Section header: ╭─ 🖥  System  or similar
        let is_section = (s.contains("System") || s.contains("Git & Code") || s.contains("Git")
            || s.contains("Tools") || s.contains("Forest") || s.contains("Security"))
            && (s.contains("🖥") || s.contains("🌿") || s.contains("🛠")
                || s.contains("📋") || s.contains("🔒"));
        if is_section {
            if let Some(sec) = current_section.take() {
                sections.push(sec);
            }
            let (icon, name) = parse_section_header(s);
            current_section = Some(Section { name, icon, checks: Vec::new() });
            continue;
        }
        // Check line: │  ✅ or │  ⚠️ or │  ❌
        if let Some(ref mut sec) = current_section {
            // Strip leading box chars and spaces
            let check_s = s.trim_start_matches('│').trim_start_matches('|').trim();
            if check_s.starts_with("✅") {
                let detail = check_s.trim_start_matches("✅").trim().to_string();
                let name = detail.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
                sec.checks.push(Check { name, detail, severity: Severity::Healthy });
            } else if check_s.starts_with("⚠") {
                let detail = check_s.trim_start_matches("⚠️").trim_start_matches("⚠").trim().to_string();
                let name = detail.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
                sec.checks.push(Check { name, detail, severity: Severity::Warning });
            } else if check_s.starts_with("❌") {
                let detail = check_s.trim_start_matches("❌").trim().to_string();
                let name = detail.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
                sec.checks.push(Check { name, detail, severity: Severity::Critical });
            }
        }
    }
    if let Some(sec) = current_section {
        sections.push(sec);
    }
    (sections, friday_line, forecast_line, health_pct)
}
fn parse_section_header(s: &str) -> (String, String) {
    if s.contains("System") { return ("🖥 ".to_string(), "System".to_string()); }
    if s.contains("Git") { return ("🌿".to_string(), "Git & Code".to_string()); }
    if s.contains("Tools") { return ("🛠 ".to_string(), "Tools".to_string()); }
    if s.contains("Forest") { return ("📋".to_string(), "Forest".to_string()); }
    if s.contains("Security") { return ("🔒".to_string(), "Security".to_string()); }
    ("•".to_string(), s.to_string())
}
fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' { in_escape = true; continue; }
        if in_escape { if c == 'm' { in_escape = false; } continue; }
        out.push(c);
    }
    out
}
fn extract_number(s: &str) -> Option<i64> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).take(3).collect();
    digits.parse().ok()
}
pub fn run_health_tui(core_root: &str) {
    // Run core doctor once with NO_COLOR for clean parsing
    let raw = std::process::Command::new("core")
        .args(["doctor", "run"])
        .env("NO_COLOR", "1")
        .env("TERM", "dumb")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string()
            + &String::from_utf8_lossy(&o.stderr))
        .unwrap_or_else(|_| "Error: could not run core doctor".to_string());
    let (sections, friday_line, forecast_line, health_pct) = parse_doctor_output(&raw);
    // Get commits today from git log
    let commits_today: i64 = std::process::Command::new("git")
        .args(["-C", core_root, "log", "--oneline", "--since=midnight"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() as i64)
        .unwrap_or(0);
    // Count active intents from in-progress files
    let active_intents: i64 = std::fs::read_dir(format!("{}/intents/future", core_root))
        .map(|dir| {
            dir.filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        content.contains("in-progress")
                    } else { false }
                })
                .count() as i64
        })
        .unwrap_or(0);
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => {
            let _ = disable_raw_mode();
            return;
        }
    };
    let mut list_state = ListState::default();
    if !sections.is_empty() {
        list_state.select(Some(0));
    }
    let mut expanded: Option<usize> = None;
    loop {
        let _ = terminal.draw(|f| {
            draw_health_ui(f, &sections, &mut list_state, expanded,
                           &friday_line, &forecast_line, health_pct, commits_today, active_intents);
        });
        if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = event::read() {
            match (code, modifiers) {
                (KeyCode::Char('q'), _) | (KeyCode::Esc, _)
                | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    if expanded.is_none() {
                        let i = list_state.selected().unwrap_or(0);
                        let next = (i + 1).min(sections.len().saturating_sub(1));
                        list_state.select(Some(next));
                    }
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    if expanded.is_none() {
                        let i = list_state.selected().unwrap_or(0);
                        let prev = i.saturating_sub(1);
                        list_state.select(Some(prev));
                    }
                }
                (KeyCode::Enter, _) | (KeyCode::Right, _) => {
                    let sel = list_state.selected().unwrap_or(0);
                    expanded = if expanded == Some(sel) { None } else { Some(sel) };
                }
                (KeyCode::Left, _) => { expanded = None; }
                (KeyCode::Char('r'), _) => {
                    // Would re-run doctor -- for now just exit and let user re-open
                    break;
                }
                _ => {}
            }
        }
    }
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}
#[allow(clippy::too_many_arguments)]
fn draw_health_ui(
    f: &mut Frame,
    sections: &[Section],
    list_state: &mut ListState,
    expanded: Option<usize>,
    friday_line: &str,
    forecast_line: &str,
    health_pct: u8,
    commits_today: i64,
    active_intents: i64,
) {
    let area = f.area();
    // Overall border color based on worst section
    let worst = sections.iter().map(|s| s.worst()).fold(&Severity::Healthy, |acc, s| {
        match (acc, s) {
            (_, Severity::Critical) => &Severity::Critical,
            (Severity::Critical, _) => acc,
            (_, Severity::Warning) => &Severity::Warning,
            (Severity::Warning, _) => acc,
            _ => s,
        }
    });
    let border_color = severity_color(worst);
    let health_str = format!("🏥 Faelight Forest 11.9.0 -- {}% -- {}/23",
        health_pct, sections.iter().map(|s| s.checks.len()).sum::<usize>());
    let outer = Block::default()
        .title(Line::from(vec![
            Span::styled(format!(" {} ", health_str),
                Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD))
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    let inner = outer.inner(area);
    f.render_widget(outer, area);
    // Layout: sections top | info+warnings bottom | footer
    let sec_height = (sections.len() as u16 + 2).min(10);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(sec_height),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(inner);
    // Section rows
    if let Some(exp_idx) = expanded {
        draw_expanded(f, chunks[0], sections, exp_idx, list_state);
    } else {
        draw_compact(f, chunks[0], sections, list_state);
    }
    // Info panel - forecast + friday + warnings + stats
    draw_info_panel(f, chunks[1], friday_line, forecast_line, sections, health_pct, commits_today, active_intents);
    // Footer
    let footer_text = if expanded.is_some() {
        "← collapse  ↑↓ navigate  r refresh  q quit"
    } else {
        "↑↓ navigate  Enter expand  r refresh  q quit"
    };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(footer_text, Style::default().fg(Color::Rgb(119, 143, 127))),
    ]))
    .block(Block::default().borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(50, 70, 55))));
    f.render_widget(footer, chunks[2]);
}
#[allow(clippy::too_many_arguments)]
fn draw_info_panel(f: &mut Frame, area: Rect, friday_line: &str, forecast_line: &str, sections: &[Section], health_pct: u8, commits_today: i64, active_intents: i64) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    // Warnings first -- most important signal
    let warnings: Vec<(String, String)> = sections.iter().flat_map(|s| {
        s.checks.iter().filter(|c| c.severity == Severity::Warning || c.severity == Severity::Critical)
            .map(|c| (s.name.clone(), c.detail.clone()))
            .collect::<Vec<_>>()
    }).collect();
    if !warnings.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  Needs attention", Style::default().fg(Color::Rgb(245, 193, 119)).add_modifier(Modifier::BOLD)),
        ]));
        for (section, detail) in &warnings {
            // Truncate detail to avoid overflow
            let detail_short = if detail.len() > 45 { &detail[..45] } else { detail };
            lines.push(Line::from(vec![
                Span::styled("  ⚠  ", Style::default().fg(Color::Rgb(245, 193, 119))),
                Span::styled(format!("{:<14}", section), Style::default().fg(Color::Rgb(215, 224, 218))),
                Span::styled(detail_short.to_string(), Style::default().fg(Color::Rgb(180, 190, 183))),
            ]));
        }
        lines.push(Line::from(""));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  ✅ All systems healthy", Style::default().fg(Color::Rgb(107, 227, 163))),
        ]));
        lines.push(Line::from(""));
    }
    // Forecast
    if !forecast_line.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  📈 ", Style::default().fg(Color::Rgb(92, 200, 255))),
            Span::styled(forecast_line.trim().to_string(), Style::default().fg(Color::Rgb(215, 224, 218))),
        ]));
    }
    // Friday
    if !friday_line.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  🌲 ", Style::default().fg(Color::Rgb(107, 227, 163))),
            Span::styled(friday_line.trim().to_string(), Style::default().fg(Color::Rgb(215, 224, 218))),
        ]));
    }
    lines.push(Line::from(""));
    // Health bar
    let filled = (health_pct as usize * 20) / 100;
    let bar: String = "█".repeat(filled) + &"░".repeat(20 - filled);
    let bar_color = if health_pct >= 95 {
        Color::Rgb(107, 227, 163)
    } else if health_pct >= 80 {
        Color::Rgb(245, 193, 119)
    } else {
        Color::Rgb(230, 126, 128)
    };
    lines.push(Line::from(vec![
        Span::styled("  Health  ", Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(bar, Style::default().fg(bar_color)),
        Span::styled(format!("  {}%", health_pct), Style::default().fg(bar_color).add_modifier(Modifier::BOLD)),
    ]));
    // Commits today + active intents
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Commits today  ", Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(commits_today.to_string(), Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD)),
        Span::styled("   Active intents  ", Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(active_intents.to_string(), Style::default().fg(Color::Rgb(92, 200, 255)).add_modifier(Modifier::BOLD)),
    ]));
    let panel = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Rgb(40, 60, 45))));
    f.render_widget(panel, area);
}
fn draw_compact(f: &mut Frame, area: Rect, sections: &[Section], list_state: &mut ListState) {
    let items: Vec<ListItem> = sections.iter().map(|sec| {
        let pass = sec.checks.iter().filter(|c| c.severity == Severity::Healthy).count();
        let total = sec.checks.len();
        let worst_color = severity_color(sec.worst());
        let strip = sec.summary_strip();
        let line = Line::from(vec![
            Span::styled(format!("  {} ", sec.icon), Style::default().fg(worst_color)),
            Span::styled(format!("{:<12}", sec.name), Style::default().fg(Color::Rgb(215, 224, 218))),
            Span::styled(format!(" {}  ", strip), Style::default()),
            Span::styled(format!("{}/{}", pass, total), Style::default().fg(Color::Rgb(119, 143, 127))),
            Span::styled("  ▸".to_string(), Style::default().fg(Color::Rgb(80, 100, 85))),
        ]);
        ListItem::new(line)
    }).collect();
    let list = List::new(items)
        .highlight_style(Style::default()
            .bg(Color::Rgb(30, 45, 35))
            .add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, list_state);
}
fn draw_expanded(f: &mut Frame, area: Rect, sections: &[Section], idx: usize, list_state: &mut ListState) {
    if idx >= sections.len() { return; }
    let sec = &sections[idx];
    // Top: compact list (smaller)
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(sections.len() as u16 + 1), Constraint::Min(4)])
        .split(area);
    draw_compact(f, split[0], sections, list_state);
    // Bottom: expanded detail
    let checks: Vec<ListItem> = sec.checks.iter().map(|c| {
        let (sym, col) = match c.severity {
            Severity::Healthy  => ("✅", Color::Rgb(107, 227, 163)),
            Severity::Info     => ("🔵", Color::Rgb(92, 200, 255)),
            Severity::Warning  => ("⚠️ ", Color::Rgb(245, 193, 119)),
            Severity::Critical => ("🔴", Color::Rgb(230, 126, 128)),
        };
        let line = Line::from(vec![
            Span::styled(format!("  {} ", sym), Style::default().fg(col)),
            Span::styled(c.detail.clone(), Style::default().fg(Color::Rgb(215, 224, 218))),
        ]);
        ListItem::new(line)
    }).collect();
    let detail = List::new(checks)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled(format!(" {} {} ", sec.icon, sec.name),
                    Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD))
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(severity_color(sec.worst()))));
    f.render_widget(detail, split[1]);
}
