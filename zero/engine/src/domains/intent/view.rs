//! INT-307: the findings screen's rows. Pure values. The terminal draws them.
//!
//! A row is built from the one finding reader and from `state_of`. Nothing here
//! parses a finding file or a Fixes trailer a second time.

use super::bug::{day_number, story, FixFacts};
use super::findings::{parse_record, Record};
use super::trace::{state_of, State};
use std::collections::HashMap;

/// The glyph a missing or not-applicable date shows. Never a guessed day.
pub const EM_DASH: &str = "\u{2014}";

/// How a row is colored when color is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Fixed. Green.
    Fixed,
    /// Open, and a severity is set. Red.
    Open,
    /// No severity. Yellow. Needs triage.
    Unrated,
}

/// One finding as the screen lists it. Columns, in order: id, description,
/// date found, date fixed, severity, state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindingRow {
    pub id: String,
    pub description: String,
    pub found: String,
    pub fixed: String,
    pub severity: String,
    pub state: String,
    pub tone: Tone,
    pub high_marker: bool,
}

impl FindingRow {
    /// The six columns, in the ruled order.
    pub fn columns(&self) -> [&str; 6] {
        [
            &self.id,
            &self.description,
            &self.found,
            &self.fixed,
            &self.severity,
            &self.state,
        ]
    }
}

/// A key beside the typed substring. `All` is the `a` key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFilter {
    All,
    Open,
    High,
}

/// What `paint` decided. `ansi` is false when color is off, so a pipe stays plain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paint {
    pub text: String,
    pub ansi: bool,
    pub tone: Tone,
    pub bold: bool,
}

/// The terminal operations a screen can undo. The real terminal and the test
/// double both answer this. Drop always leaves.
pub trait Screen {
    fn enter_raw(&mut self);
    fn leave_raw(&mut self);
    fn enter_alt(&mut self);
    fn leave_alt(&mut self);
}

/// Holds the terminal until it is restored. Drop restores, including when the
/// caller panics.
pub struct Guard<S: Screen> {
    screen: S,
    active: bool,
}

impl<S: Screen> Guard<S> {
    /// Take the terminal. Raw mode, then the alternate screen.
    pub fn enter(mut screen: S) -> Self {
        screen.enter_raw();
        screen.enter_alt();
        Self {
            screen,
            active: true,
        }
    }

    /// Give the terminal back. A second call does nothing.
    pub fn restore(&mut self) {
        if self.active {
            self.screen.leave_alt();
            self.screen.leave_raw();
            self.active = false;
        }
    }
}

impl<S: Screen> Drop for Guard<S> {
    fn drop(&mut self) {
        self.restore();
    }
}

/// A stored day, or the em dash. `unknown` and any other non-day are missing.
fn date_or_dash(value: &str) -> String {
    if day_number(value).is_some() {
        value.to_string()
    } else {
        EM_DASH.to_string()
    }
}

/// The earliest day named for these commits. A missing day is the em dash.
fn fixed_date(hashes: &[String], days: &HashMap<String, String>) -> String {
    let mut best: Option<&str> = None;
    for hash in hashes {
        let Some(day) = days.get(hash) else {
            return EM_DASH.to_string();
        };
        if day_number(day).is_none() {
            return EM_DASH.to_string();
        }
        best = Some(match best {
            Some(have) if have <= day.as_str() => have,
            _ => day.as_str(),
        });
    }
    best.unwrap_or(EM_DASH).to_string()
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 3,
    }
}

/// Rows from the records, the Fixes trailers, and the day of each fixing commit.
/// `days` maps a commit hash to YYYY-MM-DD. A hash with no day, or a finding
/// nothing fixes, shows the em dash.
pub fn assemble(
    records: &[Record],
    fixes: &Result<Vec<(String, String)>, String>,
    days: &HashMap<String, String>,
) -> Vec<FindingRow> {
    let mut rows = Vec::with_capacity(records.len());
    for record in records {
        let state = state_of(record, fixes);
        let (word, mut tone, fixed) = match &state {
            State::Fixed(hashes) => ("fixed", Tone::Fixed, fixed_date(hashes, days)),
            State::Open => ("open", Tone::Open, EM_DASH.to_string()),
            State::Closed(_) => ("closed", Tone::Open, EM_DASH.to_string()),
            State::Unread(_) => ("unknown", Tone::Open, EM_DASH.to_string()),
        };
        if !matches!(state, State::Fixed(_)) && record.severity == "unrated" {
            tone = Tone::Unrated;
        }
        let high_marker = record.severity == "high" && !matches!(state, State::Fixed(_));
        rows.push(FindingRow {
            id: record.id.clone(),
            description: record.what.clone(),
            found: date_or_dash(&record.found_on),
            fixed,
            severity: record.severity.clone(),
            state: word.to_string(),
            tone,
            high_marker,
        });
    }
    order(rows)
}

/// Open findings first: high, then medium, then low, then no severity.
/// Fixed findings sink to the bottom.
pub fn order(mut rows: Vec<FindingRow>) -> Vec<FindingRow> {
    rows.sort_by(|a, b| {
        let group = |row: &FindingRow| if row.state == "fixed" { 1 } else { 0 };
        let sev = |row: &FindingRow| {
            if row.state == "fixed" {
                0
            } else {
                severity_rank(&row.severity)
            }
        };
        group(a)
            .cmp(&group(b))
            .then_with(|| sev(a).cmp(&sev(b)))
            .then_with(|| a.id.cmp(&b.id))
    });
    rows
}

/// Substring on every column, then the key. `Open` keeps state open. `High`
/// keeps severity high. `All` keeps every row the substring left.
pub fn apply_filter(rows: &[FindingRow], query: &str, key: KeyFilter) -> Vec<FindingRow> {
    let q = query.to_lowercase();
    rows.iter()
        .filter(|row| {
            let typed = q.is_empty()
                || row
                    .columns()
                    .iter()
                    .any(|col| col.to_lowercase().contains(&q));
            let keyed = match key {
                KeyFilter::All => true,
                KeyFilter::Open => row.state == "open",
                KeyFilter::High => row.severity == "high",
            };
            typed && keyed
        })
        .cloned()
        .collect()
}

/// Color is on only for a terminal, and only when NO_COLOR is unset.
pub fn colors_enabled(no_color_set: bool, is_terminal: bool) -> bool {
    !no_color_set && is_terminal
}

/// The line the screen prints. Color off means the text and no escapes.
pub fn paint(row: &FindingRow, color: bool) -> Paint {
    let plain = row.columns().join("  ");
    if !color {
        return Paint {
            text: plain,
            ansi: false,
            tone: row.tone,
            bold: false,
        };
    }
    let (r, g, b) = match row.tone {
        Tone::Fixed => (57, 255, 20),
        Tone::Open => (255, 80, 80),
        Tone::Unrated => (255, 200, 50),
    };
    let body = format!("\x1b[38;2;{r};{g};{b}m{plain}\x1b[0m");
    let text = if row.high_marker {
        format!("\x1b[1m{body}")
    } else {
        body
    };
    Paint {
        text,
        ansi: true,
        tone: row.tone,
        bold: row.high_marker,
    }
}

/// The full finding: the record, the fixing commits, and each seal, from the
/// one story the bug report already prints.
pub fn detail_text(record: &Record, state: &State, facts: &[FixFacts]) -> String {
    story(record, state, facts).text
}

/// Read one finding record the way the ledger writes it.
pub fn record_from(body: &str) -> Record {
    parse_record(body)
}

/// A row plus the text Enter shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub row: FindingRow,
    pub detail: String,
}

/// The screen's list. Records come from the findings reader, state from
/// `state_of`, days and seals from git. A date git cannot give is the em dash.
pub fn load_cards(ctx: &crate::app::context::AppContext) -> Result<Vec<Card>, String> {
    use super::bug::fix_day;
    use super::trace::{check_commit_seal, git_read, seal_notes};

    let dir = super::intents_dir(ctx).join("findings");
    let records = super::findings::read_records(&dir)?;
    let fixes = super::trace::fixes(&ctx.core_root);
    let mut days: HashMap<String, String> = HashMap::new();
    if let Ok(named) = &fixes {
        for (_, hash) in named {
            if days.contains_key(hash) {
                continue;
            }
            if let Ok(day) = fix_day(&ctx.core_root, std::slice::from_ref(hash)) {
                days.insert(hash.clone(), day);
            }
        }
    }
    let rows = assemble(&records, &fixes, &days);
    let notes = seal_notes(&ctx.core_root).unwrap_or_default();
    let mut cards = Vec::with_capacity(records.len());
    for record in &records {
        let state = state_of(record, &fixes);
        let facts = match &state {
            State::Fixed(hashes) => hashes
                .iter()
                .filter_map(|hash| {
                    let day = days
                        .get(hash)
                        .cloned()
                        .unwrap_or_else(|| EM_DASH.to_string());
                    let subject = git_read(&ctx.core_root, &["log", "-1", "--format=%s", hash])
                        .unwrap_or_default();
                    let seal = check_commit_seal(&ctx.core_root, &notes, hash).ok()?;
                    Some(FixFacts {
                        hash: hash.clone(),
                        day,
                        subject: subject.trim().to_string(),
                        seal,
                    })
                })
                .collect(),
            _ => Vec::new(),
        };
        let detail = detail_text(record, &state, &facts);
        if let Some(row) = rows.iter().find(|row| row.id == record.id) {
            cards.push(Card {
                row: row.clone(),
                detail,
            });
        }
    }
    cards.sort_by_key(|card| {
        rows.iter()
            .position(|row| row.id == card.row.id)
            .unwrap_or(usize::MAX)
    });
    Ok(cards)
}

#[cfg(test)]
mod g7_findings_view_tests {
    use super::super::trace::SealCheck;
    use super::*;
    use std::sync::{Arc, Mutex};

    fn rec(id: &str, what: &str, found: &str, severity: Option<&str>, closed: &str) -> Record {
        let sev = match severity {
            Some(s) => format!("severity: {s}\n"),
            None => String::new(),
        };
        parse_record(&format!(
            "---\nid: {id}\nwhat: {what}\nat: unknown\nfound_by: INT-1\nfound_on: {found}\nclosed_without_fix: {closed}\n{sev}---\n"
        ))
    }

    fn fixes(pairs: &[(&str, &str)]) -> Result<Vec<(String, String)>, String> {
        Ok(pairs
            .iter()
            .map(|(id, hash)| (id.to_string(), hash.to_string()))
            .collect())
    }

    fn sample() -> (
        Vec<Record>,
        Result<Vec<(String, String)>, String>,
        HashMap<String, String>,
    ) {
        let records = vec![
            rec("F-0003", "low open", "2026-01-03", Some("low"), "no"),
            rec("F-0001", "high open", "2026-01-01", Some("high"), "no"),
            rec("F-0005", "low fixed", "2026-01-05", Some("low"), "no"),
            rec("F-0004", "no severity", "2026-01-04", None, "no"),
            rec("F-0002", "medium open", "2026-01-02", Some("medium"), "no"),
            rec("F-0006", "high undated", "unknown", Some("high"), "no"),
            rec(
                "F-0007",
                "fixed undated",
                "2026-01-07",
                Some("medium"),
                "no",
            ),
        ];
        let named = fixes(&[("F-0005", "aaa1111"), ("F-0007", "bbb2222")]);
        let mut days = HashMap::new();
        days.insert("aaa1111".to_string(), "2026-02-02".to_string());
        (records, named, days)
    }

    fn rows() -> Vec<FindingRow> {
        let (records, named, days) = sample();
        assemble(&records, &named, &days)
    }

    fn ids(rows: &[FindingRow]) -> Vec<&str> {
        rows.iter().map(|r| r.id.as_str()).collect()
    }

    #[test]
    fn g7_rows_sort_open_high_medium_low_unrated_then_fixed() {
        assert_eq!(
            ids(&rows()),
            vec!["F-0001", "F-0006", "F-0002", "F-0003", "F-0004", "F-0005", "F-0007",]
        );
    }

    #[test]
    fn g7_columns_are_id_description_found_fixed_severity_state() {
        let row = rows().into_iter().find(|r| r.id == "F-0001").unwrap();
        assert_eq!(
            row.columns(),
            ["F-0001", "high open", "2026-01-01", EM_DASH, "high", "open",]
        );
        let fixed = rows().into_iter().find(|r| r.id == "F-0005").unwrap();
        assert_eq!(fixed.columns()[3], "2026-02-02");
        assert_eq!(fixed.columns()[5], "fixed");
    }

    #[test]
    fn g7_missing_dates_are_an_em_dash_and_never_guessed() {
        let rows = rows();
        let undated = rows.iter().find(|r| r.id == "F-0006").unwrap();
        assert_eq!(undated.found, EM_DASH);
        assert_eq!(undated.fixed, EM_DASH);
        let open = rows.iter().find(|r| r.id == "F-0001").unwrap();
        assert_eq!(open.fixed, EM_DASH);
        let no_day = rows.iter().find(|r| r.id == "F-0007").unwrap();
        assert_eq!(no_day.state, "fixed");
        assert_eq!(no_day.fixed, EM_DASH);
        assert_ne!(no_day.fixed, "2026-11-10");
    }

    #[test]
    fn g7_state_color_green_fixed_red_open_yellow_unrated_bold_high() {
        let rows = rows();
        let high = rows.iter().find(|r| r.id == "F-0001").unwrap();
        assert_eq!(high.tone, Tone::Open);
        assert!(high.high_marker);
        let plain = paint(high, false);
        assert!(!plain.ansi);
        assert!(!plain.bold);
        assert!(!plain.text.contains('\u{1b}'));
        let marked = paint(high, true);
        assert!(marked.ansi);
        assert!(marked.bold);
        assert_eq!(marked.tone, Tone::Open);
        let unrated = rows.iter().find(|r| r.id == "F-0004").unwrap();
        assert_eq!(unrated.state, "open");
        assert_eq!(unrated.tone, Tone::Unrated);
        assert!(!unrated.high_marker);
        let fixed = rows.iter().find(|r| r.id == "F-0005").unwrap();
        assert_eq!(fixed.tone, Tone::Fixed);
        assert!(!fixed.high_marker);
        assert_eq!(paint(fixed, true).tone, Tone::Fixed);
    }

    #[test]
    fn g7_substring_filter() {
        let rows = rows();
        let hit = apply_filter(&rows, "0003", KeyFilter::All);
        assert_eq!(ids(&hit), vec!["F-0003"]);
        let lows = apply_filter(&rows, "low", KeyFilter::All);
        assert_eq!(ids(&lows), vec!["F-0003", "F-0005"]);
    }

    #[test]
    fn g7_key_o_is_open_only() {
        let rows = rows();
        let open = apply_filter(&rows, "", KeyFilter::Open);
        assert!(open.iter().all(|r| r.state == "open"));
        assert_eq!(
            ids(&open),
            vec!["F-0001", "F-0006", "F-0002", "F-0003", "F-0004"]
        );
    }

    #[test]
    fn g7_key_h_is_high_only() {
        let rows = rows();
        let high = apply_filter(&rows, "", KeyFilter::High);
        assert!(high.iter().all(|r| r.severity == "high"));
        assert_eq!(ids(&high), vec!["F-0001", "F-0006"]);
    }

    #[test]
    fn g7_key_a_shows_all() {
        let rows = rows();
        let all = apply_filter(&rows, "", KeyFilter::All);
        assert_eq!(all.len(), rows.len());
        assert_eq!(ids(&all), ids(&rows));
    }

    #[test]
    fn g7_color_off_when_no_color_or_not_a_terminal() {
        assert!(!colors_enabled(true, true));
        assert!(!colors_enabled(true, false));
        assert!(!colors_enabled(false, false));
        assert!(colors_enabled(false, true));
        let row = rows().into_iter().find(|r| r.id == "F-0001").unwrap();
        assert!(!paint(&row, colors_enabled(true, true)).ansi);
        assert!(!paint(&row, colors_enabled(false, false)).ansi);
        assert!(paint(&row, colors_enabled(false, true)).ansi);
    }

    struct Fake {
        log: Arc<Mutex<Vec<&'static str>>>,
    }

    impl Screen for Fake {
        fn enter_raw(&mut self) {
            self.log.lock().unwrap().push("enter_raw");
        }
        fn leave_raw(&mut self) {
            self.log.lock().unwrap().push("leave_raw");
        }
        fn enter_alt(&mut self) {
            self.log.lock().unwrap().push("enter_alt");
        }
        fn leave_alt(&mut self) {
            self.log.lock().unwrap().push("leave_alt");
        }
    }

    #[test]
    fn g7_terminal_guard_restores_on_drop_after_panic() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let seen = log.clone();
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let guard = Guard::enter(Fake { log: seen });
            let _keep = guard;
            panic!("boom");
        }));
        assert!(caught.is_err());
        assert_eq!(
            log.lock().unwrap().as_slice(),
            ["enter_raw", "enter_alt", "leave_alt", "leave_raw"]
        );
    }

    #[test]
    fn g7_detail_names_the_fixing_commit_and_the_seal() {
        let record = rec("F-0005", "low fixed", "2026-01-05", Some("low"), "no");
        let state = State::Fixed(vec!["aaa1111".to_string()]);
        let facts = vec![FixFacts {
            hash: "aaa1111".to_string(),
            day: "2026-02-02".to_string(),
            subject: "fix the low one".to_string(),
            seal: SealCheck::Intact("0123456789abcdef".to_string()),
        }];
        let text = detail_text(&record, &state, &facts);
        assert!(text.contains("F-0005"), "{text}");
        assert!(text.contains("aaa1111"), "{text}");
        assert!(text.contains("0123456789abcdef"), "{text}");
        assert!(text.contains("intact"), "{text}");
    }
}
