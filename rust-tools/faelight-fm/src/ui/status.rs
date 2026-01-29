use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use crate::app::{AppState, MessageColor};
use super::colors::FaelightColors;

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    // If there's a status message, show it instead
    let text = if let Some(ref msg) = app.status_message {
        let color = match app.message_color {
            MessageColor::Success => FaelightColors::INTENT_COMPLETE,
            MessageColor::Error => FaelightColors::INTENT_CANCELLED,
            MessageColor::Warning => FaelightColors::INTENT_FUTURE,
        };
        
        Line::from(vec![
            Span::styled(
                msg,
                Style::default().fg(color).bold()
            ),
        ])
    } else {
        // Original status bar logic
        let entry = app.selected_entry();
        
        if let Some(entry) = entry {
            if let Some(ref intent_info) = entry.intent_info {
                let status_text = match intent_info.status {
                    faelight_fm::intent::IntentStatus::Complete => "COMPLETE",
                    faelight_fm::intent::IntentStatus::Future => "FUTURE",
                    faelight_fm::intent::IntentStatus::Cancelled => "CANCELLED",
                    faelight_fm::intent::IntentStatus::Deferred => "DEFERRED",
                };
                
                let status_color = match intent_info.status {
                    faelight_fm::intent::IntentStatus::Complete => FaelightColors::INTENT_COMPLETE,
                    faelight_fm::intent::IntentStatus::Future => FaelightColors::INTENT_FUTURE,
                    faelight_fm::intent::IntentStatus::Cancelled => FaelightColors::INTENT_CANCELLED,
                    faelight_fm::intent::IntentStatus::Deferred => FaelightColors::INTENT_DEFERRED,
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
                        &intent_info.title,
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
        }
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
