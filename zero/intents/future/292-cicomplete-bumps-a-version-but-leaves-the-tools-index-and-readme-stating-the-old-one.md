---
id: 292
date: 2026-10-17
type: future
title: "cicomplete bumps a version but leaves the tools index and README stating the old one"
status: planned
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
- [ ] G0 recon, read-only. Evidence is the output, kept in ~/.cache/zero:
      a. The cicomplete bump code today: file:line of the detection, every file it writes, where the
         receipt is written.
      b. Every document that states a tool version (index, per-tool READMEs, registry, docs/, CHANGELOGs,
         census) against its Cargo.toml, as a table.
      c. Which of those INT-291 rules away.
      d. What d and the doors check about versions today.
- [ ] G1 rulings after INT-291 G1a, each approved by Christian: where the refresh lives (cicomplete,
      ship or the generator), and exactly what the close refuses on.
- [ ] G2 red. A close on a scratch clone of the repo, with a scratch intent that bumps a tool, leaves a
      stated version behind. The case fails today.
- [ ] G3 fix. G2 green. The smallest change per ruling; D4 and D5 untouched; no new crates.
- [ ] G4 live. The same scratch close on the deployed cicomplete leaves every stated version equal to
      Cargo.toml, and a deliberately stale document makes it refuse.
- [ ] G5 doors: one ship at the end where code changed; the deployed tools tested in a new shell;
      nsh-test all passing; d 0 failed; core intent validate clean. The close offers its bumps.
- [ ] G6 commits: Intent INT-292 and the plan Seal with its note under refs/notes/seals; pushed
      with the pre-push hook passing; main and refs/notes/seals reach GitHub; core intent trace
      reads every seal intact.
- [ ] G7 each gate carries evidence per INT-158.

## Non-goals
- No new crates. No sudo. Omarchy untouched.
- D4 and D5 of INT-102 are not reopened: cicomplete suggests, the human picks, and intent type never
  maps to a semver level.
- Gate counting and gate requirements belong to INT-212; the generator belongs to INT-291.
- AGENTS.md is not edited here; anything it needs is proposed to Christian.

## Dependencies
depends_on: 291 (its G1a decides whether documents state a version at all).
