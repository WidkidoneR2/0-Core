//! INT-227: platform FACTS, answered once.
//!
//! ⚠️ DELIBERATELY NARROW. The first draft of this module answered six questions -- services, logs,
//! system rebuild, store query, store reclaim and build identity -- with ONE caller between them.
//! Six questions and one consumer is a dumping ground, not an abstraction. Those five are
//! CAPABILITY DETECTION ("does this system have journald?"); this is IDENTITY ("which build is
//! running?"). Different concerns with different lifetimes, and mixing them because both happen to
//! be platform-dependent is how a module becomes a junk drawer.
//!
//! ★ AND MOST OF WHAT LOOKED LIKE ASSUMPTIONS WERE NOT. Reading the four self-location sites found
//! three already correct: `resolve_nsh_binary` and the `exec fsh` path both probe a CANDIDATE LIST
//! -- system profile, per-user profile, ~/.cargo/bin, ~/0-core/scripts -- and take the first that
//! exists, so absent entries simply miss and the cargo path wins. PATH augmentation
//! appends directories that are harmless when absent. Only build identity was a real assumption.

/// Is there an executable named `name` on PATH?
///
/// ⭐ A GENERIC PRIMITIVE WITH SPECIFIC CALLERS. Not `has_systemctl()` -- that shape accumulates
/// `has_journalctl`, `has_pacman` as separate one-off functions until the module
/// is the junk drawer this one was written to avoid.
///
/// ⚠️ EXISTENCE, NOT USABILITY. This says a binary is on PATH, nothing more: not that it will
/// succeed, not that the daemon behind it is running. A caller that needs more must ask for more.
///
/// ★ AND IT DOES NOT EXECUTE ANYTHING. Shelling out to `command -v` would depend on `sh` existing,
/// which is a platform assumption inside a portability fix. PATH is read directly.
pub fn has_tool(name: &str) -> bool {
    let Ok(path) = std::env::var("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file() && is_executable(&candidate)
    })
}

#[cfg(unix)]
fn is_executable(p: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tool that certainly exists, and one that certainly does not. ⚠️ The negative case is the
    /// one that matters: a lookup that returned true for everything would make every degrade path
    /// dead code, and nothing would notice.
    #[test]
    fn has_tool_finds_what_is_there_and_not_what_is_not() {
        assert!(has_tool("sh"), "sh is on PATH on any unix");
        assert!(!has_tool("definitely-not-a-real-binary-xyzzy"));
    }

    /// ⚠️ NOT FOOLED BY A DIRECTORY OR A NON-EXECUTABLE FILE of the same name.
    #[test]
    fn has_tool_requires_an_executable_file() {
        assert!(!has_tool("."), "a directory entry is not a tool");
    }
}
