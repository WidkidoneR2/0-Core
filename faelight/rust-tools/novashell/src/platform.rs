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
//! three already correct: `resolve_fsh_binary` and the `exec fsh` path both probe a CANDIDATE LIST
//! -- system profile, per-user profile, ~/.cargo/bin, ~/0-core/scripts -- and take the first that
//! exists, so on Void the Nix entries simply miss and the cargo path wins. PATH augmentation
//! appends directories that are harmless when absent. Only build identity was a real assumption.

/// The identity of the build this session is running.
///
/// ⚠️ current_exe() IS NOT UNIVERSALLY THE ANSWER, AND THE OLD CODE'S COMMENT SAID SO FIRST:
/// "the deploy symlink canonicalizes to a store path whose hash changes on every rebuild -- that
/// hash IS the build identity (current_exe() is unreliable here because the deployed binary is
/// makeWrapper-wrapped)". A wrapped binary reports the wrapper, not the artifact whose hash
/// distinguishes one deploy from the next. So on Nix the store path stays the identity.
///
/// ⭐ ELSEWHERE THERE IS NO STORE AND NO WRAPPER, so the running executable IS the artifact and
/// current_exe() is exactly right. The caller does not need to know which world it is in.
///
/// Returns None when identity cannot be established. That stays NON-FATAL, as it is today --
/// `reload` loses its ability to notice a newer build, which is a degraded feature rather than a
/// broken shell.
pub fn running_build_identity() -> Option<String> {
    // ⭐ ONE BRANCH, AND IT NAMES NO DISTRIBUTION. canonicalize() resolves whatever
    // indirection is in front of the binary -- a symlink, a wrapper target, a store path --
    // without the code needing to know what put it there. On a system with the deploy
    // indirection it still lands on the real artifact; here ship copies a real file and it
    // lands on that.
    //
    // ⚠️ THE BRANCH THIS REPLACES WAS ALREADY BROKEN. It read
    // /run/current-system/sw/bin/faelight-shell -- a binary name that has not existed since
    // the NovaShell rename, under a path that has not existed since 2026-08-26. It could
    // only ever have returned None.
    std::fs::canonicalize(std::env::current_exe().ok()?)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Is there an executable named `name` on PATH?
///
/// ⭐ A GENERIC PRIMITIVE WITH SPECIFIC CALLERS. Not `has_systemctl()` -- that shape accumulates
/// `has_journalctl`, `has_nix_store`, `has_pacman` as separate one-off functions until the module
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

    /// The contract, stated as a test rather than a comment: identity resolves to a CANONICAL path
    /// that EXISTS. Which path depends on the platform; that it is real does not.
    #[test]
    fn identity_is_a_real_canonical_path() {
        let id = running_build_identity().expect("a running binary has an identity");
        assert!(
            std::path::Path::new(&id).exists(),
            "identity must name something that exists: {id}"
        );
        assert!(id.starts_with('/'), "identity must be absolute: {id}");
    }

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
