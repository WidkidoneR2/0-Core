// INT-253: gt -- Git Workflow as Ratatui TUI
// ratatui + crossterm + git2. Same ConditionalEventHandler pattern as INT-250.
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::{Repository, Status, StatusOptions};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
#[derive(Debug, Clone, PartialEq)]
pub enum FileSection {
    Staged,
    Unstaged,
    Untracked,
}
pub struct GitFile {
    pub path: String,
    pub section: FileSection,
}
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum Mode {
    FileList,
    CommitInput,
    Pushing,
    Message(String),
}
pub fn run_git_tui(core_root: &str, active_intent: Option<&str>) {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => { let _ = disable_raw_mode(); return; }
    };
    run_loop(&mut terminal, core_root, active_intent);
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}
fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, core_root: &str, active_intent: Option<&str>) {
    let mut files = load_git_status(core_root);
    let mut list_state = ListState::default();
    if !files.is_empty() { list_state.select(Some(0)); }
    let mut mode = Mode::FileList;
    let mut commit_msg = String::new();
    // Pre-fill with active intent
    if let Some(intent) = active_intent {
        commit_msg = format!("{}: ", intent);
    }
    let mut status_msg = String::new();
    let mut diff_scroll: u16 = 0;
    loop {
        let branch = get_branch(core_root);
        let ahead_behind = get_ahead_behind(core_root);
        let diff = get_diff(core_root, &files, list_state.selected());
        let _ = terminal.draw(|f| {
            draw_ui(f, &files, &mut list_state, &mode, &commit_msg,
                    &branch, &ahead_behind, &diff, diff_scroll, &status_msg);
        });
        if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = event::read() {
            status_msg.clear();
            match &mode {
                Mode::CommitInput => {
                    match (code, modifiers) {
                        (KeyCode::Esc, _) => {
                            mode = Mode::FileList;
                        }
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            if !commit_msg.trim().is_empty() {
                                match do_commit(core_root, &commit_msg) {
                                    Ok(_) => {
                                        status_msg = format!("Committed: {}", &commit_msg.lines().next().unwrap_or("").chars().take(40).collect::<String>());
                                        commit_msg = active_intent.map(|i| format!("{}: ", i)).unwrap_or_default();
                                        files = load_git_status(core_root);
                                        list_state.select(if files.is_empty() { None } else { Some(0) });
                                        mode = Mode::FileList;
                                    }
                                    Err(e) => { status_msg = format!("Commit failed: {}", e); mode = Mode::FileList; }
                                }
                            }
                        }
                        (KeyCode::Enter, _) => { commit_msg.push('\n'); }
                        (KeyCode::Backspace, _) => { commit_msg.pop(); }
                        (KeyCode::Char(c), _) => { commit_msg.push(c); }
                        _ => {}
                    }
                }
                Mode::Message(_) => {
                    mode = Mode::FileList;
                }
                Mode::Pushing => {}
                Mode::FileList => {
                    match (code, modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Esc, _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return,
                        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                            let i = list_state.selected().unwrap_or(0);
                            let next = (i + 1).min(files.len().saturating_sub(1));
                            list_state.select(Some(next));
                            diff_scroll = 0;
                        }
                        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                            let i = list_state.selected().unwrap_or(0);
                            list_state.select(Some(i.saturating_sub(1)));
                            diff_scroll = 0;
                        }
                        (KeyCode::Char('s'), KeyModifiers::NONE) => {
                            if let Some(idx) = list_state.selected() {
                                if idx < files.len() {
                                    let path = files[idx].path.clone();
                                    let sec = files[idx].section.clone();
                                    if sec == FileSection::Unstaged || sec == FileSection::Untracked {
                                        match stage_file(core_root, &path) {
                                            Ok(_) => { files = load_git_status(core_root); }
                                            Err(e) => { status_msg = format!("Stage failed: {}", e); }
                                        }
                                    }
                                }
                            }
                        }
                        (KeyCode::Char('u'), KeyModifiers::NONE) => {
                            if let Some(idx) = list_state.selected() {
                                if idx < files.len() && files[idx].section == FileSection::Staged {
                                    let path = files[idx].path.clone();
                                    match unstage_file(core_root, &path) {
                                        Ok(_) => { files = load_git_status(core_root); }
                                        Err(e) => { status_msg = format!("Unstage failed: {}", e); }
                                    }
                                }
                            }
                        }
                        (KeyCode::Char('a'), KeyModifiers::NONE) => {
                            match stage_all(core_root) {
                                Ok(_) => { files = load_git_status(core_root); status_msg = "All staged".to_string(); }
                                Err(e) => { status_msg = format!("Stage all failed: {}", e); }
                            }
                        }
                        (KeyCode::Char('c'), KeyModifiers::NONE) => {
                            mode = Mode::CommitInput;
                        }
                        (KeyCode::Char('p'), KeyModifiers::NONE) => {
                            match do_push(core_root) {
                                Ok(out) => { status_msg = out; files = load_git_status(core_root); }
                                Err(e) => { status_msg = format!("Push failed: {}", e); }
                            }
                        }
                        (KeyCode::Char('r'), KeyModifiers::NONE) => {
                            files = load_git_status(core_root);
                            status_msg = "Refreshed".to_string();
                        }
                        (KeyCode::PageDown, _) => { diff_scroll = diff_scroll.saturating_add(5); }
                        (KeyCode::PageUp, _) => { diff_scroll = diff_scroll.saturating_sub(5); }
                        _ => {}
                    }
                }
            }
        }
    }
}
fn draw_ui(
    f: &mut Frame,
    files: &[GitFile],
    list_state: &mut ListState,
    mode: &Mode,
    commit_msg: &str,
    branch: &str,
    ahead_behind: &str,
    diff: &str,
    diff_scroll: u16,
    status_msg: &str,
) {
    let area = f.area();
    // Top bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    // Header
    let staged = files.iter().filter(|f| f.section == FileSection::Staged).count();
    let unstaged = files.iter().filter(|f| f.section == FileSection::Unstaged).count();
    let untracked = files.iter().filter(|f| f.section == FileSection::Untracked).count();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("  🌿 ", Style::default().fg(Color::Rgb(107, 227, 163))),
        Span::styled(branch, Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  {}", ahead_behind), Style::default().fg(Color::Rgb(92, 200, 255))),
        Span::styled(format!("   staged: {} ", staged), Style::default().fg(Color::Rgb(107, 227, 163))),
        Span::styled(format!("unstaged: {} ", unstaged), Style::default().fg(Color::Rgb(245, 193, 119))),
        Span::styled(format!("untracked: {}", untracked), Style::default().fg(Color::Rgb(180, 180, 180))),
    ]))
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(50, 80, 55))));
    f.render_widget(header, chunks[0]);
    // Main area: file list | diff
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);
    // File list
    if matches!(mode, Mode::CommitInput) {
        // Commit input widget
        let input_block = Block::default()
            .title(Line::from(vec![
                Span::styled(" Commit message  Ctrl+S to commit  Esc to cancel ",
                    Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD))
            ]))
            .borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(107, 227, 163)));
        let input_text = Paragraph::new(commit_msg)
            .style(Style::default().fg(Color::Rgb(215, 224, 218)))
            .block(input_block);
        f.render_widget(input_text, main_chunks[0]);
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        let mut last_section: Option<&FileSection> = None;
        for file in files.iter() {
            if last_section != Some(&file.section) {
                let header_text = match file.section {
                    FileSection::Staged => "  ── Staged ──────────────────",
                    FileSection::Unstaged => "  ── Unstaged ────────────────",
                    FileSection::Untracked => "  ── Untracked ───────────────",
                };
                let header_color = match file.section {
                    FileSection::Staged => Color::Rgb(107, 227, 163),
                    FileSection::Unstaged => Color::Rgb(245, 193, 119),
                    FileSection::Untracked => Color::Rgb(180, 180, 180),
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(header_text, Style::default().fg(header_color).add_modifier(Modifier::DIM))
                ])));
                last_section = Some(&file.section);
            }
            let prefix = match file.section {
                FileSection::Staged => "  + ",
                FileSection::Unstaged => "  ~ ",
                FileSection::Untracked => "  ? ",
            };
            let color = match file.section {
                FileSection::Staged => Color::Rgb(107, 227, 163),
                FileSection::Unstaged => Color::Rgb(245, 193, 119),
                FileSection::Untracked => Color::Rgb(180, 180, 180),
            };
            let name = file.path.split('/').last().unwrap_or(&file.path);
            items.push(ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(color)),
                Span::styled(name.to_string(), Style::default().fg(Color::Rgb(215, 224, 218))),
            ])));
        }
        if items.is_empty() {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  ✅ Working tree clean", Style::default().fg(Color::Rgb(107, 227, 163))),
            ])));
        }
        let file_list = List::new(items)
            .block(Block::default()
                .title(Line::from(vec![Span::styled(" Files ", Style::default().fg(Color::Rgb(107, 227, 163)).add_modifier(Modifier::BOLD))]))
                .borders(Borders::ALL).border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(50, 80, 55))))
            .highlight_style(Style::default().bg(Color::Rgb(30, 50, 35)).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");
        f.render_stateful_widget(file_list, main_chunks[0], list_state);
    }
    // Diff preview
    let diff_lines: Vec<Line> = diff.lines().map(|l| {
        let color = if l.starts_with('+') && !l.starts_with("+++") {
            Color::Rgb(107, 227, 163)
        } else if l.starts_with('-') && !l.starts_with("---") {
            Color::Rgb(230, 126, 128)
        } else if l.starts_with("@@") {
            Color::Rgb(92, 200, 255)
        } else {
            Color::Rgb(180, 190, 183)
        };
        Line::from(Span::styled(l.to_string(), Style::default().fg(color)))
    }).collect();
    let diff_widget = Paragraph::new(diff_lines)
        .scroll((diff_scroll, 0))
        .block(Block::default()
            .title(Line::from(vec![Span::styled(" Diff  PgUp/PgDn scroll ", Style::default().fg(Color::Rgb(119, 143, 127)))]))
            .borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(50, 80, 55))));
    f.render_widget(diff_widget, main_chunks[1]);
    // Footer
    let footer_text = if matches!(mode, Mode::CommitInput) {
        "  Ctrl+S commit  Esc cancel"
    } else {
        "  s stage  u unstage  a stage-all  c commit  p push  r refresh  q quit"
    };
    let status_span = if !status_msg.is_empty() {
        format!("   ✓ {}", status_msg)
    } else { String::new() };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(footer_text, Style::default().fg(Color::Rgb(119, 143, 127))),
        Span::styled(status_span, Style::default().fg(Color::Rgb(107, 227, 163))),
    ]))
    .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::Rgb(50, 70, 55))));
    f.render_widget(footer, chunks[2]);
}
fn load_git_status(core_root: &str) -> Vec<GitFile> {
    let repo = match Repository::open(core_root) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);
    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let mut files: Vec<GitFile> = Vec::new();
    // Staged first
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();
        if status.intersects(Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED | Status::INDEX_RENAMED) {
            files.push(GitFile { path: path.clone(), section: FileSection::Staged });
        }
    }
    // Unstaged
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();
        if status.intersects(Status::WT_MODIFIED | Status::WT_DELETED) {
            files.push(GitFile { path, section: FileSection::Unstaged });
        }
    }
    // Untracked
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();
        if status.contains(Status::WT_NEW) {
            files.push(GitFile { path, section: FileSection::Untracked });
        }
    }
    files
}
fn get_branch(core_root: &str) -> String {
    let repo = match Repository::open(core_root) {
        Ok(r) => r,
        Err(_) => return "unknown".to_string(),
    };
    repo.head().ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "detached".to_string())
}
fn get_ahead_behind(core_root: &str) -> String {
    let out = std::process::Command::new("git")
        .args(["-C", core_root, "status", "--short", "--branch"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").to_string())
        .unwrap_or_default();
    if out.contains("ahead") || out.contains("behind") {
        out.split('[').nth(1).and_then(|s| s.split(']').next())
            .unwrap_or("").to_string()
    } else { String::new() }
}
fn get_diff(core_root: &str, files: &[GitFile], selected: Option<usize>) -> String {
    let idx = match selected { Some(i) => i, None => return String::new() };
    if idx >= files.len() { return String::new(); }
    let file = &files[idx];
    let args = match file.section {
        FileSection::Staged => vec!["-C", core_root, "diff", "--cached", "--", &file.path],
        FileSection::Unstaged => vec!["-C", core_root, "diff", "--", &file.path],
        FileSection::Untracked => return format!("(untracked file: {})", file.path),
    };
    std::process::Command::new("git")
        .args(&args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}
fn stage_file(core_root: &str, path: &str) -> Result<(), String> {
    let status = std::process::Command::new("git")
        .args(["-C", core_root, "add", path])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err("git add failed".to_string()) }
}
fn unstage_file(core_root: &str, path: &str) -> Result<(), String> {
    let status = std::process::Command::new("git")
        .args(["-C", core_root, "reset", "HEAD", "--", path])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err("git reset failed".to_string()) }
}
fn stage_all(core_root: &str) -> Result<(), String> {
    let status = std::process::Command::new("git")
        .args(["-C", core_root, "add", "-A"])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err("git add -A failed".to_string()) }
}
fn do_commit(core_root: &str, msg: &str) -> Result<(), String> {
    let status = std::process::Command::new("git")
        .args(["-C", core_root, "commit", "-m", msg.trim()])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err("git commit failed".to_string()) }
}
fn do_push(core_root: &str) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .args(["-C", core_root, "push"])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok("Pushed to origin".to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}
