use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Widget};
use crate::app::AppState;
use super::colors::FaelightColors;
use faelight_fm::git::GitStatus;

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let items: Vec<ListItem> = app
        .filtered_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.selected;
            
            // Symlinks get special color treatment
            let base_style = if entry.is_symlink {
                if is_selected {
                    Style::default()
                        .fg(FaelightColors::SYMLINK)
                        .bg(FaelightColors::BG_SELECTED)
                        .italic()
                } else {
                    Style::default()
                        .fg(FaelightColors::SYMLINK)
                        .italic()
                }
            } else if entry.is_dir {
                FaelightColors::directory_style(is_selected)
            } else {
                FaelightColors::file_style(is_selected)
            };
            
            
            let zone_tag = format!("[Z:{}]", entry.zone.short_label());
            let mut spans = vec![
                Span::raw(format!("{} ", entry.icon())),
                // Git status marker
                Span::styled(
                    entry.git_status.marker(),
                    Style::default().fg(match entry.git_status {
                        GitStatus::Modified => Color::Yellow,
                        GitStatus::Added => Color::Green,
                        GitStatus::Deleted => Color::Red,
                        GitStatus::Untracked => FaelightColors::TEXT_DIM,
                        GitStatus::Clean => FaelightColors::TEXT_DIM,
                    })
                ),
                Span::styled(
                    format!("{:<30} ", entry.name),
                    base_style
                ),
                Span::raw(format!("{:<12} ", zone_tag)),
            ];
            
            if let Some(ref intent_info) = entry.intent_info {
                let intent_color = match intent_info.status {
                    faelight_fm::intent::IntentStatus::Complete => FaelightColors::INTENT_COMPLETE,
                    faelight_fm::intent::IntentStatus::Future => FaelightColors::INTENT_FUTURE,
                    faelight_fm::intent::IntentStatus::Cancelled => FaelightColors::INTENT_CANCELLED,
                    faelight_fm::intent::IntentStatus::Deferred => FaelightColors::INTENT_DEFERRED,
                };
                
                spans.push(Span::styled(
                    format!("[INT:{}] ", intent_info.id),
                    Style::default().fg(intent_color).bold()
                ));
            } else {
                spans.push(Span::raw(format!("{:<12} ", "")));
            }
            
            spans.push(Span::raw(entry.health.badge()));
            
            ListItem::new(Line::from(spans))
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
