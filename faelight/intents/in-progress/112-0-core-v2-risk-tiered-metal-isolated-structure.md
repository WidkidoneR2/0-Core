---
id: 112
date: 2026-07-02
type: future
title: "0-Core v2 risk-tiered metal-isolated structure"
status: in-progress
tags: [structure, restructure, lockout, lanzaboote]
priority: high
---

## Vision
Evolve 0-Core's structure from the INT-061 two-domain charter (faelight/ + nix/)
to a RISK-TIERED, metal-isolated tree that communicates operational risk at a
glance and lets tooling ENFORCE safety gates. Organize by stability and
boot-criticality, not by feature -- the arrangement that minimizes lockout risk.
Done FROM STRENGTH: after the VM harness exists (it does, INT-061) and the whole
restructure can be VM-proven before touching metal.

## Builds ON, does not redo, INT-061
061 (complete) migrated everything into faelight/ (platform) and nix/ (OS),
VM-proven, cold-boot validated. 112 is the next evolution: add risk isolation
on top of that clean foundation. NOT a second blind move -- a deliberate,
harness-proven restructure designed with the known structure-changers baked in.

## The core insight (from external design input, 2026-07-02)
Organize by STABILITY / boot-criticality, not feature. Four trust levels:
- CRITICAL: flake.nix, hardware, bootloader, initrd, disk layout, SSH rescue,
  users(account), sudo, networking, secure-boot keys. Rarely moved. HIGH lockout.
- SYSTEM: services, desktop modules, kernel, drivers, fonts. Move with testing.
- USER: home-manager, shell, editor configs. Move freely. LOW lockout.
- DATA: docs, labs, assets, runtime, experiments. Move anytime. NONE.

## Proposed shape (refine at design time)
An isolated metal/ (or critical/) peer holding the lockout-class layer:
  metal/ { boot/ disks/ hardware/ users-account/ secure-boot/ rescue/ }
kept SEPARATE from nix/ (system+user config) and faelight/ (platform). The point:
metal/ changes very slowly and is the ONLY place a lockout can originate, so it
gets the strictest gates.

## RISK.toml + enforcement (the standout idea)
Per-directory risk metadata that TOOLING reads and enforces:
  # metal/boot/RISK.toml
  risk = "critical"
  requires = ["vm-test", "boot-test"]
  review = true
A git pre-commit hook reads RISK.toml: if a critical dir changed, it BLOCKS the
commit until the VM harness (INT-061) passes. Structure stops merely COMMUNICATING
risk and starts ENFORCING it. Demonstrated-not-declared, made structural.

## Must bake in the known structure-changers (decide once, restructure once)
- INT-059 (lanzaboote secure boot): REPLACES systemd-boot (framework16 currently
  boot.loader.systemd-boot.enable=true, UEFI, disko+LUKS). Needs a keys/signed-boot
  home. The metal/boot layer MUST have a lanzaboote-shaped slot so 059 slots in
  instead of restructuring the boot layer a second time. THIS is why "decide
  structure once" matters most.
- INT-086/087 (remove pinnacle -> miracle-wm): modules/desktop must be
  compositor-SWAP-ready, not hardcoded.
- INT-039 (friday-daemon / fridayd): needs a services home for the persistent
  daemon. (fridayd idea folds here / into 039.)

## Why now-ish (sequencing)
The VM harness (INT-061) is the precondition: it lets the ENTIRE v2 restructure
be simulated in a VM (move all lockout-class dirs, boot, verify login) and then
applied to metal in ONE proven move -- not a one-dir-at-a-time crawl. Design v2
with 059/086/087/039 known, VM-prove the whole tree, single metal application.

## Gates (when built)
- [ ] Target v2 tree decided (metal-isolated, risk-tiered) WITH 059/086/087/039 slots
- [ ] RISK.toml schema defined + per-critical-dir metadata written
- [ ] git pre-commit hook enforces RISK.toml (critical change -> harness must pass)
- [ ] Whole restructure VM-simulated (boots + login) before metal
- [ ] Single VM-proven metal application; cold-boot validated at 100%
- [ ] lanzaboote (059) slot verified to accept secure-boot without re-restructure

## Related
- Foundation: INT-061 (complete). Harness: INT-061 VM boot gate.
- Structure-changers to bake in: INT-059, INT-086, INT-087, INT-039.

---


## POST-RESTRUCTURE CHECKLIST -- fsh-test path debt (added 2026-07-07)
Any directory move in this restructure WILL silently break fsh-test, which hardcodes
repo paths in its assertions (faelight/rust-tools/fsh-test/src/main.rs). Precedent:
INT-061's restructure moved dirs under faelight/ and left 17 fsh-test failures with
stale pre-061 paths (rust-tools, engine, intents, runtime, pkgs->packages) -- found
only when the suite was run much later.

After ANY dir move here:
1. Update fsh-test path references AND top-level-structure expectations (e.g. a test
   doing `ls ~/0-core` expecting a dir that moved must expect the new top-level name).
2. Rebuild: nix develop ~/0-core#faelight-forest -c cargo build -p fsh-test
3. DEPLOY -- the `fsh-test` command runs the Nix-DEPLOYED binary, not target/debug.
   A cargo build alone shows green while the live command still fails. Must `dep`.
4. Confirm 82/82 on the deployed binary before considering the move done.
