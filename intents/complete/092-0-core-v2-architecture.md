---
id: 092
date: 2026-02-20
type: active
title: "0-Core v2 — Intentional Architecture Redesign"
status: active
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

### Phase 2 — Parallel Build ✅ COMPLETE 2026-02-20
- [x] Scaffold `core` binary with CLI skeleton
- [x] Implement domains one at a time
- [x] Each domain passes v1 parity tests before ship
- [x] v1 and v2 run side by side during transition

### Phase 3 — Cutover ✅ COMPLETE 2026-02-22
- [x] All domains passing (✅ 15/15 wired, delegation complete)
  - ✅ link — fully native (adopt, deploy, plan, undeploy, redeploy)
  - ✅ update — path fixed, flag passthrough working
  - ✅ doctor — 1065 lines native, sub-tools use scripts path
  - ✅ all domains — hardcoded paths removed, portable
- [x] v1 tools removed
  - ✅ faelight-launcher — removed, superseded by faelight-palette
  - ✅ faelight-dmenu — removed, superseded by faelight-palette
  - ✅ all scripts — target/release paths removed
- [x] `runtime/` fully isolated — gitignored, state.db, logs, cache, locks, snapshots
- [x] Cold start < 50ms — measured 3ms ✅

### Phase 4 — Cleanup ⬜ NEXT
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

- [x] Single `core` binary — 15/15 domains wired (Phase 2 complete)
- [x] `core doctor run` passes health checks — 21/22 (95%, core unlocked warning only)
- [x] Cold start < 50ms — measured 3ms ✅
- [x] Zero shell scripts with logic — all scripts are thin wrappers or compiled binaries
- [x] All mutable state in runtime/ — complete, gitignored
- [ ] Capability model enforced at compile time — Phase 6
- [x] Clean migration from v1 with no data loss — confirmed
- [ ] All intents migrated to v2 registry format — Phase 4

## Open Questions — RESOLVED

1. **Does `dot-doctor` become `core doctor`?** → Yes. `dot-doctor` delegates to `core doctor run`. The doctor domain is the last migrated, acting as system validator.
2. **How does stow fit?** → Stow stays. It manages dotfiles at the filesystem layer (Layer 0). Adapters/ will eventually generate configs that stow deploys.
3. **Capability taxonomy?** → Defined and implemented. See Capability Taxonomy table below.
4. **Is `core-admin` separate?** → Deferred to Phase 3. For now, privileged ops delegate to v1.
5. **State.db schema?** → SQLite. Initialized in Phase 1. Schema defined below.

---

_"One orchestrator. Five layers. Zero ambiguity."_ 🌲

## Phase 2 Completion Record

**Date:** 2026-02-20  
**Sessions:** 12–13  
**Result:** ALL 15/15 domains migrated

| Domain | Status | Notes |
|---|---|---|
| link | ✅ | Native impl |
| zone | ✅ | Native impl |
| intent | ✅ | Native impl |
| profile | ✅ | Native impl |
| security | ✅ | Native impl |
| sandbox | ✅ | Native impl + btrfs reflink |
| fetch | ✅ | Native impl |
| git | ✅ | Native + delegates complex ops |
| workspace | ✅ | Delegates TUI tools |
| release | ✅ | Native version reading |
| notify | ✅ | Native via notify-send |
| lock | ✅ | Native via swaylock |
| launcher | ✅ | Delegates TUI tools |
| update | ✅ | Delegates (system safety) |
| doctor | ✅ | Delegates dot-doctor v1 |

**Key decisions made during Phase 2:**
- `engine/` chosen over `core/` to avoid path confusion
- `core` alias updated: was `cd ~/0-core`, now v2 binary; `0core` for navigation
- `alias-audit --doctor` flag routed to workspace target binary (not 04-runtime)
- Interactive TUI tools (fm, palette, workspace-view) delegate fully — not wrappable
- faelight-palette v3 shipped 2026-02-22 — split layout, real stats, replaces launcher+dmenu
- dot-doctor rebuilt to fix path resilience 158% → 100%

**Phase 3 progress (started 2026-02-22):**
- faelight-link fully native — adopt command, dir-folding fix, 0 false conflicts
- faelight-palette v3 — split layout, real 0-Core stats, core lock status, replaces launcher+dmenu
- faelight-fm — clean filelist, rich preview panel, SCR scratch zone, UTF-8 panic fix
- core update — path fix, allow_hyphen_values flag passthrough
- faelight-link adopt — standalone command, GNU Stow dir-folding awareness

**Phase 3 entry criteria:**
- All 15 domains have native Rust implementations (not delegation)
- v1 rust-tools/ binaries deprecated
- runtime/ fully isolated
- Doctor passes 21/21

---

## Internal Rust Module Layout

### High-Level Structure
```
core/
├── main.rs              ← thin: parse args, init logging, dispatch to app::run()
├── cli/
│   ├── mod.rs
│   ├── commands.rs      ← command grammar definitions
│   └── parser.rs        ← clap setup
├── app/
│   ├── mod.rs
│   ├── dispatcher.rs    ← routes CLI commands to domains
│   └── context.rs       ← AppContext: registry + policy + runtime + capabilities
├── domains/
│   ├── intent/
│   │   ├── mod.rs
│   │   ├── commands.rs
│   │   ├── model.rs
│   │   └── errors.rs
│   ├── security/
│   ├── doctor/
│   ├── profile/
│   ├── update/
│   ├── sandbox/
│   ├── link/
│   └── zone/
├── registry/
│   ├── loader.rs        ← parse TOML, validate schema
│   ├── models.rs        ← typed structs for all registry files
│   └── validator.rs     ← schema enforcement
├── policy/
│   ├── engine.rs        ← policy::enforce(action, context)
│   ├── rules.rs
│   └── security.rs
├── runtime/
│   ├── state.rs         ← all writes go through here
│   ├── paths.rs         ← path resolution
│   ├── lock.rs          ← prevent concurrent runs
│   ├── cache.rs
│   └── migrations.rs    ← schema version handling
├── adapters/
│   ├── pacman.rs        ← I/O only, no business logic
│   ├── systemd.rs
│   ├── sway.rs
│   ├── git.rs
│   └── filesystem.rs
├── capabilities/
│   ├── model.rs         ← capability enum definitions
│   └── check.rs         ← enforcement before adapter use
├── errors/
├── logging/             ← structured JSONL only, no println in domains
└── utils/
```

### Layer Rules (Enforced by Rust Module System)
- `cli/` imports only `app/` — never adapters or domains directly
- `domains/` never call each other — only through `app/dispatcher`
- `domains/` never execute shell commands — request capabilities instead
- `adapters/` contain zero business logic — I/O translation only
- `registry/` is immutable after load — no mutation allowed
- All writes go through `runtime/state.rs`

### Capability Model
```rust
// Each domain subcommand declares what it needs
#[capabilities(Capability::FilesystemWriteRuntime, Capability::QueryPacman)]
fn cmd_security_audit(ctx: &AppContext) -> Result<()> { ... }

// Enforced before execution
capabilities::check(&required, &ctx.capabilities)?;
```

Capabilities taxonomy:
- `FilesystemReadConfig` — read registry/policy
- `FilesystemWriteRuntime` — write to runtime/
- `QueryPacman` — read package database
- `ExecutePacman` — install/remove packages
- `ControlSystemdUser` — manage user services
- `ControlSway` — send swaymsg commands
- `NetworkQuery` — outbound network access
- `ElevatedPrivilege` — core-admin only

### Privilege Separation
- `core` — unprivileged, all normal operations
- `core-admin` — separate binary, explicit sudo scope only

---

## Safe Migration Plan (v1 → v2)

### Phase 0 — Freeze and Tag v1 ✅
- [x] `git tag v1.0.0-stable` — guaranteed rollback point
- [x] Stop adding new faelight-* tools
- [x] Document all v1 tool → v2 domain mappings

### Phase 1 — Scaffold core Skeleton ✅
- [x] Create `core/` crate with CLI skeleton
- [x] Stub commands: `core version`, `core doctor`
- [x] Install alongside v1 — no removal yet
- [x] Both systems run side by side

### Phase 2 — Wrap Existing Tools
Each v1 script becomes a thin wrapper:
```sh
#!/bin/sh
exec core <domain> <action> "$@"
```
Zero user-facing change. v1 tools delegate to v2 internally.

### Phase 3 — Migrate Domains Incrementally
Order (lowest risk first):
1. `intent` — pure file operations, no system calls
2. `link` — replaces faelight-link
3. `zone` — replaces faelight-zone
4. `profile` — replaces profile tool
5. `security` — replaces security-audit
6. `sandbox` — replaces faelight-sandbox
7. `update` — replaces faelight-update (highest risk, last)
8. `doctor` — replaces dot-doctor (validates everything else)

Each domain: migrate → test parity → remove old binary → keep wrapper.

### Phase 4 — Isolate Runtime
- [ ] Move logs/ → runtime/logs/
- [ ] Move cache/ → runtime/cache/
- [ ] Add runtime path manager
- [ ] Add locking (prevent concurrent core runs)
- [ ] Add runtime/VERSION migration file

### Phase 5 — Remove Script Layer
- [ ] Delete all shell logic
- [ ] Replace scripts with symlinks or remove entirely
- [ ] Only `core` and `core-admin` remain

### Phase 6 — Enforce Capability Model
- [ ] Capability declarations in each domain
- [ ] Policy checks before every adapter call
- [ ] Capability usage logged to runtime/logs/capabilities.jsonl

### Runtime Migration
```rust
// runtime/migrations.rs
pub fn migrate(current_version: u32) -> Result<()> {
    if current_version < 2 {
        migrate_v1_to_v2()?;
    }
    Ok(())
}
```
Runs automatically on first v2 invocation. Zero manual steps.

### Pre-flight Check
```
core doctor --preflight
```
Before enabling v2 fully — validates registry integrity, runtime paths,
adapter availability, policy conflicts. No execution side-effects.

---

## v1 → v2 Tool Mapping

| v1 Binary | v2 Command |
|---|---|
| dot-doctor | core doctor run |
| faelight-update | core update safe |
| security-audit | core security audit |
| faelight-sandbox | core sandbox run |
| faelight-link | core link status |
| faelight-zone | core zone check |
| profile | core profile switch |
| intent | core intent new |
| alias-audit | core doctor aliases |
| entropy-check | core doctor drift |
| core-protect | core lock / core unlock |
| faelight-fetch | core fetch |
| faelight-git | core git risk |


---

## Architecture Decisions (Locked)

These are settled. Do not revisit without a new intent.

| Decision | Choice | Rationale |
|---|---|---|
| Source directory | `engine/` | Avoids 0-core/core/ confusion. Clear separation: forest / engine / command |
| Binary name | `core` | Short, intentional, matches philosophy |
| Repo location | Inside 0-core/ | One repo, one system |
| Old tool names | Symlinks during transition, removed at cutover | Zero user disruption |
| State storage | SQLite (`runtime/state.db`) | Queryable, typed, single file |
| Doctor access | `OrchestratorAccess` — special capability | Only domain allowed to query all others |

## Final Layout
```
0-core/
  engine/           ← Rust source → binary: core
    src/
      domains/      ← 15 domains, strict boundaries
      cli/
      app/
      registry/
      policy/
      runtime/
      adapters/
      capabilities/
      errors/
      logging/
      utils/
      main.rs
    Cargo.toml
  registry/         ← 4 TOML files, zero logic
  policy/           ← constraints only, no execution
  adapters/         ← thin I/O translation
  runtime/          ← gitignored, all mutable state
    state.db
    logs/
    cache/
    snapshots/
    locks/
  docs/
  intents/
  Cargo.toml
  README.md
  VERSION
```

## 15 Domains

| Domain | Owns | Replaces |
|---|---|---|
| intent | ledger, files, status | intent, intent-guard |
| profile | switching, env vars | profile, dotctl |
| security | CVE scanning, permissions, SSH | security-audit |
| sandbox | isolation, snapshots, diffs | faelight-sandbox |
| link | stow verification, symlinks | faelight-link |
| zone | boundaries, write policy | faelight-zone |
| update | safe updates, cargo | faelight-update, safe-update |
| doctor | health checks (OrchestratorAccess) | dot-doctor, alias-audit, entropy-check, bin-doctor |
| fetch | system info display | faelight-fetch |
| git | workflow, risk scoring | faelight-git |
| workspace | file nav, recent files | faelight-fm, recent-files, workspace-view |
| release | versioning, changelog | bump-system-version, bump-tool-version, get-version |
| notify | notifications | faelight-notify |
| lock | screen locking | faelight-lock |
| launcher | app launching, palette | faelight-launcher (deprecated), faelight-palette (active), faelight-dmenu (deprecated) |

## SQLite Schema
```sql
CREATE TABLE domain_state (
    domain      TEXT NOT NULL,
    key         TEXT NOT NULL,
    value       TEXT NOT NULL,  -- JSON
    updated_at  INTEGER NOT NULL,
    PRIMARY KEY (domain, key)
);

CREATE TABLE events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    domain      TEXT NOT NULL,
    action      TEXT NOT NULL,
    payload     TEXT,
    timestamp   INTEGER NOT NULL
);

CREATE TABLE capabilities_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    domain      TEXT NOT NULL,
    capability  TEXT NOT NULL,
    granted     INTEGER NOT NULL,
    timestamp   INTEGER NOT NULL
);
```

## Capability Taxonomy

| Capability | Allows | Notes |
|---|---|---|
| FilesystemReadConfig | Read registry/, policy/ | All domains |
| FilesystemReadHome | Read anywhere in ~/ | Declared per domain |
| FilesystemWriteRuntime | Write to runtime/ only | Normal operations |
| FilesystemWriteHome | Write outside runtime/ | High — requires justification |
| QueryPacman | Read package database | update, security, doctor |
| ExecutePacman | Install/remove packages | core-admin only |
| ControlSystemdUser | Manage user services | update, doctor |
| ControlSway | Send swaymsg commands | launcher, lock |
| NetworkQuery | Outbound network | security, update |
| SpawnProcess | Execute subprocesses | sandbox |
| ElevatedPrivilege | Anything needing sudo | core-admin only |
| OrchestratorAccess | Query all domains | doctor only |

