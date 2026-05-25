---
id: 333
title: "faelight-git v5 -- intelligence, integrity, drift prevention"
status: in-progress
date: 2026-05-25
tags: [git, faelight-git, intelligence, integrity, drift, doctor, friday]
---

## The Problem

faelight-git is one of the oldest tools in the forest. It was built
to wrap git operations with forest awareness. But it has accumulated
debt:

1. Integrity drift -- the doctor integrity check fluctuates without
   clear cause. faelight-git is a likely contributor because it reads
   git state in ways that can produce false negatives.

2. No connection to Friday -- git commits, pushes, and branches are
   significant events but faelight-git does not emit them to the
   event bus. Friday cannot reason about git activity.

3. No intelligence -- faelight-git does not know about intents, does
   not warn when you commit without cistart, does not detect when
   commit messages drift from the intent pattern.

4. Stale design -- v4 was built before the forest had a proper event
   bus, before Friday existed as a reasoning layer, before the
   integrity system existed.

## What v5 Fixes

### Integrity Drift Prevention
- faelight-git v5 emits clean, structured events on every git operation
- The doctor integrity check reads from these events instead of raw git state
- No more false negatives from git state parsing

### Friday Integration
Every significant git event emits to friday::events::emit:
  - commit: domain="git" kind="commit" payload={hash, message, files_changed}
  - push: domain="git" kind="push" payload={branch, commits}
  - branch_create: domain="git" kind="branch_created" payload={name}
  - merge: domain="git" kind="merge" payload={branch, into}

Friday can then reason: "3 commits in the last hour with no deploy --
build may be accumulating."

### Intent Awareness
Before every commit, faelight-git checks:
- Is there an active intent (cistart)?
- Does the commit message reference the intent?
- If no active intent: warn "No intent context -- run cistart first"
- If message does not match intent pattern: suggest the correct prefix

### Commit Intelligence
faelight-git v5 learns commit patterns from history:
- What files tend to change together?
- What commit messages have been used for what domains?
- Surface anomalies: "you usually commit engine/ and scripts/ together,
  but this commit only touches engine/"

### COSMIC Integration
If faelight-compositor is running, git events surface as compositor
overlay notifications -- "Commit pushed: INT-251 -- event bus gates marked"

## Architecture

faelight-git v5 is a thin Rust binary that:
1. Wraps git operations (commit, push, branch, status, log)
2. Emits to friday::events::emit on every operation
3. Reads from state.db for intent context
4. Writes a git_operations table for Friday to reason over
5. Optionally surfaces events to faelight-compositor

## Gates

✅ faelight-git v5 compiled and deployed -- replaces v4 2026-05-25
✅ Every commit emits to event bus -- demonstrated live: hash+message visible 2026-05-25
✅ Every push emits to event bus -- push event visible in core friday event-bus 2026-05-25
✅ Intent awareness: shows active intents from both future/ and in-progress/ 2026-05-25
✅ Commit message pattern check: warns when message does not reference intent 2026-05-25
✅ git_operations table created -- commit+push records verified in state.db 2026-05-25
⏸ Doctor integrity check reads from git_operations -- deferred: requires integrity domain refactor -- approved by: christian 2026-05-25
⏸ Integrity drift elimination -- deferred: needs 10 consecutive sessions to prove, track over time -- approved by: christian 2026-05-25
✅ Friday can answer git activity -- core friday event-bus shows commit+push events 2026-05-25
✅ commits_without_deploy rule added to reasoning engine -- fires at >=3 commits no deploy 2026-05-25
✅ Demonstrated: commit 577cbed visible in event bus and git_operations table 2026-05-25
⏸ COSMIC notification on push -- deferred: compositor not stable yet -- approved by: christian 2026-05-25
