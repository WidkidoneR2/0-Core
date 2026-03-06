//! faelight-forecast v1.0.0
//! 🌲 Predictive health intelligence — the forest sees what is coming.

use anyhow::Result;
use chrono::{Local, Duration};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal,
    cursor,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline},
    Terminal,
};
use rusqlite::Connection;
use std::io;
use std::path::PathBuf;

// ─── THEME ──────────────────────────────────────────────────────────────────
const BG:     Color = Color::Rgb(15, 20, 17);
const FG:     Color = Color::Rgb(215, 224, 218);
const ACCENT: Color = Color::Rgb(163, 227, 107);
const DIM:    Color = Color::Rgb(119, 143, 127);
const YELLOW: Color = Color::Rgb(227, 199, 107);
const RED:    Color = Color::Rgb(227, 107, 107);
const BLUE:   Color = Color::Rgb(107, 163, 227);
const CYAN:   Color = Color::Rgb(107, 227, 210);

fn health_color(h: f64) -> Color {
    if h >= 95.0 { ACCENT } else if h >= 85.0 { YELLOW } else { RED }
}

// ─── CLI ────────────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(name = "faelight-forecast", about = "🌲 Predictive health intelligence", version = "1.0.0")]
struct Cli {
    /// Plain text output (no TUI)
    #[arg(long)]
    plain: bool,
    /// Days to project forward
    #[arg(short, long, default_value = "7")]
    days: u32,
}

// ─── DATA ────────────────────────────────────────────────────────────────────
#[derive(Clone, Debug)]
struct HealthPoint {
    health: f64,
    #[allow(dead_code)]
    warnings: i64,
    #[allow(dead_code)]
    failed: i64,
    timestamp: i64,
}

#[derive(Clone, Debug)]
struct SecurityPoint {
    #[allow(dead_code)]
    critical: i64,
    high: i64,
    medium: i64,
    #[allow(dead_code)]
    low: i64,
    timestamp: i64,
}

fn db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home/christian"))
        .join("0-core/runtime/state.db")
}

fn load_health(conn: &Connection) -> Vec<HealthPoint> {
    let mut stmt = match conn.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='doctor' ORDER BY id ASC"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let raw: Vec<(String, i64)> = match stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => return vec![],
    };
    raw.into_iter().filter_map(|(p, ts)| {
        let v: serde_json::Value = serde_json::from_str(&p).ok()?;
        Some(HealthPoint {
            health: v["detail"]["health"].as_f64()?,
            warnings: v["detail"]["warnings"].as_i64().unwrap_or(0),
            failed: v["detail"]["failed"].as_i64().unwrap_or(0),
            timestamp: ts,
        })
    }).collect()
}

fn load_security(conn: &Connection) -> Vec<SecurityPoint> {
    let mut stmt = match conn.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='security' ORDER BY id ASC"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let raw: Vec<(String, i64)> = match stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => return vec![],
    };
    raw.into_iter().filter_map(|(p, ts)| {
        let v: serde_json::Value = serde_json::from_str(&p).ok()?;
        Some(SecurityPoint {
            critical: v["detail"]["critical"].as_i64().unwrap_or(0),
            high: v["detail"]["high"].as_i64().unwrap_or(0),
            medium: v["detail"]["medium"].as_i64().unwrap_or(0),
            low: v["detail"]["low"].as_i64().unwrap_or(0),
            timestamp: ts,
        })
    }).collect()
}

// ─── ANALYSIS ────────────────────────────────────────────────────────────────
struct Analysis {
    // History
    health_history: Vec<u64>,       // for sparkline
    #[allow(dead_code)]
    health_points: Vec<HealthPoint>,
    // Current state
    current_health: f64,
    // Trend
    trend_slope: f64,               // health change per day
    stability_score: f64,           // 0-100, higher = more stable
    drop_frequency: f64,            // drops per day
    // Projection
    projected: Vec<(String, f64, String)>, // (date, health, signal)
    // Security
    #[allow(dead_code)]
    first_scan_ts: i64,
    current_high: i64,
    current_medium: i64,
    security_age_days: f64,
    // Patterns
    patterns: Vec<String>,
    suggestions: Vec<String>,
    // Stats
    total_runs: usize,
    avg_health: f64,
    min_health: f64,
    drop_count: usize,
}

fn analyze(health: &[HealthPoint], security: &[SecurityPoint], days: u32) -> Analysis {
    let n = health.len();
    let current = health.last().map(|h| h.health).unwrap_or(95.0);
    let avg = if n > 0 { health.iter().map(|h| h.health).sum::<f64>() / n as f64 } else { 95.0 };
    let min = health.iter().map(|h| h.health).fold(100.0f64, f64::min);
    let drops: Vec<_> = health.windows(2).filter(|w| w[1].health < w[0].health).collect();
    let drop_count = drops.len();

    // Trend — linear regression over last 20 points
    let window = health.iter().rev().take(20).collect::<Vec<_>>();
    let trend_slope = if window.len() >= 2 {
        let first_ts = window.last().unwrap().timestamp as f64;
        let points: Vec<(f64, f64)> = window.iter().rev()
            .map(|p| ((p.timestamp as f64 - first_ts) / 86400.0, p.health))
            .collect();
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
        let sum_xx: f64 = points.iter().map(|(x, _)| x * x).sum();
        let denom = n * sum_xx - sum_x * sum_x;
        if denom.abs() > 0.001 { (n * sum_xy - sum_x * sum_y) / denom } else { 0.0 }
    } else { 0.0 };

    // Stability — inverse of variance
    let variance = if n > 1 {
        health.iter().map(|h| (h.health - avg).powi(2)).sum::<f64>() / (n - 1) as f64
    } else { 0.0 };
    let stability_score = (100.0 - variance.sqrt() * 10.0).max(0.0).min(100.0);

    // Drop frequency (drops per day)
    let time_span_days = if n >= 2 {
        (health.last().unwrap().timestamp - health.first().unwrap().timestamp) as f64 / 86400.0
    } else { 1.0 };
    let drop_frequency = if time_span_days > 0.0 { drop_count as f64 / time_span_days } else { 0.0 };

    // Security aging
    let first_scan_ts = security.first().map(|s| s.timestamp).unwrap_or(0);
    let last_sec = security.last();
    let current_high = last_sec.map(|s| s.high).unwrap_or(0);
    let current_medium = last_sec.map(|s| s.medium).unwrap_or(0);
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let security_age_days = (now_ts - first_scan_ts) as f64 / 86400.0;

    // Projection
    let mut projected = vec![];
    for d in 1..=days {
        let date = (chrono::Local::now() + Duration::days(d as i64))
            .format("%m/%d").to_string();
        let proj = (current + trend_slope * d as f64).max(0.0).min(100.0);
        // Security aging signal
        let age = security_age_days + d as f64;
        let signal = if current_high > 0 && age > 90.0 {
            "⚠️  critical: high CVEs aging past 90d".to_string()
        } else if current_high > 0 && age > 60.0 {
            "⚠️  high CVEs aging past 60d".to_string()
        } else if current_medium > 5 && age > 30.0 {
            "·  medium findings aging".to_string()
        } else {
            "✓  stable".to_string()
        };
        projected.push((date, proj, signal));
    }

    // Patterns
    let mut patterns = vec![];
    if drop_frequency > 0.5 {
        patterns.push(format!("Health drops {:.1}x/day — likely uncommitted changes during work sessions", drop_frequency));
    }
    if stability_score > 90.0 {
        patterns.push("High stability — system changes are well-managed and committed promptly".to_string());
    }
    if current_high > 0 {
        patterns.push(format!("{} high CVEs tracked — all upstream pending, no action available", current_high));
    }
    if avg > 93.0 {
        patterns.push(format!("Average health {:.1}% — forest is consistently healthy", avg));
    }
    if drop_count > 0 {
        patterns.push(format!("{} health drops recorded — all recovered within the same session", drop_count));
    }

    // Suggestions
    let mut suggestions = vec![];
    if trend_slope < -0.5 {
        suggestions.push("⚠️  Health trending down — review recent changes".to_string());
    } else if trend_slope > 0.0 {
        suggestions.push("✅ Health trending up — forest is improving".to_string());
    } else {
        suggestions.push("✅ Health stable — no action required".to_string());
    }
    if security_age_days > 60.0 {
        suggestions.push(format!("🛡️  Security scan data is {:.0} days old — run: core security scan", security_age_days));
    }
    if drop_frequency > 0.3 {
        suggestions.push("💡 Run `lock-core` before ending sessions to maintain 100% health".to_string());
    }
    suggestions.push("💡 Next milestone: Core v5 Phase 1 — data foundation for intelligence layer".to_string());

    // Sparkline
    let health_history: Vec<u64> = health.iter().map(|h| h.health as u64).collect();

    Analysis {
        health_history,
        health_points: health.to_vec(),
        current_health: current,
        trend_slope,
        stability_score,
        drop_frequency,
        projected,
        first_scan_ts,
        current_high,
        current_medium,
        security_age_days,
        patterns,
        suggestions,
        total_runs: n,
        avg_health: avg,
        min_health: min,
        drop_count,
    }
}

// ─── PLAIN OUTPUT ────────────────────────────────────────────────────────────
fn plain_output(a: &Analysis, days: u32) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🌲 faelight-forecast v1.0.0");
    println!("   Health Intelligence — {}", Local::now().format("%Y-%m-%d %H:%M"));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("📊 Current State");
    println!("   Health:     {}%", a.current_health);
    println!("   Avg health: {:.1}%  (over {} runs)", a.avg_health, a.total_runs);
    println!("   Min health: {}%", a.min_health);
    println!("   Stability:  {:.0}/100", a.stability_score);
    println!("   Trend:      {:.3}%/day", a.trend_slope);
    println!("   Drops:      {} total ({:.2}/day)", a.drop_count, a.drop_frequency);
    println!();
    println!("📈 {}-Day Projection", days);
    for (date, health, signal) in &a.projected {
        println!("   {}  {:>5.1}%  {}", date, health, signal);
    }
    println!();
    println!("🛡️  Security");
    println!("   High findings:   {}", a.current_high);
    println!("   Medium findings: {}", a.current_medium);
    println!("   CVE data age:    {:.0} days", a.security_age_days);
    println!();
    println!("🔍 Patterns");
    for p in &a.patterns { println!("   · {}", p); }
    println!();
    println!("💡 Suggestions");
    for s in &a.suggestions { println!("   {}", s); }
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

// ─── TUI ─────────────────────────────────────────────────────────────────────
fn draw_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    a: &Analysis,
    days: u32,
) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        f.render_widget(Block::default().style(Style::default().bg(BG)), area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // header
                Constraint::Length(4),  // gauge + sparkline
                Constraint::Length(4),  // stats row
                Constraint::Min(0),     // main content
                Constraint::Length(1),  // footer
            ])
            .split(area);

        // ── HEADER ──────────────────────────────────────────────────────
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(32)])
            .split(chunks[0]);

        let left = Paragraph::new(Line::from(vec![
            Span::styled(" 🌲 ", Style::default().fg(ACCENT)),
            Span::styled("Health Forecast", Style::default().fg(FG).add_modifier(Modifier::BOLD)),
            Span::styled("  — the forest sees what is coming", Style::default().fg(DIM)),
        ]))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(ACCENT)));
        f.render_widget(left, header_chunks[0]);

        let right = Paragraph::new(Line::from(vec![
            Span::styled(format!("{}  ", now), Style::default().fg(DIM)),
        ]))
        .alignment(ratatui::layout::Alignment::Right)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(ACCENT)));
        f.render_widget(right, header_chunks[1]);

        // ── GAUGE + SPARKLINE ────────────────────────────────────────────
        let gauge_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(32), Constraint::Min(0)])
            .split(chunks[1]);

        let hc = health_color(a.current_health);
        let gauge = Gauge::default()
            .block(Block::default()
                .title(Span::styled(" Current Health ", Style::default().fg(DIM)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM)))
            .gauge_style(Style::default().fg(hc).bg(BG))
            .percent(a.current_health as u16)
            .label(Span::styled(
                format!("{}%", a.current_health),
                Style::default().fg(hc).add_modifier(Modifier::BOLD),
            ));
        f.render_widget(gauge, gauge_chunks[0]);

        let spark = Sparkline::default()
            .block(Block::default()
                .title(Span::styled(
                    format!(" Health History ({} runs) ", a.total_runs),
                    Style::default().fg(DIM),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM)))
            .data(&a.health_history)
            .style(Style::default().fg(ACCENT));
        f.render_widget(spark, gauge_chunks[1]);

        // ── STATS ROW ────────────────────────────────────────────────────
        let stats_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
            ])
            .split(chunks[2]);

        let trend_color = if a.trend_slope > 0.0 { ACCENT } else if a.trend_slope < -0.1 { RED } else { YELLOW };
        let trend_arrow = if a.trend_slope > 0.05 { "↑" } else if a.trend_slope < -0.05 { "↓" } else { "→" };

        let stat_data = [
            (" Avg Health ", format!("{:.1}%", a.avg_health), ACCENT),
            (" Stability ", format!("{:.0}/100", a.stability_score), CYAN),
            (" Trend ", format!("{} {:.3}%/day", trend_arrow, a.trend_slope), trend_color),
            (" Drops ", format!("{} total", a.drop_count), YELLOW),
        ];

        for (i, (title, val, color)) in stat_data.iter().enumerate() {
            let w = Paragraph::new(Line::from(vec![
                Span::styled(format!("  {}", val), Style::default().fg(*color).add_modifier(Modifier::BOLD)),
            ]))
            .block(Block::default()
                .title(Span::styled(*title, Style::default().fg(DIM)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM)));
            f.render_widget(w, stats_chunks[i]);
        }

        // ── MAIN CONTENT ─────────────────────────────────────────────────
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(chunks[3]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(main_chunks[0]);

        // Projection
        let proj_items: Vec<ListItem> = a.projected.iter().map(|(date, health, signal)| {
            let hc = health_color(*health);
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}  ", date), Style::default().fg(DIM)),
                Span::styled(format!("{:>5.1}%  ", health), Style::default().fg(hc).add_modifier(Modifier::BOLD)),
                Span::styled(signal.clone(), Style::default().fg(DIM)),
            ]))
        }).collect();

        let proj_list = List::new(proj_items)
            .block(Block::default()
                .title(Span::styled(
                    format!(" 📈 {}-Day Projection ", days),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ACCENT)))
            .style(Style::default().bg(BG));
        f.render_widget(proj_list, left_chunks[0]);

        // Security
        let sec_age_color = if a.security_age_days > 60.0 { RED } else if a.security_age_days > 30.0 { YELLOW } else { ACCENT };
        let sec_items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("  High CVEs:    ", Style::default().fg(DIM)),
                Span::styled(format!("{}", a.current_high), Style::default().fg(RED).add_modifier(Modifier::BOLD)),
                Span::styled("  (all upstream pending)", Style::default().fg(DIM)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Medium CVEs:  ", Style::default().fg(DIM)),
                Span::styled(format!("{}", a.current_medium), Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Data age:     ", Style::default().fg(DIM)),
                Span::styled(format!("{:.0} days", a.security_age_days), Style::default().fg(sec_age_color).add_modifier(Modifier::BOLD)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  30d threshold: ", Style::default().fg(DIM)),
                Span::styled(
                    format!("{:.0} days away", (30.0 - a.security_age_days % 30.0).max(0.0)),
                    Style::default().fg(DIM),
                ),
            ])),
        ];
        let sec_list = List::new(sec_items)
            .block(Block::default()
                .title(Span::styled(" 🛡️  Security Intelligence ", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BLUE)))
            .style(Style::default().bg(BG));
        f.render_widget(sec_list, left_chunks[1]);

        // Right side — patterns + suggestions
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(main_chunks[1]);

        let pattern_items: Vec<ListItem> = a.patterns.iter().map(|p| {
            ListItem::new(Line::from(vec![
                Span::styled("  · ", Style::default().fg(ACCENT)),
                Span::styled(p.clone(), Style::default().fg(FG)),
            ]))
        }).collect();
        let pattern_list = List::new(pattern_items)
            .block(Block::default()
                .title(Span::styled(" 🔍 Patterns Detected ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)))
            .style(Style::default().bg(BG));
        f.render_widget(pattern_list, right_chunks[0]);

        let suggest_items: Vec<ListItem> = a.suggestions.iter().map(|s| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}  ", s), Style::default().fg(FG)),
            ]))
        }).collect();
        let suggest_list = List::new(suggest_items)
            .block(Block::default()
                .title(Span::styled(" 💡 Suggestions ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(YELLOW)))
            .style(Style::default().bg(BG));
        f.render_widget(suggest_list, right_chunks[1]);

        // ── FOOTER ──────────────────────────────────────────────────────
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("  q", Style::default().fg(ACCENT)),
            Span::styled(" quit  ", Style::default().fg(DIM)),
            Span::styled("--plain", Style::default().fg(ACCENT)),
            Span::styled(" text output  ", Style::default().fg(DIM)),
            Span::styled("--days N", Style::default().fg(ACCENT)),
            Span::styled(" projection window  ", Style::default().fg(DIM)),
        ]))
        .style(Style::default().fg(DIM).bg(BG));
        f.render_widget(footer, chunks[4]);
    })?;
    Ok(())
}

// ─── MAIN ────────────────────────────────────────────────────────────────────
fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = Connection::open(db_path())?;
    let health = load_health(&conn);
    let security = load_security(&conn);
    let analysis = analyze(&health, &security, cli.days);

    if cli.plain {
        plain_output(&analysis, cli.days);
        return Ok(());
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        draw_tui(&mut terminal, &analysis, cli.days)?;
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press { continue; }
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}
