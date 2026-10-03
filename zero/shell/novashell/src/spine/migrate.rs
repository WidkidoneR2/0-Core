//! INT-169 spine: the legacy -> ExecutionPlan bridge. See docs/rfc-169-parser-spine.md.
//!
//! TEMPORARY BY DESIGN. This adapter makes the EXISTING execution path (ExecContext, produced
//! by exec.rs::from_line) also speak ExecutionPlan, so both models can be compared on the same
//! contract (Increment 10). It is a SNAPSHOT of legacy behavior, quirks included -- it must
//! mirror what the shell executes TODAY, not what it should. When the spine takes over
//! execution and the legacy path is deleted (end of migration), this module goes with it.
//!
//! Legacy's exact behavior, captured faithfully (verified from exec.rs::from_line +
//! commands::tokenize): the command word (first token) is LOWERCASED; the arguments are
//! quote-aware tokenized (quotes stripped, quoted spaces kept as one token) and case-PRESERVED.
//! So `GitHub commit -m "a b"` -> cmd "github", args ["commit","-m","a b"]. Both of those are
//! divergences from the spine (case-fold at argv[0]; quote handling) that the comparison layer
//! classifies -- NOT bugs in this adapter. The adapter's job is fidelity to legacy, nothing more.

use std::ffi::OsString;

use super::plan::{Environment, ExecutionPlan, IoPlan};
use crate::exec::ExecContext;

/// Build an ExecutionPlan from a legacy ExecContext -- a faithful snapshot of what the current
/// shell would execute. argv = [lowercased cmd] ++ tokenized args. cwd is the concrete dir
/// legacy captured; env is Inherit. The CALLER supplies `io`, because a redirect is stripped
/// from the line BEFORE tokenizing in the live shell -- so an audit that leaves it in argv is
/// comparing against a legacy the shell does not have. INT-200: that mismatch turned every
/// redirect the spine learned into a false 'unexpected', 2 -> 205 in one run.
pub fn plan_from_legacy(ctx: &ExecContext, io: IoPlan) -> ExecutionPlan {
    let mut argv: Vec<OsString> = Vec::with_capacity(1 + ctx.args.len());
    if !ctx.cmd.is_empty() {
        argv.push(OsString::from(&ctx.cmd));
    }
    argv.extend(ctx.args.iter().map(OsString::from));

    ExecutionPlan {
        argv,
        // Legacy always captures a concrete cwd. The comparison layer treats this as
        // semantically equivalent to the spine's None ("inherit current dir") when both are
        // evaluated in the same process context.
        cwd: Some(ctx.cwd.clone()),
        env: Environment::Inherit,
        io,
    }
}

/// The line legacy ACTUALLY tokenizes, with any command-prefix assignments removed.
///
/// LIVE LEGACY NEVER SEES THE PREFIX. `try_inline_assignment` (engine.rs:1071) strips `FOO=bar`,
/// sets the variable, and calls `execute_with_context` with the REST -- so an audit that builds
/// ExecContext from the whole line models `commands::tokenize` rather than the shell that runs.
/// Once the spine learned to lower these, every assignment row read as a divergence: 105 of them,
/// all of the audit disagreeing with itself.
///
/// THE PARSER OWNS THE POSITIONAL RULE AND THIS CONSUMES IT. Rediscovering where a prefix ends
/// would be a second implementation of the thing that just landed -- `A=1 B=2 echo hi` drops two,
/// `echo B=2 hi` drops none, and only the parser knows that without re-scanning.
///
/// The span is a byte offset into the SOURCE, and a prefix sits at the start, so it holds for the
/// redirect-stripped line too. Guarded anyway: a short line is returned untouched rather than
/// panicking on a slice.
pub fn legacy_line_without_prefix(
    node: &crate::spine::ast::Spanned<crate::spine::ast::AstNode>,
    line: &str,
) -> String {
    let crate::spine::ast::AstNode::Command(cmd) = &node.node else {
        return line.to_string();
    };
    let Some(last) = cmd.assignments.last() else {
        return line.to_string();
    };
    let end = last.span.end;
    if line.len() < end || !line.is_char_boundary(end) {
        return line.to_string();
    }
    line[end..].trim_start().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// These tests fix argv shape, not IO. INT-200 gave the adapter an `io` parameter because the
    /// AUDIT must model the redirect legacy already stripped; nothing here exercises that, so a
    /// one-line wrapper keeps the cases reading about the thing they actually assert.
    fn plan_from_legacy_simple(ctx: &ExecContext) -> ExecutionPlan {
        plan_from_legacy(ctx, IoPlan::Simple)
    }

    // ⚠️ `raw` and `expanded` are intentionally EMPTY, and that is different from the stale
    // fixtures INT-169 blocker 2 had to repair. Those claimed an execution had arguments in its
    // display text and none in its argument vector -- contradictory, a state no path can produce.
    // This one is merely INCOMPLETE: the adapter reads argv-shaped fields only, so supplying
    // execution text would invent a second execution identity for no consumer. Do not "fix" it by
    // filling those in; cmd and args ARE the identity here.
    //
    // Hand-build an ExecContext with just the fields the adapter reads (cmd, args, cwd). The
    // adapter never touches intent (the only db-dependent field), so no db is needed here. That
    // from_line ACTUALLY produces these values (lowercased cmd, quote-stripped args) is proven
    // end-to-end by the migration audit over real history, not by this unit test.
    fn ctx(cmd: &str, args: &[&str]) -> ExecContext {
        ExecContext {
            raw: String::new(),
            expanded: String::new(),
            cmd: cmd.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            cwd: PathBuf::from("/home/christian"),
            intent: None,
            timestamp: std::time::SystemTime::UNIX_EPOCH,
            execution_id: crate::exec::next_execution_id(),
            session_id: "test",
            in_pipeline: false,
        }
    }

    #[test]
    fn bare_command_maps_cmd_and_args_to_argv() {
        let plan = plan_from_legacy_simple(&ctx("git", &["add", "-A"]));
        assert_eq!(
            plan.argv,
            vec![
                OsString::from("git"),
                OsString::from("add"),
                OsString::from("-A")
            ]
        );
        assert!(plan.cwd.is_some(), "legacy carries a concrete cwd");
        assert_eq!(plan.env, Environment::Inherit);
        assert_eq!(plan.io, IoPlan::Simple);
    }

    #[test]
    fn adapter_preserves_whatever_case_the_context_holds() {
        // The adapter is faithful: it does NOT lowercase. Legacy's from_line is what lowercases
        // the cmd word -- so a context whose cmd is already "github" yields argv[0] "github",
        // and args keep their case. The adapter mirrors the context exactly.
        let plan = plan_from_legacy_simple(&ctx("github", &["Clone"]));
        assert_eq!(plan.argv[0], OsString::from("github"));
        assert_eq!(plan.argv[1], OsString::from("Clone"));
    }

    #[test]
    fn empty_cmd_yields_only_args() {
        // Defensive: an empty cmd word (blank line through from_line) produces no argv[0].
        let plan = plan_from_legacy_simple(&ctx("", &[]));
        assert!(plan.argv.is_empty());
    }

    #[test]
    fn quoted_arg_already_joined_is_one_argv_element() {
        // tokenize (in from_line) strips quotes and joins quoted spaces into ONE arg. The
        // adapter receives that already-joined arg and maps it 1:1 -- "message here" stays one.
        let plan = plan_from_legacy_simple(&ctx("git", &["commit", "-m", "message here"]));
        assert_eq!(plan.argv[3], OsString::from("message here"));
        assert_eq!(plan.argv.len(), 4);
    }
}
