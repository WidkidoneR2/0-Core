---
id: 159
date: 2026-07-15
type: future
title: "faelight-vm owns the launch: a real Rust qemu launcher -- q35 + smm=on + .ms pflash, spawning qemu AND swtpm as its own children. The prerequisite INT-059's mandatory Secure Boot rehearsal is blocked on, and the single fix for four launch bugs."
status: planned
tags: [vm, faelight-vm, qemu, secureboot, rust, 059, 027]
---

## Why this must exist (learned by trying, 2026-07-15)
NixOS's `nixos-rebuild build-vm` CANNOT do Secure Boot. Not a config oversight -- structural:
- `virtualisation.useSecureBoot` DOES NOT EXIST in the qemu-vm module. Only efi.firmware /
  efi.variables are pointable.
- Plain `pkgs.OVMF` ships ONLY OVMF.fd / OVMF_CODE.fd / OVMF_VARS.fd -- built WITHOUT Secure Boot
  support. That is why the guest reports `Secure Boot: disabled (unsupported)`.
- `pkgs.OVMFFull.fd` adds OVMF_CODE.ms.fd + OVMF_VARS.ms.fd. BUT its .firmware/.variables passthru
  STILL resolve to the PLAIN files -- so `efi.OVMF = pkgs.OVMFFull.fd` alone changes NOTHING. The
  .ms paths must be named explicitly.
- TESTED: efi.firmware = .../OVMF_CODE.ms.fd + plain VARS -> THE VM DID NOT BOOT (guest never
  initialized the display; wait-ready correctly refused to claim ready). Reverted; VM healthy in 12s.
- ROOT CAUSE: the generated launcher runs `-machine accel=kvm:tcg -cpu max` -- i440fx, NO q35,
  NO SMM. Secure Boot OVMF is built SMM_REQUIRE=TRUE and needs `-machine q35,smm=on` plus pflash
  with the cfi.pflash01 secure property. The module offers no way to set the machine type. This is
  very likely WHY it has no useSecureBoot option.
CONSEQUENCE: INT-059's MANDATORY VM rehearsal (enroll our own PK/KEK/db in setup mode, deliberate
lockout, Forest Recovery Protocol) is IMPOSSIBLE on build-vm. This intent is its prerequisite.
Note TPM2 is NOT blocked: `virtualisation.tpm.enable = true` works today (the module spawns swtpm,
wires the socket, runs tpm2_startup). Secure Boot is the only blocked half.

## The other half: four bugs, one root cause
The bash script tracks a PROCESS NAME instead of owning what it spawned. Every launch bug found in
INT-027 is that same cause:
1. [FIXED in 027] `vm up` reported the PORT, not the guest -- a measured 9s lie. Rust wait-ready
   reads the SSH banner instead. (Proof that the port is a false signal: qemu binds the host forward
   port the instant it starts.)
2. `cmd_up` calls vm_lock BEFORE vm_clean_stale -- the janitor built to clear stale state can never
   run when a stale LOCK is what blocks you. The guard outranks its own cleanup.
3. `vm_pids` matches only `qemu-system-x86_64`. A ZOMBIE SWTPM (pid 127635) survived the failed
   secboot attempt, inherited the launcher's lock fd, and held it after qemu died. `vm down` could
   not see it, the janitor could not clean it, `vm debug` reported "qemu alive: 0 / lock HELD" --
   the symptom with no way to learn more. Found only with a custom /proc fd-walker. The VM was NOT
   down; a whole process was invisible to every diagnostic the tool has.
4. No `vm unlock` escape hatch. A stale lock is a dead end without hand-editing state.
A launcher that SPAWNS qemu and swtpm knows both PIDs and can tear both down. One fix, four bugs.

## Gates
- [ ] faelight-vm spawns qemu directly: `-machine q35,smm=on,accel=kvm` (not the module's i440fx)
- [ ] pflash unit 0 = OVMF_CODE.ms.fd readonly; unit 1 = a writable per-VM copy of the vars
- [ ] TESTED: which VARS pair with .ms CODE? (.ms VARS = MS keys pre-enrolled = USER mode = wrong
      for us; we need SETUP mode so sbctl can enroll OUR keys.) Open question -- do NOT assume.
- [ ] guest `bootctl status` reports `Secure Boot: disabled (setup mode)` -- the actual gate
- [ ] swtpm spawned as a CHILD of faelight-vm, torn down with the VM; guest reports TPM2: yes
- [ ] `vm down` kills qemu AND swtpm AND any wrapper -- verified by /proc, not by name-matching
- [ ] `vm unlock` exists; cmd_up cleans BEFORE it locks
- [ ] the bash script still forwards -- `vm up/down` unchanged for the user (INT-079 G3 holds)
- [ ] existing snapshots still work (disk + EFI vars stay atomic across the launcher change)

## Honest note: this BREAKS the organic rule, deliberately
INT-027's rule was "build NEW capability in Rust; port bash as it is touched; NEVER big-bang-rewrite
working bash." This intent rewrites the launch path, which WORKS today. That is a real departure and
should be argued, not glossed:
- The module fundamentally cannot produce an SB-capable VM. Owning the invocation is the ONLY route
  to INT-059's mandatory rehearsal -- not a preference for Rust.
- The four bugs above share a root cause that only child-ownership fixes. Patching them in bash means
  four patches that each work around the same missing structure.
- Scope discipline: this intent owns LAUNCH (qemu + swtpm + lock). It does NOT rewrite vm build,
  vm ssh, vm gui, or vm status. Those keep working; port them only when touched.

## Reference
INT-027 (complete) holds the full evidence: launcher analysis, the boot-chain proof, the zombie-swtpm
hunt, and the source-filter perf fix. DEC-140 sets the boot-chain tiers this serves.
