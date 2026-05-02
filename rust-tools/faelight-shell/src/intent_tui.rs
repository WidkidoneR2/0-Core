// INT-254: it -- Intent Ledger as Ratatui TUI
// Two-pane: left=intent list (filterable), right=full intent detail
// Keys: j/k navigate, tab=cycle filter, enter=expand, e=edit, q=quit, /=search
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
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
enum StatusFilter {
    All,
    InProgress,
    Planned,
    Complete,
}
impl StatusFilter {
    fn next(&self) -> Self {
        match self {
            StatusFilter::All => StatusFilter::InProgress,
            StatusFilter::InProgress => StatusFilter::Planned,
            StatusFilter::Planned => StatusFilter::Complete,
            StatusFilter::Complete => StatusFilter::All,
        }
    }
    fn label(&self) -> &str {
        match self {
            StatusFilter::All => "all",
            StatusFilter::InProgress => "in-progress",
            StatusFilter::Planned => "planned",
            StatusFilter::Complete => "complete",
        }
    }
}
#[derive(Debug, Clone)]
struct Intent {
    id: String,
    title: String,
    status: String,
    date: String,
    tags: String,
    path: PathBuf,
    content: String,
}
pub fn run_intent_tui(core_root: &str) {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => { let _ = disable_raw_mode(); return; }
    };
    run_loop(&mut terminal, core_root);
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}
fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, core_root: &str) {
    let mut filter = StatusFilter::All;
    let mut search_query = String::new();
    let mut searching = false;
    let mut intents = load_intents(core_root);
    let mut list_state = ListState::default();
    if !intents.is_empty() { list_state.select(Some(0)); }
    let mut detail_scroll: u16 = 0;
    loop {
        let filtered = filter_intents(&intents, &filter, &search_query);
        let _ = terminal.draw(|f| {
            draw_ui(f, &filtered, &mut list_state, &filter, &search_query,
                    searching, detail_scroll, &intents);
        });
        if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = event::read() {
            if searching {
                match code {
                    KeyCode::Esc => { searching = false; search_query.clear(); }
                    KeyCode::Enter => { searching = false; }
                    KeyCode::Backspace => { search_query.pop(); }
                    KeyCode::Char(c) => {
                        search_query.push(c);
                        list_state.select(Some(0));
                        detail_scroll = 0;
                    }
                    _ => {}
                }
                continue;
            }
            match (code, modifiers) {
                (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => return,
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => return,
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    let filtered = filter_intents(&intents, &filter, &search_query);
                    let i = list_state.selected().unwrap_or(0);
                    let next = (i + 1).min(filtered.len().saturating_sub(1));
                    list_state.select(Some(next));
                    detail_scroll = 0;
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    let i = list_state.selected().unwrap_or(0);
                    list_state.select(Some(i.saturating_sub(1)));
                    detail_scroll = 0;
                }
                (KeyCode::Tab, _) => {
                    filter = filter.next();
                    list_state.select(Some(0));
                    detail_scroll = 0;
                }
                (KeyCode::Char('/'), _) => {
                    searching = true;
                    search_query.clear();
                }
                (KeyCode::Char('r'), _) => {
                    intents = load_intents(core_root);
                    list_state.select(Some(0));
                    detail_scroll = 0;
                }
                (KeyCode::Char('e'), _) => {
                    let filtered = filter_intents(&intents, &filter, &search_query);
                    if let Some(idx) = list_state.selected() {
                        if idx < filtered.len() {
                            let path = filtered[idx].path.to_string_lossy().to_string();
                            let _ = execute!(io::stdout(), LeaveAlternateScreen);
                            let _ = disable_raw_mode();
                            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
                            let _ = std::process::Command::new(&editor).arg(&path).status();
                            let _ = enable_raw_mode();
                            let _ = execute!(io::stdout(), EnterAlternateScreen);
                            intents = load_intents(core_root);
                        }
                    }
                }
                (KeyCode::PageDown, _) => { detail_scroll = detail_scroll.saturating_add(5); }
                (KeyCode::PageUp, _) => { detail_scroll = detail_scroll.saturating_sub(5); }
                _ => {}
            }
        }
    }
}
fn draw_ui(
    f: &mut Frame,
    filtered: &[Intent],
    list_state: &mut ListState,
    filter: &StatusFilter,
    search_query: &str,
    searching: bool,
    detail_scroll: u16,
    _all: &[Intent],
) {
    let area = f.area();
    // Main layout: header | content | footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    // Header
    let in_prog = filtered.iter().filter(|i| i.status == "in-progress").count();
    let planned = filtered.iter().filter(|i| i.status == "planned").count();
    let complete = filtered.iter().filter(|i| i.status == "complete").count();
    let search_display = if searching {
        format!("  🔍 /{}", search_query)
    } else if !search_query.is_empty() {
        format!("  🔍 /{}", search_query)
    } else {
        format!("  filter: [{}]", filter.label())
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled("  🌲 Intent Ledger  ", Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}  ", filtered.len()), Style::default().fg(Color::Rgb(215, 224, 218)).add_modifier(Modifier::BOLD)),
        Span::styled("in-progress ", Style::default().fg(Color::Rgb(92, 200, 255))),
        Span::styled(format!("{}  ", in_prog), Style::default().fg(Color::Rgb(92, 200, 255)).add_modifier(Modifier::BOLD)),
        Span::styled("planned ", Style::default().fg(Color::Rgb(245, 193, 119))),
        Span::styled(format!("{}  ", planned), Style::default().fg(Color::Rgb(245, 193, 119)).add_modifier(Modifier::BOLD)),
        Span::styled("complete ", Style::default().fg(Color::Rgb(107, 227, 163))),
        Span::styled(format!("{}  ", complete), Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD)),
        Span::styled(search_display, Style::default().fg(Color::Rgb(180, 190, 183))),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(50, 80, 55))));
    f.render_widget(header, chunks[0]);
    // Two-pane content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(chunks[1]);
    // Left: intent list
    let items: Vec<ListItem> = filtered.iter().map(|intent| {
        let status_icon = match intent.status.as_str() {
            "in-progress" => Span::styled("▶ ", Style::default().fg(Color::Rgb(92, 200, 255))),
            "planned"     => Span::styled("○ ", Style::default().fg(Color::Rgb(245, 193, 119))),
            "complete"    => Span::styled("✓ ", Style::default().fg(Color::Rgb(107, 227, 163))),
            _             => Span::styled("· ", Style::default().fg(Color::Rgb(119, 143, 127))),
        };
        let id_span = Span::styled(
            format!("{:>3} ", intent.id),
            Style::default().fg(Color::Rgb(119, 143, 127))
        );
        // Truncate title to fit pane
        let max_title = 32usize;
        let title = if intent.title.len() > max_title {
            format!("{}…", &intent.title[..max_title])
        } else {
            intent.title.clone()
        };
        let title_span = Span::styled(title, Style::default().fg(Color::Rgb(215, 224, 218)));
        ListItem::new(Line::from(vec![status_icon, id_span, title_span]))
    }).collect();
    let list = List::new(items)
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled(" Intents ", Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD))
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(50, 80, 55))))
        .highlight_style(Style::default()
            .bg(Color::Rgb(25, 45, 30))
            .add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, content_chunks[0], list_state);
    // Right: intent detail
    let detail_text = if let Some(idx) = list_state.selected() {
        if idx < filtered.len() {
            render_intent_detail(&filtered[idx])
        } else {
            vec![Line::from("")]
        }
    } else {
        vec![Line::from(Span::styled(
            "  Select an intent with ↑↓",
            Style::default().fg(Color::Rgb(119, 143, 127))
        ))]
    };
    let detail_title = if let Some(idx) = list_state.selected() {
        if idx < filtered.len() {
            format!(" {} -- {} ", filtered[idx].id, &filtered[idx].status)
        } else {
            " Detail ".to_string()
        }
    } else {
        " Detail ".to_string()
    };
    let detail = Paragraph::new(detail_text)
        .scroll((detail_scroll, 0))
        .wrap(Wrap { trim: false })
        .block(Block::default()
            .title(Line::from(vec![
                Span::styled(detail_title, Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD))
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(50, 80, 55))));
    f.render_widget(detail, content_chunks[1]);
    // Footer
    let footer_text = if searching {
        "Enter confirm  Esc cancel"
    } else {
        "↑↓/jk navigate  Tab filter  / search  e edit  r refresh  PgUp/Dn scroll  q quit"
    };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(format!("  {}", footer_text), Style::default().fg(Color::Rgb(119, 143, 127))),
    ]))
    .block(Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(50, 70, 55))));
    f.render_widget(footer, chunks[2]);
}
fn render_intent_detail(intent: &Intent) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    // Metadata header
    lines.push(Line::from(vec![
        Span::styled("  ID     ", Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(intent.id.clone(), Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Status ", Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(intent.status.clone(), Style::default().fg(match intent.status.as_str() {
            "in-progress" => Color::Rgb(92, 200, 255),
            "planned" => Color::Rgb(245, 193, 119),
            "complete" => Color::Rgb(107, 227, 163),
            _ => Color::Rgb(180, 190, 183),
        })),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Date   ", Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(intent.date.clone(), Style::default().fg(Color::Rgb(180, 190, 183))),
    ]));
    if !intent.tags.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  Tags   ", Style::default().fg(Color::Rgb(119, 143, 127))),
            Span::styled(intent.tags.clone(), Style::default().fg(Color::Rgb(180, 190, 183))),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ─────────────────────────────────────────────────",
            Style::default().fg(Color::Rgb(50, 70, 55)))
    ]));
    lines.push(Line::from(""));
    // Content -- render line by line with color coding
    for line in intent.content.lines() {
        let _stripped = line.trim_start_matches(|c: char| c == '-' || c == ' ')
            .to_string();
        let rendered = if line.starts_with("## ") {
            Line::from(vec![
                Span::styled(format!("  {}", line.trim_start_matches('#').trim()),
                    Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD))
            ])
        } else if line.starts_with("### ") {
            Line::from(vec![
                Span::styled(format!("  {}", line.trim_start_matches('#').trim()),
                    Style::default().fg(Color::Rgb(92, 200, 255)).add_modifier(Modifier::BOLD))
            ])
        } else if line.contains("✅") {
            Line::from(vec![
                Span::styled(format!("  {}", line.trim()),
                    Style::default().fg(Color::Rgb(107, 227, 163)))
            ])
        } else if line.contains("⬜") {
            Line::from(vec![
                Span::styled(format!("  {}", line.trim()),
                    Style::default().fg(Color::Rgb(180, 190, 183)))
            ])
        } else if line.contains("⚠") {
            Line::from(vec![
                Span::styled(format!("  {}", line.trim()),
                    Style::default().fg(Color::Rgb(245, 193, 119)))
            ])
        } else if line.starts_with("---") {
            Line::from(vec![
                Span::styled("  ─────────────────────────────────────────────────",
                    Style::default().fg(Color::Rgb(50, 70, 55)))
            ])
        } else if line.trim().is_empty() {
            Line::from("")
        } else {
            Line::from(vec![
                Span::styled(format!("  {}", line),
                    Style::default().fg(Color::Rgb(215, 224, 218)))
            ])
        };
        lines.push(rendered);
    }
    lines
}
fn load_intents(core_root: &str) -> Vec<Intent> {
    let mut intents: Vec<Intent> = Vec::new();
    let dirs = vec![
        format!("{}/intents/future", core_root),
        format!("{}/intents/complete", core_root),
    ];
    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() { continue; }
        let entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let fpath = entry.path();
            if fpath.extension().and_then(|e| e.to_str()) != Some("md") { continue; }
            let content = match std::fs::read_to_string(&fpath) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if let Some(intent) = parse_intent(&fpath, &content) {
                intents.push(intent);
            }
        }
    }
    // Sort: in-progress first, then planned, then complete, then by ID
    intents.sort_by(|a, b| {
        let order = |s: &str| match s {
            "in-progress" => 0,
            "planned" => 1,
            "complete" => 2,
            _ => 3,
        };
        order(&a.status).cmp(&order(&b.status))
            .then(a.id.parse::<u32>().unwrap_or(999).cmp(&b.id.parse::<u32>().unwrap_or(999)))
    });
    intents
}
fn parse_intent(path: &std::path::Path, content: &str) -> Option<Intent> {
    let mut id = String::new();
    let mut title = String::new();
    let mut status = String::new();
    let mut date = String::new();
    let mut tags = String::new();
    let mut in_frontmatter = false;
    let mut frontmatter_done = false;
    let mut body_lines: Vec<&str> = Vec::new();
    let mut dash_count = 0;
    for line in content.lines() {
        if line.trim() == "---" {
            dash_count += 1;
            if dash_count == 1 { in_frontmatter = true; continue; }
            if dash_count == 2 { in_frontmatter = false; frontmatter_done = true; continue; }
        }
        if in_frontmatter {
            if line.starts_with("id:") {
                id = line.trim_start_matches("id:").trim().to_string();
            } else if line.starts_with("title:") {
                title = line.trim_start_matches("title:")
                    .trim().trim_matches('"').to_string();
            } else if line.starts_with("status:") {
                status = line.trim_start_matches("status:").trim().to_string();
            } else if line.starts_with("date:") {
                date = line.trim_start_matches("date:").trim().to_string();
            } else if line.starts_with("tags:") {
                tags = line.trim_start_matches("tags:").trim()
                    .trim_matches('[').trim_matches(']').to_string();
            }
        } else if frontmatter_done {
            body_lines.push(line);
        }
    }
    if id.is_empty() || title.is_empty() { return None; }
    Some(Intent {
        id,
        title,
        status,
        date,
        tags,
        path: path.to_path_buf(),
        content: body_lines.join("\n"),
    })
}
fn filter_intents(intents: &[Intent], filter: &StatusFilter, query: &str) -> Vec<Intent> {
    intents.iter().filter(|i| {
        let status_match = match filter {
            StatusFilter::All => true,
            StatusFilter::InProgress => i.status == "in-progress",
            StatusFilter::Planned => i.status == "planned",
            StatusFilter::Complete => i.status == "complete",
        };
        let query_match = if query.is_empty() {
            true
        } else {
            let q = query.to_lowercase();
            i.title.to_lowercase().contains(&q)
                || i.id.contains(&q)
                || i.tags.to_lowercase().contains(&q)
                || i.content.to_lowercase().contains(&q)
        };
        status_match && query_match
    }).cloned().collect()
}
