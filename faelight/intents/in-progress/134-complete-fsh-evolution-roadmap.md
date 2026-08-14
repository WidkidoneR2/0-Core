---
id: 134
date: 2026-07-09
type: future
title: "Complete fsh Evolution Roadmap"
status: in-progress
tags: [fsh, faelight-shell, roadmap, ledger]
---

## Vision
Turn the fsh Evolution Roadmap from a loose document into a LIVING, ledger-tracked plan:
landed in the repo, reconciled against what is actually built, governed by its own filter,
and sequenced so it SPAWNS intents rather than accumulating checkboxes.

## The Problem
1. **The doc is not in the forest.** `docs/faelight-1.0.0-roadmap.md:11` says
   *"see fsh-evolution-roadmap"* -- but no such file exists in the repo. The roadmap that
   governs fsh's future lives outside the system it governs.
2. **Its checkboxes are unverified.** It claims items done (parallel execution, session
   save/load/replay, NL `?` prefix, sandboxed execution, split panes...) with no evidence
   link. Some cite intents (057, 060, 062, 063, 064, 065, 089, 095, 033, 046, 053); most do
   not. This is precisely the drift INT-130 was opened to fix: status asserted, not shown.
3. **Its own sequence is stated but not enforced:** foundation (INT-060) -> stability
   (INT-057) -> feature lanes. Lane 0 still lists live papercuts (bare `python3` REPL trap;
   `exec fsh` not hot-swapping the rebuilt binary) that outrank every feature lane.
4. **~100 unchecked items across 8 lanes**, unprioritized. Without the filter applied, the
   roadmap is a wish list, not a plan.

## The Solution
Reconcile and own it. NOT execute the lanes -- that is fsh's whole future and would never
close. This intent makes the roadmap TRUE and ACTIONABLE, then SHIPS it as an fsh release.
A document says what it says; a version bump is a claim you have to stand behind.

**The filter is the spine** (from the doc, kept verbatim as the governing criterion):
> A feature earns a place only if it deepens understanding + authorized, reproducible
> control. Opaque convenience and auto-magic are cut.

Every unchecked item gets judged against that filter. Items that fail it are CUT, not
deferred -- the doc already models this ("Cut -- fails the filter": smart cd with typo
correction; silent auto-magic).

## Success Criteria
- [x] Roadmap landed in the repo at a durable path (e.g. `docs/fsh-evolution-roadmap.md`),
      so the reference in `docs/faelight-1.0.0-roadmap.md:11` resolves <!-- STAMP-134-DONE / 2026-07-10: VERIFIED -- docs/fsh-evolution-roadmap.md exists (6.2k, 2026-06-26); faelight-1.0.0-roadmap.md:11 references it by name ('see fsh-evolution-roadmap'); reference resolves to the real file. -->
- [x] Every checkbox claiming DONE is verified against the ledger or the running shell -- <!-- 2026-07-11: each 'verify+keep' DONE claim checked against registry.rs + commands/mod.rs dispatch arrays + live behavior on the DEPLOYED binary. terminate/kill/jobs (INT-095) verified live; session save/load/replay (mod.rs:888-1032); ? / run / sandbox / yazi (dispatch); syntax highlighting (mod.rs:91/126, partial-honest); git/nix/health/intent/Friday in prompt+bar (live). Evidence anchored in the roadmap. Method note recorded: verify on own line, never through a pipe (pipes route forest words to sh). None required unticking -- all claims resolved to real source/behavior. -->
      demonstrated, not declared. Unverifiable claims are unticked and noted, never bulk-ticked
- [x] Each "done" item that has an intent carries its intent number; each that does not is <!-- 2026-07-11: every [x] item now carries an intent number (095,096,057,060,062,033,053,065,064,046,063,024,269) OR explicit evidence (source refs, flake.nix postFixup, focus.toml mechanism, live-verified). ade + first-class-command + active-intent-bar evidenced (no dedicated intent). None left unnumbered-and-unevidenced. -->
      either evidenced or reopened
- [x] Lane 0 papercuts each become a real intent or are explicitly closed with evidence <!-- 2026-07-11: python3 REPL trap -> INT-143 filed; `exec fsh` hot-swap -> SOLVED by INT-096 `reload` (verified main.rs:699+1012-1044); [~] sh-routing -> INT-089 clarity-fix complete, deeper routing corrected from phantom INT-267/322 to accurate 'unfiled future work, out of 134 scope'. All three resolved. -->
      (bare `python3` REPL trap; `exec fsh` no hot-swap; the `[~]` INT-089 partial ->
      RESOLVED: INT-267/322 are PHANTOM -- never filed; deeper sh-routing fix is unfiled future work, out of 134 scope)
- [x] The filter is applied to every unchecked item: KEEP (with lane + rough order), or CUT
<!-- evidence 2026-08-14: 27 items, ALL judged, zero unjudged. Commits 966d6122, 4c02db34, 144a4bf7,
     0c239a9e, 1808451a, 3954c3e8 -- each ruling written into the roadmap with what was measured.
     ⚠️ THE COUNT WAS 27, NOT the ~38 this gate was estimated at, and two items thought pending had
     already been ticked.
     ★★ NINE WERE ALREADY BUILT, which is what this gate existed to find: aliases with arguments
     (args append like bash) · autocomplete from history (the same Hinter as fish-style
     autosuggestions, cross-referenced not double-counted) · secret management (faelight-vault, its
     own crate) · event-driven hooks (the `on` DSL, one trigger fired 2081 times) · reversible undo
     (tracks mv/cp/rm) · undo command editing (inherited from rustyline keymap.rs) · the plugin
     mechanism · plus half-credit on interactive tables and the task runner.
     TEN CUT WITH REASONS: macro system · quick notes · command dependency graphs · popup palettes ·
     native Rust scripting · compile-to-binaries · named collections · env var permissions · shell
     script generation. EIGHT KEPT WITH LANE AND OWNER, including three delegated to INT-144,
     INT-170 and INT-188.
     ★★★ THE METHOD RULE THIS EARNED, the mirror of the one the sweep already carried: A SYMBOL
     BEING ABSENT IS NOT THE FEATURE BEING ABSENT. faelight-vault was invisible to a grep of the
     builtins because it is its own crate; `on` was invisible to a search for "hooks"; `pv` was
     invisible because the function name is not the command word. Evidence overturned my own
     recommendation twice.
     ⚠️ AND IT FOUND A LIVE DEFECT: INT-221 filed, because `pick` calls skim, which is not installed,
     while fzf is. Found only because a ruling required running the thing. -->
      (with the reason). No item left unjudged
- [x] Lane order decided and written down, honoring the doc's own rule:
      foundation -> stability -> features. Lane 5 (structured-data pipelines) stays an EPIC
      requiring its own decision record before any work starts
- [x] The roadmap states how it stays true: who reconciles it, and when (e.g. at each fsh
      release, or whenever an fsh intent completes)
- [x] **Version bump + release -- the close condition.** fsh is currently v3.0.5
      (`faelight/rust-tools/faelight-shell/Cargo.toml:4`; mirrored in Cargo.lock and in every
      `faelight/runtime/checkpoints/*.toml`). Once the roadmap is reconciled and Lane 0 is
      resolved, bump it per `decisions/102` (version-bumping faelight tools on Nix) and cut
      the release per `decisions/121` (release process + naming convention).
      Release notes must state: what was VERIFIED (not assumed), what the filter CUT and why,
      and which lane is next. The bump is the proof the reconciliation landed -- a release you
      must stand behind, not a document that can be quietly edited.
      Self-verifying: the checkpoint written at `cicomplete` will record the new version.
      The version NUMBER is deliberately left unpinned: decided at cicomplete from the actual
      diff, not declared now. Documentation-only -> patch. Lane 0 papercuts FIXED -> minor.
      The sh-routing fix (INT-267/322) landing -> major, since it changes fsh's execution
      model -- but that is a bigger intent than this one, and 134 must not grow into it.
<!-- evidence 2026-08-14. THE THREE THINGS THE NOTES MUST STATE, stated here since this closes on
     the TOOL axis rather than a forest release:
     WHAT WAS VERIFIED, not assumed: 27 roadmap items judged with a measurement behind each. NINE
     were already built and the document understated the shell to anyone reading it -- vault, the
     `on` trigger DSL, undo for file ops, undo in the line editor, aliases taking arguments, history
     autocomplete, the plugin loader, and half-credit on interactive tables and the task runner.
     WHAT THE FILTER CUT AND WHY: ten items, each with a reason recorded in the roadmap. The pattern
     across most of them is a SECOND OWNER of an idea that already has one -- macros beside aliases
     and scripting and triggers, notes beside the ledger, collections beside run --list, a dependency
     graph beside `core deps`, env permissions beside the sandbox, palettes beside three TUIs. One
     was cut on DIRECTION rather than absence: shell script generation, because the spine lowers to
     argv rather than to text and a generator would pull against the rebuild.
     WHICH LANE IS NEXT: LANE 4 (interactive troubleshooting) is next and actionable. LANE 5
     (JSON/YAML/TOML) is next-but-GATED -- gate 6 of this intent requires the epic to have its own
     decision record before any work starts. The UX remainder rides alongside as small increments.
     THE VERSION: MINOR, his ruling. 3.6.14 -> 3.7.0. Per docs/RELEASE.md a minor is "significant
     features, batches of intents complete", and while no shell code changed under 134 itself, the
     roadmap now describes a materially different tool than it did this morning.
     ⚠️ SCOPE CORRECTION, recorded rather than quietly satisfied: this gate says "cut the release per
     decisions/121", and 121 describes a FOREST release -- plan/preview/verify/publish, the
     generation triad, a codename. That is NOT what this earns. decisions/102 Decision 1 (settled
     today) separates the axes: a forest release SNAPSHOTS and never bumps, and 121's own inclusion
     rule keeps in-progress intents out -- there are two actives. So this closes on the TOOL axis.
     Faelight OS stays 1.0.0 "Morphwood". The bump is still the proof the reconciliation landed.
     ⚠️ The gate text says "fsh is currently v3.0.5". It was 3.6.14 by the time this closed, which
     measures how long this gate stood open rather than anything about the work. -->

## Explicitly out of scope
Building the features. This intent produces a trustworthy, sequenced plan; the lanes then
spawn their own intents. If this charter starts growing feature work, it has failed.

## Notes
- Overlaps INT-130 (reconcile mis-gated completed intents). The fsh-specific checkbox audit
  may fold into that sweep, or run alongside it -- decide at cistart. Do NOT do both blindly.
- The doc's Lane 0 explicitly ranks builtin shadowing highest, marked FIXED by INT-095
  (2026-06-26). Verify that on the deployed binary, per the trap that has bitten twice:
  a `cargo build` is not a deploy.
- Filed after fixing `intent add`'s numbering (commit 039e1211); this is the first intent to
  land on a correctly-counted ledger (133 -> 134).
