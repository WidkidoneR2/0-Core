// faelight-shell — live context-aware prompt
use crate::db::ForestDb;
use colored::*;

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
