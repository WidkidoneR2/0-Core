//! faelight-release TUI — the release ceremony made visible.

use crate::changelog::{ChangelogData, ReleaseStats};
use anyhow::Result;
use crossterm::{cursor, event::{self, Event, KeyCode, KeyEventKind}, execute, terminal};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::path::PathBuf;

const BG:     Color = Color::Rgb(15, 20, 17);
const FG:     Color = Color::Rgb(215, 224, 218);
const ACCENT: Color = Color::Rgb(163, 227, 107);
const DIM:    Color = Color::Rgb(119, 143, 127);
const YELLOW: Color = Color::Rgb(227, 199, 107);
const RED:    Color = Color::Rgb(227, 107, 107);
const BLUE:   Color = Color::Rgb(107, 163, 227);
const CYAN:   Color = Color::Rgb(107, 227, 210);

#[derive(PartialEq)]
enum State {
    Preview,     // showing gathered data
    EditTheme,   // user is typing theme
    Confirm,     // final confirmation before publishing
    Publishing,  // executing release steps
    Done,        // release complete
    Aborted,     // user quit
}

pub struct ReleaseTui {
    version: String,
    theme: String,
    data: ChangelogData,
    stats: ReleaseStats,
    state: State,
    cursor_pos: usize,
    log: Vec<String>,
}

impl ReleaseTui {
    pub fn new(version: String, theme: String, data: ChangelogData, stats: ReleaseStats) -> Self {
        let cursor_pos = theme.len();
        Self { version, theme, data, stats, state: State::Preview, cursor_pos, log: vec![] }
    }

    pub fn run(mut self, core_root: &PathBuf) -> Result<bool> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.draw(f))?;

            if self.state == State::Done || self.state == State::Aborted {
                break;
            }

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press { continue; }
                    match self.state {
                        State::Preview => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                self.state = State::Aborted;
                            }
                            KeyCode::Char('e') => {
                                self.state = State::EditTheme;
                                execute!(io::stdout(), cursor::Show)?;
                            }
                            KeyCode::Enter => {
                                self.state = State::Confirm;
                            }
                            _ => {}
                        },
                        State::EditTheme => match key.code {
                            KeyCode::Enter => {
                                self.state = State::Preview;
                                execute!(io::stdout(), cursor::Hide)?;
                            }
                            KeyCode::Esc => {
                                self.state = State::Preview;
                                execute!(io::stdout(), cursor::Hide)?;
                            }
                            KeyCode::Backspace => {
                                if self.cursor_pos > 0 {
                                    self.theme.remove(self.cursor_pos - 1);
                                    self.cursor_pos -= 1;
                                }
                            }
                            KeyCode::Char(c) => {
                                self.theme.insert(self.cursor_pos, c);
                                self.cursor_pos += 1;
                            }
                            _ => {}
                        },
                        State::Confirm => match key.code {
                            KeyCode::Char('y') | KeyCode::Enter => {
                                self.state = State::Publishing;
                                terminal.draw(|f| self.draw(f))?;
                                self.execute_release(core_root)?;
                                self.state = State::Done;
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                self.state = State::Preview;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }

        terminal::disable_raw_mode()?;
        execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen, cursor::Show)?;

        Ok(self.state == State::Done)
    }

    fn draw(&self, f: &mut ratatui::Frame) {
        let area = f.area();
        f.render_widget(Block::default().style(Style::default().bg(BG)), area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // header
                Constraint::Length(3),  // theme editor
                Constraint::Min(0),     // main content
                Constraint::Length(3),  // footer
            ])
            .split(area);

        // ── HEADER ──────────────────────────────────────────────────────
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(30)])
            .split(chunks[0]);

        let title_color = match self.state {
            State::Publishing => YELLOW,
            State::Done => ACCENT,
            _ => ACCENT,
        };

        let status_label = match self.state {
            State::Preview => " PREVIEW ",
            State::EditTheme => " EDITING ",
            State::Confirm => " CONFIRM ",
            State::Publishing => " PUBLISHING ",
            State::Done => " DONE ✅ ",
            State::Aborted => " ABORTED ",
        };

        let left = Paragraph::new(Line::from(vec![
            Span::styled(" 🌲 ", Style::default().fg(ACCENT)),
            Span::styled("faelight-release", Style::default().fg(FG).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  v{}", self.version), Style::default().fg(DIM)),
        ]))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(title_color)));
        f.render_widget(left, header_chunks[0]);

        let right = Paragraph::new(Line::from(vec![
            Span::styled(status_label, Style::default().fg(title_color).add_modifier(Modifier::BOLD | Modifier::REVERSED)),
        ]))
        .alignment(ratatui::layout::Alignment::Right)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(title_color)));
        f.render_widget(right, header_chunks[1]);

        // ── THEME EDITOR ─────────────────────────────────────────────────
        let editing = self.state == State::EditTheme;
        let theme_style = if editing {
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(FG)
        };
        let theme_block = Block::default()
            .title(Span::styled(
                if editing { " 📝 Release Theme (editing) " } else { " 📝 Release Theme " },
                Style::default().fg(if editing { YELLOW } else { DIM }),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if editing { YELLOW } else { DIM }));

        let theme_display = if editing {
            format!("{}_", self.theme)
        } else {
            self.theme.clone()
        };
        let theme_widget = Paragraph::new(Span::styled(theme_display, theme_style))
            .block(theme_block);
        f.render_widget(theme_widget, chunks[1]);

        // ── MAIN CONTENT ─────────────────────────────────────────────────
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(chunks[2]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(main_chunks[0]);

        // Intents shipped
        let intent_items: Vec<ListItem> = self.data.intents.iter().map(|i| {
            ListItem::new(Line::from(vec![
                Span::styled("  ✅ ", Style::default().fg(ACCENT)),
                Span::styled(format!("INT-{}  ", i.id), Style::default().fg(DIM)),
                Span::styled(i.title.chars().take(40).collect::<String>(), Style::default().fg(FG)),
            ]))
        }).collect();
        let intents_list = List::new(intent_items)
            .block(Block::default()
                .title(Span::styled(
                    format!(" 🎯 Intents Shipped ({}) ", self.data.intents.len()),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ACCENT)))
            .style(Style::default().bg(BG));
        f.render_widget(intents_list, left_chunks[0]);

        // Stats
        let stats_items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("  Health:    ", Style::default().fg(DIM)),
                Span::styled(format!("{}%", self.stats.health), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Commits:   ", Style::default().fg(DIM)),
                Span::styled(format!("{}", self.stats.total_commits), Style::default().fg(FG).add_modifier(Modifier::BOLD)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Tools:     ", Style::default().fg(DIM)),
                Span::styled(format!("{} deployed", self.stats.tools_deployed), Style::default().fg(FG)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Intents:   ", Style::default().fg(DIM)),
                Span::styled(format!("{} complete", self.stats.intents_complete), Style::default().fg(FG)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Since:     ", Style::default().fg(DIM)),
                Span::styled(format!("{} commits", self.data.total_commits), Style::default().fg(CYAN)),
            ])),
        ];
        let stats_list = List::new(stats_items)
            .block(Block::default()
                .title(Span::styled(" 📊 Release Stats ", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BLUE)))
            .style(Style::default().bg(BG));
        f.render_widget(stats_list, left_chunks[1]);

        // Right side — commits breakdown
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(main_chunks[1]);

        // Features & fixes
        let mut commit_items = vec![];
        if !self.data.features.is_empty() {
            commit_items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("  ✨ Features    {}", self.data.features.len()),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            ])));
            for c in self.data.features.iter().take(3) {
                let msg: String = c.message.chars().take(45).collect();
                commit_items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("    · {}", msg), Style::default().fg(DIM)),
                ])));
            }
        }
        if !self.data.fixes.is_empty() {
            commit_items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("  🔧 Fixes       {}", self.data.fixes.len()),
                    Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            ])));
            for c in self.data.fixes.iter().take(3) {
                let msg: String = c.message.chars().take(45).collect();
                commit_items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("    · {}", msg), Style::default().fg(DIM)),
                ])));
            }
        }
        if !self.data.internal.is_empty() {
            commit_items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("  🔩 Internal    {}", self.data.internal.len()),
                    Style::default().fg(DIM).add_modifier(Modifier::BOLD)),
            ])));
        }

        let commits_list = List::new(commit_items)
            .block(Block::default()
                .title(Span::styled(
                    format!(" 🔀 Commits ({} total) ", self.data.total_commits),
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)))
            .style(Style::default().bg(BG));
        f.render_widget(commits_list, right_chunks[0]);

        // Confirm panel or log
        if self.state == State::Confirm {
            let confirm = Paragraph::new(vec![
                Line::from(vec![Span::styled("  Ready to publish?", Style::default().fg(FG).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("", Style::default())]),
                Line::from(vec![
                    Span::styled("  This will: ", Style::default().fg(DIM)),
                ]),
                Line::from(vec![Span::styled("  · Update VERSION + README + CHANGELOG", Style::default().fg(FG))]),
                Line::from(vec![Span::styled("  · Write release manifest", Style::default().fg(FG))]),
                Line::from(vec![Span::styled("  · Create git tag + push", Style::default().fg(FG))]),
                Line::from(vec![Span::styled("  · Update generation pointer", Style::default().fg(FG))]),
                Line::from(vec![Span::styled("", Style::default())]),
                Line::from(vec![
                    Span::styled("  y ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD | Modifier::REVERSED)),
                    Span::styled(" confirm    ", Style::default().fg(DIM)),
                    Span::styled("n ", Style::default().fg(RED).add_modifier(Modifier::BOLD)),
                    Span::styled(" go back", Style::default().fg(DIM)),
                ]),
            ])
            .block(Block::default()
                .title(Span::styled(" ⚡ Confirm Release ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(YELLOW)))
            .style(Style::default().bg(BG));
            f.render_widget(confirm, right_chunks[1]);
        } else if self.state == State::Publishing || self.state == State::Done {
            let log_items: Vec<ListItem> = self.log.iter().map(|l| {
                let color = if l.starts_with("✅") { ACCENT }
                    else if l.starts_with("❌") { RED }
                    else { DIM };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {}", l), Style::default().fg(color)),
                ]))
            }).collect();
            let log_list = List::new(log_items)
                .block(Block::default()
                    .title(Span::styled(" 🚀 Publishing ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(YELLOW)))
                .style(Style::default().bg(BG));
            f.render_widget(log_list, right_chunks[1]);
        } else {
            let hint = Paragraph::new(vec![
                Line::from(vec![Span::styled("  What gets published:", Style::default().fg(DIM))]),
                Line::from(vec![Span::styled("  · CHANGELOG.md entry (auto-generated)", Style::default().fg(FG))]),
                Line::from(vec![Span::styled("  · README.md dynamic section updated", Style::default().fg(FG))]),
                Line::from(vec![Span::styled("  · 00-meta/VERSION bumped", Style::default().fg(FG))]),
                Line::from(vec![Span::styled("  · release manifest written", Style::default().fg(FG))]),
                Line::from(vec![Span::styled("  · git tag created + pushed", Style::default().fg(FG))]),
            ])
            .block(Block::default()
                .title(Span::styled(" 📋 Release Plan ", Style::default().fg(DIM)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM)))
            .style(Style::default().bg(BG));
            f.render_widget(hint, right_chunks[1]);
        }

        // ── FOOTER ──────────────────────────────────────────────────────
        let footer_text = match self.state {
            State::Preview => vec![
                Span::styled("  Enter", Style::default().fg(ACCENT)),
                Span::styled(" confirm    ", Style::default().fg(DIM)),
                Span::styled("e", Style::default().fg(ACCENT)),
                Span::styled(" edit theme    ", Style::default().fg(DIM)),
                Span::styled("q", Style::default().fg(ACCENT)),
                Span::styled(" abort", Style::default().fg(DIM)),
            ],
            State::EditTheme => vec![
                Span::styled("  Enter", Style::default().fg(YELLOW)),
                Span::styled(" done editing    ", Style::default().fg(DIM)),
                Span::styled("Esc", Style::default().fg(YELLOW)),
                Span::styled(" cancel", Style::default().fg(DIM)),
            ],
            State::Done => vec![
                Span::styled("  ✅ Release published! Press any key to exit.", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            ],
            _ => vec![],
        };
        let footer = Paragraph::new(Line::from(footer_text))
            .style(Style::default().fg(DIM).bg(BG));
        f.render_widget(footer, chunks[3]);
    }

    fn execute_release(&mut self, core_root: &PathBuf) -> Result<()> {
        use std::process::Command;
        use std::fs;

        self.log.push(format!("🚀 Publishing v{}...", self.version));

        // 1. Update VERSION
        let version_path = core_root.join("00-meta/VERSION");
        fs::write(&version_path, &self.version)?;
        self.log.push("✅ VERSION updated".to_string());

        // 2. Write release manifest
        let release_dir = core_root.join(format!("00-meta/releases/{}", self.version));
        fs::create_dir_all(&release_dir)?;
        let manifest = format!(
r#"version = "{}"
date = "{}"
theme = "{}"
health_at_release = {}
commits_since_last = {}

[stats]
tools_total = {}
intents_complete = {}
"#,
            self.version,
            chrono::Local::now().format("%Y-%m-%d"),
            self.theme,
            self.stats.health,
            self.data.total_commits,
            self.stats.tools_deployed,
            self.stats.intents_complete,
        );
        fs::write(release_dir.join("manifest.toml"), &manifest)?;
        self.log.push("✅ Release manifest written".to_string());

        // 3. Update README dynamic section
        let readme_path = core_root.join("README.md");
        let readme_meta_path = core_root.join("00-meta/README.md");
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        if let Err(e) = crate::readme::update_readme(
            &readme_path, &self.version, &self.theme, &date, &self.data, &self.stats
        ) {
            self.log.push(format!("⚠️  README: {}", e));
        } else {
            self.log.push("✅ README.md dynamic section updated".to_string());
        }
        if readme_meta_path.exists() {
            let _ = crate::readme::update_readme(
                &readme_meta_path, &self.version, &self.theme, &date, &self.data, &self.stats
            );
            self.log.push("✅ 00-meta/README.md updated".to_string());
        }

        // 3. Update CHANGELOG
        let changelog_path = core_root.join("00-meta/CHANGELOG.md");
        let existing = fs::read_to_string(&changelog_path).unwrap_or_default();
        let new_entry = self.data.render_markdown(&self.stats);
        let new_content = format!("# Changelog\n\n{}\n{}", new_entry,
            existing.trim_start_matches("# Changelog").trim_start());
        fs::write(&changelog_path, new_content)?;
        self.log.push("✅ CHANGELOG.md updated".to_string());

        // 4. Update generation pointer
        let gen_path = core_root.join("runtime/generation");
        fs::write(&gen_path, &self.version)?;
        self.log.push("✅ Generation pointer updated".to_string());

        // 5. Git commit
        Command::new("git")
            .args(["-C", core_root.to_str().unwrap_or("."),
                   "add", "-A"])
            .output()?;
        Command::new("git")
            .args(["-C", core_root.to_str().unwrap_or("."),
                   "commit", "-m",
                   &format!("feat: Release v{} - 🌲 {}", self.version, self.theme)])
            .output()?;
        self.log.push("✅ Git commit created".to_string());

        // 6. Git tag
        Command::new("git")
            .args(["-C", core_root.to_str().unwrap_or("."),
                   "tag", "-a", &format!("v{}", self.version),
                   "-m", &format!("v{} — {}", self.version, self.theme)])
            .output()?;
        self.log.push(format!("✅ Tag v{} created", self.version));

        self.log.push("🌲 Release complete — push with: fg sync".to_string());
        Ok(())
    }
}
