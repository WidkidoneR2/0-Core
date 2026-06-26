---
id: 094
date: 2026-06-26
type: feature
status: planned
title: "faelight-deadwood: forest-native dead-code & orphan detector"
tags: [tool, dead-code, orphans, health, nix, registry, intents, forest-native]
priority: low
---
## Why
deadnix (astro/deadnix) finds Nix-level dead BINDINGS (unused let/lambda/inherit) -- a solved,
generic, parser-level problem. It is ALIASED, not rebuilt (INT-097 pattern: upstream tools that
do a generic job are aliases). But deadnix cannot see FOREST-level orphans -- dead things that
only a tool which understands the forest's own structure can find: dead aliases, registry entries
with no binary, modules imported by nothing, stale .bak files, orphaned scripts/intents. "The
forest remembers" should include knowing what it has forgotten to remove. (Christian's idea, 2026-06-26.)

## Core principle -- CONSERVATIVE (the cardinal rule)
REPORTS, never auto-deletes. The cardinal sin is a false positive that removes LIVE code.
- Default: flag for human review, with a confidence level per finding.
- Whitelist + a `# deadwood: skip` pragma for intentional-but-indirect references (a script
  called dynamically, an intent referenced only in prose, etc.).
- NEVER an --edit/--fix that auto-removes (unlike deadnix). Human decides every cut.
"Know what's dead before you cut -- never cut what only looks dead."

## Scope (phased: high-value/easy graph -> harder graph)
Phase 1 -- the easy wins (logic already exists in the forest):
  - Dead ALIASES: alias in config.fsh whose command resolves to no binary/builtin/PATH entry.
    Reuses is_known_command() from the highlighter (INT-092). Would have caught the 4 dead
    browser aliases removed by hand on 2026-06-26.
  - Stale .bak files: the timestamped backups edit-scripts generate constantly. Age-flag old
    ones for cleanup (report, don't delete).
  - Dead KEYBINDS: mango config.conf bind -> nonexistent command.
Phase 2 -- registry / scripts:
  - Registry entries (command_registry / tool registry) with no deployed binary.
  - Orphaned scripts in pkgs/faelight/scripts/ that nothing references.
Phase 3 -- the harder graph analysis:
  - Nix MODULES (modules/**/*.nix) imported by no host configuration (import-graph walk).
  - Orphaned INTENTS: future/ intents referenced nowhere; complete/ intents with dangling
    cross-references (the ghost-INT-260 class of problem).
  - Arch-era stragglers (e.g. old high intent numbers 202/275/276 in complete/) flagged for review.
Phase 4 -- integration:
  - A "Deadwood" check in the health dashboard (d), surfacing orphan accumulation over time,
    like the other health checks. Forest-built -> registry-worthy (NOT an alias).

## Reuses (don't duplicate)
- is_known_command() (faelight-shell highlighter, INT-092) for alias/keybind liveness.
- registry reads (cheatsheet refresh logic, INT-092).
- intent-integrity engine (for the intent-orphan checks).

## Relationship to deadnix
deadnix = aliased upstream, Nix dead-bindings (file/parser level). deadwood = forest-built,
system-structure orphans (forest-graph level). Complementary, different layers. Run deadnix
for binding hygiene; run deadwood for forest hygiene.

## The Rule
"A healthy forest sheds dead wood. Know what's dead before you cut --
 and never cut what only looks dead." 🌲
