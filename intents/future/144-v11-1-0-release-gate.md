---
id: 144
date: 2026-03-21
type: future
title: "v11.1.0 Release Gate — The Forest Speaks"
status: in-progress
tags: [release, gate, v11, planning]
version: 11.1.0
priority: high
---

## Release Name
**v11.1.0 "The Forest Speaks"**

The forest has structure, awareness, judgment, evolution.
Now it speaks — through notifications, digests, voice, and a shell
that understands natural language.

## Gate Requirements

### ✅ Already Complete
- Phase 18 — time travel (snapshot/timeline/diff)
- Phase 21 — query language (SQL-like syntax)
- Phase 22 — observability dashboard
- Phase 17 — event system (triggers)
- Phase 16 — history analytics
- Phase 15 — git data engine
- Phase 14 — file system index
- Core v8 — all 6 phases complete
- faelight-term — daily driver ready

### ⬜ Required for v11.1.0
- INT-141 faelight-notify v4 — COMPLETE (replaces INT-132 gate)
  - zbus D-Bus server
  - Wayland rendering
  - notify-send works
  - Brave browser notifications work
- INT-143 faelight-digest — Phase 1 complete
  - Morning forest summary on long gap
  - Commits since last session
  - Health + forecast trend
  - Active intents summary
- INT-120 Phase 25 — NL assistant complete
  - INT-139 amplifier working
  - Confidence + confirm flow
- INT-120 Phase 9/22 — mark real-time observability complete
- INT-139 — Custom pattern support via TOML

### 🔄 Deferred to v11.2.0
- INT-132 faelight-vault (depends on INT-109 Sessions 5-8)
- INT-109 Sessions 5-8 — DRM first render on real hardware

## v11.2.0 "The Compositor Wakes" Gate
- INT-109 Sessions 5-8 complete — forest green on real hardware
- INT-132 faelight-vault complete
- Phase 6 — .fsh scripting language
- Phase 32 — Shell as OS layer

## The Phrase
**"The forest that speaks
is the forest that connects.
v11.1.0 gives the forest its voice."**
