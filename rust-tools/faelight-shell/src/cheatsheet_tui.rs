// INT-260: cheat -- Cheatsheet TUI, reads from command_registry in state.db
// Ctrl+/ or 'cheat' opens a fuzzy-searchable reference for all commands/keybinds
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
use rusqlite::Connection;
use std::io;
use std::path::PathBuf;
#[derive(Debug, Clone)]
struct Entry {
    kind: String,
    name: String,
    category: String,
    description: String,
    expansion: Option<String>,
    example: Option<String>,
}
enum KindFilter {
    All,
    Builtin,
    Command,
    Keybind,
    Alias,
}
impl KindFilter {
    fn next(&self) -> Self {
        match self {
            KindFilter::All => KindFilter::Builtin,
            KindFilter::Builtin => KindFilter::Command,
            KindFilter::Command => KindFilter::Keybind,
            KindFilter::Keybind => KindFilter::Alias,
            KindFilter::Alias => KindFilter::All,
        }
    }
    fn label(&self) -> &str {
        match self {
            KindFilter::All => "all",
            KindFilter::Builtin => "builtins",
            KindFilter::Command => "commands",
            KindFilter::Keybind => "keybinds",
            KindFilter::Alias => "aliases",
        }
    }
}
/// INT-092 Phase 1a: rebuild alias + keybind rows in command_registry from live sources.
/// Delete-and-rebuild (no UNIQUE constraint; registry is a clean projection of reality).
/// Aliases come from shell_aliases; keybinds are parsed from mango config.conf.
/// Builtins/commands are left untouched here (Phase 1b).
pub struct RefreshStats {
    pub aliases: usize,
    pub keybinds: usize,
    pub removed: usize,
}

pub fn refresh_registry(conn: &Connection) -> Result<RefreshStats, rusqlite::Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;

    // How many alias/keybind rows exist before we wipe (for the "removed" stat).
    let before: i64 = conn
        .query_row(
            "SELECT count(*) FROM command_registry WHERE kind IN ('alias','keybind')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM command_registry WHERE kind IN ('alias','keybind')",
        [],
    )?;

    // --- Aliases: parse the DEPLOYED config.fsh directly (the live source of truth) ---
    // The shell_aliases table is only refreshed at fsh startup, so it can be stale at
    // deploy time. config.fsh is what the running shell actually loaded. INT-092 Phase 3.
    // Format: alias NAME = "COMMAND"   (optional trailing # comment).
    let mut aliases = 0usize;
    {
        let cfg_path = std::env::var_os("HOME").map(|h| {
            std::path::PathBuf::from(h).join(".config/faelight-shell/config.fsh")
        });
        if let Some(path) = cfg_path {
            if let Ok(text) = std::fs::read_to_string(&path) {
                let mut ins = tx.prepare(
                    "INSERT INTO command_registry
                       (kind, name, source, category, description, expansion, example, added_at, last_seen, deprecated)
                     VALUES ('alias', ?1, 'config.fsh', 'alias', ?2, ?3, NULL, ?4, ?4, 0)",
                )?;
                for line in text.lines() {
                    let line = line.trim();
                    let rest = match line.strip_prefix("alias ") {
                        Some(r) => r,
                        None => continue,
                    };
                    let eq = match rest.find('=') {
                        Some(i) => i,
                        None => continue,
                    };
                    let name = rest[..eq].trim().to_string();
                    if name.is_empty() {
                        continue;
                    }
                    let after = rest[eq + 1..].trim();
                    // Command = text inside the first quote pair (trailing # comment ignored).
                    // Unquoted fallback: up to " #" or end of line.
                    let command = if let Some(start) = after.find('"') {
                        match after[start + 1..].find('"') {
                            Some(end) => after[start + 1..start + 1 + end].to_string(),
                            None => continue, // unterminated quote -- skip malformed line
                        }
                    } else {
                        after.split(" #").next().unwrap_or(after).trim().to_string()
                    };
                    if command.is_empty() {
                        continue;
                    }
                    let desc = format!("alias -> {}", command);
                    ins.execute(rusqlite::params![name, desc, command, now])?;
                    aliases += 1;
                }
            }
        }
    }

    // --- Keybinds: parse mango config.conf `bind=` lines ---
    let mut keybinds = 0usize;
    {
        let core_root = conn_core_root(conn);
        let cfg_path = std::path::PathBuf::from(&core_root)
            .join("config/mango/.config/mango/config.conf");
        if let Ok(text) = std::fs::read_to_string(&cfg_path) {
            let mut ins = tx.prepare(
                "INSERT INTO command_registry
                   (kind, name, source, category, description, expansion, example, added_at, last_seen, deprecated)
                 VALUES ('keybind', ?1, 'mango-config', ?2, ?3, ?4, NULL, ?5, ?5, 0)",
            )?;
            for line in text.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("bind=") {
                    // rest = MODS,key,action[,args...]
                    let parts: Vec<&str> = rest.split(',').collect();
                    if parts.len() < 3 {
                        continue;
                    }
                    let mods = parts[0].trim();
                    let key = parts[1].trim();
                    let action = parts[2].trim();
                    let args: Vec<&str> = parts[3..].iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

                    // chord: SUPER+SHIFT + key  ->  "SUPER+SHIFT+Left"
                    let chord = if mods.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}+{}", mods, key)
                    };
                    // human action: "spawn alacritty -e yazi" / "view 1" / "killclient"
                    let action_str = if args.is_empty() {
                        action.to_string()
                    } else {
                        format!("{} {}", action, args.join(" "))
                    };
                    // category by action family for nicer grouping
                    let category = match action {
                        "spawn" => "launch",
                        "view" => "workspace",
                        "tag" => "tag",
                        "chvt" => "recovery",
                        _ => "window",
                    };

                    ins.execute(rusqlite::params![chord, category, action_str, action_str, now])?;
                    keybinds += 1;
                }
            }
        }
    }

    tx.commit()?;
    let removed = (before as usize).saturating_sub(aliases + keybinds);
    Ok(RefreshStats { aliases, keybinds, removed })
}

/// Resolve the repo root from the db connection's file path (…/runtime/state.db -> repo root).
fn conn_core_root(conn: &Connection) -> String {
    if let Some(p) = conn.path() {
        if let Some(runtime) = std::path::Path::new(p).parent() {
            if let Some(reporoot) = runtime.parent() {
                return reporoot.to_string_lossy().to_string();
            }
        }
    }
    // fallback
    format!("{}/0-core", std::env::var("HOME").unwrap_or_default())
}

pub fn run_cheatsheet_tui(core_root: &str) {
    let db_path = PathBuf::from(core_root).join("runtime/state.db");
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return,
    };
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
    run_loop(&mut terminal, &conn);
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}
fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, conn: &Connection) {
    let all_entries = load_entries(conn);
    let mut filter = KindFilter::All;
    let mut search = String::new();
    let mut searching = false;
    let mut list_state = ListState::default();
    if !all_entries.is_empty() {
        list_state.select(Some(0));
    }
    let mut detail_scroll: u16 = 0;
    loop {
        let filtered = filter_entries(&all_entries, &filter, &search);
        let _ = terminal.draw(|f| {
            draw_ui(
                f,
                &filtered,
                &mut list_state,
                &filter,
                &search,
                searching,
                detail_scroll,
            );
        });
        if let Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) = event::read()
        {
            if searching {
                match code {
                    KeyCode::Esc => {
                        searching = false;
                        search.clear();
                        list_state.select(Some(0));
                    }
                    KeyCode::Enter => {
                        searching = false;
                    }
                    KeyCode::Backspace => {
                        search.pop();
                        list_state.select(Some(0));
                        detail_scroll = 0;
                    }
                    KeyCode::Char(c) => {
                        search.push(c);
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
                    let filtered = filter_entries(&all_entries, &filter, &search);
                    let i = list_state.selected().unwrap_or(0);
                    list_state.select(Some((i + 1).min(filtered.len().saturating_sub(1))));
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
                }
                (KeyCode::PageDown, _) => {
                    detail_scroll = detail_scroll.saturating_add(3);
                }
                (KeyCode::PageUp, _) => {
                    detail_scroll = detail_scroll.saturating_sub(3);
                }
                _ => {}
            }
        }
    }
}
fn draw_ui(
    f: &mut Frame,
    filtered: &[Entry],
    list_state: &mut ListState,
    filter: &KindFilter,
    search: &str,
    searching: bool,
    detail_scroll: u16,
) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);
    // Header
    let search_display = if searching {
        format!("  🔍 /{}_", search)
    } else if !search.is_empty() {
        format!("  🔍 /{}", search)
    } else {
        format!("  filter: [{}]", filter.label())
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "  🌲 Forest Cheatsheet  ",
            Style::default()
                .fg(Color::Rgb(107, 227, 163))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} entries  ", filtered.len()),
            Style::default()
                .fg(Color::Rgb(215, 224, 218))
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
    // Two panes
    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);
    // Left: entry list
    let items: Vec<ListItem> = filtered
        .iter()
        .map(|e| {
            let kind_icon = match e.kind.as_str() {
                "builtin" => Span::styled("⬡ ", Style::default().fg(Color::Rgb(107, 227, 163))),
                "command" => Span::styled("▸ ", Style::default().fg(Color::Rgb(92, 200, 255))),
                "keybind" => Span::styled("⌨ ", Style::default().fg(Color::Rgb(245, 193, 119))),
                "alias" => Span::styled("~ ", Style::default().fg(Color::Rgb(180, 190, 183))),
                _ => Span::styled("· ", Style::default().fg(Color::Rgb(119, 143, 127))),
            };
            let name = Span::styled(
                format!("{:<20}", &e.name[..e.name.len().min(20)]),
                Style::default()
                    .fg(Color::Rgb(215, 224, 218))
                    .add_modifier(Modifier::BOLD),
            );
            let desc_len = 25usize;
            let desc = if e.description.len() > desc_len {
                format!("{}…", &e.description[..desc_len])
            } else {
                e.description.clone()
            };
            let desc_span = Span::styled(desc, Style::default().fg(Color::Rgb(119, 143, 127)));
            ListItem::new(Line::from(vec![kind_icon, name, desc_span]))
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .title(Line::from(vec![Span::styled(
                    " Commands & Keybinds ",
                    Style::default()
                        .fg(Color::Rgb(107, 227, 163))
                        .add_modifier(Modifier::BOLD),
                )]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(50, 80, 55))),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(25, 45, 30))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, content[0], list_state);
    // Right: detail
    let detail_lines = if let Some(idx) = list_state.selected() {
        if idx < filtered.len() {
            render_detail(&filtered[idx])
        } else {
            vec![]
        }
    } else {
        vec![Line::from(Span::styled(
            "  Select an entry",
            Style::default().fg(Color::Rgb(119, 143, 127)),
        ))]
    };
    let detail_title = if let Some(idx) = list_state.selected() {
        if idx < filtered.len() {
            format!(" {} -- {} ", filtered[idx].name, filtered[idx].kind)
        } else {
            " Detail ".to_string()
        }
    } else {
        " Detail ".to_string()
    };
    let detail = Paragraph::new(detail_lines)
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
                .border_style(Style::default().fg(Color::Rgb(50, 80, 55))),
        );
    f.render_widget(detail, content[1]);
    // Footer
    let footer_text = if searching {
        "  Enter confirm  Esc cancel"
    } else {
        "  ↑↓/jk navigate  Tab filter  / search  PgUp/Dn scroll  q quit"
    };
    let footer = Paragraph::new(Line::from(vec![Span::styled(
        footer_text,
        Style::default().fg(Color::Rgb(119, 143, 127)),
    )]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Rgb(50, 70, 55))),
    );
    f.render_widget(footer, chunks[2]);
}
fn render_detail(entry: &Entry) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  Name     ",
            Style::default().fg(Color::Rgb(119, 143, 127)),
        ),
        Span::styled(
            entry.name.clone(),
            Style::default()
                .fg(Color::Rgb(107, 227, 163))
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  Kind     ",
            Style::default().fg(Color::Rgb(119, 143, 127)),
        ),
        Span::styled(
            entry.kind.clone(),
            Style::default().fg(Color::Rgb(92, 200, 255)),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  Category ",
            Style::default().fg(Color::Rgb(119, 143, 127)),
        ),
        Span::styled(
            entry.category.clone(),
            Style::default().fg(Color::Rgb(180, 190, 183)),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Description",
        Style::default()
            .fg(Color::Rgb(119, 143, 127))
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("  {}", entry.description),
        Style::default().fg(Color::Rgb(215, 224, 218)),
    )]));
    if let Some(exp) = &entry.expansion {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "  Expands to  ",
                Style::default().fg(Color::Rgb(119, 143, 127)),
            ),
            Span::styled(exp.clone(), Style::default().fg(Color::Rgb(245, 193, 119))),
        ]));
    }
    if let Some(ex) = &entry.example {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "  Example     ",
                Style::default().fg(Color::Rgb(119, 143, 127)),
            ),
            Span::styled(ex.clone(), Style::default().fg(Color::Rgb(107, 227, 163))),
        ]));
    }
    lines
}
fn load_entries(conn: &Connection) -> Vec<Entry> {
    let mut stmt = match conn.prepare(
        "SELECT kind, name, COALESCE(category,''), COALESCE(description,''), expansion, example
         FROM command_registry WHERE deprecated=0 ORDER BY kind, category, name",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map([], |row| {
        Ok(Entry {
            kind: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            description: row.get(3)?,
            expansion: row.get(4)?,
            example: row.get(5)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}
fn filter_entries(entries: &[Entry], filter: &KindFilter, query: &str) -> Vec<Entry> {
    entries
        .iter()
        .filter(|e| {
            let kind_match = match filter {
                KindFilter::All => true,
                KindFilter::Builtin => e.kind == "builtin",
                KindFilter::Command => e.kind == "command",
                KindFilter::Keybind => e.kind == "keybind",
                KindFilter::Alias => e.kind == "alias",
            };
            let query_match = if query.is_empty() {
                true
            } else {
                let q = query.to_lowercase();
                e.name.to_lowercase().contains(&q)
                    || e.description.to_lowercase().contains(&q)
                    || e.category.to_lowercase().contains(&q)
                    || e.expansion
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
            };
            kind_match && query_match
        })
        .cloned()
        .collect()
}
