---
id: 273
date: 2026-10-03
type: future
title: "cicomplete shows evidence at the question: diffstat, commit subjects, tier legend"
status: planned
tags: [cicomplete, versions, evidence]
---

## Vision
The close asks a human for each crate's level, and the human answers better with the work in front
of them. Today the question shows a crate name and its current version, nothing else. After this
intent each question shows, on stderr above the prompt, what this intent did to that crate: its
commits, the files and lines they changed, and a one-line reminder of what the three digits
promise. The close still proposes nothing and the human still types the digit (INT-102 D4).

## Why Now
- INT-271 (complete 2026-10-03) made the close ask honestly: no default, typos asked again,
  questions on stderr. It left the question bare.
- Christian wants the level judged with the size of the work in view: lines of code and files
  involved. INT-102 D5, Decision 6 and the 2026-08-22 ruling found that size cannot choose the
  digit (INT-271's engine change was +391/-191 and was skipped; Phase 2 changed about 80 lines and
  changed a promise). Shown beside the question, size informs the choice without making it.

## What
1. Above each question, on stderr: the number of this intent's commits that touched the crate and
   their subjects (capped, with the count of any not shown); the files changed under the crate and
   the lines added and removed; one legend line: major breaks a promise, minor adds one, patch
   fixes one or touches nothing promised.
2. A note that only commits whose message names INT-<id> are counted, since that is the join key.
3. The prompt itself is unchanged: no default, no proposed level.

## Approach
- Phase 0 reads decide_versions, ask_level and touched_crates as INT-271 left them, and how
  git log --numstat reports a crate path including renames across CRATE_PARENTS_HISTORY.
- The evidence is computed from git at question time and never stored.
- Red first: the bare question captured on the deployed core before any edit.
- Tested on a scratch HOME (the INT-271 method) with commits of known size, so every number shown
  can be checked against git by hand.
- No Omarchy code, no sudo. AGENTS.md is not edited.

## Phases
Phase 0 -- recon, reading only: the question path with file:line; what git log --grep=INT-<id>
  --numstat returns for a crate across its historical parents.
Phase 1 -- the evidence block above each question.
Phase 2 -- ship; doors.

## Gates
- [ ] Phase 0: decide_versions, ask_level and touched_crates recorded with file:line, and the numstat behaviour across a crate's historical parents recorded
- [ ] Red baseline on deployed core: on a scratch HOME the question shows only the crate and its version, captured verbatim
- [ ] After, on a scratch HOME with commits of known size: each question is preceded on stderr by the commit count and subjects, the files and added/removed lines for that crate, the legend line and the join-key note, and every number matches git log --numstat run by hand
- [ ] No level is proposed: the prompt reads as INT-271 left it, and the empty-answer and typo checks still ask again under a pty
- [ ] The evidence goes to stderr: with stdout redirected the evidence and questions stay on screen and the file holds status lines only
- [ ] Doors per commit: the result on PATH after ship, nsh-test all passing, d 0 failed, plus that commit's own door above

## Notes
- Christian raised on 2026-10-03 that the evidence should be based on LOC and files involved. This
  intent shows both; it does not turn them into a proposed level. Making size propose the digit
  would reverse INT-102 D5 and the 2026-08-22 ruling, and needs its own decision record first.
- A later possibility, not in scope: nsh-test cases tagged with their NSH-COMPATIBILITY tier, so the
  close could also show which promises an intent's commits changed.
- zero-core is outside the crates the close tracks, because how to version it is undecided (it is
  used by every tool and directory). This intent does not change that.
- Related: INT-271 (the close this builds on), INT-102 (version architecture).

## The Rule
"Show the work beside the question. The digit stays a judgement, made with the evidence in view."
