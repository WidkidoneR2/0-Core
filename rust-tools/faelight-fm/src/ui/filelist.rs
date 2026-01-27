use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Widget};
use crate::app::AppState;
use super::colors::FaelightColors;

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.selected;
            
            // Choose style based on type and selection
            let style = if entry.is_dir {
                FaelightColors::directory_style(is_selected)
            } else {
                FaelightColors::file_style(is_selected)
            };
            
            // Format with better spacing
            let zone_tag = format!("[Z:{}]", entry.zone.short_label());
            let text = format!(
                "{} {:<35} {:<12} {}",
                entry.icon(),
                entry.name,
                zone_tag,
                entry.health.badge()
            );
            
            ListItem::new(text).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .title("FILE LIST")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(FaelightColors::TEXT_DIM))
        );
    
    Widget::render(list, area, buf);
}
