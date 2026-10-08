//! nsh-test -- permanent regression suite for NovaShell
//! INT-202 (Fsh-Test 2.0). Phase 1 ported fsh_audit.sh's 75 tests to Rust.
//!
//! ⚠️ The citation here read INT-304 until 2026-09-18. `ints 304` reports NOT FOUND:
//! it is one of the phantom citations the intent-citations audit records. INT-202 is
//! real, complete 2026-08-05, and is the intent this suite actually belongs to.

mod repl;

use colored::*;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum Category {
    Repl,
    Tilde,
    Pipes,
    Vocabulary,
    Heredoc,
    Signals,
    Regression,
    Performance,
    Hostile,
    Leak,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Category::Repl => write!(f, "repl"),
            Category::Tilde => write!(f, "tilde"),
            Category::Pipes => write!(f, "pipes"),
            Category::Vocabulary => write!(f, "vocabulary"),
            Category::Heredoc => write!(f, "heredoc"),
            Category::Signals => write!(f, "signals"),
            Category::Regression => write!(f, "regression"),
            Category::Performance => write!(f, "performance"),
            Category::Hostile => write!(f, "hostile"),
            Category::Leak => write!(f, "leak"),
        }
    }
}

/// Whether a case ran, and if it did not, why not.
///
/// THREE STATES, NOT TWO. `passed: bool` could not say "this case never ran", so a case that
/// needs a 0-Core checkout reported FAILED on a machine without one -- indistinguishable from
/// the shell being broken. Measured 2026-09-06 by running this suite inside the devbox sandbox:
/// 22 of 194 went red, and NONE was a shell defect. Nineteen were probes that use the shell as
/// an instrument to check that a directory exists.
///
/// The same collapse INT-192 removed from the doctor, in the harness that tests the shell the
/// doctor reports on: could-not-run and did-not-pass are different answers.
#[derive(Debug, PartialEq)]
enum Outcome {
    Passed,
    Failed,
    /// The case did not run because a precondition it does not control was absent.
    Skipped(&'static str),
}

#[derive(Debug)]
struct TestResult {
    name: String,
    category: Category,
    outcome: Outcome,
    duration_ms: u64,
    error: Option<String>,
}

impl TestResult {
    fn passed(&self) -> bool {
        self.outcome == Outcome::Passed
    }
    fn skipped(&self) -> bool {
        matches!(self.outcome, Outcome::Skipped(_))
    }
}

/// INT-195 gate 6: invoke zero-deadwood through the same seam pattern run_fsh uses for the
/// shell. DEADWOOD_BIN lets one test prove the debug build before a deploy and the deployed
/// artifact after -- the two-binaries discipline the rest of this work relies on.
fn run_deadwood(args: &[&str]) -> Result<std::process::Output, String> {
    let bin = std::env::var("DEADWOOD_BIN").unwrap_or_else(|_| "zero-deadwood".to_string());
    Command::new(&bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("cannot invoke {bin}: {e}"))
}

/// ⚠️ BUILD BEFORE YOU RUN. `cargo test` compiles the TEST profile; it does not rebuild the binary
/// this harness executes. Running nsh-test straight after `cargo test` measures the PREVIOUS build,
/// which is INT-110's stale-binary lesson and it has bitten again since: a green 143/143 was read
/// from a shell that did not contain the change being tested.
///
///     cargo build -p novashell && NSH_BIN=target/debug/nsh ./target/debug/nsh-test
///
/// The pre-push hook builds first, which is why this only bites in manual runs.
fn run_fsh(input: &str) -> Result<String, String> {
    run_fsh_env(input, &[])
}

/// INT-221: the same launcher, with the environment under the case's control.
///
/// ONE LAUNCH PATH, not two. A case that needs to vary PATH, NSH_TRACE or NSH_SPINE used to have
/// no way to do it, and the alternative -- a second Command::new(NSH_BIN) inside the test -- would
/// be a fourth way of starting the shell in a suite whose founding finding was that two doors
/// disagree. So the capability belongs here.
///
/// `extra_env` is applied AFTER the harness defaults, so a case can deliberately override them.
fn run_fsh_env(input: &str, extra_env: &[(&str, &str)]) -> Result<String, String> {
    let mut cmd = fsh_command(extra_env);
    let out = cmd
        .arg("-c")
        .arg(input)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// INT-243: the one place a -c launch is configured, shared by every runner that uses it, so a
/// new runner cannot drift from run_fsh_env in which binary, database or cwd it uses.
fn fsh_command(extra_env: &[(&str, &str)]) -> Command {
    let mut cmd = Command::new(repl::fsh_bin());
    // INT-206: the same setting the REPL runner uses, so the suite drives ONE shell
    // configuration rather than two that differ in where they think they are.
    cmd.env("NSH_KEEP_CWD", "1")
        // INT-204: and its own database, for the same reason -- two doors that disagree about which
        // state they read is the shape of problem this suite keeps finding in the shell it tests.
        .env("ZERO_STATE_DB", repl::case_db_path());
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd
}

/// INT-243: run `nsh -c` with stdout ALREADY CLOSED -- the read end is dropped before the child
/// starts, so every write meets EPIPE. `cmd | head` races: output that fits the pipe buffer never
/// meets a closed pipe. This does not race. Returns (stderr, exit code).
fn run_fsh_closed_stdout(input: &str) -> Result<(String, Option<i32>), String> {
    let (reader, writer) = std::io::pipe().map_err(|e| e.to_string())?;
    drop(reader);
    let out = fsh_command(&[])
        .arg("-c")
        .arg(input)
        .stdout(writer)
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| e.to_string())?;
    Ok((
        String::from_utf8_lossy(&out.stderr).trim().to_string(),
        out.status.code(),
    ))
}

fn test(name: &str, category: Category, f: impl Fn() -> Result<(), String>) -> TestResult {
    let start = Instant::now();
    let result = f();
    let duration_ms = start.elapsed().as_millis() as u64;
    TestResult {
        name: name.to_string(),
        category,
        outcome: if result.is_ok() {
            Outcome::Passed
        } else {
            Outcome::Failed
        },
        duration_ms,
        error: result.err(),
    }
}

/// Is there a 0-Core checkout at $HOME/0-core?
///
/// ASKED OF $HOME, NOT HARDCODED. The class of case this guards exists because paths were
/// written as `~/0-core/...` and the suite assumed one machine. Answering the question the same
/// wrong way would be a joke at this file's expense.
/// The directory this run's FIXTURE lives in, named from the pid so two runs never collide.
///
/// Same discipline as repl::case_db_dir: the run owns one directory, creates it once, and removes
/// it at the end on the path that always executes.
fn fixture_dir() -> String {
    format!("/tmp/nsh-fixture-{}", std::process::id())
}

/// Build a MINIMAL tree that the checkout-probing cases can be pointed at, and hand back a path
/// to use as HOME.
///
/// WHY THIS EXISTS. Nineteen cases assert things like `ls ~/0-core/docs` contains PHILOSOPHY.
/// Measured 2026-09-06 by running this suite inside the devbox sandbox: they are real coverage of
/// tilde expansion through pipes, subshells and command substitution, but they use the AUTHOR'S
/// CHECKOUT as their subject, so they can only run on one machine. A fixture lets them run
/// everywhere without weakening what they assert.
///
/// ⚠️ THIS IS NOT A CHECKOUT AND MUST NEVER LOOK LIKE ONE. It lives under a REDIRECTED HOME that
/// only the converted cases pass, so repo_present() -- which reads the real $HOME -- is
/// unaffected. Two cases genuinely need 0-Core rather than a directory shaped like one
/// (repl_206 asserts nsh starts in the repo home; pick_without_fzf cannot reach its dependency
/// check because INT-230 refuses first), and they must keep skipping. A fixture that satisfied
/// repo_present would make both of them pass against three stub files and mean nothing.
///
/// ⚠️ EVERY FILE HERE EXISTS TO SATISFY A NAMED ASSERTION. The contents are the exact strings the
/// cases look for and nothing else, so a reader can see at a glance that this is scaffolding.
/// If a case needs something not in this list, add it HERE rather than widening the case.
fn fixture_home() -> Result<String, String> {
    let root = fixture_dir();
    let core = format!("{}/0-core", root);

    // `ls ~/0-core/docs` contains PHILOSOPHY
    mkdir(&format!("{}/docs", core))?;
    write(&format!("{}/docs/PHILOSOPHY.md", core), "fixture")?;

    // `ls ~/0-core/zero/packages/zero/scripts` contains deploy
    mkdir(&format!("{}/zero/packages/zero/scripts", core))?;
    write(
        &format!("{}/zero/packages/zero/scripts/deploy.sh", core),
        "fixture",
    )?;

    // `ls ~/0-core/zero/intents` contains future, and future contains a .md
    mkdir(&format!("{}/zero/intents/future", core))?;
    write(
        &format!("{}/zero/intents/future/placeholder.md", core),
        "fixture",
    )?;

    // `ls ~/0-core/zero/shell` holds novashell and nsh-test, and zero/tools holds zero-core, as the
    // real tree does. tilde_nested_pipe greps for novashell alone, so nsh-test is what shows the
    // filter ran: without it, grep keeping everything and grep keeping one look the same.
    mkdir(&format!("{}/zero/tools/zero-core", core))?;
    mkdir(&format!("{}/zero/shell/novashell/src", core))?;
    mkdir(&format!("{}/zero/shell/nsh-test", core))?;

    // `cat ~/0-core/zero/shell/novashell/Cargo.toml` contains novashell, and piped
    // through `grep name` contains name.
    write(
        &format!("{}/zero/shell/novashell/Cargo.toml", core),
        "[package]\nname = \"novashell\"\n",
    )?;

    // `grep -r expand_braces .../novashell/src/` and the same against src/main.rs
    write(
        &format!("{}/zero/shell/novashell/src/main.rs", core),
        "fn expand_braces() {}\n",
    )?;

    // `ls ~/.local/state/zero/state.db` contains state.db
    mkdir(&format!("{}/.local/state/zero", root))?;
    write(&format!("{}/.local/state/zero/state.db", root), "fixture")?;

    Ok(root)
}

fn mkdir(p: &str) -> Result<(), String> {
    std::fs::create_dir_all(p).map_err(|e| format!("fixture mkdir {}: {}", p, e))
}

fn write(p: &str, body: &str) -> Result<(), String> {
    std::fs::write(p, body).map_err(|e| format!("fixture write {}: {}", p, e))
}

fn repo_present() -> bool {
    std::path::Path::new(&home()).join("0-core/zero").is_dir()
}

/// Register a case that PROBES THE CHECKOUT rather than the shell.
///
/// These use the shell as an instrument to assert a directory exists -- `ls ~/0-core/docs` tells
/// you about the tree, not about tilde expansion, because a shell that expanded `~` perfectly
/// would still print nothing if the directory were absent. On the author's laptop the two
/// questions have the same answer; nowhere else do they.
///
/// NOT DELETED, DELIBERATELY. On a machine WITH a checkout they are real coverage of tilde
/// expansion through pipes, subshells and command substitution. The defect was never the
/// assertion; it was that the case could not say which question it was answering.
///
/// PHASE 4 (2026-09-10): `why` IS PER CASE, NOT SHARED. Nineteen callers were converted to run
/// against a fixture and no longer need this at all. The two that remain skip for DIFFERENT
/// reasons, and a single message could only ever describe one of them -- which is the same
/// collapse in miniature that Outcome::Skipped(&str) exists to prevent. If the reason is worth
/// carrying, it is worth carrying accurately.
fn repo_test(
    name: &str,
    category: Category,
    why: &'static str,
    f: impl Fn() -> Result<(), String>,
) -> TestResult {
    if !repo_present() {
        return TestResult {
            name: name.to_string(),
            category,
            outcome: Outcome::Skipped(why),
            duration_ms: 0,
            error: None,
        };
    }
    test(name, category, f)
}

/// INT-202 coverage class 1: `-c` with the STATUS kept, instead of collapsed into an error.
///
/// run_fsh answers `Err(stderr)` for any non-zero exit, which makes a status inexpressible: a case
/// cannot say "expect 143" because failure and non-zero look identical to it. That gap is why the
/// three exit-code defects fixed in the `-c` handler have no regression test -- only a probe in a
/// commit message.
///
/// ⚠️ THIS IS A SIBLING, NOT A REPLACEMENT. Over a hundred cases depend on run_fsh answering the way
/// it does, and rewriting it would mean re-verifying all of them for a change meant to be additive.
///
/// ⚠️ THE CODE IS AN Option, AND THAT MATTERS. run_repl_lines_status already establishes the rule --
/// unknown must stay unknown. On Unix `code()` is None when a process is killed by a signal, and
/// manufacturing a number there would make a comparison look performed when it was not. A caller
/// that wants the signal convention asks for it; this reports what it saw.
fn run_fsh_status(input: &str) -> Result<(String, String, Option<i32>), String> {
    let fsh = std::env::var("NSH_BIN").unwrap_or_else(|_| "nsh".to_string());
    let out = Command::new(&fsh)
        .env("NSH_KEEP_CWD", "1")
        .env("ZERO_STATE_DB", repl::case_db_path())
        .arg("-c")
        .arg(input)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| e.to_string())?;
    Ok((
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
        String::from_utf8_lossy(&out.stderr).trim().to_string(),
        out.status.code(),
    ))
}

/// Assert an exit code, refusing to treat an unknown one as a match.
fn expect_exit(got: Option<i32>, expected: i32) -> Result<(), String> {
    match got {
        Some(c) if c == expected => Ok(()),
        Some(c) => Err(format!("expected exit {} got {}", expected, c)),
        None => Err(format!(
            "expected exit {} but the status was unknown -- the process was signalled",
            expected
        )),
    }
}

fn expect_eq(got: &str, expected: &str) -> Result<(), String> {
    if got == expected {
        Ok(())
    } else {
        Err(format!("expected {:?} got {:?}", expected, got))
    }
}

/// The home directory of whoever is RUNNING the suite.
///
/// ⚠️ EARNED ON VOID 2026-08-23. Thirty-three cases failed on a second machine, and reading two of
/// them settled it: `pwd_returns_path -- expected "/build/0-core" to contain "/home/christian"`
/// and `whoami -- expected "christian" got "anon"`. The tests asserted the AUTHOR'S username and
/// home directory, so they fail for any other user on any machine -- including a second account on
/// the author's own laptop. INT-227 Category A: wrong everywhere, not merely on another OS.
///
/// ⭐ THE PATTERN ALREADY EXISTED AND WAS SIMPLY NOT USED EVERYWHERE. `tilde_basic` and
/// `tilde_in_path` ask $HOME and compare -- and those two PASSED on Void while their hardcoded
/// siblings failed. This makes the working shape the easy one to reach for.
fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

/// Who is RUNNING the suite.
///
/// ⚠️ NOT $USER, AND THE DISTINCTION MATTERS HERE. The test asserts what the shell's `whoami`
/// prints, and `whoami` reports the EFFECTIVE UID's name from the password database -- which is
/// not always what $USER holds. Under `sudo`, $USER is often the original user while `whoami`
/// says root; in a container $USER may be unset entirely.
///
/// ⭐ SO THE TEST ASKS THE SAME SOURCE THE SHELL WILL: run `whoami` and compare. That makes the
/// assertion "fsh agrees with the system" rather than "fsh agrees with a name we guessed", which
/// is the property actually worth testing.
fn whoami() -> String {
    std::process::Command::new("whoami")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Where the 0-Core checkout lives for whoever is running the suite.
fn core_root() -> String {
    format!("{}/0-core", home())
}

fn expect_contains(got: &str, needle: &str) -> Result<(), String> {
    if got.contains(needle) {
        Ok(())
    } else {
        Err(format!("expected {:?} to contain {:?}", got, needle))
    }
}

fn all_tests() -> Vec<TestResult> {
    let mut results = vec![];

    // --- TILDE EXPANSION ---
    results.push(test("tilde_basic", Category::Tilde, || {
        let out = run_fsh("echo ~")?;
        let home = std::env::var("HOME").unwrap_or_default();
        expect_eq(&out, &home)
    }));
    results.push(test("tilde_in_path", Category::Tilde, || {
        let out = run_fsh("echo ~/0-core")?;
        let home = std::env::var("HOME").unwrap_or_default();
        expect_eq(&out, &format!("{}/0-core", home))
    }));
    results.push(test("tilde_in_var_assign", Category::Tilde, || {
        let out = run_fsh("x=~/test && echo $x")?;
        let home = std::env::var("HOME").unwrap_or_default();
        expect_eq(&out, &format!("{}/test", home))
    }));

    // --- PIPES ---
    results.push(test("pipe_basic", Category::Pipes, || {
        let out = run_fsh("echo hello | tr a-z A-Z")?;
        expect_eq(&out, "HELLO")
    }));
    results.push(test("pipe_chain", Category::Pipes, || {
        let out = run_fsh("echo hello world | tr a-z A-Z | tr -d ' '")?;
        expect_eq(&out, "HELLOWORLD")
    }));
    results.push(test("pipe_with_grep", Category::Pipes, || {
        let out = run_fsh("printf 'a\\nb\\nc\\n' | grep b")?;
        expect_eq(&out, "b")
    }));

    // --- INT-109: pipeline on the left of && / || ---
    results.push(test("pipe_left_of_and", Category::Pipes, || {
        let out = run_fsh("echo hi | tr a-z A-Z && echo done")?;
        expect_eq(&out, "HI\ndone")
    }));

    // --- VOCABULARY ---
    results.push(test("vocab_list_home", Category::Vocabulary, || {
        // list is fsh vocabulary -- test via ls which it maps to
        let out = run_fsh("ls ~")?;
        if out.is_empty() {
            Err("ls produced no output".to_string())
        } else {
            Ok(())
        }
    }));
    results.push(test(
        "external_fd_is_launched",
        Category::Vocabulary,
        || {
            // ⚠️ RENAMED, BECAUSE THE OLD NAME WAS NOT WHAT THIS TESTS. It said vocab_find_basic and
            // commented "find vocabulary uses fd syntax", but its body invoked `fd` and relied on `-c`
            // delegating to sh so the EXTERNAL fd binary ran. INT-201 gate 4 routed `-c` through fsh,
            // where `fd` resolves to fsh's own find vocabulary -- which is correct, and is what typing
            // `fd` at the prompt has always done. The change did not break this test; it revealed that
            // the test depended on the old execution architecture without saying so.
            //
            // The contract it actually checks is: CAN fsh LAUNCH AN EXTERNAL EXECUTABLE. So it now says
            // so, by resolving the binary rather than naming a store path that changes on every rebuild.
            let fd = String::from_utf8_lossy(
                &Command::new("sh")
                    .args(["-c", "command -v fd"])
                    .output()
                    .map_err(|e| e.to_string())?
                    .stdout,
            )
            .trim()
            .to_string();
            if fd.is_empty() {
                return Err("fd is not on PATH -- this case needs it".to_string());
            }
            // ⚠️ THE TARGET MUST EXIST ON ANY MACHINE. This searched "zero/engine" for
            // Cargo.toml -- a RELATIVE path into the author's checkout, so it resolved against
            // whatever cwd the harness happened to have. It passed for a year because that cwd was
            // always ~/0-core, and went red the moment DevBox set a working directory (2026-09-06).
            // The contract this case states is CAN nsh LAUNCH AN EXTERNAL EXECUTABLE; the directory
            // is scenery, so it is now scenery that exists everywhere.
            let probe = std::env::temp_dir().join(format!("nsh-fd-probe-{}", std::process::id()));
            std::fs::create_dir_all(&probe).map_err(|e| e.to_string())?;
            std::fs::write(probe.join("Cargo.toml"), "[package]").map_err(|e| e.to_string())?;
            let out = run_fsh(&format!("{} Cargo.toml {}", fd, probe.display()))?;
            let _ = std::fs::remove_dir_all(&probe);
            expect_contains(&out, "Cargo.toml")
        },
    ));

    // --- HEREDOC ---
    results.push(test("heredoc_basic", Category::Heredoc, || {
        let out = run_fsh("cat << 'EOF'\nhello\nEOF")?;
        expect_eq(&out, "hello")
    }));
    results.push(test("repl_strips_trailing_comment", Category::Repl, || {
        // INT-209: THE OTHER HALF OF THE DIVERGENCE, and it must be guarded through the REPL
        // door because that is where strip_comments is called from -- repl_main, its only
        // caller. A `-c` case cannot reach it, so the state INT-209 moves would otherwise have
        // nothing protecting it on the side that actually runs it.
        //
        // Paired with comment_handling_differs_by_door, which asserts the `-c` side keeps the
        // comment. When INT-209 moves comment recognition into the canonical scanner both doors
        // agree, and BOTH cases change together -- deliberately, not one quietly following the
        // other.
        let out = crate::repl::run_repl("echo ZZA hi # ZZB tail")?;
        let joined = out.join("\n");
        if joined.contains("ZZB") {
            return Err(format!("REPL did not strip the comment: {joined:?}"));
        }
        expect_contains(&joined, "ZZA hi")
    }));
    results.push(test(
        "comment_handling_agrees_across_doors",
        Category::Regression,
        || {
            // INT-209: BOTH DOORS NOW AGREE, and this case is the record of them not agreeing.
            //
            // It was written asserting the DIVERGENCE: strip_comments was called from repl_main and
            // nowhere else, so the REPL stripped a trailing comment and `-c` did not -- the founding
            // two-doors finding in a construct nobody would expect to diverge. Measured through both
            // doors rather than reasoned about.
            //
            // The scanner now recognises a comment as a lexical state, so `-c` strips it too. The
            // assertion is INVERTED deliberately, with the reason stated, rather than the case being
            // quietly deleted once it went red -- which is what its original comment promised.
            //
            // Paired with repl_strips_trailing_comment: both doors, one rule, and neither can drift
            // without a test going red.
            let out = run_fsh("echo ZZA hi # ZZB tail")?;
            if out.contains("ZZB") {
                return Err(format!("-c did not strip the comment: {out:?}"));
            }
            expect_contains(&out, "ZZA hi")
        },
    ));
    results.push(test(
        "heredoc_body_keeps_hash_lines",
        Category::Heredoc,
        || {
            // INT-209 RED-FIRST: a heredoc body is DATA, so a line beginning with # is content and
            // must survive verbatim. arch-era INT-285 established this after comment stripping ate
            // heredoc bodies; expand::strip_comments encodes it today by tracking in_heredoc, and
            // INT-209 moves that state into the canonical scanner. This case exists so the move has
            // something that can FAIL -- the three heredoc cases already here never put a # in a
            // body, so nothing currently guards the behaviour being relocated.
            let out = run_fsh("cat <<EOF\n# not a comment\nplain\nEOF")?;
            expect_contains(&out, "# not a comment")
        },
    ));
    results.push(test(
        "heredoc_body_keeps_apostrophe",
        Category::Heredoc,
        || {
            // The same rule, second shape: an apostrophe in a body is data, not an opening quote.
            // This is the failure that hung the prompt twice -- an English possessive inside a
            // pasted block -- and it must stay fixed when comment and heredoc state move.
            let out = run_fsh("cat <<EOF\nthe parser's job\nEOF")?;
            expect_contains(&out, "parser's job")
        },
    ));
    results.push(test("heredoc_multiline", Category::Heredoc, || {
        let out = run_fsh("cat << 'EOF'\nline1\nline2\nEOF")?;
        expect_eq(&out, "line1\nline2")
    }));

    // --- BASIC ECHO/PWD/SYSTEM ---
    results.push(test("echo_simple", Category::Regression, || {
        expect_eq(&run_fsh("echo hello world")?, "hello world")
    }));
    results.push(test("echo_number", Category::Regression, || {
        expect_eq(&run_fsh("echo 42")?, "42")
    }));
    results.push(test("echo_quoted", Category::Regression, || {
        expect_eq(&run_fsh("echo 'zero grows'")?, "zero grows")
    }));
    results.push(test("pwd_returns_path", Category::Regression, || {
        expect_contains(&run_fsh("pwd")?, &home())
    }));
    // PHASE 1 of the fixture work (2026-09-10). The fixture is built and PROVEN here; nothing
    // consumes it yet. Converting the nineteen checkout probes to use it is phase 2 onward, one
    // small batch at a time, so a mistake has a blast radius of one batch.
    //
    // This case runs EVERYWHERE, including inside devbox, because it checks the harness's own
    // scaffolding rather than the author's checkout. If it ever skips, something is wrong with
    // the fixture rather than with the machine.
    results.push(test(
        "fixture_tree_is_complete",
        Category::Regression,
        || {
            let root = fixture_home()?;
            let core = format!("{}/0-core", root);
            let needed = [
                format!("{}/docs/PHILOSOPHY.md", core),
                format!("{}/zero/packages/zero/scripts/deploy.sh", core),
                format!("{}/zero/intents/future/placeholder.md", core),
                format!("{}/zero/tools/zero-core", core),
                format!("{}/zero/shell/novashell/Cargo.toml", core),
                format!("{}/zero/shell/novashell/src/main.rs", core),
                format!("{}/.local/state/zero/state.db", root),
            ];
            for p in &needed {
                if !std::path::Path::new(p).exists() {
                    return Err(format!("fixture is missing {}", p));
                }
            }
            // The two content assertions the cases rely on, checked here so a later batch cannot
            // fail for a reason that has nothing to do with the shell.
            let cargo =
                std::fs::read_to_string(format!("{}/zero/shell/novashell/Cargo.toml", core))
                    .map_err(|e| e.to_string())?;
            if !cargo.contains("novashell") || !cargo.contains("name") {
                return Err("fixture Cargo.toml lost its novashell/name strings".to_string());
            }
            let main_rs =
                std::fs::read_to_string(format!("{}/zero/shell/novashell/src/main.rs", core))
                    .map_err(|e| e.to_string())?;
            if !main_rs.contains("expand_braces") {
                return Err("fixture main.rs lost expand_braces".to_string());
            }
            Ok(())
        },
    ));
    results.push(test("uname_linux", Category::Regression, || {
        expect_contains(&run_fsh("uname")?, "Linux")
    }));
    results.push(test("whoami", Category::Regression, || {
        expect_eq(&run_fsh("whoami")?, &whoami())
    }));
    results.push(test("which_bash", Category::Regression, || {
        expect_contains(&run_fsh("which bash")?, "bash")
    }));

    // --- VARIABLES ---
    results.push(test("assign_and_echo", Category::Regression, || {
        expect_eq(&run_fsh("X=hello; echo $X")?, "hello")
    }));
    results.push(test("assign_with_spaces", Category::Regression, || {
        expect_eq(&run_fsh("MSG=world; echo $MSG")?, "world")
    }));
    results.push(test("home_variable", Category::Regression, || {
        expect_contains(&run_fsh("echo $HOME")?, &home())
    }));
    results.push(test("assign_number", Category::Regression, || {
        expect_eq(&run_fsh("N=42; echo $N")?, "42")
    }));
    results.push(test("path_not_empty", Category::Regression, || {
        expect_contains(&run_fsh("echo $PATH")?, "/")
    }));

    // --- SEMICOLON / OPERATORS ---
    results.push(test("semicolon_two_cmds", Category::Regression, || {
        expect_contains(&run_fsh("echo first; echo second")?, "second")
    }));
    results.push(test("and_operator", Category::Regression, || {
        expect_contains(&run_fsh("echo a && echo b")?, "b")
    }));
    results.push(test("and_chain_nsh_builtin", Category::Regression, || {
        // The chain must run BOTH sides, in order. This expected "3.0.0" -- core's version when it
        // was written -- and kept passing after core moved on only because core version printed an
        // invented "13.0.0" that happened to contain it. INT-247 removed the invention and
        // this went red. Assert the SHAPE of the output, never a version number.
        let out = run_fsh("echo ok && core version")?;
        let mut lines = out.lines();
        if lines.next().map(str::trim) != Some("ok") {
            return Err(format!("the left side did not run first: {:?}", out));
        }
        if !lines.any(|l| l.starts_with("core ")) {
            return Err(format!("the right side did not run: {:?}", out));
        }
        Ok(())
    }));
    results.push(test("subshell_expansion", Category::Regression, || {
        expect_eq(&run_fsh("echo $(echo nested)")?, "nested")
    }));

    // --- TILDE ---
    results.push(test("tilde_echo_subpath", Category::Tilde, || {
        expect_contains(&run_fsh("echo ~/0-core")?, &core_root())
    }));
    // PHASE 3 batch 1 (2026-09-10). Same conversion as tilde_ls_docs: the command string is
    // unchanged so `~` still does the work, and only the HOME it expands against moves.
    results.push(test("tilde_ls_root", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env("ls ~/0-core", &[("HOME", home.as_str())])?,
            "zero",
        )
    }));
    results.push(test("tilde_ls_scripts", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env(
                "ls ~/0-core/zero/packages/zero/scripts",
                &[("HOME", home.as_str())],
            )?,
            "deploy",
        )
    }));
    // Renamed from tilde_ls_runtime: runtime/ no longer exists. Machine-local
    // state moved to XDG state home, and a test named for a directory that is
    // gone is the same stale label this suite exists to catch.
    // NOT a ~/0-core path -- this one reaches the fixture ROOT, which is why fixture_home
    // creates .local/state/zero as well as the 0-core tree.
    results.push(test("tilde_ls_state", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env("ls ~/.local/state/zero", &[("HOME", home.as_str())])?,
            "state.db",
        )
    }));
    results.push(test("tilde_cat_cargo", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env(
                "cat ~/0-core/zero/shell/novashell/Cargo.toml",
                &[("HOME", home.as_str())],
            )?,
            "novashell",
        )
    }));
    results.push(test("tilde_pipe_grep", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env("ls ~/0-core | grep zero", &[("HOME", home.as_str())])?,
            "zero",
        )
    }));
    results.push(test("tilde_cat_pipe_grep", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env(
                "cat ~/0-core/zero/shell/novashell/Cargo.toml | grep name",
                &[("HOME", home.as_str())],
            )?,
            "name",
        )
    }));

    // --- PIPES (additional) ---
    results.push(test("pipe_wc_words", Category::Pipes, || {
        expect_eq(&run_fsh("echo hello world | wc -w")?, "2")
    }));
    results.push(test("pipe_tr_upper", Category::Pipes, || {
        expect_eq(&run_fsh("echo hello | tr a-z A-Z")?, "HELLO")
    }));
    results.push(test("pipe_grep_match", Category::Pipes, || {
        expect_eq(&run_fsh("echo zero | grep zero")?, "zero")
    }));
    results.push(test("pipe_twice", Category::Pipes, || {
        expect_eq(&run_fsh("echo hello | tr a-z A-Z | tr A-Z a-z")?, "hello")
    }));
    results.push(test("pipe_ls_grep", Category::Pipes, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env("ls ~/0-core | grep zero", &[("HOME", home.as_str())])?,
            "zero",
        )
    }));

    // --- REGRESSION ---
    results.push(test(
        "regression_sigpipe_no_crash",
        Category::Regression,
        || {
            // Pipe to head should not crash with SIGPIPE
            let out = run_fsh("printf 'a\\nb\\nc\\nd\\ne\\n' | head -3")?;
            expect_eq(&out, "a\nb\nc")
        },
    ));
    results.push(test(
        "regression_243_closed_stdout_exits_141",
        Category::Regression,
        || {
            // INT-243: a builtin printing into a closed stdout must exit 141 silently, never panic.
            // 3e4ecfbe (the main.rs panic hook) made this true; this case is what keeps it true.
            let (stderr, code) = run_fsh_closed_stdout("dashboard")?;
            if stderr.contains("panicked") {
                return Err(format!("nsh panicked on a closed stdout: {}", stderr));
            }
            expect_exit(code, 141)
        },
    ));
    results.push(test(
        "repl_243_builtin_pipe_leaves_shell_alive",
        Category::Repl,
        || {
            // INT-243: the REPL door. Only the LAST command's output is captured (run_repl_lines
            // doc), so the probe goes last: if the pipeline took the shell down, it never answers.
            // Matched as a whole line so the typed command text cannot satisfy it.
            let out = repl::run_repl_lines(&["dashboard | head -4", "echo nsh-test-alive-243"])?;
            if out.iter().any(|l| l.trim() == "nsh-test-alive-243") {
                Ok(())
            } else {
                Err(format!(
                    "the shell did not answer after dashboard | head -4: {:?}",
                    out
                ))
            }
        },
    ));
    results.push(test(
        "regression_281_printing_builtin_pipe_refuses",
        Category::Regression,
        || {
            // INT-281: a builtin that prints to the terminal and returns Empty fed the pipe
            // nothing, so dashboard | wc -l printed the dashboard and then 0 -- a wrong answer
            // with no signal. The rest of the pipeline must not run, and nsh must say so.
            // stdout and stderr are checked together so the order of the tuple cannot matter.
            // INT-282: run --list stands in for dashboard, which now returns its text.
            let (a, b, code) = run_fsh_status("run --list | wc -l")?;
            let both = format!("{}\n{}", a, b);
            if both.lines().any(|l| l.trim() == "0") {
                return Err(format!(
                    "wc -l ran on an empty pipe and printed 0: {:?}",
                    both
                ));
            }
            expect_contains(&both, "the rest of the pipeline did not run")?;
            expect_exit(code, 1)
        },
    ));
    results.push(test(
        "repl_281_printing_builtin_pipe_refuses",
        Category::Repl,
        || {
            // INT-281: the REPL door. Whole-line and phrase matches, so the typed command
            // text cannot satisfy either check.
            // INT-282: run --list stands in for dashboard, which now returns its text.
            let out = repl::run_repl_lines(&["run --list | wc -l"])?;
            if out.iter().any(|l| l.trim() == "0") {
                return Err(format!(
                    "wc -l ran on an empty pipe and printed 0: {:?}",
                    out
                ));
            }
            if out
                .iter()
                .any(|l| l.contains("the rest of the pipeline did not run"))
            {
                Ok(())
            } else {
                Err(format!("no refusal after run --list | wc -l: {:?}", out))
            }
        },
    ));
    results.push(test(
        "regression_282_dashboard_feeds_wc",
        Category::Regression,
        || {
            // INT-282: dashboard returns its text, so it feeds the pipe: wc -l prints one
            // count, nothing else reaches stdout or stderr, and nothing refuses.
            let (a, b, code) = run_fsh_status("dashboard | wc -l")?;
            let both = format!("{}\n{}", a, b);
            if both.contains("the rest of the pipeline did not run") {
                return Err(format!(
                    "dashboard still refuses to lead a pipeline: {:?}",
                    both
                ));
            }
            let lines: Vec<&str> = both
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect();
            match lines.as_slice() {
                [n] => match n.parse::<u32>() {
                    Ok(c) if (10..=60).contains(&c) => expect_exit(code, 0),
                    _ => Err(format!("expected a line count of 10 to 60, got {:?}", n)),
                },
                _ => Err(format!(
                    "expected exactly one line, the count, got {:?}",
                    lines
                )),
            }
        },
    ));
    results.push(test("repl_282_dashboard_feeds_wc", Category::Repl, || {
        // INT-282: the REPL door. The count is matched as a whole line, so the typed
        // command text cannot satisfy it.
        let out = repl::run_repl_lines(&["dashboard | wc -l"])?;
        if out
            .iter()
            .any(|l| l.contains("the rest of the pipeline did not run"))
        {
            return Err(format!(
                "dashboard still refuses to lead a pipeline: {:?}",
                out
            ));
        }
        if out
            .iter()
            .any(|l| matches!(l.trim().parse::<u32>(), Ok(c) if (10..=60).contains(&c)))
        {
            Ok(())
        } else {
            Err(format!(
                "no line count of 10 to 60 after dashboard | wc -l: {:?}",
                out
            ))
        }
    }));
    results.push(test(
        "regression_282_piped_output_ends_with_newline",
        Category::Regression,
        || {
            // INT-282: the peel wrote Output text as returned, and Output text has no final
            // newline (the display sites println! it), so wc -l counted one line short:
            // alias printed 244 lines and alias | wc -l said 243 (2026-10-08).
            // The first version counted run_fsh("alias") lines against alias | wc -l. run_fsh
            // trims, and alias opens with a blank line, so the trim hid the missing newline and
            // the test passed on the unfixed build. It now asserts the fact itself: the last byte
            // through the pipe. help is static text, so the case database cannot change it.
            let last = run_fsh("help | tail -c 1 | od -An -c")?;
            expect_eq(last.trim(), "\\n")
        },
    ));
    results.push(test(
        "regression_282_piped_output_carries_no_escape",
        Category::Regression,
        || {
            // INT-282: colored colours whenever stdout is a terminal, so in the REPL an Output
            // carried escape codes into the pipe. CLICOLOR_FORCE=1 makes the -c door colour too,
            // so this sees what the REPL sees. Plain dashboard must carry codes, or the test
            // proves nothing; piped through od -c it must carry none.
            let force = [("CLICOLOR_FORCE", "1")];
            let plain = run_fsh_env("dashboard", &force)?;
            if !plain.contains('\x1b') {
                return Err(
                    "CLICOLOR_FORCE=1 did not colour dashboard; this test cannot see a strip"
                        .to_string(),
                );
            }
            let dumped = run_fsh_env("dashboard | od -An -c", &force)?;
            if dumped.trim().is_empty() {
                return Err("dashboard | od -An -c printed nothing".to_string());
            }
            if dumped.contains("033") {
                let head: Vec<&str> = dumped.lines().take(3).collect();
                return Err(format!("an escape byte crossed the pipe: {:?}", head));
            }
            Ok(())
        },
    ));
    results.push(test(
        "repl_251_caret_agrees_with_exit_status",
        Category::Repl,
        || {
            // INT-251: THE CLASS IS "one door writes state the other door owns".
            //
            // DRIVEN THROUGH THE REPL ON PURPOSE. The first version of this case used
            // run_fsh_status, which spawns `nsh -c` -- the door that was ALREADY correct. It
            // passed, and it would NOT have caught the bug it was written for. The harness note
            // on run_repl_lines_env says exactly this about a different case: a test that can
            // only knock on one door cannot report on the other.
            //
            // The caret cache had ONE writer, inside execute_and_record, which the spine path
            // skips via `continue`. A spine-claimed `false` left the previous verdict standing:
            // the prompt said fail, the file said success.
            //
            // This does NOT assert "false writes failure" -- that passes again the moment a
            // third executor arrives with its own missing write. It asserts the INVARIANT: the
            // cache agrees with the status the shell itself reported, whichever door ran it.
            //
            // XDG_CACHE_HOME is redirected so the case reads its OWN caret file rather than the
            // live one, which would otherwise make this depend on whatever ran last.
            let tmp = std::env::temp_dir().join(format!("nsh-test-caret-{}", std::process::id()));
            let _ = std::fs::create_dir_all(&tmp);
            let cache = tmp.join("zero").join("last-exit-status");
            let env = [("XDG_CACHE_HOME", tmp.to_string_lossy().to_string())];
            let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (*k, v.as_str())).collect();

            for line in ["false", "true"] {
                let (_, code) = repl::run_repl_lines_status(&[line], &env)?;
                let cached = std::fs::read_to_string(&cache).unwrap_or_default();
                let cached = cached.trim().to_string();
                if (code.unwrap_or(0) == 0) != (cached == "success") {
                    let _ = std::fs::remove_dir_all(&tmp);
                    return Err(format!(
                        "caret disagrees after {} in the REPL: status {:?}, cache {}",
                        line, code, cached
                    ));
                }
            }
            let _ = std::fs::remove_dir_all(&tmp);
            Ok(())
        },
    ));
    results.push(test(
        "regression_tilde_not_literal",
        Category::Regression,
        || {
            // ~ must never appear literally in output when used as path
            let out = run_fsh("echo ~/0-core")?;
            if out.contains('~') {
                Err(format!("~ not expanded: {:?}", out))
            } else {
                Ok(())
            }
        },
    ));
    results.push(test(
        "regression_empty_pipe_ok",
        Category::Regression,
        || {
            // Empty output through pipe should not error
            run_fsh("echo '' | cat")?;
            Ok(())
        },
    ));

    // --- ADDITIONAL TESTS from fsh_audit.sh ---
    results.push(test("date_has_year", Category::Regression, || {
        expect_contains(&run_fsh("date")?, "2026")
    }));
    results.push(test("ls_la_tmp", Category::Regression, || {
        let out = run_fsh("ls /tmp")?;
        if out.is_empty() {
            Err("ls /tmp empty".to_string())
        } else {
            Ok(())
        }
    }));
    results.push(test("grep_pattern_match", Category::Regression, || {
        expect_eq(&run_fsh("printf 'foo\nbar\nbaz\n' | grep bar")?, "bar")
    }));
    results.push(test("grep_r_in_src", Category::Regression, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env(
                "grep -r 'expand_braces' ~/0-core/zero/shell/novashell/src/ | head -1",
                &[("HOME", home.as_str())],
            )?,
            "expand_braces",
        )
    }));
    results.push(test("awk_print_field", Category::Regression, || {
        expect_eq(
            &run_fsh("echo 'christian:x:1000' | awk -F: '{print $1}'")?,
            "christian",
        )
    }));
    results.push(test("awk_in_pipeline", Category::Regression, || {
        expect_eq(
            &run_fsh("printf 'a 1\nb 2\nc 3\n' | awk '{print $2}' | head -1")?,
            "1",
        )
    }));
    results.push(test("nsh_c_echo", Category::Regression, || {
        expect_eq(&run_fsh("echo hello")?, "hello")
    }));
    results.push(test("nsh_c_pipeline", Category::Regression, || {
        expect_eq(&run_fsh("echo zero | tr a-z A-Z")?, "ZERO")
    }));
    results.push(test("semicolons_pipeline", Category::Regression, || {
        expect_contains(&run_fsh("echo a; echo b | tr a-z A-Z")?, "B")
    }));
    results.push(test("pipe_wc_chars", Category::Pipes, || {
        expect_eq(&run_fsh("echo hello | wc -c")?, "6")
    }));
    results.push(test("ls_pipe_grep_tmp", Category::Pipes, || {
        // create fsh_t file first
        std::fs::write("/tmp/nsh_t1.txt", "zero writes").ok();
        expect_contains(&run_fsh("ls /tmp | grep nsh")?, "nsh")
    }));
    results.push(test("tilde_ls_pipe_sort", Category::Tilde, || {
        let home = fixture_home()?;
        let out = run_fsh_env("ls ~/0-core | sort | head -1", &[("HOME", home.as_str())])?;
        if out.is_empty() {
            Err("no output".to_string())
        } else {
            Ok(())
        }
    }));
    // A tilde path through a nested pipe. The fixture puts more than one entry under the shell
    // and grep keeps exactly one, so a count of 1 shows all three stages ran: 0 means the tilde
    // or the listing failed, more than 1 means grep kept everything. It needs no brand name.
    results.push(test("tilde_nested_pipe", Category::Tilde, || {
        let home = fixture_home()?;
        let out = run_fsh_env(
            "ls ~/0-core/zero/shell | grep novashell | wc -l",
            &[("HOME", home.as_str())],
        )?;
        match out.trim() {
            "1" => Ok(()),
            other => Err(format!("expected 1 got {:?}", other)),
        }
    }));
    results.push(test("where_delete_vocab", Category::Vocabulary, || {
        expect_contains(
            &run_fsh("core vocabulary where delete 2>/dev/null || echo vocabulary")?,
            "vocabulary",
        )
    }));
    // ⚠️ THIS IS grep_r_in_src UNDER A DIFFERENT NAME. Same command, same assertion, different
    // category -- and it does not test `fsearch` at all despite the name. Converted rather than
    // deduplicated because deleting a case is a decision and this patch is a mechanical move.
    results.push(test("fsearch_rust_finds", Category::Vocabulary, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env(
                "grep -r expand_braces ~/0-core/zero/shell/novashell/src/ | head -1",
                &[("HOME", home.as_str())],
            )?,
            "expand_braces",
        )
    }));
    results.push(test("grep_in_and_chain", Category::Regression, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env(
                "echo ok && grep 'expand_braces' ~/0-core/zero/shell/novashell/src/main.rs | head -1",
                &[("HOME", home.as_str())],
            )?,
            "expand_braces",
        )
    }));
    results.push(test("cat_hostname", Category::Regression, || {
        let out = run_fsh("cat /etc/hostname")?;
        if out.is_empty() {
            Err("hostname empty".to_string())
        } else {
            Ok(())
        }
    }));
    results.push(test("echo_env_home", Category::Regression, || {
        expect_contains(&run_fsh("echo $HOME")?, &home())
    }));

    results.push(test("tilde_ls_rust_tools", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env("ls ~/0-core/zero/shell", &[("HOME", home.as_str())])?,
            "novashell",
        )
    }));
    // PHASE 2 of the fixture work (2026-09-10). THE FIRST CONVERSION, done alone so the pattern
    // is proven before eighteen more follow it.
    //
    // The command string is UNCHANGED.  still does the work -- that is the whole reason these
    // cases are worth keeping rather than deleting. What changed is the HOME they expand against:
    // the fixture, so the case tests tilde expansion instead of testing that one laptop has a
    // checkout. It is now  rather than  because it no longer needs one.
    results.push(test("tilde_ls_docs", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env("ls ~/0-core/docs", &[("HOME", home.as_str())])?,
            "PHILOSOPHY",
        )
    }));
    results.push(test("tilde_ls_intents", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env("ls ~/0-core/zero/intents", &[("HOME", home.as_str())])?,
            "future",
        )
    }));
    results.push(test("tilde_deep_nested", Category::Tilde, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env(
                "ls ~/0-core/zero/shell/novashell/src",
                &[("HOME", home.as_str())],
            )?,
            "main.rs",
        )
    }));
    results.push(test("cat_reads_file", Category::Regression, || {
        std::fs::write("/tmp/nsh_t1.txt", "zero writes").map_err(|e| e.to_string())?;
        expect_contains(&run_fsh("cat /tmp/nsh_t1.txt")?, "zero writes")
    }));
    results.push(test("tilde_in_subshell", Category::Tilde, || {
        let home = fixture_home()?;
        let out = run_fsh_env("echo $(ls ~/0-core | head -1)", &[("HOME", home.as_str())])?;
        if out.is_empty() {
            Err("empty output".to_string())
        } else {
            Ok(())
        }
    }));
    results.push(test("ls_tmp_exists", Category::Regression, || {
        let out = run_fsh("ls /tmp")?;
        if out.is_empty() {
            Err("ls /tmp empty".to_string())
        } else {
            Ok(())
        }
    }));

    // --- PROJECT-SPECIFIC TESTS beyond fsh_audit.sh ---
    // RED ON PURPOSE, 2026-09-10, AND IT IS THE SHELL THAT IS WRONG.
    //
    // Converting this case to the fixture surfaced a divergence nobody had written down. Listing
    // a DIRECTORY works everywhere. Naming the FILE directly answers "No results." under the
    // fixture HOME, while the same command against the real HOME prints the path.
    //
    // Reproduced by hand inside devbox: create the directory, write two bytes into state.db,
    // then list it both ways. The directory listing shows state.db at 2 bytes; naming the file
    // answers "No results." -- which is value.rs:175, the EMPTY-TABLE renderer. So the builtin
    // handed back an empty Value rather than saying it could not do what was asked. That is
    // INT-245's collapse again: a refusal and an empty result are one value.
    //
    // LEFT FAILING DELIBERATELY. Rewriting the assertion to list the directory instead would go
    // green and duplicate tilde_ls_state, which already covers that. A suite that goes green by
    // asking an easier question is the defect this whole exercise exists to remove.
    results.push(test("state_db_exists", Category::Regression, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env(
                "ls ~/.local/state/zero/state.db",
                &[("HOME", home.as_str())],
            )?,
            "state.db",
        )
    }));
    // INT-097 claimed a correct tokenizer for nested quotes and escapes; the proptests said
    // in writing that escaped quoting was NOT YET INTERPRETED. It was declared done and never
    // was, and an escaped quote HUNG THE PROMPT until 2026-08-21. These run in the real REPL
    // so the claim cannot drift from the behaviour again.
    results.push(test(
        "escaped_quote_is_literal",
        Category::Regression,
        || expect_contains(&run_fsh("echo \"a\\\"b\"")?, "a\"b"),
    ));
    results.push(test(
        "escaped_dollar_does_not_expand",
        Category::Regression,
        || expect_contains(&run_fsh("echo \"\\$HOME\"")?, "$HOME"),
    ));
    results.push(test(
        "doubled_backslash_is_one",
        Category::Regression,
        || expect_contains(&run_fsh("echo \"a\\\\b\"")?, "a\\b"),
    ));
    // INT-221 G1, RED FIRST. `pick` reached for `sk`, which is not installed, while `fzf` sits on
    // the same system -- so every fuzzy selection failed. This case fails until the selector is
    // fzf AND the message names the missing dependency.
    //
    // The PATH is built at RUNTIME, dropping only directories that contain an fzf executable.
    // Hardcoding would rot: where fzf, rg, sh and git live differs between machines. Everything else
    // stays, so fsh starts normally and ONLY the selector goes missing -- otherwise the case would
    // be testing "fsh cannot start" rather than "fsh cannot find its selector".
    //
    // `pick intent` is used rather than `pick file`, which shells out to rg first: nothing else
    // can fail before the selector is reached.
    // A FIXTURE CANNOT FIX THIS ONE. The case strips fzf from PATH and asserts the failure names
    // the missing dependency. Without 0-Core, INT-230 refuses FIRST -- "pick intent: needs
    // 0-Core, which is not present" -- so the dependency check is never reached and the case
    // measures nothing. It needs a real 0-Core that MEANS something, not a directory shaped like one.
    results.push(repo_test(
        "pick_without_fzf_names_the_dependency",
        Category::Regression,
        "needs a real 0-Core: INT-230 refuses before the fzf check is reached",
        || {
            let stripped: Vec<String> = std::env::var("PATH")
                .unwrap_or_default()
                .split(':')
                .filter(|d| !std::path::Path::new(d).join("fzf").exists())
                .map(|d| d.to_string())
                .collect();
            let path = stripped.join(":");
            let got = match run_fsh_env("pick intent", &[("PATH", path.as_str())]) {
                Ok(o) => o,
                Err(e) => e,
            };
            if got.contains("sk") {
                return Err(format!(
                    "pick still reaches for skim, which is not installed on this system: {}",
                    got
                ));
            }
            if !got.contains("fzf") {
                return Err(format!(
                    "the failure does not name the missing dependency, so a reader cannot tell a \
                 missing tool from a typo: {}",
                    got
                ));
            }
            Ok(())
        },
    ));
    // INT-227 G6: A CAPABILITY THAT IS ABSENT MUST NOT LOOK LIKE AN ANSWER.
    //
    // ⚠️ THIS IS THE GATE, AND A STATIC CHECK WAS DEMOTED TO MAKE ROOM FOR IT. A scanner that read
    // source text for swallowed errors PASSED with a live guard disabled by `if false &&` -- the
    // guard's TEXT was still there. A checker that reads text cannot establish a runtime property,
    // and a comment claiming a site is safe is metadata rather than evidence.
    //
    // ★ THE ASSERTIONS ARE SEMANTIC, NOT IMPLEMENTATION-SHAPED. None of them asks whether has_tool
    // was called. They ask what a person sees: does the shell SAY the capability is missing, or
    // does it report an empty result that reads as a real answer?
    //
    // PATH is built at runtime, dropping only the directories holding the one tool, so fsh still
    // starts normally and ONLY that capability disappears.
    fn path_without(tool: &str) -> String {
        std::env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .filter(|d| !std::path::Path::new(d).join(tool).exists())
            .collect::<Vec<_>>()
            .join(":")
    }

    results.push(test(
        "services_without_systemctl_reports_unavailable",
        Category::Regression,
        || {
            let path = path_without("systemctl");
            let got = match run_fsh_env("services", &[("PATH", path.as_str())]) {
                Ok(o) => o,
                Err(e) => e,
            };
            // The defect: .ok() then unwrap_or_default() produced an EMPTY TABLE, so a machine
            // with services running was told it had none.
            if !got.to_lowercase().contains("systemctl") {
                return Err(format!(
                    "an absent service manager must be named, not reported as an empty list: {}",
                    got.trim()
                ));
            }
            Ok(())
        },
    ));

    results.push(test(
        "logs_without_journalctl_reports_unavailable",
        Category::Regression,
        || {
            let path = path_without("journalctl");
            let got = match run_fsh_env("logs", &[("PATH", path.as_str())]) {
                Ok(o) => o,
                Err(e) => e,
            };
            // The defect: an empty string became an empty table -- "no log entries" on a machine
            // with a full journal. The streaming form was worse: a header, then nothing forever.
            if !got.to_lowercase().contains("journalctl") {
                return Err(format!(
                    "an absent log source must be named, not reported as no entries: {}",
                    got.trim()
                ));
            }
            Ok(())
        },
    ));

    results.push(test(
        "packages_without_pacman_reports_unavailable",
        Category::Regression,
        || {
            // packages reads pacman. The whole point of the case: a query that COULD NOT RUN
            // must be named, never reported as an empty list. INT-227 built that distinction.
            let path = path_without("pacman");
            let got = match run_fsh_env("packages", &[("PATH", path.as_str())]) {
                Ok(o) => o,
                Err(e) => e,
            };
            // The defect: one helper returned vec![] on spawn failure, feeding four callers that
            // each read it as "nothing found".
            let lower = got.to_lowercase();
            if !(lower.contains("pacman") || lower.contains("cannot query")) {
                return Err(format!(
                    "an unqueryable package database must be named, not reported as an empty list: {}",
                    got.trim()
                ));
            }
            Ok(())
        },
    ));

    // INT-257 LAW 1: a write inside the devshell never reaches the host.
    //
    // The law everything else rests on, asserted the way the intent demands -- by the HOST
    // checking afterwards, never by the sandbox reporting on itself. The command proves it ran
    // inside (the hostname), writes a probe into HOME, and the case then looks for that probe
    // on the real filesystem. If the law ever breaks, the probe is found, removed, and the case
    // fails loudly. The session directory it creates is removed whatever happens.
    results.push(repo_test(
        "devshell_write_inside_never_reaches_host",
        Category::Regression,
        "devshell lives in the shell scripts directory and needs the checkout",
        || {
            let home = std::env::var("HOME").map_err(|e| format!("HOME: {e}"))?;
            let script = format!("{}/0-core/zero/scripts/devshell", home);
            let probe = format!("{}/.nsh-test-devshell-probe", home);
            let _ = std::fs::remove_file(&probe);
            let out = std::process::Command::new("bash")
                .arg(&script)
                .args([
                    "/bin/sh",
                    "-c",
                    "touch ~/.nsh-test-devshell-probe; echo INSIDE-$(hostname)",
                ])
                .output()
                .map_err(|e| format!("could not run devshell: {e}"))?;
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            if let Some(line) = stdout
                .lines()
                .find(|l| l.trim_start().starts_with("upper: "))
            {
                let dir = line.trim_start().trim_start_matches("upper: ").trim();
                // The sessions directory is ASKED OF DEVSHELL ITSELF (INT-257): one definition,
                // never a path hardcoded here to drift from the script's.
                let sessions = std::process::Command::new("bash")
                    .arg(&script)
                    .arg("--where")
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_default();
                if !sessions.is_empty() && dir.starts_with(&format!("{}/", sessions)) {
                    // rm -rf, not remove_dir_all: overlayfs leaves a mode-000 work dir that
                    // remove_dir_all cannot read, and the session would silently stay behind.
                    let _ = std::process::Command::new("rm").args(["-rf", dir]).status();
                }
            }
            if !out.status.success() {
                return Err(format!(
                    "devshell exited {:?}: {}",
                    out.status.code(),
                    String::from_utf8_lossy(&out.stderr).trim()
                ));
            }
            if !stdout.contains("INSIDE-devshell") {
                return Err(format!(
                    "the command did not run inside the devshell: {}",
                    stdout.trim()
                ));
            }
            if std::path::Path::new(&probe).exists() {
                let _ = std::fs::remove_file(&probe);
                return Err(
                    "LAW 1 BROKEN: a file written inside the devshell exists on the host".into(),
                );
            }
            Ok(())
        },
    ));
    // INT-257, THE CLASS: nothing the sandbox can reach is writable-and-persistent, or connectable.
    // Asked from INSIDE a real session for every path, not a list of the famous ones:
    //   mounts   every mount not shadowed by a later one is read-only, or dies with the session
    //            (overlay, tmpfs, proc, devpts, mqueue, and bwrap's own /dev nodes). Any other rw
    //            mount writes to a real disk. The mount table answers that exactly; a walk of
    //            24 GB of build output checking directories would be slow and still miss files.
    //   sockets  every socket file in the tree, /proc and /sys aside, REFUSES a connection.
    // The launch probe checks the worst holes on every launch; this checks the whole class.
    results.push(repo_test(
        "devshell_class_nothing_writable_or_connectable",
        Category::Regression,
        "devshell lives in the shell scripts directory and needs the checkout",
        || {
            let home = std::env::var("HOME").map_err(|e| format!("HOME: {e}"))?;
            let script = format!("{}/0-core/zero/scripts/devshell", home);
            let inside = r##"import os, socket, stat
holes = []
mounts = []
with open('/proc/self/mountinfo') as f:
    for line in f:
        a, b = line.rstrip('\n').split(' - ', 1)
        pa, pb = a.split(' '), b.split(' ')
        mounts.append((pa[4], pa[5].split(','), pb[0]))
DISPOSABLE = {'overlay', 'tmpfs', 'proc', 'devpts', 'mqueue'}
DEVNODES = {'/dev/null', '/dev/zero', '/dev/full', '/dev/random', '/dev/urandom', '/dev/tty'}
checked = 0
for i, (mp, opts, fs) in enumerate(mounts):
    later = mounts[i + 1:]
    if any(m == mp or (m != '/' and mp.startswith(m + '/')) for m, _, _ in later):
        continue
    checked += 1
    if 'ro' in opts or fs in DISPOSABLE or (fs == 'devtmpfs' and mp in DEVNODES):
        continue
    holes.append('HOLE mount %s %s rw' % (mp, fs))
sockets = 0
stack = ['/']
while stack:
    d = stack.pop()
    try:
        it = os.scandir(d)
    except OSError:
        continue
    with it:
        for e in it:
            try:
                if e.is_dir(follow_symlinks=False):
                    if d == '/' and e.name in ('proc', 'sys'):
                        continue
                    stack.append(e.path)
                elif not e.is_file(follow_symlinks=False) and not e.is_symlink():
                    if stat.S_ISSOCK(e.stat(follow_symlinks=False).st_mode):
                        sockets += 1
                        s = socket.socket(socket.AF_UNIX)
                        s.settimeout(1)
                        try:
                            s.connect(e.path)
                            holes.append('HOLE socket %s connected' % e.path)
                        except OSError:
                            pass
                        finally:
                            s.close()
            except OSError:
                pass
print('CLASS host=%s mounts=%d sockets=%d' % (socket.gethostname(), checked, sockets))
for h in holes:
    print(h)
print('CLASS-DONE')"##;
            let out = std::process::Command::new("bash")
                .arg(&script)
                .args(["python3", "-c", inside])
                .output()
                .map_err(|e| format!("could not run devshell: {e}"))?;
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            if let Some(line) = stdout
                .lines()
                .find(|l| l.trim_start().starts_with("upper: "))
            {
                let dir = line.trim_start().trim_start_matches("upper: ").trim();
                let sessions = std::process::Command::new("bash")
                    .arg(&script)
                    .arg("--where")
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_default();
                if !sessions.is_empty() && dir.starts_with(&format!("{}/", sessions)) {
                    let _ = std::process::Command::new("rm").args(["-rf", dir]).status();
                }
            }
            if !out.status.success() {
                return Err(format!(
                    "devshell exited {:?}: {}",
                    out.status.code(),
                    String::from_utf8_lossy(&out.stderr).trim()
                ));
            }
            if !stdout.contains("CLASS-DONE") || !stdout.contains("host=devshell") {
                return Err(format!(
                    "the class walk did not complete inside the devshell: {}",
                    stdout.trim()
                ));
            }
            let holes: Vec<&str> = stdout.lines().filter(|l| l.starts_with("HOLE ")).collect();
            if !holes.is_empty() {
                return Err(format!(
                    "LAW BROKEN, {} hole(s): {}",
                    holes.len(),
                    holes.join("; ")
                ));
            }
            Ok(())
        },
    ));
    // INT-259: the live filter, proven by the CONTRAST rather than by a count.
    //
    // A row count would drift with every commit; "the archive appears without the flag and does
    // not appear with it" stays true as the tree grows. Measured 2026-09-23: `fsearch paths.rs`
    // returned 68 rows and 61 of them were the intent archive.
    //
    // ⚠️ AN EXPLICIT ROOT, not the harness's working directory. The harness sets NSH_KEEP_CWD, so
    // the shell stays wherever it was launched -- a case that relied on the cwd would be asserting
    // about whatever directory the runner happened to be in.
    results.push(repo_test(
        "repl_259_live_excludes_the_archive",
        Category::Repl,
        "the live filter is about this repository's own archive, so it needs the checkout",
        || {
            let root = format!("{}/0-core/zero", std::env::var("HOME").unwrap_or_default());
            let (all, _) = repl::run_repl_lines_status(
                &[&format!("fsearch restore_sigpipe {}", root)],
                &[],
            )?;
            let all = all.join("\n");
            // RED FIRST, INSIDE THE CASE: if the unfiltered search does not reach the archive,
            // the filtered one proving empty would mean nothing.
            // "zero/inten", not "intents/": the table TRUNCATES the path column --
            // rows read "/home/.../zero/inten...", so the string I first asserted on
            // could never appear. The case failed and the shell was right.
            if !all.contains("zero/inten") {
                return Err(format!(
                    "the unfiltered search found no archive rows, so this case cannot test the filter: {}",
                    all.chars().take(200).collect::<String>()
                ));
            }
            let (live, _) = repl::run_repl_lines_status(
                &[&format!("fsearch restore_sigpipe {} --live", root)],
                &[],
            )?;
            let live = live.join("\n");
            if live.contains("zero/inten") {
                return Err(format!(
                    "--live still returned archive rows: {}",
                    live.lines().filter(|l| l.contains("zero/inten")).take(2).collect::<Vec<_>>().join(" | ")
                ));
            }
            // AND IT MUST STILL FIND THE LIVE CODE. A filter that returns nothing at all would
            // pass the assertion above while being useless -- the empty-is-an-answer trap.
            if !live.contains("restore_sigpipe") {
                return Err(format!("--live returned no live code either: {}", live.chars().take(200).collect::<String>()));
            }
            Ok(())
        },
    ));
    results.push(test("core_binary_exists", Category::Regression, || {
        expect_contains(&run_fsh("which core")?, "core")
    }));
    results.push(repo_test(
        "crate_dirs_match_the_workspace",
        Category::Regression,
        "needs a real 0-Core: it reads the workspace Cargo.toml, which only a checkout has",
        || {
            // INT-267 L2: zero_core::paths owns where the crates live, and the workspace members
            // in Cargo.toml say the same thing in Cargo's words. Two spellings of one fact drift
            // unless something compares them. This does, in both directions, globs expanded.
            let root = std::path::Path::new(&home()).join("0-core");
            let text = std::fs::read_to_string(root.join("Cargo.toml"))
                .map_err(|e| format!("cannot read the workspace Cargo.toml: {}", e))?;
            let start = text
                .find("members = [")
                .ok_or("no `members = [` in the workspace Cargo.toml")?;
            let body = &text[start + "members = [".len()..];
            let end = body.find(']').ok_or("`members = [` is never closed")?;
            let mut members: Vec<std::path::PathBuf> = Vec::new();
            for item in body[..end].split(',') {
                let m = item.trim().trim_matches('"');
                if m.is_empty() {
                    continue;
                }
                if let Some(parent) = m.strip_suffix("/*") {
                    let dir = root.join(parent);
                    let entries = std::fs::read_dir(&dir).map_err(|e| {
                        format!("cannot read member glob {}: {}", dir.display(), e)
                    })?;
                    for e in entries {
                        let p = e.map_err(|e| e.to_string())?.path();
                        if p.join("Cargo.toml").is_file() {
                            members.push(p);
                        }
                    }
                } else {
                    members.push(root.join(m));
                }
            }
            members.sort();
            members.dedup();
            // AN EMPTY LIST IS NOT AN AGREEMENT. Two empty answers would compare equal.
            if members.is_empty() {
                return Err("the workspace declares no members -- that is not a clean tree".into());
            }
            let owned = zero_core::paths::crate_dirs().map_err(|e| {
                format!("zero_core::paths::crate_dirs() could not read the tree: {}", e)
            })?;
            let only_cargo: Vec<String> = members
                .iter()
                .filter(|m| !owned.contains(m))
                .map(|m| m.display().to_string())
                .collect();
            let only_owner: Vec<String> = owned
                .iter()
                .filter(|o| !members.contains(o))
                .map(|o| o.display().to_string())
                .collect();
            if only_cargo.is_empty() && only_owner.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "Cargo.toml and zero_core::paths disagree about the crates.\n  in Cargo.toml only: {:?}\n  in zero_core::paths only: {:?}",
                    only_cargo, only_owner
                ))
            }
        },
    ));
    results.push(repo_test(
        "crate_directory_has_one_owner",
        Category::Regression,
        "needs a real 0-Core: it reads every tracked .rs file, which only a checkout has",
        || {
            // INT-267 L2: ONE OWNER. zero_core::paths says where the crates live; no other live
            // Rust string literal names the crate directory. Each place that did was a place a
            // move had to find by hand, and three had already gone dead without a sound
            // (anomaly, integrity, zero-zone).
            //
            // A RATCHET while the sites migrate: crate-paths-allowed.txt lists each file and
            // literal still waiting. A literal not on the list fails -- a new hardcoded path. An
            // entry no longer found fails too -- the list only shrinks, and says so. The list is
            // empty when the migration is done. It is a data file so this file does not flag
            // itself. The names it looks for come from the owner too: every parent the crates live
            // in or have lived in, so a move is seen the moment the owner records it.
            fn literals(line: &str) -> Vec<String> {
                let c: Vec<char> = line.chars().collect();
                let mut out = Vec::new();
                let mut i = 0;
                while i < c.len() {
                    let char_literal = i > 0
                        && (c[i - 1] == '\\' || (c[i - 1] == '\'' && c.get(i + 1) == Some(&'\'')));
                    if c[i] == '"' && !char_literal {
                        let mut s = String::new();
                        i += 1;
                        while i < c.len() && c[i] != '"' {
                            if c[i] == '\\' && i + 1 < c.len() {
                                s.push(c[i]);
                                i += 1;
                            }
                            s.push(c[i]);
                            i += 1;
                        }
                        out.push(s);
                    }
                    i += 1;
                }
                out
            }
            let words: Vec<&str> = zero_core::paths::CRATE_PARENTS_HISTORY
                .iter()
                .chain(zero_core::paths::CRATE_PARENTS.iter())
                .copied()
                .collect();
            let owner = "zero-core/src/paths.rs";
            let allowed: Vec<(String, String)> = include_str!("../crate-paths-allowed.txt")
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .filter_map(|l| l.split_once('\t'))
                .map(|(f, s)| (f.to_string(), s.to_string()))
                .collect();
            let root = std::path::Path::new(&home()).join("0-core");
            let out = Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(["ls-files", "-z", "--", "*.rs"])
                .output()
                .map_err(|e| format!("cannot run git ls-files: {}", e))?;
            if !out.status.success() {
                return Err(format!(
                    "git ls-files failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            let mut files = 0usize;
            let mut found: Vec<(String, String, usize)> = Vec::new();
            for raw in out.stdout.split(|b| *b == 0).filter(|p| !p.is_empty()) {
                let path = String::from_utf8_lossy(raw).to_string();
                if path.ends_with(owner) {
                    continue;
                }
                let text = std::fs::read_to_string(root.join(&path))
                    .map_err(|e| format!("cannot read {}: {}", path, e))?;
                files += 1;
                for (n, line) in text.lines().enumerate() {
                    if line.trim_start().starts_with("//") {
                        continue;
                    }
                    for lit in literals(line) {
                        if words.iter().any(|w| lit.contains(w)) {
                            found.push((path.clone(), lit, n + 1));
                        }
                    }
                }
            }
            // AN EMPTY TREE IS NOT A CLEAN ONE. Reading nothing must not pass as finding nothing.
            if files == 0 {
                return Err("read no .rs files at all -- that is not a clean tree".to_string());
            }
            // Counted, not just present: two sites with the same literal in one file need two lines,
            // so a third copy cannot hide behind an entry the first two already earned.
            let pairs: Vec<(String, String)> = found
                .iter()
                .map(|(f, l, _)| (f.clone(), l.clone()))
                .collect();
            let count = |v: &[(String, String)], f: &str, l: &str| {
                v.iter().filter(|(a, b)| a == f && b == l).count()
            };
            let mut problems: Vec<String> = Vec::new();
            for (f, l, n) in &found {
                if count(&pairs, f, l) > count(&allowed, f, l) {
                    problems.push(format!("{}:{}\t{}", f, n, l));
                }
            }
            let mut seen: Vec<(String, String)> = Vec::new();
            for (af, al) in &allowed {
                if seen.iter().any(|(a, b)| a == af && b == al) {
                    continue;
                }
                seen.push((af.clone(), al.clone()));
                let (want, have) = (count(&allowed, af, al), count(&pairs, af, al));
                if have < want {
                    problems.push(format!(
                        "allowed {} but found {}, remove the extra: {}\t{}",
                        want, have, af, al
                    ));
                }
            }
            if problems.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "{} crate-directory literal(s) outside zero_core::paths ({} read):\n  {}",
                    problems.len(),
                    files,
                    problems.join("\n  ")
                ))
            }
        },
    ));
    results.push(repo_test(
        "no_retired_display_name_in_printed_strings",
        Category::Regression,
        "needs a real 0-Core: it reads the source tree, which only a checkout has",
        || {
            // INT-247: A RETIRED NAME MUST NOT BE PRINTED. Project 0 went by two earlier names,
            // and those old names lived on in banners, headers and messages
            // long after the decision -- written by CODE, so no document edit could remove them.
            //
            // Reads every ordinary string literal under tools/ and engine/ and fails naming
            // each file:line that still carries a retired name. Comment lines are skipped:
            // history and reasoning may say the old names. A name joins RETIRED only when its
            // pass is finished, so this case is green at every commit and red the moment one
            // comes back.
            //
            // The phrases are built from PIECES so this file does not flag itself.
            fn string_literals(line: &str) -> Vec<String> {
                let c: Vec<char> = line.chars().collect();
                let mut out = Vec::new();
                let mut i = 0;
                while i < c.len() {
                    let char_literal = i > 0
                        && (c[i - 1] == '\\' || (c[i - 1] == '\'' && c.get(i + 1) == Some(&'\'')));
                    if c[i] == '"' && !char_literal {
                        let mut s = String::new();
                        i += 1;
                        while i < c.len() && c[i] != '"' {
                            if c[i] == '\\' && i + 1 < c.len() {
                                s.push(c[i]);
                                i += 1;
                            }
                            s.push(c[i]);
                            i += 1;
                        }
                        out.push(s);
                    }
                    i += 1;
                }
                out
            }
            // 2026-09-24: also the RUN-ON form. The name followed directly by a version placeholder
            // reads as one long number. A `v` separates them, so `v{}` passes.
            let retired: Vec<String> = vec![
                ["Zero", " Core"].concat(),
                ["Project 0", " {}"].concat(),
                ["fae", "light-shell"].concat(),
                ["Fae", "light Shell"].concat(),
                ["f", "sh"].concat(),
            ];
            let base = std::path::Path::new(&home()).join("0-core/zero");
            let mut stack = zero_core::paths::crate_dirs()
                .map_err(|e| format!("cannot list the crates: {}", e))?;
            let mut hits: Vec<String> = Vec::new();
            let mut files = 0usize;
            while let Some(dir) = stack.pop() {
                let entries = std::fs::read_dir(&dir)
                    .map_err(|e| format!("cannot read {}: {}", dir.display(), e))?;
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if path.file_name().map(|n| n != "target").unwrap_or(false) {
                            stack.push(path);
                        }
                        continue;
                    }
                    if path.extension().map(|x| x != "rs").unwrap_or(true) {
                        continue;
                    }
                    files += 1;
                    let text = std::fs::read_to_string(&path)
                        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
                    for (n, line) in text.lines().enumerate() {
                        if line.trim_start().starts_with("//") {
                            continue;
                        }
                        for lit in string_literals(line) {
                            if retired.iter().any(|name| lit.contains(name.as_str())) {
                                let shown = path.strip_prefix(&base).unwrap_or(&path);
                                hits.push(format!("{}:{}  \"{}\"", shown.display(), n + 1, lit));
                            }
                        }
                    }
                }
            }
            // AN EMPTY TREE IS NOT A CLEAN ONE. Reading nothing must not pass as finding nothing.
            if files == 0 {
                return Err("read no .rs files at all -- that is not a clean tree".to_string());
            }
            if hits.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "{} retired display name(s) still printed:\n  {}",
                    hits.len(),
                    hits.join("\n  ")
                ))
            }
        },
    ));
    results.push(repo_test(
        "no_live_retired_name_in_any_tracked_file",
        Category::Regression,
        "needs a real 0-Core: it reads every tracked file, which only a checkout has",
        || {
            // INT-247 FINISH LINE: no live tracked file, of ANY type, says either retired word.
            // The display-name case above reads only .rs string literals; this one reads every
            // file git tracks, line by line, any case. History is exempt by ruling, and the
            // exemptions below are the whole list -- a new one is a ruling, not an edit.
            //
            // The words are built from PIECES so this file does not flag itself.
            fn has(hay: &[u8], needle: &[u8]) -> bool {
                hay.windows(needle.len()).any(|w| w == needle)
            }
            let words: Vec<String> = vec![["fae", "light"].concat(), ["for", "est"].concat()];
            // Ruled history by Christian 2026-10-01: a release record and the incident log.
            let history: Vec<&str> = vec!["zero/meta/releases/", "zero/meta/INCIDENTS.md"];
            let root = std::path::Path::new(&home()).join("0-core");
            let out = Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(["ls-files", "-z"])
                .output()
                .map_err(|e| format!("cannot run git ls-files: {}", e))?;
            if !out.status.success() {
                return Err(format!(
                    "git ls-files failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            let mut files = 0usize;
            let mut hits: Vec<String> = Vec::new();
            let mut unreadable: Vec<String> = Vec::new();
            for raw in out.stdout.split(|b| *b == 0).filter(|p| !p.is_empty()) {
                let path = String::from_utf8_lossy(raw).to_string();
                let base = path.rsplit('/').next().unwrap_or("").to_string();
                let low_base = base.to_lowercase();
                let exempt = path.starts_with("zero/intents/")
                    || base.starts_with("CHANGELOG")
                    || path == "AGENTS.md"
                    || [".ttf", ".otf", ".woff", ".woff2"]
                        .iter()
                        .any(|x| low_base.ends_with(*x))
                    || history.iter().any(|h| path.starts_with(*h));
                if exempt {
                    continue;
                }
                if words
                    .iter()
                    .any(|w| path.to_lowercase().contains(w.as_str()))
                {
                    hits.push(format!("{}  (the path itself)", path));
                }
                let full = root.join(&path);
                let meta = std::fs::symlink_metadata(&full)
                    .map_err(|e| format!("cannot stat {}: {}", path, e))?;
                if !meta.is_file() {
                    continue;
                }
                let bytes =
                    std::fs::read(&full).map_err(|e| format!("cannot read {}: {}", path, e))?;
                files += 1;
                let lower = bytes.to_ascii_lowercase();
                if !words.iter().any(|w| has(&lower, w.as_bytes())) {
                    continue;
                }
                let text = match String::from_utf8(bytes) {
                    Ok(t) => t,
                    Err(_) => {
                        unreadable.push(path);
                        continue;
                    }
                };
                // Registry entries with retired = true are the registry's own history.
                let mut retired_lines: Vec<usize> = Vec::new();
                if path == "zero/registry/tools.toml" {
                    let mut block: Vec<usize> = Vec::new();
                    let mut retired = false;
                    for (n, line) in text.lines().enumerate() {
                        if line.trim() == "[[tool]]" {
                            if retired {
                                retired_lines.extend(block.drain(..));
                            }
                            block.clear();
                            retired = false;
                        }
                        block.push(n);
                        if line.split_whitespace().collect::<String>() == "retired=true" {
                            retired = true;
                        }
                    }
                    if retired {
                        retired_lines.extend(block.drain(..));
                    }
                }
                // zero-gen's wordlist uses the second word as a word, not the brand (ruling E).
                let dictionary = format!("\"{}\"", words[1]);
                for (n, line) in text.lines().enumerate() {
                    if retired_lines.contains(&n) {
                        continue;
                    }
                    let mut low = line.to_lowercase();
                    if path.ends_with("zero-gen/src/main.rs") {
                        low = low.replace(&dictionary, "");
                    }
                    if words.iter().any(|w| low.contains(w.as_str())) {
                        hits.push(format!("{}:{}  {}", path, n + 1, line.trim()));
                    }
                }
            }
            // AN EMPTY TREE IS NOT A CLEAN ONE, and an undecodable file is not a clean one either.
            if files == 0 {
                return Err("read no tracked files at all -- that is not a clean tree".to_string());
            }
            if !unreadable.is_empty() {
                return Err(format!(
                    "could not decode, so could not check: {}",
                    unreadable.join(", ")
                ));
            }
            if hits.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "{} live line(s) say a retired name:\n  {}",
                    hits.len(),
                    hits.join("\n  ")
                ))
            }
        },
    ));
    results.push(repo_test(
        "core_doctor_check_reads_only",
        Category::Regression,
        "needs a real 0-Core: core starts on the real ledger",
        || {
            // INT-269 step 3: core doctor check <id> runs ONE check and writes nothing. core
            // runs on a COPY of state.db and an empty XDG_CACHE_HOME, so the real ledger and
            // cache are never touched. An undeclared id refuses with 64 and lists the declared
            // ids; a real check exits 0-3; neither adds a health_patterns or prediction_outcomes
            // row, and neither writes the health-status cache.
            let dir = std::env::temp_dir()
                .join(format!("nsh-test-doctor-check-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(dir.join("cache"))
                .map_err(|e| format!("cannot make {}: {}", dir.display(), e))?;
            let real = std::path::Path::new(&home()).join(".local/state/zero/state.db");
            let db = dir.join("state.db");
            std::fs::copy(&real, &db)
                .map_err(|e| format!("cannot copy {}: {}", real.display(), e))?;
            let cache = dir.join("cache");
            let count = |table: &str| -> Option<i64> {
                let c = rusqlite::Connection::open(&db).ok()?;
                c.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |r| r.get(0))
                    .ok()
            };
            let run = |id: &str| -> Result<(i32, String), String> {
                let o = Command::new("core")
                    .args(["doctor", "check", id])
                    .env("ZERO_STATE_DB", &db)
                    .env("XDG_CACHE_HOME", &cache)
                    .output()
                    .map_err(|e| format!("cannot run core: {}", e))?;
                Ok((
                    o.status.code().unwrap_or(-1),
                    format!(
                        "{}{}",
                        String::from_utf8_lossy(&o.stdout),
                        String::from_utf8_lossy(&o.stderr)
                    ),
                ))
            };
            let result = (|| -> Result<(), String> {
                let before = (count("health_patterns"), count("prediction_outcomes"));
                let (c, out) = run("no_such_check")?;
                if c != 64 || !out.contains("declared:") || !out.contains("disk_space") {
                    return Err(format!(
                        "an undeclared id must refuse with 64 and list the declared ids: exit {} {}",
                        c,
                        out.trim()
                    ));
                }
                let (c, out) = run("disk_space")?;
                if !(0..=3).contains(&c) || !out.to_lowercase().contains("disk") {
                    return Err(format!(
                        "a declared check must exit 0-3 and name itself: exit {} {}",
                        c,
                        out.trim()
                    ));
                }
                let after = (count("health_patterns"), count("prediction_outcomes"));
                if before != after {
                    return Err(format!(
                        "check wrote to state.db -- (health_patterns, prediction_outcomes) {:?} -> {:?}",
                        before, after
                    ));
                }
                let hs = cache.join("zero/health-status");
                if hs.exists() {
                    return Err(format!("check wrote the health cache at {}", hs.display()));
                }
                Ok(())
            })();
            let _ = std::fs::remove_dir_all(&dir);
            result
        },
    ));
    results.push(repo_test(
        "core_doctor_check_fingerprint",
        Category::Regression,
        "needs a real 0-Core: core starts on the real ledger",
        || {
            // INT-269 step 3: the doctor's Fingerprint check, run through the read-only
            // `core doctor check`. The record lives in a temp ZERO_STATE_DIR, so the real one is
            // never touched; ZERO_STATE_DB keeps core on the real ledger so it can start. In
            // order: no record is UNDETERMINED (2), a fresh record is PASS (0), and one changed
            // input is FAIL (1) naming its axis. The check itself never writes the record.
            let dir = std::env::temp_dir().join(format!(
                "nsh-test-doctor-fingerprint-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("cannot make {}: {}", dir.display(), e))?;
            let db = std::path::Path::new(&home()).join(".local/state/zero/state.db");
            let record = dir.join("fingerprint");
            let core = |args: &[&str]| -> Result<(i32, String), String> {
                let o = Command::new("core")
                    .args(args)
                    .env("ZERO_STATE_DIR", &dir)
                    .env("ZERO_STATE_DB", &db)
                    .output()
                    .map_err(|e| format!("cannot run core: {}", e))?;
                Ok((
                    o.status.code().unwrap_or(-1),
                    format!(
                        "{}{}",
                        String::from_utf8_lossy(&o.stdout),
                        String::from_utf8_lossy(&o.stderr)
                    ),
                ))
            };
            let check = ["doctor", "check", "fingerprint"];
            let result = (|| -> Result<(), String> {
                let (c, out) = core(&check)?;
                if c != 2 || !out.contains("not recorded yet") {
                    return Err(format!(
                        "with no record the check must be UNDETERMINED (exit 2): exit {} {}",
                        c,
                        out.trim()
                    ));
                }
                if record.exists() {
                    return Err(
                        "the doctor wrote the record -- only core fingerprint record may"
                            .to_string(),
                    );
                }
                let (c, out) = core(&["fingerprint", "record"])?;
                if c != 0 || !record.exists() {
                    return Err(format!("record did not write: exit {} {}", c, out.trim()));
                }
                let (c, out) = core(&check)?;
                if c != 0 || !out.contains("PASS") {
                    return Err(format!(
                        "after record the check must PASS (exit 0): exit {} {}",
                        c,
                        out.trim()
                    ));
                }
                let text = std::fs::read_to_string(&record).map_err(|e| e.to_string())?;
                let changed: Vec<String> = text
                    .lines()
                    .map(|l| {
                        if l.starts_with("identity.hostname=") {
                            "identity.hostname=not-this-machine".to_string()
                        } else {
                            l.to_string()
                        }
                    })
                    .collect();
                std::fs::write(&record, changed.join("\n") + "\n").map_err(|e| e.to_string())?;
                let (c, out) = core(&check)?;
                if c != 1 || !out.contains("identity.hostname") {
                    return Err(format!(
                        "a changed record must FAIL naming identity.hostname (exit 1): exit {} {}",
                        c,
                        out.trim()
                    ));
                }
                Ok(())
            })();
            let _ = std::fs::remove_dir_all(&dir);
            result
        },
    ));
    results.push(repo_test(
        "core_fingerprint_show_colours",
        Category::Regression,
        "needs a real 0-Core: core starts on the real ledger",
        || {
            // INT-269, INT-270: core fingerprint show prints PASS green, FAIL red, UNDETERMINED
            // yellow, and under a FAIL each input bold, its recorded value red, its live value green.
            // CLICOLOR_FORCE=1 because nsh-test reads through a pipe, and colored 2.2.0 colours a
            // pipe only when forced -- control.rs: CLICOLOR_FORCE outranks NO_COLOR and the tty
            // check. The record lives in a temp ZERO_STATE_DIR, so the real one is never touched.
            const GREEN: &str = "\u{1b}[32m";
            const RED: &str = "\u{1b}[31m";
            const YELLOW: &str = "\u{1b}[33m";
            const BOLD: &str = "\u{1b}[1m";
            let dir = std::env::temp_dir().join(format!(
                "nsh-test-fingerprint-colour-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("cannot make {}: {}", dir.display(), e))?;
            let db = std::path::Path::new(&home()).join(".local/state/zero/state.db");
            let record = dir.join("fingerprint");
            let core = |verb: &str| -> Result<(i32, String), String> {
                let o = Command::new("core")
                    .args(["fingerprint", verb])
                    .env("ZERO_STATE_DIR", &dir)
                    .env("ZERO_STATE_DB", &db)
                    .env("CLICOLOR_FORCE", "1")
                    .output()
                    .map_err(|e| format!("cannot run core: {}", e))?;
                Ok((
                    o.status.code().unwrap_or(-1),
                    format!(
                        "{}{}",
                        String::from_utf8_lossy(&o.stdout),
                        String::from_utf8_lossy(&o.stderr)
                    ),
                ))
            };
            let result = (|| -> Result<(), String> {
                let (c, out) = core("show")?;
                if c != 2 || !out.contains(&format!("{}UNDETERMINED", YELLOW)) {
                    return Err(format!(
                        "with no record, UNDETERMINED must print in yellow (exit 2): exit {} {:?}",
                        c,
                        out.trim()
                    ));
                }
                let (c, out) = core("record")?;
                if c != 0 || !record.exists() {
                    return Err(format!("record did not write: exit {} {}", c, out.trim()));
                }
                let (c, out) = core("show")?;
                if c != 0 || !out.contains(&format!("{}PASS", GREEN)) {
                    return Err(format!(
                        "after record, PASS must print in green (exit 0): exit {} {:?}",
                        c,
                        out.trim()
                    ));
                }
                let text = std::fs::read_to_string(&record).map_err(|e| e.to_string())?;
                let changed: Vec<String> = text
                    .lines()
                    .map(|l| {
                        if l.starts_with("identity.hostname=") {
                            "identity.hostname=not-this-machine".to_string()
                        } else {
                            l.to_string()
                        }
                    })
                    .collect();
                std::fs::write(&record, changed.join("\n") + "\n").map_err(|e| e.to_string())?;
                let (c, out) = core("show")?;
                if c != 1 || !out.contains(&format!("{}FAIL", RED)) {
                    return Err(format!(
                        "a changed record must FAIL in red (exit 1): exit {} {:?}",
                        c,
                        out.trim()
                    ));
                }
                // INT-270: the explain line under the FAIL -- what moved is visible before it is
                // read. The record says not-this-machine; the machine says its real hostname.
                let host = std::fs::read_to_string("/proc/sys/kernel/hostname")
                    .map(|h| h.trim().to_string())
                    .map_err(|e| format!("cannot read the hostname: {}", e))?;
                for (what, want) in [
                    ("the input in bold", format!("{}identity.hostname", BOLD)),
                    ("the recorded value in red", format!("{}not-this-machine", RED)),
                    ("the live value in green", format!("{}{}", GREEN, host)),
                ] {
                    if !out.contains(&want) {
                        return Err(format!(
                            "INT-270: under a FAIL the explain line must show {} -- {:?} is missing: {:?}",
                            what,
                            want,
                            out.trim()
                        ));
                    }
                }
                Ok(())
            })();
            let _ = std::fs::remove_dir_all(&dir);
            result
        },
    ));
    results.push(repo_test(
        "core_fingerprint_lifecycle",
        Category::Regression,
        "needs a real 0-Core: core starts on the real ledger",
        || {
            // INT-269 step 2: core fingerprint show and record, on the deployed core. The record
            // goes to a temp ZERO_STATE_DIR so the real one is never touched; ZERO_STATE_DB keeps
            // core on the real ledger so it can start. Show must never write; record is the one
            // writer; the outcomes are UNDETERMINED (2), PASS (0) and FAIL (1), in that order.
            let dir =
                std::env::temp_dir().join(format!("nsh-test-fingerprint-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("cannot make {}: {}", dir.display(), e))?;
            let db = std::path::Path::new(&home()).join(".local/state/zero/state.db");
            let record = dir.join("fingerprint");
            let run = |verb: &str| -> Result<(i32, String), String> {
                let o = Command::new("core")
                    .args(["fingerprint", verb])
                    .env("ZERO_STATE_DIR", &dir)
                    .env("ZERO_STATE_DB", &db)
                    .output()
                    .map_err(|e| format!("cannot run core: {}", e))?;
                Ok((
                    o.status.code().unwrap_or(-1),
                    format!(
                        "{}{}",
                        String::from_utf8_lossy(&o.stdout),
                        String::from_utf8_lossy(&o.stderr)
                    ),
                ))
            };
            let result = (|| -> Result<(), String> {
                let (c, out) = run("show")?;
                if c != 2 || !out.contains("not recorded yet") {
                    return Err(format!(
                        "before any record, show must be UNDETERMINED (exit 2): exit {} {}",
                        c,
                        out.trim()
                    ));
                }
                if record.exists() {
                    return Err("show wrote the record -- only record may write it".to_string());
                }
                let (c, out) = run("record")?;
                if c != 0 || !record.exists() {
                    return Err(format!("record did not write: exit {} {}", c, out.trim()));
                }
                let (c, out) = run("show")?;
                if c != 0 || !out.contains("PASS") {
                    return Err(format!(
                        "after record, show must PASS (exit 0): exit {} {}",
                        c,
                        out.trim()
                    ));
                }
                let text = std::fs::read_to_string(&record).map_err(|e| e.to_string())?;
                let changed: Vec<String> = text
                    .lines()
                    .map(|l| {
                        if l.starts_with("identity.hostname=") {
                            "identity.hostname=not-this-machine".to_string()
                        } else {
                            l.to_string()
                        }
                    })
                    .collect();
                std::fs::write(&record, changed.join("\n") + "\n").map_err(|e| e.to_string())?;
                let (c, out) = run("show")?;
                if c != 1 || !out.contains("FAIL") || !out.contains("identity.hostname") {
                    return Err(format!(
                        "a changed record must FAIL naming identity.hostname (exit 1): exit {} {}",
                        c,
                        out.trim()
                    ));
                }
                Ok(())
            })();
            let _ = std::fs::remove_dir_all(&dir);
            result
        },
    ));
    results.push(repo_test(
        "readme_index_count_matches_its_file",
        Category::Regression,
        "needs a real 0-Core: it reads tools/README.md",
        || {
            // zero-docs readme-index once printed "21 tools" while the file it wrote said 18: it
            // counted every crate read, not the active tools the file lists. The number a tool
            // reports must be the number it writes. --dry-run writes nothing.
            let readme = std::path::Path::new(&home()).join("0-core/zero/tools/README.md");
            let text = std::fs::read_to_string(&readme)
                .map_err(|e| format!("cannot read {}: {}", readme.display(), e))?;
            let file_count = text
                .split(" active tools")
                .next()
                .and_then(|head| head.rsplit(' ').next())
                .and_then(|n| n.parse::<usize>().ok())
                .ok_or("tools/README.md states no active-tools count")?;
            let out = Command::new("zero-docs")
                .args(["readme-index", "--dry-run"])
                .output()
                .map_err(|e| format!("cannot run zero-docs: {}", e))?;
            let said = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            let want = format!("{} active tools", file_count);
            if said.contains(&want) {
                Ok(())
            } else {
                Err(format!(
                    "readme-index reports {:?}, but the file it writes says {}",
                    said.trim(),
                    want
                ))
            }
        },
    ));
    results.push(repo_test(
        "devshell_commands_are_on_path",
        Category::Regression,
        "needs a real 0-Core: the links resolve into the checkout's zero/scripts",
        || {
            // AGENTS.md, Devshell: devshell is typed by name. ship deploys Rust binaries only, so
            // the four devshell commands reach PATH as links in ~/.local/bin to the repo's
            // scripts -- a script has no build step, so the repo copy is what runs. Each link must
            // exist, resolve into zero/scripts, and be executable; a broken or stale link is red.
            let home = home();
            let scripts = std::path::Path::new(&home).join("0-core/zero/scripts");
            let bin = std::path::Path::new(&home).join(".local/bin");
            let mut wrong: Vec<String> = Vec::new();
            for name in [
                "devshell",
                "devshell-diff",
                "devshell-promote",
                "devshell-checkpoint",
            ] {
                let link = bin.join(name);
                let want = std::fs::canonicalize(scripts.join(name))
                    .map_err(|e| format!("the repo script {} is unreadable: {}", name, e))?;
                match std::fs::canonicalize(&link) {
                    Err(_) => wrong.push(format!("{}: not on PATH at {}", name, link.display())),
                    Ok(got) if got != want => wrong.push(format!(
                        "{}: resolves to {}, not the repo script",
                        name,
                        got.display()
                    )),
                    Ok(got) => {
                        use std::os::unix::fs::PermissionsExt;
                        let mode = std::fs::metadata(&got)
                            .map(|m| m.permissions().mode())
                            .unwrap_or(0);
                        if mode & 0o111 == 0 {
                            wrong.push(format!("{}: resolves, but is not executable", name));
                        }
                    }
                }
            }
            if wrong.is_empty() {
                Ok(())
            } else {
                Err(wrong.join("\n  "))
            }
        },
    ));
    results.push(repo_test(
        "every_backticked_repo_path_in_the_docs_exists",
        Category::Regression,
        "needs a real 0-Core: it reads the docs and the tracked tree, which only a checkout has",
        || {
            // A DOC MUST NOT CITE A PATH THAT IS NOT THERE. Every backticked repo path in docs/
            // and AGENTS.md must name a tracked file or directory. A token is a path claim when
            // it holds a slash, or is a bare file name ending .rs .py .toml .sh or .md; it is
            // valid when some tracked path ends with it, so `paths.rs` and `integrity/mod.rs`
            // resolve the way a reader resolves them. Home and absolute paths, .git internals,
            // globs and placeholders are not repo claims. There is no exemption list: a doc that
            // cites a dead path is fixed, not excused. docs/public/ is generated from docs/.
            let root = std::path::Path::new(&home()).join("0-core");
            let out = Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(["ls-files", "-z"])
                .output()
                .map_err(|e| format!("cannot run git ls-files: {}", e))?;
            if !out.status.success() {
                return Err(format!(
                    "git ls-files failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            let tracked: Vec<String> = out
                .stdout
                .split(|b| *b == 0)
                .filter(|p| !p.is_empty())
                .map(|p| String::from_utf8_lossy(p).to_string())
                .collect();
            let mut known: Vec<String> = tracked.clone();
            for f in &tracked {
                let parts: Vec<&str> = f.split('/').collect();
                for i in 1..parts.len() {
                    known.push(format!("{}/", parts[..i].join("/")));
                }
            }
            known.sort();
            known.dedup();
            let docs: Vec<&String> = tracked
                .iter()
                .filter(|f| {
                    f.ends_with(".md")
                        && ((f.starts_with("docs/") && !f.starts_with("docs/public/"))
                            || f.as_str() == "AGENTS.md")
                })
                .collect();
            let exts = [".rs", ".py", ".toml", ".sh", ".md"];
            let mut claims = 0usize;
            let mut dead: Vec<String> = Vec::new();
            for doc in &docs {
                let text = std::fs::read_to_string(root.join(doc.as_str()))
                    .map_err(|e| format!("cannot read {}: {}", doc, e))?;
                for (n, line) in text.lines().enumerate() {
                    for (i, seg) in line.split('`').enumerate() {
                        if i % 2 == 0 {
                            continue;
                        }
                        let t = seg.trim_end_matches(|c| ".,:;)".contains(c));
                        if t.is_empty()
                            || t.contains(char::is_whitespace)
                            || ["~", "/", "http", "$", "-", ".git/"]
                                .iter()
                                .any(|p| t.starts_with(*p))
                            || t.contains(|c| "*<{|=".contains(c))
                        {
                            continue;
                        }
                        let p = t.split(':').next().unwrap_or("");
                        let p = p.strip_prefix("./").unwrap_or(p);
                        if !p.contains('/') && !exts.iter().any(|x| p.ends_with(*x)) {
                            continue;
                        }
                        claims += 1;
                        let tail = format!("/{}", p);
                        if !known.iter().any(|k| k.as_str() == p || k.ends_with(&tail)) {
                            dead.push(format!("{}:{}  `{}`", doc, n + 1, t));
                        }
                    }
                }
            }
            // READING NOTHING IS NOT A CLEAN RESULT.
            if docs.is_empty() || claims == 0 {
                return Err(format!(
                    "read {} docs and found {} path claims -- that is not a clean tree",
                    docs.len(),
                    claims
                ));
            }
            if dead.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "{} backticked path(s) in the docs name nothing tracked:\n  {}",
                    dead.len(),
                    dead.join("\n  ")
                ))
            }
        },
    ));
    results.push(repo_test(
        "fpatch_tests_pass",
        Category::Regression,
        "needs a real 0-Core: it runs zero/scripts/dev/test_fpatch.py, which only a checkout has",
        || {
            // INT-278: fpatch is the edit primitive, and its guards are only as good as the tests
            // that prove them. Each test works in its own temp directory and touches nothing else.
            // A missing test file, a missing python3, or a run of zero tests is a failure, not a
            // skip: reading nothing is not a clean result.
            let dev = std::path::Path::new(&home()).join("0-core/zero/scripts/dev");
            if !dev.join("test_fpatch.py").is_file() {
                return Err(format!(
                    "{} has no test_fpatch.py -- the fpatch tests are missing",
                    dev.display()
                ));
            }
            let out = Command::new("python3")
                .args(["-m", "unittest", "test_fpatch"])
                .current_dir(&dev)
                .env("FPATCH_COLOR", "0")
                .env("PYTHONDONTWRITEBYTECODE", "1")
                .env_remove("FPATCH_DEBUG")
                .output()
                .map_err(|e| format!("cannot run python3: {}", e))?;
            let err = String::from_utf8_lossy(&out.stderr);
            if !out.status.success() {
                return Err(format!("fpatch tests failed:\n{}", err.trim_end()));
            }
            if !err
                .lines()
                .any(|l| l.starts_with("Ran ") && !l.starts_with("Ran 0 "))
            {
                return Err(format!("fpatch tests ran nothing:\n{}", err.trim_end()));
            }
            Ok(())
        },
    ));
    results.push(repo_test(
        "the_tree_map_names_every_entry",
        Category::Regression,
        "needs a real 0-Core: it reads docs/TREE.md and the tree, which only a checkout has",
        || {
            // INT-267: WHERE A FILE GOES. docs/TREE.md names every entry at the repo root and in
            // zero/, one line of purpose each. This case fails on an entry the map does not name;
            // every_backticked_repo_path_in_the_docs_exists fails on a name the map holds that does
            // not exist. Untracked entries count (--others --exclude-standard), so a new directory
            // is caught before it is committed; ignored ones such as target/ do not.
            let root = std::path::Path::new(&home()).join("0-core");
            let out = Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
                .output()
                .map_err(|e| format!("cannot run git ls-files: {}", e))?;
            if !out.status.success() {
                return Err(format!(
                    "git ls-files failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            let mut entries: Vec<String> = Vec::new();
            for raw in out.stdout.split(|b| *b == 0).filter(|p| !p.is_empty()) {
                let p = String::from_utf8_lossy(raw).to_string();
                let parts: Vec<&str> = p.split('/').collect();
                if parts.len() == 1 {
                    entries.push(parts[0].to_string());
                    continue;
                }
                entries.push(format!("{}/", parts[0]));
                if parts[0] == "zero" {
                    if parts.len() == 2 {
                        entries.push(format!("zero/{}", parts[1]));
                    } else {
                        entries.push(format!("zero/{}/", parts[1]));
                    }
                }
            }
            entries.sort();
            entries.dedup();
            let map = std::fs::read_to_string(root.join("docs/TREE.md"))
                .map_err(|e| format!("cannot read docs/TREE.md -- the map is missing: {}", e))?;
            let named: Vec<&str> = map
                .split('`')
                .enumerate()
                .filter(|(i, _)| i % 2 == 1)
                .map(|(_, s)| s.trim())
                .collect();
            // READING NOTHING IS NOT A CLEAN RESULT.
            if entries.is_empty() || named.is_empty() {
                return Err(format!(
                    "read {} entries and {} names in the map -- that is not a clean tree",
                    entries.len(),
                    named.len()
                ));
            }
            let unnamed: Vec<&String> = entries
                .iter()
                .filter(|e| !named.contains(&e.as_str()))
                .collect();
            if unnamed.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "{} entr{} the map does not name -- add each to docs/TREE.md with one line of purpose, or move it:\n  {}",
                    unnamed.len(),
                    if unnamed.len() == 1 { "y" } else { "ies" },
                    unnamed
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join("\n  ")
                ))
            }
        },
    ));
    results.push(test(
        "deadwood_strict_gate_passes",
        Category::Regression,
        || {
            // INT-195 gate 6: the architectural invariant runs somewhere it is SEEN, not only
            // when someone types it. Asserts the PUBLIC CONTRACT -- a clean tree exits zero
            // under --strict -- and deliberately does NOT arrange a finding by mutating source,
            // because a suite that edits the tree to create a failure can leave it dirty when
            // it fails. The failing direction is covered by the deadwood crate's fixture tests,
            // where constructing a finding is cheap and self-contained.
            let out = run_deadwood(&["--strict"])?;
            if out.status.success() {
                return Ok(());
            }
            let flagged: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| l.contains("[HIGH]") || l.contains("flagged"))
                .map(|l| l.trim().to_string())
                .collect();
            Err(format!(
                "zero-deadwood --strict exited {:?}: {}",
                out.status.code(),
                if flagged.is_empty() {
                    String::from_utf8_lossy(&out.stderr).trim().to_string()
                } else {
                    flagged.join(" | ")
                }
            ))
        },
    ));
    results.push(test("nsh_binary_exists", Category::Regression, || {
        expect_contains(&run_fsh("which nsh")?, "nsh")
    }));
    results.push(test("intents_future_exists", Category::Regression, || {
        let home = fixture_home()?;
        expect_contains(
            &run_fsh_env(
                "ls ~/0-core/zero/intents/future",
                &[("HOME", home.as_str())],
            )?,
            ".md",
        )
    }));
    results.push(test("pipe_multiline_output", Category::Pipes, || {
        let out = run_fsh("printf 'a\nb\nc\n' | wc -l")?;
        expect_eq(out.trim(), "3")
    }));
    results.push(test("redirect_not_crash", Category::Regression, || {
        run_fsh("echo test > /tmp/nsh_test_redirect.txt")?;
        expect_contains(&run_fsh("cat /tmp/nsh_test_redirect.txt")?, "test")
    }));
    // ---- INT-172: REPL tests. These drive a real pty, NOT `fsh -c`.
    // Every one of them PASSES on `fsh -c` even against a broken shell -- which
    // is exactly why nsh-test was 83/83 green for three months while the shell we
    // type into was turning pipelines into filenames.
    results.push(test(
        "repl_pipe_control_no_redirect",
        Category::Repl,
        || {
            let out = repl::run_repl("echo hello | grep -c hello")?;
            expect_eq(out.first().map(|s| s.as_str()).unwrap_or("(nothing)"), "1")
        },
    ));
    // INT-201 / 3c5220be: the `-c` handler used to report success it had not earned, three ways.
    // These are the regression cases that could not exist until run_fsh_status did, because run_fsh
    // collapses every non-zero exit into Err and a case cannot say "expect 143" through it.
    results.push(test(
        "dashc_exit_code_is_passed_through",
        Category::Regression,
        || {
            let (_, _, code) = run_fsh_status("exit 3")?;
            expect_exit(code, 3)
        },
    ));
    results.push(test(
        "dashc_signalled_child_is_not_a_success",
        Category::Regression,
        || {
            // ⭐ THE ONE THAT MATTERS. status.code() is None when a process dies by signal, and the
            // old unwrap_or(0) turned that into EXIT 0 -- an interrupted command telling its caller
            // it had finished fine. 143 is 128+15. The child signals ITSELF so fsh survives to
            // report it; signalling fsh instead measures the wrong process, which cost one probe.
            // ⚠️ THE PREMISE MOVED WHEN `-c` STOPPED DELEGATING TO sh (INT-201 gate 4). `$$` used
            // to be the pid of the sh fsh spawned, so `kill -TERM $$` killed a CHILD and fsh
            // reported 143 for it. Routed through fsh, `$$` is fsh's OWN pid -- the shell kills
            // itself and there is no child left to report on, which is a different thing entirely.
            // The case now spawns a real child explicitly so it still measures what it is named for.
            let (_, _, code) = run_fsh_status("sh -c 'kill -TERM $$'")?;
            expect_exit(code, 143)
        },
    ));
    results.push(test(
        "dashc_missing_operand_is_a_usage_error",
        Category::Regression,
        || {
            // ⚠️ SPAWNS DIRECTLY RATHER THAN THROUGH THE HELPER, because run_fsh_status always passes
            // an operand and this case is about there being none. `-c ""` is a present-but-empty
            // operand and exits 0, which is a different thing entirely.
            let fsh = std::env::var("NSH_BIN").unwrap_or_else(|_| "nsh".to_string());
            let out = Command::new(&fsh)
                .env("NSH_KEEP_CWD", "1")
                .env("ZERO_STATE_DB", repl::case_db_path())
                .arg("-c")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| e.to_string())?;
            expect_exit(out.status.code(), 2)?;
            expect_contains(
                &String::from_utf8_lossy(&out.stderr),
                "requires an argument",
            )
        },
    ));
    // A FIXTURE CANNOT FIX THIS ONE EITHER. It asserts that the ordinary shell starts in the
    // REPO HOME -- a default that only exists when the repo does. Pointed at a fixture it would
    // pass against three stub files and prove nothing about the behaviour it guards.
    results.push(repo_test(
        "repl_206_zero_home_is_still_the_default",
        Category::Repl,
        "needs a real 0-Core: the repo-home default only exists when the repo does",
        || {
            // INT-206 GUARDIAN. The harness sets NSH_KEEP_CWD for every other case so that a case which
            // writes a file cannot write it into the repository. That is the right default, and it has a
            // cost: every other case then runs a shell configuration nobody uses interactively.
            //
            // This case buys that back. It passes "0" to get the ORDINARY shell -- the one that starts in
            // the repo home on purpose -- and asserts that
            // behaviour is intact. So what daily use actually gets is covered by a case that says what it
            // is testing, rather than left uncovered because every other case quietly opted out of it.
            //
            // "0" rather than removing the variable: keep_launch_cwd reads it as v != "0", so the string
            // is the off switch and no env_remove is needed.
            let (out, _) = repl::run_repl_lines_status(&["pwd"], &[("NSH_KEEP_CWD", "0")])?;
            expect_contains(&out.join("\n"), "0-core")
        },
    ));
    results.push(repo_test(
        "repl_start_directory_ignores_remembered_last_dir",
        Category::Repl,
        "needs a real 0-Core: the start directory it asserts only exists when the repo does",
        || {
            // THE STARTUP DIRECTORY MUST NOT DEPEND ON REMEMBERED STATE.
            //
            // repl_206 cannot see this. Every case gets a FRESH database (repl.rs, INT-204), so its
            // shell never has a last_dir to restore -- it passes on a binary that sends every real
            // terminal to wherever the previous session exited. Found 2026-09-24, when a terminal
            // opened in ~/.local/state/zero because the flip session had ended there.
            //
            // So this case PLANTS one. Session 1 lets nsh create its own schema (NSH_KEEP_CWD is
            // the harness default, so it saves no last_dir). The row is then written into THAT
            // database, and session 2 starts the ordinary shell against it. The planted directory
            // is not /tmp on purpose: every case launches from /tmp, so a red result pointing at
            // /tmp could not tell a restore from a missing start move.
            let db = repl::case_db_path();
            let planted = format!("{}/planted-last-dir", repl::case_db_dir());
            std::fs::create_dir_all(&planted).map_err(|e| format!("create {}: {}", planted, e))?;

            repl::run_repl_lines_status(&["true"], &[("ZERO_STATE_DB", db.as_str())])?;
            let conn =
                rusqlite::Connection::open(&db).map_err(|e| format!("open {}: {}", db, e))?;
            conn.execute(
                "INSERT OR REPLACE INTO session_state (key, value) VALUES ('last_dir', ?1)",
                rusqlite::params![planted],
            )
            .map_err(|e| format!("seed last_dir: {}", e))?;
            drop(conn);

            let (out, _) = repl::run_repl_lines_status(
                &["pwd"],
                &[("NSH_KEEP_CWD", "0"), ("ZERO_STATE_DB", db.as_str())],
            )?;
            let got = out.join("\n");
            if got.contains("planted-last-dir") {
                return Err(format!(
                    "restored the remembered last_dir instead of starting in 0-core: {:?}",
                    got
                ));
            }
            expect_contains(&got, "0-core")
        },
    ));
    // ---- INT-262: persist / unpersist, RED FIRST --------------------------------------------
    // Every case below runs on its OWN case database (ZERO_STATE_DB, applied after the harness
    // default, so two sessions in one case share it). The live state.db is never opened.
    // Each was written to fail on the binary that existed when it landed, for the reason its
    // gate names. persist_262_stores_its_own_value replaced the pin on the :492 indirection
    // when fix 3 removed it, ruled 2026-10-05.
    results.push(test(
        "persist_262_bare_empty_says_so",
        Category::Repl,
        || {
            let db = repl::case_db_path();
            let (out, _) =
                repl::run_repl_lines_status(&["persist"], &[("ZERO_STATE_DB", db.as_str())])?;
            let got = out.join("\n");
            if got.contains("command not found") {
                return Err(format!("bare persist is not a command: {:?}", got));
            }
            expect_contains(&got, "nothing is persisted")
        },
    ));
    results.push(test(
        "persist_262_bare_lists_the_store",
        Category::Repl,
        || {
            let db = repl::case_db_path();
            repl::run_repl_lines_status(&["true"], &[("ZERO_STATE_DB", db.as_str())])?;
            let conn =
                rusqlite::Connection::open(&db).map_err(|e| format!("open {}: {}", db, e))?;
            conn.execute(
                "INSERT OR REPLACE INTO shell_persist (key, value) VALUES ('L262', 'listed')",
                [],
            )
            .map_err(|e| format!("seed shell_persist: {}", e))?;
            drop(conn);
            let (out, _) =
                repl::run_repl_lines_status(&["persist"], &[("ZERO_STATE_DB", db.as_str())])?;
            expect_contains(&out.join("\n"), "L262=listed")
        },
    ));
    results.push(test(
        "unpersist_262_removes_for_a_fresh_shell",
        Category::Repl,
        || {
            let db = repl::case_db_path();
            let env = [("ZERO_STATE_DB", db.as_str())];
            repl::run_repl_lines_status(&["export U262=one", "persist U262"], &env)?;
            let rows = || -> Result<i64, String> {
                let c =
                    rusqlite::Connection::open(&db).map_err(|e| format!("open {}: {}", db, e))?;
                c.query_row(
                    "SELECT COUNT(*) FROM shell_persist WHERE key = 'U262'",
                    [],
                    |r| r.get(0),
                )
                .map_err(|e| format!("count U262: {}", e))
            };
            if rows()? != 1 {
                return Err("setup: persist U262 did not store a row".to_string());
            }
            repl::run_repl_lines_status(&["unpersist U262"], &env)?;
            let left = rows()?;
            if left != 0 {
                return Err(format!(
                    "unpersist U262 left {} row(s) in shell_persist",
                    left
                ));
            }
            let (out, _) = repl::run_repl_lines_status(&["echo \"<$U262>\""], &env)?;
            expect_contains(&out.join("\n"), "<>")
        },
    ));
    results.push(test(
        "unpersist_262_not_stored_says_so",
        Category::Repl,
        || {
            let db = repl::case_db_path();
            let (out, _) = repl::run_repl_lines_status(
                &["unpersist N262"],
                &[("ZERO_STATE_DB", db.as_str())],
            )?;
            let got = out.join("\n");
            if got.contains("command not found") {
                return Err(format!("unpersist is not a command: {:?}", got));
            }
            expect_contains(&got, "not persisted")
        },
    ));
    results.push(test(
        "persist_262_failed_insert_is_reported",
        Category::Repl,
        || {
            let db = repl::case_db_path();
            let env = [("ZERO_STATE_DB", db.as_str())];
            repl::run_repl_lines_status(&["true"], &env)?;
            let conn =
                rusqlite::Connection::open(&db).map_err(|e| format!("open {}: {}", db, e))?;
            conn.execute_batch(
                "CREATE TRIGGER t262 BEFORE INSERT ON shell_persist \
             BEGIN SELECT RAISE(ABORT, 'forced'); END;",
            )
            .map_err(|e| format!("plant trigger: {}", e))?;
            drop(conn);
            let (out, status) =
                repl::run_repl_lines_status(&["export F262=x", "persist F262"], &env)?;
            let got = out.join("\n");
            if got.contains("persisted across sessions") {
                return Err(format!(
                    "a refused INSERT was reported as persisted: {:?}",
                    got
                ));
            }
            match status {
                Some(code) if code != 0 => Ok(()),
                other => Err(format!(
                    "a failed persist left status {:?}, expected non-zero",
                    other
                )),
            }
        },
    ));
    results.push(test(
        "persist_262_dashc_without_a_table_is_honest",
        Category::Regression,
        || {
            // The -c door never runs the REPL's startup restore, which is the only place that
            // creates shell_persist. Honest outcomes: the row is stored, or the command fails.
            let db = repl::case_db_path();
            let res = run_fsh_env(
                "export C262=x; persist C262",
                &[("ZERO_STATE_DB", db.as_str())],
            );
            let stored: Option<String> = rusqlite::Connection::open(&db).ok().and_then(|c| {
                c.query_row(
                    "SELECT value FROM shell_persist WHERE key = 'C262'",
                    [],
                    |r| r.get(0),
                )
                .ok()
            });
            match (stored.as_deref(), &res) {
                (Some("x"), _) => Ok(()),
                (_, Err(_)) => Ok(()),
                (_, Ok(out)) => Err(format!(
                    "-c stored nothing (row {:?}) and exited 0; output {:?}",
                    stored, out
                )),
            }
        },
    ));
    results.push(test(
        "persist_262_success_leaves_status_zero",
        Category::Repl,
        || {
            let db = repl::case_db_path();
            let (_, status) = repl::run_repl_lines_status(
                &["export S262=y", "false", "persist S262"],
                &[("ZERO_STATE_DB", db.as_str())],
            )?;
            if status != Some(0) {
                return Err(format!(
                    "persist left status {:?}, expected Some(0)",
                    status
                ));
            }
            Ok(())
        },
    ));
    results.push(test(
        "persist_262_stores_its_own_value",
        Category::Repl,
        || {
            // INT-262 fix 3 removed the engine.rs:492 indirection (ruled remove, 2026-10-05).
            // P262's own environment value is Q262, so Q262 -- not hidden -- must be stored.
            let db = repl::case_db_path();
            repl::run_repl_lines_status(
                &["Q262=hidden", "persist P262"],
                &[("ZERO_STATE_DB", db.as_str()), ("P262", "Q262")],
            )?;
            let c = rusqlite::Connection::open(&db).map_err(|e| format!("open {}: {}", db, e))?;
            let stored: Option<String> = c
                .query_row(
                    "SELECT value FROM shell_persist WHERE key = 'P262'",
                    [],
                    |r| r.get(0),
                )
                .ok();
            if stored.as_deref() != Some("Q262") {
                return Err(format!(
                    "persist P262 must store its own value Q262, but stored {:?}",
                    stored
                ));
            }
            Ok(())
        },
    ));
    results.push(test(
        "restore_262_empty_store_is_silent",
        Category::Repl,
        || {
            // A readable, empty shell_persist says nothing at startup. The banner must not be
            // empty either: a capture that saw nothing would pass this case without looking.
            let db = repl::case_db_path();
            let (_, banner) =
                repl::run_repl_lines_banner(&["true"], &[("ZERO_STATE_DB", db.as_str())])?;
            if banner.trim().is_empty() {
                return Err("the banner capture is empty -- the helper saw nothing".to_string());
            }
            if banner.contains("stored variables could not be read") {
                return Err(format!(
                    "an empty store was reported at startup: {:?}",
                    banner
                ));
            }
            Ok(())
        },
    ));
    results.push(test(
        "restore_262_unreadable_store_is_reported",
        Category::Repl,
        || {
            // A shell_persist without key and value columns, planted before nsh first opens the
            // database: CREATE TABLE IF NOT EXISTS leaves it alone and the restore's SELECT
            // cannot prepare. That failure must reach the screen before the first prompt.
            let db = repl::case_db_path();
            let conn =
                rusqlite::Connection::open(&db).map_err(|e| format!("open {}: {}", db, e))?;
            conn.execute_batch("CREATE TABLE shell_persist (x INTEGER);")
                .map_err(|e| format!("plant broken table: {}", e))?;
            drop(conn);
            let (_, banner) =
                repl::run_repl_lines_banner(&["true"], &[("ZERO_STATE_DB", db.as_str())])?;
            expect_contains(&banner, "stored variables could not be read")
        },
    ));
    results.push(test(
        "discover_262_where_and_explain_know_persist",
        Category::Vocabulary,
        || {
            // INT-262: persist and unpersist are builtins, so the two "what is this word" answers
            // must say so, as they do for unset. Both used to report not found.
            for word in ["persist", "unpersist"] {
                let out = run_fsh(&format!("where {}", word))?;
                if !out.contains("native nsh") {
                    return Err(format!("where {} does not name a builtin: {:?}", word, out));
                }
                let out = run_fsh(&format!("explain {}", word))?;
                if !out.contains("native nsh command") {
                    return Err(format!(
                        "explain {} does not name a builtin: {:?}",
                        word, out
                    ));
                }
            }
            Ok(())
        },
    ));
    results.push(test(
        "repl_241_pwd_follows_the_working_directory",
        Category::Repl,
        || {
            // INT-241. `set_current_dir` moves the process; POSIX says the shell must also set
            // PWD, and nsh never did -- so PWD held whatever the bash that exec'd it exported at
            // login, for the whole life of the shell.
            //
            // ⚠️ THREE THINGS THIS CASE DOES DELIBERATELY, each because the obvious version of the
            // test PASSES ON THE BROKEN SHELL:
            //
            // 1. THE REPL DOOR, not -c. `-c` delegates to sh, which inherits a correct PWD from
            //    its caller, so it agreed with bash the whole time. A -c test proves nothing here.
            // 2. cd SOMEWHERE THE LAUNCH DIRECTORY IS NOT. Measured 2026-09-23: on the broken
            //    binary `cd ~/0-core` then `echo $PWD` AGREED, because the stale value happened to
            //    equal the destination. /tmp is where the accident stops.
            // 3. A CHILD PROCESS reads it. printenv is a separate process, and a child reading a
            //    stale PWD is what sent the harness to a binary that does not exist -- the failure
            //    this intent was filed on.
            // ONE echo, not two: measured 2026-09-23, run_repl_lines_status returns the output of
            // the LAST command only, so a case spread over two echo lines silently asserts on half
            // of what it thinks it is checking.
            let (out, _) = repl::run_repl_lines_status(
                &[
                    "cd /tmp",
                    "echo AGREE pwd=$(pwd) var=$PWD child=$(printenv PWD)",
                ],
                &[],
            )?;
            expect_contains(&out.join("\n"), "AGREE pwd=/tmp var=/tmp child=/tmp")
        },
    ));
    results.push(test(
        "repl_230_absent_ledger_refuses_not_empties",
        Category::Repl,
        || {
            // INT-230 G5: RUNTIME proof, not a source-text check. The real REPL is
            // driven with 0-Core absent, and the doors that read it must REFUSE
            // rather than print a successful-looking empty result (INT-227).
            //
            // Absence is TWO variables, not one. intents_dir is HOME/0-core/zero/
            // intents, so HOME alone hides the ledger -- but state_home reads
            // XDG_STATE_HOME independently, so health, focus.toml and the rest stayed
            // present under a bare HOME redirect (measured 2026-09-05). Both move.
            // ZERO_STATE_DB is already per-case, set by the harness.
            //
            // Clean BEFORE, not after: after does not run when the case fails, which
            // is exactly when junk is left.
            let root = format!("/tmp/nsh-test-230-{}", std::process::id());
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).map_err(|e| format!("mkdir {}: {}", root, e))?;
            // One door per session. On the first run of this case the harness
            // handed back only the LAST line's capture (the zstats refusal), so a
            // single four-line session cannot assert the first three.
            let env = [("HOME", root.as_str()), ("XDG_STATE_HOME", root.as_str())];
            let (alive, _) = repl::run_repl_lines_status(&["echo alive"], &env)?;
            expect_contains(&alive.join("\n"), "alive")?;
            let (tools, _) = repl::run_repl_lines_status(&["tools"], &env)?;
            expect_contains(
                &tools.join("\n"),
                "tools: needs 0-Core, which is not present",
            )?;
            let (pick, _) = repl::run_repl_lines_status(&["pick intent"], &env)?;
            expect_contains(
                &pick.join("\n"),
                "pick intent: needs 0-Core, which is not present",
            )?;
            let (zstats, _) = repl::run_repl_lines_status(&["zstats intents"], &env)?;
            expect_contains(
                &zstats.join("\n"),
                "zstats intents: needs 0-Core, which is not present",
            )
        },
    ));
    results.push(test(
        "repl_205_builtin_first_stage_of_pipe",
        Category::Repl,
        || {
            // INT-205: `spine` has NO BINARY ON PATH, so this stage can only be served by the builtin.
            // spawn_pipeline built every stage with Command::new and handed the name to the operating
            // system, so the line died with "spine: No such file or directory" while the router had
            // just claimed it.
            //
            // THE CHOICE OF COMMAND IS THE TEST. A shadowed name -- cat, ps, grep -- passes either way,
            // because a real binary exists and spawning it is correct. Only a builtin with nothing
            // behind it can fail, so only that shape proves anything.
            //
            // AND IT MUST DRIVE THE REPL. Through `fsh -c` the whole string goes to sh, which reports
            // its own "command not found" and never reaches the pipeline executor at all -- so a case
            // written against that door would pass forever without testing this.
            let out = repl::run_repl("spine parse echo hi | cat")?;
            expect_contains(&out.join("\n"), "redirects")
        },
    ));
    results.push(test("repl_stderr_null_then_pipe", Category::Repl, || {
        // The simplest possible case. Printed `hello` on gen 395.
        let out = repl::run_repl("echo hello 2>/dev/null | grep -c hello")?;
        expect_eq(out.first().map(|s| s.as_str()).unwrap_or("(nothing)"), "1")
    }));
    results.push(test("repl_2to1_then_pipe", Category::Repl, || {
        let out = repl::run_repl("echo hello 2>&1 | grep -c hello")?;
        expect_eq(out.first().map(|s| s.as_str()).unwrap_or("(nothing)"), "1")
    }));
    results.push(test(
        "repl_stdout_redirect_with_2to1",
        Category::Repl,
        || {
            // `cmd > f 2>&1` wrote NO FILE. The code that would have written both
            // streams existed and was UNREACHABLE -- detect_redirect intercepted the
            // line before it could ever run.
            let _ = std::fs::remove_file("/tmp/nsh_test_g7out.txt");
            let _ = repl::run_repl("ls /tmp/nsh_test_nope /tmp > /tmp/nsh_test_g7out.txt 2>&1")?;
            let body = std::fs::read_to_string("/tmp/nsh_test_g7out.txt")
                .map_err(|e| format!("no file created: {}", e))?;
            if body.contains("No such file") {
                Ok(())
            } else {
                Err(format!(
                    "file exists ({} bytes) but stderr was not merged",
                    body.len()
                ))
            }
        },
    ));
    results.push(test(
        "repl_pipeline_never_becomes_a_filename",
        Category::Repl,
        || {
            // INT-172's signature. `2>FILE | cmd` put the WHOLE remainder into
            // File::create(), because a pipeline is a legal Linux filename. A real one
            // was found in /tmp dated 2026-07-12 13:00:25, left by actual work:
            //   'pi.err | python3 -c "import sys,json; ...print(len(d),paths)"'
            // The python never ran. This test is that fossil, made into an assertion.
            // Clear leftovers FIRST. A previous run against a broken shell leaves
            // exactly the file this test looks for, so without this the test fails
            // against a FIXED shell. Found 2026-07-17 by the red/green run itself:
            // green reported "the pipeline became a filename" on a binary that had
            // stopped doing that an hour earlier.
            for e in std::fs::read_dir("/tmp")
                .map_err(|e| e.to_string())?
                .flatten()
            {
                if e.file_name()
                    .to_string_lossy()
                    .starts_with("nsh_test_g7err")
                {
                    let _ = std::fs::remove_file(e.path());
                }
            }
            let _ = repl::run_repl("echo hello 2>/tmp/nsh_test_g7err | grep -c hello")?;
            let junk: Vec<String> = std::fs::read_dir("/tmp")
                .map_err(|e| e.to_string())?
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.starts_with("nsh_test_g7err") && n.contains('|'))
                .collect();
            if junk.is_empty() {
                Ok(())
            } else {
                Err(format!("the pipeline became a filename: {:?}", junk))
            }
        },
    ));
    results.push(test("nested_subshell", Category::Regression, || {
        expect_eq(&run_fsh("echo $(echo $(echo deep))")?, "deep")
    }));
    results.push(test("multiword_var", Category::Regression, || {
        expect_contains(&run_fsh("A=hello; echo $A world")?, "hello world")
    }));
    results.push(test("tilde_in_quoted_string", Category::Tilde, || {
        // ~ inside double quotes should expand
        let out = run_fsh("echo $HOME")?;
        expect_contains(&out, &home())
    }));
    results.push(test("exit_code_success", Category::Regression, || {
        run_fsh("true")?;
        Ok(())
    }));

    // --- PHASE 2: INT-298/299 specific regression tests ---
    results.push(test(
        "regression_nsh_c_inside_nsh",
        Category::Regression,
        || {
            // INT-299: fsh -c works inside fsh
            expect_eq(&run_fsh("echo hello")?, "hello")
        },
    ));
    results.push(test(
        "regression_sigpipe_head",
        Category::Regression,
        || {
            // INT-299: SIGPIPE does not crash on pipe to head
            let out = run_fsh("seq 1 100 | head -3")?;
            expect_eq(&out, "1\n2\n3")
        },
    ));
    results.push(test(
        "regression_awk_passthrough",
        Category::Regression,
        || {
            // INT-299: awk passes through correctly
            expect_eq(&run_fsh("echo 'a b c' | awk '{print $2}'")?, "b")
        },
    ));
    results.push(test(
        "regression_grep_passthrough",
        Category::Regression,
        || {
            // INT-299: grep passes through correctly
            expect_eq(&run_fsh("printf 'foo\nbar\n' | grep foo")?, "foo")
        },
    ));
    results.push(test(
        "regression_pipe_quote_aware",
        Category::Regression,
        || {
            // INT-299: pipe detection with quote awareness
            expect_eq(&run_fsh("echo 'a|b' | cat")?, "a|b")
        },
    ));
    results.push(test(
        "regression_heredoc_single_quote",
        Category::Heredoc,
        || {
            // INT-299: heredoc with single-quoted delimiter
            let out = run_fsh("cat << 'MARKER'\nhello $USER\nMARKER")?;
            // single-quoted heredoc should NOT expand $USER
            expect_eq(&out, "hello $USER")
        },
    ));

    // ---- INT-171 gate 3: the six INT-143 regressions, as REPL tests.
    // Each of these six bugs lived in fsh's INTERACTIVE dispatch. NONE is
    // observable through `fsh -c`, which hands the whole line to /bin/sh --
    // proven 2026-07-19: `fsh -c 'nosuchcmd && echo X'` returns sh's own
    // "command not found", never touching fsh's dispatch. So a run_fsh() test
    // for any of these would pass on a completely broken fsh. They MUST drive
    // the REPL. Commit hashes are the INT-143 fixes each one guards.
    results.push(test("repl_143_redirect_runs_once", Category::Repl, || {
        // bfe25bc9: `cmd > file` ran every external command TWICE. No visible
        // stdout tell -- the second run's effect is the evidence. A fresh mkdir
        // succeeds once; a second run errors "File exists" to stderr, so merge
        // it into the file with 2>&1 (the REPL handles that since INT-172).
        // Clear state FIRST: a failed run never reaches cleanup, and a broken
        // shell leaves exactly the dir/file a fixed shell must start without.
        let _ = std::fs::remove_file("/tmp/nsh_test_143once.txt");
        let _ = std::fs::remove_dir("/tmp/nsh_test_143once_dir");
        let _ = repl::run_repl("mkdir /tmp/nsh_test_143once_dir > /tmp/nsh_test_143once.txt 2>&1")?;
        if !std::path::Path::new("/tmp/nsh_test_143once_dir").is_dir() {
            return Err("mkdir did not run: dir absent".to_string());
        }
        let body = std::fs::read_to_string("/tmp/nsh_test_143once.txt")
            .map_err(|e| format!("no redirect file: {}", e))?;
        if body.trim().is_empty() {
            Ok(())
        } else {
            Err(format!(
                "command ran twice -- stderr captured: {:?}",
                body.trim()
            ))
        }
    }));
    results.push(test("repl_143_typo_and_no_leak", Category::Repl, || {
        // 968c7be5: a typo followed by `&&` reported SUCCESS and RAN the next
        // command. `mkae build && rm -rf dist` would have deleted dist. The
        // not-found must break the chain, so LEAKED143 must NOT appear.
        let out = repl::run_repl("nosuchcmd143zzz && echo LEAKED143")?;
        if out.iter().any(|l| l.contains("LEAKED143")) {
            Err(format!("&& ran after a failed command: {:?}", out))
        } else {
            Ok(())
        }
    }));
    results.push(test("repl_143_python3_keeps_flags", Category::Repl, || {
        // c5086945: fsh's python3 arm joined all args into `python3 -c "<args>"`,
        // so `--version` was evaluated as Python source -> NameError. The arm was
        // deleted; python3 now falls through to run_external untouched.
        let out = repl::run_repl("python3 --version")?;
        if out.iter().any(|l| l.contains("Python 3")) {
            Ok(())
        } else {
            Err(format!(
                "python3 --version did not report a version: {:?}",
                out
            ))
        }
    }));
    results.push(test("repl_143_bash_runs_script", Category::Repl, || {
        // 5cba096d: `bash script.sh` dropped into interactive bash and the script
        // never ran. Guarded to `if args.is_empty()` -- with args it falls through
        // to the real bash.
        std::fs::write("/tmp/nsh_test_143.sh", "echo MARKER143RAN\n").map_err(|e| e.to_string())?;
        let out = repl::run_repl("bash /tmp/nsh_test_143.sh")?;
        if out.iter().any(|l| l.contains("MARKER143RAN")) {
            Ok(())
        } else {
            Err(format!("bash did not run the script: {:?}", out))
        }
    }));
    results.push(test("repl_143_env_passthrough", Category::Repl, || {
        // 56aa0798: `env VAR=x cmd` printed fsh's environment table instead of
        // running cmd. Guarded to `if args.is_empty()` -- with args it is coreutils
        // env. Assert cmd RAN and the table did NOT print.
        let out = repl::run_repl("env G3TEST143=xyz echo env_ran143")?;
        let ran = out.iter().any(|l| l.contains("env_ran143"));
        let table = out.iter().any(|l| l.contains("Environment"));
        if ran && !table {
            Ok(())
        } else {
            Err(format!(
                "env did not pass through (ran={}, table={}): {:?}",
                ran, table, out
            ))
        }
    }));
    // ---- INT-173: interactive behaviours invisible through `fsh -c`. Neither
    // fsh-builtin dispatch nor alias expansion is exercised by any -c test (sh has
    // neither fsh's builtins nor fsh's aliases), and neither was covered by the
    // 172/171 REPL tests. These two close that gap. Probed on gen 402 before writing.
    results.push(test("repl_173_builtin_dispatch", Category::Repl, || {
        // fsh's own `type` builtin prints "shell builtin / handled natively by nsh".
        // sh's `type` prints nothing like it, so this output PROVES fsh dispatched
        // its builtin -- invisible through `-c`, which would run sh's `type`.
        let out = repl::run_repl("type pwd")?;
        let joined = out.join("\n");
        if joined.contains("shell builtin") {
            Ok(())
        } else {
            Err(format!(
                "nsh builtin dispatch not seen (expected 'shell builtin'): {joined:?}"
            ))
        }
    }));
    results.push(test(
        "repl_174_single_quote_no_subshell",
        Category::Repl,
        || {
            // INT-174: single quotes suppress $() ; double quotes still expand it.
            // Both directions in one test -- fails if single-quoted expands OR if
            // double-quoted stops expanding. Uses `echo INNER174` for determinism.
            let lit = repl::run_repl("echo '$(echo INNER174)'")?.join("\n");
            let exp = repl::run_repl("echo \"$(echo INNER174)\"")?.join("\n");
            if !lit.contains("$(echo INNER174)") {
                return Err(format!(
                    "single-quoted $() expanded (should be literal): {lit:?}"
                ));
            }
            if !exp.contains("INNER174") || exp.contains("$(echo") {
                return Err(format!(
                    "double-quoted $() did not expand (regression): {exp:?}"
                ));
            }
            Ok(())
        },
    ));
    results.push(test(
        "repl_173_alias_expands_at_prompt",
        Category::Repl,
        || {
            // Aliases are SESSION-SCOPED in fsh: set + use must share ONE line. This is
            // fsh's REPL alias path -- `-c` hands to sh, which does not expand aliases in
            // non-interactive mode and has none of fsh's aliases anyway.
            let out = repl::run_repl("alias grtxyz='echo ALIAS_OK_173'; grtxyz")?;
            let joined = out.join("\n");
            if joined.contains("ALIAS_OK_173") {
                Ok(())
            } else {
                Err(format!("alias did not expand at the prompt: {joined:?}"))
            }
        },
    ));
    results.push(test(
        "repl_193_expansion_happens_exactly_once",
        Category::Repl,
        || {
            // THE INVARIANT. The original reproducer: with two owners this printed the
            // marker TWICE, because the prompt expanded once and the executor expanded
            // again from an empty guard. One owner means one expansion.
            let out = repl::run_repl("alias echo='echo MARK193'; echo")?;
            if out.iter().any(|l| l.contains("MARK193 MARK193")) {
                return Err(format!("alias expanded twice: {out:?}"));
            }
            let seen = out.iter().any(|l| {
                l.contains("MARK193") && !l.trim_start().starts_with('[') && !l.contains("alias")
            });
            if seen {
                Ok(())
            } else {
                Err(format!("marker never printed at all: {out:?}"))
            }
        },
    ));
    results.push(test(
        "repl_169_alias_body_reaches_the_expansion_pipeline",
        Category::Repl,
        || {
            // INT-169 blocker 6: THE ORDERING INVARIANT. expand_aliases used to run LAST, after
            // vars, substitutions and globs, so an alias BODY was a separate unexpanded language
            // fragment -- `alias t='echo [$HOME]'; t` printed [$HOME] literally. Measured on the
            // deployed shell before the fix. The body now enters the same pipeline as typed text.
            //
            // The glob fixture is CREATED HERE rather than assuming a repo file, so the test does
            // not depend on which directory the harness runs in.
            let out = repl::run_repl(
                "touch /tmp/zz-glob-169.md; alias zzvar='echo [$HOME]'; zzvar; \
                 alias zzsub='echo [$(echo INNER)]'; zzsub; \
                 alias zzglob='echo /tmp/zz-glob-169*.md'; zzglob",
            )?;
            let joined = out.join("\n");
            // ⚠️ ASSERT ON A STANDALONE OUTPUT LINE, not anywhere in the transcript. The harness
            // captures the ECHOED INPUT too, so `alias zzvar='echo [$HOME]'` puts the literal
            // `$HOME` in the text whatever the shell does -- a `contains` check could never pass.
            // Unexpanded output appears as a line that IS the source form; expanded output does
            // not. The definition line trims to `alias zzvar='...'`, which never equals it.
            let has_line = |want: &str| out.iter().any(|l| l.trim() == want);
            if has_line("[$HOME]") {
                return Err(format!("alias body skipped variable expansion: {joined:?}"));
            }
            if has_line("[$(echo INNER)]") || !joined.contains("[INNER]") {
                return Err(format!(
                    "alias body skipped command substitution: {joined:?}"
                ));
            }
            // BOTH halves: the resolved path present AND the pattern gone. Presence alone could
            // be a coincidence; absence of the star is what proves the glob was expanded.
            if has_line("/tmp/zz-glob-169*.md") || !joined.contains("/tmp/zz-glob-169.md") {
                return Err(format!("alias body skipped glob expansion: {joined:?}"));
            }
            Ok(())
        },
    ));
    results.push(test(
        "repl_169_alias_reordering_kept_the_raw_text_boundary",
        Category::Repl,
        || {
            // The OTHER half of the contract, and the reason this is a separate test. INT-193
            // made alias expansion work on RAW TEXT so a quoted remainder survives; moving the
            // call earlier must not quietly abandon that.
            //
            // ⚠️ `printf %s.` with the DOT, matching the INT-193 tests beside this one, and the
            // first draft here proved why: bare `printf %s` emits no trailing newline, so the
            // PROMPT lands on the same line and a whole-line equality check fails on correct
            // output. The dot makes a split remainder read `a.b.` and a preserved one `a b.`,
            // which is unambiguous regardless of what follows on the line.
            let out = repl::run_repl("alias zzq='printf %s.'; zzq \"a b\"")?;
            let joined = out.join("\n");
            if joined.contains("a b.") && !joined.contains("a.b.") {
                Ok(())
            } else {
                Err(format!(
                    "alias expansion split a quoted remainder: {joined:?}"
                ))
            }
        },
    ));
    // ⚠️ THE SHAPE FOUR EXISTING REDIRECT TESTS MISS. repl_pipe_control_no_redirect,
    // repl_stdout_redirect_with_2to1, repl_pipeline_never_becomes_a_filename and
    // repl_143_redirect_runs_once were all green while `echo hi | cat > f` wrote the literal text
    // `hi | cat` into the file -- because none of them starts a redirected pipeline with a BUILTIN.
    // The builtin probe matched `echo` and took the rest of the line as ARGUMENTS.
    //
    // Follows repl_193_cat_redirect_output_matches_source's shape for reasons that comment records:
    // clears its files FIRST (INT-172 hygiene), reads back WITHOUT `cat` (aliased to bat here), and
    // filters the `[n/N]` progress lines, because the success token also appears in the echoed
    // command text -- without that filter these pass no matter what the shell does.
    // ⚠️ A QUOTED `>` IS DATA, NOT AN OPERATOR. detect_redirect used `rfind(" > ")` with no quote
    // state, so `echo "a > b"` split at the QUOTED arrow: the command became `echo "a`, the target
    // became `b"`, and a file named `b"` appeared in the working directory while the command
    // printed nothing.
    //
    // ★ THE PIPE IS LOAD-BEARING IN THIS TEST, not incidental. With routing on, `echo "a > b"` is
    // CLAIMED by the spine and never reaches detect_redirect at all -- so a test without the pipe
    // would pass on a broken binary and could never be witnessed red. The pipe makes the router
    // DECLINE, forcing the legacy path regardless of the toggle.
    //
    // ★ AND NO REAL REDIRECT HERE, also deliberate: `rfind` takes the LAST match, so
    // `echo "a > b" > file` finds the genuine arrow and behaves correctly. The bug only bites when
    // the quoted arrow is the last one.
    results.push(test(
        "repl_quoted_redirect_is_not_an_operator",
        Category::Repl,
        || {
            let out = repl::run_repl("echo \"zzq > zzmark\" | cat")?;
            // Exact line match: the echoed command contains quotes and `| cat`, so it cannot
            // satisfy this by accident.
            if out.iter().any(|l| l.trim() == "zzq > zzmark") {
                Ok(())
            } else {
                Err(format!(
                    "quoted redirect was treated as an operator: {out:?}"
                ))
            }
        },
    ));
    results.push(test(
        "repl_builtin_first_pipeline_with_redirect",
        Category::Repl,
        || {
            // grep -qx is a WHOLE-LINE match: if the bug returns and the file holds
            // `zzpipe | cat`, it does not match and no token is printed.
            let out = repl::run_repl(
                "rm -f /tmp/zzbp.txt; echo zzpipe | cat > /tmp/zzbp.txt; grep -qx zzpipe /tmp/zzbp.txt && echo BP_OK_169",
            )?;
            if out
                .iter()
                .any(|l| l.contains("BP_OK_169") && !l.trim_start().starts_with('['))
            {
                Ok(())
            } else {
                Err(format!("builtin-first pipeline did not reach the file: {out:?}"))
            }
        },
    ));
    // The gate must not be too broad: a plain builtin redirect has NO pipe and must still be
    // handled by the builtin, because sh cannot see fsh builtins like `d` or `intl`.
    results.push(test(
        "repl_plain_builtin_redirect_still_works",
        Category::Repl,
        || {
            let out = repl::run_repl(
                "rm -f /tmp/zzpb.txt; echo zzplain > /tmp/zzpb.txt; grep -qx zzplain /tmp/zzpb.txt && echo PB_OK_169",
            )?;
            if out
                .iter()
                .any(|l| l.contains("PB_OK_169") && !l.trim_start().starts_with('['))
            {
                Ok(())
            } else {
                Err(format!("plain builtin redirect broke: {out:?}"))
            }
        },
    ));
    results.push(test(
        "repl_193_self_referential_alias_survives",
        Category::Repl,
        || {
            // INT-057 was a STABILITY intent: a self-referential alias recursed forever
            // and took the terminal with it. That guard moved into expand_aliases with
            // the expansion. It expands once, stops, and runs the result as a command.
            let out = repl::run_repl("alias zzloop='zzloop -h'; zzloop")?;
            let joined = out.join("\n").to_lowercase();
            if joined.contains("not found") || joined.contains("no such") {
                Ok(())
            } else {
                Err(format!(
                    "self-referential alias did not terminate cleanly: {out:?}"
                ))
            }
        },
    ));
    results.push(test(
        "repl_193_nested_alias_preserves_quoting",
        Category::Repl,
        || {
            // INT-193: execute_impl rebuilds the line as args.join(" ") from ALREADY
            // TOKENIZED args, so a quoted multi-word argument becomes N bare ones. Only
            // NESTED chains reach it -- a direct alias resolves to a non-alias command
            // word first. printf %s. prints one arg as "a b." and two as "a.b.", which
            // echo cannot distinguish (it joins with a space).
            // RED ON HEAD as of gen 431. Proven by hand before this test existed.
            let out = repl::run_repl("alias zzq1='printf %s.'; alias zzq2='zzq1'; zzq2 \"a b\"")?;
            let joined = out.join("\n");
            if joined.contains("a.b.") {
                return Err(format!("nested alias split a quoted argument: {joined:?}"));
            }
            if !joined.contains("a b.") {
                return Err(format!("expected one argument 'a b.': {joined:?}"));
            }
            Ok(())
        },
    ));
    results.push(test(
        "repl_193_direct_alias_preserves_quoting",
        Category::Repl,
        || {
            // Control. A DIRECT alias never reaches the executor-side expansion, so this
            // passes today and must KEEP passing -- it guards the path already correct.
            let out = repl::run_repl("alias zzq3='printf %s.'; zzq3 \"a b\"")?;
            let joined = out.join("\n");
            if !joined.contains("a b.") || joined.contains("a.b.") {
                return Err(format!(
                    "direct alias mangled a quoted argument: {joined:?}"
                ));
            }
            Ok(())
        },
    ));
    results.push(test(
        "repl_193_alias_chain_resolves",
        Category::Repl,
        || {
            // Chains work BY ACCIDENT today (one pass at the prompt, the rest in the
            // executor). Consolidation must preserve them DELIBERATELY.
            let out = repl::run_repl(
                "alias zza='zzb'; alias zzb='zzc'; alias zzc='echo CHAIN_OK_193'; zza",
            )?;
            let joined = out.join("\n");
            if joined.contains("CHAIN_OK_193") {
                Ok(())
            } else {
                Err(format!("alias chain did not resolve: {joined:?}"))
            }
        },
    ));
    results.push(test(
        "repl_193_cat_redirect_output_matches_source",
        Category::Repl,
        || {
            // BUG-298-4 bypasses alias expansion for `cat` under a redirect. THE CONTRACT,
            // not the mechanism: the redirected output must be byte-identical to the
            // source. Says nothing about bat or builtins, so it stays valid if the
            // implementation changes again. Clears its files FIRST (INT-172 hygiene).
            // The success token also appears in the echoed command text, so the `[n/N]`
            // progress lines are filtered -- otherwise this passes no matter what.
            let out = repl::run_repl("rm -f /tmp/zz193c_src.txt /tmp/zz193c_out.txt; printf CATSRC193 > /tmp/zz193c_src.txt; cat /tmp/zz193c_src.txt > /tmp/zz193c_out.txt; cmp -s /tmp/zz193c_src.txt /tmp/zz193c_out.txt && echo CMP_OK_193")?;
            let ok = out
                .iter()
                .any(|l| l.contains("CMP_OK_193") && !l.trim_start().starts_with('['));
            if ok {
                Ok(())
            } else {
                Err(format!("redirected cat output did not match source: {out:?}"))
            }
        },
    ));
    // ── INT-169 / logical chains: three REGRESSIONS, expected RED until the logical
    // executor calls the canonical per-command path. main.rs splits `&&`/`||` at 1332 and runs
    // its own reduced dispatch, which REPLICATES execution logic selectively -- `cd` got bespoke
    // support (1365-1393) and works; variable expansion, alias resolution and `export` did not.
    // These assert the CONTRACT (a chained command behaves like a standalone one), never the
    // mechanism, so they stay valid whichever way the duplication is removed.
    //
    // The harness runs fsh in /tmp (repl.rs:112), so the prompt can never contain the home path --
    // that is what makes the variable assertion meaningful rather than accidental.
    results.push(test("repl_chain_expands_variables", Category::Repl, || {
        let home = std::env::var("HOME").map_err(|e| format!("no HOME: {e}"))?;
        let out = repl::run_repl("echo $HOME && echo CHAINVAR_DONE")?;
        let joined = out.join("\n");
        let expanded = out
            .iter()
            .any(|l| l.contains(&home) && !l.trim_start().starts_with('['));
        let literal = out.iter().any(|l| l.contains("$HOME"));
        if expanded && !literal {
            Ok(())
        } else if literal {
            Err(format!(
                "chained command did not expand $HOME -- got the LITERAL string, so any \
                     command using a variable inside && runs against the wrong text: {joined:?}"
            ))
        } else {
            Err(format!("expected {home:?} in the output, saw: {joined:?}"))
        }
    }));
    results.push(test("repl_chain_resolves_aliases", Category::Repl, || {
        // `uname` prints Linux, and neither word appears in the command text -- so a match
        // cannot come from the command being echoed back.
        let out = repl::run_repl("alias zzc1='uname'; zzc1 && echo CHAINALIAS_DONE")?;
        let joined = out.join("\n");
        if out.iter().any(|l| l.contains("Linux")) {
            Ok(())
        } else {
            Err(format!(
                "alias did not resolve inside a chain -- with 285 aliases configured, every \
                     one of them fails this way when chained: {joined:?}"
            ))
        }
    }));
    // ── INT-200 CONFORMANCE: what bash actually does, versus what fsh does.
    //
    // ⚠️ MIGRATED FROM `spine conform` (2026-08-03), and the reason is the finding that prompted
    // it: that harness invoked fsh with `-c`, and `fsh -c` delegates the whole string to `sh`. It
    // was comparing sh against bash and calling the result fsh conformance. Its two "unexplained"
    // results were that door, not a defect.
    //
    // ★ IT ALSO BELONGS HERE BY DESIGN, not just by convenience. Its three-verdict rule -- a
    // declared divergence that starts MATCHING bash again is a FAILURE -- is a statement about
    // drift over time, which only means something if something runs it repeatedly. It lived as a
    // typed command, ran once, and never ran again. That is the fate of a regression suite with no
    // CI home.
    //
    // ★ AND THE PTY IS THE POINT: fsh's real behaviour is what the interactive shell does, which is
    // exactly what run_repl drives. File effects are observable here too, because a case can read
    // the file back -- the limitation the old harness recorded and could not fix.
    for (line, diverges) in CONFORMANCE_CASES {
        results.push(test(
            Box::leak(format!("conform_{}", slug(line)).into_boxed_str()),
            Category::Repl,
            move || {
                // ⚠️ RUN BASH IN /tmp, NOT THE REPO -- and fsh too, which
                // run_repl_lines_status already does. Two cases execute as redirects and write
                // files named `0.5` and `=` wherever the shell runs. They landed in the repo root
                // and were committed before anyone noticed.
                //
                // ★ AND BOTH SHELLS NEED THIS NOW. The original note said fsh was the shell that
                // refused them and the reference implementation was not. Since 2026-08-07 fsh
                // executes them exactly as bash does, so the asymmetry that made this a one-sided
                // precaution is gone -- which is also why neither case declares a divergence any
                // more.
                let bash = std::process::Command::new("bash")
                    .current_dir("/tmp")
                    .args(["-c", line])
                    .output()
                    .map_err(|e| format!("bash unavailable: {e}"))?;
                let bash_out = String::from_utf8_lossy(&bash.stdout).trim().to_string();
                let bash_status = bash.status.code();
                let (fsh_lines, fsh_status) = repl::run_repl_lines_status(&[line], &[])?;
                let fsh_out = fsh_lines.join("\n");
                // ⚠️ WHAT IS EXCLUDED FROM THE COMPARISON, AND WHY. Every entry here is
                // SHELL-GENERATED rather than command output, and the list is deliberately closed:
                // anything not on it that fails a case means the case is wrong or a divergence
                // needs declaring -- not that the filter needs another clause. A filter that grows
                // to make cases green is the same mistake as measuring `sh` and calling it fsh.
                //
                //   1. OSC 133 shell-integration markers -- terminal protocol, like ANSI colour.
                //      Already removed by strip_ansi; noted so nobody re-adds them as "output".
                //   2. The prompt -- emitted to delimit interaction, not by the command under test.
                //   3. Shell-generated EXECUTION SUMMARIES (`x exited N -- ...`) -- fsh reports on
                //      the command it just ran; bash says nothing. Different voice, same execution.
                //   4. The multi-command progress display (`○ N commands`, `[n/N] <text>`) -- and
                //      this one MUST go, because it echoes the command text, which frequently
                //      contains the expected output and would make a `contains` pass by accident.
                fn is_shell_ui(line: &str) -> bool {
                    let t = line.trim();
                    t.is_empty()
                        || t.contains("nsh❯")
                        || t.starts_with('🔧')
                        || t.starts_with('○')
                        || t.starts_with('[')
                        || t.starts_with("x ")
                        || t.starts_with('✗')
                        || t.starts_with("🌳")
                }
                let fsh_body: String = fsh_out
                    .lines()
                    .filter(|l| !is_shell_ui(l))
                    .collect::<Vec<_>>()
                    .join("\n");
                // ⚠️ NO `is_empty()` SHORT-CIRCUIT. An earlier version read
                // `bash_out.is_empty() || fsh_out.contains(&bash_out)`, which scored every case
                // where bash prints nothing as a MATCH -- so a declared divergence like
                // `echo test > 0.5` (bash writes a file and prints nothing) reported "now matches
                // bash" every run. The check inverted itself, and it would have hidden a real
                // digit-guard regression behind a permanent false alarm.
                let stdout_same = if bash_out.is_empty() {
                    fsh_body.trim().is_empty()
                } else {
                    fsh_body.contains(&bash_out)
                };
                // ⚠️ A MISSING STATUS IS AN ERROR, NOT A SKIP. `133;D;<n>` lands inside the capture
                // window for every shape measured, so its absence means the harness could not read
                // something it should have. Treating unknown as "not comparable" would be the same
                // silent weakening as growing is_shell_ui to make a case pass.
                let fsh_code = fsh_status.ok_or_else(|| {
                    format!(
                        "nsh emitted no 133;D status marker, so conformance cannot be judged on \
                         status: {fsh_out:?}"
                    )
                })?;
                let status_same = bash_status == Some(fsh_code);
                // ★ STATUS FOLDS INTO THE VERDICT rather than forming a second one. The corpus
                // records ONE reason per case, not one per channel, so a declared divergence
                // excuses the case as a whole. Splitting it would claim a precision the reasons
                // do not carry. The messages below name which channel differed, so nothing hides.
                let same = stdout_same && status_same;
                let detail = match (stdout_same, status_same) {
                    (false, false) => "stdout AND exit status",
                    (false, true) => "stdout",
                    (true, false) => "exit status",
                    (true, true) => "nothing",
                };
                match (diverges, same) {
                    (None, true) => Ok(()),
                    (Some(_), false) => Ok(()),
                    (None, false) => Err(format!(
                        "nsh disagrees with bash on {detail} and nobody wrote down why -- \
                         bash: {bash_out:?} (exit {bash_status:?}), \
                         fsh: {fsh_out:?} (exit {fsh_code})"
                    )),
                    // ⚠️ THE SUBTLE FAILURE: a declared divergence that started matching bash
                    // again -- in BOTH output and status -- means the deliberate behaviour was
                    // silently lost.
                    (Some(why), true) => Err(format!(
                        "this was declared to DIVERGE from bash and now matches it in output and \
                         exit status -- the deliberate behaviour is gone: {why}"
                    )),
                }
            },
        ));
    }

    // ── INT-285 / INT-200: a control structure containing `&&` must survive intact.
    //
    // ⚠️ THIS IS A REGRESSION FROM THE BOOLEAN-CHAIN FLATTEN (7db111fa), found four days after it
    // shipped. `split_semicolons` deliberately keeps `if …; then …; fi`, `for …; do …; done` and
    // piped whiles ATOMIC -- they go to `sh` as one unit, which is INT-285 BUG 2's fix. The
    // flatten then ran `split_logical` over EVERY segment including those, and it knows nothing
    // about `then`/`fi`/`done`, so it cut the construct at the `&&` and each half reached sh as a
    // fragment: "syntax error: unexpected end of file from `if'".
    //
    // ★ NOTHING CAUGHT IT because all three chain regressions use SIMPLE commands. The bug lived
    // exactly where the tests did not look, which is the reason this one exists.
    //
    // The contract is behavioural on purpose -- both branches run, and no syntax error escapes --
    // so it stays valid whichever way the splitters are eventually reshaped.
    results.push(test(
        "repl_control_structure_with_a_boolean_chain_stays_intact",
        Category::Repl,
        || {
            let out = repl::run_repl("if true; then echo ZZIFA && echo ZZIFB; fi")?;
            let joined = out.join("\n");
            let a = out.iter().any(|l| l.contains("ZZIFA"));
            let b = out.iter().any(|l| l.contains("ZZIFB"));
            let torn = out.iter().any(|l| l.contains("syntax error"));
            if a && b && !torn {
                Ok(())
            } else if torn {
                Err(format!(
                    "the construct was torn apart -- `split_logical` cut it at the `&&` and sh \
                     received a fragment, so an `if` containing a boolean chain cannot run at \
                     all: {joined:?}"
                ))
            } else {
                Err(format!("expected both branches to run, saw: {joined:?}"))
            }
        },
    ));

    // ── INT-200 background: two REGRESSIONS, expected RED. main.rs:3144 detects a trailing
    // `&` and hands the line to JobTable::spawn, which re-derives argv with splitn(2, ' ') and
    // split_whitespace() -- a naive re-tokenizer running AFTER the shell already knew the real
    // structure. Two bugs fall out of those five lines, and both are INT-195's invariant broken:
    // every stage must consume the previous stage's output, never the original string.
    // ── INT-169: job control must survive ROUTING. `jobs` reads the JobTable that lives in
    // the REPL loop, which spine dispatch has no path to -- so the router could parse it, claim
    // it, and never run it. That is exactly what happened from gen 447 to gen 454: `jobs` printed
    // "command not found" while the table still filled and the prompt still showed [1 job]. Only
    // the inspection command was dead, which is why six generations went by without notice.
    //
    // ⚠️ THIS NEEDS ONE SESSION, not two. A background job dies with its shell, so unlike the
    // redirect tests below there is no file to carry the result across -- which is the whole
    // reason run_repl_lines exists.
    results.push(test(
        "repl_jobs_lists_a_running_job",
        Category::Repl,
        || {
            let out = repl::run_repl_lines(&["sleep 20 &", "jobs"])?;
            let joined = out.join("\n");
            // The job table prints the command name; a failure prints "command not found: jobs".
            // Asserting on the ABSENCE of the error as well as the presence of the listing, because
            // an empty capture would otherwise read as a pass on the second condition alone.
            let listed = out.iter().any(|l| l.contains("sleep"));
            let not_found = out.iter().any(|l| l.contains("command not found"));
            if listed && !not_found {
                Ok(())
            } else if not_found {
                Err(format!(
                "`jobs` was claimed by the router and never ran -- job control is half-dead: the \
                 table fills and the prompt counts, but the user cannot inspect it: {joined:?}"
            ))
            } else {
                Err(format!(
                    "expected the running job to be listed, saw: {joined:?}"
                ))
            }
        },
    ));

    results.push(test(
        "repl_background_job_honours_its_redirect",
        Category::Repl,
        || {
            // The redirect becomes ARGUMENTS: `uname > f &` spawns uname with argv [">", "f"].
            // Real uname ignores unknown args, prints to the terminal and exits 0 -- so the failure
            // is SILENT, which is why this asserts on the FILE and not on the output.
            // ONE session: the job must be launched, given time, and read back by the same
            // shell -- `cat` through a second `run_repl` returned an EMPTY capture, so the test
            // reported red without ever observing the file. A test red for the wrong reason
            // hides the transition it exists to detect.
            //
            // ⚠️ `sed -n 1p`, not `cat`: cat is aliased to bat, whose box-drawing output puts
            // the content behind a `│` and makes a plain substring match unreliable.
            let out = repl::run_repl_lines(&[
                "rm -f /tmp/zzbg1.txt",
                "uname > /tmp/zzbg1.txt &",
                "sleep 2",
                "sed -n 1p /tmp/zzbg1.txt",
            ])?;
            let joined = out.join("\n");
            if out
                .iter()
                .any(|l| l.contains("Linux") && !l.trim_start().starts_with('['))
            {
                Ok(())
            } else {
                Err(format!(
                    "a backgrounded command lost its redirect -- the file was never written, and \
                 the command still exited 0, so nothing reported the loss: {joined:?}"
                ))
            }
        },
    ));
    results.push(test(
        "repl_background_job_keeps_quoted_arguments",
        Category::Repl,
        || {
            // A quoted argument is split on spaces, so `sh -c "..."` receives only the first
            // fragment as its script. This one at least fails loudly -- the child reports an
            // unexpected EOF -- but the argv is wrong for every quoted background command.
            // TWO SESSIONS, and the FILE carries the result between them: `&` must be the last
            // thing on its line, and this harness submits exactly ONE line per call (proven: an input
            // of "echo A\necho B" returned only A). The launching shell exits while the job runs,
            // which also proves the job is genuinely detached rather than waited on.
            // ONE session: the job must be launched, given time, and read back by the same
            // shell -- `cat` through a second `run_repl` returned an EMPTY capture, so the test
            // reported red without ever observing the file. A test red for the wrong reason
            // hides the transition it exists to detect.
            //
            // ⚠️ `sed -n 1p`, not `cat`: cat is aliased to bat, whose box-drawing output puts
            // the content behind a `│` and makes a plain substring match unreliable.
            let out = repl::run_repl_lines(&[
                "rm -f /tmp/zzbg2.txt",
                "sh -c \"echo ZZBGQUOTED > /tmp/zzbg2.txt\" &",
                "sleep 2",
                "sed -n 1p /tmp/zzbg2.txt",
            ])?;
            let joined = out.join("\n");
            if out
                .iter()
                .any(|l| l.contains("ZZBGQUOTED") && !l.trim_start().starts_with('['))
            {
                Ok(())
            } else {
                Err(format!(
                    "a backgrounded command lost its quoting -- the quoted script was split on \
                 spaces, so the child received a fragment instead of the whole argument: \
                 {joined:?}"
                ))
            }
        },
    ));

    results.push(test(
        "repl_background_job_keeps_quoted_arguments_legacy",
        Category::Repl,
        || {
            // THE SAME ASSERTION THROUGH THE OTHER DOOR, and the pair is the point. The case above
            // runs spine-routed, because until now every case did: the spawn set no environment, so
            // the spine answered everything and the legacy path was never exercised. That is why the
            // case above passed for months while legacy was mangling quoted arguments -- it claims
            // `sh -c "..." &` and handles the quoting correctly, so the test never reached the code
            // its name describes. The bug was found by hand instead, and fixed at d0c04825.
            let out = repl::run_repl_lines_env(
                &[
                    "rm -f /tmp/zzbg2L.txt",
                    "sh -c \"echo ZZBGQUOTEDL > /tmp/zzbg2L.txt\" &",
                    "sleep 2",
                    "sed -n 1p /tmp/zzbg2L.txt",
                ],
                &[("NSH_SPINE", "0")],
            )?;
            let joined = out.join("\n");
            if out
                .iter()
                .any(|l| l.contains("ZZBGQUOTEDL") && !l.trim_start().starts_with('['))
            {
                Ok(())
            } else {
                Err(format!(
                    "the LEGACY background path lost its quoting -- argv was re-derived from text \
                 instead of going through the shell's one tokenizer, so the child received a \
                 fragment: {joined:?}"
                ))
            }
        },
    ));

    results.push(test(
        "repl_background_redirect_refused_on_legacy",
        Category::Repl,
        || {
            // A REFUSAL IS THE ASSERTION, not a redirect working. detect_redirect runs six hundred
            // lines before the background handler and takes everything right of the last unquoted
            // `>` as the target, so `cmd > f &` yielded a target of `f &` -- a file whose NAME ended
            // in an ampersand, run in the FOREGROUND, with no job registered and nothing reported.
            // Seven such files accumulated in /tmp before anyone noticed what they were.
            //
            // Legacy cannot be made to do this correctly without a second copy of configure_file_io,
            // so a33d6cd7 made it refuse instead. This case exists so the refusal cannot quietly
            // become a junk file again. The spine claims the simple form and honours it; only what
            // the spine declines reaches here, which is why the case is routed to legacy explicitly.
            //
            // ONE line, because only the last command's output comes back.
            let out = repl::run_repl_lines_env(
                &["echo hi | cat > /tmp/zzbgredirL.txt &"],
                &[("NSH_SPINE", "0")],
            )?;
            let joined = out.join("\n");
            if joined.contains("not supported here") {
                Ok(())
            } else {
                Err(format!(
                    "a backgrounded redirect on the legacy path was not refused -- it has \
                 probably gone back to creating a file whose name ends in an ampersand: \
                 {joined:?}"
                ))
            }
        },
    ));

    results.push(test(
        "repl_chain_runs_builtins_and_next_command_sees_the_effect",
        Category::Repl,
        || {
            // Two assertions in one: the builtin RAN, and the following chained command OBSERVED
            // its effect. Checking only the first would pass on a shell that accepted `export`
            // and then discarded it.
            let out = repl::run_repl("export ZZC2=zzvalue && echo $ZZC2")?;
            let joined = out.join("\n");
            let seen = out
                .iter()
                .any(|l| l.contains("zzvalue") && !l.contains("export"));
            if seen {
                Ok(())
            } else {
                Err(format!(
                    "export did not take effect inside a chain -- the builtin either never ran or \
                     the next command did not observe it: {joined:?}"
                ))
            }
        },
    ));

    results.push(test(
        "repl_193_redirect_from_alias_value",
        Category::Repl,
        || {
            // The redirect operator arrives FROM the alias value, not the typed line.
            // Expansion runs BEFORE detect_redirect, and moving expansion must not
            // change that. Clears its file FIRST (INT-172 hygiene rule). The marker also
            // appears in the alias confirmation line, so that line is filtered out --
            // and the file is read with sed, since `cat` is aliased to bat here.
            // otherwise this passes even when the redirect is broken.
            let out = repl::run_repl("rm -f /tmp/zz193r.txt; alias zzr='echo ZR193 >'; zzr /tmp/zz193r.txt; sed -n 1p /tmp/zz193r.txt")?;
            let ok = out.iter().any(|l| l.contains("ZR193") && !l.contains("alias"));
            if ok {
                Ok(())
            } else {
                Err(format!("redirect from alias value lost: {out:?}"))
            }
        },
    ));
    results.push(test(
        "repl_195_quoted_command_word_reaches_the_safety_guard",
        Category::Repl,
        || {
            // INT-195 REGRESSION COVER for safety_guard.rs:19, which had none at all.
            //
            // Gen 432: the guard derived its word with split_whitespace().next(), so `"rm" -rf /`
            // read as `"rm` -- matching no deny entry, no allow entry and no safe entry -- while
            // the executor, which IS quote aware, ran it. The guard stayed silent. Line 19 now
            // derives through the canonical quote-aware command_word().
            //
            // mkfs.zzz, NOT rm. safety_guard.rs:100 matches on the WORD ALONE
            // (first_word.starts_with("mkfs")) and no such binary exists, so if this ever
            // regresses the worst case is command-not-found rather than damage. A safety test
            // whose payload is destructive when the abort fails is not a safety test.
            //
            // Asserts the guard FIRED, never that the echoed line lost its quotes: the warning
            // deliberately echoes the trimmed ORIGINAL text, so `Filesystem format: "mkfs.zzz"`
            // keeps them. Firing at all is what proves the derivation.
            let out = repl::run_repl_answered("\"mkfs.zzz\"", "Type 'yes' to proceed", "no")?;
            let challenged = out.iter().any(|l| l.contains("CHALLENGE"));
            let named = out.iter().any(|l| l.contains("Filesystem format:"));
            let blocked = out
                .iter()
                .any(|l| l.contains("Command blocked by Friday safety guard"));
            if challenged && named && blocked {
                Ok(())
            } else {
                Err(format!(
                    "quoted word did not reach the guard (challenged={challenged}, named={named}, blocked={blocked}): {out:?}"
                ))
            }
        },
    ));
    results.push(test(
        "repl_195_bare_command_word_still_reaches_the_safety_guard",
        Category::Repl,
        || {
            // THE CONTROL, and it earns its second. Under a revert of safety_guard.rs:19 this case
            // must stay GREEN while the quoted one goes RED -- which is what shows the revert broke
            // the QUOTED path specifically rather than disabling the guard wholesale. Without it,
            // a red pair would prove nothing about quoting.
            let out = repl::run_repl_answered("mkfs.zzz", "Type 'yes' to proceed", "no")?;
            let challenged = out.iter().any(|l| l.contains("CHALLENGE"));
            let blocked = out
                .iter()
                .any(|l| l.contains("Command blocked by Friday safety guard"));
            if challenged && blocked {
                Ok(())
            } else {
                Err(format!(
                    "bare word did not reach the guard (challenged={challenged}, blocked={blocked}): {out:?}"
                ))
            }
        },
    ));
    results.push(test(
        "repl_169_assignment_expands_before_the_child_exists",
        Category::Repl,
        || {
            // THE ONE INTENTIONAL BEHAVIOUR CHANGE of Increment 3, and it matches bash.
            // A prefix assignment belongs to the CHILD, so parameter expansion of the same line
            // happens in the shell BEFORE that child exists -- $ZZA169 must therefore see the OLD
            // value. Legacy set the variables first and expanded afterwards, printing the new one.
            // Measured on gen 484 before the move: [new]. After: [old].
            let out = repl::run_repl("ZZA169=old; ZZA169=new echo [$ZZA169]")?;
            let old = out.iter().any(|l| l.contains("[old]"));
            let new = out.iter().any(|l| l.contains("[new]"));
            if old && !new {
                Ok(())
            } else {
                Err(format!(
                    "expected the OLD value (old={old}, new={new}): {out:?}"
                ))
            }
        },
    ));
    results.push(test(
        "repl_169_prefix_assignment_does_not_leak_into_the_session",
        Category::Repl,
        || {
            // THE GUARDRAIL, and the reason it earns a second case beside repl_143: that one runs
            // through legacy, this one runs through the SPINE. Same POSIX property, two owners.
            // The spine achieves it structurally -- the value goes on the child via cmd.env -- so
            // there is no save/restore step that can be forgotten. Legacy needed 13 lines.
            // This is the INT-143 bug B property: QEMU_OPTS survived its command and four VM boots
            // were blamed on the firmware.
            let out = repl::run_repl("ZZD169=1 printenv ZZD169; echo [$ZZD169]")?;
            let child_saw = out.iter().any(|l| l.trim() == "1");
            let leaked = out.iter().any(|l| l.contains("[1]"));
            if child_saw && !leaked {
                Ok(())
            } else {
                Err(format!("child_saw={child_saw}, leaked={leaked}: {out:?}"))
            }
        },
    ));
    results.push(test(
        "repl_169_an_assignment_prefix_does_not_hide_the_alias",
        Category::Repl,
        || {
            // MATCHES BASH, and this case exists because the reasoning that predicted otherwise
            // was wrong TWICE, in opposite directions, inside one session.
            //
            // THE CAUSE WAS THE TWO DOORS. Every probe used `fsh -c`, where the alias defined in
            // the first segment is not visible to the second, so `ZZB=1 zzt` reported command not
            // found and looked like the prefix hiding the alias. Through the REPL door it expands
            // -- and it expands on gen 484 as well, so Increment 3 changed nothing here. The
            // harness said so when the reasoning could not.
            //
            // ⚠️ WHICHEVER DOOR A CLAIM COMES FROM, SAY WHICH. This is the same trap
            // repl.rs and `spine conform` were both built to close.
            let out = repl::run_repl("alias zzt169=\"echo ALIAS_EXPANDED\"; ZZB169=1 zzt169")?;
            if out.iter().any(|l| l.contains("ALIAS_EXPANDED")) {
                Ok(())
            } else {
                Err(format!(
                    "the alias did not expand through the REPL door: {out:?}"
                ))
            }
        },
    ));
    results.push(test(
        "repl_169_bare_assignment_still_persists",
        Category::Repl,
        || {
            // THE SHAPE THE SPINE REFUSES ON PURPOSE. A bare assignment is a shell STATEMENT that
            // persists for the session, not a process -- an ExecutionPlan describes one process and
            // there is none here, so lowering declines and legacy keeps it. Before the guard existed
            // this produced an empty argv and the executor answered "empty plan: nothing to
            // execute", turning a valid statement into an error.
            let out = repl::run_repl("ZZC169=kept; echo [$ZZC169]")?;
            if out.iter().any(|l| l.contains("[kept]")) {
                Ok(())
            } else {
                Err(format!("bare assignment did not persist: {out:?}"))
            }
        },
    ));
    results.push(test(
        "repl_220_a_stderr_redirect_survives_a_pipeline",
        Category::Repl,
        || {
            // INT-220. spawn_pipeline read stdin and stdout from the plan and then set stderr to
            // inherit UNCONDITIONALLY, so a 2> on any stage was parsed, lowered, and discarded.
            // Measured before the fix: 1> in a pipeline worked and 2> did not, on either side.
            let f = "/tmp/nsh-test-int220.txt";
            let _ = std::fs::remove_file(f);
            let out = repl::run_repl_lines(&[
                &format!("sh -c \"echo ZZ220 >&2\" 2>{f} | cat"),
                &format!("cat {f}"),
            ])?;
            let joined = out.join("|");
            let _ = std::fs::remove_file(f);
            if joined.contains("ZZ220") {
                Ok(())
            } else {
                Err(format!("stderr redirect lost in a pipeline: {out:?}"))
            }
        },
    ));
    results.push(test(
        "repl_203_a_heredoc_body_is_not_brace_expanded",
        Category::Heredoc,
        || {
            // INT-203 gate 4. A quoted delimiter means the body is DATA. Bracketed paste delivers
            // the whole construct as ONE line, so brace expansion saw the body and ate any range
            // in it -- proven expanding before this fix.
            let out =
                repl::run_repl_lines(&["cat << 'EOF'\nfor i in {a..c}; do echo $i; done\nEOF"])?;
            if !out.join("|").contains("{a..c}") {
                return Err(format!("heredoc body was brace-expanded: {out:?}"));
            }
            Ok(())
        },
    ));
    results.push(test(
        "repl_203_a_quoted_brace_range_is_not_expanded",
        Category::Repl,
        || {
            // INT-203 gate 3. A quoted range is data; an unquoted one still expands. The control
            // is here so a fix that simply stops expanding cannot pass.
            //
            // TWO SEPARATE SESSIONS, deliberately. With both lines in one session the harness
            // captured only one of the two outputs, which read as the quoted case being expanded
            // when it had simply not been captured. Separate sessions make each answer its own.
            let q = repl::run_repl_lines(&["echo \"{a..c}\""])?;
            if !q.join("|").contains("{a..c}") {
                return Err(format!("quoted range was expanded: {q:?}"));
            }
            let u = repl::run_repl_lines(&["echo {a..c}"])?;
            if !u.join("|").contains("a b c") {
                return Err(format!("unquoted range stopped expanding: {u:?}"));
            }
            Ok(())
        },
    ));
    results.push(test(
        "repl_197_an_alias_expanding_to_a_gated_command_is_gated",
        Category::Repl,
        || {
            // INT-197 gates 3 and 6. The guard previously evaluated the PRE-EXPANSION command
            // identity while execution subsequently expanded the alias, so an alias whose expansion
            // is a gated command was not gated. Proven red-first on gen 493: the aliased invocation
            // reported command-not-found with no guard output at all.
            //
            // TWO LINES, DELIBERATELY. A version with the definition and invocation on ONE line
            // separated by a semicolon is testing something else: the line command word is then
            // `alias`, and the second segment is OUTSIDE this gate guarantee by the documented
            // compound-line limitation.
            //
            // Payload is mkfs.zzz because the guard matches on the word alone and no such binary
            // exists, so a failed guard costs command-not-found rather than damage.
            let out = repl::run_repl_answered_after(
                &["alias zzq219=mkfs.zzz"],
                "zzq219",
                "CHALLENGE",
                "no",
            )?;
            if out.iter().any(|l| l.contains("CHALLENGE")) {
                Ok(())
            } else {
                Err(format!(
                    "an aliased gated command reached execution unguarded: {out:?}"
                ))
            }
        },
    ));
    results.push(test(
        "repl_196_a_declined_construct_is_still_guarded",
        Category::Repl,
        || {
            // INT-196 M8/M9: a line the SPINE DECLINES is still guarded. A 2-and-angle redirect
            // parses Complete but lowering refuses it, so it falls back to legacy -- and the
            // guard must still fire before it gets there. That shape is also the INT-172 defect,
            // which makes it the right thing to guard.
            //
            // TWO EARLIER VERSIONS OF THIS CASE WERE WRONG, and both were caught rather than
            // shipped. The first used a pipe and claimed to test a REFUSAL: it passed with the
            // scanner fallback DISABLED, proving it tested neither arm. The second used an
            // unterminated quote, which is genuinely Incomplete -- and the REPL holds such a
            // line as a multi-line BUFFER waiting for more input, so it never submits, the
            // harness never sees a prompt, and the case timed out at the wait_for boundary.
            //
            // THAT INCOMPLETE-PASTE PROPERTY IS REAL AND HOLDS, proven by hand outside the
            // harness: completing the quote on a second line produces a Filesystem format
            // warning and the guard BLOCKS. It is simply not reachable through
            // run_repl_answered, which sends ONE line and waits for one prompt.
            // Payload is mkfs.zzz rather than rm: the thing under test IS the abort, so a
            // destructive payload would run if it ever regressed. No such binary exists.
            let out = repl::run_repl_answered("mkfs.zzz 2> /tmp/zz196.txt", "CHALLENGE", "no")?;
            if out.iter().any(|l| l.contains("CHALLENGE")) {
                Ok(())
            } else {
                Err(format!(
                    "a declined construct reached execution unguarded: {out:?}"
                ))
            }
        },
    ));
    results.push(test(
        "repl_196_an_operator_leading_line_makes_no_guard_decision",
        Category::Repl,
        || {
            // INT-196 M4 in its reachable form. A line with no command word yields None, so no
            // guard decision is made -- and the shell must not hang, panic, or challenge. It
            // reports an ordinary error and carries on.
            let out = repl::run_repl("| mkfs.zzz")?;
            if out.iter().any(|l| l.contains("CHALLENGE")) {
                Err(format!(
                    "a line with no command word produced a guard decision: {out:?}"
                ))
            } else {
                Ok(())
            }
        },
    ));
    results.push(test("repl_143_inline_var_scoped", Category::Repl, || {
        // d5a52c1c: `VAR="a b" cmd` -- the QEMU_OPTS incident. The var was set and
        // NEVER unset, leaking into the session. POSIX scopes it to that command
        // only. One REPL line, two effects: cmd sees a prefix, the next echo must
        // show the var GONE. If it leaks, the bracket holds the value.
        let out = repl::run_repl("G3SCOPE143=leaked echo scope_ok143; echo [$G3SCOPE143]")?;
        let ran = out.iter().any(|l| l.contains("scope_ok143"));
        let leaked = out.iter().any(|l| l.contains("leaked"));
        if ran && !leaked {
            Ok(())
        } else {
            Err(format!(
                "inline var leaked or cmd failed (ran={}, leaked={}): {:?}",
                ran, leaked, out
            ))
        }
    }));

    // --- HOSTILE ---
    //
    // The suite proves fsh handles CORRECT input. These ask whether it survives input
    // nobody sanitised, which is where quoting and globbing bugs live. The bar is NOT
    // exact output -- it is: no panic, no hang, and something sensible said. Each case
    // builds its own directory under /tmp so nothing touches the working tree.
    results.push(test(
        "hostile_filename_with_space",
        Category::Hostile,
        || {
            let d = "/tmp/nsh-hostile-space";
            let _ = std::fs::remove_dir_all(d);
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            std::fs::write(format!("{}/two words.txt", d), "x").map_err(|e| e.to_string())?;
            let out = run_fsh(&format!("ls {}", d))?;
            let r = expect_contains(&out, "two words.txt");
            let _ = std::fs::remove_dir_all(d);
            r
        },
    ));
    results.push(test(
        "hostile_filename_with_quote",
        Category::Hostile,
        || {
            let d = "/tmp/nsh-hostile-quote";
            let _ = std::fs::remove_dir_all(d);
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            std::fs::write(format!("{}/it\u{27}s.txt", d), "x").map_err(|e| e.to_string())?;
            let out = run_fsh(&format!("ls {}", d))?;
            let r = expect_contains(&out, "s.txt");
            let _ = std::fs::remove_dir_all(d);
            r
        },
    ));
    results.push(test(
        "hostile_filename_with_glob_chars",
        Category::Hostile,
        || {
            let d = "/tmp/nsh-hostile-glob";
            let _ = std::fs::remove_dir_all(d);
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            std::fs::write(format!("{}/star*name.txt", d), "x").map_err(|e| e.to_string())?;
            std::fs::write(format!("{}/quest?name.txt", d), "x").map_err(|e| e.to_string())?;
            let out = run_fsh(&format!("ls {}", d))?;
            let r = expect_contains(&out, "star*name.txt")
                .and_then(|_| expect_contains(&out, "quest?name.txt"));
            let _ = std::fs::remove_dir_all(d);
            r
        },
    ));
    results.push(test(
        "hostile_filename_with_newline",
        Category::Hostile,
        || {
            let d = "/tmp/nsh-hostile-newline";
            let _ = std::fs::remove_dir_all(d);
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            std::fs::write(format!("{}/line\nbreak.txt", d), "x").map_err(|e| e.to_string())?;
            let out = run_fsh(&format!("ls {}", d))?;
            let r = expect_contains(&out, "break.txt");
            let _ = std::fs::remove_dir_all(d);
            r
        },
    ));

    // --- HOSTILE: environment ---
    //
    // Through the REPL door, not -c. INT-207 established that -c delegates the whole
    // string to sh, so no alias, spine or guard applies -- an environment case run that
    // way would be testing sh's handling of a broken environment, not fsh's.
    //
    // run_repl_lines_status takes an env slice and drives the real REPL, which is what
    // these need. It SETS variables; it cannot unset them (.envs, no env_remove), so
    // HOME-unset is not expressible yet and is not attempted here rather than being
    // faked with an empty string, which is a different environment.
    results.push(test("hostile_home_nonexistent", Category::Hostile, || {
        let (out, _) = repl::run_repl_lines_status(
            &["echo start", "cd ~", "echo survived"],
            &[("HOME", "/nonexistent/path/for/testing")],
        )?;
        // The bar is that the shell is still answering afterwards, not that cd succeeded.
        expect_contains(&out.join("\n"), "survived")
    }));
    results.push(test("hostile_term_empty", Category::Hostile, || {
        let (out, _) = repl::run_repl_lines_status(&["echo rendered"], &[("TERM", "")])?;
        expect_contains(&out.join("\n"), "rendered")
    }));
    results.push(test(
        "hostile_path_empty_builtin_still_works",
        Category::Hostile,
        || {
            // A BUILTIN MUST NOT NEED PATH. echo is fsh's own; if it fails when nothing
            // external resolves, the shell is shelling out for something it owns.
            let (out, _) = repl::run_repl_lines_status(&["echo builtin-ok"], &[("PATH", "")])?;
            expect_contains(&out.join("\n"), "builtin-ok")
        },
    ));

    // --- HOSTILE: parser ---
    //
    // Through the REPL door for a reason the other door cannot give: run_session waits
    // with a timeout and kills the child, so a shell that HANGS on malformed input fails
    // the case rather than blocking the suite.
    //
    // Each case submits the malformed line, then a known-good line. The assertion is on
    // the SECOND line: the shell must still be answering. Whether it errored or ignored
    // the first is a different question from whether it survived.
    //
    // UNTERMINATED QUOTES ARE DELIBERATELY ABSENT. Two such cases were written, timed
    // out at 30s, and were WRONG: bash hangs on identical input, because an unterminated
    // quote at an interactive prompt means keep reading, not error. The real contract is
    // that the shell waits and completes when the quote closes -- and
    // run_repl_lines_status waits for a READY marker after every line, so it cannot
    // express a line that deliberately does not complete. Testing that needs a harness
    // change, not a test.
    results.push(test("hostile_bare_pipe", Category::Hostile, || {
        let (out, _) = repl::run_repl_lines_status(&["echo hi |", "echo alive"], &[])?;
        expect_contains(&out.join("\n"), "alive")
    }));
    results.push(test("hostile_bare_operator", Category::Hostile, || {
        let (out, _) = repl::run_repl_lines_status(&["&&", "echo alive"], &[])?;
        expect_contains(&out.join("\n"), "alive")
    }));
    results.push(test(
        "hostile_very_long_argument",
        Category::Hostile,
        || {
            // 64KB in ONE word -- looks for a fixed buffer assumption in the lexer or argv.
            let big = "a".repeat(65536);
            let (out, _) =
                repl::run_repl_lines_status(&[&format!("echo {}", big), "echo alive"], &[])?;
            expect_contains(&out.join("\n"), "alive")
        },
    ));
    results.push(test(
        "hostile_nested_substitution",
        Category::Hostile,
        || {
            let (out, _) = repl::run_repl_lines_status(
                &["echo $(echo $(echo $(echo deep)))", "echo alive"],
                &[],
            )?;
            expect_contains(&out.join("\n"), "alive")
        },
    ));

    // CONTINUATION. fsh waits for an unterminated quote to close, exactly as bash does --
    // measured 2026-09-01 by piping two lines and a close into it: the quoted string came
    // back spanning both lines, then the next command ran, and the session counted three
    // commands. Two earlier cases asserted the opposite and timed out at 30s; they were
    // wrong about the shell, not the shell about the input.
    //
    // Whether the HARNESS can express it is the open question this case answers. Every
    // line waits for READY (bracketed-paste-on, the line editor asking for input) before
    // the next is sent. If rustyline re-announces that while continuing a command, this
    // passes and nothing needs changing. If it does not, this times out and the harness
    // needs a second synchronisation point.
    results.push(test(
        "hostile_quote_continuation",
        Category::Hostile,
        || {
            // THE CONSTRUCT IS LAST, deliberately: the capture window holds the final
            // command output only, so a trailing echo would be the thing captured.
            let (out, _) = repl::run_repl_buffered(&["echo 'first", "second'"], &[])?;
            let joined = out.join("\n");
            expect_contains(&joined, "first").and_then(|_| expect_contains(&joined, "second"))
        },
    ));

    // --- MULTI-LINE CONSTRUCTS ---
    //
    // Nothing has ever tested these. run_repl_multiline made them expressible for the
    // first time; expand.rs says the scanner handles for/while/if/case and paren, brace
    // and bracket blocks, and this is the first time anything has checked.
    //
    // The construct is LAST in every case, because the capture window holds the final
    // command's output only.
    results.push(test("multiline_for_loop", Category::Hostile, || {
        let (out, _) =
            repl::run_repl_multiline(&["for i in alpha beta", "do echo item-$i", "done"], &[])?;
        let joined = out.join("\n");
        expect_contains(&joined, "item-alpha").and_then(|_| expect_contains(&joined, "item-beta"))
    }));
    results.push(test("multiline_if_block", Category::Hostile, || {
        let (out, _) = repl::run_repl_multiline(&["if true", "then echo branch-taken", "fi"], &[])?;
        expect_contains(&out.join("\n"), "branch-taken")
    }));
    results.push(test("multiline_while_loop", Category::Hostile, || {
        let (out, _) = repl::run_repl_multiline(
            &[
                "n=0",
                "while [ $n -lt 2 ]",
                "do echo tick-$n",
                "n=$((n+1))",
                "done",
            ],
            &[],
        )?;
        let joined = out.join("\n");
        expect_contains(&joined, "tick-0").and_then(|_| expect_contains(&joined, "tick-1"))
    }));
    results.push(test("multiline_heredoc", Category::Hostile, || {
        // ⚠️ EXPECTED TO BE THE INTERESTING ONE. spine/lexer.rs says the scanner knows it
        // is inside a heredoc continuation and states plainly that this is AWARENESS, NOT
        // EXECUTION -- it does not claim fsh can run one. If this fails, that is a real
        // finding rather than a harness artifact, and the comment predicted it.
        let (out, _) = repl::run_repl_buffered(&["cat <<EOF", "heredoc-body", "EOF"], &[])?;
        expect_contains(&out.join("\n"), "heredoc-body")
    }));

    // ─── LEAK: text nsh treats as literal must not come out evaluated ───
    //
    // INT-236. A double-quoted string keeps its substitutions literal on ONE line and
    // EXECUTES them when the same string spans TWO. Measured 2026-09-01:
    //
    //     echo \"one `echo X`\"          -> one `echo X`   LITERAL
    //     echo \"one `echo X`\\ntwo\"    -> one X / two    EXECUTED
    //
    // THE COMPARISON IS THE ASSERTION. Each case runs the same payload twice -- once on
    // one line, once split across two -- and requires the two to agree. That needs no
    // table of expected strings and stays true if the quoting rules are ever changed
    // DELIBERATELY: a rule that changes changes both forms together.
    //
    // ⚠️ THE TWO-LINE FORM IS BUFFERED, NOT INCREMENTAL. rustyline stays inside one
    // read_line for a held quote and emits no intermediate prompt, so the harness must
    // write the whole construct. run_repl_multiline would time out at 30s here.
    //
    // NOT A TEST FOR ONE BUG -- a test for the CLASS. Any metacharacter that survives
    // quoting on one line and does not on two is the same defect wearing a different hat.
    for (name, payload, diverges) in [
        ("backtick", "`echo ZZL`", Some("INT-236: literal on one line, SUBSTITUTED across a line break. Dollar-paren does not do this -- it substitutes in both forms, which is consistent. Two syntaxes for one operation, disagreeing.")),
        ("dollar_paren", "$(echo ZZL)", None),
        ("variable", "$HOME", None),
        ("braced_variable", "${HOME}", None),
        ("semicolon", "a;b", None),
        ("and_operator", "a && b", None),
        ("pipe", "a | b", None),
        ("redirect", "a > zzleak.txt", None),
        ("glob", "*", None),
        ("brace_expansion", "{a,b}", None),
    ] {
        results.push(test(
            Box::leak(format!("leak_{}", name).into_boxed_str()),
            Category::Leak,
            move || {
                let one = repl::run_repl_lines_status(
                    &[&format!("echo \"AA {} BB\"", payload)],
                    &[],
                )?;
                let two = repl::run_repl_buffered(
                    &[&format!("echo \"AA {}", payload), " BB\""],
                    &[],
                )?;
                let a1 = one.0.join(" ");
                let a2 = two.0.join(" ");
                // Compare the PAYLOAD REGION only -- the two-line form legitimately
                // carries a newline the one-line form cannot have.
                let pick = |s: &str| -> String {
                    let t: String = s.chars().filter(|c| !c.is_whitespace()).collect();
                    match (t.find("AA"), t.rfind("BB")) {
                        (Some(i), Some(j)) if j > i => t[i + 2..j].to_string(),
                        _ => t,
                    }
                };
                let (p1, p2) = (pick(&a1), pick(&a2));
                // THE FOUR ARMS COME FROM INT-202, borrowed rather than reinvented: the
                // conformance cases already solved this. A KNOWN divergence must not block
                // a push, and it must not be silenced either. A declaration that starts
                // AGREEING fails too -- it means the defect was fixed and nobody removed
                // the note, so the next reader inherits a warning about something that no
                // longer happens.
                match (diverges, p1 == p2) {
                    (None, true) => Ok(()),
                    (Some(_), false) => Ok(()),
                    (None, false) => Err(format!(
                        "quoting does not survive the line break and nobody wrote down why -- one line gave {p1:?}, two lines gave {p2:?}"
                    )),
                    (Some(why), true) => Err(format!(
                        "this was declared to leak and now does not -- if it was fixed, remove the declaration: {why}"
                    )),
                }
            },
        ));
    }
    results.push(test(
        "repl_197_alias_expansion_reaches_the_safety_guard",
        Category::Repl,
        || {
            // INT-197 REMAINDER, and the gate above it was ticked while this was open.
            //
            // The guard receives TWO things: a word and a line. The WORD is derived from
            // guard_line, which is engine.expand_aliases(&line) -- so it is expanded. The LINE
            // passed as cmd is the TYPED one. main.rs:2683 computes the expansion, 2688 logs
            // it, and 2699 discards it and passes &line instead.
            //
            // ⭐ THE SCOPE IS NARROWER THAN IT FIRST LOOKED, and measuring corrected it twice.
            //
            // A WORD-ONLY rule survives aliasing. safety_guard.rs:100 matches
            // first_word.starts_with("mkfs"), and first_word IS expanded -- so
            // `alias a = mkfs.zzz` then `a` challenges, reporting "Filesystem format: a".
            // Verified 2026-09-02; a test with that payload passes and proves nothing.
            //
            // A RULE NEEDING BOTH HALVES DOES NOT. safety_guard.rs:69 is
            //     first_word == "rm" && lower.contains("-rf")
            // where first_word is expanded and `lower` comes from cmd, the TYPED line. Behind
            // an alias the word half matches and the line half cannot: the typed line is
            // `zzguard`, which contains no -rf. The rule silently fails open.
            //
            // Line 71 has the same shape -- cmd.contains(t) for the safe-target check.
            //
            // PAYLOAD, AND IT TOOK TWO WRONG ONES TO GET HERE.
            //
            // mkfs.zzz was wrong: word-only rule, passes without proving anything.
            // rm -rf /tmp/... was ALSO wrong, and worse -- safety_guard.rs:69 carries
            // safe_targets = ["/tmp/", "/tmp ", "target/", "./target"], so a /tmp payload is
            // the payload the guard DELIBERATELY PERMITS. The case went red and proved only
            // that /tmp is allowed. The path chosen to make a failed abort harmless was the
            // same path that made the rule not fire.
            //
            // /nonexistent-nsh-zzguard: outside every safe target, and absent, so a failed
            // abort removes nothing. A safety test whose payload is destructive when the
            // abort fails is not a safety test -- and one whose payload is allowlisted is
            // not a test at all.
            let out = repl::run_repl_answered_after(
                &["alias zzguard = 'rm -rf /nonexistent-nsh-zzguard'"],
                "zzguard",
                "Type 'yes' to proceed",
                "no",
            )?;
            let challenged = out.iter().any(|l| l.contains("CHALLENGE"));
            let blocked = out
                .iter()
                .any(|l| l.contains("Command blocked by Friday safety guard"));
            if challenged && blocked {
                Ok(())
            } else {
                Err(format!(
                    "an alias hid the payload from the guard (challenged={challenged}, blocked={blocked}) -- the guard judged the TYPED word, not the expansion: {out:?}"
                ))
            }
        },
    ));
    results.push(test(
        "dashc_reaches_the_safety_guard",
        Category::Regression,
        || {
            // ITEM 1: the guard was REPL-only. `nsh -c 'rm -rf /etc'` ran ungated while the
            // same line typed at a prompt was challenged, and the comment at the REPL site
            // claimed BEFORE any execution path. Measured 2026-09-02: the old binary exited 0
            // silently; the new one refuses and exits 1.
            //
            // `-c` REFUSES RATHER THAN PROMPTS. challenge_gate reads stdin for yes, and on this
            // door stdin belongs to the CALLER -- a pipe, a file, a closed descriptor. It was
            // already blocking closed by accident (EOF trims to empty), after printing a
            // question nobody could answer. Worse, a piped `yes` could have answered a
            // challenge the caller never saw.
            //
            // THE REFUSAL GOES TO STDERR. stdout belongs to the program on this door, which is
            // what keeps `nsh -c 'echo hi' | wc -l` answering 1 -- asserted below, because a
            // guard that contaminates stdout breaks the contract INT-201 established.
            //
            // Payload outside every safe_target and absent, so a failed refusal removes
            // nothing.
            let (out, err, code) = run_fsh_status("rm -rf /nonexistent-nsh-dashc-guard")?;
            let refused = err.contains("REFUSED") || out.contains("REFUSED");
            let named = err.contains("Destructive remove")
                || out.contains("Destructive remove");
            let nonzero = code != Some(0);
            if refused && named && nonzero {
                Ok(())
            } else {
                Err(format!(
                    "the -c door did not reach the guard (refused={refused}, named={named}, code={code:?}) -- out={out:?} err={err:?}"
                ))
            }
        },
    ));
    results.push(test(
        "zero_gate_runs_gitleaks_redacted",
        Category::Regression,
        || {
            // AGENTS.md, Security: every commit is scanned by zero-gate's pre-commit hook. A
            // throwaway repository stages a RANDOM fake token in GitHub's ghp_ format, and the
            // deployed zero-gate must report a gitleaks finding that does NOT repeat the token --
            // a secrets gate must not print the secret into a terminal's scrollback.
            let dir =
                std::env::temp_dir().join(format!("nsh-test-gitleaks-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("cannot make {}: {}", dir.display(), e))?;
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(7);
            let alphabet = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            let mut state = seed;
            let tail: String = (0..36)
                .map(|_| {
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    alphabet[((state >> 33) % alphabet.len() as u128) as usize] as char
                })
                .collect();
            let token = format!("ghp_{}", tail);
            let git = |args: &[&str]| Command::new("git").args(args).current_dir(&dir).output();
            let ready = git(&["init", "-q"])
                .map(|o| o.status.success())
                .unwrap_or(false)
                && std::fs::write(dir.join("leak.txt"), format!("token = {}\n", token)).is_ok()
                && git(&["add", "leak.txt"])
                    .map(|o| o.status.success())
                    .unwrap_or(false);
            if !ready {
                let _ = std::fs::remove_dir_all(&dir);
                return Err("could not build the throwaway repository".to_string());
            }
            let out = Command::new("zero-gate")
                .arg("pre-commit")
                .current_dir(&dir)
                .output()
                .map_err(|e| format!("cannot run zero-gate: {}", e));
            let _ = std::fs::remove_dir_all(&dir);
            let out = out?;
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if stderr.contains(&token) {
                return Err(
                    "zero-gate printed the secret itself: a scanner's finding repeats the token"
                        .to_string(),
                );
            }
            let line = stderr
                .lines()
                .find(|l| l.contains("gitleaks"))
                .map(str::to_string);
            match line {
                None => Err(format!(
                    "zero-gate reported no gitleaks finding:\n{}",
                    stderr.trim()
                )),
                Some(l) if l.contains(&token) => Err(
                    "the gitleaks finding repeats the secret instead of redacting it".to_string(),
                ),
                Some(l) if !l.contains('\u{2717}') => {
                    Err(format!("the gitleaks line is not a failure: {}", l))
                }
                Some(_) => Ok(()),
            }
        },
    ));
    results.push(test(
        "dashc_never_reaches_sudo",
        Category::Regression,
        || {
            // AGENTS.md, Sudo: never sudo rm, and nothing non-interactive elevates through nsh.
            // A FAKE sudo stands first on PATH and only writes a marker, so this case never runs
            // the real one. Before any sudo line is sent, the case asks nsh which sudo it would
            // run and refuses to continue unless the answer is the fake.
            //
            // NSH_CONFIG points at an EMPTY file, so this tests the CODE. A before_run rule in a
            // user's config.nsh (`contains "sudo rm"`) blocks the plain form on one machine; it
            // is untracked, untested, and misses `sudo -u root rm` and `sudo /usr/bin/rm`.
            let dir =
                std::env::temp_dir().join(format!("nsh-test-fake-sudo-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("cannot make {}: {}", dir.display(), e))?;
            let marker = dir.join("reached");
            let fake = dir.join("sudo");
            std::fs::write(
                &fake,
                format!("#!/bin/sh\ntouch '{}'\nexit 0\n", marker.display()),
            )
            .map_err(|e| format!("cannot write the fake sudo: {}", e))?;
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| format!("cannot chmod the fake sudo: {}", e))?;
            }
            let config = dir.join("empty-config.nsh");
            std::fs::write(&config, "")
                .map_err(|e| format!("cannot write the empty config: {}", e))?;
            let config = config.display().to_string();
            let path = format!(
                "{}:{}",
                dir.display(),
                std::env::var("PATH").unwrap_or_default()
            );
            let resolved = run_fsh_env(
                "command -v sudo",
                &[("PATH", path.as_str()), ("NSH_CONFIG", config.as_str())],
            )
            .unwrap_or_default();
            if resolved.trim() != fake.display().to_string() {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(format!(
                    "nsh would not run the fake sudo (it resolved {:?}), so nothing was sent",
                    resolved.trim()
                ));
            }
            let mut reached: Vec<&str> = Vec::new();
            for line in [
                "sudo rm /nonexistent-nsh-sudo-probe",
                "sudo -u root rm /nonexistent-nsh-sudo-probe",
                "sudo /usr/bin/rm /nonexistent-nsh-sudo-probe",
                "sudo true",
            ] {
                let _ = std::fs::remove_file(&marker);
                let _ = run_fsh_env(
                    line,
                    &[("PATH", path.as_str()), ("NSH_CONFIG", config.as_str())],
                );
                if marker.exists() {
                    reached.push(line);
                }
            }
            let _ = std::fs::remove_dir_all(&dir);
            if reached.is_empty() {
                Ok(())
            } else {
                Err(format!("nsh -c reached sudo for: {}", reached.join(" | ")))
            }
        },
    ));
    results.push(test(
        "dashc_guard_refusal_stays_off_stdout",
        Category::Regression,
        || {
            // The companion assertion, and the reason the refusal uses eprintln. A guard that
            // printed to stdout would break `nsh -c 'echo hi' | wc -l` == 1, which INT-201
            // established and the diagnostics rule protects.
            let (out, _, _) = run_fsh_status("echo hi")?;
            let lines = out.lines().filter(|l| !l.trim().is_empty()).count();
            if lines == 1 {
                Ok(())
            } else {
                Err(format!("stdout carried {lines} lines, not 1: {out:?}"))
            }
        },
    ));
    results.push(test(
        "repl_segment_after_and_reaches_the_guard",
        Category::Repl,
        || {
            // THE COMPOUND GAP. `zzguard2` alone challenges; `true && zzguard2` ran SILENTLY
            // until 2026-09-02. The line-level guard judges the WHOLE line, so its word is
            // `true` and the rm rule -- which needs the word AND the flags -- got neither.
            // Every segment after the first was outside the gate.
            //
            // split_into_segments stays the sole segmenter; the guard consumes its output at
            // the execution boundary rather than learning about `&&`.
            let out = repl::run_repl_answered_after(
                &["alias zzguard2 = 'rm -rf /nonexistent-nsh-zzguard2'"],
                "true && zzguard2",
                "Type 'yes' to proceed",
                "no",
            )?;
            let n = out.iter().filter(|l| l.contains("CHALLENGE")).count();
            if n >= 1 {
                Ok(())
            } else {
                Err(format!("no challenge for the executing segment: {out:?}"))
            }
        },
    ));
    results.push(test(
        "repl_skipped_segment_is_not_challenged",
        Category::Repl,
        || {
            // CONSTRAINT 2: the guard sits AFTER chain_skips, so a segment the chain will not
            // run is never judged. `false && rm -rf x` cannot execute the rm, and challenging
            // for it would be an alarm answered yes every time -- which teaches the answer.
            //
            // Not a policy invented for this case: the loop's `continue` at chain_skips draws
            // the line, and the guard is placed below it.
            let out = repl::run_repl_lines(&[
                "alias zzguard3 = 'rm -rf /nonexistent-nsh-zzguard3'",
                "false && zzguard3",
            ])?;
            let n = out.iter().filter(|l| l.contains("CHALLENGE")).count();
            if n == 0 {
                Ok(())
            } else {
                Err(format!("a segment the chain SKIPS was challenged: {out:?}"))
            }
        },
    ));
    results
}

fn store_results(results: &[TestResult]) {
    let db_path = zero_core::paths::state_db();
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        eprintln!("  ⚠️  could not open state.db -- results not stored");
        return;
    };
    let commit = std::process::Command::new("git")
        .args(["-C", &core_root(), "rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let nsh_version = std::process::Command::new(repl::fsh_bin())
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    // ⚠️ NOTHING IN THE TREE CREATED THIS TABLE. It existed only in state.db, put there by
    // hand or by code since removed, so a fresh machine ran the suite and stored NOTHING --
    // silently, because the insert below is best-effort and only counts what succeeded.
    //
    // THE INDEX IS THE SAME STORY AS shell_history, ONE DAY LATER. The banner reads this
    // table at every interactive start, filtering on the newest timestamp. With 110,582 rows
    // and no index that was a full SCAN: 60ms measured, 0.43ms with the index -- and the
    // index had to be created by hand to find that out, which is exactly why it belongs here.
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS nsh_test_results (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            test_name   TEXT NOT NULL,
            category    TEXT NOT NULL,
            passed      INTEGER NOT NULL,
            duration_ms INTEGER NOT NULL,
            commit_hash TEXT NOT NULL,
            timestamp   INTEGER NOT NULL,
            nsh_version TEXT
         );
         CREATE INDEX IF NOT EXISTS idx_nsh_test_results_timestamp
             ON nsh_test_results(timestamp);",
    );
    // ONE TRANSACTION, NOT 191 AUTOCOMMITS. config::apply learned this already -- 285 alias
    // inserts at roughly 0.7ms each was 210ms of startup until they were wrapped.
    let _ = conn.execute_batch("BEGIN");
    let mut stored = 0;
    for r in results {
        // A skipped case produces NO ROW. `passed INTEGER NOT NULL` has two states, and writing
        // 0 for a case that never ran would put the collapse back in the database.
        if r.skipped() {
            continue;
        }
        if conn.execute(
            "INSERT INTO nsh_test_results (test_name, category, passed, duration_ms, commit_hash, timestamp, nsh_version) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![r.name, r.category.to_string(), r.passed() as i32, r.duration_ms as i64, commit, ts, nsh_version],
        ).is_ok() { stored += 1; }
    }
    let _ = conn.execute_batch("COMMIT");
    println!("  💾 {} results stored in state.db", stored);
    // Phase 5: update Friday knowledge with test health
    let total = results.len();
    let passed_count = results.iter().filter(|r| r.passed()).count();
    let pass_rate = (passed_count * 100) / total.max(1);
    let _ = conn.execute(
        "INSERT OR REPLACE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
         VALUES ('testing', 'nsh_test_last_run', ?1, 0.95, 'nsh-test', ?2, ?2)",
        rusqlite::params![
            format!("nsh-test last run: {}/{} passed ({}%%). Commit: {}. All categories: heredoc, pipes, regression, tilde, vocabulary.", passed_count, total, pass_rate, commit),
            ts
        ],
    );
    if pass_rate < 100 {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
             VALUES ('testing', 'nsh_test_regression_alert', ?1, 0.99, 'nsh-test', ?2, ?2)",
            rusqlite::params![
                format!("ALERT: nsh-test regression detected. Only {}/{} tests passing ({}%%). Immediate attention required.", passed_count, total, pass_rate),
                ts
            ],
        );
    }
}

/// One conformance case: the line, and the reason fsh deliberately differs (or `None` if it must
/// match bash exactly).
///
/// ⚠️ THE DIVERGENCE REASONS ARE THE ASSET HERE, not the harness. Each records a decision someone
/// made on purpose, and a case that starts matching bash again means that decision was lost.
/// Carried over verbatim from spine/conform.rs when the suite moved (2026-08-03).
type ConformCase = (&'static str, Option<&'static str>);

const CONFORMANCE_CASES: &[ConformCase] = &[
    // --- pipelines: the construct the spine took over most recently ---
    ("echo hi | grep h", None),
    // ⚠️ DOUBLE-ESCAPED, matching the convention at pipe_with_grep above: the Rust literal must
    // deliver a literal backslash-n to the shell. With single backslashes it is a real multi-line
    // string, and run_repl submits ONE line -- the shell saw `printf 'a` and waited for a closing
    // quote, so the capture held prompt redraw instead of output. That is what "..." was.
    ("printf 'a\\nb\\nc\\n' | wc -l", None),
    ("echo one | grep one | wc -c", None),
    // POSIX says a pipeline's status is the LAST stage's. INT-189 settled this the hard way.
    ("false | true", None),
    ("true | false", None),
    // --- redirects ---
    ("echo written > /tmp/nsh_conform_a.txt; sed -n 1p /tmp/nsh_conform_a.txt", None),
    ("echo one > /tmp/nsh_conform_b.txt; echo two >> /tmp/nsh_conform_b.txt; sed -n 1,2p /tmp/nsh_conform_b.txt", None),
    // Truncation: the second write must REPLACE, not append.
    ("echo one > /tmp/nsh_conform_c.txt; echo two > /tmp/nsh_conform_c.txt; sed -n 1p /tmp/nsh_conform_c.txt", None),
    // --- file descriptors ---
    // ★ THIS ONE SURVIVES `ls`->`eza` because the semantic under test is "stderr is suppressed":
    // both shells print NOTHING on stdout whichever program the name resolves to.
    ("ls /nonexistent 2>/dev/null", None),
    // ⚠️ `sed`, NOT `ls`, AND THE REASON IS THE PRINCIPLE: fsh aliases `ls` to `eza`, so comparing
    // its error text against bash's compares eza against ls -- the behaviour of a different program,
    // not a shell semantic. The case is about whether `2>&1` sends stderr to the same file as
    // stdout, so it uses a command both shells resolve identically.
    ("sed -n 1p /nonexistent > /tmp/nsh_conform_d.txt 2>&1; sed -n 1p /tmp/nsh_conform_d.txt", None),
    // ⚠️ ADJACENCY: a SPACED numeral is an argument, not a descriptor.
    ("echo 2 > /tmp/nsh_conform_e.txt; sed -n 1p /tmp/nsh_conform_e.txt", None),
    // --- quoting ---
    ("echo \"a > b\"", None),
    ("echo \"a|b\"", None),
    // --- THE DECLARED DIVERGENCES. Matching bash here would be the regression. ---
    // ⭐ THESE TWO WERE THE ONLY DECLARED DIVERGENCES, AND THEY ARE GONE (2026-08-07). fsh used to
    // read any digit-initial or `=`-initial redirect target as a comparison, so that
    // `ps | where cpu > 0.5` kept working -- but that refused `echo test > 0.5` too, which has
    // nothing to do with the query language. The guard now asks whether the line is QUERY-SHAPED
    // (has any word so far named a value verb or source) before treating `>` as a comparison, so
    // both of these agree with bash and both write the file bash writes.
    ("echo test > 0.5", None),
    ("echo test >= x", None),
];

/// A stable test name from a case line: alphanumerics kept, everything else collapsed to `_`.
fn slug(line: &str) -> String {
    let mut s: String = line
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    s.truncate(40);
    s
}

// INT-256-EXEMPT: INT-219. Exits 2 with its own hook ON PURPOSE -- a truncated test run must not read as a complete one, and 141 would claim we died of SIGPIPE when we caught EPIPE and chose to stop.
fn main() {
    // INT-219: A TRUNCATED RUN MUST NOT LOOK LIKE A COMPLETE ONE.
    //
    // Rust ignores SIGPIPE, so a closed stdout consumer -- `| head` -- turns the next println into
    // an EPIPE panic. The suite stops partway, never prints its Results line, and the pipeline
    // reports the FILTER status rather than this process, so the whole thing reads as success.
    // That produced four wrong conclusions in one session on 2026-08-13: a case that appeared to
    // time out, a trace that appeared to print nothing, log files that appeared not to exist, and a
    // ghost-check that appeared not to discriminate. All one cause.
    //
    // ONE HOOK, NOT 200 CALL SITES. The payload is matched on TWO substrings rather than the whole
    // message, because the exact wording is a std implementation detail that moves between Rust
    // versions.
    //
    // EXIT 2, chosen from this tool own vocabulary -- 0 passes, 1 fails, nothing above claimed it.
    // NOT 141: that would claim we died from SIGPIPE, and we did not; we caught EPIPE and chose to
    // stop. The status must be distinct and testable, not conventional.
    //
    // ⚠️ THE STDERR MARKER IS WHAT SURVIVES A PIPELINE, since the exit status there belongs to the
    // filter. See INT-220: fsh currently drops a stderr redirect inside a pipeline, so this marker
    // reaches a terminal and a bash redirect but not yet an fsh one.
    {
        let prior = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let msg = info.to_string();
            if msg.contains("failed printing to stdout") && msg.contains("Broken pipe") {
                eprintln!(
                    "nsh-test: TRUNCATED -- stdout consumer closed before the suite completed"
                );
                std::process::exit(2);
            }
            prior(info);
        }));
    }

    let args: Vec<String> = std::env::args().collect();
    // AN IDENTITY QUESTION IS ANSWERED BEFORE THE WORK STARTS.
    //
    // Unrecognised arguments fell through to "run everything", so `nsh-test help` ran
    // all 193 cases for 31 seconds. The INT-249 census read that as a HANG and filed
    // it UNDETERMINED against a 10-second timeout -- a tool that runs a full suite
    // when asked its own name cannot be surveyed, installed by anything automated, or
    // safely poked at by someone who does not know its flags yet.
    //
    // Same ruling as zero-gate on 2026-09-15: what a tool can answer without doing its
    // job, it answers first.
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("nsh-test {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("nsh-test -- the NovaShell behaviour suite");
        println!();
        println!("  nsh-test                  run every case");
        println!("  nsh-test --failed         show only what failed");
        println!("  nsh-test --category=<c>   one category (repl, pipes, hostile, ...)");
        println!();
        println!("The shell under test is NSH_BIN, and the suite ASKS it who it is rather");
        println!("than trusting the path. A path that exists and is the wrong build is the");
        println!("case that cost a session.");
        return;
    }

    let show_only_failed = args.contains(&"--failed".to_string());
    let category_filter = args
        .iter()
        .find(|a| a.starts_with("--category="))
        .map(|a| a.trim_start_matches("--category=").to_string());

    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!(
        "{}",
        concat!("  nsh-test v", env!("CARGO_PKG_VERSION")).bold()
    );
    // ASK THE SHELL WHO IT IS, rather than trusting the path we passed it. Refusing a
    // MISSING binary catches a typo; this catches the case that actually cost a session --
    // a path that exists and is the WRONG BUILD. /run/current-system/... exists perfectly
    // well, which is exactly why the silent fallback was invisible.
    //
    // The BASH_VERSION pattern: bash exposes its identity so a script can prove which shell
    // it reached. fsh now exposes NSH_VERSION and NSH_BUILD for the same reason.
    {
        let ident = std::process::Command::new(repl::fsh_bin())
            // ⚠️ CLEARED FIRST, OR THE ANSWER IS THE PARENT'S. These are exported at
            // startup, so a child INHERITS them -- and a shell too old to set them would
            // echo ours back and report an identity it does not have. Measured the moment
            // this banner was added: it named the debug build while testing the deployed
            // one. An empty answer now means the shell cannot identify itself, which is
            // the honest result and a detectable one.
            .env_remove("NSH_VERSION")
            .env_remove("NSH_BUILD")
            .arg("-c")
            .arg("echo $NSH_VERSION $NSH_BUILD")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|e| format!("could not ask: {}", e));
        // The BUILD PATH is not shown here. It matters only when the byte comparison below
        // disagrees, and that block prints both paths where they can be acted on. In the
        // normal case the version is the identity and the path is noise.
        //
        // ident keeps both halves because the comparison needs the path; only the display
        // narrows.
        let shown = ident.split_whitespace().next().unwrap_or(&ident);
        println!("  under test: {}", shown);

        // ⚠️ ASKING WAS NEVER THE PROBLEM -- NOT COMPARING WAS. The line above has always been
        // honest, and on 2026-08-24 a full green run reported `3.8.1` while the change under
        // examination was in 3.8.4. The banner said so and was read past, three times in one
        // session, because a version printed among other version-shaped text does not announce
        // that it is the WRONG one.
        //
        // ⭐ SO THE SUITE STATES THE CONSEQUENCE rather than the fact. nsh-test is versioned in
        // lockstep with the shell in this workspace, so its own CARGO_PKG_VERSION is a usable
        // expectation. A mismatch is not an error -- testing a deployed build on purpose is
        // legitimate -- but it MUST NOT look like a run that covered your working tree.
        // ⚠️ NOT nsh-test's OWN VERSION -- it is 2.0.0 while the shell is 3.8.x, so comparing
        // them would warn on EVERY run, and a warning that always fires is one nobody reads.
        // That is the failure risk-gate.sh already recorded about gates people route around.
        //
        // ⭐ THE RIGHT EXPECTATION IS THE SHELL'S VERSION IN THIS WORKSPACE, read from its
        // Cargo.toml at RUNTIME rather than baked in: the suite and the shell are separate crates
        // that version independently, so a compile-time constant would go stale silently.
        let expected = std::fs::read_to_string("zero/shell/novashell/Cargo.toml")
            .ok()
            .and_then(|t| {
                t.lines()
                    .find(|l| l.trim_start().starts_with("version"))
                    .and_then(|l| l.split('"').nth(1).map(str::to_string))
            })
            .unwrap_or_default();
        let saw_version = ident.split_whitespace().next().unwrap_or("");
        // ⚠️ BOTH SIDES MUST BE KNOWN. An unreadable Cargo.toml (running from another directory)
        // leaves `expected` empty, and comparing against nothing would warn on every run from
        // outside the repo -- the same noise problem, arriving by a different door. Unknown stays
        // unknown and says nothing.
        if !saw_version.is_empty() && !expected.is_empty() && saw_version != expected {
            println!(
                "  ⚠️  this suite is {expected}; the binary above is {saw_version} -- UNBUILT CHANGES ARE NOT COVERED"
            );
            println!(
                "     to test what you just built: NSH_BIN=target/debug/nsh ./target/debug/nsh-test"
            );
        }

        // ⭐ THE VERSION CANNOT SEE A MID-VERSION REBUILD, and that is every case that cost
        // time on 2026-09-01. Five stale-binary confusions in one session, all at 3.8.4
        // against source that was also 3.8.4 -- the check above was correct, fired on
        // nothing, and each failure looked like a regression instead.
        //
        // COMPARE BYTES, NOT LABELS. If what cargo last built differs from what is being
        // tested, the suite is measuring something other than the working tree.
        //
        // WARN RATHER THAN REFUSE, for the reason the version check already gives: testing a
        // deployed build on purpose is legitimate. What must not happen is a green run that
        // LOOKS like it covered your changes.
        //
        // Silent when the release artifact is absent -- a clean tree has nothing to compare,
        // and a warning that fires on every fresh clone is one nobody reads.
        let built = zero_core::paths::core_dir().join("target/release/nsh");
        let tested = std::path::PathBuf::from(repl::fsh_bin());
        if built.exists() && tested.exists() && zero_core::differs(&built, &tested) {
            println!(
                "  ⚠️  the binary above is NOT what cargo last built -- SAME VERSION, DIFFERENT BYTES"
            );
            println!("     built:  {}", built.display());
            println!("     tested: {}", tested.display());
            println!("     ship nsh, or point NSH_BIN at the build you meant");
        }
    }
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    let results = all_tests();
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for r in &results {
        if let Some(ref cat) = category_filter {
            if r.category.to_string() != *cat {
                continue;
            }
        }
        if show_only_failed && r.passed() {
            continue;
        }

        let status = match r.outcome {
            Outcome::Passed => "\u{2705}".to_string(),
            Outcome::Failed => "\u{274c}".to_string(),
            // Not a tick and not a cross. A skip rendered as either is the collapse the
            // Outcome enum exists to prevent.
            Outcome::Skipped(_) => "\u{2754}".to_string(),
        };

        println!(
            "  {} [{:>11}] {} {}ms",
            status,
            r.category.to_string().dimmed(),
            r.name,
            r.duration_ms.to_string().dimmed()
        );

        match r.outcome {
            Outcome::Passed => passed += 1,
            Outcome::Failed => {
                if let Some(ref err) = r.error {
                    println!("      {}", err.red());
                }
                failed += 1;
            }
            Outcome::Skipped(why) => {
                println!("      {}", why.dimmed());
                skipped += 1;
            }
        }
    }

    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!(
        "  Results: {} / {} passed",
        passed.to_string().green().bold(),
        (passed + failed).to_string().bold()
    );
    // Reported SEPARATELY and NOT in the denominator. A case that could not run has not passed
    // and has not failed; folding it into either number makes the headline lie one way or the
    // other.
    if skipped > 0 {
        println!(
            "  {} skipped -- a precondition was absent, not a failure",
            skipped.to_string().yellow().bold()
        );
    }
    store_results(&results);
    // Phase 5: coverage reporting
    if args.contains(&"--coverage".to_string()) {
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
        );
        println!("{}", "  📊 Coverage Report".bold());
        let categories = [
            "tilde",
            "pipes",
            "vocabulary",
            "heredoc",
            "regression",
            "performance",
        ];
        for cat in &categories {
            let count = results
                .iter()
                .filter(|r| r.category.to_string() == *cat)
                .count();
            let passed = results
                .iter()
                .filter(|r| r.category.to_string() == *cat && r.passed())
                .count();
            let pct = if count > 0 { (passed * 100) / count } else { 0 };
            let bar = "█".repeat(pct / 10);
            println!(
                "  [{:>11}] {}/{} {}% {}",
                cat.dimmed(),
                passed,
                count,
                pct,
                bar.green()
            );
        }
        println!("");
        println!("  Vocabulary words tested: delete, find, list, gt, fsearch, where");
        println!("  Untested paths: parallel blocks, signal handling, fd leak detection");
    }
    // Phase 3: performance summary
    if args.contains(&"--perf".to_string()) {
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
        );
        println!("{}", "  ⏱️  Performance Summary".bold());
        let mut by_cat: std::collections::HashMap<String, Vec<u64>> =
            std::collections::HashMap::new();
        for r in &results {
            by_cat
                .entry(r.category.to_string())
                .or_default()
                .push(r.duration_ms);
        }
        let mut cats: Vec<_> = by_cat.iter().collect();
        cats.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (cat, times) in &cats {
            let avg = times.iter().sum::<u64>() / times.len() as u64;
            let max = times.iter().max().unwrap_or(&0);
            println!(
                "  [{:>11}] avg: {}ms  max: {}ms  count: {}",
                cat.dimmed(),
                avg,
                max,
                times.len()
            );
        }
    }
    // INT-204: the run owns one directory of per-case databases; remove it now the run is over.
    //
    // ⚠️ BEFORE the branch below, not after it -- that branch exits the process, so a cleanup placed
    // after it would never run on a RED run, which is precisely when files get left behind. The same
    // trap this harness already records against its own cases: clean on the path that always
    // executes, not the happy one.
    //
    // A crash still leaves them, and that is deliberate -- a run that collapsed is one you want to be
    // able to look inside.
    let _ = std::fs::remove_dir_all(repl::case_db_dir());
    // The fixture goes with it, and for the same reason it is removed HERE rather than after the
    // exit branch below: that branch never runs on a red run, which is exactly when files get
    // left behind.
    let _ = std::fs::remove_dir_all(fixture_dir());
    if failed > 0 {
        println!("  {} tests failed", failed.to_string().red().bold());
        std::process::exit(1);
    } else {
        println!("  {}", "✅ All tests passed".green().bold());
    }
}

// Additional tests will be added here via append

// This won't work as append - need to insert before main()
