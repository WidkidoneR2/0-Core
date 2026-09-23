//! INT-241: THE ONE PLACE THE SHELL CHANGES DIRECTORY.
//!
//! `std::env::set_current_dir` moves the process and nothing else. POSIX requires a shell to set
//! `PWD` on every directory change, and bash does; nsh never did -- so `PWD` kept whatever the
//! bash that exec'd nsh exported at login, for the whole life of the shell.
//!
//! Measured 2026-09-23, interactive session:
//!
//! ```text
//!     cd ~/0-core
//!     pwd        -> /home/christian/0-core
//!     echo $PWD  -> /home/christian          NEVER UPDATED, not merely stale
//! ```
//!
//! ⚠️ THE `-c` DOOR AGREED WITH BASH THE WHOLE TIME, because it delegates to `sh`, which inherits
//! a correct PWD from its caller. A regression test written against `-c` would have passed without
//! this fix and proved nothing -- which is why INT-241 G6 drives the REPL door.
//!
//! ⭐ PWD IS READ BACK FROM `current_dir()` AFTER THE MOVE, never derived from the argument. A
//! relative path, a path through a symlink and `~` expansion then agree with `pwd` by
//! construction, rather than by re-implementing the resolution `pwd` already performs.
//!
//! EVERY site that moves the shell calls this. TEN existed when it was written -- cd, z_jump
//! twice, the crate jump, scope enter and leave, session restore, the cwd adopted after yazi
//! exits, startup, and a restore path in main -- and setting PWD beside each would be the
//! multi-owner shape this ledger keeps removing.
//!
//! ⚠️ THE TENTH WAS FOUND ONLY BECAUSE THE CENSUS WAS RE-RUN WITHOUT `head`. INT-241 G2 asks for
//! every site ENUMERATED rather than grepped for once, and the first pass missed main.rs:3604.

use std::path::Path;

/// Change the shell's working directory AND the `PWD` it advertises.
///
/// Signature matches `std::env::set_current_dir`, so a call site swaps one for the other without
/// touching its error handling.
pub fn chdir<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    std::env::set_current_dir(path)?;
    if let Ok(now) = std::env::current_dir() {
        std::env::set_var("PWD", now);
    }
    Ok(())
}
