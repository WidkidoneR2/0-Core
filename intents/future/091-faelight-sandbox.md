---
id: 091
date: 2026-02-20
type: future
title: "faelight-sandbox - Controlled Experimentation Environment"
status: planned
tags: [rust, sandbox, btrfs, namespaces, zones, experimentation]
version: 10.0.0
---

## Vision
Fearless experimentation with full comprehension. Not isolation for security theater —
experimentation with visibility and instant rollback.

## Core Questions It Answers
- What did this process touch?
- What files did it modify?
- What system state changed?
- Can I revert instantly?
- Can I see exactly what happened?

## Philosophy
"Experiment freely. Understand completely. Revert instantly."

## MVP (v1.0.0) Scope
- Btrfs snapshot wrapper (`run --snapshot`)
- Network quarantine (`--net=off` via unshare)
- Zone-aware write restrictions
- Post-execution diff report
- `keep/discard` prompt

## Deferred (v2.0+)
- LD_PRELOAD dry-run mode
- Bar integration (amber/sandbox mode indicator)
- Risk scoring
- Intent-aware prompting

## CLI Design
```
faelight-sandbox run <cmd>          # Run in snapshot sandbox
faelight-sandbox run --net=off <cmd> # No network access
faelight-sandbox diff               # Show what changed
faelight-sandbox discard            # Rollback snapshot
faelight-sandbox commit             # Keep changes
faelight-sandbox status             # Is sandbox active?
```

## Architecture
- Layer 1: Btrfs snapshot of workspace
- Layer 2: unshare namespaces (mount, network optional)
- Layer 3: Zone-aware write allowlist
- Layer 4: Post-execution report (files changed, disk delta)

## Success Criteria
- [ ] Btrfs snapshot creation and rollback
- [ ] Network namespace isolation
- [ ] Zone write restrictions enforced
- [ ] Post-execution diff report
- [ ] Keep/discard prompt
- [ ] dot-doctor integration (sandbox active warning)
