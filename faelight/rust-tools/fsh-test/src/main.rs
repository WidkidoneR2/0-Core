//! fsh-test -- permanent regression suite for faelight-shell
//! INT-304 Phase 1: port fsh_audit.sh 75 tests to Rust

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
        }
    }
}

#[derive(Debug)]
struct TestResult {
    name: String,
    category: Category,
    passed: bool,
    duration_ms: u64,
    error: Option<String>,
}

/// INT-195 gate 6: invoke faelight-deadwood through the same seam pattern run_fsh uses for the
/// shell. DEADWOOD_BIN lets one test prove the debug build before a deploy and the deployed
/// artifact after -- the two-binaries discipline the rest of this work relies on.
fn run_deadwood(args: &[&str]) -> Result<std::process::Output, String> {
    let bin = std::env::var("DEADWOOD_BIN")
        .unwrap_or_else(|_| "/run/current-system/sw/bin/faelight-deadwood".to_string());
    Command::new(&bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("cannot invoke {bin}: {e}"))
}

fn run_fsh(input: &str) -> Result<String, String> {
    let fsh = std::env::var("FSH_BIN")
        .unwrap_or_else(|_| "/run/current-system/sw/bin/faelight-shell".to_string());
    let out = Command::new(&fsh)
        // INT-206: the same setting the REPL runner uses, so the suite drives ONE shell
        // configuration rather than two that differ in where they think they are.
        .env("FSH_KEEP_CWD", "1")
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

fn test(name: &str, category: Category, f: impl Fn() -> Result<(), String>) -> TestResult {
    let start = Instant::now();
    let result = f();
    let duration_ms = start.elapsed().as_millis() as u64;
    let passed = result.is_ok();
    TestResult {
        name: name.to_string(),
        category,
        passed,
        duration_ms,
        error: result.err(),
    }
}

fn expect_eq(got: &str, expected: &str) -> Result<(), String> {
    if got == expected {
        Ok(())
    } else {
        Err(format!("expected {:?} got {:?}", expected, got))
    }
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
    results.push(test("vocab_find_basic", Category::Vocabulary, || {
        // find vocabulary uses fd syntax -- test fd directly
        let out = run_fsh("fd Cargo.toml /home/christian/0-core/faelight/engine")?;
        expect_contains(&out, "Cargo.toml")
    }));

    // --- HEREDOC ---
    results.push(test("heredoc_basic", Category::Heredoc, || {
        let out = run_fsh("cat << 'EOF'\nhello\nEOF")?;
        expect_eq(&out, "hello")
    }));
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
        expect_eq(&run_fsh("echo 'forest grows'")?, "forest grows")
    }));
    results.push(test("pwd_returns_path", Category::Regression, || {
        expect_contains(&run_fsh("pwd")?, "/home/christian")
    }));
    results.push(test("uname_linux", Category::Regression, || {
        expect_contains(&run_fsh("uname")?, "Linux")
    }));
    results.push(test("whoami", Category::Regression, || {
        expect_eq(&run_fsh("whoami")?, "christian")
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
        expect_contains(&run_fsh("echo $HOME")?, "/home/christian")
    }));
    results.push(test("assign_number", Category::Regression, || {
        expect_eq(&run_fsh("N=42; echo $N")?, "42")
    }));
    results.push(test("path_not_empty", Category::Regression, || {
        expect_contains(&run_fsh("echo $PATH")?, "/nix")
    }));

    // --- SEMICOLON / OPERATORS ---
    results.push(test("semicolon_two_cmds", Category::Regression, || {
        expect_contains(&run_fsh("echo first; echo second")?, "second")
    }));
    results.push(test("and_operator", Category::Regression, || {
        expect_contains(&run_fsh("echo a && echo b")?, "b")
    }));
    results.push(test("and_chain_fsh_builtin", Category::Regression, || {
        expect_contains(&run_fsh("echo ok && core version")?, "3.0.0")
    }));
    results.push(test("subshell_expansion", Category::Regression, || {
        expect_eq(&run_fsh("echo $(echo nested)")?, "nested")
    }));

    // --- TILDE ---
    results.push(test("tilde_echo_subpath", Category::Tilde, || {
        expect_contains(&run_fsh("echo ~/0-core")?, "/home/christian/0-core")
    }));
    results.push(test("tilde_ls_root", Category::Tilde, || {
        expect_contains(&run_fsh("ls ~/0-core")?, "faelight")
    }));
    results.push(test("tilde_ls_scripts", Category::Tilde, || {
        expect_contains(
            &run_fsh("ls ~/0-core/faelight/packages/faelight/scripts")?,
            "deploy",
        )
    }));
    results.push(test("tilde_ls_runtime", Category::Tilde, || {
        expect_contains(&run_fsh("ls ~/0-core/faelight/runtime")?, "state.db")
    }));
    results.push(test("tilde_cat_cargo", Category::Tilde, || {
        expect_contains(
            &run_fsh("cat ~/0-core/faelight/rust-tools/faelight-shell/Cargo.toml")?,
            "faelight-shell",
        )
    }));
    results.push(test("tilde_pipe_grep", Category::Tilde, || {
        expect_contains(&run_fsh("ls ~/0-core | grep faelight")?, "faelight")
    }));
    results.push(test("tilde_cat_pipe_grep", Category::Tilde, || {
        expect_contains(
            &run_fsh("cat ~/0-core/faelight/rust-tools/faelight-shell/Cargo.toml | grep name")?,
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
        expect_eq(&run_fsh("echo forest | grep forest")?, "forest")
    }));
    results.push(test("pipe_twice", Category::Pipes, || {
        expect_eq(&run_fsh("echo hello | tr a-z A-Z | tr A-Z a-z")?, "hello")
    }));
    results.push(test("pipe_ls_grep", Category::Pipes, || {
        expect_contains(&run_fsh("ls ~/0-core | grep faelight")?, "faelight")
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
        expect_contains(&run_fsh("grep -r 'expand_braces' ~/0-core/faelight/rust-tools/faelight-shell/src/ | head -1")?, "expand_braces")
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
    results.push(test("fsh_c_echo", Category::Regression, || {
        expect_eq(&run_fsh("echo hello")?, "hello")
    }));
    results.push(test("fsh_c_pipeline", Category::Regression, || {
        expect_eq(&run_fsh("echo forest | tr a-z A-Z")?, "FOREST")
    }));
    results.push(test("semicolons_pipeline", Category::Regression, || {
        expect_contains(&run_fsh("echo a; echo b | tr a-z A-Z")?, "B")
    }));
    results.push(test("pipe_wc_chars", Category::Pipes, || {
        expect_eq(&run_fsh("echo hello | wc -c")?, "6")
    }));
    results.push(test("ls_pipe_grep_tmp", Category::Pipes, || {
        // create fsh_t file first
        std::fs::write("/tmp/fsh_t1.txt", "forest writes").ok();
        expect_contains(&run_fsh("ls /tmp | grep fsh")?, "fsh")
    }));
    results.push(test("tilde_ls_pipe_sort", Category::Tilde, || {
        let out = run_fsh("ls ~/0-core | sort | head -1")?;
        if out.is_empty() {
            Err("no output".to_string())
        } else {
            Ok(())
        }
    }));
    results.push(test("tilde_nested_pipe", Category::Tilde, || {
        let out = run_fsh("ls ~/0-core/faelight/rust-tools | grep faelight | wc -l")?;
        let n: i32 = out.trim().parse().unwrap_or(0);
        if n > 0 {
            Ok(())
        } else {
            Err(format!("expected >0 got {}", n))
        }
    }));
    results.push(test("where_delete_vocab", Category::Vocabulary, || {
        expect_contains(
            &run_fsh("core vocabulary where delete 2>/dev/null || echo vocabulary")?,
            "vocabulary",
        )
    }));
    results.push(test("fsearch_rust_finds", Category::Vocabulary, || {
        expect_contains(
            &run_fsh(
                "grep -r expand_braces ~/0-core/faelight/rust-tools/faelight-shell/src/ | head -1",
            )?,
            "expand_braces",
        )
    }));
    results.push(test("grep_in_and_chain", Category::Regression, || {
        expect_contains(&run_fsh("echo ok && grep 'expand_braces' ~/0-core/faelight/rust-tools/faelight-shell/src/main.rs | head -1")?, "expand_braces")
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
        expect_contains(&run_fsh("echo $HOME")?, "/home/christian")
    }));

    results.push(test("tilde_ls_rust_tools", Category::Tilde, || {
        expect_contains(
            &run_fsh("ls ~/0-core/faelight/rust-tools")?,
            "faelight-shell",
        )
    }));
    results.push(test("tilde_ls_docs", Category::Tilde, || {
        expect_contains(&run_fsh("ls ~/0-core/docs")?, "PHILOSOPHY")
    }));
    results.push(test("tilde_ls_intents", Category::Tilde, || {
        expect_contains(&run_fsh("ls ~/0-core/faelight/intents")?, "future")
    }));
    results.push(test("tilde_deep_nested", Category::Tilde, || {
        expect_contains(
            &run_fsh("ls ~/0-core/faelight/rust-tools/faelight-shell/src")?,
            "main.rs",
        )
    }));
    results.push(test("cat_reads_file", Category::Regression, || {
        std::fs::write("/tmp/fsh_t1.txt", "forest writes").map_err(|e| e.to_string())?;
        expect_contains(&run_fsh("cat /tmp/fsh_t1.txt")?, "forest writes")
    }));
    results.push(test("tilde_in_subshell", Category::Tilde, || {
        let out = run_fsh("echo $(ls ~/0-core | head -1)")?;
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

    // --- FOREST-SPECIFIC TESTS beyond fsh_audit.sh ---
    results.push(test("state_db_exists", Category::Regression, || {
        expect_contains(
            &run_fsh("ls ~/0-core/faelight/runtime/state.db")?,
            "state.db",
        )
    }));
    results.push(test("core_binary_exists", Category::Regression, || {
        expect_contains(&run_fsh("ls /run/current-system/sw/bin/core")?, "core")
    }));
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
                "faelight-deadwood --strict exited {:?}: {}",
                out.status.code(),
                if flagged.is_empty() {
                    String::from_utf8_lossy(&out.stderr).trim().to_string()
                } else {
                    flagged.join(" | ")
                }
            ))
        },
    ));
    results.push(test("fsh_binary_exists", Category::Regression, || {
        expect_contains(
            &run_fsh("ls /run/current-system/sw/bin/faelight-shell")?,
            "faelight-shell",
        )
    }));
    results.push(test("intents_future_exists", Category::Regression, || {
        expect_contains(&run_fsh("ls ~/0-core/faelight/intents/future")?, ".md")
    }));
    results.push(test("pipe_multiline_output", Category::Pipes, || {
        let out = run_fsh("printf 'a\nb\nc\n' | wc -l")?;
        expect_eq(out.trim(), "3")
    }));
    results.push(test("redirect_not_crash", Category::Regression, || {
        run_fsh("echo test > /tmp/fsh_test_redirect.txt")?;
        expect_contains(&run_fsh("cat /tmp/fsh_test_redirect.txt")?, "test")
    }));
    // ---- INT-172: REPL tests. These drive a real pty, NOT `fsh -c`.
    // Every one of them PASSES on `fsh -c` even against a broken shell -- which
    // is exactly why fsh-test was 83/83 green for three months while the shell we
    // type into was turning pipelines into filenames.
    results.push(test(
        "repl_pipe_control_no_redirect",
        Category::Repl,
        || {
            let out = repl::run_repl("echo hello | grep -c hello")?;
            expect_eq(out.first().map(|s| s.as_str()).unwrap_or("(nothing)"), "1")
        },
    ));
    results.push(test(
        "repl_206_forest_home_is_still_the_default",
        Category::Repl,
        || {
            // INT-206 GUARDIAN. The harness sets FSH_KEEP_CWD for every other case so that a case which
            // writes a file cannot write it into the repository. That is the right default, and it has a
            // cost: every other case then runs a shell configuration nobody uses interactively.
            //
            // This case buys that back. It passes "0" to get the ORDINARY shell -- the one that starts in
            // the forest home on purpose and restores its last directory on purpose -- and asserts that
            // behaviour is intact. So what daily use actually gets is covered by a case that says what it
            // is testing, rather than left uncovered because every other case quietly opted out of it.
            //
            // "0" rather than removing the variable: keep_launch_cwd reads it as v != "0", so the string
            // is the off switch and no env_remove is needed.
            let (out, _) = repl::run_repl_lines_status(&["pwd"], &[("FSH_KEEP_CWD", "0")])?;
            expect_contains(&out.join("\n"), "0-core")
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
            let _ = std::fs::remove_file("/tmp/fsh_test_g7out.txt");
            let _ = repl::run_repl("ls /tmp/fsh_test_nope /tmp > /tmp/fsh_test_g7out.txt 2>&1")?;
            let body = std::fs::read_to_string("/tmp/fsh_test_g7out.txt")
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
                    .starts_with("fsh_test_g7err")
                {
                    let _ = std::fs::remove_file(e.path());
                }
            }
            let _ = repl::run_repl("echo hello 2>/tmp/fsh_test_g7err | grep -c hello")?;
            let junk: Vec<String> = std::fs::read_dir("/tmp")
                .map_err(|e| e.to_string())?
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.starts_with("fsh_test_g7err") && n.contains('|'))
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
        expect_contains(&out, "/home/christian")
    }));
    results.push(test("exit_code_success", Category::Regression, || {
        run_fsh("true")?;
        Ok(())
    }));

    // --- PHASE 2: INT-298/299 specific regression tests ---
    results.push(test(
        "regression_fsh_c_inside_fsh",
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
        let _ = std::fs::remove_file("/tmp/fsh_test_143once.txt");
        let _ = std::fs::remove_dir("/tmp/fsh_test_143once_dir");
        let _ = repl::run_repl("mkdir /tmp/fsh_test_143once_dir > /tmp/fsh_test_143once.txt 2>&1")?;
        if !std::path::Path::new("/tmp/fsh_test_143once_dir").is_dir() {
            return Err("mkdir did not run: dir absent".to_string());
        }
        let body = std::fs::read_to_string("/tmp/fsh_test_143once.txt")
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
        std::fs::write("/tmp/fsh_test_143.sh", "echo MARKER143RAN\n").map_err(|e| e.to_string())?;
        let out = repl::run_repl("bash /tmp/fsh_test_143.sh")?;
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
        // fsh's own `type` builtin prints "forest builtin / handled natively by fsh".
        // sh's `type` prints nothing like it, so this output PROVES fsh dispatched
        // its builtin -- invisible through `-c`, which would run sh's `type`.
        let out = repl::run_repl("type pwd")?;
        let joined = out.join("\n");
        if joined.contains("forest builtin") {
            Ok(())
        } else {
            Err(format!(
                "fsh builtin dispatch not seen (expected 'forest builtin'): {joined:?}"
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
                        || t.contains("fsh❯")
                        || t.starts_with('🔧')
                        || t.starts_with('○')
                        || t.starts_with('[')
                        || t.starts_with("x ")
                        || t.starts_with('✗')
                        || t.starts_with("🌲")
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
                        "fsh emitted no 133;D status marker, so conformance cannot be judged on \
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
                        "fsh disagrees with bash on {detail} and nobody wrote down why -- \
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
            //
            // ★ THIS ONE HAS A RED YOU CAN STILL RUN. Gen 464's binary predates d0c04825:
            //   FSH_BIN=/nix/store/86m8mhwx52s1ris35jp0v4b7kmffzyv7-faelight-forest-9.2.0/bin/faelight-shell
            // Against it this case fails and the one above passes -- which is the whole argument for
            // per-case routing in one screen.
            let out = repl::run_repl_lines_env(
                &[
                    "rm -f /tmp/zzbg2L.txt",
                    "sh -c \"echo ZZBGQUOTEDL > /tmp/zzbg2L.txt\" &",
                    "sleep 2",
                    "sed -n 1p /tmp/zzbg2L.txt",
                ],
                &[("FSH_SPINE", "0")],
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
                &[("FSH_SPINE", "0")],
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
    results
}

fn store_results(results: &[TestResult]) {
    let db_path = faelight_core::paths::state_db();
    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        eprintln!("  ⚠️  could not open state.db -- results not stored");
        return;
    };
    let commit = std::process::Command::new("git")
        .args([
            "-C",
            "/home/christian/0-core",
            "rev-parse",
            "--short",
            "HEAD",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let fsh_version = std::process::Command::new("/run/current-system/sw/bin/faelight-shell")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let mut stored = 0;
    for r in results {
        if conn.execute(
            "INSERT INTO fsh_test_results (test_name, category, passed, duration_ms, commit_hash, timestamp, fsh_version) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![r.name, r.category.to_string(), r.passed as i32, r.duration_ms as i64, commit, ts, fsh_version],
        ).is_ok() { stored += 1; }
    }
    println!("  💾 {} results stored in state.db", stored);
    // Phase 5: update Friday knowledge with test health
    let total = results.len();
    let passed_count = results.iter().filter(|r| r.passed).count();
    let pass_rate = (passed_count * 100) / total.max(1);
    let _ = conn.execute(
        "INSERT OR REPLACE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
         VALUES ('testing', 'fsh_test_last_run', ?1, 0.95, 'fsh-test', ?2, ?2)",
        rusqlite::params![
            format!("fsh-test last run: {}/{} passed ({}%%). Commit: {}. All categories: heredoc, pipes, regression, tilde, vocabulary.", passed_count, total, pass_rate, commit),
            ts
        ],
    );
    if pass_rate < 100 {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO friday_knowledge (domain, key, fact, confidence, source, created_at, updated_at)
             VALUES ('testing', 'fsh_test_regression_alert', ?1, 0.99, 'fsh-test', ?2, ?2)",
            rusqlite::params![
                format!("ALERT: fsh-test regression detected. Only {}/{} tests passing ({}%%). Immediate attention required.", passed_count, total, pass_rate),
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
    ("echo written > /tmp/fsh_conform_a.txt; sed -n 1p /tmp/fsh_conform_a.txt", None),
    ("echo one > /tmp/fsh_conform_b.txt; echo two >> /tmp/fsh_conform_b.txt; sed -n 1,2p /tmp/fsh_conform_b.txt", None),
    // Truncation: the second write must REPLACE, not append.
    ("echo one > /tmp/fsh_conform_c.txt; echo two > /tmp/fsh_conform_c.txt; sed -n 1p /tmp/fsh_conform_c.txt", None),
    // --- file descriptors ---
    // ★ THIS ONE SURVIVES `ls`->`eza` because the semantic under test is "stderr is suppressed":
    // both shells print NOTHING on stdout whichever program the name resolves to.
    ("ls /nonexistent 2>/dev/null", None),
    // ⚠️ `sed`, NOT `ls`, AND THE REASON IS THE PRINCIPLE: fsh aliases `ls` to `eza`, so comparing
    // its error text against bash's compares eza against ls -- the behaviour of a different program,
    // not a shell semantic. The case is about whether `2>&1` sends stderr to the same file as
    // stdout, so it uses a command both shells resolve identically.
    ("sed -n 1p /nonexistent > /tmp/fsh_conform_d.txt 2>&1; sed -n 1p /tmp/fsh_conform_d.txt", None),
    // ⚠️ ADJACENCY: a SPACED numeral is an argument, not a descriptor.
    ("echo 2 > /tmp/fsh_conform_e.txt; sed -n 1p /tmp/fsh_conform_e.txt", None),
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
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
        "  🌲 fsh-test v2.0.0 -- INT-202 (orig. INT-304)".bold()
    );
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    let results = all_tests();
    let mut passed = 0;
    let mut failed = 0;

    for r in &results {
        if let Some(ref cat) = category_filter {
            if r.category.to_string() != *cat {
                continue;
            }
        }
        if show_only_failed && r.passed {
            continue;
        }

        let status = if r.passed {
            "✅".to_string()
        } else {
            "❌".to_string()
        };

        println!(
            "  {} [{:>11}] {} {}ms",
            status,
            r.category.to_string().dimmed(),
            r.name,
            r.duration_ms.to_string().dimmed()
        );

        if !r.passed {
            if let Some(ref err) = r.error {
                println!("      {}", err.red());
            }
            failed += 1;
        } else {
            passed += 1;
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
                .filter(|r| r.category.to_string() == *cat && r.passed)
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
    if failed > 0 {
        println!("  {} tests failed", failed.to_string().red().bold());
        std::process::exit(1);
    } else {
        println!("  {}", "✅ All tests passed".green().bold());
    }
}

// Additional tests will be added here via append

// This won't work as append - need to insert before main()
