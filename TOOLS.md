# 🌲 0-Core Tools
> Last Updated: 2026-02-22 (v10.1.0)

## Core Orchestrator

| Tool | Version | Purpose |
|------|---------|---------|
| **core** | v2.0.0 | Single orchestrator binary — 15 native Rust domains |

### Domains
`doctor` `security` `git` `workspace` `intent` `profile` `zone` `link` `fetch` `lock` `notify` `launcher` `sandbox` `release` `update`

---

## UI Tools

| Tool | Version | Status | Purpose |
|------|---------|--------|---------|
| **faelight-bar** | v4.0.0 | ✅ Production | Custom Wayland status bar |
| **faelight-palette** | v3.0.0 | ✅ Production | App launcher + 0-Core stats |
| **faelight-menu** | v4.0.0 | ✅ Production | Power menu — forest palette aesthetic |
| **faelight-fm** | v2.2.0 | ✅ Production | Semantic file manager with zone/git/intent metadata |
| **faelight-term** | v0.3.0 | 🔄 WIP | Terminal emulator |
| **faelight-browser** | v0.1.0 | 🔄 WIP | TUI browser |

---

## Git Tools

| Tool | Version | Status | Purpose |
|------|---------|--------|---------|
| **faelight-git** | v3.2.0 | ✅ Production | Risk-aware git workflow TUI |
| **faelight-hooks** | v10.1.0 | ✅ Production | Git hooks — rustfmt, clippy, secrets, conflicts |

---

## System Tools

| Tool | Version | Status | Purpose |
|------|---------|--------|---------|
| **faelight-update** | v3.1.0 | ✅ Production | Interactive system update manager |
| **core-protect** | v2.0.0 | ✅ Production | Immutable flag on ~/0-core |
| **safe-update** | v2.0.0 | ✅ Production | Safe package update wrapper |
| **faelight-sandbox** | v2.0.0 | ✅ Production | Controlled experimentation environment |
| **faelight-snapshot** | v2.0.0 | ✅ Production | Btrfs snapshot management |
| **faelight-lock** | v2.1.0 | ✅ Production | Screen locker |
| **faelight-notify** | v2.0.0 | ✅ Production | Notification daemon |

---

## Development Tools

| Tool | Version | Status | Purpose |
|------|---------|--------|---------|
| **bump-system-version** | v9.2.0 | ✅ Production | Release automation |
| **bump-tool-version** | v2.0.0 | ✅ Production | Individual tool versioning |
| **get-version** | v4.0.0 | ✅ Production | Version queries |
| **core-diff** | v2.0.0 | ✅ Production | Policy-mode diffs |
| **latest-update** | v4.0.0 | ✅ Production | Last update info |

---

## Shell & Navigation

| Tool | Version | Status | Purpose |
|------|---------|--------|---------|
| **dotctl** | v3.1.0 | ✅ Production | Zone-aware package management |
| **profile** | v2.1.0 | ✅ Production | Profile switching |
| **intent** | v3.0.0 | ✅ Production | Intent ledger CLI |
| **faelight-zone** | v2.1.0 | ✅ Production | Zone detection and boundaries |
| **faelight-link** | v3.0.0 | ✅ Production | Symlink management — Stow replacement |
| **faelight-fetch** | v2.1.0 | ✅ Production | Zone-aware system info display |
| **workspace-view** | v2.0.0 | ✅ Production | File navigation and recent files |

---

## Audit Tools

> Note: These are also natively absorbed into `core doctor` — standalone binaries kept for direct use.

| Tool | Version | Status | Purpose |
|------|---------|--------|---------|
| **alias-audit** | v9.1.0 | ✅ Production | Alias coverage and conflict detection |
| **bin-doctor** | v2.0.0 | ✅ Production | Binary manifest and drift detection |
| **entropy-check** | v2.0.0 | ✅ Production | Configuration drift detection |
| **archaeology-0-core** | v3.0.0 | ✅ Production | Git history analysis |

---

## Bootstrap & Utility

| Tool | Version | Status | Purpose |
|------|---------|--------|---------|
| **faelight-bootstrap** | v2.0.0 | ✅ Production | One-command system setup |
| **faelight-daemon** | v1.0.0 | ✅ Production | Background service manager |
| **faelight-cleanup** | v1.0.0 | ✅ Production | System cleanup utility |
| **keyscan** | v3.0.0 | ✅ Production | Keybinding scanner |
| **teach** | v3.0.0 | ✅ Production | Interactive learning system |
| **intent-guard** | v2.0.0 | ✅ Production | Command safety enforcement |
| **verify-bootstrap** | v2.0.0 | ✅ Production | Installation verification |

---

## Statistics

| Metric | Value |
|--------|-------|
| Total tools | 34 |
| Production-ready | 32 (94%) |
| WIP | 2 (faelight-term, faelight-browser) |
| Core domains | 15 |
| Aliases | 318 |
| Health checks | 22 |
| Cold start | 3ms |

---

## Absorbed into Core

These tools exist as standalone binaries but are also natively implemented inside `core`:

| Standalone | Core Command |
|-----------|-------------|
| `alias-audit` | `core doctor aliases` |
| `bin-doctor` | `core doctor bins` |
| `entropy-check` | `core doctor entropy` |

