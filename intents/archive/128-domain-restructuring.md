---
id: 128
date: 2026-03-15
type: future
title: "Domain Restructuring — Subdirectory Per Domain"
status: complete
tags: [architecture, domains, structure, organization, v10.9]
version: 10.9.0
priority: high
depends_on: [127]
---

## Vision

`engine/src/domains/` currently has 20+ domains as flat files.
`doctor/mod.rs` is 1500+ lines. `security/mod.rs` is 800+ lines.

As Core v7 and v8 add more domains, flat files become unnavigable.

The solution — one subdirectory per domain, following Smithay's pattern.

## Target Structure
```
engine/src/domains/
├── doctor/
│   ├── mod.rs        — public interface, run()
│   ├── checks.rs     — all 22 health checks
│   ├── cockpit.rs    — cockpit rendering
│   └── events.rs     — event emission
├── security/
│   ├── mod.rs        — public interface
│   ├── scan.rs       — cargo audit, arch-audit
│   ├── advise.rs     — INT-119 judgment layer
│   ├── report.rs     — findings display
│   └── events.rs     — event emission
├── decisions/
│   ├── mod.rs        — public interface
│   ├── ledger.rs     — CRUD operations
│   ├── context.rs    — DecisionContext, fingerprint
│   ├── advise.rs     — Core v6 advisory
│   ├── heuristics.rs — pattern learning
│   └── simulate.rs   — scenario simulation
├── audit/
│   ├── mod.rs        — public interface
│   ├── score.rs      — scoring engine
│   └── report.rs     — output formatting
└── ...               — same pattern for all domains
```

## Migration Strategy

**Non-breaking. Incremental.**

One domain at a time, starting with the largest:
1. doctor (1500+ lines) — highest priority
2. security (800+ lines)
3. decisions (600+ lines)
4. Each subsequent domain as they grow

The public interface (`mod.rs`) stays identical.
Only the internal organization changes.
No command changes. No dispatcher changes.

## Benefits

- doctor/checks.rs is readable alone — 22 checks, one file
- security/advise.rs is the judgment layer, isolated
- Each submodule independently testable
- Adding a new check = adding to checks.rs, not searching mod.rs
- Core v7 anomaly domain starts clean from the beginning

## Gate Check

- [ ] 04-schema/ exists (INT-127 complete)
- [ ] doctor/ subdirectory — split checks, cockpit, events
- [ ] security/ subdirectory — split scan, advise, report
- [ ] decisions/ subdirectory — split ledger, context, advise
- [ ] audit/ subdirectory
- [ ] All other domains migrated
- [ ] No public API changes — only internal restructuring

---
*"Nothing grows without intention. Nothing is added without a home."* 🌲
