use ratatui::prelude::*;

pub fn create_layout(area: Rect) -> (Rect, Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Top bar
            Constraint::Length(1),  // Path bar
            Constraint::Min(0),     // Main area
            Constraint::Length(1),  // Status bar
            Constraint::Length(1),  // Command line
        ])
        .split(area);
    
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Zones
            Constraint::Min(0),     // File list
        ])
        .split(chunks[2]);
    
    (chunks[0], chunks[1], main_chunks[0], main_chunks[1], chunks[3])
}
