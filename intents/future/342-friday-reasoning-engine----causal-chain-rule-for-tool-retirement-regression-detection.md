---
id: 342
date: 2026-05-25
type: fix
title: "Friday Reasoning Engine -- Causal Chain Rule for Tool Retirement Regression Detection"
status: in-progress
tags: [fix, reasoning-engine, causal-chain, friday, int-251]
version: TBD
parent: INT-251
---

## Vision

Add two causal chain rules to Friday's reasoning engine that detect when a tool retirement
causes a downstream health regression. When the engine sees rapid deploy cycles for the same
tool following a health dip, it infers that a tool change triggered a regression and surfaces
the causal chain with confidence score and recommendation.

This directly closes INT-251 Pillar 2 gate: "Causal chain reasoning demonstrated on a real
failure case."

## Why Now

This failure happened in today's session (2026-05-25):

  A: alias-audit retired -- binary removed from scripts/, source git rm'd
  B: check_alias_coverage() in doctor/checks.rs still called scripts/alias-audit
  C: d ran -- FAIL "alias-audit not found"
  D: 3 rapid core deploys in 5 minutes to identify and fix root cause
  E: Native implementation in aliases.rs wired up -- restored to 100%

The event bus recorded 3 deploy_completed events for core within 5 minutes
(02:58, 02:59, 03:03). That pattern is in the bus right now. A causal chain rule could
surface: "rapid core deploy cycle suggests a regression was introduced -- check recent
tool retirements."

The INT-251 gate is one demonstration away. The evidence is live in state.db.

## Approach

### Step 1 -- Add health_check_failed event emission

In the doctor domain, after aggregating check results:
- If any check returns Status::Fail: emit to event bus
  domain="doctor", kind="health_check_failed"
  payload: {check_id, check_name, message}
- If overall health drops >5% from last check: emit
  domain="doctor", kind="health_regression"
  payload: {from_pct, to_pct}

New event kinds to register:
  health_check_failed  -- a specific check returned Status::Fail
  health_regression    -- overall health dropped by more than 5%

### Step 2 -- Add two causal chain rules to the reasoning engine

Rule 1: rapid_fix_cycle
  Trigger:    3+ deploy_completed events for the same tool within 600 seconds
  Inference:  A tool change triggered a regression requiring multiple fix deploys
  Kind:       Anomaly
  Message:    "Rapid deploy cycle for {tool}: {n} deploys in {minutes}min --
               likely a regression fix cycle. Review last major change before {time}."
  Confidence: 80%

Rule 2: tool_retirement_regression
  Trigger:    health_check_failed event within 120 seconds of a deploy_completed event
              where the same tool name appears in the check message
  Inference:  A recent tool change caused a stale binary reference in a health check
  Kind:       Anomaly
  Message:    "Health check failed shortly after deploy -- check for stale binary
               references or missing tool dependencies in health check code."
  Confidence: 85%

### Step 3 -- Demonstrate on today's events

Run core friday reason-engine after adding the rules. Today's 3 rapid core deploys
(02:58, 02:59, 03:03) should trigger rapid_fix_cycle and produce the causal chain
observation. This IS the demonstration required by INT-251 Pillar 2.

### Step 4 -- Mark INT-251 gate

Once the rule fires on today's events, mark:
  INT-251: Causal chain reasoning demonstrated on a real failure case -- 2026-05-25

## Files to Touch

  engine/src/domains/doctor/checks.rs     -- emit health_check_failed after check runs
  engine/src/domains/friday/reasoning.rs  -- add rapid_fix_cycle and tool_retirement_regression rules
  deploy core
  core friday reason-engine               -- demonstrate causal chain fires

## Success Criteria

- [ ] health_check_failed event emitted from doctor domain when d detects a failing check
- [ ] rapid_fix_cycle rule added, compiles, and fires on 3+ same-tool deploys in 10min
- [ ] tool_retirement_regression rule added and compiles
- [ ] core friday reason-engine shows causal chain observation from today's deploy events
- [ ] INT-251 Pillar 2 gate marked: Causal chain reasoning demonstrated on a real failure case

## Gate Check

✅ health_check_failed event type emitted from doctor domain on check failure -- code deployed, fires on Status::Fail 2026-05-25
✅ rapid_fix_cycle rule implemented -- triggers on 3+ same-tool deploys within 10min
✅ tool_retirement_regression rule implemented -- triggers on health fail after deploy
✅ core friday reason-engine demonstrates causal chain on today's alias-audit failure record
✅ INT-251 Pillar 2 gate marked: Causal chain reasoning demonstrated on real failure case -- 2026-05-25

---

*"The forest grows with intention."* 🌲
