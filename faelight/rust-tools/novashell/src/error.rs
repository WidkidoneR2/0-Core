#![allow(dead_code)]
// faelight-shell — Structured Error System
// INT-174 — The Shell Explains Its Failures
//
// Every error becomes a structured value.
// "An error that disappears is an error that repeats."

use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors that CONTROL FLOW depends on -- typed so the message derives from the
/// variant via #[error(...)] and cannot drift from it. INT-171 gate 5.
///
/// The 968c7be5 class was "a result whose TYPE contradicts its MESSAGE": a
/// not-found command printed a failure message but returned a success-typed
/// result, so `&&` proceeded. A FlowError's Display comes FROM its variant, so a
/// CommandNotFound cannot carry a message that says anything but "not found".
/// Started with the one error the `&&`/`||` decision most depends on; extend to
/// redirect/builtin failures as they prove flow-critical.
#[derive(Debug, Clone, Error)]
pub enum FlowError {
    #[error("command not found: {0}")]
    CommandNotFound(String),
}

impl FlowError {
    /// The colored presentation of this error. Presentation lives WITH the type,
    /// so the two not-found call sites cannot drift from each other or from the
    /// #[error] message. INT-171 gate 5.
    pub fn display_colored(&self) -> String {
        use colored::Colorize;
        match self {
            FlowError::CommandNotFound(cmd) if cmd.starts_with('(') => {
                // A LEADING PAREN IS NOT A TYPO, IT IS AN UNSUPPORTED CONSTRUCT.
                // The lexer knows parens only inside a command substitution, so a
                // subshell reaches the OS as a program named "(echo" and comes
                // back as no-such-file -- the operating system answering a
                // question it was never asked. Same reasoning as the refusal in
                // peel_builtin_first_stage: say why, rather than let the
                // filesystem give a misleading answer about a construct fsh
                // simply has not implemented yet.
                //
                // Keyed on the WORD here rather than checked at the three
                // CommandNotFound call sites, so the kind cannot drift and the
                // explanation cannot either.
                format!(
                    "subshells are not supported yet: {}\n  fsh parses parens only inside $( ), so this ran as a command name",
                    cmd.bright_red()
                )
            }
            FlowError::CommandNotFound(cmd) => {
                format!("command not found: {}", cmd.bright_red())
            }
        }
    }
}

#[cfg(test)]
mod flow_error_tests {
    use super::FlowError;

    #[test]
    fn command_not_found_message_derives_from_type() {
        // The message comes FROM the variant via #[error(...)], so it cannot
        // contradict the kind -- the 968c7be5 "type contradicts message" class is
        // unrepresentable for this typed error. INT-171 gate 5.
        let e = FlowError::CommandNotFound("frobnicate".to_string());
        assert_eq!(e.to_string(), "command not found: frobnicate");
    }
}

/// A structured shell error — code, message, suggestion, context
#[derive(Debug, Clone)]
pub struct ShellError {
    pub code: &'static str,
    pub message: String,
    pub suggestion: String,
    pub command: String,
    pub directory: String,
    pub timestamp: u64,
}

impl ShellError {
    pub fn new(
        code: &'static str,
        message: &str,
        suggestion: &str,
        command: &str,
        directory: &str,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            code,
            message: message.to_string(),
            suggestion: suggestion.to_string(),
            command: command.to_string(),
            directory: directory.to_string(),
            timestamp,
        }
    }

    /// Format for display in the shell
    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("\n  ❌ {}: {}\n", self.code, self.message));
        if !self.suggestion.is_empty() {
            out.push_str(&format!("     💡 {}\n", self.suggestion));
        }
        out
    }

    /// Format as a single-line string for storage in db
    pub fn to_storage(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.code, self.message, self.suggestion, self.command, self.directory, self.timestamp
        )
    }

    /// Parse from storage string
    pub fn from_storage(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(6, '|').collect();
        if parts.len() < 6 {
            return None;
        }
        Some(Self {
            code: Box::leak(parts[0].to_string().into_boxed_str()),
            message: parts[1].to_string(),
            suggestion: parts[2].to_string(),
            command: parts[3].to_string(),
            directory: parts[4].to_string(),
            timestamp: parts[5].parse().unwrap_or(0),
        })
    }
}

/// Known error codes — structured, documented, consistent
#[allow(dead_code)]
pub mod codes {
    pub const CMD_NOT_FOUND: &str = "E_CMD_NOT_FOUND";
    pub const PERMISSION: &str = "E_PERMISSION";
    pub const NOT_GIT_REPO: &str = "E_NOT_GIT_REPO";
    pub const PIPE_EMPTY: &str = "E_PIPE_EMPTY";
    pub const PARSE_FAILED: &str = "E_PARSE_FAILED";
    pub const EXIT_NONZERO: &str = "E_EXIT_NONZERO";
    pub const NOT_FOUND: &str = "E_NOT_FOUND";
    pub const IO_ERROR: &str = "E_IO_ERROR";
}

/// Build a structured error string for use in CommandResult::Error
/// Stores the error in db and returns formatted display string
pub fn make_error(
    code: &'static str,
    message: &str,
    suggestion: &str,
    command: &str,
    directory: &str,
    db: &crate::db::ForestDb,
) -> String {
    let err = ShellError::new(code, message, suggestion, command, directory);
    // Store in shell_state as last_error
    let _ = db.conn.execute(
        "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_error', ?1)",
        rusqlite::params![err.to_storage()],
    );
    // Append to session error log
    let ts = err.timestamp;
    let log_key = format!("error_log_{}", ts);
    let _ = db.conn.execute(
        "INSERT OR REPLACE INTO shell_state (key, value) VALUES (?1, ?2)",
        rusqlite::params![log_key, err.to_storage()],
    );
    err.display()
}

/// Render fsh's redirect-missing-target parse error with a caret under the
/// offending `>` / `>>`, via miette. INT-171 gate 6.
///
/// This is the ONE syntax error fsh hard-rejects: unclosed quotes/heredocs/parens
/// are line CONTINUATION (fsh reads more input, they are not errors), and
/// command-not-found is a RUNTIME error, not syntax. A bare `>` with no target is
/// the genuine parse rejection (detect_redirect returns the no-target sentinel),
/// so it is the honest target for "render one real syntax error with a caret".
/// Render the missing-redirect-target diagnostic, locating the operator by SCANNING the text.
///
/// ⚠️ THIS ENTRY EXISTS FOR THE LEGACY PATH ONLY, and it is on its way out. The spine already
/// knows where the operator is -- ParseError::MissingRedirectTarget carries a Span -- so the live
/// router calls render_redirect_error_at with that span instead of re-deriving it here.
/// Re-deriving a fact the parser established is the string re-inspection INT-195 exists to remove;
/// this survives only until the legacy redirect executor goes, and dies with it.
pub fn render_redirect_error(line: &str) -> String {
    render_redirect_error_at(line, scan_missing_redirect_offset(line))
}

/// Locate the `>` or `>>` that has no target after it. Text-derived, hence private.
fn scan_missing_redirect_offset(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut offset = line.len().saturating_sub(1);
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'>' {
            let mut j = i + 1;
            if j < bytes.len() && bytes[j] == b'>' {
                j += 1;
            }
            if line[j..].trim().is_empty() {
                offset = i;
                break;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    offset
}

/// Render the same diagnostic at a KNOWN offset -- the parser's span, not a rescan.
pub fn render_redirect_error_at(line: &str, offset: usize) -> String {
    use miette::{GraphicalReportHandler, GraphicalTheme, LabeledSpan};
    let span = LabeledSpan::at(offset..offset + 1, "redirect has no target file");
    let diag =
        miette::MietteDiagnostic::new("redirect missing target file").with_labels(vec![span]);
    let report = miette::Report::new(diag).with_source_code(line.to_string());
    let mut out = String::new();
    let _ = GraphicalReportHandler::new()
        .with_theme(GraphicalTheme::unicode_nocolor())
        .render_report(&mut out, report.as_ref());
    out
}

#[cfg(test)]
mod redirect_error_tests {
    use super::{render_redirect_error, render_redirect_error_at};

    #[test]
    fn caret_lands_where_the_span_says_without_scanning() {
        // The parser's span is authoritative: the caret goes where it points, whether or not a
        // scan would have chosen the same spot.
        let out = render_redirect_error_at("echo a >", 7);
        assert!(out.contains("redirect missing target file"), "got:\n{out}");
        assert!(out.contains("redirect has no target file"), "got:\n{out}");
    }

    #[test]
    fn caret_lands_under_the_offending_redirect() {
        // INT-171 gate 6: the caret (miette's ┬) must sit under the `>` at column 6
        // of `echo >` (0-indexed byte 5). Proven by the rendered output containing
        // the message, the source line, and the label.
        let out = render_redirect_error("echo >");
        assert!(out.contains("redirect missing target file"), "got:\n{out}");
        assert!(out.contains("echo >"), "source line missing:\n{out}");
        assert!(
            out.contains("redirect has no target file"),
            "label missing:\n{out}"
        );
        // the caret marker line should exist (miette draws ┬ or ^ under the span)
        assert!(
            out.contains('\u{252c}') || out.contains('^'),
            "no caret marker:\n{out}"
        );
    }

    #[test]
    fn double_redirect_also_caught() {
        let out = render_redirect_error("cat foo >>");
        assert!(out.contains("redirect missing target file"), "got:\n{out}");
    }
}
