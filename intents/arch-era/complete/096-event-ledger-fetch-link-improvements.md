---
id: 096
date: 2026-02-27
type: complete
title: "Core v3 Phase 1 + faelight-fetch v2.3.0 + core link sync"
status: complete
tags: [v10.2, event-ledger, fetch, link, tools, aliases]
version: 10.2.0
---

## What Was Built

### Core v3 Phase 1 — Event Ledger
First phase of the v3 Living System (INT-093). Additive only — no existing
code changed, no daemon, no async. Pure write path.

- `EventWriter` added to runtime/
- 4 domains wired: doctor, git, security, update
- New commands: `core events list`, `core events since`, `core events filter`
- Aliases: `ce`, `ces`, `cef`
- Pre-wired aliases for Phase 2: `cw` (core why), `ctr` (core trace)

### faelight-fetch v2.3.0
- Health reads from cache file (instant) — was running full dot-doctor
- Live CPU, memory, disk via sysinfo
- Terminal emulator detection via process tree walk
- Commit count and tool count from live system
- Rust version display
- Sectioned output (system / env / resources / 0-core)

### core link sync
GNU Stow replacement workflow reduced from 4 manual steps to 1 command.
- `core link sync` — deploys clean, surfaces conflicts with exact fix commands
- Removes DEBUG eprintln that was shipping to users
- redeploy now snapshots by default
- Aliases: `cls` (sync), `clp` (plan)

### New Tool Aliases (8)
atuin, tokei, hyperfine, ouch, difft, btm, onefetch integrated into daily workflow.
`top`, `repo`, `bench`, `extract`, `compress`, `diff`, `loc`, `loch`

## Rationale

Event Ledger is the foundation for Phase 2 (core why) and Phase 3 (simulation).
Building write path first means event schema is proven before the bus is built.

faelight-fetch was slow (running dot-doctor on launch) and missing basic system info.

core link sync eliminates the most painful recurring workflow in the system —
stow/restow has caused 3 incidents (INT-INC-003).

## Session Rules Followed
- One phase per session ✅
- Ended with doctor at 95%+ ✅
- No v3 work without prior planning ✅
