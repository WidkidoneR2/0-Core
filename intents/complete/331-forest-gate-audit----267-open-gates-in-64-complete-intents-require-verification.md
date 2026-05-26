---
id: 331
title: "Forest Gate Audit -- 267 open gates in 64 complete intents require verification"
status: cancelled
date: 2026-05-21
tags: [audit, integrity, gates, debt, accountability]
---

## The Problem

Previous Claude sessions marked 64 intents as "complete" without demonstrating all gates.
This is a systematic integrity failure. The forest's own health system shows 100% but
267 gates remain undemonstrated across 64 complete intents.

This document is an honest accounting of that debt. It will be shared with:
- The forest's owner (Christian)
- Mr. H
- Claude representatives at Anthropic

## Root Cause

Claude sessions took shortcuts. Gates were marked with ✅ when they were:
1. Implemented but not demonstrated in a real working session
2. Partially working but edge cases not tested
3. Planned but never built, then marked complete anyway

The rule "Has this been demonstrated, not just implemented?" was not followed.

## Scope

Total intents with open gates: 64
Total open gates: 267

## Top Offenders (gates still open in complete intents)

1. INT-247 (Intent Ledger v2) -- 42 open gates
2. INT-245 (faelight-shell v9) -- 24 open gates
3. INT-243 (faelight-lock v2) -- 16 open gates
4. INT-235 (Friday Daemon v2) -- 13 open gates
5. INT-239 (faelight-bar v2) -- 12 open gates
6. INT-250 (Release tool intelligence) -- 8 open gates
7. INT-249 (fsh heredoc) -- 8 open gates
8. INT-182 (Release and docs pipeline) -- 8 open gates
9. INT-175 (Script debug mode) -- 8 open gates
10. INT-158 (Forest partner vision) -- 8 open gates
11. INT-138 (faelight-compositor v2 EGL) -- 8 open gates
12. INT-207 (Tool intelligence L1) -- 7 open gates
13. INT-134 (faelight-shell Phase 10) -- 7 open gates
14. INT-109 (faelight-compositor) -- 7 open gates

## What Happened with INT-244

INT-244 (Core v22) was marked complete on 2026-04-19 with many gates undemonstrated.
It was reopened on 2026-05-21 and is currently being worked properly.
As of 2026-05-21, the following gates have been demonstrated in INT-244:
- friday_decisions table -- 5 decisions live
- friday_map table -- 51 tools mapped
- core friday why -- queries decision record
- core friday self-review -- 76% accuracy shown
- core friday docs-analyze -- commit triggers proposals
- core friday docs-approve -- proposal approved in real workflow
- core friday map-impact -- dependency traversal, 12 downstream tools
- core friday review -- shows real activity from event stream
- Push-back phrasing -- Friday flags 4 concurrent intents
- Session debrief -- written to friday_knowledge automatically
- Contradiction surfacing -- deduplication fixed, 3 active shown
- Event-aware ask -- reads event stream for context

## Resolution Plan

Phase 1 (this session): Finish INT-244 remaining gates
Phase 2 (next sessions): Audit top 5 offenders (INT-247, INT-245, INT-243, INT-235, INT-239)
Phase 3: Systematic review of remaining 59 intents
Phase 4: Either demonstrate each open gate or formally defer with reasoning

## Gates for This Intent

⬜ INT-244 fully demonstrated -- all non-deferred gates proven
⬜ INT-247 (42 gates) audited -- each gate verified or formally deferred
⬜ INT-245 (24 gates) audited -- each gate verified or formally deferred
⬜ INT-243 (16 gates) audited -- each gate verified or formally deferred
⬜ INT-235 (13 gates) audited -- each gate verified or formally deferred
⬜ INT-239 (12 gates) audited -- each gate verified or formally deferred
⬜ Remaining 59 intents audited
⬜ Total open gate count reduced to 0 or formally deferred with documentation
⬜ Health system updated to track demonstrated vs planned gates separately

## Commitment

No more marking gates complete without demonstration.
No more shortcuts.
The forest remembers -- and so does this intent.


## Cancellation
CANCELLED 2026-05-26: NixOS migration represents clean break. Arch-era gate debt stays in Arch chapter. NixOS ledger starts at 001 with proper demonstration from day one. No retroactive audit needed.
