//! faelight-nix -- Nix package search TUI (INT-076)
//! Phase 1b: candy-neon interactive search. Search nixpkgs, browse, view detail.
//! "Find it, then let the config own it."

mod app;
mod config_edit;
mod search;
mod theme;

use app::{App, Mode};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io;

fn main() -> anyhow::Result<()> {
    // INT-076 Phase 2 scratch test: `faelight-nix --test-add <pkg> <file>`
    // runs the declarative-add engine and prints the diff. WRITES NOTHING.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("--test-add") {
        let pkg = args.get(2).cloned().unwrap_or_default();
        let path = args.get(3).cloned().unwrap_or_default();
        let content = std::fs::read_to_string(&path)?;
        match config_edit::plan_add(&content, &pkg) {
            Ok(plan) => {
                println!("--- diff preview (original file NOT modified) ---\n");
                println!("{}", plan.diff);
                // Write the full proposed result to a .preview file so the whole
                // modified config can be inspected. This reads new_content.
                let preview_path = format!("{path}.preview");
                std::fs::write(&preview_path, &plan.new_content)?;
                println!("(would add '{}')", plan.pkg);
                println!("full proposed result written to: {preview_path}");
            }
            Err(e) => println!("plan_add error: {e}"),
        }
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Editing => match key.code {
                    KeyCode::Enter => app.run_search(),
                    KeyCode::Char(c) => app.query.push(c),
                    KeyCode::Backspace => {
                        app.query.pop();
                    }
                    KeyCode::Down => {
                        if !app.results.is_empty() {
                            app.mode = Mode::Browsing;
                        }
                    }
                    KeyCode::Esc => app.should_quit = true,
                    _ => {}
                },
                Mode::Browsing => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                    KeyCode::Char('/') => app.mode = Mode::Editing,
                    KeyCode::Char('a') => app.plan_add_selected(),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev(),
                    KeyCode::Enter => app.mode = Mode::Editing,
                    _ => {}
                },
                Mode::Confirm => match key.code {
                    KeyCode::Char('y') => app.confirm_add(),
                    KeyCode::Char('n') | KeyCode::Esc => app.cancel_add(),
                    _ => {}
                },
            }
        }
        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.area();
    let bg = Style::default().bg(theme::BG);
    f.render_widget(Block::default().style(bg), size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(size);

    draw_search(f, app, chunks[0]);
    draw_body(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

fn draw_search(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let editing = app.mode == Mode::Editing;
    let border = if editing { theme::CYAN } else { theme::GRAY };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            " search nixpkgs ",
            Style::default().fg(theme::CYAN),
        ))
        .style(Style::default().bg(theme::BG));
    let caret = if editing { "_" } else { "" };
    let line = Line::from(vec![
        Span::styled(" \u{1f50d} ", Style::default().fg(theme::GREEN)),
        Span::styled(
            app.query.clone(),
            Style::default()
                .fg(theme::GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(caret, Style::default().fg(theme::GREEN)),
    ]);
    f.render_widget(Paragraph::new(line).block(block), area);
}

fn draw_body(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
    draw_results(f, app, cols[0]);
    draw_detail(f, app, cols[1]);
}

fn draw_results(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let browsing = app.mode == Mode::Browsing;
    let border = if browsing { theme::GREEN } else { theme::GRAY };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(" results ", Style::default().fg(theme::GREEN)))
        .style(Style::default().bg(theme::BG));

    let inner_w = area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|p| {
            let ver = &p.version;
            let attr = &p.attr;
            let pad = inner_w.saturating_sub(attr.len() + ver.len() + 3);
            let line = Line::from(vec![
                Span::styled(format!(" {attr}"), Style::default().fg(theme::WHITE)),
                Span::raw(" ".repeat(pad)),
                Span::styled(format!("{ver} "), Style::default().fg(theme::YELLOW)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme::BG_SEL)
                .fg(theme::GREEN)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{25b8} ");

    let mut state = app.list_state.clone();
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_detail(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::GRAY))
        .title(Span::styled(
            " detail ",
            Style::default().fg(theme::MAGENTA),
        ))
        .style(Style::default().bg(theme::BG));

    if app.mode == Mode::Confirm {
        let mut v: Vec<Line> = vec![
            Line::from(Span::styled(
                "REVIEW -- nothing written yet",
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];
        for l in app.pending_diff.lines() {
            let color = if l.trim_start().starts_with('+') {
                theme::GREEN
            } else {
                theme::WHITE
            };
            v.push(Line::from(Span::styled(
                l.to_string(),
                Style::default().fg(color),
            )));
        }
        v.push(Line::from(""));
        v.push(Line::from(Span::styled(
            "press y to write, n to cancel",
            Style::default().fg(theme::CYAN),
        )));
        f.render_widget(
            Paragraph::new(v).block(block).wrap(Wrap { trim: false }),
            area,
        );
        return;
    }

    let lines: Vec<Line> = if let Some(p) = app.selected() {
        let mut v = vec![
            Line::from(Span::styled(
                p.attr.clone(),
                Style::default()
                    .fg(theme::GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];
        v.push(Line::from(vec![
            Span::styled("pname:   ", Style::default().fg(theme::CYAN)),
            Span::styled(p.pname.clone(), Style::default().fg(theme::WHITE)),
        ]));
        v.push(Line::from(vec![
            Span::styled("version: ", Style::default().fg(theme::CYAN)),
            Span::styled(p.version.clone(), Style::default().fg(theme::YELLOW)),
        ]));
        v.push(Line::from(""));
        v.push(Line::from(Span::styled(
            p.description.clone(),
            Style::default().fg(theme::WHITE),
        )));
        v
    } else {
        vec![Line::from(Span::styled(
            "(no selection)",
            Style::default().fg(theme::GRAY),
        ))]
    };

    f.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_status(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let hint = " / search  \u{2191}\u{2193} nav  a add  q quit ";
    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", app.status),
            Style::default().fg(theme::GREEN),
        ),
        Span::styled(hint, Style::default().fg(theme::GRAY)),
    ]);
    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(theme::BG)),
        area,
    );
}
