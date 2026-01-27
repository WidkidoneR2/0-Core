use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use crate::app::AppState;
use super::colors::FaelightColors;

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let status_text = if let Some(entry) = app.selected_entry() {
        format!(
            "STATUS │ Zone: {} │ Health: {} │ Path: {}",
            entry.zone.short_label(),
            entry.health.badge(),
            entry.path.display()
        )
    } else {
        format!("STATUS │ Zone: {} │ No selection", app.zone.short_label())
    };
    
    let paragraph = Paragraph::new(status_text)
        .style(Style::default()
            .bg(FaelightColors::BG_DARK)
            .fg(FaelightColors::TEXT_DIM));
    
    paragraph.render(area, buf);
}
