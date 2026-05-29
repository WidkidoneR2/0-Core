// INT-278: Friday Chat -- The Forest Speaks Back
// ratatui TUI, FridayBackend reads state.db, never invents
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use rusqlite::Connection;
use std::io;

// Forest palette (design-system.md)
const BG: Color = Color::Rgb(10, 15, 10);
const FG: Color = Color::Rgb(168, 197, 176);
const GREEN: Color = Color::Rgb(42, 255, 213);
const DIM: Color = Color::Rgb(74, 107, 82);
const ACCENT: Color = Color::Rgb(0, 191, 255);
const AMBER: Color = Color::Rgb(255, 212, 59);

#[derive(Clone)]
struct Message {
    author: Author,
    text: String,
}

#[derive(Clone, PartialEq)]
enum Author {
    User,
    Friday,
    #[allow(dead_code)]
    System,
}

struct App {
    messages: Vec<Message>,
    input: String,
    scroll: usize,
    db: Connection,
    intent_hint: String,
    health_hint: String,
}

impl App {
    fn new() -> anyhow::Result<Self> {
        let home = dirs::home_dir().unwrap_or_default();
        let db_path = home.join("0-core/runtime/state.db");
        let db = Connection::open(&db_path)?;

        // Get context
        let intent_hint = get_active_intent(&db);
        let health_hint = get_health(&db);

        let welcome = format!(
            "INT-{} | {}% | /help",
            intent_hint, health_hint
        );

        Ok(Self {
            messages: vec![Message { author: Author::Friday, text: welcome }],
            input: String::new(),
            scroll: 0,
            db,
            intent_hint,
            health_hint,
        })
    }

    fn send_message(&mut self) {
        let input = self.input.trim().to_string();
        if input.is_empty() { return; }
        self.input.clear();

        // Log user message
        self.messages.push(Message { author: Author::User, text: input.clone() });

        // Friday responds
        let response = friday_respond(&self.db, &input);
        self.messages.push(Message { author: Author::Friday, text: response });

        // Scroll to bottom
        self.scroll = self.messages.len().saturating_sub(1);

        // Log to state.db
        log_conversation(&self.db, &input);
    }
}

fn get_active_intent(db: &Connection) -> String {
    db.query_row(
        "SELECT value FROM shell_state WHERE key = 'focus_intent'",
        [], |r| r.get::<_, String>(0),
    ).unwrap_or_else(|_| "none".to_string())
}

fn get_health(db: &Connection) -> String {
    db.query_row(
        "SELECT value FROM shell_state WHERE key = 'last_health'",
        [], |r| r.get::<_, String>(0),
    ).unwrap_or_else(|_| "100%".to_string())
}

fn log_conversation(db: &Connection, message: &str) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_secs() as i64;
    let _ = db.execute(
        "INSERT INTO events (timestamp, domain, action, payload, source_tool)
         VALUES (?1, 'friday', 'chat_message', ?2, 'friday-chat')",
        rusqlite::params![now, &message[..message.len().min(200)]],
    );
}

fn friday_respond(db: &Connection, input: &str) -> String {
    let lower = input.to_lowercase();

    // Slash commands
    if lower.starts_with("/help") || lower == "help" {
        return "Commands:\n  /status   -- forest health + active intent\n  /intent   -- current intent details\n  /events   -- recent event bus activity\n  /patterns -- Friday's learned patterns\n  /facts    -- recent friday_knowledge\n  /why [x]  -- explain a recent event\n  /recall [x] -- search past events\n  q / /quit -- exit".to_string();
    }

    if lower.starts_with("/status") || lower == "status" {
        return friday_status(db);
    }

    if lower.starts_with("/intent") || lower == "intent" {
        return friday_intent(db);
    }

    if lower.starts_with("/events") || lower == "events" {
        return friday_events(db);
    }

    if lower.starts_with("/patterns") || lower == "patterns" {
        return friday_patterns(db);
    }

    if lower.starts_with("/facts") || lower == "facts" {
        return friday_facts(db);
    }

    if lower.starts_with("/why") || lower.starts_with("why") {
        let term = input.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        return friday_why(db, &term);
    }

    if lower.starts_with("/recall") || lower.starts_with("recall") {
        let term = input.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        return friday_recall(db, &term);
    }

    if lower.starts_with("/trace") || lower.starts_with("trace") {
        let term = input.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        return friday_trace(db, &term);
    }

    if lower.starts_with("/where") || lower.starts_with("where ") {
        let condition = input.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        return friday_where(db, &condition);
    }
    if lower.starts_with("/show") || lower.starts_with("show ") {
        let subject = input.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        return friday_show(db, &subject);
    }
    if lower.starts_with("/explain") || lower.starts_with("explain ") {
        let subject = input.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        return friday_explain(db, &subject);
    }
    // Natural language -- search knowledge base
    friday_natural(db, &lower)
}

fn friday_status(db: &Connection) -> String {
    let health = get_health(db);
    let intent = get_active_intent(db);
    let facts: i64 = db.query_row("SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0)).unwrap_or(0);
    let patterns: i64 = db.query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0)).unwrap_or(0);
    let decisions: i64 = db.query_row("SELECT COUNT(*) FROM friday_decisions", [], |r| r.get(0)).unwrap_or(0);
    let attention: i64 = db.query_row("SELECT COUNT(*) FROM friday_attention WHERE spoke=1", [], |r| r.get(0)).unwrap_or(0);
    format!(
        "Forest Status:\n  Health: {}\n  Active intent: {}\n  Facts: {}\n  Patterns: {}\n  Decisions: {}\n  Attention events spoken: {}",
        health, intent, facts, patterns, decisions, attention
    )
}

fn friday_intent(db: &Connection) -> String {
    let intent_id = get_active_intent(db);
    if intent_id == "none" || intent_id.is_empty() {
        return "No active intent. Run: cistart <INT-NNN>".to_string();
    }
    // Get recent commits for this intent
    let id: i64 = intent_id.parse().unwrap_or(0);
    let commits: Vec<String> = {
        let mut stmt = db.prepare(
            "SELECT message FROM intent_commits WHERE intent_id = ?1 ORDER BY committed_at DESC LIMIT 3"
        ).unwrap();
        stmt.query_map(rusqlite::params![id], |r| r.get::<_,String>(0))
            .unwrap().flatten()
            .map(|m| format!("  · {}", &m[..m.len().min(60)]))
            .collect()
    };
    format!(
        "Active intent: INT-{}\nRecent commits:\n{}",
        intent_id,
        if commits.is_empty() { "  (none yet)".to_string() } else { commits.join("\n") }
    )
}

fn friday_events(db: &Connection) -> String {
    let mut stmt = db.prepare(
        "SELECT domain, action, payload FROM events ORDER BY timestamp DESC LIMIT 8"
    ).unwrap();
    let rows: Vec<String> = stmt.query_map([], |r| {
        Ok(format!("  [{}:{}] {}", r.get::<_,String>(0)?, r.get::<_,String>(1)?,
            r.get::<_,String>(2).unwrap_or_default()))
    }).unwrap().flatten().collect();
    if rows.is_empty() {
        "No recent events in event bus.".to_string()
    } else {
        format!("Recent events:\n{}", rows.join("\n"))
    }
}

fn friday_patterns(db: &Connection) -> String {
    let mut stmt = db.prepare(
        "SELECT trigger, action, confidence, frequency FROM friday_patterns ORDER BY confidence DESC LIMIT 8"
    ).unwrap();
    let rows: Vec<String> = stmt.query_map([], |r| {
        Ok(format!("  {:.0}% ({}) {} → {}", r.get::<_,f64>(2)?*100.0, r.get::<_,i64>(3)?,
            r.get::<_,String>(0)?, r.get::<_,String>(1)?))
    }).unwrap().flatten().collect();
    if rows.is_empty() { "No patterns learned yet.".to_string() }
    else { format!("Friday patterns (confidence %):\n{}", rows.join("\n")) }
}

fn friday_facts(db: &Connection) -> String {
    let mut stmt = db.prepare(
        "SELECT fact FROM friday_knowledge ORDER BY id DESC LIMIT 6"
    ).unwrap();
    let rows: Vec<String> = stmt.query_map([], |r| {
        r.get::<_,String>(0)
    }).unwrap().flatten()
      .map(|s| format!("  · {}", &s[..s.len().min(80)]))
      .collect();
    if rows.is_empty() { "No facts in friday_knowledge yet.".to_string() }
    else { format!("Recent Friday knowledge:\n{}", rows.join("\n")) }
}

fn friday_why(db: &Connection, term: &str) -> String {
    if term.is_empty() {
        return "Usage: why [event description]\nExample: why did health drop".to_string();
    }
    let pattern = format!("%{}%", term);
    let events: Vec<String> = {
        let mut stmt = db.prepare(
            "SELECT domain, action, payload, timestamp FROM events WHERE payload LIKE ?1 ORDER BY timestamp DESC LIMIT 5"
        ).unwrap();
        stmt.query_map(rusqlite::params![pattern], |r| {
            let ts: i64 = r.get(3)?;
            let dt = chrono::DateTime::from_timestamp(ts, 0)
                .map(|d| d.format("%m-%d %H:%M").to_string())
                .unwrap_or_default();
            Ok(format!("  [{}] {}:{} -- {}", dt, r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?))
        }).unwrap().flatten().collect()
    };
    if events.is_empty() {
        format!("No events found matching: {}\nFriday does not invent. It only reports what it observed.", term)
    } else {
        format!("Events matching '{}':\n{}", term, events.join("\n"))
    }
}

fn friday_recall(db: &Connection, term: &str) -> String {
    if term.is_empty() {
        return "Usage: recall [query]\nExample: recall deploy faelight-shell".to_string();
    }
    let pattern = format!("%{}%", term);
    // Search knowledge + events
    let knowledge: Vec<String> = {
        let mut stmt = db.prepare(
            "SELECT fact FROM friday_knowledge WHERE fact LIKE ?1 OR key LIKE ?1 LIMIT 3"
        ).unwrap();
        stmt.query_map(rusqlite::params![pattern], |r| r.get::<_,String>(0))
            .unwrap().flatten()
            .map(|s| format!("  [knowledge] {}", &s[..s.len().min(80)]))
            .collect()
    };
    let commits: Vec<String> = {
        let mut stmt = db.prepare(
            "SELECT commit_hash, message FROM intent_commits WHERE message LIKE ?1 ORDER BY committed_at DESC LIMIT 3"
        ).unwrap();
        stmt.query_map(rusqlite::params![pattern], |r| {
            Ok(format!("  [commit] {} {}", { let h = r.get::<_,String>(0)?; let end = h.len().min(8); h[..end].to_string() }, r.get::<_,String>(1)?))
        }).unwrap().flatten().collect()
    };
    let mut results = Vec::new();
    results.extend(knowledge);
    results.extend(commits);
    if results.is_empty() {
        format!("Nothing found matching: {}", term)
    } else {
        format!("Recalled for '{}':\n{}", term, results.join("\n"))
    }
}

fn friday_trace(db: &Connection, term: &str) -> String {
    if term.is_empty() {
        return "Usage: trace [signal]\nExample: trace health drop".to_string();
    }
    let pattern = format!("%{}%", term);
    let signals: Vec<String> = {
        let mut stmt = db.prepare(
            "SELECT timestamp, domain, action, payload FROM events WHERE action LIKE ?1 OR payload LIKE ?1 ORDER BY timestamp DESC LIMIT 5"
        ).unwrap();
        stmt.query_map(rusqlite::params![pattern], |r| {
            let ts: i64 = r.get(0)?;
            let dt = chrono::DateTime::from_timestamp(ts, 0)
                .map(|d| d.format("%m-%d %H:%M").to_string())
                .unwrap_or_default();
            Ok(format!("  {} [{}:{}] {}", dt, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,String>(3)?))
        }).unwrap().flatten().collect()
    };
    if signals.is_empty() {
        format!("No signal trace found for: {}", term)
    } else {
        format!("Signal trace for '{}':\n{}", term, signals.join("\n"))
    }
}

fn friday_natural(db: &Connection, input: &str) -> String {
    // Extract keywords and search knowledge
    let keywords: Vec<&str> = input.split_whitespace()
        .filter(|w| w.len() > 3)
        .collect();
    if keywords.is_empty() {
        return "I didn't understand that. Try /help for available commands.".to_string();
    }
    let term = keywords.join(" ");
    let pattern = format!("%{}%", keywords.first().unwrap_or(&""));
    let facts: Vec<String> = {
        let mut stmt = db.prepare(
            "SELECT fact FROM friday_knowledge WHERE fact LIKE ?1 OR key LIKE ?1 LIMIT 3"
        ).unwrap();
        stmt.query_map(rusqlite::params![pattern], |r| r.get::<_,String>(0))
            .unwrap().flatten()
            .map(|s| format!("  · {}", &s[..s.len().min(100)]))
            .collect()
    };
    if facts.is_empty() {
        format!("Friday has no knowledge about '{}'. Try /recall {} or /why {}", term, term, term)
    } else {
        format!("From friday_knowledge about '{}':\n{}", term, facts.join("\n"))
    }
}


/// FQL: friday where [field] [op] [value]
fn friday_where(db: &Connection, condition: &str) -> String {
    if condition.is_empty() {
        return "Usage: where [field] [op] [value]\nFields: risk, domain, confidence, health, source\nOps: > < = !=\nExample: where confidence > 0.8".to_string();
    }
    let parts: Vec<&str> = condition.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return format!("FQL parse error: expected 'field op value', got: {}", condition);
    }
    let field = parts[0].to_lowercase();
    let op = parts[1];
    let value = parts[2];

    match field.as_str() {
        "confidence" => {
            let threshold: f64 = value.parse().unwrap_or(0.7);
            let (q, comp) = match op {
                ">" => ("SELECT trigger, action, confidence FROM friday_patterns WHERE confidence > ?1 ORDER BY confidence DESC LIMIT 10", ">"),
                "<" => ("SELECT trigger, action, confidence FROM friday_patterns WHERE confidence < ?1 ORDER BY confidence DESC LIMIT 10", "<"),
                "=" | "==" => ("SELECT trigger, action, confidence FROM friday_patterns WHERE ABS(confidence - ?1) < 0.05 ORDER BY confidence DESC LIMIT 10", "="),
                _ => return format!("Unknown operator: {}", op),
            };
            let mut stmt = db.prepare(q).unwrap();
            let rows: Vec<String> = stmt.query_map(rusqlite::params![threshold], |r| {
                Ok(format!("  {:.0}% {} → {}", r.get::<_,f64>(2)?*100.0, r.get::<_,String>(0)?, r.get::<_,String>(1)?))
            }).unwrap().flatten().collect();
            if rows.is_empty() { format!("No patterns where confidence {} {}", comp, threshold) }
            else { format!("Patterns where confidence {} {}:\n{}", comp, threshold, rows.join("\n")) }
        }
        "domain" => {
            let mut stmt = db.prepare(
                "SELECT kind, detail FROM events WHERE domain = ?1 ORDER BY timestamp DESC LIMIT 10"
            ).unwrap();
            let rows: Vec<String> = stmt.query_map(rusqlite::params![value], |r| {
                Ok(format!("  [{}] {}", r.get::<_,String>(0)?, r.get::<_,String>(1)?))
            }).unwrap().flatten().collect();
            if rows.is_empty() { format!("No events in domain: {}", value) }
            else { format!("Events in domain '{}':\n{}", value, rows.join("\n")) }
        }
        "health" => {
            let threshold: i64 = value.parse().unwrap_or(95);
            let mut stmt = db.prepare(
                "SELECT timestamp, detail FROM events WHERE domain = 'health' AND CAST(detail AS INTEGER) < ?1 ORDER BY timestamp DESC LIMIT 5"
            ).unwrap();
            let rows: Vec<String> = stmt.query_map(rusqlite::params![threshold], |r| {
                let ts: i64 = r.get(0)?;
                let dt = chrono::DateTime::from_timestamp(ts, 0)
                    .map(|d| d.format("%m-%d %H:%M").to_string()).unwrap_or_default();
                Ok(format!("  [{}] {}", dt, r.get::<_,String>(1)?))
            }).unwrap().flatten().collect();
            if rows.is_empty() { format!("No health events matching where health {} {}", op, threshold) }
            else { format!("Health events where health {} {}:\n{}", op, threshold, rows.join("\n")) }
        }
        "risk" => {
            // Map risk levels: low=0.3, medium=0.5, high=0.7, critical=0.9
            let threshold = match value.to_lowercase().as_str() {
                "low" => 0.3f64,
                "medium" => 0.5,
                "high" => 0.7,
                "critical" => 0.9,
                v => v.parse().unwrap_or(0.5),
            };
            let mut stmt = db.prepare(
                "SELECT event_type, attention_score, event_detail FROM friday_attention WHERE risk > ?1 ORDER BY timestamp DESC LIMIT 8"
            ).unwrap();
            let rows: Vec<String> = stmt.query_map(rusqlite::params![threshold], |r| {
                Ok(format!("  risk={:.2} [{}] {}", r.get::<_,f64>(1)?, r.get::<_,String>(0)?, r.get::<_,String>(2)?))
            }).unwrap().flatten().collect();
            if rows.is_empty() {
                format!("No attention events where risk > {} ({})", value, threshold)
            } else {
                format!("Attention events where risk > {}:\n{}", value, rows.join("\n"))
            }
        }
        _ => format!("Unknown field: {}\nSupported: confidence, domain, health, risk, source", field)
    }
}

/// FQL: friday show [data]
fn friday_show(db: &Connection, subject: &str) -> String {
    let lower = subject.to_lowercase();
    if lower.contains("pattern") || lower.contains("patterns") {
        return friday_patterns(db);
    }
    if lower.contains("decision") || lower.contains("decisions") {
        let mut stmt = db.prepare(
            "SELECT what, why, ties_to FROM friday_decisions ORDER BY id DESC LIMIT 5"
        ).unwrap();
        let rows: Vec<String> = stmt.query_map([], |r| {
            Ok(format!("  · {} (why: {}) ties_to: {}", r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?))
        }).unwrap().flatten().collect();
        if rows.is_empty() { return "No decisions recorded yet.".to_string(); }
        return format!("Recent decisions:\n{}", rows.join("\n"));
    }
    if lower.contains("intent") || lower.contains("intents") {
        return friday_intent(db);
    }
    if lower.contains("attention") {
        let mut stmt = db.prepare(
            "SELECT event_type, attention_score, spoke, event_detail FROM friday_attention ORDER BY timestamp DESC LIMIT 8"
        ).unwrap();
        let rows: Vec<String> = stmt.query_map([], |r| {
            Ok(format!("  {:.3} {} [{}] {}", r.get::<_,f64>(1)?,
                if r.get::<_,i64>(2)? == 1 { "spoke " } else { "silent" },
                r.get::<_,String>(0)?, r.get::<_,String>(3)?))
        }).unwrap().flatten().collect();
        return format!("Attention log:\n{}", rows.join("\n"));
    }
    format!("friday show: unknown subject '{}'\nTry: patterns, decisions, intents, attention", subject)
}

/// FQL: friday explain [subject]
fn friday_explain(db: &Connection, subject: &str) -> String {
    if subject.is_empty() {
        return "Usage: explain [subject]\nExample: explain drift, explain health, explain patterns".to_string();
    }
    let pattern = format!("%{}%", subject);
    // Search knowledge base
    let mut stmt = db.prepare(
        "SELECT domain, key, fact FROM friday_knowledge WHERE fact LIKE ?1 OR key LIKE ?1 ORDER BY id DESC LIMIT 4"
    ).unwrap();
    let rows: Vec<String> = stmt.query_map(rusqlite::params![pattern], |r| {
        Ok(format!("  [{}:{}] {}", r.get::<_,String>(0)?, r.get::<_,String>(1)?,
            r.get::<_,String>(2)?))
    }).unwrap().flatten().collect();
    if rows.is_empty() {
        format!("Friday has no knowledge about '{}'. Try /recall {} or /where domain = {}", subject, subject, subject)
    } else {
        format!("Friday explains '{}':\n{}", subject, rows.join("\n"))
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    // Support: friday chat why [term] -- direct query mode
    if args.len() > 2 && args.get(1).map(|s| s == "chat").unwrap_or(false) {
        let query = args[2..].join(" ");
        let home = dirs::home_dir().unwrap_or_default();
        let db = Connection::open(home.join("0-core/runtime/state.db"))?;
        println!("{}", friday_respond(&db, &query));
        return Ok(());
    }

    let mut app = App::new()?;
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture);
        panic_hook(info);
    }));

    loop {
        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(ratatui::widgets::Block::default().style(Style::default().bg(BG)), area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(area);

            // Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled(" 🌲 Friday Chat ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("│ ", Style::default().fg(DIM)),
                Span::styled(app.intent_hint.as_str(), Style::default().fg(ACCENT)),
                Span::styled(" │ health: ", Style::default().fg(DIM)),
                Span::styled(app.health_hint.as_str(), Style::default().fg(GREEN)),
                Span::styled("  /help for commands  q to quit", Style::default().fg(DIM)),
            ])).style(Style::default().bg(BG));
            f.render_widget(header, chunks[0]);

            // Messages
            let items: Vec<ListItem> = app.messages.iter().map(|m| {
                match m.author {
                    Author::Friday => {
                        let mut lines = vec![Line::from(vec![
                            Span::styled("🌲 Friday  ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
                        ])];
                        for line in m.text.lines() {
                            lines.push(Line::from(vec![
                                Span::styled("          ", Style::default()),
                                Span::styled(line.to_string(), Style::default().fg(FG)),
                            ]));
                        }
                        ListItem::new(lines)
                    }
                    Author::User => {
                        ListItem::new(Line::from(vec![
                            Span::styled("  You  ", Style::default().fg(AMBER).add_modifier(Modifier::BOLD)),
                            Span::styled(m.text.clone(), Style::default().fg(FG)),
                        ]))
                    }
                    Author::System => {
                        ListItem::new(Line::from(Span::styled(m.text.clone(), Style::default().fg(DIM))))
                    }
                }
            }).collect();

            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(ACCENT))
                    .style(Style::default().bg(BG)));
            f.render_widget(list, chunks[1]);

            // Input
            let input = Paragraph::new(format!("> {}", app.input))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(DIM))
                    .title(Span::styled(" Ask Friday ", Style::default().fg(DIM))))
                .style(Style::default().fg(FG).bg(BG))
                .wrap(Wrap { trim: false });
            f.render_widget(input, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => app.send_message(),
                KeyCode::Char('q') if app.input.is_empty() => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Esc => break,
                KeyCode::Backspace => { app.input.pop(); }
                KeyCode::Char(c) => app.input.push(c),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
