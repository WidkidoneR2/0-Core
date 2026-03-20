// faelight-shell — live context-aware prompt
use crate::db::ForestDb;
use colored::*;

#[allow(dead_code)]
pub fn render(db: &ForestDb) -> String {
    let health = db.health_score().unwrap_or(95);

    let health_str = if health >= 95 {
        format!("{}%", health).bright_green().to_string()
    } else if health >= 80 {
        format!("{}%", health).yellow().to_string()
    } else {
        format!("{}%", health).bright_red().to_string()
    };

    // Current directory — shortened
    let cwd = std::env::current_dir()
        .map(|p| {
            let home = std::env::var("HOME").unwrap_or_default();
            let path = p.to_string_lossy().to_string();
            if path.starts_with(&home) {
                format!("~{}", &path[home.len()..])
            } else {
                path
            }
        })
        .unwrap_or_else(|_| "?".to_string());

    // Shorten long paths
    let cwd = if cwd.len() > 30 {
        let parts: Vec<&str> = cwd.split('/').collect();
        if parts.len() > 3 {
            format!(".../{}/{}", parts[parts.len()-2], parts[parts.len()-1])
        } else {
            cwd
        }
    } else {
        cwd
    };

    format!(
        "\n{} {} {} {}\n{} ",
        "🌲".normal(),
        cwd.bright_cyan(),
        health_str,
        "forest".dimmed(),
        "❯".bright_green(),
    )
}

/// Wrap ANSI escape sequences with readline ignore markers \x01 and \x02
/// so rustyline can correctly calculate the visible prompt width.
/// Without this, Tab completion breaks silently.
pub fn rl_wrap(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            out.push('\x01'); // start ignore
            out.push('\x1b');
            while let Some(&nc) = chars.peek() {
                out.push(nc);
                chars.next();
                if nc == 'm' { break; }
            }
            out.push('\x02'); // end ignore
        } else {
            out.push(c);
        }
    }
    out
}

/// Split prompt into pre-print (status line) and readline prompt (single line).
/// Rustyline cannot handle newlines in the prompt string — this fixes Tab completion.
pub fn render_split(db: &ForestDb) -> (String, String) {
    let health = db.health_score().unwrap_or(95);
    let health_str = if health >= 95 {
        format!("{}%", health).bright_green().to_string()
    } else if health >= 80 {
        format!("{}%", health).yellow().to_string()
    } else {
        format!("{}%", health).bright_red().to_string()
    };
    let cwd = std::env::current_dir()
        .map(|p| {
            let home = std::env::var("HOME").unwrap_or_default();
            let path = p.to_string_lossy().to_string();
            if path.starts_with(&home) {
                format!("~{}", &path[home.len()..])
            } else { path }
        })
        .unwrap_or_else(|_| "?".to_string());
    let cwd = if cwd.len() > 30 {
        let parts: Vec<&str> = cwd.split('/').collect();
        if parts.len() > 3 {
            format!(".../{}/{}", parts[parts.len()-2], parts[parts.len()-1])
        } else { cwd }
    } else { cwd };
    let pre = format!("\n{} {} {} {}\n",
        "🌲".normal(),
        cwd.bright_cyan(),
        health_str,
        "forest".dimmed(),
    );
    let prompt_raw = format!("{} ", "❯".bright_green());
    let prompt = rl_wrap(&prompt_raw);
    (pre, prompt)
}

/// Single-line prompt safe for rustyline — no newlines, ANSI wrapped.
/// Status line is printed after command output instead.
pub fn render_line(db: &ForestDb) -> String {
    let health = db.health_score().unwrap_or(95);
    let health_str = if health >= 95 {
        format!("{}%", health).bright_green().to_string()
    } else if health >= 80 {
        format!("{}%", health).yellow().to_string()
    } else {
        format!("{}%", health).bright_red().to_string()
    };
    let cwd = std::env::current_dir()
        .map(|p| {
            let home = std::env::var("HOME").unwrap_or_default();
            let path = p.to_string_lossy().to_string();
            if path.starts_with(&home) {
                format!("~{}", &path[home.len()..])
            } else { path }
        })
        .unwrap_or_else(|_| "?".to_string());
    let cwd = if cwd.len() > 20 {
        let parts: Vec<&str> = cwd.split('/').collect();
        if parts.len() > 2 {
            format!("~/{}", parts.last().copied().unwrap_or(""))
        } else { cwd }
    } else { cwd };
    // No emoji in readline prompt — wide chars break cursor position and Tab completion
    let raw = format!("{} {} > ",
        cwd.bright_cyan(),
        health_str,
    );
    rl_wrap(&raw)
}
