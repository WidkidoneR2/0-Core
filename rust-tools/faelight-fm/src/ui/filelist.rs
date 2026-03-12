use super::colors::FaelightColors;
use crate::app::AppState;
use faelight_fm::git::GitStatus;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) -> Vec<(u16, u16, usize)> {
    let mut file_regions = Vec::new();

    let items: Vec<ListItem> = app
        .filtered_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.selected;

            let (git_marker, git_color) = match entry.git_status {
                GitStatus::Modified => ("▲", Color::Rgb(227, 163, 107)),
                GitStatus::Added    => ("✚", Color::Rgb(163, 227, 107)),
                GitStatus::Deleted  => ("✖", Color::Rgb(200, 100, 100)),
                GitStatus::Untracked => ("◌", FaelightColors::TEXT_DIM),
                GitStatus::Clean    => (" ", FaelightColors::TEXT_DIM),
            };

            let name_style = if entry.is_symlink {
                if is_selected {
                    Style::default()
                        .fg(FaelightColors::SYMLINK)
                        .bg(FaelightColors::BG_SELECTED)
                        .italic()
                } else {
                    Style::default().fg(FaelightColors::SYMLINK).italic()
                }
            } else if entry.is_dir {
                FaelightColors::directory_style(is_selected)
            } else {
                FaelightColors::file_style(is_selected)
            };

            let size_str = if entry.is_dir {
                "      ".to_string()
            } else {
                match std::fs::metadata(&entry.path) {
                    Ok(m) => format_size(m.len()),
                    Err(_) => "      ".to_string(),
                }
            };

            let suffix = if entry.is_symlink { " →" } else { "" };

            let bg = if is_selected {
                FaelightColors::BG_SELECTED
            } else {
                FaelightColors::BG_DARK
            };

            // Intent indicator
            let intent_marker = if entry.intent_info.is_some() { "◆" } else { " " };

            let spans = vec![
                Span::styled(
                    format!(" {} ", git_marker),
                    Style::default().fg(git_color).bg(bg),
                ),
                Span::styled(
                    format!("{} ", intent_marker),
                    Style::default()
                        .fg(FaelightColors::ACCENT_GREEN)
                        .bg(bg),
                ),
                Span::styled(format!("{} ", entry.icon()), name_style),
                Span::styled(format!("{}{}", entry.name, suffix), name_style),
                Span::styled(
                    format!("  {}", size_str),
                    Style::default().fg(FaelightColors::TEXT_DIM).bg(bg),
                ),
            ];

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if app.search_mode {
        format!(
            " 🔍 {} ({} matches) ",
            app.search_query,
            app.filtered_entries.len()
        )
    } else {
        format!(
            " 🌲 {} items ",
            app.filtered_entries.len()
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(FaelightColors::ACCENT_GREEN)
                        .bold(),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(FaelightColors::ACCENT_GREEN))
                .style(Style::default().bg(FaelightColors::BG_DARK)),
        )
        .highlight_style(
            Style::default()
                .bg(FaelightColors::BG_SELECTED)
                .fg(FaelightColors::ACCENT_GREEN)
                .bold(),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.selected));
    ratatui::widgets::StatefulWidget::render(list, area, buf, &mut state);

    let start_y = area.y + 1;
    let visible_count = (area.height.saturating_sub(2)) as usize;
    for i in 0..visible_count {
        if i < app.filtered_entries.len() {
            file_regions.push((start_y + i as u16, area.width, i));
        }
    }

    file_regions
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{:>4}B ", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:>3}KB ", bytes / 1024)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:>3}MB ", bytes / (1024 * 1024))
    } else {
        format!("{:>3}GB ", bytes / (1024 * 1024 * 1024))
    }
}
