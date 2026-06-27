---
id: 074
date: 2026-06-22
type: future
title: "Faelight-Update v-next: update manager + generation browser"
status: in-progress
tags: [faelight-update, nix, flake-update, generations, generation-browser, tui, post-1.0.0]
---

## Why
Faelight-Update is the forest's update tool. Post-1.0.0 it grows to oversee the whole
update-and-generation story -- two capabilities folded into ONE existing tool rather than
two new silos: an update manager (deliberate, legible flake updates) and a generation
browser (the generation timeline, made legible). Both serve manual-control and
understanding-over-convenience, and compose with INT-073 (gen control), INT-034 (triad),
and INT-056 (recovery).

## What
Update manager:
- Per-input view of what `nix flake update` would change; update one input at a time.
- Closure diff shown BEFORE the switch (nvd / nix store diff-closures).
- Rebuild gated on review.
Generation browser:
- Browse generations: timeline, dates, sizes.
- Closure diff between any two generations.
- Each generation tied to its commit + intent (via INT-034 triad data).
- Roll back / boot a chosen generation (serves INT-056 recovery).

## Approach
Extend the existing Faelight-Update tool (Rust/ratatui). The update side wraps
flake-update + nvd/diff-closures with per-input granularity and a review gate. The
generation side reads the generation list + closure diffs + INT-034's commit/intent
mapping. Wrap nvd; do not reinvent the diff engine.

## Phases
Phase 0 -- survey current Faelight-Update; map where update-manager + gen-browser slot in.
Phase 1 -- update manager: per-input flake update + pre-switch closure diff + review gate.
Phase 2 -- generation browser: timeline + closure diff between gens + roll-back.
Phase 3 -- integration: tie generations to commit + intent (depends on INT-034 data).

## Gates
- [x] Phase 0: current Faelight-Update surveyed; update-manager + gen-browser integration points recorded
- [x] update manager: per-input flake update with pre-switch closure diff and a review gate
- [ ] generation browser: timeline + closure diff between generations + roll-back
- [ ] generations tied to commit + intent (via INT-034 triad data)

## Notes
- Consolidates two post-1.0.0 ideas into the existing tool, not two new silos.
- Composes with INT-073 (gen control), INT-034 (triad), INT-056 (recovery).
- Build on nvd / nix store diff-closures; do not reinvent the diff engine.

## The Rule
"See what changes before it changes you." 🌲


## Phase 0 -- DONE (2026-06-26): survey + integration map
ARCHITECTURE (surveyed): main.rs (1802L) orchestrates; per-source CHECKER modules each expose
check_X_updates()/update_X() -- cargo, npm, pip, rustup, flatpak, firmware, neovim, yazi, git,
cleanup. TUI in tui_v2.rs (405L, ratatui): UpdateTUI state + CategoryState, category/package
nav, render_categories/render_packages/render_status_bar. Data model: UpdateCategory{items}.
CLI (clap): --dry-run/-n, --interactive/-i, --preview, --json, --only/--skip, --maintain, etc.

GAPS 074 FILLS (integration points recorded):
 1. UPDATE MANAGER (flake) -- NO flake_checker.rs exists. git_checker only pulls ~/repos.
    -> NEW flake_checker.rs (checker pattern): parse `nix flake update`/metadata for per-INPUT
       update candidates; show pre-switch closure diff via nvd / nix store diff-closures;
       gate the rebuild on review. Slots into main.rs's category aggregation like any checker.
 2. GENERATION BROWSER -- entirely absent (no gen module, no TUI view).
    -> NEW generation.rs: read generation list (nix-env --list-generations / profile), dates,
       sizes; closure-diff between any two gens (wrap nvd); rollback/boot a chosen gen.
    -> NEW TUI view/tab in tui_v2.rs alongside the update view (timeline + diff + rollback).
 3. TRIAD tie-in (gen -> commit + intent) -- reads INT-034 data. DEFERRED to Phase 3.

VERDICT: the checker-module + ratatui pattern extends cleanly -- flake_checker slots in as a
checker; the generation browser is a new TUI view. No architectural rework needed.
SEQUENCING for next session: Phase 1 (flake_checker per-input + closure diff + review gate),
then Phase 2 (generation.rs + gen TUI view), then Phase 3 (INT-034 triad). Build on nvd; do
NOT reinvent the diff engine.


## Phase 1a -- DONE (2026-06-27): flake_checker.rs (legible per-input view)
Built rust-tools/faelight-update/src/flake_checker.rs following the existing checker pattern
(returns Vec<UpdateItem>, mod-declared, slotted into check_all_updates as a "❄ Flake Inputs"
category). check_flake_updates() runs `nix flake metadata --json` against ~/0-core and reports
each root input: locked rev (8-char) + lock age (Nd ago) + tracked ref. Unit test
(parses_basic_metadata) PASSES; cargo check clean. Live output verified against the real flake
-- all 7 inputs render correctly:
  crane 469fd08d (8d) github · disko ff8702b4 (16d) github · home-manager 7bfff44b (6d)
  release-26.05 · nixos-hardware 08018c72 (11d) github · nixpkgs e8210c64 (8d) nixos-26.05 ·
  nixvim 7afca458 (6d) nixos-26.05 · pinnacle 5ae72933 (6d) github
This is the legible per-input FOUNDATION. Phase 1 gate stays UNCHECKED -- 1a does enumeration
only; 1b adds the actual per-input update + pre-switch closure diff (nvd) + review gate.

### SIGNIFICANT FINDING (de-Arching needed -- blocks the tool on NixOS):
faelight-update still carries Arch-era code that BREAKS it on NixOS:
- TWO clap bugs: `verbose` and `count_only` fields lacked #[arg(...)] attributes -> clap
  panicked at startup ("positional ... but action is SetTrue"), making the tool UN-RUNNABLE.
  Fixed both (added #[arg(short, long)] / #[arg(long)]). The tool literally could not run before.
- check_all_updates() + print_suggestions() call `sudo pacman`/`paru`/`pacman -Qtdq` even in
  --dry-run, and these fire REGARDLESS of --only (so `--dry-run --only flake` still blocks on a
  sudo password prompt). The Arch update/maintenance paths must be removed or NixOS-gated before
  the tool is usable. This is real 074 scope (or a dedicated de-Arch cleanup): the update manager
  cannot function on NixOS until the pacman/paru/sudo-maintenance code is retired. flake_checker
  is the correct NixOS-native direction; the old Arch checkers need retiring.

NEXT (074): Phase 1b -- per-input update probe + closure diff + review gate; AND de-Arch the
tool (retire pacman/paru/Arch-maintenance so --dry-run is truly dry and the tool runs clean).


## De-Arch pass -- DONE (2026-06-27): faelight-update now RUNS on NixOS
The Arch residue was deep (~15 sites). Removed/rewired across 5 clusters, cargo-checking after
each (zero warnings throughout), ~470 lines of Arch code gone:
- Cluster 1 (checkers): removed pacman/paru/aur-rebuild/pacnew checker calls + deleted the 5
  dead fns (check_pacman_updates, parse_pacman_output, check_paru_updates, check_pacnew,
  check_aur_rebuilds).
- Cluster 2 (suggestions/preflight): removed pacman-orphans, 2x pacnew, mirrorlist-age,
  partial-upgrade blocks. REWIRED get_drift_score from /var/log/pacman.log -> flake.lock mtime
  (NixOS-native drift: "days since last flake update", same FRESH/LOW/MED/HIGH/CRITICAL scale).
- Cluster 3 (maintenance): rewrote run_maintenance 129->42 lines -- kept cargo-cache +
  journal-vacuum (cross-platform), dropped pacman-cache/orphans/pacnew/pacdiff, points users to
  the forest's existing `nhclean` for store cleanup (no duplication).
- Cluster 4 (update paths): removed update_pacman + update_aur + their dispatch arms
  (catch-all _ => handles the rest).
- Cluster 5 (lists/filters): dropped pacman/aur from --only help + category-filter arms (added
  "flake"), removed pacman/paru from the 2 critical-package lists + IMPORTANT list, removed the
  cleanup_pacman_cache call + the fn in cleanup_checker.rs (kept cleanup_cargo_cache).
RESULT (verified live): `faelight-update --dry-run` runs START TO FINISH with NO sudo prompt,
no panic. Shows the NixOS-native System Profile (drift FRESH from flake.lock), "Checking flake
inputs...", the cross-platform checkers, and a clean Update Summary. The tool was UN-RUNNABLE
before (clap panic + sudo-pacman hang); it now works on NixOS. Also fixed the two latent clap
bugs (verbose/count_only missing #[arg]) earlier this session.
This unblocks Phase 1b (per-input update + closure diff + review gate) and Phase 2 (generation
browser) -- both can now be built on a tool that actually runs.


## Phase 1b -- DONE (2026-06-27): per-input update + closure diff + review gate -- PHASE 1 GATE MET
Built rust-tools/faelight-update/src/flake_update.rs (run_flake_update) + wired --flake-update
<INPUT> flag. The SAFE FLOW ("see what changes before it changes you"):
  1. back up flake.lock (revert point)
  2. nix flake update <input>  -- updates just that input's lock entry (reversible)
  3. nixos-rebuild build --flake .#framework16  -- BUILD ONLY, unprivileged (-> ./result)
  4. nvd diff /run/current-system ./result  -- the closure diff, read-only
  5. REVIEW GATE: dry-run reverts the lock; live mode prompts apply? (y/N) -> sudo switch on y
SAFETY PROVEN LIVE (twice): (a) when the build FAILED (untracked module file), the flow caught
it and reverted the lock automatically -- "build failed -- lock reverted, system untouched";
(b) successful dry-run on `disko` showed a REAL nvd diff -- "Closure size: 1658 -> 1658, 13
paths added, 13 removed, delta +0, disk usage -79.0KiB, No version or selection state changes"
-- then reverted the lock, system byte-identical. Nothing touches the running system until
explicit y at the gate. (Re-confirmed the flake-tracking lesson: nixos-rebuild build evaluates
git-TRACKED files, so the new module had to be git-added before the build could see it.)
Phase 1 (update manager) gate now MET: per-input update + pre-switch closure diff + review gate.
NEXT: Phase 2 -- the generation browser (timeline + closure diff between gens + rollback).
