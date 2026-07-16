//! INT-246 Pillar 1 -- safety_guard
//! Enforces confidence tier rules before command execution.
//! CHALLENGE level (0.9+) blocks dangerous commands and requires explicit approval.
//! Friday produces insight, not authority -- but some commands require a gate.

/// Check if a command is dangerous enough to require explicit human confirmation.
/// Returns Some(warning) if CHALLENGE level applies, None if safe to proceed.
pub fn check(cmd: &str) -> Option<String> {
    let lower = cmd.to_lowercase();
    let trimmed = cmd.trim();
    // Check first word only -- never match on arguments or paths
    let first_word = trimmed.split_whitespace().next().unwrap_or("");
    // INT-134: user-managed command allow/deny lists (DB-backed, managed via `guard`).
    // Deny is checked FIRST and wins over everything (even the static safe list below).
    // Allow is checked next -- an explicitly vetted command skips the guard. Both match
    // on first_word only, consistent with the rest of this module (never args/paths).
    if guard_list_contains("deny", first_word) {
        return Some(format!(
            "Denylisted command (user guard): {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }
    if guard_list_contains("allow", first_word) {
        return None;
    }
    // Safe commands -- fsh builtins and forest tools never trigger guard
    let safe = [
        "fg",
        "core",
        "python3",
        "python",
        "cargo",
        "git",
        "deploy",
        "cistart",
        "cicomplete",
        "intent",
        "faelight-docs",
        "docs",
        "cat",
        "echo",
        "grep",
        "sed",
        "awk",
        "curl",
        "ls",
        "cd",
    ];
    if safe.contains(&first_word) {
        return None;
    }

    // rm -rf on non-temp paths -- high risk
    if first_word == "rm" && (lower.contains("-rf") || lower.contains("-fr")) {
        let safe_targets = ["/tmp/", "/tmp ", "target/", "./target"];
        let is_safe_target = safe_targets.iter().any(|t| cmd.contains(t));
        if !is_safe_target {
            return Some(format!(
                "Destructive remove: {}",
                trimmed.chars().take(60).collect::<String>()
            ));
        }
    }

    // SQL DROP TABLE -- only trigger when running sqlite3 directly
    if (first_word == "sqlite3")
        && (lower.contains("drop table") || lower.contains("drop database"))
    {
        return Some(format!(
            "Destructive SQL: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // Direct state.db DELETE -- only sqlite3 direct calls
    if first_word == "sqlite3" && lower.contains("state.db") && lower.contains("delete from") {
        return Some(format!(
            "Deleting from state.db: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // dd -- low level disk operations
    if first_word == "dd" {
        return Some(format!(
            "Low-level disk operation: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // mkfs -- filesystem formatting
    if first_word.starts_with("mkfs") {
        return Some(format!(
            "Filesystem format: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // git reset --hard
    if first_word == "git" && lower.contains("reset") && lower.contains("--hard") {
        return Some(format!(
            "Destructive git reset: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // chmod -R on sensitive paths
    if first_word == "chmod" && lower.contains("-r") && lower.contains("/etc") {
        return Some(format!(
            "Recursive chmod on /etc: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    None
}

/// INT-134: read the user-managed guard allow/deny list from state.db.
/// Returns true if `word` is present with the given kind ("allow" or "deny").
/// Opens the db read-only itself so check()'s signature stays unchanged. Best-effort:
/// any error -> false (fail-open for allow, fail-safe for deny is handled by the caller
/// order -- deny miss just falls through to the normal heuristics).
fn guard_list_contains(kind: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let db_path = faelight_core::paths::state_db();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS fsh_guard_list (
            word TEXT NOT NULL,
            kind TEXT NOT NULL,
            PRIMARY KEY (word, kind)
        );",
    );
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM fsh_guard_list WHERE kind = ?1 AND word = ?2",
            rusqlite::params![kind, word],
            |r| r.get(0),
        )
        .unwrap_or(0);
    count > 0
}

/// Display the CHALLENGE gate and wait for explicit confirmation.
/// Returns true if the human approved, false if blocked.
pub fn challenge_gate(warning: &str) -> bool {
    println!();
    println!(
        "  {} CHALLENGE -- Friday requires explicit approval",
        "\u{26a0}"
    );
    println!("  {} {}", "\u{2192}".to_string(), warning);
    println!(
        "  {} Confidence: CHALLENGE tier (0.9+)",
        "\u{1f332}".to_string()
    );
    println!();
    print!("  Type 'yes' to proceed, anything else to abort: ");
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or(0);
    let approved = input.trim() == "yes";
    if approved {
        println!(
            "  {} Proceeding with explicit approval.",
            "\u{2705}".to_string()
        );
    } else {
        println!(
            "  {} Command blocked by Friday safety guard.",
            "\u{274c}".to_string()
        );
    }
    println!();
    approved
}
