//! INT-191: the streaming classifier.
//!
//! ★★ A LIFECYCLE IS NOT A RELATIONSHIP BETWEEN TWO ROWS. It is a relationship between two rows
//! AFTER THE WALK HAS DECIDED THEY WERE CONSUMED TOGETHER. That statefulness is the whole point,
//! and it is what four earlier SQL attempts could not express -- each produced a different
//! confident wrong answer because the decision at row N depends on what was consumed at N-1.
//!
//! Typing `c`, an alias for `clear`, emits two rows: `c`, `clear`. Typing `c` and then separately
//! typing `clear` emits four: `c`, `clear`, `clear`, `clear`. A stateless query sees the same
//! textual opportunities in both. Only consumption distinguishes them.
//!
//! ⚠️ THE TRAVERSAL IS DELIBERATELY BORING -- a pending slot and one `try_pair`. Sophistication
//! belongs in the evidence predicates, never in the walk. If a future reader can replace this with
//! a join without noticing, the invariant has been lost.
//!
//! DIRECTION OF INFERENCE, and the error it prevents:
//!
//!     consume evidence -> establish lifecycle -> explain transformation if possible
//!
//! never `find an explanation -> pretend it found a lifecycle`.

use super::model::*;

/// One row as the producer wrote it. `timestamp` and `cwd` are evidence, not decoration: both
/// writes of a single command share a cwd and land within a second or two of each other.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct HistoryRow {
    pub id: i64,
    pub command: String,
    pub timestamp: i64,
    pub cwd: String,
}

/// Rows that are not lifecycle candidates at all. They share the table and land BETWEEN the halves
/// of a pair, which is why row adjacency is not command adjacency.
#[allow(dead_code)]
fn excluded(row: &HistoryRow) -> Option<ExcludedReason> {
    if row.command.starts_with("TIMING:") || row.command.starts_with("SUGGEST:") {
        Some(ExcludedReason::Bookkeeping)
    } else if row.command == "__fsh_doctor_test__" {
        Some(ExcludedReason::SelfTest)
    } else {
        None
    }
}

/// Producer-shaped evidence that `executed` is the execution half of `typed`.
///
/// ⚠️ DELIBERATELY CONSERVATIVE AND KNOWN TO BE INCOMPLETE. It rests only on facts the producer
/// guarantees: both writes happen inside one command execution, so they share a cwd and sit within
/// a second or two. It will therefore OVER-PAIR two independent commands typed in rapid succession
/// in the same directory. That is a predicate weakness, not a traversal weakness, and this is where
/// future evidence belongs -- session ids, sequence markers, or a producer-emitted lifecycle id
/// would each settle it outright.
#[allow(dead_code)]
fn try_pair(typed: &HistoryRow, executed: &HistoryRow) -> Option<LifecycleEvidence> {
    if typed.cwd != executed.cwd {
        return None;
    }
    if (executed.timestamp - typed.timestamp).abs() > 2 {
        return None;
    }
    Some(LifecycleEvidence::TwoWrite)
}

/// Walk the stream in producer order, consuming what can be proven.
///
/// Transformation evidence is deliberately left `Unknown` here: establishing a lifecycle and
/// explaining the text difference are separate claims, and this commit proves the first only.
#[allow(dead_code)]
pub fn classify(rows: impl IntoIterator<Item = HistoryRow>) -> Vec<Observation> {
    let mut out = Vec::new();
    let mut pending: Option<HistoryRow> = None;

    for row in rows {
        // Bookkeeping never disturbs lifecycle state -- it is emitted where it sits and the
        // pending candidate survives across it.
        if let Some(reason) = excluded(&row) {
            out.push(Observation::Excluded(reason));
            continue;
        }
        match pending.take() {
            None => pending = Some(row),
            Some(held) => match try_pair(&held, &row) {
                Some(lifecycle) => {
                    // CONSUMES BOTH. `row` does not remain available to pair forwards -- that is
                    // the invariant the negative test guards.
                    out.push(Observation::Proven(CommandLifecycle {
                        typed: held.command,
                        executed: Some(row.command),
                        lifecycle,
                        transformation: TransformationEvidence::Unknown(
                            UnknownTransformationReason::UnsupportedTransformation,
                        ),
                        source_rows: vec![held.id, row.id],
                    }));
                }
                None => {
                    out.push(Observation::Unknown(UnknownReason::InsufficientEvidence));
                    pending = Some(row);
                }
            },
        }
    }
    if pending.is_some() {
        out.push(Observation::Unknown(UnknownReason::InsufficientEvidence));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64, command: &str, timestamp: i64) -> HistoryRow {
        HistoryRow {
            id,
            command: command.to_string(),
            timestamp,
            cwd: "/home/christian/0-core".to_string(),
        }
    }

    /// ★★★ THE INVARIANT. `c, clear, clear` must give ONE lifecycle and one unresolved row -- not
    /// two pairings. Once `c`+`clear` is consumed, the second `clear` is evaluated fresh with
    /// nothing to pair backwards to. This is what proves the classifier models the PRODUCER rather
    /// than doing graph matching over similar text.
    #[test]
    fn consumed_rows_do_not_pair_again() {
        let out = classify(vec![
            row(1, "c", 100),
            row(2, "clear", 100),
            row(3, "clear", 100),
        ]);
        assert_eq!(out.len(), 2, "{out:#?}");
        assert!(matches!(out[0], Observation::Proven(_)));
        assert!(matches!(out[1], Observation::Unknown(_)));
    }

    #[test]
    fn bookkeeping_does_not_break_a_pair() {
        // TIMING: rows land BETWEEN the halves of a pair in the real corpus.
        let out = classify(vec![
            row(1, "c", 100),
            row(2, "TIMING:c:12", 100),
            row(3, "clear", 100),
        ]);
        assert!(matches!(
            out[0],
            Observation::Excluded(ExcludedReason::Bookkeeping)
        ));
        assert!(matches!(out[1], Observation::Proven(_)));
    }

    #[test]
    fn distant_rows_do_not_pair() {
        let out = classify(vec![row(1, "c", 100), row(2, "clear", 900)]);
        assert!(out.iter().all(|o| matches!(o, Observation::Unknown(_))));
    }

    #[test]
    fn different_cwd_does_not_pair() {
        let mut b = row(2, "clear", 100);
        b.cwd = "/tmp".to_string();
        let out = classify(vec![row(1, "c", 100), b]);
        assert!(out.iter().all(|o| matches!(o, Observation::Unknown(_))));
    }
}
