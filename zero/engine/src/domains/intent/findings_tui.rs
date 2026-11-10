//! INT-307: `findings` -- the findings screen. Same shape as cheat: ratatui,
//! crossterm, the alternate screen. The rows come from `view`, which asks the
//! one findings reader.

use super::view::{
    apply_filter, colors_enabled, load_cards, paint, Card, Guard, KeyFilter, Screen, Tone,
};
use crate::app::context::AppContext;
use crate::errors::{CoreError, CoreResult};
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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, IsTerminal, Write};

struct TermScreen;

impl Screen for TermScreen {
    fn enter_raw(&mut self) {
        let _ = enable_raw_mode();
    }
    fn leave_raw(&mut self) {
        let _ = disable_raw_mode();
    }
    fn enter_alt(&mut self) {
        let _ = execute!(io::stdout(), EnterAlternateScreen);
    }
    fn leave_alt(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn tone_style(tone: Tone, color: bool, bold: bool) -> Style {
    if !color {
        return Style::default();
    }
    let rgb = match tone {
        Tone::Fixed => Color::Rgb(57, 255, 20),
        Tone::Open => Color::Rgb(255, 80, 80),
        Tone::Unrated => Color::Rgb(255, 200, 50),
    };
    let style = Style::default().fg(rgb);
    if bold {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

/// Open the screen. A pipe prints the rows in plain text and does not take
/// the terminal. NO_COLOR keeps the screen and drops the color.
pub fn run(ctx: &AppContext) -> CoreResult<()> {
    let cards = load_cards(ctx).map_err(CoreError::Runtime)?;
    let color = colors_enabled(
        std::env::var_os("NO_COLOR").is_some(),
        io::stdout().is_terminal(),
    );
    if !io::stdout().is_terminal() {
        let mut out = io::stdout();
        for card in &cards {
            let _ = writeln!(out, "{}", paint(&card.row, false).text);
        }
        return Ok(());
    }

    let mut guard = Guard::enter(TermScreen);
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            guard.restore();
            return Err(CoreError::Runtime(format!(
                "findings: the terminal did not open ({e}). The screen was restored."
            )));
        }
    };
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_loop(&mut terminal, &cards, color);
    }));
    drop(terminal);
    guard.restore();
    if let Err(payload) = panicked {
        std::panic::resume_unwind(payload);
    }
    Ok(())
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, cards: &[Card], color: bool) {
    let mut key = KeyFilter::All;
    let mut search = String::new();
    let mut searching = false;
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut detail = false;
    let mut scroll: u16 = 0;
    loop {
        let rows: Vec<_> = cards.iter().map(|c| c.row.clone()).collect();
        let shown = apply_filter(&rows, &search, key);
        let _ = terminal.draw(|f| {
            draw(
                f,
                cards,
                &shown.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
                &mut list_state,
                key,
                &search,
                searching,
                detail,
                scroll,
                color,
            );
        });
        let Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) = event::read()
        else {
            continue;
        };
        if searching {
            match code {
                KeyCode::Esc => {
                    searching = false;
                    search.clear();
                    list_state.select(Some(0));
                }
                KeyCode::Enter => searching = false,
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
        if detail {
            match code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => detail = false,
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return,
                KeyCode::Down | KeyCode::Char('j') => scroll = scroll.saturating_add(1),
                KeyCode::Up | KeyCode::Char('k') => scroll = scroll.saturating_sub(1),
                _ => {}
            }
            continue;
        }
        match (code, modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => return,
            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => return,
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                let i = list_state.selected().unwrap_or(0);
                let n = shown.len().saturating_sub(1);
                list_state.select(Some((i + 1).min(n)));
                scroll = 0;
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                let i = list_state.selected().unwrap_or(0);
                list_state.select(Some(i.saturating_sub(1)));
                scroll = 0;
            }
            (KeyCode::Char('o'), _) => {
                key = KeyFilter::Open;
                list_state.select(Some(0));
            }
            (KeyCode::Char('h'), _) => {
                key = KeyFilter::High;
                list_state.select(Some(0));
            }
            (KeyCode::Char('a'), _) => {
                key = KeyFilter::All;
                search.clear();
                list_state.select(Some(0));
            }
            (KeyCode::Char('/'), _) => searching = true,
            (KeyCode::Enter, _) => detail = true,
            _ => {}
        }
    }
}

fn selected_id<'a>(ids: &'a [&str], state: &ListState) -> Option<&'a str> {
    ids.get(state.selected().unwrap_or(0)).copied()
}

fn draw(
    f: &mut Frame,
    cards: &[Card],
    ids: &[&str],
    list_state: &mut ListState,
    key: KeyFilter,
    search: &str,
    searching: bool,
    detail: bool,
    scroll: u16,
    color: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.area());
    let key_name = match key {
        KeyFilter::All => "all",
        KeyFilter::Open => "open",
        KeyFilter::High => "high",
    };
    let query = if searching {
        format!("/{search}_")
    } else if search.is_empty() {
        String::new()
    } else {
        format!("/{search}")
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "  findings  ",
            Style::default()
                .fg(Color::Rgb(107, 227, 163))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{}  {key_name}  {query}", ids.len())),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" F-findings "));
    f.render_widget(header, chunks[0]);

    if detail {
        let id = selected_id(ids, list_state).unwrap_or("");
        let text = cards
            .iter()
            .find(|c| c.row.id == id)
            .map(|c| c.detail.clone())
            .unwrap_or_default();
        let lines: Vec<Line> = text
            .lines()
            .skip(scroll as usize)
            .map(|l| Line::from(l.to_string()))
            .collect();
        let body = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {id}  enter/esc closes ")),
        );
        f.render_widget(body, chunks[1]);
    } else {
        let items: Vec<ListItem> = ids
            .iter()
            .filter_map(|id| cards.iter().find(|c| c.row.id == *id))
            .map(|card| {
                let style = tone_style(card.row.tone, color, card.row.high_marker && color);
                let text = card.row.columns().join("  ");
                let text = if card.row.high_marker {
                    format!("* {text}")
                } else {
                    text
                };
                ListItem::new(text).style(style)
            })
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" id  description  found  fixed  severity  state "),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        f.render_stateful_widget(list, chunks[1], list_state);
    }

    let footer = Paragraph::new(
        "  / filter    o open    h high    a all    enter finding    j/k move    q quit",
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
