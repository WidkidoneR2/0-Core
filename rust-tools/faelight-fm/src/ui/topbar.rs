use super::colors::FaelightColors;
use crate::app::AppState;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    // Show critical status if in Core or Workspace
    let lock_status = if app.zone.is_critical() {
        "🔒 CRITICAL"
    } else {
        "UNL"
    };

    let text = format!(
        "🌲 Faelight FM │ Z: {} │ P: DEF │ {} │ 🏥 HEALTH: OK │ v2.2.0",
        app.zone.short_label(),
        lock_status
    );

    let paragraph = Paragraph::new(text).style(
        Style::default()
            .bg(FaelightColors::BG_DARK)
            .fg(FaelightColors::ACCENT_GREEN),
    );

    paragraph.render(area, buf);
}
