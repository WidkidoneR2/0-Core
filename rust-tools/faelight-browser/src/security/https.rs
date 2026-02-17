//! HTTPS security validation

use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityStatus {
    Secure,    // HTTPS with valid cert
    Insecure,  // HTTP
    LocalFile, // file:// or about:
    Unknown,
}

impl SecurityStatus {
    pub fn check(url: &str) -> Self {
        match Url::parse(url) {
            Ok(parsed) => match parsed.scheme() {
                "https" => Self::Secure,
                "http" => Self::Insecure,
                "file" | "about" => Self::LocalFile,
                _ => Self::Unknown,
            },
            Err(_) => Self::Unknown,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Secure => "🔒",
            Self::Insecure => "⚠️",
            Self::LocalFile => "📄",
            Self::Unknown => "❓",
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            Self::Secure => Color::Rgb(163, 227, 107),    // Green
            Self::Insecure => Color::Rgb(200, 100, 100),  // Red
            Self::LocalFile => Color::Rgb(107, 163, 227), // Blue
            Self::Unknown => Color::Rgb(119, 127, 111),   // Dim
        }
    }
}
