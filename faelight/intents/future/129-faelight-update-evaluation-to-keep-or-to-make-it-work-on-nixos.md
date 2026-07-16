---
id: 129
date: 2026-07-07
type: future
title: "Faelight-Update evaluation to keep or to make it work on NixOS"
status: planned
tags: [faelight-update, nix, fsh, faelight]
priority: high
---

## Why
faelight-update is Arch-era heritage. A live run on NixOS (2026-07-07) shows it PARTLY
works but has stale assumptions. Decide: adapt it to be a true NixOS updater, or retire
it in favor of the flake-native path (update-flake + deploy). Until resolved, Friday does
NOT teach `fu`/`update` as commands (see INT-117) -- they're unverified on NixOS.

## Findings (live run 2026-07-07, v1.0.0)
WORKS on NixOS:
- Host/kernel/WM/health detection correct (framework16, mango, 100%).
- Flake-input detection WORKS: found 9 available (attic, crane, disko, git-hooks,
  home-manager, +4), flagged git-hooks as important. So it already SEES the nix update surface.
- Neovim plugins, 0-Core workspace, git repos, firmware, flatpak checks all run.

BROKEN / stale on NixOS:
- Shell shows "bash" -- does NOT detect fsh (faelight-shell) as the shell. Wrong.
- Cargo check errors: `no such command: install-update` -- depends on cargo-install-update
  (an Arch/imperative-cargo assumption). On NixOS, cargo tools are flake-managed, not
  `cargo install`-ed -- this check may be meaningless here.
- Detects updates but it is UNKNOWN whether it can safely APPLY them the NixOS way
  (flake input bump -> rebuild). Applying must go through update-flake + deploy, not
  imperative package installs.

## The decision this intent makes
Option A -- ADAPT: make faelight-update NixOS-native. Detect fsh. Drop/replace the
  cargo-install-update check with the flake-managed reality. Make "apply" mean:
  nix flake update (or per-input) -> deploy. Keep it as the single update dashboard.
Option B -- RETIRE: the flake-native path (update-flake alias = nix flake update +
  rebuild-safe) already does the real work. faelight-update becomes a thin status
  display or is removed. `update`/`fu` aliases repoint or drop.

## Success Criteria (draft -- refine at cistart)
- [ ] Decision made: adapt (A) or retire (B), with reasoning recorded
- [ ] If A: shell detection fixed (fsh recognized); cargo-install-update dependency
      removed/replaced; "apply updates" routes through flake update + deploy safely
- [ ] If A: demonstrated applying a flake input update end-to-end (e.g. git-hooks) via
      the tool, landing in a new generation
- [ ] If B: update-flake/deploy confirmed as the sanctioned path; aliases reconciled;
      faelight-update retired or reduced to status-only
- [ ] Once resolved: Friday can teach the verified update command (unblocks the INT-117
      `fu` fact that was deliberately omitted)

## Relationship
- Unblocks a deferred INT-117 item (the `fu`/update knowledge fact, omitted pending this).
- Related INT-125 (Cargo.lock sync) and the deploy/rebuild script family.
- Feeds INT-128 (once the real update command is known, it's a verified nixos fact).

## The Rule
"An updater that speaks the old system's language is a map to a country you left.
 Teach it the new roads -- or retire it and trust the ones you built." 🌲

teach/main.rs faelight-update entry has accuracy drift beyond the Arch language 117 fixed:
- version says "3.3.0" but the live tool reports v1.0.0
- replaces: Some("topgrade") -- topgrade was removed May 2026; nothing to replace
Both are accuracy, not Arch-language, so 117 left them. Fix as part of the 129 evaluation
(if faelight-update is adapted, correct the teach entry to match reality; if retired,
remove the entry).
