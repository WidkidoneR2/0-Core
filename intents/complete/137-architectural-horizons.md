---
id: 137
date: 2026-03-18
type: future
title: "Architectural Horizons — Known Future Limits"
status: complete
tags: [architecture, monolith, decisions, capabilities, future, planning]
version: 13.0.0
priority: low
depends_on: [126, 133]
---

## Purpose

This intent exists to remember what the forest cannot yet see clearly
but knows is coming. Not problems today. Known future limits.

The forest that knows its own limits is a forest that will not be
surprised by them. Core v8 Architecture Reflection will reference
this document when generating drift warnings.

## The Three Horizons

### Horizon 1 — The Orchestrator Will Become a Monolith

**Current state:** ~20 domains, clean separation, fast compile.

**When this becomes a problem:**
- Compile times exceed 30 seconds for incremental builds
- Domain A imports from Domain B imports from Domain C
- Capability logic duplicated across 5+ domains
- New developer cannot understand a domain without reading 3 others

**Early warning signals to watch (Core v8 will detect):**
```
coupling index > 0.4 between any two domains
domain file count > 15 in a single mod.rs
cross-domain imports > 3 per domain
```

**Options when the time comes:**
```
Option A — Plugin ABI (dynamic loading)
  Pro: true isolation, hot-reload possible
  Con: complex, unsafe boundary, Rust ABI instability
  When: if domains become independently deployable

Option B — IPC micro-domains
  Pro: clean isolation, language agnostic
  Con: latency, serialization overhead
  When: if domains need independent scaling

Option C — Workspace crates per domain
  Pro: compile isolation, clean dependency graph
  Con: more boilerplate, workspace complexity
  When: RECOMMENDED — natural Rust evolution
  
Recommended path: evolve engine/ into a workspace where
each domain is its own crate. Single surface preserved.
```

**The rule:** Do not refactor until Core v8 detects a real signal.
Not before. Evidence first.

### Horizon 2 — The Decision System Needs Graph Evolution

**Current state:** SQLite, context fingerprints, heuristics engine.

**When this becomes a problem:**
- Query: "what decisions failed under similar entropy + git churn?"
  This is a graph traversal, not a relational join.
- Decision chains: A → B → C where each influenced the next
  Hard to express in flat tables.
- Intent genealogy: which intents spawned which other intents
  Relationships matter as much as entries.

**Early warning signals:**
```
SQL queries exceeding 5 joins
heuristics returning false positives > 20%
decision lookup time > 100ms
```

**Options when the time comes:**
```
Option A — SurrealDB
  Pro: Rust-native, graph + relational hybrid, embedded
  Con: newer, less battle-tested
  
Option B — DuckDB analytical layer
  Pro: SQL on top of existing data, fast analytics
  Con: not a graph DB, still relational

Option C — Custom graph layer on SQLite
  Pro: no new dependency, surgical addition
  Con: reinventing wheels
  
Recommended path: add DuckDB as analytical layer first.
If graph queries become essential, migrate to SurrealDB.
Keep SQLite as the operational store.
```

**The rule:** SQLite serves the forest well today.
Migrate only when query complexity demands it.
Core v9 Intent engine will stress-test this the most.

### Horizon 3 — The Capability System Will Get Painful

**Current state:** capability gating per domain, declarative policies
in sandbox-policies.toml, logged to capabilities.jsonl.

**When this becomes a problem:**
- Same capability combinations repeated across 8+ domains
- Edge cases in capability composition cause silent failures
- New domain author unsure which capabilities to declare
- Audit of "what can access the network?" requires reading all domains

**Early warning signals:**
```
capability declarations duplicated > 3 times
capability-related bugs in > 2 releases
new domain takes > 30 minutes to wire capabilities correctly
```

**Options when the time comes:**
```
Option A — Capability inheritance (you almost have this)
  [[capability_policy]]
  domain = "security"
  inherits = "base_read"
  adds = ["NetworkQuery"]

Option B — Capability DSL
  A small declarative language for composing capabilities
  Similar to what sandbox-policies.toml already does

Option C — Macro-based capability declaration
  #[requires(FilesystemReadHome, NetworkQuery)]
  pub fn my_domain_fn() { }
  
Recommended path: Option A first — extend the existing
TOML policy pattern to domain capabilities.
Low friction, consistent with forest philosophy.
```

**The rule:** The current system is sufficient for 30+ domains.
Extend when duplication becomes measurable, not before.

## The Meta-Rule

**All three horizons share one principle:**

Do not optimize for a problem you do not yet have.
The forest knows these limits exist.
Core v8 will detect when they are approaching.
Core v9 will propose the right time to act.

The ledger remembers. The human decides.

## Relationship to Other Intents
```
INT-126 Core v8    — will detect Horizon 1 drift signals
INT-133 Core v9    — will stress-test Horizon 2 (decision graph)
INT-135 Shell v11  — will expose Horizon 3 capability complexity
```

## Gate Check
```
✅ Core v8 Phase 1 detects Horizon 1 signals — core evolution map (2026-03-20)
✅ Core v10 stress-tested Horizon 2 — 9750 events, SQLite stable (2026-03-26)
✅ Core v10 Horizon 3 check — 34 domains, capability stable (2026-03-26)
✅ core react audit — domains, classes, boundaries visible live (2026-03-26)
```

## The Phrase

**"A forest that knows its limits
does not fear them.
It watches for them,
and acts only when the evidence is clear."**

---
*"Not premature optimization. Patient preparation."* 🌲
