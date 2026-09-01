//! INT-191: the vocabulary of the history classifier.
//!
//! ⚠️ THESE TYPES DEFINE WHAT THE ALGORITHM IS ALLOWED TO CLAIM, which is why they exist before it
//! does. Four earlier SQL attempts each produced a different CONFIDENT WRONG ANSWER precisely
//! because they had no way to say "I cannot tell".
//!
//! The classifier answers one question:
//!
//!     Given the current `shell_history`, what command LIFECYCLES can we prove?
//!
//! It deliberately does NOT answer "what exact shell mechanism produced this rewrite?" -- see the
//! two-claims note on `CommandLifecycle`.

/// What the classifier concluded about one position in the history stream.
///
/// ⚠️ THREE ARMS, NOT TWO. `Excluded` is separate from `Unknown` so bookkeeping cannot drift into
/// being "a kind of failure to pair". It is not a failure; it is outside the domain, and folding
/// them together would make the report's denominator lie.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Observation {
    Proven(CommandLifecycle),
    Unknown(UnknownReason),
    Excluded(ExcludedReason),
}

/// One command lifecycle: two endpoints, and separately whatever can be said about the pipeline
/// between them.
///
/// ★★ TWO CLAIMS, TWO FIELDS, AND THE DEPENDENCY RUNS ONE WAY.
///
///     lifecycle proof  ->  transformation explanation
///
/// and never the reverse. "We found a plausible transformation, therefore we found a pair" is the
/// error this shape prevents. `c -> clear` has overwhelming LIFECYCLE evidence -- the producer
/// exhibits that relationship thousands of times -- while its TRANSFORMATION evidence is weak,
/// because today's alias table is not proof of what the table said months ago. An unknown
/// transformation must NOT demote a proven lifecycle.
///
/// The migration depends only on the first claim: the settled schema records the endpoints and
/// deliberately does not enumerate transformations, because that list has already grown from one
/// mechanism to six. Audit observability is what benefits from the second.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct CommandLifecycle {
    /// Exactly what the user typed.
    pub typed: String,
    /// What ran. `None` where the producer emitted no separate executed form.
    pub executed: Option<String>,
    /// Why these observations are one lifecycle.
    pub lifecycle: LifecycleEvidence,
    /// Why their text differs, where that can be established at all.
    pub transformation: TransformationEvidence,
    /// The rows the conclusion rests on -- the audit trail either way.
    pub source_rows: Vec<i64>,
}

/// Why a `TwoWrite` claim holds. ★ The CLAIM stays separate from the REASON for it -- the same
/// discipline applied to transformations, and the reason `TwoWrite` is not two enum variants.
/// A consumer that cares only "is this one lifecycle" reads the claim; a consumer auditing
/// evidence quality reads the basis.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TwoWriteBasis {
    /// The producer stamped both observations with the same execution identity. Not inference:
    /// the rows say so.
    ExecutionId,
    /// Reconstructed from producer-shaped signals -- ordering, timestamp proximity, cwd equality,
    /// and one-pass consumption. The only path available for rows written before the producer
    /// emitted its execution identity.
    Inferred,
}

/// Why these observations belong to one producer lifecycle.
///
/// ⚠️ `SingleWrite` is not a degraded `TwoWrite`. A lifecycle is NOT inherently two rows: `exit`,
/// state-changing builtins and the prefix handlers are intentionally single-write in the current
/// producer.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum LifecycleEvidence {
    /// One producer lifecycle, written as TWO observations.
    ///
    /// ⚠️ Named for what the PRODUCER did, not for the storage shape -- the same reason this type
    /// is not called `PairKind`. "Paired" would describe how the rows sit in a table; `TwoWrite`
    /// describes how the lifecycle was emitted.
    ///
    /// ⚠️ `TwoWrite` means the classifier has evidence these observations are one PRODUCER
    /// lifecycle. It does NOT mean the classifier has explained every transformation between
    /// them -- that claim lives in `transformation`, and its absence must never demote this one.
    TwoWrite { basis: TwoWriteBasis },
    /// The producer emitted a complete lifecycle in one observation.
    SingleWrite,
}

/// A mechanism the shell applies between typed and executed. Internal building blocks that feed a
/// transformation claim -- NOT a list of every interesting thing the shell does.
///
/// ⚠️ ONLY MECHANISMS THAT PRESERVE ONE LIFECYCLE BELONG HERE. Pipeline segmentation deliberately
/// does not: when a typed `a | b` is recorded with an executed form that dropped the pipe, the
/// shell changed the UNIT OF EXECUTION, so the rows may not be endpoints of the same lifecycle at
/// all. That is `UnknownReason::LifecycleBoundaryChanged`, not a transformation.
///
/// ★ `Composed` is the GENERAL case, not a late special case: single-step proofs are compositions
/// of length one. `d -> core doctor run` is alias AND path resolution together, and matched no
/// single rule on the first attempt.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Transformation {
    Identical,
    Alias,
    PathResolution,
    Wrapper,
    ArgumentExpansion,
    GlobExpansion,
    VariableExpansion,
    Composed(Vec<Transformation>),
}

/// Whether the CAUSE of the difference could be established. Separate from the lifecycle claim on
/// purpose -- see `CommandLifecycle`.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TransformationEvidence {
    Proven(Transformation),
    Unknown(UnknownTransformationReason),
}

/// Why no lifecycle could be justified.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum UnknownReason {
    /// Nothing in the stream supports or refutes a relationship.
    InsufficientEvidence,
    /// Two readings are equally supported.
    ConflictingEvidence,
    /// The rows do not sit in a shape the producer is known to emit.
    UnexpectedSequence,
    /// The observations are not comparable endpoints, because the shell changed the unit of
    /// execution between them.
    LifecycleBoundaryChanged,
}

/// Why the cause of a difference could not be established, even where the lifecycle is proven.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum UnknownTransformationReason {
    /// A difference is visible but matches no mechanism the classifier can name yet.
    UnsupportedTransformation,
    /// ⚠️ The explanation source is not the source that produced the row. `shell_aliases` holds
    /// TODAY's mapping; a two-month-old row was produced by whatever it said THEN, and no alias
    /// history exists. A live lookup answers "would this expand this way now?", not "can we prove
    /// it expanded this way then?" -- and can give a false negative in one direction and a false
    /// positive in the other.
    HistoricalDefinitionUnavailable,
    /// A known mechanism explains part of the difference and something else acted on the result.
    PostProcessedResidual,
}

/// Why a row was never a lifecycle candidate.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ExcludedReason {
    /// `TIMING:` and `SUGGEST:` rows -- a separate stream sharing the table, landing BETWEEN the
    /// halves of a pair. That interleaving is why row adjacency is not command adjacency.
    Bookkeeping,
    /// The doctor's `__fsh_doctor_test__` self-test insert.
    SelfTest,
}
