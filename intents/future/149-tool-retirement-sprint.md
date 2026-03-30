---
id: 149
date: 2026-03-25
type: future
title: "Tool Retirement Sprint — Clean What the Core Has Absorbed"
status: in-progress
tags: [tools, retirement, cleanup, core, shell, hygiene, v12]
version: 12.0.0
priority: medium
depends_on: [133, 146]
---

## The Problem

The core engine and faelight-shell have quietly absorbed the jobs
of several older tools. Those tools still exist in scripts/ —
taking up space, confusing the tool count, and creating false aliases.

A forest that documents itself must also clean itself.

## The Retirement Rule

A tool is ready for retirement when ALL of these are true:
1. Its primary function is fully covered by core or faelight-shell
2. No active alias points to it exclusively
3. Its removal does not break any health check
4. It has been documented as retired in the intent ledger

**Retirement is not deletion of history — it is acknowledgment of evolution.**

## Confirmed Retirement Candidates

| Tool | Replaced By | Status |
|------|------------|--------|
| `archaeology-0-core` | `gc`, `gchurn` in faelight-shell | confirm |
| `workspace-view` | `dashboard` in faelight-shell | confirm |
| `entropy-check` | `core security` domain | confirm |
| `bin-doctor` | `core doctor bins` | investigate |
| `faelight-search` | `?` NL queries in faelight-shell | confirm |

## The Retirement Process (per tool)
```
1. core evolution tools          — confirm tool is active/dormant
2. grep aliases for tool name    — find all alias references
3. Remove binary from scripts/   — requires unlock-core + sudo
4. Update aliases.zsh            — remove or remap aliases
5. Run: d                        — verify 100% health
6. fg commit with intent ref     — document the retirement
7. lock-core                     — seal the change
```

One tool at a time. Never batch retire without verifying health between each.

## Gate Check
```
⬜ archaeology-0-core — deferred to INT-162 layer audit — health check in doctor/checks.rs must be updated first
⬜ workspace-view — deferred to INT-162 layer audit — workspace/mod.rs calls it directly
⬜ entropy-check — deferred — absorbed into doctor/entropy.rs (457 lines), binary retirement safe after INT-162
⬜ bin-doctor — deferred — absorbed into doctor/bins.rs (253 lines), binary retirement safe after INT-162
✅ faelight-search RETIRED (2026-03-26) — binary removed, source removed, registry cleaned, aliases commented out
✅ 100% path resilience verified after faelight-search retirement (44/44)
✅ tools.toml updated — 49 tools registered
```

## Retirement Philosophy (2026-03-30)
Tool retirement is not a sprint — it is a careful process.
The rule: core absorbs the function → verify completely → retire safely → one at a time.
Remaining candidates are deferred to INT-162 (Shell Architecture Hardening)
where the layer audit will make the dependencies explicit and safe to remove.
INT-179 (faelight-shell v3) will handle zsh alias retirement as fsh coverage grows.

## The Phrase

**"The forest that grows
must also prune.
Not every branch that once bore fruit
belongs on the tree today."**

---
*"Retirement is not failure. It is evolution made visible."* 🌲
