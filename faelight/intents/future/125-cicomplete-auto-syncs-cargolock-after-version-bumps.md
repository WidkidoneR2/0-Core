---
id: 125
date: 2026-07-07
type: future
title: "cicomplete auto-syncs Cargo.lock after version bumps"
status: planned
tags: [cicomplete, cargo, workflow, core, intent-domain]
---

## Why
cicomplete bumps crate Cargo.toml versions but leaves Cargo.lock STALE. Every
version-bumped intent then requires a manual `cargo check -p <crate>` to regen the lock
(or the next crane build rejects the Cargo.toml-vs-Cargo.lock mismatch), plus a second
commit for the move + bump + lock. Hit THREE times in one session (INT-043 faelight-git,
INT-119 core+faelight-git, INT-120 faelight-shell). A repeating papercut = worth closing.

## Design (already scoped -- the hard part is done)
After cicomplete writes version N+1 to a crate's Cargo.toml, run:
    cargo update -p <crate> --precise <N+1>
A SURGICAL lock-only update -- changes just that crate's version line in Cargo.lock, no
full workspace re-resolve, no compile. Preferred over `cargo check` (which works but does
more than needed). Run it per bumped crate, right after the Cargo.toml write.

## Location + blast radius
Bump logic lives in the CORE ENGINE: faelight/engine/src/domains/intent/mod.rs (the
cicomplete version-bump prompt). The change recompiles + redeploys `core` -- the
highest-blast-radius crate in the system. A botched change breaks EVERY intent close-out.
Treat with care: focused session, thorough testing, not a rushed edit.

## Approach (demonstrated, not declared)
- Read the cicomplete bump function in domains/intent/mod.rs; find where each crate's
  Cargo.toml version is written.
- Add `cargo update -p <crate> --precise <newver>` right after each write (handle: multiple
  crates bumped in one cicomplete; cargo-not-on-PATH edge case; run from repo root).
- Build core, deploy (commit->rebuild per crane rule), then DEMONSTRATE: cicomplete an
  intent that bumps a crate, confirm Cargo.lock is consistent with NO manual step, and a
  crane build accepts it.

## Gates
- [ ] cicomplete, after bumping a crate version, leaves Cargo.lock consistent (no manual step)
- [ ] Existing cicomplete flow otherwise unchanged (intent move, checkpoint, prompts)
- [ ] A bumped crate builds via crane with no Cargo.toml-vs-lock mismatch
- [ ] Demonstrated on a real version-bumped cicomplete
- [ ] core builds clean, zero warnings

## Relationship
- Workflow-friction fix in the intent domain (core). NOT a 1.0.0 blocker.
- Removes a step hit on every version-bumped intent.

## Breadcrumbs (found 2026-07-07, NOT part of this intent)
- fsh glob papercuts: `*/src/` and `*/120-*.md` both failed to expand in fsh this session
  (fell back to find). Worth a small fsh glob fix or folding into INT-109 (fsh command
  handling). Noting so it is not lost.
