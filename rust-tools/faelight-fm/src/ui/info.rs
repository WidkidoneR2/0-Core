use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use crate::app::AppState;
use super::colors::FaelightColors;
use std::fs;

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let popup_area = centered_rect(50, 60, area);
    
    Clear.render(popup_area, buf);
    
    let entry = app.selected_entry();
    
    let info_lines = if let Some(entry) = entry {
        // Get file metadata
        let metadata = fs::metadata(&entry.path);
        let size = metadata
            .as_ref()
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|_| "Unknown".to_string());
        
        let _modified = metadata
            .as_ref()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| format!("{:?}", t))
            .unwrap_or_else(|| "Unknown".to_string());
        
        let file_type = if entry.is_symlink {
            "Symlink"
        } else if entry.is_dir {
            "Directory"
        } else {
            "File"
        };
        
        let intent_text = if let Some(ref info) = entry.intent_info {
            let status = match info.status {
                crate::intent::IntentStatus::Complete => "COMPLETE",
                crate::intent::IntentStatus::Future => "FUTURE",
                crate::intent::IntentStatus::Cancelled => "CANCELLED",
                crate::intent::IntentStatus::Deferred => "DEFERRED",
            };
            format!("#{} - {} - {}", info.id, status, info.title)
        } else {
            "None".to_string()
        };
        
        vec![
            Line::from(vec![
                Span::styled("FILE INFORMATION", Style::default().fg(FaelightColors::ACCENT_GREEN).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Name:     ", Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::styled(&entry.name, Style::default().fg(FaelightColors::TEXT_BRIGHT).bold()),
            ]),
            Line::from(vec![
                Span::styled("Type:     ", Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::styled(file_type, Style::default().fg(FaelightColors::ACCENT_BLUE)),
            ]),
            Line::from(vec![
                Span::styled("Zone:     ", Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::styled(
                    format!("{} ({})", entry.zone.short_label(), entry.zone.icon()),
                    Style::default().fg(FaelightColors::zone_color(entry.zone))
                ),
            ]),
            Line::from(vec![
                Span::styled("Intent:   ", Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::styled(intent_text.clone(), Style::default().fg(FaelightColors::TEXT_BRIGHT)),
            ]),
            Line::from(vec![
                Span::styled("Size:     ", Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::styled(size, Style::default().fg(FaelightColors::TEXT_BRIGHT)),
            ]),
            Line::from(vec![
                Span::styled("Path:     ", Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::styled(
                    entry.path.to_string_lossy().to_string(),
                    Style::default().fg(FaelightColors::TEXT_BRIGHT)
                ),
            ]),
            Line::from(vec![
                Span::styled("Symlink:  ", Style::default().fg(FaelightColors::TEXT_DIM)),
                Span::styled(
                    if entry.is_symlink { "Yes" } else { "No" },
                    Style::default().fg(if entry.is_symlink { 
                        FaelightColors::SYMLINK 
                    } else { 
                        FaelightColors::TEXT_BRIGHT 
                    })
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press any key to close", Style::default().fg(FaelightColors::TEXT_DIM).italic()),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("No file selected", Style::default().fg(FaelightColors::TEXT_DIM)),
            ]),
        ]
    };
    
    let paragraph = Paragraph::new(info_lines)
        .block(
            Block::default()
                .title(" INFO ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(FaelightColors::ACCENT_BLUE))
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

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
