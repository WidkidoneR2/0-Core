//! File preview — persistent pane, always visible
use crate::app::AppState;
use crate::ui::colors::FaelightColors;
use faelight_fm::git::GitStatus;
use faelight_zone::Zone;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render_pane(area: Rect, buf: &mut Buffer, app: &AppState) {
    let entry = match app.selected_entry() {
        Some(e) => e,
        None => {
            let empty = Paragraph::new("\n  No file selected")
                .block(empty_block())
                .style(Style::default().fg(FaelightColors::TEXT_DIM));
            Widget::render(empty, area, buf);
            return;
        }
    };

    let title = format!(" {} ", entry.name);
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(FaelightColors::TEXT_BRIGHT),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(FaelightColors::TEXT_DIM))
        .style(Style::default().bg(FaelightColors::BG_DARK));

    let inner_height = area.height.saturating_sub(2) as usize;
    let mut lines: Vec<Line> = Vec::new();

    // ── metadata header ──────────────────────────────────
    // Zone
    let zone_color = FaelightColors::zone_color(entry.zone);
    let zone_label = zone_label(entry.zone);
    lines.push(Line::from(vec![
        Span::styled("  Zone    ", Style::default().fg(FaelightColors::TEXT_DIM)),
        Span::styled(zone_label, Style::default().fg(zone_color).bold()),
    ]));

    // Git status
    let (git_label, git_color) = match entry.git_status {
        GitStatus::Modified => ("Modified", Color::Yellow),
        GitStatus::Added => ("Added", Color::Green),
        GitStatus::Deleted => ("Deleted", Color::Red),
        GitStatus::Untracked => ("Untracked", FaelightColors::TEXT_DIM),
        GitStatus::Clean => ("Clean", FaelightColors::TEXT_DIM),
    };
    lines.push(Line::from(vec![
        Span::styled("  Git     ", Style::default().fg(FaelightColors::TEXT_DIM)),
        Span::styled(git_label, Style::default().fg(git_color)),
    ]));

    // Intent
    if let Some(ref intent) = entry.intent_info {
        let intent_color = match intent.status {
            faelight_fm::intent::IntentStatus::Complete => FaelightColors::INTENT_COMPLETE,
            faelight_fm::intent::IntentStatus::Future => FaelightColors::INTENT_FUTURE,
            faelight_fm::intent::IntentStatus::Cancelled => FaelightColors::INTENT_CANCELLED,
            faelight_fm::intent::IntentStatus::Deferred => FaelightColors::INTENT_DEFERRED,
        };
        lines.push(Line::from(vec![
            Span::styled("  Intent  ", Style::default().fg(FaelightColors::TEXT_DIM)),
            Span::styled(
                format!("#{} {}", intent.id, intent.title),
                Style::default().fg(intent_color),
            ),
        ]));
    }

    // File size / type
    if !entry.is_dir {
        if let Ok(meta) = std::fs::metadata(&entry.path) {
            let size = format_size(meta.len());
            lines.push(Line::from(vec![
                Span::styled("  Size    ", Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::styled(size, Style::default().fg(FaelightColors::TEXT_BRIGHT)),
            ]));
        }
    }

    // Symlink target
    if entry.is_symlink {
        if let Ok(target) = std::fs::read_link(&entry.path) {
            lines.push(Line::from(vec![
                Span::styled("  Link →  ", Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::styled(
                    target.to_string_lossy().to_string(),
                    Style::default().fg(FaelightColors::SYMLINK),
                ),
            ]));
        }
    }

    // Separator
    lines.push(Line::from(Span::styled(
        "  ─────────────────────────────",
        Style::default().fg(FaelightColors::TEXT_DIM),
    )));

    let header_lines = lines.len();
    let content_height = inner_height.saturating_sub(header_lines);

    // ── content ──────────────────────────────────────────
    if entry.is_dir {
        // Show directory listing
        match std::fs::read_dir(&entry.path) {
            Ok(entries) => {
                let mut items: Vec<_> = entries.flatten().collect();
                items.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));
                for entry in items.iter().take(content_height) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = entry.path().is_dir();
                    let (icon, color) = if is_dir {
                        (" ", FaelightColors::ACCENT_BLUE)
                    } else {
                        (" ", FaelightColors::TEXT_BRIGHT)
                    };
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(format!("{} {}", icon, name), Style::default().fg(color)),
                    ]));
                }
                if items.len() > content_height {
                    lines.push(Line::from(Span::styled(
                        format!("  … {} more", items.len() - content_height),
                        Style::default().fg(FaelightColors::TEXT_DIM).italic(),
                    )));
                }
            }
            Err(_) => {
                lines.push(Line::from(Span::styled(
                    "  [Permission denied]",
                    Style::default().fg(FaelightColors::LOCKED),
                )));
            }
        }
    } else if let Some(ref content) = app.preview_content {
        // File content with line numbers
        for (i, line) in content.iter().take(content_height).enumerate() {
            let line_num = format!("{:3} │ ", i + 1);
            // Truncate long lines to fit panel
            let max_width = area.width.saturating_sub(8) as usize;
            let truncated = if line.len() > max_width {
                format!("{}…", &line[..max_width.saturating_sub(1)])
            } else {
                line.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(line_num, Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::raw(truncated),
            ]));
        }
        if content.len() > content_height {
            lines.push(Line::from(Span::styled(
                format!("  … {} more lines", content.len() - content_height),
                Style::default().fg(FaelightColors::TEXT_DIM).italic(),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  [Binary or unreadable]",
            Style::default().fg(FaelightColors::TEXT_DIM),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    Widget::render(paragraph, area, buf);
}

fn empty_block() -> Block<'static> {
    Block::default()
        .title(" PREVIEW ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(FaelightColors::TEXT_DIM))
        .style(Style::default().bg(FaelightColors::BG_DARK))
}

fn zone_label(zone: Zone) -> &'static str {
    match zone {
        Zone::Core => "Core",
        Zone::Workspace => "Workspace",
        Zone::Src => "Source",
        Zone::Project => "Project",
        Zone::Archive => "Archive",
        Zone::Scratch => "Scratch",
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
