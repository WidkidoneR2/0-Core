//! faelight-browser v0.4.0 — w3m-style inline link navigation

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

use faelight_browser::security::SecurityStatus;
use faelight_browser::storage::BookmarkStore;

// ── Faelight Forest palette ───────────────────────────────────────────────────
const BG: Color = Color::Rgb(17, 20, 15);
const BG_SEL: Color = Color::Rgb(45, 52, 38);
const FG: Color = Color::Rgb(218, 224, 215);
const GREEN: Color = Color::Rgb(163, 227, 107);
const BLUE: Color = Color::Rgb(107, 163, 227);
const DIM: Color = Color::Rgb(90, 100, 80);
const ACCENT: Color = Color::Rgb(120, 190, 80);
const WARNING: Color = Color::Rgb(230, 180, 60);
const RED: Color = Color::Rgb(220, 80, 80);

// ── A content line — may contain an inline link ───────────────────────────────
#[derive(Clone)]
struct ContentLine {
    text: String,
    link: Option<String>, // URL if this line contains a link
}

#[derive(Clone)]
struct HistoryEntry {
    title: String,
    url: String,
    security: SecurityStatus,
}

#[derive(Clone)]
struct Tab {
    title: String,
    url: String,
    lines: Vec<ContentLine>,
    security: SecurityStatus,
    scroll: usize,
    link_cursor: usize,         // index into link positions
    link_positions: Vec<usize>, // line indices that have links
}

impl Tab {
    fn home() -> Self {
        let lines = vec![
            ContentLine {
                text: "🌲 Faelight Browser v0.4.0".to_string(),
                link: None,
            },
            ContentLine {
                text: "".to_string(),
                link: None,
            },
            ContentLine {
                text: "Security-first TUI browser — 0-Core Edition".to_string(),
                link: None,
            },
            ContentLine {
                text: "".to_string(),
                link: None,
            },
            ContentLine {
                text: "Navigation:".to_string(),
                link: None,
            },
            ContentLine {
                text: "  g / Ctrl+L   Enter address bar".to_string(),
                link: None,
            },
            ContentLine {
                text: "  Tab          Jump to next link".to_string(),
                link: None,
            },
            ContentLine {
                text: "  Shift+Tab    Jump to prev link".to_string(),
                link: None,
            },
            ContentLine {
                text: "  Enter        Open highlighted link".to_string(),
                link: None,
            },
            ContentLine {
                text: "  j/k          Scroll content".to_string(),
                link: None,
            },
            ContentLine {
                text: "  d/u          Page down/up".to_string(),
                link: None,
            },
            ContentLine {
                text: "  B            Go back".to_string(),
                link: None,
            },
            ContentLine {
                text: "  [/]          Switch tabs".to_string(),
                link: None,
            },
            ContentLine {
                text: "  t            New tab".to_string(),
                link: None,
            },
            ContentLine {
                text: "  x            Close tab".to_string(),
                link: None,
            },
            ContentLine {
                text: "  r            Reload".to_string(),
                link: None,
            },
            ContentLine {
                text: "  Ctrl+B       Bookmark page".to_string(),
                link: None,
            },
            ContentLine {
                text: "  y            Copy content".to_string(),
                link: None,
            },
            ContentLine {
                text: "  q            Quit".to_string(),
                link: None,
            },
        ];
        Tab {
            title: "Home".to_string(),
            url: "about:home".to_string(),
            lines,
            security: SecurityStatus::LocalFile,
            scroll: 0,
            link_cursor: 0,
            link_positions: vec![],
        }
    }

    fn current_link_line(&self) -> Option<usize> {
        self.link_positions.get(self.link_cursor).copied()
    }

    fn next_link(&mut self) {
        if !self.link_positions.is_empty() {
            self.link_cursor = (self.link_cursor + 1) % self.link_positions.len();
            self.scroll_to_link();
        }
    }

    fn prev_link(&mut self) {
        if !self.link_positions.is_empty() {
            if self.link_cursor == 0 {
                self.link_cursor = self.link_positions.len() - 1;
            } else {
                self.link_cursor -= 1;
            }
            self.scroll_to_link();
        }
    }

    fn scroll_to_link(&mut self) {
        if let Some(line_idx) = self.current_link_line() {
            if line_idx < self.scroll || line_idx > self.scroll + 30 {
                self.scroll = line_idx.saturating_sub(5);
            }
        }
    }

    fn open_current_link(&self) -> Option<String> {
        self.current_link_line()
            .and_then(|i| self.lines.get(i))
            .and_then(|l| l.link.clone())
    }
}

#[derive(PartialEq, Clone)]
enum Focus {
    Content,
    History,
    Bookmarks,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Search,
    EditAddress,
}

struct App {
    tabs: Vec<Tab>,
    back_stack: Vec<String>,
    history: Vec<HistoryEntry>,
    history_selected: usize,
    bookmark_store: BookmarkStore,
    bookmark_selected: usize,
    active_tab: usize,
    focus: Focus,
    mode: Mode,
    search_input: String,
    address_bar: String,
    status_message: String,
    status_color: Color,
}

impl App {
    fn new() -> Self {
        let bookmark_store = BookmarkStore::new().unwrap_or_default();
        Self {
            tabs: vec![Tab::home()],
            back_stack: vec![],
            history: vec![],
            history_selected: 0,
            bookmark_store,
            bookmark_selected: 0,
            active_tab: 0,
            focus: Focus::Content,
            mode: Mode::Normal,
            search_input: String::new(),
            address_bar: "about:home".to_string(),
            status_message: "🌲 Welcome — g to enter URL, Tab to navigate links".to_string(),
            status_color: GREEN,
        }
    }

    fn set_status(&mut self, msg: &str, color: Color) {
        self.status_message = msg.to_string();
        self.status_color = color;
    }

    fn push_history(&mut self, title: &str, url: &str, security: SecurityStatus) {
        if let Some(last) = self.history.last() {
            if last.url == url {
                return;
            }
        }
        self.history.push(HistoryEntry {
            title: title.to_string(),
            url: url.to_string(),
            security,
        });
        self.history_selected = self.history.len().saturating_sub(1);
    }

    fn fetch_url(&mut self, url: String) {
        let url = if !url.contains("://") && !url.starts_with("about:") {
            format!("https://{}", url)
        } else {
            url
        };

        let security = SecurityStatus::check(&url);
        if matches!(security, SecurityStatus::Insecure) {
            self.set_status("❌ HTTP blocked — HTTPS only", RED);
            return;
        }

        // Save current URL to back stack
        let current_url = self.tabs[self.active_tab].url.clone();
        if !current_url.starts_with("about:") {
            self.back_stack.push(current_url);
        }

        self.set_status(&format!("⏳ Loading {}...", url), WARNING);

        let (lines, meta) = if url.starts_with("about:") {
            (Tab::home().lines, vec![])
        } else {
            match fetch_web_content(&url) {
                Ok(result) => result,
                Err(e) => (
                    vec![ContentLine {
                        text: format!("❌ {}", e),
                        link: None,
                    }],
                    vec![],
                ),
            }
        };

        // Build link positions
        let link_positions: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter_map(|(i, l)| if l.link.is_some() { Some(i) } else { None })
            .collect();

        let title = meta
            .first()
            .filter(|t| !t.is_empty())
            .cloned()
            .unwrap_or_else(|| url.split('/').nth(2).unwrap_or(&url).to_string());
        self.push_history(&title, &url, security.clone());

        self.tabs[self.active_tab] = Tab {
            title: title.clone(),
            url: url.clone(),
            lines,
            security,
            scroll: 0,
            link_cursor: 0,
            link_positions,
        };
        self.address_bar = url;
        self.focus = Focus::Content;
        self.set_status("✅ Loaded — Tab to navigate links", GREEN);
    }

    fn go_back(&mut self) {
        if let Some(url) = self.back_stack.pop() {
            let security = SecurityStatus::check(&url);
            let (lines, meta) = match fetch_web_content(&url) {
                Ok(r) => r,
                Err(e) => (
                    vec![ContentLine {
                        text: format!("❌ {}", e),
                        link: None,
                    }],
                    vec![],
                ),
            };
            let link_positions: Vec<usize> = lines
                .iter()
                .enumerate()
                .filter_map(|(i, l)| if l.link.is_some() { Some(i) } else { None })
                .collect();
            let title = meta
                .first()
                .filter(|t| !t.is_empty())
                .cloned()
                .unwrap_or_else(|| url.split('/').nth(2).unwrap_or(&url).to_string());
            self.tabs[self.active_tab] = Tab {
                title,
                url: url.clone(),
                lines,
                security,
                scroll: 0,
                link_cursor: 0,
                link_positions,
            };
            self.address_bar = url;
            self.set_status("◀ Back", GREEN);
        } else {
            self.set_status("No history to go back to", DIM);
        }
    }

    fn scroll_down(&mut self) {
        let tab = &mut self.tabs[self.active_tab];
        if tab.scroll < tab.lines.len().saturating_sub(1) {
            tab.scroll += 1;
        }
    }

    fn scroll_up(&mut self) {
        let tab = &mut self.tabs[self.active_tab];
        if tab.scroll > 0 {
            tab.scroll -= 1;
        }
    }

    fn page_down(&mut self) {
        for _ in 0..20 {
            self.scroll_down();
        }
    }
    fn page_up(&mut self) {
        for _ in 0..20 {
            self.scroll_up();
        }
    }

    fn navigate_down(&mut self) {
        match self.focus {
            Focus::History => {
                if self.history_selected < self.history.len().saturating_sub(1) {
                    self.history_selected += 1;
                }
            }
            Focus::Bookmarks => {
                if self.bookmark_selected < self.bookmark_store.list().len().saturating_sub(1) {
                    self.bookmark_selected += 1;
                }
            }
            Focus::Content => self.scroll_down(),
        }
    }

    fn navigate_up(&mut self) {
        match self.focus {
            Focus::History => {
                if self.history_selected > 0 {
                    self.history_selected -= 1;
                }
            }
            Focus::Bookmarks => {
                if self.bookmark_selected > 0 {
                    self.bookmark_selected -= 1;
                }
            }
            Focus::Content => self.scroll_up(),
        }
    }

    fn enter_selected(&mut self) {
        match self.focus {
            Focus::Content => match self.tabs[self.active_tab].open_current_link() {
                Some(url) => {
                    self.fetch_url(url);
                }
                None => {
                    let line_idx = self.tabs[self.active_tab].current_link_line();
                    self.set_status(
                        &format!("⚠ No URL on line {:?} — link matching failed", line_idx),
                        WARNING,
                    );
                }
            },
            Focus::History => {
                if let Some(entry) = self.history.get(self.history_selected) {
                    let url = entry.url.clone();
                    self.fetch_url(url);
                }
            }
            Focus::Bookmarks => {
                if let Some(bm) = self.bookmark_store.list().get(self.bookmark_selected) {
                    let url = bm.url.clone();
                    self.fetch_url(url);
                }
            }
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Content => Focus::History,
            Focus::History => Focus::Bookmarks,
            Focus::Bookmarks => Focus::Content,
        };
        let name = match self.focus {
            Focus::Content => "Content",
            Focus::History => "History",
            Focus::Bookmarks => "Bookmarks",
        };
        self.set_status(&format!("→ {}", name), BLUE);
    }

    fn new_tab(&mut self) {
        self.tabs.push(Tab::home());
        self.active_tab = self.tabs.len() - 1;
        self.address_bar = "about:home".to_string();
    }

    fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
            self.address_bar = self.tabs[self.active_tab].url.clone();
        }
    }

    fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
        self.address_bar = self.tabs[self.active_tab].url.clone();
    }

    fn prev_tab(&mut self) {
        if self.active_tab == 0 {
            self.active_tab = self.tabs.len() - 1;
        } else {
            self.active_tab -= 1;
        }
        self.address_bar = self.tabs[self.active_tab].url.clone();
    }

    fn add_bookmark(&mut self) {
        let tab = &self.tabs[self.active_tab];
        match self
            .bookmark_store
            .add(tab.title.clone(), tab.url.clone(), vec![])
        {
            Ok(_) => self.set_status("✅ Bookmarked!", GREEN),
            Err(e) => self.set_status(&format!("❌ {}", e), RED),
        }
    }

    fn reload(&mut self) {
        let url = self.tabs[self.active_tab].url.clone();
        if !url.starts_with("about:") {
            self.fetch_url(url);
        }
    }

    fn yank_content(&mut self) {
        let text: String = self.tabs[self.active_tab]
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        use std::io::Write;
        use std::process::{Command, Stdio};
        if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            self.set_status(&format!("📋 Copied {} chars", text.len()), GREEN);
        } else {
            self.set_status("❌ wl-copy not found", RED);
        }
    }
}

fn fetch_web_content(url: &str) -> Result<(Vec<ContentLine>, Vec<String>), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("faelight-browser/0.4.0 (TUI; Linux)")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Client error: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status.as_u16()));
    }

    let html = response.text().map_err(|e| format!("Read error: {}", e))?;

    // Extract links with their anchor text
    let mut link_map: Vec<(String, String)> = vec![]; // (anchor_text, url)
    let mut rest = html.as_str();
    while let Some(pos) = rest.find("<a ") {
        rest = &rest[pos..];
        // Find href
        let href = if let Some(h) = rest.find("href=\"") {
            let after = &rest[h + 6..];
            if let Some(end) = after.find('"') {
                let url_str = &after[..end];
                if url_str.starts_with("https://") || url_str.starts_with("http://") {
                    Some(url_str.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Find anchor text
        let anchor_text = if let Some(gt) = rest.find('>') {
            let after = &rest[gt + 1..];
            if let Some(end) = after.find("</a>") {
                let raw = &after[..end];
                // Strip inner tags
                let mut clean = String::new();
                let mut in_tag = false;
                for c in raw.chars() {
                    match c {
                        '<' => in_tag = true,
                        '>' => in_tag = false,
                        _ if !in_tag => clean.push(c),
                        _ => {}
                    }
                }
                let t = clean.trim().to_string();
                if t.is_empty() {
                    None
                } else {
                    Some(t)
                }
            } else {
                None
            }
        } else {
            None
        };

        if let (Some(href_url), Some(text)) = (href, anchor_text) {
            link_map.push((text, href_url));
        }

        if let Some(end) = rest.find("</a>") {
            rest = &rest[end + 4..];
        } else {
            break;
        }
    }

    // Convert HTML to text lines
    let text = html2text::from_read(html.as_bytes(), 100);
    // Extract <title> tag
    let page_title = {
        let lower = html.to_lowercase();
        if let Some(start) = lower.find("<title>") {
            if let Some(end) = lower.find("</title>") {
                html[start + 7..end].trim().to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };

    let raw_lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();

    // Match link text to content lines
    let mut link_iter = link_map.iter();
    let mut current_link = link_iter.next();
    let mut used_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

    let lines: Vec<ContentLine> = raw_lines
        .into_iter()
        .map(|line| {
            // Try to match this line to a link anchor text
            let matched_link = if let Some((anchor, url)) = current_link {
                let anchor_trimmed = anchor.trim();
                let line_trimmed = line.trim();
                if !anchor_trimmed.is_empty()
                    && line_trimmed.contains(anchor_trimmed)
                    && !used_urls.contains(url)
                {
                    let url_clone = url.clone();
                    used_urls.insert(url_clone.clone());
                    current_link = link_iter.next();
                    Some(url_clone)
                } else {
                    None
                }
            } else {
                None
            };

            ContentLine {
                text: line,
                link: matched_link,
            }
        })
        .collect();

    let links: Vec<String> = link_map.iter().map(|(_, u)| u.clone()).collect();
    let _ = links;
    Ok((lines, vec![page_title]))
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => {
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::SHIFT)
                            {
                                if app.focus == Focus::Content {
                                    app.tabs[app.active_tab].prev_link();
                                    let n = app.tabs[app.active_tab].link_positions.len();
                                    let c = app.tabs[app.active_tab].link_cursor;
                                    app.set_status(&format!("← Link {}/{}", c + 1, n), BLUE);
                                } else {
                                    app.cycle_focus();
                                }
                            } else if app.focus == Focus::Content {
                                app.tabs[app.active_tab].next_link();
                                let n = app.tabs[app.active_tab].link_positions.len();
                                let c = app.tabs[app.active_tab].link_cursor;
                                app.set_status(&format!("→ Link {}/{}", c + 1, n), BLUE);
                            } else {
                                app.cycle_focus();
                            }
                        }
                        KeyCode::BackTab => {
                            if app.focus == Focus::Content {
                                app.tabs[app.active_tab].prev_link();
                            } else {
                                app.cycle_focus();
                            }
                        }
                        KeyCode::Char('j') => app.navigate_down(),
                        KeyCode::Char('k') => app.navigate_up(),
                        KeyCode::Char('d') => app.page_down(),
                        KeyCode::Char('u') => app.page_up(),
                        KeyCode::Enter => app.enter_selected(),
                        KeyCode::Char('B') => app.go_back(),
                        KeyCode::Char(']') => app.next_tab(),
                        KeyCode::Char('[') => app.prev_tab(),
                        KeyCode::Char('t') => app.new_tab(),
                        KeyCode::Char('x') => app.close_tab(),
                        KeyCode::Char('r') => app.reload(),
                        KeyCode::Char('y') => app.yank_content(),
                        KeyCode::Char('f') => app.cycle_focus(),
                        KeyCode::Char('g') => app.mode = Mode::EditAddress,
                        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.add_bookmark();
                        }
                        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.mode = Mode::EditAddress;
                        }
                        KeyCode::Char('/') => {
                            app.mode = Mode::Search;
                            app.search_input.clear();
                        }
                        _ => {}
                    },
                    Mode::Search => match key.code {
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                            app.search_input.clear();
                        }
                        KeyCode::Enter => {
                            let q = app.search_input.clone();
                            app.mode = Mode::Normal;
                            app.fetch_url(format!(
                                "https://search.brave.com/search?q={}",
                                q.replace(' ', "+")
                            ));
                        }
                        KeyCode::Char(c) => app.search_input.push(c),
                        KeyCode::Backspace => {
                            app.search_input.pop();
                        }
                        _ => {}
                    },
                    Mode::EditAddress => match key.code {
                        KeyCode::Esc => app.mode = Mode::Normal,
                        KeyCode::Enter => {
                            let url = app.address_bar.clone();
                            app.mode = Mode::Normal;
                            app.fetch_url(url);
                        }
                        KeyCode::Char(c) => app.address_bar.push(c),
                        KeyCode::Backspace => {
                            app.address_bar.pop();
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn focused_block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    if focused {
        Block::default()
            .title(Span::styled(
                format!(" {} ", title),
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(ACCENT))
            .style(Style::default().bg(BG))
    } else {
        Block::default()
            .title(Span::styled(
                format!(" {} ", title),
                Style::default().fg(DIM),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(DIM))
            .style(Style::default().bg(BG))
    }
}

fn ui(f: &mut Frame, app: &App) {
    f.render_widget(Block::default().style(Style::default().bg(BG)), f.area());

    // Outer: body | status bar
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    // Main: left 28% | right 72%
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(outer[0]);

    // Left: address(3) | history | bookmarks
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(main[0]);

    // ── Address bar ──────────────────────────────────────────────────────────
    let (addr_title, addr_text, addr_color) = match app.mode {
        Mode::EditAddress => ("󰏫 URL", format!("{}█", app.address_bar), WARNING),
        Mode::Search => ("󰍉 Search", format!("{}█", app.search_input), BLUE),
        Mode::Normal => ("󰖟 Address", app.address_bar.clone(), FG),
    };
    let addr = Paragraph::new(Span::styled(&addr_text, Style::default().fg(addr_color)))
        .block(focused_block(addr_title, app.mode != Mode::Normal));
    f.render_widget(addr, left[0]);

    // ── History ──────────────────────────────────────────────────────────────
    let hist_focused = app.focus == Focus::History;
    let hist_items: Vec<ListItem> = app
        .history
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let selected = i == app.history_selected;
            let style = if selected && hist_focused {
                Style::default()
                    .fg(BG)
                    .bg(GREEN)
                    .add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default().fg(GREEN).bg(BG_SEL)
            } else {
                Style::default().fg(DIM)
            };
            let label = if entry.title.len() > 20 {
                format!("{}..", &entry.title[..18])
            } else {
                entry.title.clone()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", entry.security.icon()), style),
                Span::styled(label, style),
            ]))
        })
        .collect();
    f.render_widget(
        List::new(hist_items).block(focused_block("󰋇 History", hist_focused)),
        left[1],
    );

    // ── Bookmarks ────────────────────────────────────────────────────────────
    let bm_focused = app.focus == Focus::Bookmarks;
    let bm_items: Vec<ListItem> = app
        .bookmark_store
        .list()
        .iter()
        .enumerate()
        .map(|(i, bm)| {
            let selected = i == app.bookmark_selected;
            let style = if selected && bm_focused {
                Style::default()
                    .fg(BG)
                    .bg(BLUE)
                    .add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default().fg(BLUE).bg(BG_SEL)
            } else {
                Style::default().fg(DIM)
            };
            let label = if bm.name.len() > 20 {
                format!("{}..", &bm.name[..18])
            } else {
                bm.name.clone()
            };
            ListItem::new(Line::from(vec![
                Span::styled(" ⭐ ", style),
                Span::styled(label, style),
            ]))
        })
        .collect();
    f.render_widget(
        List::new(bm_items).block(focused_block("󰃃 Bookmarks", bm_focused)),
        left[2],
    );

    // ── Content (full height, inline links) ──────────────────────────────────
    let content_focused = app.focus == Focus::Content;
    let tab = &app.tabs[app.active_tab];

    let sec_color = match tab.security {
        SecurityStatus::Secure => GREEN,
        SecurityStatus::Insecure => RED,
        SecurityStatus::LocalFile => BLUE,
        _ => DIM,
    };

    // Tab bar in title
    let tab_bar: String = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let label = if t.title.len() > 10 {
                format!("{}..", &t.title[..8])
            } else {
                t.title.clone()
            };
            if i == app.active_tab {
                format!("[{}]", label)
            } else {
                format!(" {} ", label)
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let link_count = tab.link_positions.len();
    let link_info = if link_count > 0 {
        format!("  󰌹 {}/{}", tab.link_cursor + 1, link_count)
    } else {
        String::new()
    };

    let content_title = format!("{} {}{}", tab.security.icon(), tab_bar, link_info);

    let current_link_line = tab.current_link_line();

    let content_items: Vec<ListItem> = tab
        .lines
        .iter()
        .enumerate()
        .skip(tab.scroll)
        .map(|(i, line)| {
            let is_link = line.link.is_some();
            let is_active_link = current_link_line == Some(i) && content_focused;

            let style = if is_active_link {
                Style::default()
                    .fg(BG)
                    .bg(GREEN)
                    .add_modifier(Modifier::BOLD)
            } else if is_link {
                Style::default().fg(BLUE).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(FG)
            };

            let prefix = if is_active_link { "▶ " } else { "  " };
            ListItem::new(Span::styled(format!("{}{}", prefix, line.text), style))
        })
        .collect();

    let content_block = Block::default()
        .title(Span::styled(
            format!(" {} ", content_title),
            Style::default().fg(sec_color).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if content_focused { ACCENT } else { DIM }))
        .style(Style::default().bg(BG));

    f.render_widget(List::new(content_items).block(content_block), main[1]);

    // ── Status bar (full width) ───────────────────────────────────────────────
    let focus_label = match app.focus {
        Focus::Content => "Content — Tab:next link  Enter:open  j/k:scroll  B:back",
        Focus::History => "History — j/k:navigate  Enter:open  f:switch panel",
        Focus::Bookmarks => "Bookmarks — j/k:navigate  Enter:open  f:switch panel",
    };
    let help = match app.mode {
        Mode::Normal => focus_label,
        Mode::EditAddress => "Enter:load  Esc:cancel",
        Mode::Search => "Enter:brave search  Esc:cancel",
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(&app.status_message, Style::default().fg(app.status_color)),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled(help, Style::default().fg(DIM)),
    ]))
    .block(focused_block("󰋼 Status", false));

    f.render_widget(status, outer[1]);
}
