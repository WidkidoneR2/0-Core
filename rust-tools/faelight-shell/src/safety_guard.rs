//! INT-246 Pillar 1 -- safety_guard
//! Enforces confidence tier rules before command execution.
//! CHALLENGE level (0.9+) blocks dangerous commands and requires explicit approval.
//! Friday produces insight, not authority -- but some commands require a gate.

/// Check if a command is dangerous enough to require explicit human confirmation.
/// Returns Some(warning) if CHALLENGE level applies, None if safe to proceed.
pub fn check(cmd: &str) -> Option<String> {
    let lower = cmd.to_lowercase();
    let trimmed = cmd.trim();

    // rm -rf on non-temp paths -- high risk
    if lower.contains("rm") && (lower.contains("-rf") || lower.contains("-fr")) {
        let safe_targets = ["/tmp/", "/tmp ", "target/", "./target"];
        let is_safe_target = safe_targets.iter().any(|t| cmd.contains(t));
        if !is_safe_target {
            return Some(format!(
                "Destructive remove: {}",
                trimmed.chars().take(60).collect::<String>()
            ));
        }
    }

    // SQL DROP TABLE or DELETE FROM critical tables
    if lower.contains("drop table") || lower.contains("drop database") {
        return Some(format!(
            "Destructive SQL: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // Direct state.db manipulation with DELETE
    if lower.contains("state.db") && lower.contains("delete from") {
        return Some(format!(
            "Deleting from state.db: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // dd -- low level disk operations
    if trimmed.starts_with("dd ") || lower.contains(" dd if=") {
        return Some(format!(
            "Low-level disk operation: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // mkfs -- filesystem formatting
    if trimmed.starts_with("mkfs") {
        return Some(format!(
            "Filesystem format: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // git reset --hard without being in a safe state
    if lower.contains("git reset") && lower.contains("--hard") {
        return Some(format!(
            "Destructive git reset: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    // chmod -R on sensitive paths
    if lower.contains("chmod") && lower.contains("-r") && lower.contains("/etc") {
        return Some(format!(
            "Recursive chmod on /etc: {}",
            trimmed.chars().take(60).collect::<String>()
        ));
    }

    None
}

/// Display the CHALLENGE gate and wait for explicit confirmation.
/// Returns true if the human approved, false if blocked.
pub fn challenge_gate(warning: &str) -> bool {
    println!();
    println!("  {} CHALLENGE -- Friday requires explicit approval", "\u{26a0}");
    println!("  {} {}", "\u{2192}".to_string(), warning);
    println!("  {} Confidence: CHALLENGE tier (0.9+)", "\u{1f332}".to_string());
    println!();
    print!("  Type 'yes' to proceed, anything else to abort: ");
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or(0);
    let approved = input.trim() == "yes";
    if approved {
        println!("  {} Proceeding with explicit approval.", "\u{2705}".to_string());
    } else {
        println!("  {} Command blocked by Friday safety guard.", "\u{274c}".to_string());
    }
    println!();
    approved
}
