//! faelight-diff v1.0.0 -- The Forest Sees What Changed
//! Phase 1: file diff, two-panel layout, forest colors, navigation
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::process::Command;
// Forest color palette
const FG: Color = Color::Rgb(0xda, 0xe0, 0xd7);
const BG: Color = Color::Rgb(0x11, 0x14, 0x0f);
const GREEN: Color = Color::Rgb(0xa3, 0xe3, 0x6b);
const AMBER: Color = Color::Rgb(0xff, 0xaa, 0x00);
const RED: Color = Color::Rgb(0xff, 0x6b, 0x6b);
const DIM: Color = Color::Rgb(0x55, 0x60, 0x50);
const BLUE: Color = Color::Rgb(0x6b, 0xa3, 0xe3);
#[derive(Clone, PartialEq, Eq)]
enum LineKind {
    Added,
    Removed,
    Changed,
    Context,
    Header,
}
#[derive(Clone)]
struct DiffLine {
    line_no: Option<usize>,
    content: String,
    kind: LineKind,
}
struct App {
    left: Vec<DiffLine>,
    right: Vec<DiffLine>,
    scroll: usize,
    changes: Vec<usize>, // row indices of change lines
    change_idx: usize,
    left_title: String,
    right_title: String,
    status: String,
    _mode: AppMode,
}
#[allow(dead_code)]
enum AppMode {
    FileDiff,
    GitDiff,
    DirDiff,
    Help,
}
impl App {
    fn new(left_title: String, right_title: String, left: Vec<DiffLine>, right: Vec<DiffLine>) -> Self {
        let changes: Vec<usize> = left.iter().enumerate()
            .filter(|(_, l)| matches!(l.kind, LineKind::Added | LineKind::Removed | LineKind::Changed))
            .map(|(i, _)| i)
            .collect();
        let change_count = changes.len();
        let mut app = Self {
            left, right, scroll: 0, changes, change_idx: 0,
            left_title, right_title,
            status: String::new(),
            _mode: AppMode::FileDiff,
        };
        app.status = format!("{} changes", change_count);
        app
    }
    fn scroll_down(&mut self) {
        let max = self.left.len().saturating_sub(1);
        if self.scroll < max { self.scroll += 1; }
    }
    fn scroll_up(&mut self) {
        if self.scroll > 0 { self.scroll -= 1; }
    }
    fn next_change(&mut self) {
        if self.changes.is_empty() { return; }
        if self.change_idx + 1 < self.changes.len() { self.change_idx += 1; }
        self.scroll = self.changes[self.change_idx];
    }
    fn prev_change(&mut self) {
        if self.changes.is_empty() { return; }
        if self.change_idx > 0 { self.change_idx -= 1; }
        self.scroll = self.changes[self.change_idx];
    }
}
fn diff_files(file_a: &str, file_b: &str) -> (Vec<DiffLine>, Vec<DiffLine>) {
    let content_a = std::fs::read_to_string(file_a).unwrap_or_default();
    let _content_b = std::fs::read_to_string(file_b).unwrap_or_default();
    let lines_a: Vec<&str> = content_a.lines().collect();
    // Use system diff for reliable output
    let output = Command::new("diff")
        .args(["-u", file_a, file_b])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![], stderr: vec![],
        });
    let diff_out = String::from_utf8_lossy(&output.stdout).to_string();
    // Parse unified diff
    parse_unified_diff(&lines_a, &diff_out)
}
fn parse_unified_diff(lines_a: &[&str], diff: &str) -> (Vec<DiffLine>, Vec<DiffLine>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    if diff.is_empty() {
        // Files identical
        for (i, line) in lines_a.iter().enumerate() {
            let dl = DiffLine { line_no: Some(i + 1), content: line.to_string(), kind: LineKind::Context };
            left.push(dl.clone());
            right.push(dl);
        }
        return (left, right);
    }
    let mut left_no = 1usize;
    let mut right_no = 1usize;
    for line in diff.lines() {
        if line.starts_with("---") || line.starts_with("+++") {
            continue;
        }
        if line.starts_with("@@") {
            let hdr = DiffLine { line_no: None, content: line.to_string(), kind: LineKind::Header };
            left.push(hdr.clone());
            right.push(hdr);
            // Parse @@ -L,N +L,N @@
            if let Some(rest) = line.strip_prefix("@@ ") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() >= 2 {
                    let l_str = parts[0].trim_start_matches('-');
                    let r_str = parts[1].trim_start_matches('+');
                    left_no = l_str.split(',').next().unwrap_or("1").parse().unwrap_or(left_no);
                    right_no = r_str.split(',').next().unwrap_or("1").parse().unwrap_or(right_no);
                }
            }
        } else if let Some(content) = line.strip_prefix('-') {
            left.push(DiffLine { line_no: Some(left_no), content: content.to_string(), kind: LineKind::Removed });
            right.push(DiffLine { line_no: None, content: String::new(), kind: LineKind::Added });
            left_no += 1;
        } else if let Some(content) = line.strip_prefix('+') {
            if left.last().map(|l| l.kind == LineKind::Removed).unwrap_or(false)
                && right.last().map(|r| r.content.is_empty()).unwrap_or(false) {
                // Pair removed/added as Changed
                if let Some(l) = left.last_mut() { l.kind = LineKind::Changed; }
                right.pop();
                right.push(DiffLine { line_no: Some(right_no), content: content.to_string(), kind: LineKind::Changed });
            } else {
                left.push(DiffLine { line_no: None, content: String::new(), kind: LineKind::Removed });
                right.push(DiffLine { line_no: Some(right_no), content: content.to_string(), kind: LineKind::Added });
            }
            right_no += 1;
        } else if let Some(content) = line.strip_prefix(' ') {
            let ctx_l = DiffLine { line_no: Some(left_no), content: content.to_string(), kind: LineKind::Context };
            let ctx_r = DiffLine { line_no: Some(right_no), content: content.to_string(), kind: LineKind::Context };
            left.push(ctx_l);
            right.push(ctx_r);
            left_no += 1;
            right_no += 1;
        }
    }
    (left, right)
}
fn diff_git(git_ref: Option<&str>) -> (Vec<DiffLine>, Vec<DiffLine>, String, String) {
    let reference = git_ref.unwrap_or("HEAD");
    let output = Command::new("git")
        .args(["diff", reference])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![], stderr: vec![],
        });
    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    let left_title = format!("{}", reference);
    let right_title = "working tree".to_string();
    let (left, right) = parse_git_diff_output(&diff);
    (left, right, left_title, right_title)
}
fn parse_git_diff_output(diff: &str) -> (Vec<DiffLine>, Vec<DiffLine>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut left_no = 1usize;
    let mut right_no = 1usize;
    for line in diff.lines() {
        if line.starts_with("diff --git") || line.starts_with("index ") || line.starts_with("new file") || line.starts_with("deleted file") {
            let hdr = DiffLine { line_no: None, content: line.to_string(), kind: LineKind::Header };
            left.push(hdr.clone()); right.push(hdr);
        } else if line.starts_with("---") || line.starts_with("+++") {
            let hdr = DiffLine { line_no: None, content: line.to_string(), kind: LineKind::Header };
            left.push(hdr.clone()); right.push(hdr);
        } else if line.starts_with("@@") {
            let hdr = DiffLine { line_no: None, content: line.to_string(), kind: LineKind::Header };
            left.push(hdr.clone()); right.push(hdr);
            if let Some(rest) = line.strip_prefix("@@ ") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() >= 2 {
                    left_no = parts[0].trim_start_matches('-').split(',').next().unwrap_or("1").parse().unwrap_or(left_no);
                    right_no = parts[1].trim_start_matches('+').split(',').next().unwrap_or("1").parse().unwrap_or(right_no);
                }
            }
        } else if let Some(c) = line.strip_prefix('-') {
            left.push(DiffLine { line_no: Some(left_no), content: c.to_string(), kind: LineKind::Removed });
            right.push(DiffLine { line_no: None, content: String::new(), kind: LineKind::Removed });
            left_no += 1;
        } else if let Some(c) = line.strip_prefix('+') {
            left.push(DiffLine { line_no: None, content: String::new(), kind: LineKind::Added });
            right.push(DiffLine { line_no: Some(right_no), content: c.to_string(), kind: LineKind::Added });
            right_no += 1;
        } else if let Some(c) = line.strip_prefix(' ') {
            left.push(DiffLine { line_no: Some(left_no), content: c.to_string(), kind: LineKind::Context });
            right.push(DiffLine { line_no: Some(right_no), content: c.to_string(), kind: LineKind::Context });
            left_no += 1; right_no += 1;
        }
    }
    (left, right)
}
fn line_style(kind: &LineKind) -> Style {
    match kind {
        LineKind::Added => Style::default().fg(GREEN).bg(Color::Rgb(0x1a, 0x2a, 0x10)),
        LineKind::Removed => Style::default().fg(RED).bg(Color::Rgb(0x2a, 0x10, 0x10)),
        LineKind::Changed => Style::default().fg(AMBER).bg(Color::Rgb(0x2a, 0x20, 0x05)),
        LineKind::Context => Style::default().fg(FG),
        LineKind::Header => Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
    }
}
fn render_panel(f: &mut Frame, area: Rect, lines: &[DiffLine], scroll: usize, title: &str) {
    let block = Block::default()
        .title(Span::styled(format!(" {} ", title), Style::default().fg(GREEN).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let visible_height = inner.height as usize;
    let visible_lines: Vec<Line> = lines.iter()
        .skip(scroll)
        .take(visible_height)
        .map(|dl| {
            let no_str = match dl.line_no {
                Some(n) => format!("{:4} ", n),
                None => "     ".to_string(),
            };
            let style = line_style(&dl.kind);
            let prefix = match dl.kind {
                LineKind::Added => "+ ",
                LineKind::Removed => "- ",
                LineKind::Changed => "~ ",
                LineKind::Header => "  ",
                LineKind::Context => "  ",
            };
            let content = format!("{}{}{}", no_str, prefix, dl.content);
            let truncated = if content.len() > inner.width as usize {
                format!("{}…", &content[..inner.width as usize - 1])
            } else {
                content
            };
            Line::from(Span::styled(truncated, style))
        })
        .collect();
    let para = Paragraph::new(visible_lines).style(Style::default().bg(BG));
    f.render_widget(para, inner);
}
fn render_ui(f: &mut Frame, app: &App) {
    let size = f.area();
    // Clear background
    f.render_widget(
        Block::default().style(Style::default().bg(BG)),
        size,
    );
    // Layout: header + panels + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // panels
            Constraint::Length(1), // status
        ])
        .split(size);
    // Header
    let header_text = format!(
        " faelight-diff  {}  vs  {} ",
        app.left_title, app.right_title
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(BG).bg(GREEN).add_modifier(Modifier::BOLD));
    f.render_widget(header, chunks[0]);
    // Two panels
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    render_panel(f, panels[0], &app.left, app.scroll, &app.left_title);
    render_panel(f, panels[1], &app.right, app.scroll, &app.right_title);
    // Status bar
    let change_hint = if !app.changes.is_empty() {
        format!("  change {}/{}", app.change_idx + 1, app.changes.len())
    } else {
        "  no changes".to_string()
    };
    let hint_style = Style::default().fg(DIM).bg(BG);
    let change_color = if app.changes.is_empty() { GREEN } else { AMBER };
    let status_line = Line::from(vec![
        Span::styled("  │  ", hint_style),
        Span::styled("j/k", Style::default().fg(FG).bg(BG)),
        Span::styled(" scroll    ", hint_style),
        Span::styled("]", Style::default().fg(if app.changes.is_empty() { DIM } else { AMBER }).bg(BG)),
        Span::styled(" next    ", hint_style),
        Span::styled("[", Style::default().fg(if app.changes.is_empty() { DIM } else { AMBER }).bg(BG)),
        Span::styled(" prev    ", hint_style),
        Span::styled("q", Style::default().fg(RED).bg(BG)),
        Span::styled(" quit    ", hint_style),
        Span::styled(&change_hint, Style::default().fg(change_color).bg(BG)),
        Span::styled("  │  ", hint_style),
        Span::styled(&app.status, Style::default().fg(change_color).bg(BG)),
    ]);
    let status_bar = Paragraph::new(status_line);
    f.render_widget(status_bar, chunks[2]);
}
fn run_app(mut app: App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    loop {
        terminal.draw(|f| render_ui(f, &app))?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                    (KeyCode::Char('j'), _) | (KeyCode::Down, _) => app.scroll_down(),
                    (KeyCode::Char('k'), _) | (KeyCode::Up, _) => app.scroll_up(),
                    (KeyCode::Char(']'), _) => app.next_change(),
                    (KeyCode::Char('['), _) => app.prev_change(),
                    (KeyCode::Char('G'), _) => app.scroll = app.left.len().saturating_sub(1),
                    (KeyCode::Char('g'), _) => app.scroll = 0,
                    _ => {}
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
fn print_usage() {
    println!("faelight-diff v1.0.0 -- The Forest Sees What Changed");
    println!();
    println!("USAGE:");
    println!("  compare file1 file2     side by side file diff");
    println!("  compare --git [REF]     diff against git commit");
    println!("  compare --staged        diff staged changes");
    println!();
    println!("NAVIGATION:");
    println!("  j/k         scroll down/up");
    println!("  ] / [       next / prev change");
    println!("  g / G       top / bottom");
    println!("  q           quit");
}
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_usage();
        return Ok(());
    }
    if args[0] == "--health" {
        println!("faelight-diff v1.0.0 -- healthy");
        return Ok(());
    }
    let app = if args[0] == "--git" || args[0] == "--staged" {
        let git_ref = if args[0] == "--staged" {
            Some("--cached")
        } else {
            args.get(1).map(|s| s.as_str())
        };
        let (left, right, lt, rt) = diff_git(git_ref);
        if left.is_empty() && right.is_empty() {
            println!("No changes found.");
            return Ok(());
        }
        App::new(lt, rt, left, right)
    } else if args.len() >= 2 {
        let file_a = &args[0];
        let file_b = &args[1];
        if !std::path::Path::new(file_a).exists() {
            eprintln!("Cannot compare: {} does not exist.", file_a);
            return Ok(());
        }
        if !std::path::Path::new(file_b).exists() {
            eprintln!("Cannot compare: {} does not exist.", file_b);
            return Ok(());
        }
        let (left, right) = diff_files(file_a, file_b);
        App::new(file_a.clone(), file_b.clone(), left, right)
    } else {
        print_usage();
        return Ok(());
    };
    run_app(app)
}
