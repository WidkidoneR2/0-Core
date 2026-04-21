//! faelight-term v2 -- Configuration
//! Faelight color palette preserved from v1.
pub const FONT_REGULAR: &str = "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf";
pub const FONT_BOLD:    &str = "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Bold.ttf";
pub const FONT_ITALIC:  &str = "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Italic.ttf";
pub const FONT_EMOJI:   &str = "/usr/share/fonts/noto/NotoColorEmoji.ttf";
pub struct Config {
    pub font_size:   f32,
    pub font_family: String,
    pub shell:       String,
    pub bg:          [f32; 4],
    pub fg:          [f32; 4],
    pub colors:      [[f32; 4]; 16],
}
impl Config {
    pub fn load() -> Self {
        Self {
            font_size:   14.0,
            font_family: "JetBrainsMonoNerdFont".to_string(),
            shell:       std::env::var("SHELL")
                             .unwrap_or_else(|_| "/bin/bash".to_string()),
            bg: [0.059, 0.078, 0.067, 1.0], //
            fg: [0.843, 0.878, 0.855, 1.0], //
            colors: Self::faelight_palette(),
        }
    }
    fn faelight_palette() -> [[f32; 4]; 16] {
        [
            [0.059, 0.078, 0.067, 1.0], // black
            [0.902, 0.494, 0.502, 1.0], // red
            [0.420, 0.890, 0.639, 1.0], // green
            [0.961, 0.757, 0.467, 1.0], // yellow
            [0.361, 0.784, 1.000, 1.0], // blue
            [0.839, 0.600, 0.714, 1.0], // magenta
            [0.498, 0.784, 0.784, 1.0], // cyan
            [0.843, 0.878, 0.855, 1.0], // white
            [0.467, 0.561, 0.498, 1.0], // bright black
            [0.902, 0.494, 0.502, 1.0], // bright red
            [0.420, 0.890, 0.639, 1.0], // bright green
            [0.961, 0.757, 0.467, 1.0], // bright yellow
            [0.361, 0.784, 1.000, 1.0], // bright blue
            [0.839, 0.600, 0.714, 1.0], // bright magenta
            [0.498, 0.784, 0.784, 1.0], // bright cyan
            [1.000, 1.000, 1.000, 1.0], // bright white
        ]
    }
}
