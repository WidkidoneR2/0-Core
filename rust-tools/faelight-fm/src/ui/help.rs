use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use super::colors::FaelightColors;

pub fn render(area: Rect, buf: &mut Buffer) {
    // Center the help box
    let popup_area = centered_rect(60, 70, area);
    
    // Clear the background
    Clear.render(popup_area, buf);
    
    let help_text = vec![
        Line::from(vec![
            Span::styled("KEYBOARD SHORTCUTS", Style::default().fg(FaelightColors::ACCENT_GREEN).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(FaelightColors::ACCENT_BLUE).bold()),
        ]),
        Line::from("  j / ↓      - Move down"),
        Line::from("  k / ↑      - Move up"),
        Line::from("  l / → / ⏎  - Enter directory"),
        Line::from("  h / ←      - Go to parent"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Zone Jumping:", Style::default().fg(FaelightColors::ACCENT_BLUE).bold()),
        ]),
        Line::from("  0 - Core       (~/0-core)"),
        Line::from("  1 - Workspace  (~/0-core/rust-tools)"),
        Line::from("  2 - Src        (~/1-src)"),
        Line::from("  3 - Project    (~/2-projects)"),
        Line::from("  4 - Archive    (~/3-archive)"),
        Line::from("  5 - Scratch    (temporary)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Other:", Style::default().fg(FaelightColors::ACCENT_BLUE).bold()),
        ]),
        Line::from("  ?   - Toggle this help"),
        Line::from("  q   - Quit"),
        Line::from("  Esc - Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press any key to close", Style::default().fg(FaelightColors::TEXT_DIM).italic()),
        ]),
    ];
    
    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" HELP ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(FaelightColors::ACCENT_GREEN))
        )
        .style(Style::default().bg(FaelightColors::BG_DARK));
    
    paragraph.render(popup_area, buf);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
