use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Url {
    pub row: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub url: String,
}

static URL_REGEX: OnceLock<Regex> = OnceLock::new();

fn url_regex() -> &'static Regex {
    URL_REGEX.get_or_init(|| {
        // Match http://, https://, www., or common TLDs
        Regex::new(r"(https?://[^\s]+|www\.[^\s]+|[a-zA-Z0-9-]+\.(com|org|net|io|dev|ai|rs|md|sh|co)[^\s]*)").unwrap()
    })
}

/// Detect URLs in a line of text
pub fn detect_urls_in_line(text: &str, row: usize) -> Vec<Url> {
    let mut urls = Vec::new();
    let regex = url_regex();

    for mat in regex.find_iter(text) {
        let mut url_text = mat.as_str().to_string();

        // Add http:// prefix if it starts with www.
        if url_text.starts_with("www.") {
            url_text = format!("https://{}", url_text);
        }

        // Remove trailing punctuation (., ), ], etc.)
        let url_clean = url_text
            .trim_end_matches(&['.', ',', ')', ']', '}', '!', '?'][..])
            .to_string();

        urls.push(Url {
            row,
            start_col: mat.start(),
            end_col: mat.end() - (url_text.len() - url_clean.len()),
            url: url_clean,
        });
    }

    urls
}

/// Open URL in default browser
pub fn open_url(url: &str) -> std::io::Result<()> {
    use std::process::Command;

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_detection() {
        let line = "Check out https://github.com/user/repo and www.example.com!";
        let urls = detect_urls_in_line(line, 0);

        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0].url, "https://github.com/user/repo");
        assert_eq!(urls[1].url, "https://www.example.com");
    }

    #[test]
    fn test_trailing_punctuation() {
        let line = "Visit https://example.com. and (see www.test.io)";
        let urls = detect_urls_in_line(line, 0);

        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0].url, "https://example.com");
        assert_eq!(urls[1].url, "https://www.test.io");
    }
}
