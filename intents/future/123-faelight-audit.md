---
id: 123
date: 2026-03-12
type: future
title: "faelight-audit — Tool Intelligence Layer"
status: planned
tags: [audit, tools, intelligence, rust, core-v7, health, v10.8]
version: 10.8.0
priority: high
---

## Vision

The forest notices when parts of itself are being neglected.

Not just "is this tool updated" — but "does this tool still
deserve to exist in its current form?"

52 tools. Each one should be understood, maintained, and active.
faelight-audit makes the forest self-auditing.

## Commands
```bash
faelight-audit scan          # audit all 52 tools — full report
faelight-audit show <tool>   # deep audit of a specific tool
faelight-audit stale         # tools not touched in 90+ days
faelight-audit coverage      # tools missing docs, tests, changelogs
faelight-audit health <tool> # code health score for a tool
faelight-audit score         # ranked list by audit score
```

## Scoring Model

Each tool gets a score (0-100) based on:

| Factor | Weight | Description |
|--------|--------|-------------|
| Last modified | 25% | Days since last meaningful change |
| Usage frequency | 25% | Events in state.db referencing tool |
| Documentation | 20% | README, CHANGELOG present |
| Alias coverage | 15% | Properly aliased in aliases.zsh |
| Version currency | 15% | Version bumped when code changed |

## The Jarvis Integration

Tools scoring below threshold surface in `core advise`:
```
Advisory:
  → faelight-browser: last touched 47 days ago, low usage
    Suggest: archive, improve, or document
  → faelight-sandbox: no README — documentation debt
  → faelight-term: high usage but no CHANGELOG
```

The forest becomes self-aware about its own tool health.

## Integration Points

- Reads from state.db events to measure tool usage
- Reads from registry (tools.toml) for tool metadata
- Reads git log to determine last meaningful change per tool
- Feeds into `core advise` when tools score below threshold
- Part of Core v7 anomaly detection pillar (INT-122)

## Success Criteria

- [ ] `faelight-audit scan` produces scored report for all tools
- [ ] Usage data pulled from state.db events
- [ ] Stale tools surface in `core advise`
- [ ] doctor gains optional tool audit check
- [ ] Each tool has an audit score in the registry

---
*"A forest that knows its own health knows when a tree needs care."* 🌲
