//! INT-212: the gate rule -- ONE pure function over file content.
//!
//! "Complete" means the gates were demonstrated. This module reads the gate list out of an
//! intent file and decides whether completion may proceed. It touches no filesystem, so every
//! rule here is tested against plain strings. Gates live in the `## Success Criteria` section.
//! An open checkbox anywhere else is not a gate, but it is never silently ignored: it is named
//! apart from the open gates, so the author can tell which of the two they are looking at.

/// The pause mark that introduces a signed deferral (U+23F8).
const PAUSE: char = '\u{23F8}';
/// The legacy emoji gate marks (INT-130 superseded them with markdown boxes).
const LEGACY_OPEN: char = '\u{2B1C}';
const LEGACY_DONE: char = '\u{2705}';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateState {
    Open,
    Done,
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    /// `G<n>` when the text starts with one.
    #[allow(dead_code)] // read by the G6 and G7 consumers, which migrate next
    pub id: Option<String>,
    pub state: GateState,
    /// An INT-158 evidence comment follows the gate, or a format 2 `evidence:` field is set.
    #[allow(dead_code)] // read by the G6 and G7 consumers, which migrate next
    pub evidence: bool,
    pub text: String,
    /// Indented `ask:` field, including wrap lines.
    pub ask: String,
    /// Indented `ruling:` field, including wrap lines.
    pub ruling: String,
    /// Indented `evidence:` field, including wrap lines. Empty is not a receipt.
    pub evidence_text: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct GateReport {
    pub section_found: bool,
    pub gates: Vec<Gate>,
    /// Open checkboxes outside the section, as written.
    pub stray_open: Vec<String>,
    /// Pause-mark lines inside the section that carry no approval signature.
    pub bad_deferrals: Vec<String>,
}

/// Why completion was refused. Each variant is a distinct reason with its own message.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    BadDeferral(Vec<String>),
    NoGates {
        section_found: bool,
    },
    OpenGates(Vec<String>),
    /// A done format 2 (or later) gate whose `evidence:` field is empty.
    MissingEvidence(Vec<String>),
    StrayOpen(Vec<String>),
}

// -- the rule --
fn is_any_heading(line: &str) -> bool {
    line.starts_with("## ")
}

/// `## Success Criteria`, in any case, with or without trailing words.
fn is_section_heading(line: &str) -> bool {
    is_any_heading(line)
        && line[3..]
            .trim()
            .to_ascii_lowercase()
            .starts_with("success criteria")
}

/// The state a checkbox-style line declares, and the text after the marker.
fn box_state(trimmed: &str) -> Option<(GateState, &str)> {
    let markers = [
        ("- [ ]", GateState::Open),
        ("- [~]", GateState::Open),
        ("- [x]", GateState::Done),
        ("- [X]", GateState::Done),
    ];
    for (marker, state) in markers {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return Some((state, rest.trim()));
        }
    }
    if let Some(rest) = trimmed.strip_prefix(LEGACY_OPEN) {
        return Some((GateState::Open, rest.trim()));
    }
    if let Some(rest) = trimmed.strip_prefix(LEGACY_DONE) {
        return Some((GateState::Done, rest.trim()));
    }
    None
}

/// The full signed form: a pause mark, the approval signature and a date, on one line.
fn is_signed_deferral(line: &str) -> bool {
    line.contains(PAUSE) && line.contains("approved by: christian") && line.contains("20")
}

fn gate_id(text: &str) -> Option<String> {
    let token = text.split_whitespace().next()?;
    let token = token.trim_end_matches(|c: char| !c.is_ascii_alphanumeric());
    let digits = token.strip_prefix('G')?;
    if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
        Some(token.to_string())
    } else {
        None
    }
}

/// The lines that count: not inside a fenced code block, not inside a multi-line HTML comment.
fn live_lines(content: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut fenced, mut commented) = (false, false);
    for line in content.lines() {
        let t = line.trim_start();
        if commented {
            commented = !t.contains("-->");
            continue;
        }
        if t.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if t.starts_with("<!--") && !t.contains("-->") {
            commented = true;
        }
        out.push(line);
    }
    out
}

struct GateTail {
    ask: String,
    ruling: String,
    evidence_text: String,
    evidence: bool,
}

fn append_field(buf: &mut String, more: &str) {
    if more.is_empty() {
        return;
    }
    if !buf.is_empty() {
        buf.push(' ');
    }
    buf.push_str(more);
}

/// Indented ask/ruling/evidence fields, their wrap lines, and a receipt that
/// follows an unindented RULED line. A RULED line does not end the scan.
fn gate_tail(lines: &[&str], at: usize) -> GateTail {
    let mut ask = String::new();
    let mut ruling = String::new();
    let mut evidence_text = String::new();
    let mut html = false;
    let mut which = "";
    let mut j = at + 1;
    while j < lines.len() {
        let line = lines[j];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            j += 1;
            continue;
        }
        if line.starts_with([' ', '\t']) {
            if let Some(rest) = trimmed.strip_prefix("ask:") {
                which = "ask";
                append_field(&mut ask, rest.trim());
            } else if let Some(rest) = trimmed.strip_prefix("ruling:") {
                which = "ruling";
                append_field(&mut ruling, rest.trim());
            } else if let Some(rest) = trimmed.strip_prefix("evidence:") {
                which = "evidence";
                append_field(&mut evidence_text, rest.trim());
            } else if which == "ask" {
                append_field(&mut ask, trimmed);
            } else if which == "ruling" {
                append_field(&mut ruling, trimmed);
            } else if which == "evidence" {
                append_field(&mut evidence_text, trimmed);
            }
            j += 1;
            continue;
        }
        if trimmed.starts_with("RULED") {
            j += 1;
            continue;
        }
        if trimmed.starts_with("<!--") && trimmed.to_ascii_lowercase().contains("evidence") {
            html = true;
            j += 1;
            continue;
        }
        break;
    }
    GateTail {
        evidence: html || !evidence_text.trim().is_empty(),
        ask,
        ruling,
        evidence_text,
    }
}

/// Format 2 or higher: a done gate needs a non-empty `evidence:` field.
/// A missing number, or a number below 2, is format 1 and does not.
fn evidence_is_required(content: &str) -> bool {
    content.split("---").nth(1).is_some_and(|fm| {
        fm.lines().any(|line| {
            let Some(value) = line.trim().strip_prefix("format:") else {
                return false;
            };
            canonical_format(value.trim()).is_some_and(|n| n >= 2)
        })
    })
}

/// Digits only, and not `02`. `format: 2` stays 2. A leading zero is not 2.
fn canonical_format(value: &str) -> Option<u32> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }
    value.parse().ok()
}

/// Read the gates out of an intent file. Only the Success Criteria section holds gates.
pub fn read_gates(content: &str) -> GateReport {
    let lines = live_lines(content);
    let mut report = GateReport::default();
    let mut in_section = false;
    for (at, line) in lines.iter().enumerate() {
        if is_any_heading(line) {
            in_section = is_section_heading(line);
            report.section_found |= in_section;
            continue;
        }
        let t = line.trim();
        if !in_section {
            if let Some((GateState::Open, _)) = box_state(t) {
                if !is_signed_deferral(line) {
                    report.stray_open.push(t.to_string());
                }
            }
            continue;
        }
        let tail = gate_tail(&lines, at);
        if let Some((state, text)) = box_state(t) {
            if text.is_empty() || text == "..." {
                continue; // the scaffold's placeholder is not a gate
            }
            let state = if is_signed_deferral(line) {
                GateState::Deferred
            } else {
                state
            };
            report.gates.push(Gate {
                id: gate_id(text),
                state,
                evidence: tail.evidence,
                text: text.to_string(),
                ask: tail.ask,
                ruling: tail.ruling,
                evidence_text: tail.evidence_text,
            });
        } else if t.starts_with(PAUSE) {
            if is_signed_deferral(line) {
                let text = t.trim_start_matches(PAUSE).trim();
                report.gates.push(Gate {
                    id: gate_id(text),
                    state: GateState::Deferred,
                    evidence: tail.evidence,
                    text: text.to_string(),
                    ask: tail.ask,
                    ruling: tail.ruling,
                    evidence_text: tail.evidence_text,
                });
            } else if !line.contains("[date]") && !line.contains("[reason]") {
                report.bad_deferrals.push(t.to_string());
            }
        }
    }
    report
}

/// How many gates an intent has and how many are done: ONE owner for the count (INT-212 G6).
/// A deferred gate counts in the total and not as done.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GateCount {
    pub total: usize,
    pub done: usize,
    /// The count came from legacy emoji marks outside any Success Criteria section.
    pub legacy: bool,
}

/// Count the gates in an intent file. When the Success Criteria section holds gates, they are
/// the count. A file with none there but legacy emoji marks elsewhere is an old-format intent:
/// those marks are counted and the result says so. Anything else is zero.
pub fn count(content: &str) -> GateCount {
    let gates = read_gates(content).gates;
    if !gates.is_empty() {
        return GateCount {
            total: gates.len(),
            done: gates.iter().filter(|g| g.state == GateState::Done).count(),
            legacy: false,
        };
    }
    let open = content.matches(LEGACY_OPEN).count();
    let done = content.matches(LEGACY_DONE).count();
    GateCount {
        total: open + done,
        done,
        legacy: open + done > 0,
    }
}

/// May this intent complete? Refuses with one distinct reason, in this order: a deferral
/// without its signature, no gates at all, an open gate, an open checkbox outside the section.
pub fn check_completion(content: &str) -> Result<(), Refusal> {
    let report = read_gates(content);
    if !report.bad_deferrals.is_empty() {
        return Err(Refusal::BadDeferral(report.bad_deferrals));
    }
    if report.gates.is_empty() {
        return Err(Refusal::NoGates {
            section_found: report.section_found,
        });
    }
    let open: Vec<String> = report
        .gates
        .iter()
        .filter(|g| g.state == GateState::Open)
        .map(|g| g.text.clone())
        .collect();
    if !open.is_empty() {
        return Err(Refusal::OpenGates(open));
    }
    if evidence_is_required(content) {
        let bare: Vec<String> = report
            .gates
            .iter()
            .filter(|g| g.state == GateState::Done && g.evidence_text.trim().is_empty())
            .map(|g| g.text.clone())
            .collect();
        if !bare.is_empty() {
            return Err(Refusal::MissingEvidence(bare));
        }
    }
    if !report.stray_open.is_empty() {
        return Err(Refusal::StrayOpen(report.stray_open));
    }
    Ok(())
}

/// The full text of a refusal, one entry per line, for the person at the terminal.
pub fn explain(refusal: &Refusal, today: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    match refusal {
        Refusal::BadDeferral(lines) => {
            out.push("Deferral format error -- missing approval signature:".to_string());
            for l in lines {
                out.push(format!("  {}", l));
            }
            out.push(String::new());
            out.push("Required format:".to_string());
            out.push(format!(
                "  {} gate -- deferred: [reason] -- approved by: christian {}",
                PAUSE, today
            ));
        }
        Refusal::NoGates { section_found } => {
            if *section_found {
                out.push(
                    "cicomplete BLOCKED -- the Success Criteria section holds no gates".to_string(),
                );
            } else {
                out.push("cicomplete BLOCKED -- this intent has no Success Criteria section, so it has no gates".to_string());
            }
            out.push(String::new());
            out.push("Complete means the gates were demonstrated; an intent with none cannot be complete.".to_string());
            out.push(
                "Add at least one real gate (a bare `...` placeholder does not count):".to_string(),
            );
            out.push("  ## Success Criteria".to_string());
            out.push("  - [ ] G1 what must be true".to_string());
        }
        Refusal::OpenGates(gates) => {
            out.push(format!(
                "cicomplete BLOCKED -- {} open gate(s) require demonstration or formal deferral:",
                gates.len()
            ));
            out.push(String::new());
            for g in gates {
                out.push(format!("  {}", g));
            }
            out.push(String::new());
            out.push("To defer a gate, edit the intent file and use:".to_string());
            out.push(format!(
                "  {} gate description -- deferred: [reason] -- approved by: christian {}",
                PAUSE, today
            ));
            out.push(String::new());
            out.push("cicomplete blocked. Demonstrate gates or formally defer them.".to_string());
        }
        Refusal::MissingEvidence(gates) => {
            out.push(format!(
                "cicomplete BLOCKED -- {} done format 2 gate(s) have an empty evidence: field:",
                gates.len()
            ));
            for g in gates {
                out.push(format!("  {}", g));
            }
        }
        Refusal::StrayOpen(lines) => {
            out.push(format!(
                "cicomplete BLOCKED -- {} unticked checkbox(es) outside the Success Criteria section:",
                lines.len()
            ));
            out.push(String::new());
            for l in lines {
                out.push(format!("  {}", l));
            }
            out.push(String::new());
            out.push(
                "These are not gates, and they are not ignored either. Tick them, move them into"
                    .to_string(),
            );
            out.push(
                "Success Criteria as gates, or reword them so they are not checkboxes.".to_string(),
            );
        }
    }
    out
}

/// One line naming the reason, for the error that ends the command.
pub fn summary(refusal: &Refusal) -> String {
    match refusal {
        Refusal::BadDeferral(v) => {
            format!("{} deferral(s) missing the approval signature", v.len())
        }
        Refusal::NoGates {
            section_found: true,
        } => "the Success Criteria section holds no gates".to_string(),
        Refusal::NoGates {
            section_found: false,
        } => "no Success Criteria section, so no gates".to_string(),
        Refusal::OpenGates(v) => format!("{} open gate(s)", v.len()),
        Refusal::MissingEvidence(v) => {
            format!(
                "{} done format 2 gate(s) with an empty evidence field",
                v.len()
            )
        }
        Refusal::StrayOpen(v) => format!(
            "{} unticked checkbox(es) outside the Success Criteria section",
            v.len()
        ),
    }
}

/// One intent gate standing, for the unmet-gates query (INT-212 G9). A deferred gate counts in
/// the total and is neither done nor open.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    pub total: usize,
    pub done: usize,
    pub deferred: usize,
    pub open: usize,
    /// The gates are legacy emoji marks outside any Success Criteria section.
    pub legacy: bool,
}

/// The standing of the gates of an intent. When the Success Criteria section holds gates, they are the
/// standing. A file with none there but legacy emoji marks elsewhere is an old-format intent: the
/// marks are counted and the result says so. Anything else is zero.
pub fn standing(content: &str) -> Standing {
    let gates = read_gates(content).gates;
    if !gates.is_empty() {
        let of = |s: GateState| gates.iter().filter(|g| g.state == s).count();
        return Standing {
            total: gates.len(),
            done: of(GateState::Done),
            deferred: of(GateState::Deferred),
            open: of(GateState::Open),
            legacy: false,
        };
    }
    let c = count(content);
    Standing {
        total: c.total,
        done: c.done,
        deferred: 0,
        open: c.total - c.done,
        legacy: c.legacy,
    }
}

#[cfg(test)]
mod standing_tests {
    use super::*;

    fn section(gates: &[&str]) -> String {
        format!("## Success Criteria\n\n{}\n", gates.join("\n"))
    }

    #[test]
    fn markdown_gates_are_counted_by_state() {
        let pause = char::from_u32(0x23F8).unwrap();
        let deferral = format!(
            "{} G4 later -- deferred: no time -- approved by: christian 2026-10-22",
            pause
        );
        let c = section(&["- [x] G1 a", "- [x] G2 b", "- [ ] G3 c", &deferral]);
        assert_eq!(
            standing(&c),
            Standing {
                total: 4,
                done: 2,
                deferred: 1,
                open: 1,
                legacy: false
            }
        );
    }

    #[test]
    fn an_intent_with_every_gate_done_has_none_open() {
        let c = section(&["- [x] G1 a", "- [x] G2 b"]);
        let s = standing(&c);
        assert_eq!((s.total, s.done, s.open), (2, 2, 0));
    }

    #[test]
    fn legacy_marks_outside_a_section_count_and_say_so() {
        let open = char::from_u32(0x2B1C).unwrap();
        let done = char::from_u32(0x2705).unwrap();
        let c = format!("## Notes\n{} a\n{} b\n", open, done);
        assert_eq!(
            standing(&c),
            Standing {
                total: 2,
                done: 1,
                deferred: 0,
                open: 1,
                legacy: true
            }
        );
    }

    #[test]
    fn an_intent_with_no_gates_has_a_zero_standing() {
        assert_eq!(standing("## Notes\nsome text\n"), Standing::default());
    }

    #[test]
    fn a_stray_checkbox_outside_the_section_is_not_a_gate() {
        let c = "## Notes\n- [ ] stray\n## Success Criteria\n- [x] G1 a\n";
        let s = standing(c);
        assert_eq!((s.total, s.done, s.open), (1, 1, 0));
    }

    #[test]
    fn the_section_wins_over_legacy_marks_in_prose() {
        let done = char::from_u32(0x2705).unwrap();
        let c = format!(
            "## Notes\n{} prose\n## Success Criteria\n- [ ] G1 a\n",
            done
        );
        let s = standing(&c);
        assert_eq!((s.total, s.done, s.open, s.legacy), (1, 0, 1, false));
    }
}

#[cfg(test)]
mod explain_tests {
    use super::*;
    use std::collections::HashSet;

    fn all() -> Vec<Refusal> {
        vec![
            Refusal::BadDeferral(vec!["\u{23F8} G2 b".to_string()]),
            Refusal::NoGates {
                section_found: false,
            },
            Refusal::NoGates {
                section_found: true,
            },
            Refusal::OpenGates(vec!["G1 x".to_string()]),
            Refusal::MissingEvidence(vec!["G1 done".to_string()]),
            Refusal::StrayOpen(vec!["- [ ] remember".to_string()]),
        ]
    }

    #[test]
    fn every_refusal_has_its_own_headline() {
        let heads: HashSet<String> = all()
            .iter()
            .map(|r| explain(r, "2026-10-22")[0].clone())
            .collect();
        assert_eq!(heads.len(), 6);
    }

    #[test]
    fn every_refusal_has_its_own_summary() {
        let sums: HashSet<String> = all().iter().map(summary).collect();
        assert_eq!(sums.len(), 6);
        assert!(sums.iter().all(|s| !s.is_empty()));
    }

    #[test]
    fn the_deferral_hint_carries_the_pause_mark_and_the_date() {
        let text = explain(&Refusal::OpenGates(vec!["G1 x".to_string()]), "2026-10-22").join("\n");
        assert!(text.contains(PAUSE));
        assert!(text.contains("approved by: christian 2026-10-22"));
        assert!(text.contains("G1 x"));
    }

    #[test]
    fn a_stray_box_is_named_as_outside_the_section() {
        let first = explain(&Refusal::StrayOpen(vec!["- [ ] a".to_string()]), "d")[0].clone();
        assert!(first.contains("outside the Success Criteria section"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DONE: &str = "## Success Criteria\n\n- [x] G1 first\n<!-- evidence: shown -->\n- [x] G2 second\n<!-- evidence: shown -->\n";

    fn sc(body: &str) -> String {
        format!("## Success Criteria\n\n{}", body)
    }

    fn open_len(r: Result<(), Refusal>) -> usize {
        match r {
            Err(Refusal::OpenGates(v)) => v.len(),
            _ => 0,
        }
    }

    #[test]
    fn all_done_completes() {
        assert_eq!(check_completion(DONE), Ok(()));
    }

    #[test]
    fn an_open_gate_refuses() {
        assert_eq!(
            open_len(check_completion(&sc("- [x] G1 a\n- [ ] G2 b\n"))),
            1
        );
    }

    #[test]
    fn a_partial_gate_reads_as_open() {
        assert_eq!(open_len(check_completion(&sc("- [~] G1 half\n"))), 1);
    }

    #[test]
    fn a_legacy_open_gate_still_refuses() {
        assert_eq!(open_len(check_completion(&sc("\u{2B1C} G1 old\n"))), 1);
    }

    #[test]
    fn zero_gates_refuse() {
        let r = check_completion(&sc("Prose only, no checkboxes.\n"));
        assert_eq!(
            r,
            Err(Refusal::NoGates {
                section_found: true
            })
        );
    }

    #[test]
    fn a_missing_section_refuses() {
        let r = check_completion("## Gates\n- [x] G1 a\n");
        assert_eq!(
            r,
            Err(Refusal::NoGates {
                section_found: false
            })
        );
    }

    #[test]
    fn a_placeholder_alone_is_not_a_gate() {
        let r = check_completion(&sc("- [x] ...\n"));
        assert_eq!(
            r,
            Err(Refusal::NoGates {
                section_found: true
            })
        );
    }

    #[test]
    fn a_bare_pause_mark_does_not_exempt_an_open_gate() {
        assert_eq!(
            open_len(check_completion(&sc("- [ ] G1 later \u{23F8} someday\n"))),
            1
        );
    }

    #[test]
    fn a_signed_deferral_completes() {
        let body =
            "- [x] G1 a\n\u{23F8} G2 b -- deferred: no data -- approved by: christian 2026-10-22\n";
        assert_eq!(check_completion(&sc(body)), Ok(()));
    }

    #[test]
    fn an_unsigned_deferral_refuses() {
        let r = check_completion(&sc("- [x] G1 a\n\u{23F8} G2 b -- deferred: no data\n"));
        assert!(matches!(r, Err(Refusal::BadDeferral(ref v)) if v.len() == 1));
    }

    #[test]
    fn the_format_example_line_is_skipped() {
        let body =
            "- [x] G1 a\n\u{23F8} gate -- deferred: [reason] -- approved by: christian [date]\n";
        assert_eq!(check_completion(&sc(body)), Ok(()));
    }

    #[test]
    fn an_open_box_outside_the_section_is_named_apart() {
        let r = check_completion(&format!("{}\n## Notes\n- [ ] remember this\n", DONE));
        assert!(matches!(r, Err(Refusal::StrayOpen(ref v)) if v.len() == 1));
    }

    #[test]
    fn a_done_box_outside_the_section_does_not_block() {
        assert_eq!(
            check_completion(&format!("{}\n## Notes\n- [x] aside\n", DONE)),
            Ok(())
        );
    }

    #[test]
    fn a_done_box_outside_the_section_is_not_counted() {
        let r = read_gates(&format!("{}\n## Notes\n- [x] aside\n", DONE));
        assert_eq!(r.gates.len(), 2);
    }

    #[test]
    fn fenced_and_commented_boxes_are_ignored() {
        let tail = "\n## Notes\n```\n- [ ] in a fence\n```\n<!-- note\n- [ ] in a comment\n-->\n";
        assert_eq!(check_completion(&format!("{}{}", DONE, tail)), Ok(()));
    }

    #[test]
    fn the_heading_is_found_in_the_forms_the_ledger_uses() {
        assert!(read_gates("## Success criteria\n- [ ] G1 a\n").section_found);
        assert!(
            read_gates("## Success Criteria (gates -- refine at cistart)\n- [ ] G1 a\n")
                .section_found
        );
        assert!(!read_gates("## Gates\n- [ ] G1 a\n").section_found);
    }

    #[test]
    fn states_ids_and_evidence_are_read() {
        let body = "- [x] G1 a\n      wrapped on a second line\n<!-- evidence: c0ffee -->\n- [ ] G2 b\n- [x] G3 c\n\u{23F8} G4 d -- deferred: x -- approved by: christian 2026-10-22\n";
        let g = read_gates(&sc(body)).gates;
        let got: Vec<(Option<&str>, GateState, bool)> = g
            .iter()
            .map(|x| (x.id.as_deref(), x.state, x.evidence))
            .collect();
        assert_eq!(
            got,
            vec![
                (Some("G1"), GateState::Done, true),
                (Some("G2"), GateState::Open, false),
                (Some("G3"), GateState::Done, false),
                (Some("G4"), GateState::Deferred, false),
            ]
        );
    }

    #[test]
    fn legacy_marks_and_a_capital_x_are_read_in_the_section() {
        let g = read_gates(&sc(
            "\u{2B1C} G1 old open\n\u{2705} G2 old done\n- [X] G3 big x\n",
        ))
        .gates;
        let states: Vec<GateState> = g.iter().map(|x| x.state).collect();
        assert_eq!(
            states,
            vec![GateState::Open, GateState::Done, GateState::Done]
        );
    }

    #[test]
    fn a_wrapped_evidence_field_is_the_receipt() {
        let c =
            "## Success Criteria\n- [x] G1 done\n      evidence: the receipt\n      continued\n";
        let g = &read_gates(c).gates[0];
        assert!(g.evidence, "a wrapped evidence field is the receipt");
    }

    #[test]
    fn a_ruled_line_does_not_hide_the_receipt() {
        let c = "## Success Criteria\n- [x] G1 done\nRULED 2026-11-14 (Christian): yes\n<!-- evidence: shown -->\n";
        assert!(
            read_gates(c).gates[0].evidence,
            "RULED must not hide the receipt"
        );
    }

    #[test]
    fn an_empty_v2_evidence_refuses_completion() {
        let c = "---\nformat: 2\n---\n## Success Criteria\n- [x] G1 done\n      evidence:\n";
        assert!(
            check_completion(c).is_err(),
            "a ticked v2 gate with an empty evidence field refuses"
        );
    }

    /// INT-329 G1. Format 3 keeps the format 2 evidence rule. Format 2 still
    /// refuses an empty receipt. Format 1 still completes without one.
    #[test]
    fn a_format_3_ticked_gate_with_empty_evidence_is_refused() {
        let v3 = "---\nformat: 3\n---\n## Success Criteria\n- [x] G1 done\n      evidence:\n";
        assert_eq!(
            check_completion(v3),
            Err(Refusal::MissingEvidence(vec!["G1 done".to_string()])),
            "a format 3 ticked gate with empty evidence is refused"
        );
        let v2 = "---\nformat: 2\n---\n## Success Criteria\n- [x] G1 done\n      evidence:\n";
        assert!(
            matches!(check_completion(v2), Err(Refusal::MissingEvidence(_))),
            "format 2 still refuses an empty evidence field"
        );
        let v1 = "---\n---\n## Success Criteria\n- [x] G1 done\n";
        assert_eq!(check_completion(v1), Ok(()), "format 1 is unchanged");
    }

    #[test]
    fn a_real_v1_intent_on_disk_still_completes() {
        let content = include_str!(
            "testdata/308-ledger-commands-for-cistart-intent-add-and-the-seal-note.md"
        );
        let fm = content.split("---").nth(1).unwrap_or("");
        assert!(
            !fm.lines().any(|line| line
                .trim()
                .strip_prefix("format:")
                .is_some_and(|v| v.trim() == "2")),
            "308 is a v1 file"
        );
        assert_eq!(check_completion(&content), Ok(()));
    }

    #[test]
    fn the_v1_completion_test_does_not_open_the_real_ledger_file() {
        let src = include_str!("gates.rs");
        let needle = format!("intents/open/{}-ledger-commands", 308);
        assert!(
            !src.contains(&needle),
            "a_real_v1_intent_on_disk_still_completes still opens the real ledger file"
        );
    }

    #[test]
    fn the_orphaned_v1_receipt_test_is_marked_as_a_test() {
        let src = include_str!("gates.rs");
        let marker = "fn a_v1_ticked_gate_without_a_receipt_still_completes";
        let idx = src.rfind(marker).expect("the v1 receipt test exists");
        let before = src[..idx].trim_end();
        assert!(
            before.ends_with("#[test]"),
            "a_v1_ticked_gate_without_a_receipt_still_completes has no #[test]"
        );
    }

    #[test]
    fn a_v1_ticked_gate_without_a_receipt_still_completes() {
        let c = "---\n---\n## Success Criteria\n- [x] G1 done\n";
        assert_eq!(check_completion(c), Ok(()));
    }

    #[test]
    fn a_v2_gate_with_evidence_completes_and_a_deferral_still_counts() {
        let done = "---\nformat: 2\n---\n## Success Criteria\n- [x] G1 done\n      evidence: shown\n      in the log\n";
        assert_eq!(check_completion(done), Ok(()));
        let deferred = "---\nformat: 2\n---\n## Success Criteria\n\u{23f8} G1 later -- deferred: no data -- approved by: christian 2026-11-14\n";
        assert_eq!(check_completion(deferred), Ok(()));
    }
}

#[cfg(test)]
mod count_tests {
    use super::*;

    fn with_section(body: &str) -> String {
        format!("# t\n\n## Success Criteria\n\n{}\n", body)
    }

    #[test]
    fn markdown_gates_are_counted() {
        let c = count(&with_section("- [x] G1 a\n- [ ] G2 b\n- [x] G3 c\n"));
        assert_eq!((c.total, c.done, c.legacy), (3, 2, false));
    }

    #[test]
    fn legacy_marks_inside_the_section_are_counted() {
        let c = count(&with_section(
            "\u{2B1C} G1 old open\n\u{2705} G2 old done\n",
        ));
        assert_eq!((c.total, c.done), (2, 1));
    }

    #[test]
    fn legacy_marks_without_a_section_are_counted_and_flagged() {
        let c = count("# old\n\n\u{2B1C} a\n\u{2705} b\n\u{2705} c\n");
        assert_eq!((c.total, c.done, c.legacy), (3, 2, true));
    }

    #[test]
    fn no_gates_at_all_is_zero_and_not_flagged() {
        let c = count("# nothing here\n");
        assert_eq!((c.total, c.done, c.legacy), (0, 0, false));
    }

    #[test]
    fn a_stray_open_box_outside_the_section_is_not_a_gate() {
        let c = count("- [ ] remember\n\n## Success Criteria\n\n- [x] G1 a\n");
        assert_eq!((c.total, c.done), (1, 1));
    }

    #[test]
    fn marks_in_prose_and_comments_do_not_count_when_the_section_has_gates() {
        let c = count(&with_section(
            "- [x] G1 a\n<!-- evidence: x \u{2705} y -->\nprose \u{2B1C} here\n",
        ));
        assert_eq!((c.total, c.done), (1, 1));
    }
}
