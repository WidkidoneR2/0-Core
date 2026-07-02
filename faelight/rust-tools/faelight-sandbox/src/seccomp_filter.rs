// faelight-sandbox seccomp module
// INT-125 — Syscall filtering via seccompiler BPF

use seccompiler::{BpfProgram, SeccompAction, SeccompFilter, SeccompRule};
use std::collections::BTreeMap;

/// Build a BPF seccomp filter blocking dangerous syscalls
pub fn build_filter(strict: bool) -> Option<BpfProgram> {
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    // Syscall numbers for x86_64 — block dangerous ones
    let basic_blocked: &[i64] = &[
        175, // init_module
        313, // finit_module
        176, // delete_module
        169, // reboot
        246, // kexec_load
        320, // kexec_file_load
    ];

    let strict_blocked: &[i64] = &[
        101, // ptrace
        165, // mount
        166, // umount2
    ];

    for &syscall in basic_blocked {
        rules.insert(syscall, vec![]);
    }

    if strict {
        for &syscall in strict_blocked {
            rules.insert(syscall, vec![]);
        }
    }

    let arch = std::env::consts::ARCH;
    let target_arch = match arch {
        "x86_64" => seccompiler::TargetArch::x86_64,
        "aarch64" => seccompiler::TargetArch::aarch64,
        _ => return None,
    };

    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::KillProcess,
        target_arch,
    )
    .ok()?;

    filter.try_into().ok()
}

pub fn apply_filter(strict: bool) -> Result<(), String> {
    let filter =
        build_filter(strict).ok_or_else(|| "Failed to build seccomp filter".to_string())?;
    seccompiler::apply_filter(&filter).map_err(|e| format!("Failed to apply seccomp filter: {}", e))
}
