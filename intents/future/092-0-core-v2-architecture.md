---
id: 092
date: 2026-02-20
type: future
title: "0-Core v2 — Intentional Architecture Redesign"
status: planned
tags: [architecture, v2, rust, security, philosophy, long-term]
version: 2.0.0
priority: high
---

## Vision

Move from "many tools organized cleanly" to "one orchestrator managing domains."

v1 proved the philosophy works. v2 makes it structurally sound for the next decade.

## Why Now

v1 has real strengths — 40+ tools, 100% path resilience, rigorous health checking.
But it has inherited structural debt:

- Tools bleed across layer boundaries
- Mutable state is scattered (logs, state, cache at top level)
- 40 binaries means 40 cold starts, 40 attack surfaces
- No formal capability model — trust is implicit
- Wrapper scripts introduce dynamic execution risk
- No migration path for schema changes

v2 fixes the structure without abandoning the philosophy.

## Five Hard Rules

If something violates these, it does not ship.

1. **Single binary surface** — one `core` binary, subcommands for domains
2. **Strict layer boundaries** — no cross-layer leakage, enforced at compile time
3. **All mutable state isolated** — nothing outside `runtime/` may change at runtime
4. **Declarative over imperative** — registry contains zero logic, only truth
5. **Capability-based execution** — every subcommand declares what it needs

## Layer Model
```
LAYER 0 — Substrate (Untouched)
  Kernel, systemd, Wayland, Sway, Network, Filesystem
  Treated as external environment. Never owned. Never modified directly.

LAYER 1 — Core Engine (Single Binary)
  /core/                  ← Rust source
  Binary: core
  Domains: intent, profile, security, doctor, update, sandbox, link, zone

  Replaces: all faelight-* binaries
  Interface: core <domain> <command> [flags]
  Examples:
    core intent new
    core security audit
    core doctor run
    core profile switch work
    core update safe
    core sandbox run --net-off <cmd>

LAYER 2 — Declarative Registry (Zero Logic)
  /registry/
    packages.toml     ← system packages under management
    profiles.toml     ← profile definitions
    zones.toml        ← zone boundaries and permissions
    aliases.toml      ← shell alias declarations
    tools.toml        ← tool registry (replaces scattered metadata)

  Rule: If it contains an if statement, it does not belong here.

LAYER 3 — Policy (What Is Allowed)
  /policy/
    security/         ← security rules and baselines
    hooks/            ← hook definitions (no execution logic)
    health/           ← doctor check definitions

  Rule: Policy defines constraints. It never executes.

LAYER 4 — Runtime (All Mutable State)
  /runtime/           ← gitignored entirely
    logs/             ← structured JSONL logs by domain
    cache/            ← precomputed indices
    snapshots/        ← sandbox and rollback state
    state.db          ← single state database
    locks/            ← operation locks

  Rule: Nothing outside runtime/ changes during normal operation.
  Benefit: `rm -rf runtime/` is always safe. Full reset, no data loss.

LAYER 5 — Adapters (Thin Translation Only)
  /adapters/
    sway/             ← sway config generation from registry
    systemd-user/     ← service definitions
    pacman/           ← package management hooks
    git/              ← git integration

  Rule: No business logic. Only translation between core and external systems.
```

## Security Surface Minimization

### Capability Model
Each domain subcommand declares capabilities in source:
```rust
#[capabilities(filesystem_write_runtime, pacman_query)]
fn cmd_security_audit() { ... }
```

Core enforces before execution. No implicit trust.

### Privilege Separation
Two binaries only:
- `core` — unprivileged, all normal operations
- `core-admin` — explicit privilege escalation, audited separately

### No Shell Scripts
- No dynamic eval
- No sourcing external files
- No inline bash logic
- Everything compiled, everything typed

### Structured Logging
```
runtime/logs/security.jsonl
runtime/logs/doctor.jsonl
runtime/logs/intent.jsonl
```
No scattered log files. All queryable.

## Execution Speed

- **Cold start**: one process instead of 40 binary spawns
- **Lazy domain loading**: `core security audit` loads only security module
- **Precomputed index**: `core registry build-index` → `runtime/cache/index.bin`
- **Shared state**: domains share parsed registry without re-reading disk

## Long-Term Scalability

### Versioned Schema
```
VERSION file at root
core migrates runtime schema automatically on upgrade
```

### Domain Isolation
Each domain in `/core/src/domains/<name>/`:
- Own module boundary
- Own capability declarations
- Own tests
- No circular dependencies enforced by Rust module system

### Future Plugin Boundary
```
core plugin add <name>
```
Plugins run in sandboxed subprocess (faelight-sandbox as substrate).
Core API surface is explicit and versioned.

## Proposed v2 Layout
```
0-core/
  core/
    src/
      domains/
        intent/
        security/
        doctor/
        profile/
        update/
        sandbox/
        link/
        zone/
      registry/
      policy/
      capabilities/
      main.rs
    Cargo.toml
  registry/
  policy/
  adapters/
  runtime/           ← gitignored
  docs/
  intents/
  Cargo.toml
  README.md
  VERSION
```

## Migration Strategy

v2 is a rewrite, not a refactor. Migration must be intentional:

### Phase 1 — Design (No Code)
- [ ] Finalize domain boundaries
- [ ] Define capability model formally
- [ ] Design registry schema
- [ ] Map all v1 tools to v2 domains
- [ ] Identify what gets deleted entirely

### Phase 2 — Parallel Build
- [ ] Scaffold `core` binary with CLI skeleton
- [ ] Implement domains one at a time
- [ ] Each domain passes v1 parity tests before ship
- [ ] v1 and v2 run side by side during transition

### Phase 3 — Cutover
- [ ] All domains passing
- [ ] v1 tools removed
- [ ] Adapters regenerated from registry
- [ ] `runtime/` fully isolated

### Phase 4 — Cleanup
- [ ] Remove v1 artifacts
- [ ] Update docs, intents, registry
- [ ] Bump VERSION to 2.0.0

## What Gets Deleted in v2

- All individual `faelight-*` binaries (replaced by `core` domains)
- Wrapper scripts with logic
- Scattered state files outside runtime/
- Duplicate documentation
- Top-level logs and backups
- `.dotmeta` workaround files

## What Stays

- Intent ledger system (refined, not replaced)
- Zone model (formalized in registry)
- Security audit domain
- Sandbox domain
- Philosophy documents
- Git governance model
- Stow package structure (adapters/systemd-user etc)

## Success Criteria

- [ ] Single `core` binary replaces all faelight-* tools
- [ ] `core doctor run` passes 20/20 checks
- [ ] Cold start < 50ms for any subcommand
- [ ] Zero shell scripts with logic
- [ ] All mutable state in runtime/
- [ ] Capability model enforced at compile time
- [ ] Clean migration from v1 with no data loss
- [ ] All intents migrated to v2 registry format

## Open Questions (Decide Before Coding)

1. Does `dot-doctor` become `core doctor` or stay separate during transition?
2. How does stow fit — do adapters/ replace the stow package model?
3. What is the exact capability taxonomy?
4. Is `core-admin` a separate crate or a feature flag on `core`?
5. What is the state.db schema? SQLite or custom TOML?

---

_"One orchestrator. Five layers. Zero ambiguity."_ 🌲
