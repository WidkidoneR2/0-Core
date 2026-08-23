---
id: 227
title: "fsh assumes NixOS in forty-two places, so portability is a test result rather than a property"
status: planned
type: infrastructure
priority: high
date: 2026-08-23
tags: [fsh, portability, platform, void, omarchy, architecture]
---

## Vision
fsh asks the platform what it is, in one place, and everything else reads the answer.

## The Problem
MEASURED 2026-08-23 across `faelight-shell/src/`, non-comment lines only:

    /home/christian        15      systemctl               5
    /run/current-system     7      journalctl              3
    /nix/store              5      nixos-rebuild           2
    nix-env                 1      pacman                  4  (see below)

Forty-two live sites. But they are THREE DIFFERENT PROBLEMS, and treating them as one
would produce a bad fix:

A -- WRONG ON EVERY SYSTEM, not only off NixOS.
    `mod.rs:11868` falls back to `"/home/christian"` when `$HOME` is unset.
    `mod.rs:14487` hardcodes `/home/christian/0-core/scripts/...`.
    These are defects for ANY user. Portability merely made them visible.

B -- REAL PLATFORM CAPABILITIES that should DEGRADE, not disappear.
    `systemctl`, `journalctl`, `nixos-rebuild`, `nix-store`, `/run/current-system`.
    A shell that shows journal logs on a systemd machine is not wrong. A shell that
    BREAKS without journald is.

C -- NOISE. `pacman` is a TYPO-CORRECTION list entry beside `vim`, `curl` and `ssh`
    (`mod.rs:9713`), not a package-manager call. Comments account for the rest.

## THE RULING (2026-08-23): DECLARE THE PLATFORM ONCE
Rejected: per-site detect-and-degrade. It spreads the same question across forty-two
places and answers it forty-two times, which is the shape this ledger keeps removing --
the doctor's hardcoded lists, `env`'s ten variables, three `sk` call sites that drifted.

CHOSEN: ONE module answers what the platform IS -- service manager, log source,
package manager, system-rebuild command, store paths -- and every caller reads it.
The design is Oligarchy's `custom.platform` applied to a shell.

⚠️ WHAT THIS IS NOT. It is not "support three distros". It is REMOVING ASSUMPTIONS.
The distinction matters because fsh may end up connected to Zero Core or independent,
and nobody knows which yet. A shell with no platform assumptions works BOTH ways -- so
this does not presume a direction, it makes both directions possible.

★ AND THE DESIGN RULE IT INSTALLS, which outlives the intent: every time the thought is
"I'll just make fsh assume X exists", ask -- would this work on Void, Omarchy, NixOS, or
any other distro? If no, that is a dependency worth questioning. Void is the sharp axis:
runit rather than systemd, optionally musl rather than glibc. Omarchy and NixOS both give
systemd and glibc, so neither would find these bugs.

## Success Criteria
- [ ] G1 THE CENSUS IS AN ARTIFACT, NOT A PARAGRAPH: a committed file listing every
      site with its category (A wrong-everywhere / B platform-capability / C noise).
      Produced mechanically so it can be re-run and diffed
- [ ] G2 CATEGORY A IS FIXED FIRST AND SEPARATELY: no fallback to a literal home
      directory, no hardcoded absolute path into one user's checkout. These are defects
      independent of portability and should not wait for an abstraction
- [ ] G3 THE PLATFORM MODULE EXISTS AND IS THE ONLY PLACE THAT ANSWERS: what is the
      service manager, the log source, the package manager, the system-rebuild command.
      One owner, per the same rule that produced `fuzzy_select` and `correlation`
- [ ] G4 A CATEGORY B CAPABILITY DEGRADES RATHER THAN BREAKS, demonstrated: with the
      platform reporting no journald, `logs` says so and exits cleanly rather than
      failing with a not-found error. The message names what is missing, per INT-215
- [ ] G5 A TEST CONTROLS THE PLATFORM, so this is provable without a second machine.
      `run_fsh_env` already exists (INT-221) and can set whatever the module reads
- [ ] G6 THE CENSUS RE-RUNS CLEAN: no site outside the platform module names a service
      manager, a package manager or a store path. A mechanical check, red first
- [ ] G7 each gate carries evidence per INT-158

## Non-goals
- Supporting distros nobody will run. The matrix is NixOS, Void, Omarchy -- and the
  point is the absence of assumptions, not the presence of three code paths.
- Removing NixOS capabilities. `nixos-rebuild` and `nix-store` are genuinely useful on
  the machine that has them; they simply must not be assumed.
- The separate fsh repository. That is structural and belongs in its own intent, though
  it expresses the same idea: a shell that lives inside 0-Core cannot be installed
  without it.
- Anything outside `faelight-shell/src/`. The engine and the other tools have the same
  question and a different answer; mixing them would make this too large to finish.
