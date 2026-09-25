//! The probe registry -- CLOSED, and closed is the whole point.
//!
//! A probe is a named QUESTION the engine knows how to ask. A definition NAMES one.
//!
//! ⚠️ THERE IS NO VARIANT THAT CARRIES A COMMAND STRING, AND THAT ABSENCE IS THE GATE.
//! INT-222: "there is no path from a definition to arbitrary code". A definition cannot smuggle
//! a shell command through this type, because the type has nowhere to put one. Adding a probe is
//! an edit to this enum and to the code that answers it -- deliberate, and reviewable.
//!
//! ## ⭐ ONE PROBE PER MEASUREMENT, NOT PER MECHANISM
//!
//! The first version of this file had FIFTEEN entries -- `systemctl_units`, `which_binary`,
//! `fs_read` -- named after the mechanisms the census found. THAT WAS THE `subprocess` MISTAKE
//! ONE LEVEL DOWN: a definition saying `probe: fs_read` is no more reviewable than one saying
//! `probe: subprocess`, because neither says WHAT IS BEING ASKED.
//!
//! It also could not express the checks that matter. `check_services` asks systemctl three
//! different questions and DISCOVERS its unit list from the answer to the first; `check_drift`
//! is 1608 lines and 24 calls. Composing those from mechanism-probes would require the
//! definition format to become a programming language, which this intent forbids.
//!
//! So: THE DEFINITION DECLARES, THE PROBE MEASURES. Each probe here is one check's whole
//! measurement, however complex, behind one reviewable name. The declaration beside it -- tier,
//! severity range, recovery -- is data, and that is where every defect this intent was filed
//! against actually lived.

use serde::Deserialize;

/// What a definition may ask. One variant per measurement the doctor performs.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Probe {
    /// Dotfile symlinks that point at nothing.
    BrokenSymlinks,
    /// Every binary the manifest expects is present.
    Binaries,
    /// Working tree state and whether commits are pushed.
    GitStatus,
    /// core.hooksPath is set and the hooks are executable.
    GitHooks,
    /// cargo doc builds clean.
    RustDocs,
    /// Every intent file parses and validates.
    IntentLedger,
    /// zero-deadwood: stale files and structural orphans.
    DeadwoodScan,
    /// The config files parse.
    ZeroConfig,
    /// The state and config aliases resolve to the faelight directories.
    ZeroAlias,
    /// Firewall, sshd policy, and the rest of the hardening surface.
    SecurityHardening,
    /// cargo audit over the dependency tree.
    SecurityAudit,
    /// Every deployed tool has an alias, or is declared not to need one.
    AliasCoverage,
    /// The toolchain is present and usable.
    RustToolchain,
    /// Free space against the filesystem's own size.
    DiskSpace,
    /// Every tool in the registry is installed.
    ToolInstallation,
    /// Every tool resolves through paths.rs rather than a hand-built path.
    PathResilience,
    /// The registry files are present and their fields check out.
    SchemaValidation,
    /// zero-sandbox is deployed and its policies are active.
    Sandbox,
    /// Critical kernel errors since boot.
    BootErrors,
    /// systemd-analyze: how long userspace took.
    BootTime,
    /// The running kernel against the installed one.
    RebootNeeded,
    /// Whether it is safe to update -- kernel current, tree clean.
    UpdateReadiness,
    /// Size and count of the package cache.
    PackageCache,
    /// Packages nothing depends on.
    OrphanPackages,
    /// Friday's pattern and fact counts, and its confidence.
    Friday,
    /// Online, and DNS resolving.
    Network,
    /// Running QEMU VMs.
    VmState,
}

impl Probe {
    /// The stable name used in TOML, and in any message naming the probe.
    pub fn as_str(&self) -> &'static str {
        match self {
            Probe::BrokenSymlinks => "broken_symlinks",
            Probe::Binaries => "binaries",
            Probe::GitStatus => "git_status",
            Probe::GitHooks => "git_hooks",
            Probe::RustDocs => "rust_docs",
            Probe::IntentLedger => "intent_ledger",
            Probe::DeadwoodScan => "deadwood_scan",
            Probe::ZeroConfig => "zero_config",
            Probe::ZeroAlias => "zero_alias",
            Probe::SecurityHardening => "security_hardening",
            Probe::SecurityAudit => "security_audit",
            Probe::AliasCoverage => "alias_coverage",
            Probe::RustToolchain => "rust_toolchain",
            Probe::DiskSpace => "disk_space",
            Probe::ToolInstallation => "tool_installation",
            Probe::PathResilience => "path_resilience",
            Probe::SchemaValidation => "schema_validation",
            Probe::Sandbox => "sandbox",
            Probe::BootErrors => "boot_errors",
            Probe::BootTime => "boot_time",
            Probe::RebootNeeded => "reboot_needed",
            Probe::UpdateReadiness => "update_readiness",
            Probe::PackageCache => "package_cache",
            Probe::OrphanPackages => "orphan_packages",
            Probe::Friday => "friday",
            Probe::Network => "network",
            Probe::VmState => "vm_state",
        }
    }

    /// Every probe the engine knows. Finite, and the test proves it.
    pub fn all() -> &'static [Probe] {
        &[
            Probe::BrokenSymlinks,
            Probe::Binaries,
            Probe::GitStatus,
            Probe::GitHooks,
            Probe::RustDocs,
            Probe::IntentLedger,
            Probe::DeadwoodScan,
            Probe::ZeroConfig,
            Probe::ZeroAlias,
            Probe::SecurityHardening,
            Probe::SecurityAudit,
            Probe::AliasCoverage,
            Probe::RustToolchain,
            Probe::DiskSpace,
            Probe::ToolInstallation,
            Probe::PathResilience,
            Probe::SchemaValidation,
            Probe::Sandbox,
            Probe::BootErrors,
            Probe::BootTime,
            Probe::RebootNeeded,
            Probe::UpdateReadiness,
            Probe::PackageCache,
            Probe::OrphanPackages,
            Probe::Friday,
            Probe::Network,
            Probe::VmState,
        ]
    }
}
