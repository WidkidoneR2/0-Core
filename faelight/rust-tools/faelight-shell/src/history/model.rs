//! INT-191: the vocabulary of the history classifier.
//!
//! ⚠️ THESE TYPES DEFINE WHAT THE ALGORITHM IS ALLOWED TO CLAIM, which is why they exist before it
//! does. The streaming walk is straightforward; the semantic contract is the hard part, and four
//! earlier SQL attempts each produced a different CONFIDENT WRONG ANSWER precisely because they had
//! no way to say "I cannot tell".
//!
//! The classifier answers exactly one question:
//!
//!     Given the current `shell_history`, what command lifecycles can we PROVE?
//!
//! It is not trying to maximise pairs. It is trying to maximise PROVABLE LIFECYCLES while marking
//! explicitly where the evidence runs out.

/// What the classifier concluded about one position in the history stream.
///
/// ⚠️ THREE ARMS, NOT TWO. `Excluded` is deliberately separate from `Unknown` so that bookkeeping
/// cannot drift into being "a kind of failure to pair". It is not a failure -- it is outside the
/// classifier's domain, and folding the two together would make the report's denominator lie.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Observation {
    /// A lifecycle the evidence justifies.
    Proven(CommandLifecycle),
    /// Examined, and no lifecycle can be justified. Carries WHY.
    Unknown(UnknownReason),
    /// Never a lifecycle candidate in the first place.
    Excluded(ExcludedReason),
}

/// One command lifecycle, as the destination model defines it: two endpoints and nothing about the
/// pipeline between them.
///
/// ⚠️ `executed` IS AN `Option`, AND THAT IS THE POINT. A lifecycle is NOT inherently two rows.
/// `exit`, state-changing builtins, and the prefix handlers are intentionally single-write in the
/// current producer, so sometimes the evidence for a lifecycle is two rows and sometimes one.
/// Modelling this as a pair would bake the legacy two-row storage accident into the very tool built
/// to escape it.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct CommandLifecycle {
    /// Exactly what the user typed.
    pub typed: String,
    /// What ran, when the evidence shows it differed or confirms it did not.
    pub executed: Option<String>,
    /// How the conclusion was reached.
    pub proof: Proof,
    /// Row ids the conclusion rests on -- one or two today, and the audit trail either way.
    pub source_rows: Vec<i64>,
}

/// HOW a lifecycle was justified.
///
/// ⚠️ Named `Proof` rather than `PairKind` or `MatchKind` on purpose. The implementation may be
/// heuristic internally; the public vocabulary is about confidence in the conclusion, not about
/// which rule happened to fire.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Proof {
    /// Typed and executed are the same text: no transformation fired.
    Identical,
    /// A single row that is the whole lifecycle -- the producer never wrote an executed form.
    SingleWrite,
    /// The alias table maps typed to executed.
    Alias,
    /// The program was replaced by an absolute path.
    PathResolution,
    /// The program was rewritten to a wrapper or script path.
    Wrapper,
    /// Tilde or variable expansion occurred in the ARGUMENTS, not the program.
    ArgumentExpansion,
    /// More than one transformation applied at once. `d` -> `core doctor run` is alias AND path
    /// resolution composed, and it matched no single rule on the first attempt.
    Composed(Vec<Proof>),
}

/// WHY no lifecycle could be justified. Named now even where unused, so later additions are
/// additive rather than disruptive.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum UnknownReason {
    /// Nothing in the stream supports or refutes a transformation.
    InsufficientEvidence,
    /// Two readings are equally supported.
    ConflictingEvidence,
    /// The rows do not sit in a shape the producer is known to emit.
    UnexpectedSequence,
    /// A transformation is visible but is not one the classifier can name yet.
    UnsupportedTransformation,
}

/// WHY a row was never a candidate.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ExcludedReason {
    /// `TIMING:` and `SUGGEST:` rows -- a separate stream that happens to share the table, and
    /// which lands BETWEEN the halves of a pair. That interleaving is why row adjacency is not
    /// command adjacency.
    Bookkeeping,
    /// The doctor's `__fsh_doctor_test__` self-test insert.
    SelfTest,
}
