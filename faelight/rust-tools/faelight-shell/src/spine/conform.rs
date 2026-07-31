//! INT-200: CONFORMANCE -- what bash actually does, versus what fsh does.
//!
//! ★ THE HALF `spine migrate` CANNOT REACH. That audit compares PARSERS on the same input and never
//! runs anything, so it measures coverage. This runs both shells on the same line and compares
//! OBSERVED BEHAVIOUR, which is the only way to measure correctness.
//!
//! ★ THE METHOD IS BORROWED FROM OILS-FOR-UNIX, NOT THE CORPUS. Their spec tests run each case
//! against bash, dash, zsh and osh and record agreement -- the value is asking *what does a real
//! shell do* rather than *what should a shell do*. But importing their corpus wholesale would drag
//! in historical shell warts as requirements, and fsh has deliberately chosen to diverge from bash
//! at least twice. So the cases here are fsh's own, and bash is the reference because it is already
//! on the box.
//!
//! ⚠️⚠️ THREE VERDICTS, NOT TWO, AND THAT IS THE WHOLE POINT. A difference from bash is not
//! automatically a bug: `echo test > 0.5` creates a file in bash and prints text in fsh, because
//! the query language needs `where cpu > 0.5` to be a comparison. That divergence is DECLARED, with
//! its reason, and stays green. Only an UNDECLARED difference is a defect -- which means a NEW
//! divergence announces itself the first time it appears, instead of being discovered months later.

use std::process::Command;

/// What a case is expected to do relative to bash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    /// fsh must behave exactly as bash does.
    Match,
    /// fsh deliberately differs, and the reason is recorded with the case. Divergence here is the
    /// PASSING state; matching bash would mean the deliberate behaviour was lost.
    Diverges(&'static str),
}

/// One conformance case: a line, and what fsh owes bash on it.
pub struct Case {
    pub line: &'static str,
    pub expect: Expect,
}

/// How a case turned out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Same output and same exit status as bash, as required.
    Agrees,
    /// Differs from bash, and was declared to.
    DivergesAsDeclared,
    /// ⚠️ THE ONLY FAILURE. Either an undeclared difference, or a declared divergence that has
    /// silently started matching bash again -- both mean the recorded understanding is now wrong.
    Unexplained,
}

/// The corpus. ★ SCOPED TO WHAT THE SPINE OWNS, deliberately: redirects, fd redirects, pipelines,
/// exit status and quoting. Cases for constructs legacy still runs would measure legacy, and there
/// is already an audit for that.
///
/// ⚠️ Every case must TERMINATE ON ITS OWN. A conformance run that hangs is worse than one that
/// fails, and pipelines are exactly where a stdio mistake hangs rather than errors.
pub const CASES: &[Case] = &[
    // --- pipelines: the construct the spine took over most recently ---
    Case { line: "echo hi | grep h", expect: Expect::Match },
    Case { line: "printf 'a\\nb\\nc\\n' | wc -l", expect: Expect::Match },
    Case { line: "echo one | grep one | wc -c", expect: Expect::Match },
    // POSIX says a pipeline's status is the LAST stage's. INT-189 settled this the hard way.
    Case { line: "false | true", expect: Expect::Match },
    Case { line: "true | false", expect: Expect::Match },

    // --- redirects ---
    Case { line: "echo written > /tmp/fsh_conform_a.txt; cat /tmp/fsh_conform_a.txt", expect: Expect::Match },
    Case { line: "echo one > /tmp/fsh_conform_b.txt; echo two >> /tmp/fsh_conform_b.txt; cat /tmp/fsh_conform_b.txt", expect: Expect::Match },
    // Truncation: the second write must REPLACE, not append.
    Case { line: "echo one > /tmp/fsh_conform_c.txt; echo two > /tmp/fsh_conform_c.txt; cat /tmp/fsh_conform_c.txt", expect: Expect::Match },

    // --- file descriptors ---
    Case { line: "ls /nonexistent 2>/dev/null", expect: Expect::Match },
    Case { line: "ls /nonexistent 2>&1", expect: Expect::Match },
    Case { line: "ls /nonexistent > /tmp/fsh_conform_d.txt 2>&1; cat /tmp/fsh_conform_d.txt", expect: Expect::Match },
    // ⚠️ ADJACENCY: a SPACED numeral is an argument, not a descriptor.
    Case { line: "echo 2 > /tmp/fsh_conform_e.txt; cat /tmp/fsh_conform_e.txt", expect: Expect::Match },

    // --- quoting ---
    Case { line: "echo \"a > b\"", expect: Expect::Match },
    Case { line: "echo \"a|b\"", expect: Expect::Match },

    // --- THE DECLARED DIVERGENCES. Matching bash here would be the regression. ---
    Case {
        line: "echo test > 0.5",
        expect: Expect::Diverges(
            "bash creates a file named 0.5; fsh treats a digit-initial target as a COMPARISON so \
             `ps | where cpu > 0.5` keeps working. The query language depends on this.",
        ),
    },
    Case {
        line: "echo test >= x",
        expect: Expect::Diverges(
            "same rule, `=` instead of a digit -- `where score >= 70` must stay a comparison.",
        ),
    },
];

/// What one shell did with one line.
struct Observed {
    stdout: String,
    status: i32,
}

/// Run a line through a shell and capture what happened.
///
/// ⚠️ STDOUT AND STATUS ONLY, deliberately. stderr TEXT is not compared: fsh's error messages are
/// its own on purpose and were never meant to match bash word for word. Comparing them would report
/// every case as a divergence and drown the signal this exists to find.
fn observe(shell: &str, args: &[&str], line: &str) -> Option<Observed> {
    let out = Command::new(shell).args(args).arg(line).output().ok()?;
    Some(Observed {
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        status: out.status.code().unwrap_or(-1),
    })
}

/// Run every case against bash and fsh, and render the report.
pub fn run(fsh_bin: &str) -> String {
    let mut out = String::from("Conformance (fsh vs bash)\n\n");
    let (mut agree, mut declared, mut unexplained) = (0usize, 0usize, 0usize);
    let mut detail = String::new();

    for case in CASES {
        let Some(b) = observe("bash", &["-c"], case.line) else {
            return String::from("conform: bash not available -- nothing to compare against\n");
        };
        let Some(f) = observe(fsh_bin, &["-c"], case.line) else {
            return format!("conform: could not run {fsh_bin}\n");
        };
        let same = b.stdout == f.stdout && b.status == f.status;
        let verdict = match (case.expect, same) {
            (Expect::Match, true) => Verdict::Agrees,
            (Expect::Diverges(_), false) => Verdict::DivergesAsDeclared,
            // ⚠️ BOTH REMAINING COMBINATIONS ARE FAILURES, and the second is the subtle one: a
            // declared divergence that started matching bash again means the deliberate behaviour
            // was silently lost. That is how the digit guard would disappear unnoticed.
            _ => Verdict::Unexplained,
        };
        match verdict {
            Verdict::Agrees => agree += 1,
            Verdict::DivergesAsDeclared => declared += 1,
            Verdict::Unexplained => {
                unexplained += 1;
                detail.push_str(&format!(
                    "  UNEXPLAINED  {}\n    bash: {:?} (exit {})\n    fsh:  {:?} (exit {})\n",
                    case.line, b.stdout, b.status, f.stdout, f.status
                ));
                if let Expect::Diverges(why) = case.expect {
                    detail.push_str(&format!(
                        "    ⚠️ declared to DIVERGE but matched bash -- the deliberate behaviour is \
                         gone: {why}\n"
                    ));
                }
            }
        }
    }

    out.push_str(&format!("Cases:             {}\n", CASES.len()));
    out.push_str(&format!("Agrees with bash:  {agree}\n"));
    out.push_str(&format!("Declared divergence: {declared}\n"));
    out.push_str(&format!("UNEXPLAINED:       {unexplained}\n\n"));
    if unexplained > 0 {
        out.push_str(&detail);
        out.push_str(
            "\nAn unexplained result is either a defect or a divergence nobody wrote down.\n",
        );
    } else {
        out.push_str(
            "Every case is understood: it matches bash, or it differs for a recorded reason.\n",
        );
    }
    out
}
