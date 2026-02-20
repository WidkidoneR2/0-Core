//! File preview - persistent pane and overlay
use crate::app::AppState;
use crate::ui::colors::FaelightColors;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Persistent side pane — always visible, shows selected file
pub fn render_pane(area: Rect, buf: &mut Buffer, app: &AppState) {
    let border_color = FaelightColors::TEXT_DIM;

    let title = if let Some(ref path) = app.preview_path {
        format!(" 📄 {} ", path)
    } else {
        " PREVIEW ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(FaelightColors::BG_DARK));

    if app.preview_content.is_none() {
        // Nothing selected or empty
        let empty = Paragraph::new("\n  Select a file to preview")
            .block(block)
            .style(Style::default().fg(FaelightColors::TEXT_DIM));
        Widget::render(empty, area, buf);
        return;
    }

    let mut lines = vec![];
    let inner_height = area.height.saturating_sub(2) as usize; // subtract borders

    if let Some(ref content) = app.preview_content {
        for (i, line) in content.iter().take(inner_height).enumerate() {
            let line_num = format!("{:3} │ ", i + 1);
            lines.push(Line::from(vec![
                Span::styled(line_num, Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::raw(line.as_str()),
            ]));
        }

        // If file has more lines than visible
        if content.len() > inner_height {
            lines.push(Line::from(Span::styled(
                format!("  ... {} more lines", content.len() - inner_height),
                Style::default().fg(FaelightColors::TEXT_DIM).italic(),
            )));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    Widget::render(paragraph, area, buf);
}
