---
id: 182
date: 2026-07-20
type: improvement
title: "fsh startup residual: git-counts + faelight-export still block the banner (~555ms)"
status: planned
tags: [fsh, faelight-shell, startup, performance]
---

## Vision
The fsh prompt renders as close to instant as honestly possible. INT-176 removed the big
~700ms blocking health scan; this finishes the job on the residual startup work that still
runs synchronously before the banner.

## The Problem
MEASURED 2026-07-20 (during INT-176): after detaching the health scan, first-launch startup
dropped from ~1260ms to ~555ms. That ~555ms is NOT zero and NOT the shell core (fsh -c init is
8ms). It is OTHER interactive-startup work that still runs synchronously before the prompt:
- The banner's git counts (commits this week, "since last session" -- these shell out to git).
- `faelight-export` at main.rs:725, called with `.output()` (spawn AND WAIT) -- the same
  blocking pattern 176 fixed for the health scan, still present here.
Both run on the path between the human and the prompt. 176 named this residual explicitly as
out-of-scope-then, on-the-table-later. This is later.

## The Solution
Same discipline as 176: MEASURE FIRST (attribute the ~555ms across git-counts vs faelight-export
vs prompt rendering vs everything else -- do not guess), then move whatever is both costly and
non-essential OFF the critical path. Candidates:
- faelight-export main.rs:725: `.output()` -> detached `.spawn()` if its result is not needed
  before the prompt (same fix as 176's health scan). Verify it is not needed synchronously first.
- Banner git-counts: cache the last-known counts and refresh async, or compute them in the
  background -- the banner can show last-known and self-correct, exactly like 176 did for health.
Keep the numbers honest: state before/after, release-vs-release, measured with a standalone bash
script (fsh mangles its own $(...) probes -- see 176's measurement lesson).

## Success Criteria
- [ ] The ~555ms is MEASURED and attributed across its sources (git-counts, faelight-export,
      prompt render, other) before any fix -- release binary, standalone bash script, forced
      cold each run. State the breakdown.
- [ ] The dominant non-essential blocking call(s) are moved off the startup path (detach/spawn or
      cache-and-refresh-async), proven with the code diff.
- [ ] Before/after startup measured release-vs-release; state both numbers. No regression to the
      health-freshness behaviour 176 established.
- [ ] Nothing essential is lost: whatever was backgrounded still happens (proven), banner still
      renders correctly.
- [ ] fsh still boots, deploys; fsh-test green on the deployed binary.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"176 proved the slow thing was one unmeasured blocking call. This finishes the sweep: measure the
rest, move what can wait off the path the human is standing on." 🌲
