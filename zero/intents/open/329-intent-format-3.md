---
id: 329
date: 2026-11-19
type: future
title: "Intent format 3: every intent states its Limits, and the tools enforce it"
status: in-progress
tags: [core, intent, ledger, format]
format: 2
---

## Vision
An intent says how far the work goes before anyone reads its gates. A file
that says `format: 3` gets every format 2 rule plus a required `## Limits`
section, and `core intent validate` (and so `d` and doors) refuses a format
3 file whose Limits is missing or half filled. `core intent new` writes a
format 3 skeleton that already has the five Limits labels. Format 2 and
format 1 files behave exactly as they do today.

## The Problem
1. gates.rs `is_format_2` checks for `format: 2` exactly. A file that says
   `format: 3` today is treated as format 1: `check_completion` stops
   requiring `evidence:`, so a ticked gate with an empty evidence line
   would close. This must be fixed before anyone writes `format: 3`.
2. intent/mod.rs `validate_issues` uses `== Some("2")`, so format 3 shape
   problems become notes, not issues. Nothing knows about `## Limits`.
3. `create_in` writes `format: 2`, no Limits, and the INT-158 comment that
   says evidence goes in an HTML comment and "nothing enforces this",
   which has been wrong since format 2 (check_completion refuses an empty
   `evidence:`).
4. zero/intents/README.md describes in-progress/, complete/, future/ and
   arch-era/, none of which exist (folders.rs LIFECYCLE = ["open"]). Format
   2 fields and closed.tsv are not in docs/CONVENTIONS.md.
Read against main 8c6afbd4.

## The Solution
One shared format reader, `format_version(fm) -> u32`, used by gates.rs and
validate_issues. Format 2 or higher gets the format 2 rules; a format above
3 is an issue ("unknown format 4"). A new `limits_issues(content)` runs only
for format 3 or more and checks the `## Limits` heading and that each of In
scope, Out of scope, How deep, Stop and ask, Done means starts a bullet with
text after the colon. `create_in` writes `format: 3`, a Limits skeleton
between The Solution and Success Criteria, and drops the INT-158 comment.
The README is rewritten for the real folders, format 3, and the history
path after close (closed.tsv, `core intent show <id>`, `git show`).

## Limits
- In scope: the format number check, the Limits validator, the new-intent
  skeleton, the README and a short CONVENTIONS.md paragraph.
- Out of scope: migrating existing intents (an intent moves to format 3
  when its work starts, not before); the INT-321 open/ sweep; the double
  Versions sections in 316 and 317 (INT-321); `core intent list --json`
  and the Quickshell ledger panel (next intents); zero-doctor, nsh-test,
  doors and the `it` panel (they need no change).
- How deep: gates.rs, intent/mod.rs, bug.rs (test only), the README and
  CONVENTIONS.md. About 8 jobs, one production file each. No new
  dependencies.
- Stop and ask: a job that needs a second production file, a change to how
  format 1 or format 2 files validate, or anything in zero-doctor. Land
  what passes and file a follow-up instead of improvising.
- Done means: G1 to G5 ticked with evidence.
- Must not change: format 2 and format 1 validation results for every file
  in open/ today, the nsh prompt path, live state.db, anything outside the
  repo, root.

## Success Criteria
- [x] G1 format 3 keeps the evidence rule (gates.rs). A format 3 file with
      a ticked gate and an empty `evidence:` is refused (MissingEvidence).
      Format 2 still is; format 1 is unchanged.
      ask: a format 3 ticked gate with empty evidence is refused
      ruling:
      evidence: cargo test -p core: a format 3 ticked gate with empty evidence returned MissingEvidence; format 2 still refused, format 1 unchanged
- [x] G2 Limits is required for format 3 (intent/mod.rs). A format 3 file
      with no `## Limits` is an issue; with all five labels filled it is
      clean; one missing "Stop and ask:" names that label; a format 2 file
      with no Limits is still clean; `format: 4` is an issue.
      ask: validate refuses a format 3 file without full Limits, and only that
      ruling:
      evidence: cargo test -p core and scratch HOME validate: format 3 without Limits and format 4 were issues; a missing Stop and ask was named; format 2 without Limits stayed clean
- [x] G3 a new intent starts in format 3 (create_in). `core intent new`
      writes `format: 3`, the Limits skeleton before Success Criteria, and
      no INT-158 comment (bug.rs create_in test updated).
      ask: core intent new writes a format 3 skeleton with the five labels
      ruling:
      evidence: scratch HOME core intent new wrote format: 3 with the five Limits labels before Success Criteria and no INT-158 comment; bug.rs create_in test passed
- [x] G4 the README tells the truth (docs only). zero/intents/README.md
      names the real folders, the format 3 template, and how to read a
      closed intent; CONVENTIONS.md has a short format 3 paragraph.
      ask: the README matches the folders and explains history after close
      ruling:
      evidence: zero/intents/README.md sections Folders, Format 2, Format 3, Template, Reading a closed intent; CONVENTIONS.md has Intent format 3 (INT-329)
- [x] G5 doors, nsh-test and d green on the laptop, and `core intent
      validate` gives the same results for every file in open/ as on
      8c6afbd4.
      ask: doors, nsh-test and d green; no existing intent changes verdict
      ruling:
      evidence: doors and d green after the INT-329 merge; core intent validate on main vs int-329 differed only in the intent count (94 to 95)

## Job order
- Wave 1, side by side: red tests for G1 (gates.rs tests), G2 (mod.rs
  tests), G3 (bug.rs create_in test).
- Wave 2: G1 fix in gates.rs, side by side with the G4 docs job.
- Wave 3, in order (same file): format_version and the format check in
  validate_issues, then limits_issues, then create_in.
- G5 on the laptop.

## Versions
- core minor (format 3 accepted and written)
