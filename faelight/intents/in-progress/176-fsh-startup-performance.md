---
id: 176
date: 2026-07-18
type: improvement
title: "Fsh startup performance"
status: in-progress
tags: [fsh, faelight-shell, startup, performance]
---

## Vision
The fsh prompt renders immediately on every launch -- including the first one after a
reboot. Startup work that can happen a beat later happens a beat later, off the path
between the user and the prompt.

## The Problem
MEASURED 2026-07-20, and it is NOT what "startup performance" first suggests:
- fsh's own init is FINE: `faelight-shell -c 'exit'` = 8ms, steady over 10 runs. The
  core shell is not slow. Neither is Alacritty or the terminal -- both were ruled out
  by measuring the shell in isolation first.
- THE COST IS ONE BLOCKING CALL. refresh_health_if_stale() (main.rs, called at
  interactive startup before the banner renders) checks whether the doctor health
  event is older than boot. On the FIRST launch after a reboot it always is -- so it
  shells out to `core doctor run` SYNCHRONOUSLY and waits. That is the full 34-check
  health scan: MEASURED ~696ms. The prompt blocks on it.
- The felt pattern matches exactly: the first terminal after a reboot is slow (~700ms),
  every terminal after that is instant (the event is now post-boot, the check
  early-returns to the 8ms path).

So "fsh startup performance" is really "one synchronous health scan blocks the first
post-reboot prompt for ~700ms." Not the shell, not the terminal -- a single blocking
subprocess on the startup path.

## The Solution
Detach the stale-health refresh so it never blocks the prompt. The banner shows the
last-known health number; `core doctor run` happens in the BACKGROUND; the event
updates a beat later. Prompt renders immediately, health self-corrects async.

REVERSES A PRIOR DECISION, RECORDED HONESTLY: INT-124 deliberately made this refresh
BLOCKING so the splash "never shows a stale (pre-boot) health number." 176 says that
correctness is not worth ~700ms on every first-boot prompt. The trade: for ONE launch
after a reboot, the banner may show the pre-reboot health number, which then corrects
itself. A one-launch-stale health number is nearly invisible; a 700ms prompt block is
felt every reboot. If fresh-at-splash is later deemed essential, the honest answer is a
faster health check (INT-167 P0 territory), not blocking the prompt.

## Success Criteria
- [ ] The startup cost is MEASURED and its source named before any fix: `-c` init time,
      and the `core doctor run` cost that blocks refresh_health_if_stale when stale.
      (Done 2026-07-20: 8ms init, ~696ms doctor run, event-vs-boot mechanism confirmed.)
- [ ] refresh_health_if_stale no longer BLOCKS the prompt: the stale-path `core doctor
      run` is detached/backgrounded, not waited on. Prove it with the code diff.
- [ ] First-launch-after-reboot startup no longer pays the ~700ms: measured before/after
      on the DEPLOYED binary (a real reboot, or by forcing the stale path), prompt-ready
      time drops to the fast path. State both numbers.
- [ ] Nothing regresses: health STILL refreshes (just async) -- the doctor event updates
      after a stale first launch, proven by checking the event timestamp post-launch. The
      banner still renders. fsh-test stays green on the deployed binary.
- [ ] The INT-124 reversal is recorded in both intents (176 here + a note on INT-124), so
      the trade is not silently overridden.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"Measure before you optimize. The slow thing was not the shell -- it was one blocking
scan nobody timed. Off the path, not on it." 🌲
