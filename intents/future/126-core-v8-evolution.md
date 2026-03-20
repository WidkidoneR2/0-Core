---
id: 126
date: 2026-03-15
type: future
title: "Core v8 — Evolution: The Forest Refines Itself"
status: planned
tags: [core, v8, evolution, architecture, intelligence, proposals, v12]
version: 12.0.0
priority: medium
depends_on: [122]
---

## The Core Timeline

| Version | Capability | Meaning |
|---------|-----------|---------|
| v2 | Structure | the forest has shape |
| v3 | Awareness | the forest observes itself |
| v4 | Discipline | the forest enforces rules |
| v5 | Intelligence | the forest detects patterns |
| v6 | Judgment | the forest remembers outcomes |
| v7 | Resilience | the forest can rebuild |
| **v8** | **Evolution** | **the forest refines itself** |

v8 is the moment the system moves from self-healing → self-improving.

## The Core Insight

Right now the forest can remember its history, detect anomalies,
reconstruct itself, and reason about risk.

But it cannot yet recognize when its own architecture should change.

v8 allows the forest to answer:
- Which domains are becoming too complex?
- Which tools are unused or redundant?
- Which dependencies are increasing risk?
- Which patterns in decisions indicate architectural drift?

v8 transforms the forest from historian → strategist.

**Critically: improvement proposals require human approval.**
The forest suggests evolution. The human authorizes it.


## The Foundational Rule

**"No suggestion without strong evidence."**

This is the non-negotiable law of Core v8.
Every suggestion the forest makes must cite:
```
Source      — which data (events, audit scores, git log, decisions, dep graph)
Threshold   — what specific value crossed what specific line
Evidence    — the actual numbers, not interpretations
Confidence  — how many data points support the signal
```

### What this looks like in practice

❌ WRONG — noise:
```
"dependency risk seems elevated"
"tool X may be redundant"  
"architecture could be drifting"
```

✅ RIGHT — evidence:
```
"openssl used by 6 tools (core deps risk), 0 decisions recorded
 for openssl in last 90 days, 2 findings in security audit — HIGH RISK
 Source: deps domain + security domain + decisions table
 Confidence: 3 independent signals"

"faelight-update and safe-update: 68% function overlap measured
 across codebase, safe-update has 0 events in last 30 days,
 audit score delta < 5 — REDUNDANT
 Source: audit scores + event log + static analysis
 Confidence: 2 signals, 1 corroborating"

"security domain: 4 new functions added outside declared scope
 in v10.7→v11.0, coupling index increased 0.3, cross-domain
 imports: 3 new in last 2 versions — DRIFT DETECTED
 Source: git log + static analysis + coupling metric
 Confidence: 3 signals over 2 versions"
```

### Evidence Thresholds (must be calibrated in Phase 1)

| Signal Type | Minimum Evidence | Confidence Levels |
|-------------|-----------------|-------------------|
| Tool redundancy | 2 independent signals | Low/Med/High |
| Dependency risk | CVE data + usage count + decision age | Low/Med/High |
| Architecture drift | 2 versions of increasing coupling | Low/Med/High |
| Tool lifecycle | 30 days inactivity + audit score trend | Low/Med/High |
| Decision pattern | 3+ similar decisions with same outcome | Low/Med/High |

A LOW confidence suggestion is still shown — but labeled clearly.
The human decides what to act on. The forest never decides alone.

## The Five Pillars

### Pillar 1 — Architectural Reflection
The forest studies its own architecture.
```bash
core architecture map        # visualize system structure
core architecture drift      # detect architectural entropy
core architecture hotspots   # find highest-change zones
core architecture suggest    # propose improvements
```

Example — architecture map:
```
core
 ├── domains (20+)
 │   ├── intelligence
 │   ├── system
 │   ├── judgment
 │   └── security
 ├── registry
 ├── runtime
 └── rust-tools (43)
```

Example — architecture drift:
```
Drift detected:
  domain: security
  issue: responsibilities expanding beyond scope
  recommendation: consider security/intelligence split
```

### Pillar 2 — Decision Pattern Intelligence
The forest studies its own decision ledger.
```bash
core decisions patterns    # find repeating decision types
core decisions friction    # detect decisions requiring repeated corrections
core decisions reversal    # detect architectural reversals
```

Example — reversal detection:
```
v8.2: removed tool X
v8.4: reintroduced tool X
Recommendation: investigate root cause
```

### Pillar 3 — Tool Ecosystem Intelligence
```bash
core tools usage        # identify rarely-used tools
core tools redundancy   # detect overlapping tools
core tools lifecycle    # track tool maturity stages
```

Example — redundancy:
```
faelight-update / safe-update
Function overlap detected: 68%
Suggestion: consider merge
```

### Pillar 4 — Evolution Proposals
```bash
core evolve propose     # generate architectural proposal
core evolve review 12   # review a proposal
core evolve accept 12   # accept → creates intent automatically
```

Example proposal:
```
Proposal 12
Type: Domain Refactor
Change: create intelligence/dependency domain
Reason: deps logic growing beyond security domain
```

Accepting a proposal automatically creates an intent in the ledger.
The forest's suggestion becomes part of the permanent record.

### Pillar 5 — Future Simulation
Extends the v6 simulate domain.
```bash
core future simulate    # simulate architectural change
core future risk        # risk of a proposed change
core future impact      # impact analysis
```

Example:
```
core future simulate "remove dependency graph module"

Affected domains: security, bootstrap, narrative
Risk: high
```

## Relationship to Architectural Horizons (INT-137)

Core v8 Phase 1 actively monitors for the three known horizons:
- Horizon 1: coupling index, domain file count, cross-domain imports
- Horizon 2: query complexity, decision lookup time
- Horizon 3: capability duplication count, new domain setup time

When a threshold is crossed, Core v8 surfaces it with full evidence.
The human decides when to act.

## The Three Guardrails

**These are non-negotiable.**

**Rule 1** — The forest never changes architecture automatically.
Only proposes. Always waits for human authorization.

**Rule 2** — Every accepted proposal creates an intent record.
The ledger is the bridge between suggestion and decision.

**Rule 3** — Architectural changes remain human-authored code.
The forest assists. The human writes.

## Structural Changes Required
```
engine/src/domains/evolution/
   architecture/
   decisions/
   tools/
   future/

runtime/evolution/
runtime/architecture-snapshots/
runtime/proposals/
```

## Build Order
```
Phase 1 — architecture map
Phase 2 — tools usage analysis
Phase 3 — decision pattern detection
Phase 4 — architecture suggestions
Phase 5 — evolution proposals
Phase 6 — future simulation
```

## Gate Check
```
✅ Core v7 complete — all 7 phases done (2026-03-17)
✅ Phase 1 — architecture map (2026-03-20)
✅ Phase 2 — tools usage analysis (2026-03-20)
⬜ Phase 3 — decision pattern detection
⬜ Phase 4 — architecture suggestions
⬜ Phase 5 — evolution proposals
⬜ Phase 6 — future simulation
```

## The Phrase

**"A wise forest does not grow randomly.
It studies its own rings and chooses
where the next branch should grow."**

---
*"v8 is not automation. It is wisdom applied to architecture."* 🌲
