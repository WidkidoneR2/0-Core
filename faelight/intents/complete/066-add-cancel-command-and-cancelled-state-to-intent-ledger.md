---
id: 066
date: 2026-06-16
type: feature
title: "Add cancel command and cancelled state to intent ledger"
status: complete
tags: [intent-ledger, cancel, cli, rust, faelight]
version: TBD
---

## Vision

The ledger needs a first-class way to close an intent that was overtaken by
reality -- superseded by a better tool, or dropped because the need evaporated.
Today the only verbs are `complete` (files it under done and counts it toward
velocity and burndown, a stat lie for something that never shipped), `defer`,
and `override` (both per-gate, not whole-intent). The honest gap: there is no
cancelled state.

  core intent cancel <id> --reason "..."   -- transition intent to cancelled
  reason is mandatory                       -- the record must say WHY
  cancelled intents leave in-progress/      -- they are not active work
  excluded from velocity and burndown       -- a drop is not a completion
  still queryable                           -- show works, list has a filter

A cancelled intent is not a failure and not a completion. It is a decision,
recorded with its reason.

## Why Now

INT-047 (faelight-menu v2 launcher) was just dropped: the faelight-menu crate
was retired and replaced by faelight-logout, and a launcher is not used. There
was no honest verb to close it. `complete` would have lied to velocity, so it
was removed by hand with git rm, which loses the structured record of why. The
next overtaken intent should close in one honest command instead.

## Approach

- New `cancel` subcommand in the engine intent domain.
- New status value cancelled and an intents/cancelled/ directory mirroring
  intents/complete/. cancel moves the file there and rewrites frontmatter
  status to cancelled.
- --reason is mandatory. Record reason, timestamp, and approved-by, matching
  the shape defer already uses.
- Velocity and burndown queries exclude cancelled (no false completion).
- list and show are aware of intents/cancelled/. Add a list --cancelled filter.
  The doctor active-intent count excludes cancelled.
- If a cancelled intent is referenced by another intent Depends On, surface it
  (a dangling-dependency check) rather than leaving a ghost pointer.

## Success Criteria

- [x] `core intent cancel <id> --reason "..."` transitions the intent to cancelled
- [x] cancel refuses without --reason (the honest record is mandatory)
- [x] cancelled intent moves out of in-progress/ into intents/cancelled/
- [x] reason, timestamp, and approval are written into the intent file
- [x] cancelled intents are excluded from velocity and burndown
- [x] cancelled intents still appear in show, and list --cancelled filters them
- [x] doctor active-intent count excludes cancelled
- [x] a dangling-dependency check flags intents that depend on a cancelled one

## Gate Check
✅ All 8 criteria demonstrated 2026-06-17 (faelight-forest debug build, throwaway intents) -- approved by: christian
- C1-C4: cancel verb + cancelled state + move into intents/cancelled/ + reason/date/approval stamp; refuses without --reason (clap) and on empty --reason (domain guard).
- C5: burndown subtracts cancelled from remaining and shows a Cancelled column; velocity counts only complete, so cancelled never appears there.
- C6: show renders cancelled intents; list hides them by default, list --cancelled lists them under a CANCELLED header.
- C7: no active/in-progress count includes cancelled -- every count filters by status, and stats lists Cancelled on its own line.
- C8: cancel scans for active intents whose depends_on names the cancelled id and warns (demonstrated INT-998 depends_on 999).

---

*"The forest grows with intention."* 🌲
