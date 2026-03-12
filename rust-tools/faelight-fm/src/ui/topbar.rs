use super::colors::FaelightColors;
use crate::app::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let lock_icon = if app.zone.is_critical() { "🔒" } else { "🔓" };
    let zone_color = FaelightColors::zone_color(app.zone);

    let line = Line::from(vec![
        Span::styled(
            " 🌲 FAELIGHT FM ",
            Style::default()
                .fg(FaelightColors::ACCENT_GREEN)
                .bold(),
        ),
        Span::styled(
            "│",
            Style::default().fg(FaelightColors::TEXT_DIM),
        ),
        Span::styled(
            " Z: ",
            Style::default().fg(FaelightColors::TEXT_DIM),
        ),
        Span::styled(
            app.zone.short_label(),
            Style::default().fg(zone_color).bold(),
        ),
        Span::styled(
            " │ ",
            Style::default().fg(FaelightColors::TEXT_DIM),
        ),
        Span::styled(
            lock_icon,
            Style::default().fg(if app.zone.is_critical() {
                FaelightColors::LOCKED
            } else {
                FaelightColors::ACCENT_GREEN
            }),
        ),
        Span::styled(
            " │ 🏥 ",
            Style::default().fg(FaelightColors::TEXT_DIM),
        ),
        Span::styled(
            "95%",
            Style::default().fg(FaelightColors::ACCENT_GREEN).bold(),
        ),
        Span::styled(
            " │ v2.3.0 ",
            Style::default().fg(FaelightColors::TEXT_DIM),
        ),
    ]);

    let paragraph = Paragraph::new(line)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(FaelightColors::ACCENT_GREEN))
                .style(Style::default().bg(FaelightColors::BG_DARK)),
        );

    paragraph.render(area, buf);
}
