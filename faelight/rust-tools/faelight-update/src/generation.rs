// generation.rs -- INT-074 Phase 2: the generation browser.
//
// Data layer: read the NixOS system-profile generations into structured form, and provide the
// store path for any generation so two can be closure-diffed via nvd. The TUI lives below /
// alongside; this part is pure + unit-testable.

use std::process::Command;

/// One NixOS system generation.
#[derive(Debug, Clone)]
pub struct Generation {
    pub number: u32,
    pub date: String,          // "2026-06-27 00:08:36"
    pub nixos_version: String, // "26.05.20260618.e8210c6"
    pub kernel: String,        // "6.18.35"
    pub commit: String,        // Configuration Revision (git rev), may end "-dirty" or be "--"
    pub intent: String,        // INT-NNN shipped in this commit (from commit message), or "--"
    pub current: bool,         // the running generation
}

/// Look up the INT-NNN tag in a commit's subject line (commits are intent-tagged).
fn intent_for_commit(commit: &str) -> String {
    let rev = commit.trim_end_matches("-dirty");
    if rev.is_empty() || rev == "--" {
        return "--".to_string();
    }
    let out = Command::new("git")
        .args(["-C", &flake_dir(), "log", "-1", "--format=%s", rev])
        .output();
    let subject = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return "--".to_string(),
    };
    // Find INT-NNN (case-insensitive) in the subject.
    let upper = subject.to_uppercase();
    if let Some(pos) = upper.find("INT-") {
        let rest = &subject[pos + 4..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return format!("INT-{digits}");
        }
    }
    "--".to_string()
}

fn flake_dir() -> String {
    std::env::var("HOME")
        .map(|h| format!("{h}/0-core"))
        .unwrap_or_else(|_| ".".into())
}

/// Store-profile path for a generation number.
pub fn gen_path(n: u32) -> String {
    format!("/nix/var/nix/profiles/system-{n}-link")
}

/// List all system generations, newest first.
pub fn list_generations() -> Vec<Generation> {
    let out = Command::new("nixos-rebuild")
        .arg("list-generations")
        .output();
    let stdout = match out {
        Ok(o) if o.status.success() => o.stdout,
        _ => return Vec::new(),
    };
    parse_generations(&String::from_utf8_lossy(&stdout))
}

/// Parse the `nixos-rebuild list-generations` table. Split into its own fn for testing.
/// Columns: Generation  Build-date(2 cols)  NixOS-version  Kernel  Config-Revision  Spec  Current
fn parse_generations(text: &str) -> Vec<Generation> {
    let mut gens = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("Generation") {
            continue; // header / blank
        }
        let cols: Vec<&str> = t.split_whitespace().collect();
        // Minimum: number, date, time, version, kernel, ... current
        if cols.len() < 6 {
            continue;
        }
        let number = match cols[0].parse::<u32>() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let date = format!("{} {}", cols[1], cols[2]);
        let nixos_version = cols[3].to_string();
        let kernel = cols[4].to_string();
        // Configuration Revision is col 5 when present; some rows may lack it.
        let commit = cols
            .get(5)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "--".into());
        // "Current" is the last column: True/False.
        let current = cols.last().map(|s| *s == "True").unwrap_or(false);
        let intent = intent_for_commit(&commit);
        gens.push(Generation {
            number,
            date,
            nixos_version,
            kernel,
            commit,
            intent,
            current,
        });
    }
    // newest first
    gens.sort_by(|a, b| b.number.cmp(&a.number));
    gens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_generation_table() {
        let sample = "\
Generation  Build-date           NixOS version           Kernel   Configuration Revision                          Specialisation  Current
254         2026-06-27 00:08:36  26.05.20260618.e8210c6  6.18.35  186156380f750511520c928105a4ae0971bd98b7        []              True
253         2026-06-27 00:00:13  26.05.20260618.e8210c6  6.18.35  265f548edcb88bf88923e2899c0d490324507537-dirty  []              False
";
        let gens = parse_generations(sample);
        assert_eq!(gens.len(), 2);
        // newest first
        assert_eq!(gens[0].number, 254);
        assert!(gens[0].current);
        assert_eq!(gens[0].kernel, "6.18.35");
        assert!(gens[0].commit.starts_with("18615638"));
        assert_eq!(gens[1].number, 253);
        assert!(!gens[1].current);
        assert!(gens[1].commit.ends_with("-dirty"));
    }

    #[test]
    fn gen_path_format() {
        assert_eq!(gen_path(254), "/nix/var/nix/profiles/system-254-link");
    }
}

// ── TUI: the generation browser ──────────────────────────────────────────────
// A candy-neon ratatui timeline. Newest gen at top. j/k+arrows navigate; Space marks a second
// generation; d/Enter diffs (selected vs marked, or selected vs current) via nvd; r = gated
// rollback; q quits. Colors pulled from faelight-core::theme for forest consistency.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

// Candy-neon palette (mirrors faelight-core::theme; ratatui needs Color::Rgb).
const C_BG: Color = Color::Rgb(17, 20, 15); // near-black forest
const C_BG_SEL: Color = Color::Rgb(45, 52, 38);
const C_LIME: Color = Color::Rgb(57, 255, 20); // current / selected glow
const C_FOREST: Color = Color::Rgb(107, 227, 163);
const C_AQUA: Color = Color::Rgb(50, 220, 255); // commit hashes
const C_AMBER: Color = Color::Rgb(255, 200, 50); // dirty / mark
const C_CORAL: Color = Color::Rgb(255, 80, 80); // rollback / danger
const C_TEXT: Color = Color::Rgb(215, 224, 218);
const C_DIM: Color = Color::Rgb(120, 140, 130);
const C_PURPLE: Color = Color::Rgb(180, 130, 255); // intent (forest philosophy color)

struct GenBrowser {
    gens: Vec<Generation>,
    selected: usize,
    marked: Option<usize>,
    status: String,
}

impl GenBrowser {
    fn new(gens: Vec<Generation>) -> Self {
        Self {
            gens,
            selected: 0,
            marked: None,
            status: "j/k move · space mark · d diff · r rollback · q quit".into(),
        }
    }
    fn next(&mut self) {
        if !self.gens.is_empty() {
            self.selected = (self.selected + 1).min(self.gens.len() - 1);
        }
    }
    fn prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
    fn mark(&mut self) {
        self.marked = Some(self.selected);
        self.status = format!(
            "marked gen {} -- move to another and press d to diff",
            self.gens[self.selected].number
        );
    }
}

/// Launch the generation browser TUI.
pub fn run_generation_browser() -> io::Result<()> {
    let gens = list_generations();
    if gens.is_empty() {
        println!("No generations found (is this a NixOS system profile?).");
        return Ok(());
    }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = GenBrowser::new(gens);
    let res = run_loop(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut GenBrowser,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('j') | KeyCode::Down => app.next(),
                KeyCode::Char('k') | KeyCode::Up => app.prev(),
                KeyCode::Char(' ') => app.mark(),
                KeyCode::Char('d') | KeyCode::Enter => {
                    // Diff: marked (or current) vs selected. Leave the TUI, run nvd, pause.
                    let sel = app.gens[app.selected].number;
                    let other = match app.marked {
                        Some(m) => app.gens[m].number,
                        None => app
                            .gens
                            .iter()
                            .find(|g| g.current)
                            .map(|g| g.number)
                            .unwrap_or(sel),
                    };
                    run_diff(terminal, other, sel)?;
                }
                KeyCode::Char('r') => {
                    let n = app.gens[app.selected].number;
                    run_rollback(terminal, app, n)?;
                }
                _ => {}
            }
        }
    }
}

fn render(f: &mut Frame, app: &GenBrowser) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header.
    let header = Paragraph::new(Line::from(vec![
        Span::styled("❄ ", Style::default().fg(C_AQUA)),
        Span::styled(
            "Generation Browser",
            Style::default().fg(C_LIME).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  --  the forest's history", Style::default().fg(C_DIM)),
    ]))
    .style(Style::default().bg(C_BG));
    f.render_widget(header, chunks[0]);

    // Timeline list.
    let rows: Vec<ListItem> = app
        .gens
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let is_sel = i == app.selected;
            let is_marked = app.marked == Some(i);
            let bg = if is_sel { C_BG_SEL } else { C_BG };
            let marker = if g.current {
                "●"
            } else if is_marked {
                "◆"
            } else {
                "○"
            };
            let marker_color = if g.current {
                C_LIME
            } else if is_marked {
                C_AMBER
            } else {
                C_DIM
            };
            let num_color = if g.current { C_LIME } else { C_FOREST };
            let commit_short: String = g.commit.chars().take(8).collect();
            let dirty = g.commit.ends_with("-dirty");
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {marker} "),
                    Style::default().fg(marker_color).bg(bg),
                ),
                Span::styled(
                    format!("{:>4}", g.number),
                    Style::default()
                        .fg(num_color)
                        .bg(bg)
                        .add_modifier(if is_sel {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(format!("  {}", g.date), Style::default().fg(C_DIM).bg(bg)),
                Span::styled(
                    format!("  {}", g.kernel),
                    Style::default().fg(C_TEXT).bg(bg),
                ),
                Span::styled(
                    format!("  {}", g.nixos_version),
                    Style::default().fg(C_DIM).bg(bg),
                ),
                Span::styled(
                    format!("  {commit_short}"),
                    Style::default().fg(C_AQUA).bg(bg),
                ),
                Span::styled(
                    if dirty { " *" } else { "" },
                    Style::default().fg(C_AMBER).bg(bg),
                ),
                Span::styled(
                    if g.intent != "--" {
                        format!("  {}", g.intent)
                    } else {
                        String::new()
                    },
                    Style::default().fg(C_PURPLE).bg(bg),
                ),
                Span::styled(
                    if g.current { "  (current)" } else { "" },
                    Style::default().fg(C_LIME).bg(bg),
                ),
            ]))
        })
        .collect();
    let list = List::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM))
                .style(Style::default().bg(C_BG)),
        )
        .style(Style::default().bg(C_BG));
    f.render_widget(list, chunks[1]);

    // Status bar.
    let marked_txt = match app.marked {
        Some(m) => format!("  [marked: gen {}]", app.gens[m].number),
        None => String::new(),
    };
    let status = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {}", app.status), Style::default().fg(C_DIM)),
        Span::styled("  [r = rollback]", Style::default().fg(C_CORAL)),
        Span::styled(marked_txt, Style::default().fg(C_AMBER)),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(C_DIM)),
    )
    .style(Style::default().bg(C_BG));
    f.render_widget(status, chunks[2]);
}

/// Leave the alt screen, run nvd diff between two generations, wait for a keypress, return.
fn run_diff(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    a: u32,
    b: u32,
) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    println!("\n  ❄ nvd diff: gen {a} -> gen {b}\n");
    let _ = std::process::Command::new("nvd")
        .args(["diff", &gen_path(a), &gen_path(b)])
        .status();
    println!("\n  (press Enter to return to the browser)");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    terminal.clear()?;
    Ok(())
}

/// Gated rollback: confirm, then `sudo nixos-rebuild switch --rollback`-style activation.
fn run_rollback(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut GenBrowser,
    n: u32,
) -> io::Result<()> {
    if app.gens.iter().find(|g| g.current).map(|g| g.number) == Some(n) {
        app.status = format!("gen {n} is already current");
        return Ok(());
    }
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    println!("\n  ⚠  Roll back to generation {n}? This will switch the system. (y/N): ");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
    if buf.trim().to_lowercase() == "y" {
        println!("  → Activating generation {n}...");
        let _ = std::process::Command::new("sudo")
            .args([
                "nix-env",
                "--profile",
                "/nix/var/nix/profiles/system",
                "--switch-generation",
                &n.to_string(),
            ])
            .status();
        let _ = std::process::Command::new("sudo")
            .arg(format!(
                "/nix/var/nix/profiles/system-{n}-link/bin/switch-to-configuration"
            ))
            .arg("switch")
            .status();
        println!("  ✓ Rolled back to gen {n}. (press Enter)");
    } else {
        println!("  Aborted. (press Enter)");
    }
    let mut b2 = String::new();
    io::stdin().read_line(&mut b2).ok();
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    terminal.clear()?;
    app.gens = list_generations();
    app.status = "rolled back -- list refreshed".into();
    Ok(())
}
