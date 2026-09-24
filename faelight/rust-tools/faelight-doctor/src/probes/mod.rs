//! The probe implementations -- one per measurement, answering the names in `probe::Probe`.
//!
//! ⚠️ A PROBE MEASURES AND JUDGES HOW BAD IT IS. It does not decide its tier, its name, or what
//! to tell the user to do about it -- those are DECLARED, and the engine supplies them.
//!
//! Each probe returns a `Measurement`. Where it cannot take the measurement at all, it returns
//! `Measurement::unknown` with the reason -- never a pass, and never a warn standing in for
//! "I could not look".

pub mod files;
pub mod forest;
pub mod git;
pub mod hardening;
pub mod runtime;
pub mod security;
pub mod system;
pub mod tools;

use crate::measurement::Measurement;
use crate::probe::Probe;

/// Run one probe.
///
/// ⚠️ EVERY VARIANT IS MATCHED EXPLICITLY, with no catch-all arm. That is deliberate: adding a
/// probe to the enum makes this fail to compile until it is answered, which is what "adding a
/// probe is a deliberate, reviewable act" means in practice.
///
/// Probes not yet ported return `unknown` NAMING THEMSELVES -- so a partially-ported engine
/// says "not yet implemented" out loud rather than reporting a clean pass for a check that has
/// never run. A missing measurement rendered as a pass is the defect INT-222 exists to remove.
pub fn run(p: Probe) -> Measurement {
    match p {
        Probe::PackageCache => system::package_cache(),
        Probe::RebootNeeded => system::reboot_needed(),
        Probe::DiskSpace => system::disk_space(),
        Probe::Binaries => system::binaries(),
        Probe::BootErrors => system::boot_errors(),
        Probe::GitStatus => git::status(),
        Probe::GitHooks => git::hooks(),
        Probe::RustToolchain => runtime::rust_toolchain(),
        Probe::Network => runtime::network(),
        Probe::VmState => runtime::vm_state(),
        Probe::BootTime => runtime::boot_time(),
        Probe::BrokenSymlinks => files::broken_symlinks(),
        Probe::ZeroAlias => files::zero_alias(),
        Probe::ZeroConfig => files::zero_config(),
        Probe::ToolInstallation => tools::tool_installation(),
        Probe::PathResilience => tools::path_resilience(),
        Probe::SchemaValidation => tools::schema_validation(),
        Probe::AliasCoverage => tools::alias_coverage(),
        Probe::Sandbox => security::sandbox(),
        Probe::SecurityAudit => security::security_audit(),
        Probe::IntentLedger => forest::intent_ledger(),
        Probe::Friday => forest::friday(),
        Probe::RustDocs => forest::rust_docs(),
        Probe::DeadwoodScan => forest::deadwood_scan(),
        Probe::OrphanPackages => forest::orphan_packages(),

        // ── not yet ported ─────────────────────────────────────────────────────────────
        Probe::SecurityHardening => hardening::security_hardening(),
        Probe::UpdateReadiness => hardening::update_readiness(),
    }
}
