use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use crate::app::AppState;
use super::colors::FaelightColors;

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let entry = app.selected_entry();
    
    let text = if let Some(entry) = entry {
        if let Some(ref intent_info) = entry.intent_info {
            let status_text = match intent_info.status {
                crate::intent::IntentStatus::Complete => "COMPLETE",
                crate::intent::IntentStatus::Future => "FUTURE",
                crate::intent::IntentStatus::Cancelled => "CANCELLED",
                crate::intent::IntentStatus::Deferred => "DEFERRED",
            };
            
            let status_color = match intent_info.status {
                crate::intent::IntentStatus::Complete => FaelightColors::INTENT_COMPLETE,
                crate::intent::IntentStatus::Future => FaelightColors::INTENT_FUTURE,
                crate::intent::IntentStatus::Cancelled => FaelightColors::INTENT_CANCELLED,
                crate::intent::IntentStatus::Deferred => FaelightColors::INTENT_DEFERRED,
            };
            
            Line::from(vec![
                Span::raw("Intent "),
                Span::styled(
                    format!("#{} ", intent_info.id),
                    Style::default().fg(status_color).bold()
                ),
                Span::raw("- "),
                Span::styled(
                    status_text,
                    Style::default().fg(status_color).bold()
                ),
                Span::raw(" - "),
                Span::styled(
                    &intent_info.title,  // Show full title!
                    Style::default().fg(FaelightColors::TEXT_BRIGHT)
                ),
            ])
        } else {
            Line::from(vec![
                Span::raw("Selected: "),
                Span::styled(
                    &entry.name,
                    Style::default().fg(FaelightColors::TEXT_BRIGHT)
                ),
            ])
        }
    } else {
        Line::from(vec![
            Span::styled(
                "No selection",
                Style::default().fg(FaelightColors::TEXT_DIM)
            ),
        ])
    };
    
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("STATUS")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(FaelightColors::TEXT_DIM))
        );
    
    Widget::render(paragraph, area, buf);
}
