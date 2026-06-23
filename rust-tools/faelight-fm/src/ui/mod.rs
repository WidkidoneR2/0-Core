// faelight-fm v3.1 -- broot-style ratatui rendering

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::{io, path::PathBuf};
use crate::types::{FlatNode, GitStatus, Mode, Panel};

// Forest color palette -- INT-033: neon candy semantic colors
const GREEN:      Color = Color::Rgb(57,  255, 20);  // neon green  -- active, success
const DIM_GREEN:  Color = Color::Rgb(100, 180, 100); // muted green -- intent display
const CYAN:       Color = Color::Rgb(50,  220, 255); // neon cyan   -- links, keys
const YELLOW:     Color = Color::Rgb(255, 200, 50);  // neon amber  -- warnings, dirty
const MAGENTA:    Color = Color::Rgb(180, 130, 255); // neon purple -- active intent
const GRAY:       Color = Color::Rgb(120, 140, 130); // muted gray  -- secondary text
const DIM_GRAY:   Color = Color::Rgb(70,  80,  75);  // dim gray    -- borders, dim
const WHITE:      Color = Color::Rgb(215, 224, 218); // fog white   -- primary text
const BG_SEL:     Color = Color::Rgb(22,  35,  25);  // forest night -- selection bg
const BG:         Color = Color::Rgb(8,   13,  8);   // deep forest black -- app bg (faelight-logout)

pub struct PanelState<'a> {
    pub root: &'a PathBuf,
    pub flat: &'a [FlatNode],
    pub filtered: &'a [usize],
    pub list_state: &'a mut ListState,
    pub mode: &'a Mode,
    pub active_intent: &'a str,
}

pub fn render_single(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    panel: PanelState,
    preview: &str,
    status_msg: &str,
) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();
        // INT-069: deep forest-black backdrop so neon colors pop (faelight-logout feel).
        f.render_widget(Block::default().style(Style::default().bg(BG)), size);
        // header | body | status
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(size);

        // Header
        render_header(f, chunks[0], panel.root, panel.mode, false);

        // Body: tree | preview
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Percentage(55),
            ])
            .split(chunks[1]);

        render_tree(f, body[0], panel.flat, panel.filtered, panel.list_state, true);
        render_preview(f, body[1], preview);
        render_status(f, chunks[2], panel.mode, panel.active_intent, status_msg);
    })?;
    Ok(())
}

pub fn render_dual(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    left: PanelState,
    right: PanelState,
    active_panel: &Panel,
    status_msg: &str,
) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(size);

        // Dual header
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);
        render_header(f, header_chunks[0], left.root, left.mode, *active_panel == Panel::Left);
        render_header(f, header_chunks[1], right.root, right.mode, *active_panel == Panel::Right);

        // Two tree panels
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        render_tree(f, body[0], left.flat, left.filtered, left.list_state, *active_panel == Panel::Left);
        render_tree(f, body[1], right.flat, right.filtered, right.list_state, *active_panel == Panel::Right);

        // Status spanning full width
        render_dual_status(f, chunks[2], active_panel, left.active_intent, status_msg);
    })?;
    Ok(())
}

fn render_header(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    root: &PathBuf,
    mode: &Mode,
    active: bool,
) {
    let path_str = root.display().to_string();
    let home = dirs_next::home_dir().unwrap_or_default().to_string_lossy().to_string();
    let short = path_str.replace(&home, "~");
    let filter_part = match mode {
        Mode::Filter(q) => format!("/{}", q),
        Mode::Command(c) => format!(":{}", c),
        _ => String::new(),
    };

    // INT-069: title + breadcrumb. Segments joined by a separator; current dir brightest.
    let title_style = Style::default().fg(GREEN).add_modifier(Modifier::BOLD).bg(BG);
    let dim_seg = Style::default().fg(if active { DIM_GREEN } else { DIM_GRAY }).bg(BG);
    let cur_seg = Style::default().fg(if active { CYAN } else { GRAY }).add_modifier(Modifier::BOLD).bg(BG);
    let sep_style = Style::default().fg(DIM_GRAY).bg(BG);

    let segs: Vec<&str> = short.split('/').filter(|x| !x.is_empty()).collect();
    let mut spans: Vec<Span> = vec![
        Span::styled(" 🌲 Faelight-FM ", title_style),
        Span::styled(" ", Style::default().bg(BG)),
    ];
    let start = segs.len().saturating_sub(4);
    if start > 0 {
        spans.push(Span::styled("… › ", sep_style));
    }
    for (i, seg) in segs[start..].iter().enumerate() {
        let is_last = start + i == segs.len().saturating_sub(1);
        spans.push(Span::styled(seg.to_string(), if is_last { cur_seg } else { dim_seg }));
        if !is_last {
            spans.push(Span::styled(" › ", sep_style));
        }
    }
    if !filter_part.is_empty() {
        spans.push(Span::styled(format!("   {}", filter_part),
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD).bg(BG)));
    }

    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(BG)), area);
}

fn render_tree(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    flat: &[FlatNode],
    filtered: &[usize],
    list_state: &mut ListState,
    active: bool,
) {
    let items: Vec<ListItem> = filtered.iter().enumerate().map(|(display_idx, &real_idx)| {
        let node = &flat[real_idx];
        let selected = list_state.selected().unwrap_or(0) == display_idx;

        // Tree indent lines
        let indent = if node.depth == 0 {
            String::new()
        } else {
            format!("{}{}", "│  ".repeat(node.depth.saturating_sub(1)), "├─ ")
        };

        if node.is_unlisted_marker {
            let text = format!("{}  {} unlisted", indent, node.unlisted);
            return ListItem::new(text).style(Style::default().fg(DIM_GRAY));
        }

        // Size column (right-aligned, 5 chars)
        let size_str = crate::fs::format_size(node.size);
        let size_pad = format!("{:>5} ", size_str);

        // Icon
        let icon = if node.is_symlink { "→" }
                   else if node.is_dir && flat[real_idx].is_dir { 
                       if node.depth > 0 { "▸" } else { "▸" }
                   }
                   else { " " };

        // Git badge
        let git_badge = match node.git_status {
            GitStatus::Modified  => " ✎",
            GitStatus::Untracked => " +",
            GitStatus::Staged    => " ●",
            GitStatus::Clean     => "",
        };

        // Name color -- INT-033: semantic colors for intent files
        let intent_color = if !node.is_dir && node.name.ends_with(".md") {
            let path_str = node.node_path.to_string_lossy();
            if path_str.contains("/intents/in-progress/") {
                Some(GREEN)    // neon green -- active intent
            } else if path_str.contains("/intents/future/") {
                Some(MAGENTA)  // neon purple -- planned intent
            } else if path_str.contains("/intents/complete/") {
                Some(DIM_GREEN) // muted green -- complete intent
            } else {
                None
            }
        } else {
            None
        };
        let name_color = if selected {
            GREEN
        } else if let Some(ic) = intent_color {
            ic
        } else if node.is_symlink {
            MAGENTA
        } else if node.is_dir {
            CYAN
        } else {
            match node.git_status {
                GitStatus::Modified  => YELLOW,
                GitStatus::Untracked => Color::Rgb(100, 200, 100),
                GitStatus::Staged    => CYAN,
                GitStatus::Clean     => WHITE,
            }
        };

        let git_color = match node.git_status {
            GitStatus::Modified  => YELLOW,
            GitStatus::Untracked => Color::Rgb(100, 200, 100),
            GitStatus::Staged    => CYAN,
            GitStatus::Clean     => GRAY,
        };

        let bg = if selected && active { BG_SEL } else { BG };  // INT-069: forest-black, not terminal grey

        let name_style = Style::default().fg(name_color).bg(bg)
            .add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() });

        let size_style = Style::default().fg(DIM_GRAY).bg(bg);
        let indent_style = Style::default().fg(DIM_GRAY).bg(bg);
        let git_style = Style::default().fg(git_color).bg(bg);

        ListItem::new(Line::from(vec![
            Span::styled(size_pad, size_style),
            Span::styled(indent, indent_style),
            Span::styled(format!("{} ", icon), Style::default().fg(if node.is_dir { CYAN } else { GRAY }).bg(bg)),
            Span::styled(node.name.clone(), name_style),
            Span::styled(git_badge.to_string(), git_style),
        ]))
    }).collect();

    let border_style = if active {
        Style::default().fg(DIM_GREEN)
    } else {
        Style::default().fg(DIM_GRAY)
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::RIGHT)
            .border_style(border_style));
    f.render_stateful_widget(list, area, list_state);
}

fn render_preview(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    preview: &str,
) {
    let widget = Paragraph::new(preview.to_string())
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(GRAY));
    f.render_widget(widget, area);
}

fn render_status(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    mode: &Mode,
    active_intent: &str,
    status_msg: &str,
) {
    use ratatui::text::Span;
    let width = area.width as usize;

    if !status_msg.is_empty() {
        let truncated = if status_msg.len() > width.saturating_sub(2) {
            format!("  {}…", &status_msg[..width.saturating_sub(4)])
        } else {
            format!("  {}", status_msg)
        };
        f.render_widget(
            Paragraph::new(truncated).style(Style::default().fg(GREEN)),
            area,
        );
        return;
    }

    let line = match mode {
        Mode::Filter(q) => Line::from(vec![
            Span::styled(format!("  /{}", q), Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("  esc", Style::default().fg(CYAN)),
            Span::styled(" clear  ", Style::default().fg(GRAY)),
            Span::styled("enter", Style::default().fg(CYAN)),
            Span::styled(" focus", Style::default().fg(GRAY)),
        ]),
        Mode::Command(c) => Line::from(vec![
            Span::styled(format!("  :{}", c), Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled("  enter", Style::default().fg(CYAN)),
            Span::styled(" run  ", Style::default().fg(GRAY)),
            Span::styled("esc", Style::default().fg(CYAN)),
            Span::styled(" cancel", Style::default().fg(GRAY)),
        ]),
        Mode::ConfirmDelete(msg) => Line::from(vec![
            Span::styled(format!("  {}", msg), Style::default().fg(YELLOW)),
            Span::styled("  y", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("/", Style::default().fg(GRAY)),
            Span::styled("n", Style::default().fg(Color::Rgb(248,81,73)).add_modifier(Modifier::BOLD)),
        ]),
        Mode::Normal => {
            let w = area.width as usize;
            let intent_max = w.saturating_sub(60).max(10).min(25);
            let _intent_short = if active_intent.len() > intent_max {
                format!("{}…", &active_intent[..intent_max])
            } else { active_intent.to_string() };
            // Short hint set that fits most terminals
            Line::from(vec![
                Span::styled("  ", Style::default().bg(BG)),
                Span::styled("j/k", Style::default().fg(CYAN).bg(BG)),
                Span::styled(" move    ", Style::default().fg(GRAY).bg(BG)),
                Span::styled("↵", Style::default().fg(CYAN).bg(BG)),
                Span::styled(" open    ", Style::default().fg(GRAY).bg(BG)),
                Span::styled("h", Style::default().fg(CYAN).bg(BG)),
                Span::styled(" up    ", Style::default().fg(GRAY).bg(BG)),
                Span::styled("/", Style::default().fg(CYAN).bg(BG)),
                Span::styled(" filter    ", Style::default().fg(GRAY).bg(BG)),
                Span::styled(":", Style::default().fg(CYAN).bg(BG)),
                Span::styled(" cmd    ", Style::default().fg(GRAY).bg(BG)),
                Span::styled("q", Style::default().fg(CYAN).bg(BG)),
                Span::styled(" quit", Style::default().fg(GRAY).bg(BG)),
            ])
        },
    };
    f.render_widget(Paragraph::new(line), area);
}

fn render_dual_status(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    active_panel: &Panel,
    active_intent: &str,
    status_msg: &str,
) {
    let (panel_color, panel_label) = match active_panel {
        Panel::Left  => (GREEN,  "◀ left"),
        Panel::Right => (CYAN,   "right ▶"),
    };
    let _intent_short = if active_intent.len() > 25 {
        format!("{}…", &active_intent[..24])
    } else { active_intent.to_string() };

    let line1 = if !status_msg.is_empty() {
        Line::from(vec![
            Span::styled(format!("  {}", status_msg), Style::default().fg(GREEN)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  ", Style::default().fg(DIM_GRAY)),
            Span::styled("│", Style::default().fg(DIM_GRAY)),
            Span::styled(" tab", Style::default().fg(CYAN)),
            Span::styled(" switch  ", Style::default().fg(GRAY)),
            Span::styled("enter", Style::default().fg(CYAN)),
            Span::styled(" expand  ", Style::default().fg(GRAY)),
            Span::styled("s", Style::default().fg(CYAN)),
            Span::styled(" stage  ", Style::default().fg(GRAY)),
            Span::styled("y", Style::default().fg(CYAN)),
            Span::styled(" yank  ", Style::default().fg(GRAY)),
            Span::styled(":cp :mv", Style::default().fg(YELLOW)),
            Span::styled(" panels", Style::default().fg(GRAY)),
        ])
    };

    let line2 = Line::from(vec![
        Span::styled("  [", Style::default().fg(DIM_GRAY)),
        Span::styled(panel_label, Style::default().fg(panel_color).add_modifier(Modifier::BOLD)),
        Span::styled("]  ", Style::default().fg(DIM_GRAY)),
        Span::styled("j/k", Style::default().fg(CYAN)),
        Span::styled(" ↕  ", Style::default().fg(GRAY)),
        Span::styled("/", Style::default().fg(CYAN)),
        Span::styled(" filter  ", Style::default().fg(GRAY)),
        Span::styled(":", Style::default().fg(CYAN)),
        Span::styled(" cmd  ", Style::default().fg(GRAY)),
        Span::styled("n", Style::default().fg(CYAN)),
        Span::styled(" nix  ", Style::default().fg(GRAY)),
        Span::styled("r", Style::default().fg(CYAN)),
        Span::styled(" gc  ", Style::default().fg(GRAY)),
        Span::styled("q", Style::default().fg(CYAN)),
        Span::styled(" quit", Style::default().fg(GRAY)),
    ]);

    let text = ratatui::text::Text::from(vec![line1, line2]);
    f.render_widget(Paragraph::new(text), area);
}
