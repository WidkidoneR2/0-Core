pub mod colors;
pub mod filelist;
pub mod help;
pub mod info;
pub mod layout;
pub mod search;
pub mod status;
pub mod topbar;
pub mod zones;

use crate::app::AppState;
use colors::FaelightColors;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, app: &mut AppState) {
    let (topbar_area, path_area, zones_area, filelist_area, status_area) =
        layout::create_layout(frame.area());

    // Top bar
    topbar::render(topbar_area, frame.buffer_mut(), app);

    // Path bar (or search bar if searching)
    if app.search_mode {
        search::render(path_area, frame.buffer_mut(), app);
    } else {
        let path_text = format!("PATH: {}", app.cwd.display());
        let path_paragraph = Paragraph::new(path_text).style(
            Style::default()
                .bg(FaelightColors::BG_DARK)
                .fg(FaelightColors::TEXT_BRIGHT),
        );
        frame.render_widget(path_paragraph, path_area);
    }

    // Zones panel - capture click regions
    let zone_regions = zones::render(zones_area, frame.buffer_mut(), app.zone);
    app.zone_click_regions = zone_regions;

    // File list (filtered) - capture and store click regions
    let file_regions = filelist::render(filelist_area, frame.buffer_mut(), app);
    app.file_click_regions = file_regions;

    // Status bar
    status::render(status_area, frame.buffer_mut(), app);

    // Overlays (render on top)
    if app.help_visible {
        help::render(frame.area(), frame.buffer_mut());
    }

    if app.info_visible {
        info::render(frame.area(), frame.buffer_mut(), app);
    }

    if app.preview_visible {
        preview::render(frame.area(), frame.buffer_mut(), app);
    }
}
pub mod preview;
