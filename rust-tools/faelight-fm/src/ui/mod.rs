pub mod layout;
pub mod topbar;
pub mod zones;
pub mod filelist;
pub mod status;
pub mod help;
pub mod info;
pub mod search;
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
    
    // Path bar (or search bar if searching)
    if app.search_mode {
        search::render(path_area, frame.buffer_mut(), app);
    } else {
        let path_text = format!("PATH: {}", app.cwd.display());
        let path_paragraph = Paragraph::new(path_text)
            .style(Style::default()
                .bg(FaelightColors::BG_DARK)
                .fg(FaelightColors::TEXT_BRIGHT));
        frame.render_widget(path_paragraph, path_area);
    }
    
    // Zones panel
    zones::render(zones_area, frame.buffer_mut(), app.zone);
    
    // File list (filtered)
    filelist::render(filelist_area, frame.buffer_mut(), app);
    
    // Status bar
    status::render(status_area, frame.buffer_mut(), app);
    
    // Overlays (render on top)
    if app.help_visible {
        help::render(frame.area(), frame.buffer_mut());
    }
    
    if app.info_visible {
        info::render(frame.area(), frame.buffer_mut(), app);
    }
}
