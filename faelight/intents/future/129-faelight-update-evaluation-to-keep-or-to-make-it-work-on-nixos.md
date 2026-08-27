---
id: 129
date: 2026-07-07
type: future
title: "faelight-update owns everything pacman cannot see"
status: planned
tags: [faelight-update, nix, fsh, faelight]
priority: low
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

## THE EVALUATION IS ANSWERED 2026-08-26 -- KEEP IT, and here is why

The original title asked: keep faelight-update, or make it work on NixOS. Both
halves are now wrong. There is no NixOS, and the keep question was settled by
measurement rather than by argument.

HIS POSITION GOING IN: it has no purpose because of omarchy-update. HALF TRUE.
The test settled it: faelight-update --dry-run reported Important: 1 (Global
NPM packages) -- a real pending update that pacman knows nothing about.

THE HONEST SCOPE: faelight-update is NOT a system-package updater on Omarchy.
omarchy-update owns that, and reimplementing it is how you end up fighting the
distribution. It is the one place that knows about EVERYTHING ELSE.

MEASURED, of 3,348 lines:
- flake_checker + flake_update (324 lines) -- genuinely dead, delete
- generation.rs (476 lines: timeline browser, rollback, closure diff) -- NOT
  dead, POINTED AT THE WRONG BACKEND. Omarchy has generations: snapper takes
  snapshots on package transactions and limine-snapper-sync puts them in the
  boot menu. The timeline and rollback halves repoint. The closure-diff half is
  genuinely Nix-only and has no snapper equivalent.
- cargo, npm, pip, rustup, flatpak, neovim, git, firmware -- ALL WORK ON ARCH
  UNCHANGED, and degrade honestly (nix not available) rather than crashing.

The checker contract is two functions per file: check_x_updates() ->
Vec<String> and update_x() -> io::Result<()>, guarding on the tool absence and
returning empty rather than erroring. rustup_checker.rs is the 27-line model.

SUCCESS CRITERIA
- [ ] flake_checker and flake_update deleted, no callers left
- [ ] generation.rs reads snapper; the closure-diff path degrades honestly
- [ ] omarchy-update availability REPORTED, never invoked -- the distribution
      owns system packages
- [ ] --count-only verified against a Quickshell bar widget
