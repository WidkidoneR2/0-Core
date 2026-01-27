use ratatui::prelude::*;
use faelight_zone::Zone;

pub struct FaelightColors;

impl FaelightColors {
    // Background colors
    pub const BG_DARK: Color = Color::Rgb(17, 20, 15);
    #[allow(dead_code)]
    pub const BG_NORMAL: Color = Color::Rgb(25, 30, 22);
    pub const BG_SELECTED: Color = Color::Rgb(45, 52, 38);
    
    // Faelight Forest accent colors
    pub const ACCENT_GREEN: Color = Color::Rgb(163, 227, 107);
    pub const ACCENT_BLUE: Color = Color::Rgb(107, 163, 227);
    #[allow(dead_code)]
    pub const ACCENT_ORANGE: Color = Color::Rgb(227, 163, 107);
    
    // Text colors
    pub const TEXT_BRIGHT: Color = Color::Rgb(218, 224, 215);
    pub const TEXT_DIM: Color = Color::Rgb(119, 127, 111);
    #[allow(dead_code)]
    pub const TEXT_DIMMER: Color = Color::Rgb(80, 85, 75);
    
    // Status colors
    pub const LOCKED: Color = Color::Rgb(200, 100, 100);
    #[allow(dead_code)]
    pub const MODIFIED: Color = Color::Rgb(227, 163, 107);
    #[allow(dead_code)]
    pub const UNTRACKED: Color = Color::Rgb(119, 127, 111);
    
    pub fn zone_color(zone: Zone) -> Color {
        match zone {
            Zone::Core => Self::LOCKED,                      // Red - locked
            Zone::Workspace => Color::Rgb(227, 163, 107),    // Orange - active work
            Zone::Src => Self::ACCENT_GREEN,                 // Green - source
            Zone::Project => Self::ACCENT_BLUE,              // Blue - projects
            Zone::Archive => Color::Rgb(180, 150, 200),      // Purple - dormant (BRIGHTER!)
            Zone::Scratch => Color::Rgb(150, 150, 80),       // Yellow - temporary
        }
    }
    
    pub fn directory_style(selected: bool) -> Style {
        if selected {
            Style::default()
                .fg(Self::ACCENT_BLUE)
                .bg(Self::BG_SELECTED)
                .bold()
        } else {
            Style::default().fg(Self::ACCENT_BLUE)
        }
    }
    
    pub fn file_style(selected: bool) -> Style {
        if selected {
            Style::default()
                .fg(Self::TEXT_BRIGHT)
                .bg(Self::BG_SELECTED)
        } else {
            Style::default().fg(Self::TEXT_BRIGHT)
        }
    }
}
