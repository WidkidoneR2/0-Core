---
id: 159
date: 2026-07-15
type: future
title: "faelight-vm owns the launch: a real Rust qemu launcher -- q35 + smm=on + .ms pflash, spawning qemu AND swtpm as its own children. The prerequisite INT-059's mandatory Secure Boot rehearsal is blocked on, and the single fix for four launch bugs."
status: complete
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
- [x] vm script sets QEMU_OPTS (q35 + smm=on + pflash secure) so `vm up` gets setup mode by default <!-- evidence: deployed gen 372, 2026-07-15. the script now owns QEMU_OPTS outright -- it does NOT prepend an inherited value. The launcher's line 92 (`$QEMU_OPTS \`, unquoted) word-splits it into real qemu args; qemu MERGES the second -machine over the module's built-in accel=kvm:tcg and KVM survives -->
- [x] guest `bootctl status` reports `Secure Boot: disabled (setup)` through the plain `vm up` <!-- evidence: deployed gen 372, 2026-07-15. PROVEN: bare `vm up` -> 'guest is UP after 12s' -> `vm ssh 'bootctl status'` -> 'Secure Boot: disabled (setup)'. Was 'disabled (unsupported)'. SETUP MODE = no PK enrolled = the door sbctl needs to enroll our own PK/KEK/db (INT-059). No env var, no launcher rewrite -->
- [x] virtualisation.tpm.enable = true; guest reports TPM2 Support: yes <!-- evidence: nix/hosts/vm/base.nix tpm.enable, 2026-07-15. PROVEN: bare `vm up` -> guest `bootctl status` reports 'Secure Boot: disabled (setup)' AND 'TPM2 Support: yes'. One line -- the module spawns swtpm, wires the socket, runs tpm2_startup. The VM can now do BOTH halves of INT-059's rehearsal: enroll our own keys (setup mode) and measure boot (TPM2). -->
- [x] `vm down` kills qemu AND swtpm AND any wrapper -- scoped by the state dir, not by name <!-- evidence: deployed gen 375, 2026-07-15. deployed `vm down` -> 'stopped qemu (98059)' + 'stopped swtpm (98069)' + 'stopped launcher (98074)' + 'all faelight-vm processes gone'. cmd_down now forwards to `fvm kill`. TRAP FOUND LIVE: matching on exe.starts_with("qemu-system") made qemu INVISIBLE -- makeWrapper means the real ELF is `.qemu-system-x86_64-wrapped`, which starts with a DOT. `procs` listed swtpm on a live VM and no qemu; `kill` would have ORPHANED the VM. `ss -ltnp` gave it away (comm was '.qemu-system-x8'). Fixed to substring matching -- the banked forest rule about wrapped binaries needing -f, not -x -->
- [x] cmd_up cleans stale state BEFORE it locks -- done, but THE GATE'S PREMISE WAS WRONG <!-- evidence: deployed gen 375, 2026-07-15. reorder is deployed (vm_clean_stale then vm_lock) and is defensible tidiness, but it fixes NOTHING. An flock is RELEASED WHEN ITS HOLDER DIES, so a 'stale lock' that blocks you cannot exist -- if vm_lock refuses, a LIVE process holds it (the zombie swtpm, exactly what happened). And vm_clean_stale only clears spice.sock and vm.pid; it never touches the lock. The real fixes were gate 4 (kill sees swtpm) and gate 6 (unlock escape hatch). Recorded rather than claimed -->
- [x] `vm unlock` exists as an escape hatch <!-- evidence: deployed gen 375, 2026-07-15. orphaned lock with no holder -> 'orphaned lock cleared'. LIVE VM -> refuses, naming every holder: 'pid 80415 (swtpm) still HOLDS the lock', 'refusing to unlock a live VM. Run: vm down' (exit 1). Also removed a FALSE advice line: `procs` used to say 'vm.lock exists but NOBODY holds it -- orphaned' after every clean shutdown. An flock releases when its holder dies; the FILE always survives. That advice would fire every time and train the user to ignore the tool -->
- [x] vm debug SEES every child and every lock holder <!-- evidence: deployed gen 375, 2026-07-15. deployed `vm debug` on a live VM -> 'faelight-vm processes : 3 (qemu=1)' listing PID 98870 qemu / 98880 swtpm / 98885 launcher, each marked '<- HOLDS THE LOCK'. Was: 'qemu (faelight-vm) alive : 1' and nothing about swtpm. Also warns when qemu is gone but swtpm survives -- that zombie owns the tpm socket, so the NEXT launch dies at set -e before qemu starts -->

## Gates (SUPERSEDED -- kept for the record; the launcher rewrite is not needed)
- (superseded) faelight-vm spawns qemu directly: `-machine q35,smm=on,accel=kvm` (not the module's i440fx)
- (superseded) pflash unit 0 = OVMF_CODE.ms.fd readonly; unit 1 = a writable per-VM copy of the vars
- (superseded) TESTED: which VARS pair with .ms CODE? (.ms VARS = MS keys pre-enrolled = USER mode = wrong
      for us; we need SETUP mode so sbctl can enroll OUR keys.) Open question -- do NOT assume.
- (superseded) guest `bootctl status` reports `Secure Boot: disabled (setup mode)` -- the actual gate
- (superseded) swtpm spawned as a CHILD of faelight-vm, torn down with the VM; guest reports TPM2: yes
- (superseded) `vm down` kills qemu AND swtpm AND any wrapper -- verified by /proc, not by name-matching
- (superseded) `vm unlock` exists; cmd_up cleans BEFORE it locks
- (superseded) the bash script still forwards -- `vm up/down` unchanged for the user (INT-079 G3 holds)
- (superseded) existing snapshots still work (disk + EFI vars stay atomic across the launcher change)

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

## The hour this cost, and why (2026-07-15) -- an fsh trap worth INT-143's attention
Four consecutive VM boots failed with `qemu-system-x86_64: unsupported machine type: "-machine"`.
Root cause: an earlier attempt to test the flag ran `QEMU_OPTS="-machine q35,smm=on" vm up` at the
fsh prompt. fsh does NOT support `VAR="a b" cmd` inline assignment -- it WORD-SPLIT the value,
errored on the remainder ("command not found: q35,smm=on\""), and SILENTLY LEFT QEMU_OPTS="-machine"
in the session environment. The script's `${QEMU_OPTS:-}` then prepended that fragment, producing
`-machine -machine q35,smm=on`, and qemu read the second -machine as the FIRST one's value.
The damage surfaced an hour later, in a different tool, as a firmware failure. `unset QEMU_OPTS`
fixed it instantly.
LESSON for INT-143: fsh's builtins that shadow real binaries (`bash`, `env`, `time`, and inline
VAR= assignment) do not merely fail -- they can POISON THE SESSION and misattribute the blame.
`bash script.sh` drops into interactive bash and never runs the script. `env VAR=x cmd` prints the
environment. A silent wrong result is worse than a clean error.

## Reference
INT-027 (complete) holds the full evidence: launcher analysis, the boot-chain proof, the zombie-swtpm
hunt, and the source-filter perf fix. DEC-140 sets the boot-chain tiers this serves.

## Followup (2026-07-15, after close): the serial log claim was OVERSTATED
This intent's completion commit said "BONUS: -serial file: restores the boot log ... an INT-049
prerequisite closed as a side effect." NOT TRUE AT CLOSE. -serial file: was only ever proven in a
throwaway /tmp script; the vm script's QEMU_OPTS never had it, so vm.log stayed 0 bytes. Fixed in a
followup commit, and recorded here rather than left as a false claim in a closed intent.
NOW REAL: QEMU_OPTS carries -serial file:$STATE/vm-serial.log. TWO logs, on purpose:
  vm.log        = qemu's own stdout/stderr -> "did qemu even start?" (0 bytes on a clean launch;
                  this is where `unsupported machine type: "-machine"` surfaced)
  vm-serial.log = the guest's console      -> "how far did the guest get?"
Never point -serial at vm.log -- qemu truncates on open and the launch error is lost.
PROVEN: bare `vm up` -> vm-serial.log 21,414 bytes, ending "Reached target Graphical Interface".
ALSO FIXED, a live bug nobody had hit: cmd_gui ran `env QEMU_OPTS="$QOPTS"` -- REPLACING the base
opts with SPICE flags only. Since the .ms firmware landed, `vm gui` would have launched Secure Boot
firmware on i440fx with no SMM and hung before OVMF came up -- the exact hour-long failure of this
session, lying in wait on a path nobody had run. Now appends: QEMU_OPTS="$QEMU_OPTS $QOPTS".
NOT YET TESTED: `vm gui` itself. The clobber is fixed by inspection, not demonstrated.
