---
id: 166
date: 2026-07-16
type: fix
title: "VM substituter points at dead Cachix; framework16 moved to Attic 2026-07-07"
status: cancelled
tags: [fix, bugfix]
---

## Vision
The VM pulls from a cache that works.

## The Problem -- MEASURED 2026-07-16
nix/hosts/vm/base.nix:

    nix.settings.extra-substituters = [ "https://faelight-forest.cachix.org" ];

Its comment used to say this was "mirroring hosts/framework16". IT IS NOT. framework16 left
Cachix for a self-hosted Attic on 2026-07-07. configuration.nix:38 records exactly why:

    "Replaced Cachix, whose multi-tenant content-dedup REFUSED to serve our crane paths
     (proven 2026-07-07; Attic clean-pull of the full 667-path closure verified)"

So the VM has been asking a cache that was MEASURED not to serve this repo's paths. It gets
nothing, and every VM build pays full price for the privilege.

The false "mirroring" comment was corrected on 2026-07-16 (eaba44f2) -- it now states what is
true and why it is wrong -- but the SUBSTITUTER ITSELF was deliberately left alone, because
the fix is not a copy-paste. See below.

This was the THIRD "mirrors framework16" claim found false in one evening. The others:
vm/login-mirror.nix called itself an "exact replica" of metal's login while rendering
button=lightmagenta against metal's button=white (fixed, INT-061 Phase 2, 328b2e4c), and
vm/base.nix:11 said "metal stays on plain systemd-boot" eight hours after INT-161 put
lanzaboote on metal. Three copies, three drifts, one evening. A comment saying "mirrors X" is
a claim that decays the moment X moves.

## The Solution -- why this needs testing, not a one-line edit
framework16 points at http://127.0.0.1:8080/faelight. The VM CANNOT COPY THAT LINE: 127.0.0.1
inside a guest means the GUEST, not the host. Under QEMU user networking the host is reachable
at 10.0.2.2 -- so the VM would want http://10.0.2.2:8080/faelight. That is untested, and it
depends on how the VM is networked (user mode vs bridged, INT-077's serial console setup).

Three candidate outcomes, all legitimate:
  1. 10.0.2.2 works -> the VM pulls from Attic, builds get fast, done.
  2. It does not -> REMOVE the dead Cachix line. A substituter that serves nothing is worse
     than none: it costs a lookup on every path and teaches nobody anything.
  3. The VM should not share the host's cache at all -> record that reasoning and remove it.
Do NOT leave it pointing at Cachix. That is the only outcome ruled out, because it is measured
to be useless.

## Success Criteria
- [ ] The VM's network mode determined by READING the config (INT-077), not assumed
- [ ] From INSIDE the VM: curl the host's Attic endpoint. It answers, or it does not. Measured
- [ ] If reachable: substituter + trusted-public-key updated; a build inside the VM PULLS a
      path from Attic. Proven by the build log, not by the setting existing
- [ ] If not: the dead Cachix line removed, with the reason recorded in base.nix
- [ ] Either way: vm/base.nix's comment matches reality when this closes
- [ ] Consider whether the substituter belongs in nix/profiles/base.nix after all -- INT-061
      deliberately left it out because the two hosts genuinely differ. If they stop differing,
      that reasoning changes

## Gate Check
🚫 166 -- cancelled: The VM substituter pointed at a dead Cachix and framework16 moved to Attic. Both are Nix binary caches. No store, no substituter. -- approved by: christian 2026-08-27
