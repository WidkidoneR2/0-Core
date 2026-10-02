//! Identity -- is this the machine and tree Project 0 recorded? (INT-269)
//!
//! ⭐ ONE COLLECTOR. This probe reads none of the nine inputs itself: zero_core::fingerprint
//! collects them and compares, the same functions `core fingerprint show` calls, so the doctor
//! and core cannot disagree about what this machine is.
//!
//! ⚠️ THE DOCTOR NEVER WRITES THE RECORD. Only `core fingerprint record` does. With no record the
//! answer is unknown -- "not recorded yet" -- never a pass, and never a default filled in.

use crate::measurement::Measurement;
use zero_core::fingerprint::{collect, compare, Outcome, Record};

/// PASS: every input read and equal to the record. FAIL: all read, some differ -- named.
/// Unknown: no record, an unreadable record, or an input that could not be read.
pub fn fingerprint() -> Measurement {
    let path = zero_core::paths::fingerprint_file();
    let expected = match std::fs::read_to_string(&path) {
        Ok(t) => Record::from_text(&t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Measurement::unknown("not recorded yet -- run: core fingerprint record");
        }
        Err(e) => {
            return Measurement::unknown(format!(
                "could not read the record at {} -- {}",
                path.display(),
                e
            ));
        }
    };
    let live = collect();
    match compare(&live, &expected) {
        Outcome::Pass => Measurement::pass(match live.digest() {
            Some(d) => format!("this is the recorded machine and tree -- digest {}", d),
            None => "this is the recorded machine and tree".to_string(),
        }),
        Outcome::Fail(axes) => {
            Measurement::fail(format!("differs from the record: {}", axes.join(", ")))
        }
        Outcome::Undetermined(missing) => {
            Measurement::unknown(format!("could not read: {}", missing.join(", ")))
        }
    }
}
