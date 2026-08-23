---
id: 226
title: "work done without an intent leaves no evidentiary record and is invisible to every tool keyed on INT numbers"
status: planned
type: infrastructure
priority: high
date: 2026-08-22
tags: [ledger, release, versioning, evidence, int-102, int-211, int-212]
---

## Vision
The forest can tell you what changed, not only what was planned.

## The Problem
FOUND 2026-08-22, completing INT-221: `cicomplete` did not offer a version bump. The tool was
right. `engine/src/domains/intent/mod.rs:1342` detects touched tools with
`git log --name-only --grep=INT-<id>`, and not one commit in that session carried the number.
The retrospective said so plainly -- "~1 commits referencing INT-221".

That is the small version. The large version is the day it exposed.

THAT SESSION SHIPPED, WITH NO INTENT AND THEREFORE NO GATES:
  - `fsh -c` startup 297ms -> 13ms (a SQLite transaction around 285 alias writes)
  - a typed command 105ms -> 22ms (the next-command predictor deleted after measurement)
  - the command_execution lifecycle, dead for three days, restored and closed
  - every event the shell writes made traceable, plus a guard enforcing it
  - `trace`, `--version`, `--help`, and an `env` that shows the environment
  - the escape-handling fix, and a prefix assignment that reaches the child

None of it was declared. All of it was demonstrated. And the ledger -- the thing whose whole
purpose is recording what was VERIFIED rather than DECLARED -- holds no record of any of it.

## WHY THIS IS NOT "just write more intents"
Most of that work was FOUND, not planned. An intent written after the fact is a receipt, not a
charter, and INT-217 already recorded what a stub written to satisfy a process looks like: it
"would have passed cicomplete unchallenged on a status line". Manufacturing intents to feed a
tool is the failure mode, not the fix.

⚠️ AND THE OPPOSITE IS ALSO WRONG. Decision 6 (INT-102) built a release PRECONDITION check on the
ledger's inclusion rule -- "stable + DEMONSTRATED + not mid-flight". A check that reads only the
ledger will report NO BLOCKERS while a week of unrecorded change sits underneath it. The check is
not lying; it is answering a narrower question than the one being asked.

## WHAT IS ALREADY TRUE (measured 2026-08-22, before any work)
- `faelight-release/src/changelog.rs:1` -- "reads git log + intent ledger". Release NOTES already
  see both. `Commit::parse` extracts an optional `intent_id`; `find_shipped_intents` reads
  `complete/` for files added since the last tag. So the notes are NOT the blind spot.
- The BLIND consumers are the ones keyed on `INT-<id>`: the version detector at
  `intent/mod.rs:1342`, and anything else that greps commit subjects.
- `is_noise()` exists on `Commit` (`changelog.rs:52`) and its behaviour on an intentless commit is
  UNMEASURED. G1 measures it rather than assuming.

## Success Criteria
- [ ] G1 MEASURE BEFORE RULING: report, from real history since the last tag, how many commits
      carry an `INT-<id>` and how many do not; and for the intentless ones, whether `is_noise()`
      keeps or drops them from generated notes. A number, not an impression
- [ ] G2 THE RULING, RECORDED WITH WHAT IT GIVES UP: what IS the record for demonstrated work that
      had no charter? Candidate shapes -- a retroactive intent (rejected unless argued: INT-217's
      stub lesson), a decisions/ entry, a distinct ledger state such as `recorded/`, or the commit
      itself as the record with tooling taught to read it. One choice, its cost named
- [ ] G3 THE DETECTOR STOPS BEING SILENTLY BLIND: when `--grep=INT-<id>` matches nothing,
      `cicomplete` must SAY SO rather than skip the prompt. A tool that finds nothing and stays
      quiet is indistinguishable from a tool that found nothing to do -- INT-192's complaint
- [ ] G4 THE PRECONDITION CHECK REPORTS ITS OWN SCOPE: whatever Decision 6's check becomes, it must
      state what it did NOT examine. "No blockers found in the ledger" is honest; "no blockers" is
      not, while unrecorded work exists
- [ ] G5 DEMONSTRATED ON THE 2026-08-22 SESSION: run the finished tooling against that exact range
      and show it surfacing the twelve-plus intentless commits. Real history, not a fixture
- [ ] G6 CANNOT REGRESS SILENTLY: a test covers the intentless-commit path, red first
- [ ] G7 each gate carries evidence per INT-158

## Non-goals
- Deriving a version from anything. D5 and Decision 6 both rule the human picks the digit, and
  "weight of work" was proposed and rejected on 2026-08-22 for the same reason as type and count:
  a one-line change to an output contract outranks three thousand lines of internal repair.
- Blocking commits that lack an intent number. Opportunistic fixes are how most of that session's
  value was found; the goal is that they leave a trace, not that they stop happening.
