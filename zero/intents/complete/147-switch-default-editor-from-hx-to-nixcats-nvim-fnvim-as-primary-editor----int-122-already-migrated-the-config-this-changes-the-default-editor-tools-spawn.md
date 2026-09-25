---
id: 147
date: 2026-07-12
type: future
title: "Switch default $EDITOR from hx to nixCats nvim (fnvim) as primary editor -- INT-122 already migrated the config; this changes the default editor tools spawn."
status: complete
tags: [fsh, editor, nixcats, config]
---

## Vision
`nvim` (the INT-122 nixCats "forest-nvim" build) is the primary editor across the system --
every tool that spawns `$EDITOR` opens nvim, resolved from a single declarative source with no
competing definitions. Moving off hx, cleanly, not by piling on another override.

## The Problem
`$EDITOR`/`$VISUAL` were set to `hx` in TWO places that disagreed once we started changing them:
- `nix/home/christian/home.nix:111` -- `home.sessionVariables` (home-manager, per-user)
- `nix/hosts/framework16/configuration.nix:178` -- `environment.sessionVariables` (system-wide)
Duplication across the system and home-manager layers is the real defect: whichever wins depends
on profile.d source ordering, which is fragile and non-obvious. On top of that, NixOS ships a
nano default that also asserts `EDITOR` at the system level, so removing our line surfaced nano.

## The Solution
Single source of truth: home-manager owns `$EDITOR`, the system layer asserts nothing.
1. `home.sessionVariables` -> EDITOR=nvim, VISUAL=nvim (the sole intended source).
2. Remove the system-level `environment.sessionVariables` EDITOR/VISUAL entry entirely.
3. Disable the NixOS nano default (`programs.nano.enable = false`) so it stops claiming EDITOR.
Verification done by resolving the actual login chain on demand
(`bash --noprofile --norc`, unset EDITOR, source /etc/profile then home-manager vars) rather than
rebooting to observe -- the deterministic, fsh-proof test.

## Success Criteria
- [x] home.sessionVariables sets EDITOR=nvim + VISUAL=nvim; deployed -- verified home-manager
      hm-session-vars.sh shows `export EDITOR="nvim"` on the deployed system
- [x] system-level environment.sessionVariables EDITOR/VISUAL removed from configuration.nix;
      home-manager is the sole intended source -- verified: no EDITOR/VISUAL entry remains in
      the system config (only home.nix defines it)
- [x] NixOS nano default disabled (programs.nano.enable = false); deployed -- verified activation
      removed /etc/nanorc, nano no longer asserts an editor default
- [x] login chain resolves EDITOR=nvim with NO competing value -- demonstrated deterministically:
      `bash --noprofile --norc` with EDITOR unset, sourcing /etc/profile then home-manager vars,
      resolves empty -> nvim (home-manager wins, nothing overrides it). The set-environment nano
      line is inert in the interactive login chain.
- [x] nvim is the INT-122 nixCats forest-nvim build -- verified: `which nvim` resolves the
      `-forest-nvim` store path
- [x] work committed -- commit 6d9a23e1 (home.nix + configuration.nix)

## Relationship
Builds on: INT-122 (nixCats forest-nvim migration -- the nvim build already existed; this switches
the default editor to it). 
Filter: single declarative source for a user preference deepens reproducible control; competing
overrides across layers erode it. In-filter.

## Notes
- Live-in-current-shell caveat: sessionVariables freeze at login, so THIS session still shows
  EDITOR=hx (started before the deploy). Config is proven-correct via the login-chain resolution
  test; a reboot (already due for generation drift) flushes the frozen value -- housekeeping, not
  a test, since the chain already resolves nvim.
- Lesson banked: for any env/sessionVariable question on this box, resolve the login chain with
  `bash --noprofile --norc` sourcing /etc/profile + home-manager vars. fsh freezes its own session
  env and its env/bash builtins intercept clean-shell launches, so in-session checks lie; the
  on-demand chain resolution is the truth and needs no reboot.
- `programs.nano.enable = false` also removes the nano binary from PATH. Intended (moving to nvim);
  if nano-as-a-command is ever wanted back, add it to environment.systemPackages separately.
