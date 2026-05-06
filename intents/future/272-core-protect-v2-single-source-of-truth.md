---
id: 272
title: "core-protect v2 -- Single Source of Truth"
status: in-progress
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
---
ENHANCED MESSAGING
Every lock state message tells you what to do next:
  Blocked by lock:
    "Core is locked. Run unlock-core to proceed.
     Lock again with lock-core when finished."
  Not just a wall -- a door with instructions.
Bar left zone enhanced:
  Locked:   green   "LOCKED"   -- calm, protected
  Unlocked: amber   "OPEN"     -- working, intentional
  Unlocked + uncommitted changes: red "OPEN*" -- urgent, act soon
  The * means: you have changes and the core is open. Finish and relock.
---
GRACE PERIOD WARNING
If core has been unlocked for more than 10 minutes:
  Friday surfaces once: "Core has been unlocked for 10 minutes.
  Uncommitted changes detected. Commit and relock when ready."
  Once only. Not every minute. Not nagging.
  Friday respects your flow. It just makes sure you know.
If core has been unlocked for more than 30 minutes with no commits:
  Bar changes: red "OPEN!" -- escalated urgency
  One faelight-notify notification: "Core still open -- 30 minutes"
  After that: silence. You know. The choice is yours.
---
CORE-PROTECT HISTORY
Every lock and unlock recorded in state.db:
  Table: core_protect_log
    action: TEXT (locked / unlocked)
    timestamp: INTEGER
    duration_unlocked: INTEGER (seconds, on relock)
    had_uncommitted: BOOLEAN (were there changes when unlocked?)
    triggered_by: TEXT (manual / cistart / cicomplete)
Commands:
  core-protect log          -- full lock/unlock history
  core-protect log --today  -- today only
  core-protect last         -- last lock/unlock with duration
Example output:
  core-protect log --today
  09:14  locked    (session start)
  11:32  unlocked  (manual)
  11:38  locked    (manual -- 6 minutes open, 2 commits)
  14:15  unlocked  (manual)
  14:47  locked    (manual -- 32 minutes open, 5 commits)  ⚠️
The ⚠️ means: open for more than 20 minutes.
Not a failure. Just a flag for review.
The history does not judge. It records.
"You think you locked it. The log knows." 🌲

"The lock either holds or it does not.
There is no middle ground.
The forest knows which." 🌲
