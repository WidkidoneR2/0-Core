use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Widget};
use faelight_zone::Zone;
use super::colors::FaelightColors;

pub fn render(area: Rect, buf: &mut Buffer, current_zone: Zone) {
    let zones = [
        Zone::Core,
        Zone::Workspace,
        Zone::Src,
        Zone::Project,
        Zone::Archive,
        Zone::Scratch,
    ];
    
    let items: Vec<ListItem> = zones
        .iter()
        .map(|zone| {
            let is_match = *zone == current_zone;
            
            let style = if is_match {
                // Current zone: bright color + background highlight + bold
                Style::default()
                    .fg(FaelightColors::zone_color(*zone))
                    .bg(FaelightColors::BG_SELECTED)
                    .bold()
            } else {
                // Other zones: dim text, no background
                Style::default().fg(FaelightColors::TEXT_DIM)
            };
            
            let label = format!("{} {}", zone.icon(), zone.short_label());
            ListItem::new(label).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .title("ZONES")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(FaelightColors::TEXT_DIM))
        );
    
    Widget::render(list, area, buf);
}
