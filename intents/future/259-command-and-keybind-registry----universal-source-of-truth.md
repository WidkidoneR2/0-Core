---
id: 259
date: 2026-05-01
type: arch
title: "Command and Keybind Registry -- Universal Source of Truth"
status: planned
tags: [arch, registry, infrastructure, commands, keybinds, single-source-of-truth, foundation]
version: TBD
---

## Vision

Today, the forest's commands and keybinds live in scattered locations:

- Niri keybinds: `~/.config/niri/config.kdl` (one parseable file)
- Aliases: state.db alias registry (queryable)
- Tools: state.db registry (queryable, but lacks per-command detail)
- fsh builtins: hardcoded in `rust-tools/faelight-shell/src/commands/mod.rs`
  (Rust source — would require parsing or extraction)
- faelight-term shortcuts: hardcoded in faelight-term source
- Workflow patterns (cistart/cicomplete, gc/gp/dep): documented in
  `docs/COMMAND-GUIDE.md` (curated markdown)
- Per-tool `--help` output: each tool's own implementation
- `which`, `type`, `explain`, `debug` fsh builtins: all partial answers
  to "what is this command and what does it do?"

The result: when a human (Christian) asks "what's the keybind for X?" or
"what tools handle Y?", the answer is scattered across at least 6
locations. There is no single source of truth.

This intent creates that source of truth — a unified, queryable registry
that tools publish to on build/deploy, and that consumers (TUIs, help
systems, Friday's documentation steward, future tools) read from.

This is NOT a TUI. This is the data layer underneath INT-260 (the
cheatsheet TUI) and underneath any future tool that needs to know
"what commands and keybinds exist in this system."

## Why Now

Three converging signals:

1. **Tool retirement requires a replacement substrate.** The 50-tools-to-25
   trajectory (v23 Pillar 3) needs infrastructure that absorbs scattered
   help-system functionality. `which`, `type`, `explain`, `debug` are
   four fsh builtins that all answer different facets of "what is this
   command?" A central registry collapses them into one source.

2. **INT-260 (cheatsheet TUI) needs this to exist.** Without a registry,
   the TUI either curates static markdown (goes stale immediately) or
   parses 6+ source files at runtime (fragile, slow, brittle). Neither
   serves the "live reference" goal.

3. **Friday v22 Pillar 1 (documentation steward) needs this too.** When
   Friday proposes doc updates after a commit, it needs to know what
   commands/keybinds the diff added or removed. A registry makes that
   query cheap. No registry = Friday has to grep source code, which is
   exactly the kind of brittle work this project is designed to avoid.

## Approach

### Schema (proposed)

A single SQLite table (`command_registry`) in state.db with rows for
every command, builtin, alias, keybind, and tool entry point in the
forest:
command_registry:
id           INTEGER PRIMARY KEY
kind         TEXT     -- 'command' | 'builtin' | 'alias' | 'keybind' | 'tool'
name         TEXT     -- the literal command/keybind ('gc', 'Mod+k', 'fsearch')
source       TEXT     -- which tool published this ('faelight-shell',
--   'niri', 'alias-registry', 'faelight-term')
category     TEXT     -- 'git', 'navigation', 'window-mgmt', 'forest-workflow', etc.
description  TEXT     -- one-line human description
expansion    TEXT     -- for aliases: what it expands to
args         TEXT     -- JSON array of accepted arguments/flags
example      TEXT     -- one example invocation
related      TEXT     -- comma-separated ids of related entries
added_at     INTEGER  -- timestamp when published
last_seen    INTEGER  -- timestamp last confirmed by source publisher
deprecated   INTEGER  -- 0 = active, 1 = deprecated (with reason in description)

Indexed on (kind, name) and (source, last_seen) for fast lookup and
staleness detection.

### Publishing

Each tool publishes its commands/keybinds to the registry on
build/deploy via a small library function:

```rust
faelight_core::registry::publish_command(RegistryEntry { ... })
faelight_core::registry::publish_batch(source, entries)
```

For non-Rust sources (niri config.kdl, alias-registry, COMMAND-GUIDE.md),
a small importer runs on appropriate hooks:

- niri: parse config.kdl on faelight-bootstrap or core integrity run
- aliases: already in alias-registry table; add a publishing pass
- COMMAND-GUIDE workflow patterns: parse markdown sections on docs deploy

### Reading

Consumers query the registry directly via SQL or a Rust API:

```rust
faelight_core::registry::all_kinds(kind: Kind) -> Vec<RegistryEntry>
faelight_core::registry::find(query: &str) -> Vec<RegistryEntry>
faelight_core::registry::by_source(source: &str) -> Vec<RegistryEntry>
faelight_core::registry::related_to(id: i64) -> Vec<RegistryEntry>
```

### Staleness detection

If a source hasn't published in N days (e.g. 14), entries from that
source get a "stale" flag. Consumers can choose to surface or hide
stale entries. This is the mechanism that prevents the registry from
silently rotting if a tool is retired or a config file moves.

### Migration plan (incremental)

Phase 1: schema + publishing API live; faelight-shell publishes its
builtins as proof-of-concept.

Phase 2: niri config.kdl importer; aliases re-publish on every alias
add/edit.

Phase 3: faelight-term, faelight-fm, faelight-git publish their
shortcuts and key commands.

Phase 4: tools registry table merges with command_registry (or links
cleanly); per-tool subcommands published.

Phase 5: COMMAND-GUIDE workflow patterns importer; fsh which/type/
explain/debug start consulting the registry instead of their own
hardcoded lists.

Each phase ships independently. No flag-day migration. Existing tools
keep working while their registry entries land.

## Hard Dependencies

- state.db (already exists)
- faelight-core registry primitives (extend, don't rebuild)
- Build/deploy hooks to trigger publishing (deploy script already
  exists; add a small post-deploy publish step)

## Success Criteria

- [ ] command_registry schema created in state.db with documented columns
- [ ] faelight_core::registry::publish_command + publish_batch APIs live
- [ ] faelight-shell publishes all its builtins on deploy
- [ ] Niri config.kdl keybinds imported into registry (114 entries)
- [ ] Aliases re-publish to registry on every alias add/edit
- [ ] At least 3 additional tools (faelight-term, faelight-git,
      faelight-fm) publish their commands/shortcuts
- [ ] Staleness detection live: entries unseen for >14 days flagged
- [ ] Consumer API: at least 4 query functions documented and working
- [ ] No regression in existing per-tool --help, alias resolution,
      or which/type/explain/debug behavior (registry coexists with
      current mechanisms during migration)

## Scope

### In scope
- The registry table and its schema
- Publishing API for Rust tools
- Importers for non-Rust sources (niri config, aliases, markdown)
- Consumer query API
- Staleness detection
- Migration of fsh builtins as proof-of-concept

### Out of scope
- The TUI consumer (INT-260)
- Tool retirement based on registry availability (separate intents
  per tool, written only after replacement is daily-driven)
- Cross-host sharing (single-host only)
- Permission/visibility controls (everything in the registry is
  public to every consumer)
- Auto-generated --help for tools (could be a future intent — tools
  generate --help from their published registry entries)

### Deliberately deferred
- Replacing fsh's which/type/explain/debug builtins entirely (these
  retire in a separate intent after the cheatsheet TUI proves itself)
- Per-tool registry-based --help generation (future work)
- Full COMMAND-GUIDE absorption (curated docs still serve narrative
  purposes the registry can't replace)

## Risks and Mitigations

### Risk 1: Registry drift (sources stop publishing)
**Mitigation**: Staleness detection flags this within 14 days. Doctor
check could verify all expected sources have published recently.

### Risk 2: Schema migration pain as new fields are needed
**Mitigation**: Start narrow. Add fields via ALTER TABLE migrations
when concrete needs emerge, not speculatively.

### Risk 3: Performance — large registry slows consumers
**Mitigation**: 359 aliases + 114 keybinds + ~50 tools + ~30 builtins
+ ~200 workflow patterns = ~750 rows max. SQLite handles this trivially.
Index on (kind, name).

### Risk 4: Coupling tools to faelight-core just to publish
**Mitigation**: Publishing is optional. Tools that don't publish still
work; they just don't appear in registry-driven consumers like the TUI.
Keeps the registry opt-in, not mandatory.

## Gate Check
⬜ Not started

---

*"Six locations, six answers, one human asking the question.
The answer should live in one place — and every tool should know
how to put its piece of it there."* 🌲
