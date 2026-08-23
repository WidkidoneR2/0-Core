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

/// Is this a Nix-deployed system?
///
/// The question is not "is the distro NixOS" but "does the deploy indirection exist here", which is
/// what the answer below actually depends on.
fn is_nix_deployed() -> bool {
    std::path::Path::new("/run/current-system/sw/bin").is_dir()
}

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
    let path = if is_nix_deployed() {
        std::path::PathBuf::from("/run/current-system/sw/bin/faelight-shell")
    } else {
        std::env::current_exe().ok()?
    };
    std::fs::canonicalize(path)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
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

    /// ⚠️ THE NIX BRANCH IS ASSERTED ONLY WHERE NIX IS, so this test says something true on Void
    /// rather than something convenient here.
    #[test]
    fn nix_identity_is_a_store_path_when_nix_is_present() {
        if !is_nix_deployed() {
            return;
        }
        let id = running_build_identity().expect("identity");
        assert!(
            id.starts_with("/nix/store/"),
            "on a Nix deployment the identity is the store path, not the wrapper: {id}"
        );
    }
}
