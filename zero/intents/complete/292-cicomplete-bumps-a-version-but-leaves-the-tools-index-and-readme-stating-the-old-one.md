---
id: 292
date: 2026-10-17
type: future
title: "cicomplete bumps a version but leaves the tools index and README stating the old one"
status: complete
tags: [cicomplete, versions, docs, tools, ledger]
depends_on: [291]
---

## Vision
Closing an intent that bumps a tool leaves no document stating the old version. Either the close
refreshes every document that states a version, or, if INT-291 rules that only Cargo.toml and the
derived index state one, it refreshes the index and refuses on any other. cicomplete never writes
prose.

## The Problem
Measured 2026-10-17, closing INT-290.

- cicomplete bumped novashell 5.3.3 to 5.3.4 and nsh-test 2.0.5 to 2.0.6 in Cargo.toml, Cargo.lock
  and zero/registry/tools.toml, and its receipt says so. It left zero/tools/README.md stating
  novashell 5.3.3 and nsh-test 2.0.4 (already two bumps behind). The fix was a hand-run
  `zero-docs readme-generate`, which also rewrote 19 per-tool READMEs and the hand-maintained one
  (INT-291).
- No door refuses a commit that leaves a stated version behind: d reported 0 failed with the stale
  index in the tree.
- How cicomplete finds what to bump is known from INT-102: it reads `git log --grep=INT-<id>`, so a
  commit that does not name the intent is invisible to it (the line was intent/mod.rs:1342 in
  August; G0 re-reads it). Decisions D4 (cicomplete suggests, the human picks) and D5 (no mapping
  from intent type to semver level) stand and are not reopened here.
- The tools index states a version for every tool and nsh-test reads its count, but nothing checks
  its versions.
- INT-212 is a different defect (cicomplete never requires gates to exist; 10 gates open, none
  done). This intent does not extend it.

## The Solution
Wait for INT-291 G1a, then pick one of two shapes there:

- A. cicomplete runs the index refresh for the crates it bumped, and nothing else.
- B. Only Cargo.toml and the derived index state a version, the index refresh is part of the close,
  and the close refuses if any other document states a different one.

Either way the close ends with a check that every document stating a version agrees with Cargo.toml.

## Success Criteria
- [x] G0 recon, read-only. Evidence is the output, kept in ~/.cache/zero:
      a. The cicomplete bump code today: file:line of the detection, every file it writes, where the
         receipt is written.
      b. Every document that states a tool version (index, per-tool READMEs, registry, docs/, CHANGELOGs,
         census) against its Cargo.toml, as a table.
      c. Which of those INT-291 rules away.
      d. What d and the doors check about versions today.
    Findings, 2026-10-21 (output in 292-g0.txt and 292-g0-reads*.txt):
    a. Detection is touched_crates (intent/mod.rs:1113-1138): git log --grep=INT-<id> --since=60
       days ago. write_versions (:1613) writes Cargo.toml (zero_core::version::apply_bump), the
       registry row (zero_core::registry::write_version) and Cargo.lock (sync_lock :1737, cargo
       update -p <crate> --precise <new>). A sync_lock failure only prints a yellow warning (:1762).
       The receipt is appended at :2162, after write_versions at :2122.
    b. Documents that state a version: Cargo.toml (the owner), zero/registry/tools.toml,
       Cargo.lock and zero/tools/README.md (generated). History files are never refreshed.
    c. INT-291 ruling a: Cargo.toml owns a tool's version; no per-tool README states a version,
       a date, a status or a category.
    d. d reported 0 failed with a stale index in the tree (measured 2026-10-17), and cicomplete
       never calls zero-docs. zero-docs check already exits 1 and names the stale tools.
- [x] G1 rulings after INT-291 G1a, each approved by Christian: where the refresh lives (cicomplete,
      ship or the generator), and exactly what the close refuses on.
    Rulings, approved by Christian 2026-10-21:
    a. Shape B: only Cargo.toml, the registry, Cargo.lock and the derived index state a version.
    b. The refresh lives in cicomplete (complete_intent), after the last bump in write_versions:
       zero-docs readme-index by subprocess, then an index-only check. Not ship (it runs before
       the close), not a generator trigger (nothing triggers one), not a hand edit of an index
       row (generated files are regenerated, never hand-edited).
    c. The close refuses when: the zero-docs binary is missing, checked before any write; the
       refresh exits non-zero (the message says Cargo was written and gives the finishing command
       and the rerun path); or, after the refresh, the registry row, the Cargo.lock entry or the
       index row of a bumped crate differs from Cargo.toml. This also closes the sync_lock gap.
    d. A close that bumps nothing runs no refresh. D4 and D5 stay as they are.
    e. Filed with core intent find, not fixed here: --grep=INT-<id> is an unanchored substring,
       and --since=60 days ago drops commits of an intent open longer than that.
- [x] G2 red. A close on a scratch clone of the repo, with a scratch intent that bumps a tool, leaves a
      stated version behind. The case fails today.
    Evidence 2026-10-21 (scratch tree, deployed core before the fix): `core intent complete 9001
      --bump demo=patch` exited 0 and bumped demo 1.2.3 -> 1.2.4, the index row still read
      `demo | 1.2.3`, and `zero-docs check` exited 1: stale for demo. Script:
      ~/.cache/zero/292-close-d9d0560c.py.
- [x] G3 fix. G2 green. The smallest change per ruling; D4 and D5 untouched; no new crates.
    Evidence 2026-10-21: write_versions (intent/mod.rs) finds zero-docs before any write, runs
      zero-docs readme-index after the last bump, then refuses unless the registry row, the
      Cargo.lock entry and zero-docs check agree with each bumped Cargo.toml. zero-docs is
      unchanged: its check already exits 1 only for a stale index. 8 tests in int292_refresh_tests
      pass and the 53 domains::intent tests pass. Plan Seal 41aa3a1cab83b085.
- [x] G4 live. The same scratch close on the deployed cicomplete leaves every stated version equal to
      Cargo.toml, and a deliberately stale document makes it refuse.
    Evidence 2026-10-21 (shipped core 4.4.2, new shell): the same scratch close printed `tools index
      refreshed and checked`, left `demo | 1.2.4` and `zero-docs check` exited 0. With a zero-docs
      whose readme-index does nothing the close exited 1 (`bump written but the docs do not
      agree`), named demo 1.2.4 and did not move the intent.
- [x] G5 doors: one ship at the end where code changed; the deployed tools tested in a new shell;
      nsh-test all passing; d 0 failed; core intent validate clean. (Amended 2026-10-21: the bump offer of the close cannot precede
      the close, because cicomplete refuses while a gate is open; its output is cited in the closing
      commit.)
    Evidence 2026-10-21 (G5): one ship at 18:51 (core 4.4.2, nsh-test 2.0.8, zero-docs 2.0.3), then a
      new shell: nsh-test 257 / 257; d 26 passed, 2 warnings, 0 failed; core intent validate all 332
      valid; the scratch close ran on the shipped core (see G4). clippy -D warnings and fmt --check
      are clean for the engine. The bump offer is cited in the closing commit.
- [x] G6 commits: Intent INT-292 and the plan Seal with its note under refs/notes/seals; pushed
      with the pre-push hook passing; main and refs/notes/seals reach GitHub; core intent trace
      reads every seal intact.
    Evidence 2026-10-21 (G6): commit 308bafd9 on main, fix: cicomplete refreshes the tools index after
      a bump and refuses on any disagreement (INT-292), trailers Intent: INT-292 and Seal:
      41aa3a1cab83b085, four files, pre-commit hook passed. The note under refs/notes/seals is the saved
      plan text (~/.cache/zero/int292-plan-engine.txt). The first attachment used git notes add -F,
      which changed the text: core intent trace read it BROKEN. Re-attached with -C of a hash-object
      blob, trace read it intact. git push origin main refs/notes/seals passed the pre-push hook: main
      9dfe1401..308bafd9, refs/notes/seals 77a585c2..6a609b58 and 6a609b58..b9618ae5; ls-remote equals
      local for both refs. core intent seals lists both INT-292 seals sealed (308bafd9, 13fa65e6).
      Left alone: ef539129 (INT-296) still reads BROKEN, probably the same -F cause. The AGENTS.md
      commit bcc4e6c7 went out in the same push.
- [x] G7 each gate carries evidence per INT-158.
    Evidence 2026-10-21 (G7): read after the ticks, G0 carries its findings, G1 the rulings approved by
      Christian, G2 to G6 an Evidence line each, and G7 this one.

## Non-goals
- No new crates. No sudo. Omarchy untouched.
- D4 and D5 of INT-102 are not reopened: cicomplete suggests, the human picks, and intent type never
  maps to a semver level.
- Gate counting and gate requirements belong to INT-212; the generator belongs to INT-291.
- AGENTS.md is not edited here; anything it needs is proposed to Christian.

## Dependencies
depends_on: 291 (its G1a decides whether documents state a version at all).

## Versions
- core 4.4.2 -> 4.4.3 (patch)
