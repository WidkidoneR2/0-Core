#![allow(dead_code)]

//! Tab bar - horizontal widget layout like faelight-bar

use crate::render::{draw_gradient_separator, draw_text};
use crate::ui::colors::{ACCENT_COLOR, DIM_COLOR};

pub struct Tab {
    pub title: String,
    #[allow(dead_code)]
    pub url: String,
    pub active: bool,
}

pub struct TabBar {
    pub tabs: Vec<Tab>,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: vec![
                Tab {
                    title: "Home".to_string(),
                    url: "about:home".to_string(),
                    active: true,
                },
                Tab {
                    title: "Example".to_string(),
                    url: "https://example.com".to_string(),
                    active: false,
                },
            ],
        }
    }

    pub fn render(&self, canvas: &mut [u8], width: u32, height: u32) {
        let mut x = 10;
        let y = 8;

        for (i, tab) in self.tabs.iter().enumerate() {
            // Tab icon + title
            let color = if tab.active { ACCENT_COLOR } else { DIM_COLOR };
            let icon = if tab.active { "●" } else { "○" };
            let text = format!("{} {}", icon, tab.title);

            draw_text(canvas, width, &text, x, y, color);
            x += (text.len() as i32 * 8) + 10;

            // Gradient separator (YOUR signature style)
            if i < self.tabs.len() - 1 {
                draw_gradient_separator(canvas, width, height, x, DIM_COLOR);
                x += 15;
            }
        }

        // New tab button
        x += 10;
        draw_text(canvas, width, "+", x, y, ACCENT_COLOR);
    }
}
