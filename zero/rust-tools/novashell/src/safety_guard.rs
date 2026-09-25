//! INT-246 Pillar 1 -- safety_guard
//! Enforces confidence tier rules before command execution.
//! CHALLENGE level (0.9+) blocks dangerous commands and requires explicit approval.
//! Friday produces insight, not authority -- but some commands require a gate.

/// Check if a command is dangerous enough to require explicit human confirmation.
/// Returns Some(warning) if CHALLENGE level applies, None if safe to proceed.
pub fn check(cmd: &str, first_word: &str) -> Option<String> {
    let lower = cmd.to_lowercase();
    let trimmed = cmd.trim();
    // Check first word only -- never match on arguments or paths.
    // INT-195: derive it through the CANONICAL quote-aware command_word(), never
    // split_whitespace().next(). The old derivation read `"rm" -rf /` as `"rm`, which
    // matched no deny entry, no allow entry, no safe entry, and failed the
    // `first_word == "rm"` test -- so the gate stayed silent while the executor, which
    // IS quote-aware, ran rm. Proven on gen 432 before the fix: the unquoted form was
    // CHALLENGED and blocked, the quoted form produced no guard output at all.
    // The first-word-only design is deliberate and unchanged; only the instrument moved.
    // INT-196 M5: THE WORD ARRIVES DERIVED. This called command_word, which calls tokenize --
    // a second tokenizer running ahead of the scanner, inside the highest-stakes consumer in the
    // shell. The caller now supplies the word from the PARSED line, so this file performs no
    // tokenization at all and the property is provable by grep rather than by reading.
    //
    // THE LINE IS STILL TAKEN, and that is not a compromise. The rules below read the whole line
    // for things a word cannot answer -- a -rf flag, a drop table clause, a /tmp target -- and the
    // warning text quotes what the user typed. What moved is the DERIVATION, which is the thing
    // this intent is about. The first-word-only design is unchanged.
    // INT-134: user-managed command allow/deny lists (DB-backed, managed via `guard`).
    // Deny is checked FIRST and wins over everything (even the static safe list below).
    // Allow is checked next -- an explicitly vetted command skips the guard. Both match
    // on first_word only, consistent with the rest of this module (never args/paths).
    // ⚠️ THE TWO LISTS FAIL IN OPPOSITE DIRECTIONS, which is why they read the same answer
    // differently. An UNREADABLE list used to be indistinguishable from an EMPTY one -- both
    // returned false -- and that is safe for allow and unsafe for deny:
    //   allow, undetermined -> heuristics still run, which is MORE caution. Silence is correct.
    //   deny,  undetermined -> a word you explicitly banned is judged by heuristics alone, and
    //                          if none match it runs. Silence there is the defect.
    match guard_list_contains("deny", first_word) {
        Some(true) => {
            return Some(format!(
                "Denylisted command (user guard): {}",
                trimmed.chars().take(60).collect::<String>()
            ));
        }
        Some(false) => {}
        None => {
            // Not a challenge: the user cannot fix an unreadable database mid-command, and a
            // prompt they must answer every line is the loud-shell failure. Observable instead
            // -- NSH_OBSERVE=guard answers "is my deny list actually being consulted".
            crate::observe::emit(crate::observe::Event {
                level: crate::observe::Level::Warn,
                target: crate::observe::Target::Guard,
                message: "deny-list-unreadable",
                fields: &[("word", first_word.to_string())],
            });
        }
    }
    if guard_list_contains("allow", first_word) == Some(true) {
        return None;
    }
    // Safe commands -- nsh builtins and forest tools never trigger guard.
    //
    // ⚠️ THIS CATEGORY OUTRANKS EVERY HEURISTIC BELOW. A word here returns None before any
    // pattern runs, so a rule written for a command in this list is unreachable by
    // construction. `git reset --hard` had such a rule and it was deleted rather than
    // resurrected -- see the note where it used to be.
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

    // ⚠️ THE `git reset --hard` RULE WAS HERE AND IS DELETED, not disabled. `git` sits in the
    // safe list above, which returns before this line, so the rule had never fired -- a gate
    // that cannot fail, in the file where that matters most.
    //
    // DELETED RATHER THAN MADE REACHABLE, and the cost decided it. Taking git out of safe puts
    // every gp, gc and fg done through the heuristics to catch one pattern whose entire purpose
    // is discarding work deliberately -- and which has the reflog behind it. A challenge answered
    // yes every time is the loud-shell failure the philosophy names.
    //
    // ⭐ AND IT MAY HAVE BEEN THE WRONG SUBCOMMAND ANYWAY. `git clean -xfd` removes untracked
    // files with no reflog and is equally unguarded. If any git verb earns a gate it is that one,
    // and it needs git out of safe -- a decision with a stated cost, not a revived dead branch.
    //
    // ⭐ PROVEN DEAD THE HARD WAY. This rule was verified unreachable by running
    // `nsh -c 'git reset --hard'` -- which ran ungated, exactly as predicted, and discarded the
    // uncommitted edit that had just deleted it. The guard was right: that command means discard,
    // and it was typed deliberately. Check `git status` before proving a destructive command is
    // unguarded.
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
/// `None` means COULD NOT DETERMINE -- not "absent".
///
/// This returned bool, so an unreadable database and an empty list were the same answer. There
/// were two ways to reach it: the open at the top, and a query error swallowed by unwrap_or(0).
/// The callers want opposite things from that state, so the function stopped deciding for them.
///
/// Same shape as Status::Unknown in the doctor and Verdict::Unknown in zero-gate: a thing that
/// could not be measured is its own outcome, never a clean one.
fn guard_list_contains(kind: &str, word: &str) -> Option<bool> {
    if word.is_empty() {
        return Some(false);
    }
    let db_path = faelight_core::paths::state_db();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return None,
    };
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS nsh_guard_list (
            word TEXT NOT NULL,
            kind TEXT NOT NULL,
            PRIMARY KEY (word, kind)
        );",
    );
    // unwrap_or(0) was the second fail-open: a query error read as "not on the list".
    conn.query_row(
        "SELECT COUNT(*) FROM nsh_guard_list WHERE kind = ?1 AND word = ?2",
        rusqlite::params![kind, word],
        |r| r.get::<_, i64>(0),
    )
    .ok()
    .map(|c| c > 0)
}

/// Display the CHALLENGE gate and wait for explicit confirmation.
/// Returns true if the human approved, false if blocked.
pub fn challenge_gate(warning: &str) -> bool {
    // ⭐ `-c` REFUSES RATHER THAN ASKS, and the alternative is worse than it looks.
    //
    // This reads stdin for the word yes. Through `-c`, stdin belongs to the CALLER -- a pipe, a
    // file, a closed descriptor. Asking a question into that is not asking anyone: EOF trims to
    // an empty string, which already returns false. So the door was ALREADY blocking closed,
    // by accident, after printing a prompt nobody could answer.
    //
    // Worse, a piped `yes` on the next line would ANSWER a challenge the caller never saw.
    //
    // Deliberate now: name the refusal, do not prompt, return false. And write to STDERR --
    // stdout belongs to the program on this door, which is what keeps `nsh -c 'echo hi' | wc -l`
    // answering 1.
    if crate::IS_DASH_C.load(std::sync::atomic::Ordering::SeqCst) {
        eprintln!();
        eprintln!("  \u{26a0} REFUSED -- this needs a human, and `-c` has no one to ask");
        eprintln!("  \u{2192} {}", warning);
        eprintln!("  Run it at an interactive prompt if you meant it.");
        // Same instrument, the other door. A refusal nobody could answer is still a decision
        // the guard made, and `NSH_OBSERVE=guard nsh -c ...` should say so.
        crate::observe::emit(crate::observe::Event {
            level: crate::observe::Level::Info,
            target: crate::observe::Target::Guard,
            message: "refused-no-tty",
            fields: &[("warning", warning.to_string())],
        });
        return false;
    }
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
    // ⭐ THE FIRST RECORD THIS GATE HAS EVER KEPT. It has been stopping commands since April and
    // logging nothing: no event, no history row, nothing saying a human was asked or what they
    // answered. So nobody can say how often it fires, on what, or whether anyone types yes --
    // and after widening it to a second door there is no BASELINE to compare against.
    //
    // ⚠️ THIS IS AN INSTRUMENT, NOT A RECORD. observe::emit is silent unless NSH_OBSERVE is set,
    // so `NSH_OBSERVE=guard` answers what the guard is doing while you watch. NOTHING
    // ACCUMULATES. A persistent count of every challenge is a different thing and it belongs to
    // the history model INT-191 owns -- inventing a table here would be a second owner of shell
    // events, which is the defect this codebase keeps finding.
    crate::observe::emit(crate::observe::Event {
        level: crate::observe::Level::Info,
        target: crate::observe::Target::Guard,
        message: if approved { "approved" } else { "blocked" },
        fields: &[("warning", warning.to_string())],
    });
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
