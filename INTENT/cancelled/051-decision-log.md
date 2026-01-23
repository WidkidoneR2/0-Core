---
id: 051
date: 2026-01-14
type: future
title: "Decision Log - Capture the Why"
status: cancelled
tags: [v8.0, infrastructure, knowledge, 0-core]
version: 8.0.0
---

## Vision
Never forget why you chose X over Y. Decisions are valuable knowledge that degrades over time.

**Philosophy:** "Future you should understand past you."

## The Problem
- Decisions made daily ("why Rust over Python?")
- Rationale forgotten in weeks/months
- Alternatives not documented
- New contributors confused by choices
- Repeat same debates

## The Solution

### Decision Structure
```markdown
---
id: DEC-XXX
date: YYYY-MM-DD
category: architecture|tooling|workflow
status: cancelled|superseded|experimental
intent: XXX  # Which intent prompted this
---

## Decision
[One sentence: what was decided]

## Context
[Why this decision was needed]

## Alternatives Considered
1. **Option A**
   - ✅ Benefits
   - ❌ Drawbacks

2. **Option B** (Chosen)
   - Rationale

## Impact
[What changed as a result]

## Review Date
[When to reconsider]
```

### Commands
```bash
decision log "Decision text" \
  --rationale "Why" \
  --alternatives "What else" \
  --intent XXX

decision list
decision show <id>
decision search <term>
decision timeline
```

## Success Criteria
- [ ] CLI tool: `decision`
- [ ] Storage: `~/.local/state/0-core/decisions/`
- [ ] Search by category, date, intent
- [ ] Markdown format (human readable)
- [ ] Integration with intent system
- [ ] Timeline view

## Timeline
**v8.0.0 (Post v7.0.0)**

---

_"Decisions documented are decisions remembered."_ 🌲
