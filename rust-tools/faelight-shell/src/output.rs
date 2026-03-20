#[allow(dead_code)]
// faelight-shell — output formatting
use colored::*;

#[allow(dead_code)]
pub fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return format!("  {}", "No results.".dimmed());
    }

    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let mut out = String::new();

    // Header
    out.push_str("  ");
    for (i, h) in headers.iter().enumerate() {
        out.push_str(&format!("{:<width$}  ", h.bright_white().bold(), width = widths[i]));
    }
    out.push('\n');

    // Separator
    out.push_str("  ");
    for w in &widths {
        out.push_str(&"─".repeat(w + 2).dimmed().to_string());
    }
    out.push('\n');

    // Rows
    for row in rows {
        out.push_str("  ");
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                out.push_str(&format!("{:<width$}  ", cell.dimmed(), width = widths[i]));
            }
        }
        out.push('\n');
    }

    out
}

#[allow(dead_code)]
pub fn section(title: &str) -> String {
    format!("\n{}\n{}",
        format!("  ╭─ {} ", title).bright_cyan(),
        "".to_string()
    )
}
