pub mod layout;
pub mod topbar;
pub mod zones;
pub mod filelist;
pub mod status;
pub mod colors;

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use crate::app::AppState;
use colors::FaelightColors;

pub fn render(frame: &mut Frame, app: &AppState) {
    let (topbar_area, path_area, zones_area, filelist_area, status_area) = 
        layout::create_layout(frame.area());
    
    // Top bar
    topbar::render(topbar_area, frame.buffer_mut(), app);
    
    // Path bar
    let path_text = format!("PATH: {}", app.cwd.display());
    let path_paragraph = Paragraph::new(path_text)
        .style(Style::default()
            .bg(FaelightColors::BG_DARK)
            .fg(FaelightColors::TEXT_BRIGHT));
    frame.render_widget(path_paragraph, path_area);
    
    // Zones panel
    zones::render(zones_area, frame.buffer_mut(), app.zone);
    
    // File list
    filelist::render(filelist_area, frame.buffer_mut(), app);
    
    // Status bar
    status::render(status_area, frame.buffer_mut(), app);
}
