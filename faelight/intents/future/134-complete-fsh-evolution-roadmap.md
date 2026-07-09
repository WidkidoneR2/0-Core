---
id: 134
date: 2026-07-09
type: future
title: "Complete fsh Evolution Roadmap"
status: planned
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
- [ ] Roadmap landed in the repo at a durable path (e.g. `docs/fsh-evolution-roadmap.md`),
      so the reference in `docs/faelight-1.0.0-roadmap.md:11` resolves
- [ ] Every checkbox claiming DONE is verified against the ledger or the running shell --
      demonstrated, not declared. Unverifiable claims are unticked and noted, never bulk-ticked
- [ ] Each "done" item that has an intent carries its intent number; each that does not is
      either evidenced or reopened
- [ ] Lane 0 papercuts each become a real intent or are explicitly closed with evidence
      (bare `python3` REPL trap; `exec fsh` no hot-swap; the `[~]` INT-089 partial ->
      confirm INT-267/322 own the routing fix)
- [ ] The filter is applied to every unchecked item: KEEP (with lane + rough order), or CUT
      (with the reason). No item left unjudged
- [ ] Lane order decided and written down, honoring the doc's own rule:
      foundation -> stability -> features. Lane 5 (structured-data pipelines) stays an EPIC
      requiring its own decision record before any work starts
- [ ] The roadmap states how it stays true: who reconciles it, and when (e.g. at each fsh
      release, or whenever an fsh intent completes)
- [ ] **Version bump + release -- the close condition.** fsh is currently v3.0.5
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
