---
id: 272
title: "core-protect v2 -- Single Source of Truth"
status: planned
date: 2026-05-05
tags: [core-protect, security, lock, integrity, reliability, v2]
---
The current core-protect has three sources of truth.
Three sources means three ways to lie.
The bar reads .core-locked.
fg commit reads something else.
core-protect status reads something else.
chattr reads the filesystem.
When you unlock and re-lock quickly -- they disagree.
The core shows locked when it is open.
The core shows open when it is locked.
That is not protection. That is a false sense of safety.
v2 has one source of truth.
Everything reads from it.
Nothing can disagree.
---
THE PROBLEM IN DETAIL
Current architecture:
  .core-locked sentinel file -- bar reads this
  chattr +i immutable flags -- filesystem protection
  core-protect status -- reads... what exactly?
  fg commit check -- reads... what exactly?
Race conditions observed:
  unlock-core removes sentinel but chattr flags persist
  lock-core sets sentinel but chattr fails silently
  fg commit says locked when bar says unlocked
  Status commands contradict each other
This is a security and reliability failure.
The presentation cannot have a bar showing wrong lock state.
---
v2 ARCHITECTURE
Single source of truth: /home/christian/0-core/runtime/.lock-state
  Format: JSON -- {"locked": true, "locked_at": timestamp, "reason": "session start"}
  Written atomically -- temp file + rename, never partial
  Read by: bar, fg commit, core-protect status, all lock checks
  chattr flags applied AFTER sentinel written
  chattr flags removed BEFORE sentinel removed
Lock sequence (atomic):
  1. Write .lock-state {"locked": true}  -- sentinel first
  2. Apply chattr +i to protected paths   -- then filesystem
  3. Verify chattr applied correctly      -- confirm
  4. If chattr fails: remove sentinel, report error
Unlock sequence (atomic):
  1. Remove chattr +i from all paths     -- filesystem first
  2. Verify chattr removed correctly     -- confirm
  3. If verified: remove .lock-state     -- then sentinel
  4. If chattr fails: sentinel stays, report error
Status check (always accurate):
  1. Read .lock-state (fast -- file read)
  2. Spot-check one chattr flag (verify not just sentinel)
  3. If mismatch: report INCONSISTENT, suggest repair
Repair command:
  core-protect repair
  Reads filesystem chattr state as ground truth
  Updates sentinel to match
  Reports what was fixed
---
EVERY CONSUMER READS THE SAME SOURCE
Bar: reads .lock-state -- never chattr directly
fg commit: reads .lock-state -- never chattr directly
core-protect status: reads .lock-state + spot-checks chattr
shell prompt: reads .lock-state
cistart: reads .lock-state before allowing intent start
One file. One format. One truth.
---
GATES
[ ] .lock-state JSON format defined and documented
[ ] lock-core writes .lock-state atomically before chattr
[ ] unlock-core removes chattr before removing .lock-state
[ ] core-protect status reads .lock-state + verifies chattr
[ ] core-protect repair detects and fixes desync
[ ] bar reads .lock-state instead of .core-locked
[ ] fg commit reads .lock-state
[ ] Lock/unlock cycle tested 10 times -- no desync observed
[ ] Rapid lock/unlock/lock tested -- no false state
[ ] Presentation scenario tested: lock, update, unlock, commit, relock
[ ] bar, status, and fg commit always agree
[ ] Health check verifies .lock-state integrity on every d
"The lock either holds or it does not.
There is no middle ground.
The forest knows which." 🌲
