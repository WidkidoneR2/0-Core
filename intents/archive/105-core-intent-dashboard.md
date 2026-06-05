---
id: 105
date: 2026-03-02
type: future
title: "core intent dashboard — Terminal Intent Overview"
status: complete
tags: [core, intent, tui, rust, dashboard, glow]
version: TBD
priority: medium
---

## Vision

A single terminal view showing the complete intent landscape:
focused intent, workflow state, drift status, checkpoint timeline,
active intents, and upcoming work — all in one place.
```
core intent dashboard
┌─ 🎯 Focus ──────────────────────────────────┐
│  INT-098  Core v4 — The Reliable System      │
│  State: in-progress  Since: 2h 14m           │
│  Drift: ✅ on course (4 commits)             │
└──────────────────────────────────────────────┘
┌─ 📸 Recent Checkpoints ─────────────────────┐
│  pre-phase2-intent-focus   95%  12:20        │
│  auto-intent-098-start     95%  12:32        │
└──────────────────────────────────────────────┘
┌─ 📋 Active Intents ─────────────────────────┐
│  098  Core v4 — The Reliable System  🟡      │
│  099  Niri Migration                 📋      │
└──────────────────────────────────────────────┘
```

## Success Criteria

- [x] Single command: `faelight-intent`
- [x] Focus + drift inline
- [x] Recent checkpoints
- [x] Active intent list
- [x] Refreshes on keypress

---

*"The forest knows where it is going."* 🌲
