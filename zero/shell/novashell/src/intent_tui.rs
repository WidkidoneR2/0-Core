// INT-254: it -- Intent Ledger as Ratatui TUI
// INT-319: Tab focuses a pane. PgUp/PgDn page that pane. Home/End and the wheel scroll.
// Keys: j/k move, tab=pane, f=filter, home/end, e=edit, q=quit, /=search
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
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
}
impl StatusFilter {
    fn next(&self) -> Self {
        match self {
            StatusFilter::All => StatusFilter::InProgress,
            StatusFilter::InProgress => StatusFilter::Planned,
            StatusFilter::Planned => StatusFilter::All,
        }
    }
    fn label(&self) -> &str {
        match self {
            StatusFilter::All => "all",
            StatusFilter::InProgress => "in-progress",
            StatusFilter::Planned => "planned",
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
/// What a failed open restores: mouse capture, the alternate screen, then raw mode.
#[cfg(test)]
pub(crate) fn failed_open_steps() -> &'static [&'static str] {
    &["mouse-off", "alt-off", "raw-off"]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    List,
    Detail,
}

struct Split {
    list: Rect,
    detail: Rect,
    list_rows: usize,
}

fn pane_under(split: &Split, column: u16, row: u16) -> Option<Focus> {
    let top = split.list.y;
    let bottom = split.list.y.saturating_add(split.list.height);
    if row < top || row >= bottom {
        return None;
    }
    if column >= split.list.x && column < split.list.x.saturating_add(split.list.width) {
        Some(Focus::List)
    } else if column >= split.detail.x && column < split.detail.x.saturating_add(split.detail.width)
    {
        Some(Focus::Detail)
    } else {
        None
    }
}

fn pane_border(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Rgb(107, 227, 163))
    } else {
        Style::default().fg(Color::Rgb(50, 80, 55))
    }
}

struct ScreenGuard;
impl Drop for ScreenGuard {
    fn drop(&mut self) {
        // Quit, Terminal::new failing, and a panic all drop this guard.
        let mut stdout = io::stdout();
        let _ = execute!(stdout, DisableMouseCapture, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub fn run_intent_tui(core_root: &str) {
    let _guard = ScreenGuard;
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen, EnableMouseCapture);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => return,
    };
    run_loop(&mut terminal, core_root);
}
fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, core_root: &str) {
    let mut filter = StatusFilter::All;
    let mut search_query = String::new();
    let mut searching = false;
    let mut intents = load_intents(core_root);
    let mut list_state = ListState::default();
    if !intents.is_empty() {
        list_state.select(Some(0));
    }
    let mut detail_scroll: u16 = 0;
    let mut focus = Focus::List;
    let mut split = Split {
        list: Rect::default(),
        detail: Rect::default(),
        list_rows: 1,
    };
    loop {
        let filtered = filter_intents(&intents, &filter, &search_query);
        let _ = terminal.draw(|f| {
            split = draw_ui(
                f,
                &filtered,
                &mut list_state,
                &filter,
                &search_query,
                searching,
                detail_scroll,
                &intents,
                focus,
            );
        });
        match event::read() {
            Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) => {
                if searching {
                    match code {
                        KeyCode::Esc => {
                            searching = false;
                            search_query.clear();
                        }
                        KeyCode::Enter => {
                            searching = false;
                        }
                        KeyCode::Backspace => {
                            search_query.pop();
                        }
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
                        focus = match focus {
                            Focus::List => Focus::Detail,
                            Focus::Detail => Focus::List,
                        };
                    }
                    (KeyCode::Char('f'), _) => {
                        filter = filter.next();
                        let filtered = filter_intents(&intents, &filter, &search_query);
                        if filtered.is_empty() {
                            list_state.select(None);
                        } else {
                            list_state.select(Some(0));
                        }
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
                                let _ = execute!(
                                    io::stdout(),
                                    DisableMouseCapture,
                                    LeaveAlternateScreen
                                );
                                let _ = disable_raw_mode();
                                let editor =
                                    std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
                                let _ = std::process::Command::new(&editor).arg(&path).status();
                                let _ = enable_raw_mode();
                                let _ = execute!(
                                    io::stdout(),
                                    EnterAlternateScreen,
                                    EnableMouseCapture
                                );
                                intents = load_intents(core_root);
                            }
                        }
                    }
                    (KeyCode::PageDown, _) => match focus {
                        Focus::List => {
                            let filtered = filter_intents(&intents, &filter, &search_query);
                            apply_list_nav(
                                &mut list_state,
                                filtered.len(),
                                split.list_rows,
                                ListNav::PageDown,
                                &mut detail_scroll,
                            );
                        }
                        Focus::Detail => {
                            detail_scroll = detail_scroll.saturating_add(5);
                        }
                    },
                    (KeyCode::PageUp, _) => match focus {
                        Focus::List => {
                            let filtered = filter_intents(&intents, &filter, &search_query);
                            apply_list_nav(
                                &mut list_state,
                                filtered.len(),
                                split.list_rows,
                                ListNav::PageUp,
                                &mut detail_scroll,
                            );
                        }
                        Focus::Detail => {
                            detail_scroll = detail_scroll.saturating_sub(5);
                        }
                    },
                    (KeyCode::Home, _) => {
                        if focus == Focus::List {
                            let filtered = filter_intents(&intents, &filter, &search_query);
                            apply_list_nav(
                                &mut list_state,
                                filtered.len(),
                                split.list_rows,
                                ListNav::Home,
                                &mut detail_scroll,
                            );
                        }
                    }
                    (KeyCode::End, _) => {
                        if focus == Focus::List {
                            let filtered = filter_intents(&intents, &filter, &search_query);
                            apply_list_nav(
                                &mut list_state,
                                filtered.len(),
                                split.list_rows,
                                ListNav::End,
                                &mut detail_scroll,
                            );
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Mouse(mouse)) => {
                let down = match mouse.kind {
                    MouseEventKind::ScrollDown => true,
                    MouseEventKind::ScrollUp => false,
                    _ => continue,
                };
                match pane_under(&split, mouse.column, mouse.row) {
                    Some(Focus::List) => {
                        let filtered = filter_intents(&intents, &filter, &search_query);
                        let i = list_state.selected().unwrap_or(0);
                        let next = if down {
                            (i + 1).min(filtered.len().saturating_sub(1))
                        } else {
                            i.saturating_sub(1)
                        };
                        if next != i {
                            detail_scroll = 0;
                        }
                        list_state.select(Some(next));
                    }
                    Some(Focus::Detail) => {
                        detail_scroll = if down {
                            detail_scroll.saturating_add(3)
                        } else {
                            detail_scroll.saturating_sub(3)
                        };
                    }
                    None => {}
                }
            }
            _ => {}
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
    focus: Focus,
) -> Split {
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
    let in_prog = filtered
        .iter()
        .filter(|i| i.status == "in-progress")
        .count();
    let planned = filtered.iter().filter(|i| i.status == "planned").count();
    let search_display = if searching {
        format!("  🔍 /{}", search_query)
    } else if !search_query.is_empty() {
        format!("  🔍 /{}", search_query)
    } else {
        format!("  filter: [{}]", filter.label())
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "  Intent Ledger  ",
            Style::default()
                .fg(Color::Rgb(107, 227, 163))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}  ", filtered.len()),
            Style::default()
                .fg(Color::Rgb(215, 224, 218))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "in-progress ",
            Style::default().fg(Color::Rgb(92, 200, 255)),
        ),
        Span::styled(
            format!("{}  ", in_prog),
            Style::default()
                .fg(Color::Rgb(92, 200, 255))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("planned ", Style::default().fg(Color::Rgb(245, 193, 119))),
        Span::styled(
            format!("{}  ", planned),
            Style::default()
                .fg(Color::Rgb(245, 193, 119))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            search_display,
            Style::default().fg(Color::Rgb(180, 190, 183)),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(50, 80, 55))),
    );
    f.render_widget(header, chunks[0]);
    // Two-pane content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(chunks[1]);
    let list_area = content_chunks[0];
    let detail_area = content_chunks[1];
    // Left: intent list
    let items: Vec<ListItem> = filtered
        .iter()
        .map(|intent| {
            let status_icon = match intent.status.as_str() {
                "in-progress" => Span::styled("▶ ", Style::default().fg(Color::Rgb(92, 200, 255))),
                "planned" => Span::styled("○ ", Style::default().fg(Color::Rgb(245, 193, 119))),
                _ => Span::styled("· ", Style::default().fg(Color::Rgb(119, 143, 127))),
            };
            let id_span = Span::styled(
                format!("{:>3} ", intent.id),
                Style::default().fg(Color::Rgb(119, 143, 127)),
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
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .title(Line::from(vec![Span::styled(
                    " Intents ",
                    Style::default()
                        .fg(Color::Rgb(107, 227, 163))
                        .add_modifier(Modifier::BOLD),
                )]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(pane_border(focus == Focus::List)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(25, 45, 30))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, list_area, list_state);
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
            Style::default().fg(Color::Rgb(119, 143, 127)),
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
        .block(
            Block::default()
                .title(Line::from(vec![Span::styled(
                    detail_title,
                    Style::default()
                        .fg(Color::Rgb(107, 227, 163))
                        .add_modifier(Modifier::BOLD),
                )]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(pane_border(focus == Focus::Detail)),
        );
    f.render_widget(detail, detail_area);
    // Footer
    let footer_text = if searching {
        "Enter confirm  Esc cancel"
    } else {
        "Tab pane  ↑↓/jk move  PgUp/Dn page  Home/End  f filter  / search  e edit  r refresh  wheel scroll  q quit"
    };
    let footer = Paragraph::new(Line::from(vec![Span::styled(
        format!("  {}", footer_text),
        Style::default().fg(Color::Rgb(119, 143, 127)),
    )]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Rgb(50, 70, 55))),
    );
    f.render_widget(footer, chunks[2]);
    Split {
        list: list_area,
        detail: detail_area,
        list_rows: list_area.height.saturating_sub(2) as usize,
    }
}
fn render_intent_detail(intent: &Intent) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    // Metadata header
    lines.push(Line::from(vec![
        Span::styled("  ID     ", Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(
            intent.id.clone(),
            Style::default()
                .fg(Color::Rgb(107, 227, 163))
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Status ", Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(
            intent.status.clone(),
            Style::default().fg(match intent.status.as_str() {
                "in-progress" => Color::Rgb(92, 200, 255),
                "planned" => Color::Rgb(245, 193, 119),
                _ => Color::Rgb(180, 190, 183),
            }),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Date   ", Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(
            intent.date.clone(),
            Style::default().fg(Color::Rgb(180, 190, 183)),
        ),
    ]));
    if !intent.tags.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  Tags   ", Style::default().fg(Color::Rgb(119, 143, 127))),
            Span::styled(
                intent.tags.clone(),
                Style::default().fg(Color::Rgb(180, 190, 183)),
            ),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  ─────────────────────────────────────────────────",
        Style::default().fg(Color::Rgb(50, 70, 55)),
    )]));
    lines.push(Line::from(""));
    // Content -- render line by line with color coding
    for line in intent.content.lines() {
        let _stripped = line
            .trim_start_matches(|c: char| c == '-' || c == ' ')
            .to_string();
        let rendered = if line.starts_with("## ") {
            Line::from(vec![Span::styled(
                format!("  {}", line.trim_start_matches('#').trim()),
                Style::default()
                    .fg(Color::Rgb(107, 227, 163))
                    .add_modifier(Modifier::BOLD),
            )])
        } else if line.starts_with("### ") {
            Line::from(vec![Span::styled(
                format!("  {}", line.trim_start_matches('#').trim()),
                Style::default()
                    .fg(Color::Rgb(92, 200, 255))
                    .add_modifier(Modifier::BOLD),
            )])
        } else if line.contains("✅") {
            Line::from(vec![Span::styled(
                format!("  {}", line.trim()),
                Style::default().fg(Color::Rgb(107, 227, 163)),
            )])
        } else if line.contains("⬜") {
            Line::from(vec![Span::styled(
                format!("  {}", line.trim()),
                Style::default().fg(Color::Rgb(180, 190, 183)),
            )])
        } else if line.contains("⚠") {
            Line::from(vec![Span::styled(
                format!("  {}", line.trim()),
                Style::default().fg(Color::Rgb(245, 193, 119)),
            )])
        } else if line.starts_with("---") {
            Line::from(vec![Span::styled(
                "  ─────────────────────────────────────────────────",
                Style::default().fg(Color::Rgb(50, 70, 55)),
            )])
        } else if line.trim().is_empty() {
            Line::from("")
        } else {
            Line::from(vec![Span::styled(
                format!("  {}", line),
                Style::default().fg(Color::Rgb(215, 224, 218)),
            )])
        };
        lines.push(rendered);
    }
    lines
}
fn front_field(content: &str, key: &str) -> String {
    let fm = content.split("---").nth(1).unwrap_or("");
    fm.lines()
        .find(|line| line.trim_start().starts_with(&format!("{key}:")))
        .map(|line| {
            line.split_once(':')
                .map(|(_, value)| value)
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .trim_matches(|c| c == '[' || c == ']')
                .to_string()
        })
        .unwrap_or_default()
}

fn body_after_frontmatter(content: &str) -> String {
    let mut seen = 0;
    let mut body = Vec::new();
    for line in content.lines() {
        if line.trim() == "---" {
            seen += 1;
            if seen == 2 {
                continue;
            }
            if seen > 2 {
                body.push(line);
            }
            continue;
        }
        if seen >= 2 {
            body.push(line);
        }
    }
    body.join("\n")
}

fn load_intents(_core_root: &str) -> Vec<Intent> {
    let mut intents: Vec<Intent> = Vec::new();
    // Same shelf walk as intent::read_open. Planned and in-progress only.
    // Records stay out. Status, id, and title come from the ledger, not a second parse.
    if crate::core_integration::intents_root().is_none() {
        return intents;
    }
    for file in zero_core::ledger::load() {
        if file.is_record() {
            continue;
        }
        if !matches!(file.status.as_str(), "planned" | "in-progress") {
            continue;
        }
        intents.push(Intent {
            id: file.id,
            title: file.title,
            status: file.status,
            date: front_field(&file.content, "date"),
            tags: front_field(&file.content, "tags"),
            path: file.path,
            content: body_after_frontmatter(&file.content),
        });
    }
    // Sort: in-progress first, then planned, then by ID
    intents.sort_by(|a, b| {
        let order = |s: &str| match s {
            "in-progress" => 0,
            "planned" => 1,
            _ => 2,
        };
        order(&a.status).cmp(&order(&b.status)).then(
            a.id.parse::<u32>()
                .unwrap_or(999)
                .cmp(&b.id.parse::<u32>().unwrap_or(999)),
        )
    });
    intents
}
enum ListNav {
    PageDown,
    PageUp,
    Home,
    End,
}

fn list_nav(len: usize, height: usize, selected: usize, action: ListNav) -> usize {
    if len == 0 {
        return 0;
    }
    let last = len - 1;
    let selected = selected.min(last);
    let step = height.max(1);
    match action {
        ListNav::PageDown => selected.saturating_add(step).min(last),
        ListNav::PageUp => selected.saturating_sub(step),
        ListNav::Home => 0,
        ListNav::End => last,
    }
}

fn apply_list_nav(
    list_state: &mut ListState,
    len: usize,
    rows: usize,
    action: ListNav,
    detail_scroll: &mut u16,
) {
    let selected = list_state.selected().unwrap_or(0);
    let next = list_nav(len, rows, selected, action);
    if next != selected {
        *detail_scroll = 0;
    }
    list_state.select(Some(next));
}

fn filter_intents(intents: &[Intent], filter: &StatusFilter, query: &str) -> Vec<Intent> {
    intents
        .iter()
        .filter(|i| {
            let status_match = match filter {
                StatusFilter::All => true,
                StatusFilter::InProgress => i.status == "in-progress",
                StatusFilter::Planned => i.status == "planned",
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
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{list_nav, load_intents, ListNav, StatusFilter};

    #[test]
    fn list_nav_page_down_moves_by_height_and_clamps() {
        assert_eq!(list_nav(20, 5, 0, ListNav::PageDown), 5);
        assert_eq!(list_nav(20, 5, 16, ListNav::PageDown), 19);
        assert_eq!(list_nav(20, 1, 3, ListNav::PageDown), 4);
    }

    #[test]
    fn list_nav_page_up_clamps_at_zero() {
        assert_eq!(list_nav(20, 5, 12, ListNav::PageUp), 7);
        assert_eq!(list_nav(20, 5, 3, ListNav::PageUp), 0);
        assert_eq!(list_nav(20, 5, 0, ListNav::PageUp), 0);
    }

    #[test]
    fn list_nav_home_is_zero() {
        assert_eq!(list_nav(20, 8, 11, ListNav::Home), 0);
    }

    #[test]
    fn list_nav_end_is_last() {
        assert_eq!(list_nav(20, 8, 2, ListNav::End), 19);
    }

    #[test]
    fn list_nav_empty_len_returns_zero() {
        assert_eq!(list_nav(0, 4, 7, ListNav::PageDown), 0);
        assert_eq!(list_nav(0, 4, 7, ListNav::PageUp), 0);
        assert_eq!(list_nav(0, 4, 7, ListNav::Home), 0);
        assert_eq!(list_nav(0, 4, 7, ListNav::End), 0);
        assert_eq!(list_nav(0, 0, 1, ListNav::End), 0);
    }

    #[test]
    fn list_nav_height_zero_moves_by_one() {
        assert_eq!(list_nav(10, 0, 4, ListNav::PageDown), 5);
        assert_eq!(list_nav(10, 0, 4, ListNav::PageUp), 3);
        assert_eq!(list_nav(10, 0, 0, ListNav::PageUp), 0);
        assert_eq!(list_nav(10, 0, 9, ListNav::PageDown), 9);
    }

    #[test]
    fn tab_cycles_all_in_progress_and_planned() {
        let mut filter = StatusFilter::All;
        let mut labels = vec![filter.label().to_string()];
        for _ in 0..3 {
            filter = filter.next();
            labels.push(filter.label().to_string());
        }
        assert_eq!(
            labels,
            vec![
                "all".to_string(),
                "in-progress".to_string(),
                "planned".to_string(),
                "all".to_string(),
            ]
        );
    }

    #[test]
    fn a_finished_intent_is_not_on_the_screen() {
        if std::env::var_os("NSH_IT_LEDGER_CHILD").is_none() {
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "intent_tui::tests::a_finished_intent_is_not_on_the_screen",
                    "--exact",
                    "--test-threads=1",
                ])
                .env("NSH_IT_LEDGER_CHILD", "1")
                .status()
                .unwrap();
            assert!(status.success());
            return;
        }
        let dir = std::env::temp_dir().join(format!(
            "nsh-it-ledger-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let open = dir.join("0-core/zero/intents/open");
        std::fs::create_dir_all(&open).unwrap();
        std::fs::write(
            open.join("010-planned.md"),
            "---\nid: 10\nstatus: planned\ntitle: \"Stay\"\n---\n\n",
        )
        .unwrap();
        std::fs::write(
            open.join("020-active.md"),
            "---\nid: 20\nstatus: in-progress\ntitle: \"Working\"\n---\n\n",
        )
        .unwrap();
        std::fs::write(
            open.join("030-done.md"),
            "---\nid: 30\nstatus: complete\ntitle: \"Done\"\n---\n\n",
        )
        .unwrap();
        let prev = std::env::var("HOME").ok();
        // SAFETY: restored before the test returns. This process is the child.
        unsafe { std::env::set_var("HOME", &dir) };
        let shown: Vec<(String, String)> = load_intents("")
            .into_iter()
            .map(|i| (i.id, i.status))
            .collect();
        match prev {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            shown,
            vec![
                ("20".to_string(), "in-progress".to_string()),
                ("10".to_string(), "planned".to_string()),
            ]
        );
    }

    #[test]
    fn a_failed_open_leaves_the_alternate_screen() {
        assert_eq!(
            super::failed_open_steps(),
            ["mouse-off", "alt-off", "raw-off"],
            "Terminal::new failing must leave the alternate screen too"
        );
    }
}
