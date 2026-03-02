---
id: 110
date: 2026-03-02
type: future
title: "core why visual — Workspace Topology in Event Ledger"
status: planned
tags: [core, why, visual, workspace, topology, events, glow]
version: TBD
priority: medium
depends_on: [109]
---

## Vision

When faelight-compositor joins the family, the event ledger
gains a new domain: visual topology.

core why visual answers questions no tool has ever answered:

- What visual topology correlates with git churn?
- Does focus instability precede health drift?
- When did attention fragment across workspaces?
- What was on screen when the last incident occurred?

## What Gets Tracked
```
visual_event domain:
  workspace.switch    — when, from where, to where
  window.focus        — which app, how long held
  window.open         — what opened, on which workspace
  layout.change       — tiling mode transitions
  attention.fragment  — multiple workspace switches in <30s
```

## Commands
```
core why visual           — visual activity summary
core why workspace 3      — what happened on workspace 3
core why attention        — attention fragmentation analysis
core why topology         — current workspace map
```

## Depends On

- INT-109 faelight-compositor (event source)
- Core v3 causality engine (event analysis)

## Success Criteria

- [ ] Visual events flowing from compositor
- [ ] Workspace topology queryable
- [ ] Attention fragmentation detection
- [ ] Correlation with health events
- [ ] `core why visual` summary

---

*"The forest knows where your eyes have been."* 🌲
