//! faelight-dashboard v0.1 - TUI System Overview
//! 🌲 Faelight Forest

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io::{stdout, Result};
use std::process::Command;

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    
    let result = run(&mut terminal);
    
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    
    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    loop {
        let data = gather_data();
        
        terminal.draw(|frame| {
            render_dashboard(frame, &data);
        })?;
        
        if event::poll(std::time::Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('h') => run_command("dot-doctor"),
                        KeyCode::Char('g') => run_command("lazygit"),
                        KeyCode::Char('i') => run_command("intent list"),
                        KeyCode::Char('r') => continue, // refresh
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

struct DashboardData {
    version: String,
    health: String,
    git_status: String,
    profile: String,
    intents: String,
    stats: String,
    security: String,
    snapshots: String,
}

fn gather_data() -> DashboardData {
    // Version
    let version = std::fs::read_to_string(format!("{}/0-core/VERSION", std::env::var("HOME").unwrap_or_default()))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    // Health (quick check)
    let health_output = Command::new("sh")
        .args(["-c", "dot-doctor 2>/dev/null | grep -E '✅|⚠️|❌' | head -6"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|_| "Unable to check".to_string());
    
    // Git status
    let git_output = Command::new("sh")
        .args(["-c", "cd ~/0-core && git status -s 2>/dev/null | head -5"])
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            if out.trim().is_empty() {
                "✅ Clean".to_string()
            } else {
                format!("⚠️ {} changes", out.lines().count())
            }
        })
        .unwrap_or_else(|_| "Unknown".to_string());
    
    // Profile
    let profile = Command::new("sh")
        .args(["-c", "faelight status 2>/dev/null | grep Profile | awk '{print $2}'"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "default".to_string());
    
    // Intents
    let intents = Command::new("sh")
        .args(["-c", "ls ~/0-core/INTENT/future/*.md 2>/dev/null | wc -l"])
        .output()
        .map(|o| {
            let count = String::from_utf8_lossy(&o.stdout).trim().to_string();
            format!("{} planned", count)
        })
        .unwrap_or_else(|_| "Unknown".to_string());
    
    // Stats
    let rust_tools = Command::new("sh")
        .args(["-c", "ls ~/0-core/rust-tools/ 2>/dev/null | wc -l"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "0".to_string());
    
    let packages = Command::new("sh")
        .args(["-c", "cat ~/0-core/pkglist.txt 2>/dev/null | wc -l"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "0".to_string());
    
    let stats = format!("🦀 {} Rust tools | 📦 {} packages", rust_tools, packages);
    
    // Security
    let vpn = Command::new("sh")
        .args(["-c", "mullvad status 2>/dev/null | head -1"])
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            if out.contains("Connected") { "🟢 VPN" } else { "🔴 VPN" }
        })
        .unwrap_or("❓ VPN");
    
    let ufw = Command::new("sh")
        .args(["-c", "sudo ufw status 2>/dev/null | head -1"])
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            if out.contains("active") { "🟢 UFW" } else { "🔴 UFW" }
        })
        .unwrap_or("❓ UFW");
    
    let security = format!("{} | {}", vpn, ufw);
    
    // Snapshots
    let snapshots = Command::new("sh")
        .args(["-c", "sudo snapper -c root list 2>/dev/null | tail -1 | awk '{print $7, $8}'"])
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if out.is_empty() { "No snapshots".to_string() } else { format!("Latest: {}", out) }
        })
        .unwrap_or_else(|_| "Unknown".to_string());
    
    DashboardData {
        version,
        health: health_output,
        git_status: git_output,
        profile,
        intents,
        stats,
        security,
        snapshots,
    }
}

fn render_dashboard(frame: &mut Frame, data: &DashboardData) {
    let area = frame.area();
    
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(area);
    
    // Header
    let header = Paragraph::new(format!("🌲 Faelight Dashboard v{}", data.version))
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
    frame.render_widget(header, chunks[0]);
    
    // Main content - split into columns
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    // Left column
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[0]);
    
    // Health panel
    let health = Paragraph::new(data.health.clone())
        .block(Block::default().title(" 🏥 Health ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
        .wrap(Wrap { trim: true });
    frame.render_widget(health, left_chunks[0]);
    
    // Git panel
    let git_content = format!(
        "Status: {}\nProfile: {}\nIntents: {}",
        data.git_status, data.profile, data.intents
    );
    let git = Paragraph::new(git_content)
        .block(Block::default().title(" 🔧 System ").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)))
        .wrap(Wrap { trim: true });
    frame.render_widget(git, left_chunks[1]);
    
    // Right column
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);
    
    // Security panel
    let security_content = format!(
        "{}\n\n📸 {}",
        data.security, data.snapshots
    );
    let security = Paragraph::new(security_content)
        .block(Block::default().title(" 🔒 Security ").borders(Borders::ALL).border_style(Style::default().fg(Color::Red)))
        .wrap(Wrap { trim: true });
    frame.render_widget(security, right_chunks[0]);
    
    // Stats panel
    let stats = Paragraph::new(data.stats.clone())
        .block(Block::default().title(" 📊 Stats ").borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)))
        .wrap(Wrap { trim: true });
    frame.render_widget(stats, right_chunks[1]);
    
    // Footer
    let footer = Paragraph::new(" [h]ealth  [g]it  [i]ntents  [r]efresh  [q]uit ")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(footer, chunks[2]);
}

fn run_command(cmd: &str) {
    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);
    
    let _ = Command::new("sh")
        .args(["-c", cmd])
        .status();
    
    // Wait for keypress
    println!("\nPress Enter to return...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    
    let _ = stdout().execute(EnterAlternateScreen);
    let _ = enable_raw_mode();
}
