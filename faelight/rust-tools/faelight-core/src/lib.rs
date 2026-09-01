//! faelight-core v1.0.0 - Shared Foundation Library
//!
//! Provides common functionality for all Faelight tools:
//! - Glyph caching (70%+ CPU reduction)
//! - Canvas drawing primitives
//! - Theme system (consistent styling)
//! - Wayland helpers (layer-shell configs)
//! - Error handling

#[cfg(feature = "ui")]
pub mod canvas;
pub mod error;
#[cfg(feature = "ui")]
pub mod glyph;
pub mod paths;
pub mod theme;
#[cfg(feature = "ui")]
pub mod wayland;

#[cfg(feature = "ui")]
pub use canvas::Canvas;
pub use error::{FaelightError, Result};
#[cfg(feature = "ui")]
pub use glyph::GlyphCache;
pub use theme::Theme;
#[cfg(feature = "ui")]
pub use wayland::{Anchor, Layer, LayerSurfaceConfig};

#[cfg(all(test, feature = "ui"))]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_cache_basic() {
        // INT-106: load the font at runtime and skip gracefully if absent (e.g. the
        // Nix build sandbox has no system fonts). Previously include_bytes! with an
        // absolute path hard-failed the test build. GlyphCache::new takes font bytes,
        // so production is unaffected -- only this test referenced a system path.
        let font_path = "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf";
        let font_data = match std::fs::read(font_path) {
            Ok(d) => d,
            Err(_) => return, // font not available in this environment -- skip
        };
        let mut cache = GlyphCache::new(&font_data).unwrap();

        let glyph1 = cache.rasterize('A', 16.0);
        assert!(!glyph1.bitmap.is_empty());

        let glyph2 = cache.rasterize('A', 16.0);
        assert!(!glyph2.bitmap.is_empty());

        let (hits, misses, hit_rate) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(hit_rate, 50.0);
    }
}

/// Restore the default SIGPIPE disposition, so a tool piped into head or less
/// exits quietly instead of panicking.
///
/// WHY THIS IS NEEDED: the Rust runtime sets SIGPIPE to SIG_IGN at startup, so a
/// write to a closed pipe returns EPIPE rather than terminating the process.
/// println! unwraps that error and panics. The result is that EVERY CLI in this
/// workspace dies with "failed printing to stdout: Broken pipe (os error 32)"
/// when the reader goes away first -- normal Unix behaviour that head relies on.
/// Measured 2026-08-27 on faelight-deadwood and ship; the tools that appeared to
/// survive only printed fewer lines than head had asked for.
///
/// UNSAFE AND PROCESS-WIDE, which is why it lives in ONE place rather than being
/// pasted into each main. Call it as the first statement of main, before any
/// output.
/// Do two files hold different bytes? Length first, then contents.
///
/// ⚠️ THE ANSWER TO "IS THE INSTALLED BINARY WHAT THE SOURCE BUILDS", and it lived privately
/// in ship, where only ship could ask it. On 2026-09-01 a stale binary produced a confident
/// wrong answer FIVE separate times in one session -- nsh-test testing a shell the machine was
/// no longer running, the shell exporting variable names the source had renamed, and the
/// harness itself. Each looked like a regression and none was.
///
/// VERSIONS CANNOT ANSWER IT. nsh-test already compared the deployed shell's version against
/// the workspace's and warned on a mismatch, with sound reasoning -- and it stayed silent
/// through all five, because the version does not change on every edit. A mid-version rebuild
/// is invisible to a version check and obvious to a byte comparison.
///
/// Missing on either side counts as different: nothing installed cannot match anything built.
pub fn differs(a: &std::path::Path, b: &std::path::Path) -> bool {
    match (std::fs::metadata(a), std::fs::metadata(b)) {
        (Ok(x), Ok(y)) => {
            if x.len() != y.len() {
                return true;
            }
            match (std::fs::read(a), std::fs::read(b)) {
                (Ok(da), Ok(db)) => da != db,
                _ => true,
            }
        }
        _ => true,
    }
}

pub fn restore_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
