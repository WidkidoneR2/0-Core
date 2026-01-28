//! File preview overlay
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use crate::app::AppState;
use crate::ui::colors::FaelightColors;

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    if !app.preview_visible {
        return;
    }
    
    // Small box (45% width, 30% height), positioned low
    let preview_area = bottom_rect(45, 30, area);
    
    // Build content
    let mut lines = vec![];
    
    // Title
    if let Some(ref path) = app.preview_path {
        lines.push(Line::from(vec![
            Span::styled("📄 ", Style::default().fg(FaelightColors::INTENT_FUTURE)),
            Span::styled(path, Style::default().fg(FaelightColors::TEXT_BRIGHT).bold()),
        ]));
        lines.push(Line::from(""));
    }
    
    // Content (show up to 20 lines for smaller box)
    if let Some(ref content) = app.preview_content {
        for (i, line) in content.iter().take(20).enumerate() {
            let line_num = format!("{:2} │ ", i + 1);
            lines.push(Line::from(vec![
                Span::styled(line_num, Style::default().fg(FaelightColors::INTENT_FUTURE)),
                Span::raw(line),
            ]));
        }
    }
    
    // Footer
    lines.push(Line::from(""));
    lines.push(Line::from(
        Span::styled("Press any key to close", Style::default().fg(FaelightColors::INTENT_FUTURE).italic())
    ));
    
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" PREVIEW ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(FaelightColors::INTENT_COMPLETE))
                .style(Style::default().bg(Color::Rgb(10, 12, 8)))  // Very dark green background
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(Color::Rgb(10, 12, 8)));  // Very dark green
    
    Widget::render(paragraph, preview_area, buf);
}

// Position in bottom portion (starts at 55% down)
fn bottom_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55),  // 55% top (pushes preview down)
            Constraint::Percentage(percent_y),  // Preview takes 30%
            Constraint::Percentage(15),  // 15% bottom spacing
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
