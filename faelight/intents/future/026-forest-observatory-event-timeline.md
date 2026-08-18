---
id: 026
date: 2026-06-04
type: feature
title: "Forest Observatory: searchable event timeline"
status: planned
tags: [friday, observatory, timeline, history, events]
priority: low
---

## Vision

Ask the forest: "what was I working on in March?"
The forest answers with a rich timeline of intents, commits, rebuilds, and experiments.

## Why

After a year of development, context is everything.
Friday already collects events -- they need a searchable interface.

## Approach

- core observatory command with date range queries
- Sources: intent_commits, events, health_patterns, git log
- Display: timeline view with intent/commit/rebuild markers
- Friday integration: natural language date queries
- "core obs --week" "core obs --intent 016" "core obs --since march"

## Gate

- [ ] core observatory shows last 7 days by default
- [ ] Date range filtering works
- [ ] Intent + commit + rebuild events unified in one timeline
- [ ] Friday can answer "what was I working on last week"
