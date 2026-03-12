use super::colors::FaelightColors;
use crate::app::{AppState, MessageColor};
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let line = if let Some(ref msg) = app.status_message {
        let color = match app.message_color {
            MessageColor::Success => FaelightColors::INTENT_COMPLETE,
            MessageColor::Error   => FaelightColors::INTENT_CANCELLED,
            MessageColor::Warning => FaelightColors::INTENT_FUTURE,
        };
        Line::from(vec![
            Span::styled(" ● ", Style::default().fg(color)),
            Span::styled(msg, Style::default().fg(color).bold()),
        ])
    } else {
        let entry = app.selected_entry();

        if let Some(entry) = entry {
            let mut spans = vec![
                Span::styled(" ", Style::default()),
            ];

            // File name
            spans.push(Span::styled(
                &entry.name,
                Style::default().fg(FaelightColors::TEXT_BRIGHT).bold(),
            ));

            // Size
            if !entry.is_dir {
                if let Ok(m) = std::fs::metadata(&entry.path) {
                    spans.push(Span::styled(
                        "  │  ",
                        Style::default().fg(FaelightColors::TEXT_DIM),
                    ));
                    spans.push(Span::styled(
                        format_size(m.len()),
                        Style::default().fg(FaelightColors::ACCENT_BLUE),
                    ));
                }
            }

            // Intent info
            if let Some(ref intent) = entry.intent_info {
                let status_color = match intent.status {
                    faelight_fm::intent::IntentStatus::Complete  => FaelightColors::INTENT_COMPLETE,
                    faelight_fm::intent::IntentStatus::Future    => FaelightColors::INTENT_FUTURE,
                    faelight_fm::intent::IntentStatus::Cancelled => FaelightColors::INTENT_CANCELLED,
                    faelight_fm::intent::IntentStatus::Deferred  => FaelightColors::INTENT_DEFERRED,
                };
                spans.push(Span::styled(
                    "  │  ◆ INT-",
                    Style::default().fg(FaelightColors::TEXT_DIM),
                ));
                spans.push(Span::styled(
                    format!("{}", intent.id),
                    Style::default().fg(status_color).bold(),
                ));
                spans.push(Span::styled(
                    format!(" {}", intent.title),
                    Style::default().fg(FaelightColors::TEXT_DIM),
                ));
            }

            // Symlink indicator
            if entry.is_symlink {
                spans.push(Span::styled(
                    "  │  → symlink",
                    Style::default().fg(FaelightColors::SYMLINK).italic(),
                ));
            }

            spans.push(Span::styled(" ", Style::default()));
            Line::from(spans)
        } else {
            Line::from(vec![Span::styled(
                " No selection",
                Style::default().fg(FaelightColors::TEXT_DIM),
            )])
        }
    };

    let paragraph = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(FaelightColors::TEXT_DIM))
            .style(Style::default().bg(FaelightColors::BG_DARK)),
    );

    Widget::render(paragraph, area, buf);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{}KB", bytes / 1024)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{}MB", bytes / (1024 * 1024))
    } else {
        format!("{}GB", bytes / (1024 * 1024 * 1024))
    }
}
