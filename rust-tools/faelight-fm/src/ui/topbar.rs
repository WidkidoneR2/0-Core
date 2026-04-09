use super::colors::FaelightColors;
use crate::app::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let lock_icon = if app.zone.is_critical() {
        "🔒"
    } else {
        "🔓"
    };
    let zone_color = FaelightColors::zone_color(app.zone);

    // v3 — read real health and active intent from cache
    let health_str = std::fs::read_to_string(
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".cache/faelight/health-status")
    ).unwrap_or_else(|_| "?".to_string());
    let health_str = health_str.trim();
    let health_color = match health_str.parse::<u32>().unwrap_or(0) {
        100 => FaelightColors::ACCENT_GREEN,
        80..=99 => Color::Rgb(227, 200, 100),
        _ => Color::Rgb(200, 80, 80),
    };
    // Read active intent from filesystem
    let core_root = format!("{}/0-core", std::env::var("HOME").unwrap_or_default());
    let active_intent = std::fs::read_dir(format!("{}/intents/future", core_root))
        .ok()
        .and_then(|d| {
            d.filter_map(|e| e.ok())
                .find(|e| {
                    std::fs::read_to_string(e.path())
                        .map(|c| c.contains("status: in-progress"))
                        .unwrap_or(false)
                })
                .map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let num = name.split('-').next().unwrap_or("").to_string();
                    format!("INT-{}", num)
                })
        })
        .unwrap_or_else(|| "—".to_string());
    let line = Line::from(vec![
        Span::styled(
            " 🌲 FAELIGHT FM ",
            Style::default().fg(FaelightColors::ACCENT_GREEN).bold(),
        ),
        Span::styled("│ ", Style::default().fg(FaelightColors::TEXT_DIM)),
        Span::styled(
            active_intent,
            Style::default().fg(Color::Rgb(100, 180, 255)).bold(),
        ),
        Span::styled(" │ Z: ", Style::default().fg(FaelightColors::TEXT_DIM)),
        Span::styled(
            app.zone.short_label(),
            Style::default().fg(zone_color).bold(),
        ),
        Span::styled(" │ ", Style::default().fg(FaelightColors::TEXT_DIM)),
        Span::styled(
            lock_icon,
            Style::default().fg(if app.zone.is_critical() {
                FaelightColors::LOCKED
            } else {
                FaelightColors::ACCENT_GREEN
            }),
        ),
        Span::styled(" │ 🏥 ", Style::default().fg(FaelightColors::TEXT_DIM)),
        Span::styled(
            format!("{}%", health_str),
            Style::default().fg(health_color).bold(),
        ),
        Span::styled(" │ v3.0.0 ", Style::default().fg(FaelightColors::TEXT_DIM)),
        Span::styled(" │ sort: ", Style::default().fg(FaelightColors::TEXT_DIM)),
        Span::styled(
            app.sort_mode.label(),
            Style::default().fg(Color::Rgb(180, 140, 255)).bold(),
        ),
    ]);

    let paragraph = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(FaelightColors::ACCENT_GREEN))
            .style(Style::default().bg(FaelightColors::BG_DARK)),
    );

    paragraph.render(area, buf);
}
