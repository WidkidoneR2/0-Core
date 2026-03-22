---
id: 145
date: 2026-03-21
type: future
title: "faelight-docs — Living Documentation Engine"
status: in-progress
tags: [docs, readme, welcome, automation, core, v11, v12]
version: 11.2.0
priority: medium
---

## The Problem

Documentation lags behind reality. Every release:
- zshrc welcome message needs manual update
- README static section needs manual rewrite
- TOOLS.md drifts from actual tool count
- The forest knows everything but doesn't write it down automatically

This is a solvable problem. The forest already has all the data.

## The Boundary Rule — NON-NEGOTIABLE
```
faelight-release owns README lines 1-37 (dynamic section)
faelight-docs owns README lines 38+ (static section)
These two tools NEVER cross this boundary.
```

faelight-docs reads the marker:
```
<!-- END DYNAMIC SECTION -->
```
and only writes below it. It never touches above it. Ever.

## Commands
```bash
faelight-docs sync       # update all docs from forest state
faelight-docs check      # show what is out of date (dry run)
faelight-docs welcome    # regenerate zshrc welcome message only
faelight-docs readme     # regenerate README static section only
faelight-docs preview    # show what would change without writing
faelight-docs status     # what docs exist, last updated, sync status
```

## Data Sources

All data already exists — no new infrastructure needed:

| Data | Source |
|------|--------|
| Version + theme | `00-meta/VERSION` + `00-meta/CHANGELOG.md` |
| Tool count | `scripts/` directory count |
| Tool domains | `01-registry/tools.toml` |
| Intent counts | `intents/complete/` + `intents/future/` |
| Health | `runtime/state.db` events table |
| Commits | git rev-list count |
| Core domains | `engine/src/domains/` directory |
| Shell phases | `intents/future/120-faelight-shell.md` gate check |
| Recent features | `00-meta/CHANGELOG.md` latest entry |

## What It Writes

### 1. zshrc welcome message
One line in `03-interfaces/stow/shell-zsh/.zshrc`:
```bash
echo "🌲 Welcome to Faelight Forest vX.X.X — Theme Name"
```
Reads VERSION and CHANGELOG for the theme name. Updates in place.

### 2. README static section (lines 38+)
Regenerates the entire static documentation section:
- Tool count (from scripts/)
- Core domain count (from engine/src/domains/)
- Intent counts (from intents/)
- Core intelligence timeline (from CHANGELOG + intents)
- Shell phase completion status (from INT-120 gate check)
- Tool ecosystem table (from tools.toml)
- Journey table (from CHANGELOG)

### 3. TOOLS.md (optional)
Auto-generated tool reference from tools.toml.
Never conflicts with README.

## Architecture
```
rust-tools/faelight-docs/
├── Cargo.toml
└── src/
    main.rs       — CLI dispatch
    sources.rs    — read forest state (version, tools, intents, etc.)
    templates.rs  — README section templates
    welcome.rs    — zshrc welcome line updater
    readme.rs     — README static section writer
    check.rs      — diff current vs generated (dry run)
```

## Integration with faelight-release

faelight-release calls `faelight-docs sync` as a post-publish step:
```
faelight-release publish X.X.X
  → bumps VERSION
  → updates dynamic README section (lines 1-37)
  → calls faelight-docs sync  ← NEW
  → commits everything together
  → pushes
```

This makes every release self-documenting automatically.
Zero manual README updates ever again.

## The Boundary Enforcement
```rust
// In readme.rs — enforced in code, not just convention
const BOUNDARY_MARKER: &str = "<!-- END DYNAMIC SECTION -->";

fn find_static_start(content: &str) -> Option<usize> {
    content.find(BOUNDARY_MARKER)
        .map(|pos| pos + BOUNDARY_MARKER.len())
}

// ONLY writes from this position onwards
// If marker not found — abort, never guess
```

## Gate Check
```
⬜ faelight-docs sync — updates README static section
⬜ faelight-docs welcome — updates zshrc welcome message
⬜ faelight-docs check — dry run diff
⬜ faelight-docs preview — show generated output
⬜ faelight-release integration — auto-sync on publish
⬜ TOOLS.md generation
```

## The Phrase

**"A forest that documents itself
never loses its history.
Every version is a chapter
the forest writes on its own."**
