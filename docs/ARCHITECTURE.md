# 0-Core Architecture
> **Philosophy:** Understanding over convenience. Manual control over automation.
> **Version:** 0-Core v2 — single orchestrator, five layers, zero ambiguity.

This document explains the complete structure of Faelight Forest and how each component interacts.

---

## Layer Model
```
LAYER 0 — Substrate (Untouched)
  Kernel, systemd, Wayland, Niri, Network, Filesystem
  Treated as external environment. Never owned. Never modified directly.

LAYER 1 — Core Engine (Single Binary)
  engine/               ← Rust source
  Binary: core
  Interface: core <domain> <command> [flags]
  56+ domains: intent, profile, security, doctor, friday, friday_arch,
              genealogy, knowledge, predict, deploy, git, release, and more.
              Run: core --help for full domain list

LAYER 2 — Declarative Registry (Zero Logic)
  registry/
    packages.toml       ← system packages under management
    profiles.toml       ← profile definitions
    zones.toml          ← zone boundaries and permissions
    aliases.toml        ← shell alias declarations
  Rule: If it contains an if statement, it does not belong here.

LAYER 3 — Policy (What Is Allowed)
  policy/               ← constraints only, no execution
  docs/                 ← human-readable documentation
  Rule: Policy defines constraints. It never executes.

LAYER 4 — Runtime (All Mutable State)
  runtime/              ← gitignored entirely
    logs/               ← structured JSONL logs by domain
    cache/              ← precomputed indices
    snapshots/          ← sandbox and rollback state
    state.db            ← single SQLite state database
    locks/              ← operation locks
  Rule: rm -rf runtime/ is always safe. Full reset, no data loss.

LAYER 5 — Adapters (Thin Translation Only)
  03-interfaces/stow/   ← dotfile packages (GNU Stow managed)
  adapters/             ← systemd, niri config generation
  Rule: No business logic. Only translation between core and external systems.
```

---

## Directory Structure
```
0-core/
  engine/               ← Core v2 Rust source → binary: core
    src/
      domains/          ← 15 domains, strict layer boundaries
      cli/              ← command grammar + clap parser
      app/              ← dispatcher + AppContext
      registry/         ← TOML loader + schema validation
      policy/           ← constraint enforcement
      runtime/          ← state, locks, migrations
      adapters/         ← I/O translation only
      capabilities/     ← capability model
      errors/
      logging/
      utils/
    Cargo.toml
  rust-tools/           ← Specialist TUI tools (not replaceable by CLI)
    faelight-bar/       ← Custom Wayland status bar
    faelight-fm/        ← File manager with zone awareness
    faelight-palette/   ← App launcher (replaces launcher + dmenu)
    faelight-git/       ← Git workflow governance TUI
    faelight-update/    ← Interactive update manager TUI
    faelight-term/      ← Terminal emulator (WIP)
    faelight-browser/   ← TUI browser (WIP)
    faelight-core/      ← Shared library (config, paths, health)
    [51 total tools]
  03-interfaces/
    stow/               ← ALL dotfile packages (GNU Stow managed)
      niri/             ← Niri compositor config
      shell-zsh/        ← Zsh + 318+ aliases
      editor-nvim/      ← Neovim + Faelight theme
      term-foot/        ← Foot terminal emulator
      config-faelight/  ← Typed TOML configs for Rust tools
      [6 more packages]
  registry/             ← Zero-logic TOML declarations
  policy/               ← Security rules, health check definitions
  runtime/              ← Gitignored. All mutable state lives here.
  scripts/              ← Compiled binaries + thin shell wrappers
  intents/              ← Intent ledger (markdown files)
  docs/                 ← Human documentation
  00-meta/              ← Version, changelog, philosophy
  VERSION               ← 0-Core v2 engine version
  Cargo.toml            ← Workspace root
  README.md             ← GitHub readme
```

---

## Core Engine Domains

| Domain    | Owns                              | Replaces                          |
|-----------|-----------------------------------|-----------------------------------|
| intent    | ledger, files, status             | intent, intent-guard (partially)  |
| profile   | switching, env vars               | profile                           |
| security  | CVE scanning, permissions, SSH    | security-audit                    |
| sandbox   | isolation, snapshots, diffs       | faelight-sandbox                  |
| link      | stow verification, symlinks       | faelight-link                     |
| zone      | boundaries, write policy          | faelight-zone                     |
| update    | safe updates, cargo               | faelight-update (delegates TUI)   |
| doctor    | health checks (OrchestratorAccess)| dot-doctor, alias-audit           |
| fetch     | system info display               | faelight-fetch                    |
| git       | workflow, risk scoring            | faelight-git (delegates TUI)      |
| workspace | file nav, recent files            | faelight-fm (delegates TUI)       |
| release   | versioning, changelog             | bump-system-version, get-version  |
| notify    | notifications                     | faelight-notify                   |
| lock      | screen locking                    | faelight-lock                     |
| launcher  | app launching                     | faelight-palette                  |
| friday    | intelligence layer                | cross-domain intelligence         |
| genealogy | intent family tree                | core genealogy tree/show/roots    |
| predict   | prediction engine                 | pattern-based anticipation        |
| ...       | 56+ domains total                 | run: core --help for full list    |

---

## Key Design Principles

**Single binary surface** — one `core` binary, subcommands for domains.
All user-facing operations: `core <domain> <command> [flags]`

**Strict layer boundaries** — domains never call each other directly.
All cross-domain communication goes through `app/dispatcher`.

**All mutable state isolated** — nothing outside `runtime/` changes at runtime.
`rm -rf runtime/` is always a safe full reset.

**Declarative over imperative** — registry contains zero logic, only truth.

**TUI tools stay separate** — faelight-fm, faelight-palette, faelight-bar, etc.
are specialist tools too rich to wrap in a CLI. They delegate through `core` where possible.

---

## Health System

Health is a single source of truth across all tools:
```
core doctor run → writes ~/.cache/faelight/health-status
                ↓
    faelight-bar reads cache (fast, no subprocess)
    faelight-palette reads cache (consistent with bar)
    prompt-health-dot reads cache (shell prompt dot)
```

Run `d` (alias for `doctor`) to update health across all tools.

---

## Stow Package Deployment

All dotfiles are managed as GNU Stow packages:
```bash
cd ~/0-core/03-interfaces/stow
stow package-name          # deploy
stow -D package-name       # undeploy
```

Or use the native implementation:
```bash
core link deploy           # deploy all packages
core link adopt            # convert existing files to symlinks
core link plan             # preview deployment
core link status           # check symlink health
```

---

## Build System
```bash
# Build entire workspace
cd ~/0-core && cargo build --release --workspace

# Build specific tool
cargo build --release -p faelight-fm

# Binaries land in
~/0-core/target/release/<binary>
~/0-core/scripts/<binary>   ← deployed copies
```

Cold start: **3ms** (core binary, measured 2026-02-22)
