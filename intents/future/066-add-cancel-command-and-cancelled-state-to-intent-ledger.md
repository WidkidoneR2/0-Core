---
id: 066
date: 2026-06-16
type: feature
title: "Add cancel command and cancelled state to intent ledger"
status: planned
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

- [ ] `core intent cancel <id> --reason "..."` transitions the intent to cancelled
- [ ] cancel refuses without --reason (the honest record is mandatory)
- [ ] cancelled intent moves out of in-progress/ into intents/cancelled/
- [ ] reason, timestamp, and approval are written into the intent file
- [ ] cancelled intents are excluded from velocity and burndown
- [ ] cancelled intents still appear in show, and list --cancelled filters them
- [ ] doctor active-intent count excludes cancelled
- [ ] a dangling-dependency check flags intents that depend on a cancelled one

## Gate Check
⬜ Not started

---

*"The forest grows with intention."* 🌲
