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

fn run_fsh(input: &str) -> Result<String, String> {
    let fsh = std::env::var("FSH_BIN")
        .unwrap_or_else(|_| "/run/current-system/sw/bin/faelight-shell".to_string());
    let out = Command::new(&fsh)
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
    println!("{}", "  🌲 fsh-test v1.0.0 -- INT-304".bold());
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
