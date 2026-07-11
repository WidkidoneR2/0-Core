// faelight-glog -- TUI git-log reader (INT-139)
// Phase 1: launch, load real `git log`, scroll a commit list, quit clean.
// Architecture mirrors the cheatsheet TUI (faelight-shell/src/cheatsheet_tui.rs):
// load-once + filter-each-frame + ListState + clean raw-mode teardown.
// Data source (Phase 0 decision): shell out to `git log` (zero-dep, snappy);
// git2 (already a workspace dep) is a known upgrade path if we need typed diffs.

use std::io;
use std::process::Command;

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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

use faelight_core::theme;

/// NixOS daily-driver began 2026-06-01. Commits dated before that are Arch-era.
fn is_arch_era(date: &str) -> bool {
    // date is strict-ISO (%aI), e.g. "2026-05-31T21:22:44-05:00". Lexical compare on the
    // YYYY-MM-DD prefix is correct for ISO dates.
    date.len() >= 10 && &date[..10] < "2026-06-01"
}

/// theme (u8,u8,u8) -> ratatui Color::Rgb.
fn rgb(c: (u8, u8, u8)) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

/// If a subject begins with an "INT-NNN" token, split it into (int_token, rest)
/// so the token can be colored separately. Otherwise (None, whole subject).
fn split_intent(subject: &str) -> (Option<String>, String) {
    // Match a leading "INT-<digits>" (case-insensitive on INT).
    let up = subject.to_uppercase();
    if up.starts_with("INT-") {
        // find end of the digit run after "INT-"
        let after = &subject[4..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            let token_len = 4 + digits.len();
            let token = subject[..token_len].to_string();
            let rest = subject[token_len..].to_string();
            return (Some(token), rest);
        }
    }
    (None, subject.to_string())
}

#[derive(Clone)]
// date/author/refs are parsed now but consumed in later phases:
// date+author -> Phase 2 filters, date+refs -> Phase 3 candy-neon.
#[allow(dead_code)]
struct Commit {
    hash: String,
    date: String,   // strict ISO (%aI)
    author: String,
    subject: String,
    refs: String,    // ref names (%D), may be empty
}

impl Commit {
    fn short_hash(&self) -> &str {
        let n = self.hash.len().min(8);
        &self.hash[..n]
    }
}

/// Load the full history via `git log`, unit-separated fields (\x1f) so commit
/// text can't collide with the delimiter.
fn load_commits() -> Vec<Commit> {
    let out = Command::new("git")
        .args([
            "log",
            "--pretty=format:%H%x1f%aI%x1f%an%x1f%s%x1f%D",
        ])
        .output();

    let out = match out {
        Ok(o) if o.status.success() => o.stdout,
        _ => return Vec::new(),
    };

    String::from_utf8_lossy(&out)
        .lines()
        .filter_map(|line| {
            let mut f = line.split('\u{1f}');
            Some(Commit {
                hash: f.next()?.to_string(),
                date: f.next().unwrap_or("").to_string(),
                author: f.next().unwrap_or("").to_string(),
                subject: f.next().unwrap_or("").to_string(),
                refs: f.next().unwrap_or("").to_string(),
            })
        })
        .collect()
}

/// Case-insensitive substring match over subject (covers INT-number AND keyword).
/// Empty search returns everything.
fn filter_commits<'a>(all: &'a [Commit], search: &str) -> Vec<&'a Commit> {
    if search.is_empty() {
        return all.iter().collect();
    }
    let needle = search.to_lowercase();
    all.iter()
        .filter(|c| c.subject.to_lowercase().contains(&needle))
        .collect()
}

/// Fetch full detail for one commit on demand (body + changed files).
fn load_detail(hash: &str) -> String {
    let out = std::process::Command::new("git")
        .args(["show", "--stat", "--format=%b", hash])
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => String::from("(could not load commit detail)"),
    }
}

/// Copy text to the Wayland clipboard via wl-copy. Returns true on success.
fn copy_to_clipboard(s: &str) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(s.as_bytes());
        }
        return child.wait().map(|st| st.success()).unwrap_or(false);
    }
    false
}

fn main() {
    let commits = load_commits();

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

    run_loop(&mut terminal, &commits);

    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, commits: &[Commit]) {
    let mut search = String::new();
    let mut searching = false;
    let mut list_state = ListState::default();
    if !commits.is_empty() {
        list_state.select(Some(0));
    }

    // Detail-view (toggle) state.
    let mut detail: Option<(String, String)> = None; // (hash, body+stat)
    let mut detail_scroll: u16 = 0;
    let mut status: Option<String> = None; // transient message (e.g. "copied")

    loop {
        let filtered = filter_commits(commits, &search);

        // Detail view takes over the screen when active.
        if let Some((hash, body)) = &detail {
            let _ = terminal.draw(|f| draw_detail(f, hash, body, detail_scroll, status.as_deref()));
            if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = event::read() {
                match (code, modifiers) {
                    (KeyCode::Esc, _) | (KeyCode::Char('q'), _) => { detail = None; detail_scroll = 0; status = None; }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => return,
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => detail_scroll = detail_scroll.saturating_add(1),
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => detail_scroll = detail_scroll.saturating_sub(1),
                    (KeyCode::PageDown, _) => detail_scroll = detail_scroll.saturating_add(10),
                    (KeyCode::PageUp, _) => detail_scroll = detail_scroll.saturating_sub(10),
                    (KeyCode::Char('y'), _) => {
                        status = Some(if copy_to_clipboard(hash) {
                            format!("copied {}", &hash[..hash.len().min(8)])
                        } else { "copy failed (wl-copy?)".to_string() });
                    }
                    _ => {}
                }
            }
            continue;
        }

        let _ = terminal.draw(|f| draw_ui(f, &filtered, &mut list_state, &search, searching));

        if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = event::read() {
            // Search-input mode (mirrors cheatsheet_tui): typing edits the query live.
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
                    }
                    KeyCode::Char(c) => {
                        search.push(c);
                        list_state.select(Some(0));
                    }
                    _ => {}
                }
                continue;
            }

            match (code, modifiers) {
                (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => return,
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => return,
                (KeyCode::Char('/'), _) => {
                    searching = true;
                }
                (KeyCode::Enter, _) => {
                    let filtered = filter_commits(commits, &search);
                    if let Some(i) = list_state.selected() {
                        if let Some(c) = filtered.get(i) {
                            detail = Some((c.hash.clone(), load_detail(&c.hash)));
                            detail_scroll = 0;
                            status = None;
                        }
                    }
                }
                (KeyCode::Char('y'), _) => {
                    let filtered = filter_commits(commits, &search);
                    if let Some(i) = list_state.selected() {
                        if let Some(c) = filtered.get(i) {
                            let _ = copy_to_clipboard(&c.hash);
                        }
                    }
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    let len = filter_commits(commits, &search).len();
                    let i = list_state.selected().unwrap_or(0);
                    list_state.select(Some((i + 1).min(len.saturating_sub(1))));
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    let i = list_state.selected().unwrap_or(0);
                    list_state.select(Some(i.saturating_sub(1)));
                }
                _ => {}
            }
        }
    }
}

fn draw_ui(
    f: &mut ratatui::Frame,
    commits: &[&Commit],
    list_state: &mut ListState,
    search: &str,
    searching: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    // Title bar (shows active search + filtered count)
    let mut title_spans = vec![
        Span::styled("  faelight-glog ", Style::default().fg(rgb(theme::NEON_GREEN)).add_modifier(Modifier::BOLD)),
        Span::styled(format!("· {} commits", commits.len()), Style::default().fg(rgb(theme::MUTED_GRAY))),
    ];
    if searching || !search.is_empty() {
        title_spans.push(Span::styled(
            format!("  /{}", search),
            Style::default().fg(rgb(theme::NEON_CYAN)).add_modifier(if searching { Modifier::BOLD } else { Modifier::DIM }),
        ));
    }
    let title = Paragraph::new(Line::from(title_spans));
    f.render_widget(title, chunks[0]);

    // Commit list
    let items: Vec<ListItem> = commits
        .iter()
        .map(|c| {
            let arch = is_arch_era(&c.date);
            // Era-dim: Arch-era commits render muted + [arch] marker; NixOS-era full candy-neon.
            let (int_color, subj_color, ref_color, subj_mod) = if arch {
                (rgb(theme::MUTED_GRAY), rgb(theme::MUTED_GRAY), rgb(theme::MUTED_GRAY), Modifier::DIM)
            } else {
                (rgb(theme::NEON_AMBER), rgb(theme::FOG_WHITE), rgb(theme::NEON_GREEN), Modifier::empty())
            };

            let mut spans = vec![
                Span::styled(format!("{} ", c.short_hash()), Style::default().fg(rgb(theme::MUTED_GRAY))),
            ];
            if arch {
                spans.push(Span::styled("[arch] ", Style::default().fg(rgb(theme::MUTED_GRAY)).add_modifier(Modifier::DIM)));
            }
            let (intent, rest) = split_intent(&c.subject);
            if let Some(tok) = intent {
                spans.push(Span::styled(tok, Style::default().fg(int_color).add_modifier(if arch { Modifier::DIM } else { Modifier::BOLD })));
                spans.push(Span::styled(rest, Style::default().fg(subj_color).add_modifier(subj_mod)));
            } else {
                spans.push(Span::styled(c.subject.clone(), Style::default().fg(subj_color).add_modifier(subj_mod)));
            }
            if !c.refs.is_empty() {
                spans.push(Span::styled(format!("  ({})", c.refs), Style::default().fg(ref_color)));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" history "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("› ");
    f.render_stateful_widget(list, chunks[1], list_state);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  j/k ", Style::default().fg(rgb(theme::NEON_GREEN))),
        Span::styled("scroll  ", Style::default().fg(Color::DarkGray)),
        Span::styled("/ ", Style::default().fg(rgb(theme::NEON_GREEN))),
        Span::styled("search  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc ", Style::default().fg(rgb(theme::NEON_GREEN))),
        Span::styled("clear  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q ", Style::default().fg(rgb(theme::NEON_GREEN))),
        Span::styled("quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}


fn draw_detail(
    f: &mut ratatui::Frame,
    hash: &str,
    body: &str,
    scroll: u16,
    status: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    // Header: full hash in amber + optional status (e.g. "copied") in green.
    let mut head = vec![
        Span::styled("  commit ", Style::default().fg(rgb(theme::MUTED_GRAY))),
        Span::styled(hash.to_string(), Style::default().fg(rgb(theme::NEON_AMBER)).add_modifier(Modifier::BOLD)),
    ];
    if let Some(s) = status {
        head.push(Span::styled(format!("   {}", s), Style::default().fg(rgb(theme::NEON_GREEN))));
    }
    f.render_widget(Paragraph::new(Line::from(head)), chunks[0]);

    // Body: message body + --stat, scrollable, fog-white.
    let para = Paragraph::new(body.to_string())
        .style(Style::default().fg(rgb(theme::FOG_WHITE)))
        .block(Block::default().borders(Borders::ALL).title(" detail "))
        .scroll((scroll, 0));
    f.render_widget(para, chunks[1]);

    // Footer.
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  j/k ", Style::default().fg(rgb(theme::NEON_GREEN))),
        Span::styled("scroll  ", Style::default().fg(rgb(theme::MUTED_GRAY))),
        Span::styled("y ", Style::default().fg(rgb(theme::NEON_GREEN))),
        Span::styled("copy hash  ", Style::default().fg(rgb(theme::MUTED_GRAY))),
        Span::styled("Esc ", Style::default().fg(rgb(theme::NEON_GREEN))),
        Span::styled("back", Style::default().fg(rgb(theme::MUTED_GRAY))),
    ]));
    f.render_widget(footer, chunks[2]);
}
