//! INT-208: a failure is a VALUE before it is a string.
//!
//! WHY. `CommandResult::Error` carried a `String`, and 304 sites constructed one while about twelve
//! consumed it -- but those twelve want DIFFERENT THINGS from the same failure. `engine.rs:1369`
//! wants an exit code. `exec.rs:483` clones the text as stderr for the knowledge engine.
//! `exec.rs:1023` wraps it in a capture error. Only three or four actually print it. Four consumers,
//! four projections, and a string can only serve the one it was formatted for.
//!
//! ⚠️ THE SHELL HAS PAID FOR THIS ONCE ALREADY. INT-169 recorded an exit status formatted INTO a
//! message and then parsed back out, so the shell printed "exited 2" while $? said 1. The fix was to
//! carry the code as data. This is that fix applied to the whole diagnostic rather than one field.
//!
//! ★ AND THE SPANS ALREADY EXIST. `spine/lexer.rs` puts a `Span` on every token and segment, and
//! `ParseError` carries one on five of six variants -- two also carrying operator identity, because
//! (its own comment) "a refusal that cannot say which construct it declined is a worse diagnostic
//! and a weaker test". Nothing consumed any of it: `exec.rs` flattened a `ParseError` with
//! `format!("spine: {e:?}")`. The parser knew exactly where the problem was and said so in Debug.
//!
//! ⚠️ NOT DEFINED IN TERMS OF A CRATE, per the intent and per INT-207's precedent. INT-198 ruled
//! miette as a RENDERER of this model, not the model. If the crate becomes the type, the crate
//! becomes the architecture and the JSON, editor and snapshot renderers never happen.
//!
//! ⭐ ONE DOOR, NOT TWO. A second `CommandResult` variant would have let migrated and unmigrated
//! sites coexist -- and left twelve consumers handling both arms forever. That is the two-owners
//! shape this ledger keeps removing. `From<String>` instead makes the 304-site pass MECHANICAL and
//! COMPILER-VERIFIED: it cannot miss one, because it will not build until every site converts.
//! A Diagnostic built from a bare string has a message and nothing else -- nothing is faked, and a
//! site that knows more can say more, later, on its own.

/// How bad it is. Separate from the exit code, which answers a different question.
///
/// ONE VARIANT FOR NOW. Warning and Note were written and deleted the same hour: nothing in fsh
/// constructs them, and an enum arm no caller reaches is a promise about a future that has not
/// arrived. They return when a site actually needs them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
        }
    }
}

/// A region of the source line this diagnostic is about, with what to say against it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// One failure, as a value.
///
/// The fields are INT-199's convention made structural rather than re-argued at each site: what
/// happened (`message`), where (`labels`), and what to do next (`help`). `code` is for a tool or a
/// test to match on without reading prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    /// Empty when the failure has no position -- most builtins. NOT a placeholder span, because a
    /// fabricated position is worse than none: it points a reader at innocent text.
    pub labels: Vec<Label>,
    pub help: Option<String>,
    /// A stable identifier (`fsh::spine::unsupported_operator`), not a number. Numbers get reused
    /// and reordered; a path stays meaningful when the code around it moves.
    pub code: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            message: message.into(),
            labels: Vec::new(),
            help: None,
            code: None,
        }
    }

    pub fn with_label(mut self, start: usize, end: usize, text: impl Into<String>) -> Self {
        self.labels.push(Label {
            start,
            end,
            text: text.into(),
        });
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// ⭐ THE CONVERSION THAT MAKES THE MIGRATION MECHANICAL. 304 sites become `msg.into()` and the
/// compiler proves none was missed. A diagnostic made this way carries a message and NOTHING ELSE,
/// which is honest: the site did not know a span, so none is invented.
impl From<String> for Diagnostic {
    fn from(message: String) -> Self {
        Diagnostic::error(message)
    }
}

impl From<&str> for Diagnostic {
    fn from(message: &str) -> Self {
        Diagnostic::error(message)
    }
}

/// Rendering is a SEPARATE CONCERN with more than one implementation -- the whole point of the
/// model. This one matches what the shell printed before, so migration changes nothing visible.
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Diagnostic {
    /// INT-208 GATE 3: the SECOND renderer, and deliberately not cosmetic.
    ///
    /// A terminal rendering serves one reader. This one serves everything else -- a tool asking
    /// what fsh refused and why, an editor placing a squiggle, a test asserting a snapshot. That
    /// those are possible at all is the argument for a model: the same failure, three projections,
    /// none of them reconstructed by parsing prose.
    ///
    /// Built by hand rather than derived. The wire format is a CONTRACT, and a contract that falls
    /// out of struct field names changes silently when a field is renamed.
    pub fn to_json(&self) -> String {
        let labels: Vec<serde_json::Value> = self
            .labels
            .iter()
            .map(|l| serde_json::json!({ "start": l.start, "end": l.end, "text": l.text }))
            .collect();
        serde_json::json!({
            "severity": self.severity.as_str(),
            "message": self.message,
            "labels": labels,
            "help": self.help,
            "code": self.code,
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test asserts by VALUE rather than by matching printed text -- INT-208's fourth gate.
    #[test]
    fn a_diagnostic_is_inspectable_not_just_readable() {
        let d = Diagnostic::error("unsupported operator")
            .with_label(4, 6, "this operator")
            .with_help("the legacy executor handles this")
            .with_code("fsh::spine::unsupported_operator");
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.labels.len(), 1);
        assert_eq!(d.labels[0].start, 4);
        assert_eq!(d.code.as_deref(), Some("fsh::spine::unsupported_operator"));
    }

    /// The string conversion must not FABRICATE a position. A label pointing at innocent text is
    /// worse than no label at all.
    #[test]
    fn a_string_becomes_a_message_and_nothing_more() {
        let d: Diagnostic = "sk not found".to_string().into();
        assert_eq!(d.message, "sk not found");
        assert!(d.labels.is_empty());
        assert!(d.help.is_none());
        assert!(d.code.is_none());
    }

    /// GATE 3's renderer asserted as a CONTRACT, not as text. The wire format is built by hand
    /// rather than derived precisely so a field rename cannot change it silently -- and this parses
    /// the output back rather than string-matching it, so formatting is free to change and the
    /// contract is not.
    #[test]
    fn the_json_contract_is_stable() {
        let d = Diagnostic::error("no target after RedirectOut")
            .with_label(7, 8, "this redirect has nothing to write to")
            .with_code("fsh::spine::missing_redirect_target");
        let v: serde_json::Value = serde_json::from_str(&d.to_json()).expect("valid json");
        assert_eq!(v["severity"], "error");
        assert_eq!(v["code"], "fsh::spine::missing_redirect_target");
        assert_eq!(v["labels"][0]["start"], 7);
        assert_eq!(v["labels"][0]["end"], 8);
        assert!(
            v["help"].is_null(),
            "absent help is null, not an empty string"
        );
    }

    /// Today's terminal output is unchanged for a migrated site.
    #[test]
    fn display_renders_exactly_the_old_string() {
        let d: Diagnostic = "no such file".to_string().into();
        assert_eq!(format!("{}", d), "no such file");
    }
}
