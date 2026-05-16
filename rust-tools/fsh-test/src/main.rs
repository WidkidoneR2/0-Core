//! fsh-test -- permanent regression suite for faelight-shell
//! INT-304 Phase 1: port fsh_audit.sh 75 tests to Rust

use colored::*;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
enum Category {
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
            Category::Tilde      => write!(f, "tilde"),
            Category::Pipes      => write!(f, "pipes"),
            Category::Vocabulary => write!(f, "vocabulary"),
            Category::Heredoc    => write!(f, "heredoc"),
            Category::Signals    => write!(f, "signals"),
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
        .unwrap_or_else(|_| "/home/christian/0-core/scripts/faelight-shell".to_string());
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

    // --- VOCABULARY ---
    results.push(test("vocab_list_home", Category::Vocabulary, || {
        // list is fsh vocabulary -- test via ls which it maps to
        let out = run_fsh("ls ~")?;
        if out.is_empty() { Err("ls produced no output".to_string()) } else { Ok(()) }
    }));
    results.push(test("vocab_find_basic", Category::Vocabulary, || {
        // find vocabulary uses fd syntax -- test fd directly
        let out = run_fsh("fd Cargo.toml /home/christian/0-core/engine")?;
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
        expect_contains(&run_fsh("echo $PATH")?, "/usr")
    }));

    // --- SEMICOLON / OPERATORS ---
    results.push(test("semicolon_two_cmds", Category::Regression, || {
        expect_contains(&run_fsh("echo first; echo second")?, "second")
    }));
    results.push(test("and_operator", Category::Regression, || {
        expect_contains(&run_fsh("echo a && echo b")?, "b")
    }));
    results.push(test("subshell_expansion", Category::Regression, || {
        expect_eq(&run_fsh("echo $(echo nested)")?, "nested")
    }));

    // --- TILDE ---
    results.push(test("tilde_echo_subpath", Category::Tilde, || {
        expect_contains(&run_fsh("echo ~/0-core")?, "/home/christian/0-core")
    }));
    results.push(test("tilde_ls_root", Category::Tilde, || {
        expect_contains(&run_fsh("ls ~/0-core")?, "rust-tools")
    }));
    results.push(test("tilde_ls_scripts", Category::Tilde, || {
        expect_contains(&run_fsh("ls ~/0-core/scripts")?, "deploy")
    }));
    results.push(test("tilde_ls_runtime", Category::Tilde, || {
        expect_contains(&run_fsh("ls ~/0-core/runtime")?, "state.db")
    }));
    results.push(test("tilde_cat_cargo", Category::Tilde, || {
        expect_contains(&run_fsh("cat ~/0-core/rust-tools/faelight-shell/Cargo.toml")?, "faelight-shell")
    }));
    results.push(test("tilde_pipe_grep", Category::Tilde, || {
        expect_contains(&run_fsh("ls ~/0-core | grep engine")?, "engine")
    }));
    results.push(test("tilde_cat_pipe_grep", Category::Tilde, || {
        expect_contains(&run_fsh("cat ~/0-core/rust-tools/faelight-shell/Cargo.toml | grep name")?, "name")
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
        expect_contains(&run_fsh("ls ~/0-core | grep rust")?, "rust-tools")
    }));

    // --- REGRESSION ---
    results.push(test("regression_sigpipe_no_crash", Category::Regression, || {
        // Pipe to head should not crash with SIGPIPE
        let out = run_fsh("printf 'a\\nb\\nc\\nd\\ne\\n' | head -3")?;
        expect_eq(&out, "a\nb\nc")
    }));
    results.push(test("regression_tilde_not_literal", Category::Regression, || {
        // ~ must never appear literally in output when used as path
        let out = run_fsh("echo ~/0-core")?;
        if out.contains('~') {
            Err(format!("~ not expanded: {:?}", out))
        } else {
            Ok(())
        }
    }));
    results.push(test("regression_empty_pipe_ok", Category::Regression, || {
        // Empty output through pipe should not error
        run_fsh("echo '' | cat")?;
        Ok(())
    }));

    results
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let show_only_failed = args.contains(&"--failed".to_string());
    let category_filter = args.iter()
        .find(|a| a.starts_with("--category="))
        .map(|a| a.trim_start_matches("--category=").to_string());

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "  🌲 fsh-test v1.0.0 -- INT-304".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let results = all_tests();
    let mut passed = 0;
    let mut failed = 0;

    for r in &results {
        if let Some(ref cat) = category_filter {
            if r.category.to_string() != *cat { continue; }
        }
        if show_only_failed && r.passed { continue; }

        let status = if r.passed {
            "✅".to_string()
        } else {
            "❌".to_string()
        };

        println!("  {} [{:>11}] {} {}ms",
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

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  Results: {} / {} passed",
        passed.to_string().green().bold(),
        (passed + failed).to_string().bold()
    );
    if failed > 0 {
        println!("  {} tests failed", failed.to_string().red().bold());
        std::process::exit(1);
    } else {
        println!("  {}", "✅ All tests passed".green().bold());
    }
}

// Additional tests will be added here via append
