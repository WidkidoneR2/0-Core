//! faelight-browser-tui v0.1.0
//! Dual-pane + fuzzy search + web rendering + bookmarks + security

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

use faelight_browser::storage::BookmarkStore;
use faelight_browser::security::SecurityStatus;

// EXACT colors from faelight-fm
const BG_DARK: Color = Color::Rgb(17, 20, 15);
const BG_SELECTED: Color = Color::Rgb(45, 52, 38);
const ACCENT_GREEN: Color = Color::Rgb(163, 227, 107);
const ACCENT_BLUE: Color = Color::Rgb(107, 163, 227);
const TEXT_BRIGHT: Color = Color::Rgb(218, 224, 215);
const TEXT_DIM: Color = Color::Rgb(119, 127, 111);

#[derive(Clone)]
struct Tab {
    title: String,
    url: String,
    content: Vec<String>,
    security: SecurityStatus,
}

enum Mode {
    Normal,
    Search,
    EditAddress,
}

struct App {
    tabs: Vec<Tab>,
    bookmark_store: BookmarkStore,
    active_tab: usize,
    active_bookmark: usize,
    mode: Mode,
    search_input: String,
    filtered_tabs: Vec<(usize, i64)>,
    address_bar: String,
    status_message: String,
}

impl App {
    fn new() -> Self {
        let bookmark_store = BookmarkStore::new().unwrap_or_default();
        
        let tabs = vec![
            Tab {
                title: "Home".to_string(),
                url: "about:home".to_string(),
                content: vec![
                    "🌲 Faelight Browser - 0-Core Edition".to_string(),
                    "".to_string(),
                    "Security-first, transparent web browsing".to_string(),
                    "".to_string(),
                    "Features:".to_string(),
                    "  🔒 HTTPS-only by default".to_string(),
                    "  📝 Flat-file bookmarks".to_string(),
                    "  🔍 Fuzzy search".to_string(),
                    "  📊 Dual-pane layout".to_string(),
                    "".to_string(),
                    "Keys:".to_string(),
                    "  j/k       Navigate tabs".to_string(),
                    "  J/K       Navigate bookmarks".to_string(),
                    "  /         Search tabs".to_string(),
                    "  Ctrl+L    Edit address".to_string(),
                    "  Ctrl+B    Bookmark current page".to_string(),
                    "  Ctrl+D    Delete bookmark".to_string(),
                    "  Enter     Open (address/bookmark)".to_string(),
                    "  g         Go to URL".to_string(),
                    "  q         Quit".to_string(),
                ],
                security: SecurityStatus::LocalFile,
            },
        ];
        
        Self {
            tabs: tabs.clone(),
            bookmark_store,
            active_tab: 0,
            active_bookmark: 0,
            mode: Mode::Normal,
            search_input: String::new(),
            filtered_tabs: tabs.iter().enumerate().map(|(i, _)| (i, 0)).collect(),
            address_bar: "about:home".to_string(),
            status_message: String::new(),
        }
    }
    
    fn fetch_url(&mut self, url: String) {
        // Auto-add https:// if no scheme
        let url = if !url.contains("://") && !url.starts_with("about:") {
            format!("https://{}", url)
        } else {
            url
        };
        
        self.status_message = format!("Loading {}...", url);
        
        let security = SecurityStatus::check(&url);
        
        // Only allow HTTPS or local
        if matches!(security, SecurityStatus::Insecure) {
            self.status_message = "❌ HTTP blocked - HTTPS only!".to_string();
            return;
        }
        
        let content = if url.starts_with("about:") {
            vec!["About page".to_string()]
        } else {
            // Fetch actual web content
            match fetch_web_content(&url) {
                Ok(lines) => lines,
                Err(e) => vec![
                    "Error loading page".to_string(),
                    "".to_string(),
                    format!("❌ {}", e),
                ],
            }
        };
        
        let title = url.split('/').nth(2).unwrap_or(&url).to_string();
        
        self.tabs[self.active_tab] = Tab {
            title,
            url: url.clone(),
            content,
            security,
        };
        
        self.address_bar = url;
        self.status_message = "✅ Loaded".to_string();
    }
    
    fn add_bookmark(&mut self) {
        let tab = &self.tabs[self.active_tab];
        match self.bookmark_store.add(tab.title.clone(), tab.url.clone(), vec![]) {
            Ok(_) => self.status_message = "✅ Bookmarked!".to_string(),
            Err(e) => self.status_message = format!("❌ Failed: {}", e),
        }
    }
    
    fn update_search(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_tabs = self.tabs.iter().enumerate().map(|(i, _)| (i, 0)).collect();
        } else {
            let matcher = SkimMatcherV2::default();
            let mut results: Vec<(usize, i64)> = self.tabs
                .iter()
                .enumerate()
                .filter_map(|(i, tab)| {
                    matcher.fuzzy_match(&tab.title, &self.search_input)
                        .map(|score| (i, score))
                })
                .collect();
            results.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_tabs = results;
        }
    }
    
    fn next_tab(&mut self) {
        if self.active_tab < self.tabs.len().saturating_sub(1) {
            self.active_tab += 1;
            self.update_address();
        }
    }
    
    fn prev_tab(&mut self) {
        if self.active_tab > 0 {
            self.active_tab -= 1;
            self.update_address();
        }
    }
    
    fn next_bookmark(&mut self) {
        let count = self.bookmark_store.list().len();
        if count > 0 && self.active_bookmark < count - 1 {
            self.active_bookmark += 1;
        }
    }
    
    fn prev_bookmark(&mut self) {
        if self.active_bookmark > 0 {
            self.active_bookmark -= 1;
        }
    }
    
    fn open_bookmark(&mut self) {
        if let Some(bookmark) = self.bookmark_store.list().get(self.active_bookmark) {
            self.fetch_url(bookmark.url.clone());
        }
    }
    
    fn update_address(&mut self) {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            self.address_bar = tab.url.clone();
        }
    }
}

fn fetch_web_content(url: &str) -> Result<Vec<String>, String> {
    // Use reqwest with proper headers
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) faelight-browser/0.1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Client error: {}", e))?;
    
    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;
    
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Error")));
    }
    
    let html = response.text()
        .map_err(|e| format!("Read error: {}", e))?;
    
    // Convert HTML to text
    let text = html2text::from_read(html.as_bytes(), 80);
    
    let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    
    if lines.is_empty() {
        Ok(vec!["(empty page)".to_string()])
    } else {
        Ok(lines)
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') => app.next_tab(),
                        KeyCode::Char('k') => app.prev_tab(),
                        KeyCode::Char('J') => app.next_bookmark(),
                        KeyCode::Char('K') => app.prev_bookmark(),
                        KeyCode::Char('g') => {
                            app.mode = Mode::EditAddress;
                        }
                        KeyCode::Char('/') => {
                            app.mode = Mode::Search;
                            app.search_input.clear();
                        }
                        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.mode = Mode::EditAddress;
                        }
                        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.add_bookmark();
                        }
                        KeyCode::Enter => {
                            app.open_bookmark();
                        }
                        _ => {}
                    },
                    Mode::Search => match key.code {
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                            app.search_input.clear();
                            app.update_search();
                        }
                        KeyCode::Enter => {
                            app.mode = Mode::Normal;
                            if let Some((idx, _)) = app.filtered_tabs.first() {
                                app.active_tab = *idx;
                                app.update_address();
                            }
                        }
                        KeyCode::Char(c) => {
                            app.search_input.push(c);
                            app.update_search();
                        }
                        KeyCode::Backspace => {
                            app.search_input.pop();
                            app.update_search();
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
                        KeyCode::Backspace => { app.address_bar.pop(); }
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(f.area());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(main_chunks[0]);

    // Search/Address bar
    let search_title = match app.mode {
        Mode::Search => "🔍 Search",
        Mode::EditAddress => "📝 Go to URL",
        Mode::Normal => "Navigation",
    };
    
    let search_text = match app.mode {
        Mode::Search => &app.search_input,
        Mode::EditAddress => &app.address_bar,
        Mode::Normal => "",
    };
    
    let search_style = match app.mode {
        Mode::Search | Mode::EditAddress => Style::default().fg(Color::Yellow),
        Mode::Normal => Style::default().fg(TEXT_DIM),
    };
    
    let cursor = if matches!(app.mode, Mode::Search | Mode::EditAddress) { "█" } else { "" };
    let search = Paragraph::new(format!("{}{}", search_text, cursor))
        .style(search_style.bg(BG_DARK))
        .block(Block::default().borders(Borders::ALL).title(search_title));
    f.render_widget(search, left_chunks[0]);

    // Tabs with security indicators
    let tab_items: Vec<ListItem> = app.tabs.iter().enumerate().map(|(i, tab)| {
        let selected = i == app.active_tab;
        let style = if selected {
            Style::default().fg(ACCENT_BLUE).bg(BG_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT_BRIGHT)
        };
        let security_icon = tab.security.icon();
        ListItem::new(format!("  {} {}", security_icon, tab.title)).style(style)
    }).collect();

    let tabs_list = List::new(tab_items)
        .block(Block::default().borders(Borders::ALL).title("📑 Tabs (j/k, g=go)"))
        .style(Style::default().bg(BG_DARK));
    f.render_widget(tabs_list, left_chunks[1]);

    // Bookmarks
    let bookmark_items: Vec<ListItem> = app.bookmark_store.list().iter().enumerate().map(|(i, bm)| {
        let selected = i == app.active_bookmark;
        let style = if selected {
            Style::default().fg(ACCENT_GREEN).bg(BG_SELECTED)
        } else {
            Style::default().fg(TEXT_DIM)
        };
        ListItem::new(format!("  ⭐ {}", bm.name)).style(style)
    }).collect();

    let bookmarks_list = List::new(bookmark_items)
        .block(Block::default().borders(Borders::ALL).title("🔖 Bookmarks (J/K, Enter)"))
        .style(Style::default().bg(BG_DARK));
    f.render_widget(bookmarks_list, left_chunks[2]);

    // Right pane
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(main_chunks[1]);

    // Title with security status
    let current_tab = &app.tabs[app.active_tab];
    let security_color = current_tab.security.color();
    let title_text = format!("{} {} - {}", 
        current_tab.security.icon(),
        current_tab.title,
        current_tab.url
    );

    let title = Paragraph::new(title_text)
        .style(Style::default().fg(security_color).add_modifier(Modifier::BOLD).bg(BG_DARK))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, right_chunks[0]);

    // Content
    let content_items: Vec<ListItem> = current_tab.content.iter().map(|line| {
        ListItem::new(line.as_str()).style(Style::default().fg(TEXT_BRIGHT).bg(BG_DARK))
    }).collect();

    let content_list = List::new(content_items)
        .block(Block::default().borders(Borders::ALL).title("Content"))
        .style(Style::default().bg(BG_DARK));
    f.render_widget(content_list, right_chunks[1]);

    // Status bar
    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(TEXT_DIM).bg(BG_DARK))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, right_chunks[2]);
}
