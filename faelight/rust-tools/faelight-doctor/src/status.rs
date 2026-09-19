//! States, tiers and the verdict.
//!
//! ⚠️ ADOPTED, NOT INVENTED. Every type here already existed in `core`'s doctor and is carried
//! over unchanged in meaning. INT-222 says "do not invent a second scale", and a new engine
//! defining its own vocabulary beside a working one would be the two-owners defect it exists
//! to remove.

/// What a check reported.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Pass,
    Warn,
    Fail,
    /// Deliberately not run -- a precondition was absent by design.
    Blocked,
    /// The check COULD NOT RUN. NOT a failure.
    ///
    /// ⭐ THE STATE THAT MAKES THE DIFFERENCE BETWEEN "checked and bad" and "could not check".
    /// Without it a tool reports clean when it has learned nothing, which is INT-192's defect.
    Unknown,
}

/// How much a problem in this check matters.
///
/// The first three are RISK.toml's tiers, deliberately the same names. `Info` is NOT RISK.toml's
/// fourth: there it means nothing reads it at boot; here it means the check MEASURES TRULY BUT
/// NEVER JUDGES, so it is excluded from the verdict entirely.
///
/// ⭐ Tier derives Deserialize and Status/Verdict do NOT. That is the direction of the data:
/// a tier is DECLARED in a TOML definition, while a status and a verdict are PRODUCED by
/// running the check. Nothing should ever read a status out of a file.
#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Critical,
    System,
    User,
    Info,
}

/// The single verdict. Derived, never stored, never weighted.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Verdict {
    Green,
    Amber,
    Red,
}

/// One result, as produced by running a definition.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub id: String,
    pub name: String,
    pub tier: Tier,
    pub status: Status,
    pub message: String,
    pub recovery: Option<String>,
}

/// The worst thing in the highest tier. NO ARITHMETIC.
///
/// A weight factor is subjective and a number built from one has to be defended forever.
///
/// GREEN means every judging check RAN and PASSED -- an Unknown is never green, because a tool
/// that cannot express an undetermined outcome reports clean.
pub fn verdict(results: &[Outcome]) -> Verdict {
    let mut amber = false;
    for r in results.iter().filter(|r| r.tier != Tier::Info) {
        match (r.tier, r.status) {
            (Tier::Critical, Status::Fail)
            | (Tier::Critical, Status::Unknown)
            | (Tier::Critical, Status::Blocked) => return Verdict::Red,
            (_, Status::Fail) | (_, Status::Warn) => amber = true,
            (_, Status::Unknown) | (_, Status::Blocked) => amber = true,
            (_, Status::Pass) => {}
        }
    }
    if amber {
        Verdict::Amber
    } else {
        Verdict::Green
    }
}
