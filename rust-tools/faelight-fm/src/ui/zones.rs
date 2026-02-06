use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Widget};
use faelight_zone::Zone;
use super::colors::FaelightColors;

pub fn render(area: Rect, buf: &mut Buffer, current_zone: Zone) -> Vec<(u16, u16, u16, u16, u8)> {
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
    
    // Calculate click regions for each zone (x, y, width, height, zone_num)
    let mut zone_regions = Vec::new();
    let start_y = area.y + 1;  // After top border
    let zone_height = 1;  // Each zone is one row
    
    for i in 0..6 {  // 6 zones
        zone_regions.push((
            area.x,           // x
            start_y + i,      // y
            area.width,       // width
            zone_height,      // height
            i as u8,          // zone number 0-5
        ));
    }
    
    zone_regions
}
