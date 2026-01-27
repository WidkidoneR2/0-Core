use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use crate::app::AppState;
use super::colors::FaelightColors;

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    if !app.search_mode {
        return;
    }
    
    let status = if app.search_query.is_empty() { 
        "type to filter...".to_string()
    } else { 
        format!("{} matches", app.filtered_entries.len())
    };
    
    let search_text = format!("SEARCH: {} ({})", app.search_query, status);
    
    let paragraph = Paragraph::new(search_text)
        .style(Style::default()
            .bg(FaelightColors::ACCENT_BLUE)
            .fg(FaelightColors::BG_DARK)
            .bold());
    
    paragraph.render(area, buf);
}
