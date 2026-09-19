//! Rendering an outcome the way INT-199 says a failure should read.
//!
//! ⭐ FOUR QUESTIONS, SAME ORDER, EVERY TIME: what happened, did anything change, why, and what
//! to do next. INT-199 was earned from `fpatch` aborting six times and printing only a Python
//! traceback -- the tool behaved correctly every time, refusing a patch whose anchor no longer
//! matched, but THE ONE FACT THAT MATTERED, that nothing had been modified, appeared nowhere.
//! Twice in one session a safe abort was read as a broken tool and the wrong recovery attempted.
//!
//! ⚠️ ONLY RED OUTCOMES GET THIS SHAPE. INT-199's own scope guardrail says adopt it "where
//! failures are actually being read" -- a passing line is read as a tick, and wrapping it in six
//! sections would bury the report it belongs to.

use crate::status::{Outcome, Status};

/// INT-199's severity taxonomy, which the doctor's statuses already map onto.
///
/// ⭐ THE DISTINCTION THAT CARRIES THE DESIGN: "there was not enough information to judge this
/// safely" and "this check hit a defect" must never look the same. `Unknown` IS a safe abort --
/// the check stopped rather than report something it could not stand behind.
fn severity(status: Status) -> &'static str {
    match status {
        Status::Pass => "Info",
        Status::Warn => "Warning",
        Status::Unknown | Status::Blocked => "Safe abort",
        Status::Fail => "Failure",
    }
}

/// Whether this outcome is one a reader needs the full shape for.
pub fn is_red(status: Status) -> bool {
    matches!(
        status,
        Status::Fail | Status::Warn | Status::Unknown | Status::Blocked
    )
}

/// Render one outcome in the INT-199 shape.
///
/// ⚠️ THERE IS NO "POSSIBLE CAUSES" SECTION, AND THAT IS DELIBERATE. `fpatch` could list causes
/// because it knew what it had attempted. A CHECK KNOWS WHAT IT MEASURED, NOT WHY THE MACHINE IS
/// THAT WAY. Inventing plausible causes would be decoration presented as diagnosis, which is the
/// defect this whole intent exists to remove -- so the section is omitted rather than guessed.
pub fn render(o: &Outcome) -> String {
    let rule = "\u{2501}".repeat(58);
    let mut out = String::new();

    out.push_str(&format!("{}\n", rule));
    out.push_str(&format!("  {}\n", o.name.to_uppercase()));
    out.push_str(&format!("{}\n\n", rule));

    out.push_str("Status\n");
    out.push_str(&format!("  {}.\n\n", severity(o.status)));

    // ⭐ PRINCIPLE 2: TELL THE USER WHAT DID NOT HAPPEN. For a doctor the answer is constant and
    // that is exactly why it must be stated -- the absence of side effects is the most
    // reassuring fact available and the one hardest to infer. A reader who has just been told
    // something is wrong should not have to wonder whether the checking made it worse.
    out.push_str("Result\n");
    out.push_str("  Nothing was changed. The doctor only reads.\n\n");

    out.push_str("Reason\n");
    out.push_str(&format!("  {}\n\n", o.message));

    // ⭐ PRINCIPLE 4: RECOVERY IS PART OF THE INTERFACE. An error should begin the debugging
    // workflow, not end it. Where a definition declares no recovery, say so plainly rather than
    // printing an empty heading -- an absent step is itself information.
    out.push_str("Recovery\n");
    match &o.recovery {
        Some(r) => out.push_str(&format!("  {}\n", r)),
        None => out.push_str("  No recovery step is declared for this check.\n"),
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Tier;

    fn outcome(status: Status, recovery: Option<&str>) -> Outcome {
        Outcome {
            id: "example".into(),
            name: "Example Check".into(),
            tier: Tier::System,
            status,
            message: "something measurable was measured".into(),
            recovery: recovery.map(String::from),
        }
    }

    #[test]
    fn a_red_render_answers_all_four_questions() {
        let text = render(&outcome(Status::Fail, Some("do the thing")));
        // ⭐ THE FOUR, IN ORDER. Not a formatting preference -- the order is the design.
        let status = text.find("Status").expect("what happened");
        let result = text.find("Result").expect("did anything change");
        let reason = text.find("Reason").expect("why");
        let recovery = text.find("Recovery").expect("what to do next");
        assert!(status < result && result < reason && reason < recovery);
    }

    #[test]
    fn it_always_says_nothing_was_changed() {
        // ⚠️ THE FACT fpatch NEVER PRINTED. A reader told something is wrong must not be left
        // wondering whether the checking made it worse.
        for s in [Status::Fail, Status::Warn, Status::Unknown, Status::Blocked] {
            let text = render(&outcome(s, None));
            assert!(
                text.contains("Nothing was changed"),
                "{:?} did not state the absence of side effects",
                s
            );
        }
    }

    #[test]
    fn an_unknown_reads_as_a_safe_abort_not_a_failure() {
        // ⭐ "Not enough information to judge safely" and "this check hit a defect" must never
        // look the same.
        let unknown = render(&outcome(Status::Unknown, None));
        let failure = render(&outcome(Status::Fail, None));
        assert!(unknown.contains("Safe abort"));
        assert!(failure.contains("Failure"));
        assert!(!unknown.contains("Failure"));
    }

    #[test]
    fn a_missing_recovery_is_stated_rather_than_left_blank() {
        let text = render(&outcome(Status::Warn, None));
        assert!(text.contains("No recovery step is declared"));
    }

    #[test]
    fn it_never_invents_a_cause() {
        // ⚠️ A check knows what it measured, not why the machine is that way.
        let text = render(&outcome(Status::Fail, Some("do the thing")));
        assert!(!text.contains("Possible cause"));
    }

    #[test]
    fn passes_are_not_red_and_do_not_get_the_shape() {
        assert!(!is_red(Status::Pass));
        assert!(is_red(Status::Fail));
        assert!(is_red(Status::Warn));
        assert!(is_red(Status::Unknown));
        assert!(is_red(Status::Blocked));
    }
}
