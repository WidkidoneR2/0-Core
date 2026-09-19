//! The probe registry -- CLOSED, and closed is the whole point.
//!
//! A probe is a named question the engine knows how to ask. A definition NAMES one.
//!
//! ⚠️ THERE IS NO VARIANT THAT CARRIES A COMMAND STRING, AND THAT ABSENCE IS THE GATE.
//! INT-222: "there is no path from a definition to arbitrary code". A definition cannot
//! smuggle a shell command through this type, because the type has nowhere to put one.
//! Adding a probe is an edit to this enum and to the code that answers it -- a deliberate,
//! reviewable act, which is exactly what the intent asked for.
//!
//! ★ ENUMERATED BY CENSUS, NOT BY DESIGN. These are what the doctor's own helpers already do,
//! measured 2026-09-18. The first census scanned check BODIES and found seven "pure" checks --
//! wrong, because the access is one level down in helpers like `running_kernel()`. A registry
//! invented from imagination would have been both shorter and incorrect.

use serde::Deserialize;

/// What a definition may ask the outside world.
///
/// ⭐ NAMED FOR WHAT IT ASKS, NEVER FOR HOW IT ASKS. `Subprocess` would not be a probe -- it is
/// a mechanism, and a definition naming it would be unreviewable. `PacmanOrphans` is a question
/// with an answer someone can reason about.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Probe {
    /// Is a systemd unit loaded and active. (7 sites today)
    SystemctlUnits,
    /// Working tree state, hooks path, HEAD. (5 sites)
    GitStatus,
    /// Is a command on PATH. (4 sites)
    WhichBinary,
    /// Toolchain present, and its version.
    RustcVersion,
    /// Does `cargo doc` build clean.
    CargoDoc,
    /// `uname -r`.
    KernelRunning,
    /// What is on disk to boot.
    InstalledKernels,
    /// `systemd-analyze time`.
    BootTime,
    /// `pgrep` -- is a named process running.
    ProcessRunning,
    /// `pacman -Qdtq`.
    PacmanOrphans,
    /// `journalctl` since boot.
    JournalErrors,
    /// `faelight-deadwood`.
    DeadwoodScan,
    /// A package name to the real path of its binary.
    ///
    /// ⚠️ THIS ONE WAS THE DOOR. Until 2026-09-18 it was `sh -c "readlink -f $(command -v X)"`,
    /// written twice, with the name interpolated into the shell string. Native now: `which`
    /// plus `canonicalize`. The registry could not be called closed while one member was a shell.
    ResolveBinaryPath,
    /// The forest database.
    SqliteQuery,
    /// read_to_string / read_dir / exists / metadata.
    FsRead,
}

impl Probe {
    /// The stable name used in TOML, and in any message naming the probe.
    pub fn as_str(&self) -> &'static str {
        match self {
            Probe::SystemctlUnits => "systemctl_units",
            Probe::GitStatus => "git_status",
            Probe::WhichBinary => "which_binary",
            Probe::RustcVersion => "rustc_version",
            Probe::CargoDoc => "cargo_doc",
            Probe::KernelRunning => "kernel_running",
            Probe::InstalledKernels => "installed_kernels",
            Probe::BootTime => "boot_time",
            Probe::ProcessRunning => "process_running",
            Probe::PacmanOrphans => "pacman_orphans",
            Probe::JournalErrors => "journal_errors",
            Probe::DeadwoodScan => "deadwood_scan",
            Probe::ResolveBinaryPath => "resolve_binary_path",
            Probe::SqliteQuery => "sqlite_query",
            Probe::FsRead => "fs_read",
        }
    }

    /// Every probe the engine knows. Used to report the registry, and to prove it is finite.
    pub fn all() -> &'static [Probe] {
        &[
            Probe::SystemctlUnits,
            Probe::GitStatus,
            Probe::WhichBinary,
            Probe::RustcVersion,
            Probe::CargoDoc,
            Probe::KernelRunning,
            Probe::InstalledKernels,
            Probe::BootTime,
            Probe::ProcessRunning,
            Probe::PacmanOrphans,
            Probe::JournalErrors,
            Probe::DeadwoodScan,
            Probe::ResolveBinaryPath,
            Probe::SqliteQuery,
            Probe::FsRead,
        ]
    }
}
