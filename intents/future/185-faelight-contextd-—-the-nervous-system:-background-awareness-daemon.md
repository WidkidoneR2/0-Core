---
id: 185
date: 2026-04-03
type: arch
title: "faelight-contextd — The Nervous System: Background Awareness Daemon"
status: in-progress
tags: [contextd, daemon, events, awareness, nervous-system, v14, architecture]
priority: critical
depends_on: [156, 184]
---

## The Problem
core is a transactional executor: parse -> dispatch -> exit.
No persistence of awareness. No continuous observation.
The system does things but does not experience them.
This is the single biggest gap between what exists and Jarvis.

## The Architecture
Current:  command -> result -> exit
New:      command -> result -> event -> awareness -> insight -> surface

Three components:
1. Event Layer (core modification) — emit structured events after every dispatch
2. faelight-contextd (background daemon) — observe, detect patterns, write insights
3. Intervention Hook (fsh) — surface insights at the right moment

## Event Layer
Events written to forest_events table in state.db (non-blocking):
CommandSucceeded, CommandFailed, IntentStarted, IntentCompleted, HealthChanged, DeployExecuted

## faelight-contextd Signals
- 4+ consecutive failures -> failure loop detected
- deploy with no d after -> health unchecked
- rapid context switching -> focus fragmentation
- intent started, no commits in 2h -> possible stall
- tool unused vs expected_usage in registry -> drift

## Intervention Gate (non-negotiable)
fn should_intervene(insight) -> bool:
  importance > 0.7 AND confidence > 0.6 AND cooldown_not_active AND context_allows

Better to miss an insight than to fire incorrectly. Trust collapses if noisy.

## Output Format
💡 Pattern: 4 failed commands in sequence
   Confidence: 81% | Causality: identical error pattern
   Expires: after next successful command

## Commands
contextd start/stop/status/insights/log/signal-log
core registry reality-check (tool usage vs expected_usage)

## Runs as systemd user service
Starts at login. Survives shell exits. Observes always.

## Why Before v14
v14 opinions require observed truth.
Observed truth requires continuous observation.
Without contextd, partner suggestions come from stale snapshots not presence.

## Gate Check
✅ Event emission in core — CommandSucceeded/CommandFailed after every dispatch
✅ forest_events table in state.db
✅ faelight-contextd polls events, detects signals (failure-loop, deploy-unchecked, focus-fragmentation)
✅ forest_insights table populated by signal detection
✅ Intervention gate enforced — importance >= 0.7, confidence >= 0.6, 1h cooldown
✅ fsh surfaces insights after each command
✅ Tool usage logged vs registry expected_usage — forest_events domain tracking
✅ core registry reality-check live — compares actual vs expected usage
✅ contextd runs as systemd user service — active (running), 404K

## The Phrase
"You have a brain. Now give it nerves.
The system that only acts on commands is a calculator.
The system that observes between commands is beginning to think." 🌲
