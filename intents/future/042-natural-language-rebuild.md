---
id: 042
date: 2026-06-09
type: feature
title: "natural-language-rebuild: natural language triggers system actions"
status: planned
tags: [nl, fsh, natural-language, friday, vocabulary, system-actions]
priority: medium
---
## Why
The `?` prefix already translates natural language to pipelines (INT-139).
It handles filesystem, process, and git queries well.
What it does not handle: forest system actions.

"?rebuild the system" should trigger rebuild.
"?check health" should run d.
"?start the vm" should run vm start.
"?show active intents" should run intent list.
"?what am I working on" should show active intent.

These are not pipelines -- they are forest vocabulary commands.
The natural language layer should know the difference.

## What Already Exists
nl.rs: 35+ patterns, ?prefix, confirm-before-execute discipline
Pattern struct: phrases[], pipeline, context
THE RULE: show generated command before executing -- never silent

## What This Intent Adds

1. System action patterns
   Extend PATTERNS in nl.rs with forest-native system actions:
   "rebuild", "rebuild system", "switch config"  → rebuild
   "health check", "check health", "how is forest" → d
   "start vm", "launch vm", "open lab"            → vm start nixos-lab
   "active intent", "what am I working on"        → intent list (in-progress filter)
   "show intents", "what is planned"              → intent list
   "lock core", "lock the forest"                 → core-protect lock
   "unlock core"                                  → core-protect unlock

2. Action vs pipeline distinction
   Current Pattern.pipeline is always a pipeline string.
   System actions need a new field: action_type (Pipeline | Command | ForestVerb)
   ForestVerb actions bypass the pipe renderer and show as: "→ run: <command>"

3. Confirm discipline maintained
   System actions still show before executing.
   Destructive actions (rebuild, lock) require explicit y/n.
   Read-only actions (health, intent list) can auto-confirm.

4. User-defined action patterns
   nl-patterns.toml already supports user patterns.
   Extend schema to support action_type field.

## Depends On
- INT-030 (fsh semantic domains) -- forest verb vocabulary must exist first
- INT-261 (fsh vocabulary) -- verb taxonomy informs pattern design

## Phases

Phase 1 -- System action patterns
  Add 10 system action patterns to PATTERNS in nl.rs
  Add action_type field to Pattern struct (Pipeline | Command)
  Gate: ?rebuild shows confirm prompt and runs rebuild on y

Phase 2 -- Forest verb patterns
  Add intent, vm, project, experiment patterns
  Gate: ?active intents shows intent list filtered to in-progress

Phase 3 -- User-defined action patterns
  Extend nl-patterns.toml schema with action_type
  Gate: user can define custom system action patterns

## Gates
- [ ] Pattern struct has action_type field (Pipeline | Command)
- [ ] ?rebuild triggers rebuild with confirm
- [ ] ?health check triggers d
- [ ] ?start vm triggers vm start with qcow2 picker
- [ ] ?active intents triggers intent list in-progress filter
- [ ] ?what am I working on shows focused intent
- [ ] Destructive actions require explicit y/n confirm
- [ ] Read-only actions auto-confirm
- [ ] User-defined action patterns work via nl-patterns.toml

## The Rule
"The shell understands intent, not just syntax.
 Natural language is the door.
 The forest decides what it means." 🌲
