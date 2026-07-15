---
id: 159
date: 2026-07-15
type: future
title: "faelight-vm owns the launch: a real Rust qemu launcher -- q35 + smm=on + .ms pflash, spawning qemu AND swtpm as its own children. The prerequisite INT-059's mandatory Secure Boot rehearsal is blocked on, and the single fix for four launch bugs."
status: in-progress
tags: [vm, faelight-vm, qemu, secureboot, rust, 059, 027]
---

## CORRECTION (2026-07-15, same day): the above is WRONG -- Secure Boot IS reachable
The finding below ("the module CANNOT do Secure Boot, therefore faelight-vm must own the qemu
invocation") was based on a BAD TEST. That attempt changed efi.firmware to OVMF_CODE.ms.fd while
leaving the machine type at the module's default i440fx -- which has no SMM. The .ms firmware is
built SMM_REQUIRE=TRUE, so of course it could not come up. The test proved we forgot a flag, not
that the module has a limit.
WHAT IS ACTUALLY TRUE (proven by boot, 2026-07-15):
- The generated launcher ends with `$QEMU_OPTS \` then `"$@"` -- TWO pass-through seams the module
  hands you. QEMU_OPTS is unquoted, so it word-splits into real args.
- qemu MERGES a second -machine over the built-in one. PROVEN by /proc on a live guest:
    -machine accel=kvm:tcg          (from the module)
    -machine q35,smm=on             (from QEMU_OPTS)
    /dev/kvm OPEN  -> KVM SURVIVED
  The machine type is reachable WITHOUT owning the launch, and virtualization is not lost.
- efi.firmware = "${pkgs.OVMFFull.fd}/FV/OVMF_CODE.ms.fd" + efi.variables = ".../OVMF_VARS.fd"
  (plain vars = NO keys = SETUP MODE) + QEMU_OPTS="-machine q35,smm=on -global
  driver=cfi.pflash01,property=secure,value=on"
  -> guest bootctl: `Secure Boot: disabled (setup)`.  Was `disabled (unsupported)`.
  SETUP MODE IS THE GATE -- it is the state sbctl needs to enroll our OWN PK/KEK/db (INT-059).
- Still true and worth keeping: OVMFFull.fd.firmware/.variables resolve to the PLAIN files, so the
  .ms paths must be named EXPLICITLY; and there is no `useSecureBoot` option. But neither blocks us.
- Also learned from the guest's own bootctl: systemd-boot advertises "Enroll SecureBoot keys" and
  "Boot counting" as features -- the loader can enroll keys itself (relevant to INT-059).
CONSEQUENCE: INT-159 loses its main justification. Owning the launch is NOT required for INT-059's
rehearsal. What remains real is child-ownership (the zombie swtpm), which is a much smaller job.
Leaving the original text below verbatim -- the wrong turn is part of the record.

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

## SCOPE REVISED (2026-07-15): this is now a CHILD-OWNERSHIP intent, not a launcher rewrite
Secure Boot no longer needs it (see CORRECTION above) -- QEMU_OPTS + .ms firmware gets setup mode
on the module's own launcher. So the gates below are REPLACED by the four real bugs. faelight-vm
does NOT need to own the qemu invocation; the vm script just needs to set QEMU_OPTS, and the tool
needs to know about the processes it spawns.
This also RESTORES INT-027's organic rule -- no working bash gets big-bang rewritten after all.

## Gates (revised)
- [ ] vm script sets QEMU_OPTS (q35 + smm=on + pflash secure) so `vm up` gets setup mode by default
- [ ] guest `bootctl status` reports `Secure Boot: disabled (setup)` through the plain `vm up`
- [ ] virtualisation.tpm.enable = true; guest reports TPM2 Support: yes
- [ ] `vm down` kills qemu AND swtpm AND any wrapper -- matched by /proc ancestry or fd, NOT by
      name. (A zombie swtpm survived vm down holding the launch lock; vm_pids only greps
      qemu-system-x86_64, so nothing could see it.)
- [ ] cmd_up cleans stale state BEFORE it locks (today the guard outranks its own janitor)
- [ ] `vm unlock` exists as an escape hatch
- [ ] vm debug can SEE a non-qemu child holding the lock (today it reports "qemu alive: 0 / lock
      HELD" and offers nothing -- it took a custom /proc fd-walker to find the holder)

## Gates (SUPERSEDED -- kept for the record; the launcher rewrite is not needed)
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
