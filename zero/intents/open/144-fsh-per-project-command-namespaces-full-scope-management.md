---
id: 144
date: 2026-07-11
type: future
title: "fsh per-project scope system: command namespaces + project configuration"
status: planned
tags: [fsh, namespaces, scope, aliases, projects]
---

## Vision
Per-project isolated command namespaces: when you are "in" a project scope, that project's
aliases/commands are available and isolated from other projects -- project A's `deploy` is not
project B's. Being in a scope changes not just your directory but your command vocabulary.

## The Problem
Aliases (`shell_aliases`, db.rs:58: name TEXT PRIMARY KEY, command, created) are GLOBAL --
`get_alias(name)` (db.rs:169) resolves by name only, no scope. Project CONTEXT exists via
`fsh enter <project>` (INT-322 Phase 7, commands/mod.rs:11318 -- sets shell_state `scope_name`,
saves return path/intent, cd's in; `fsh leave` restores), and `project`/`projects` list
(commands/mod.rs:386). But being in a scope changes cwd/context only -- every project shares
one global alias set. There is NO command-namespace isolation today.

## The Solution
Make aliases scope-aware, in two layers:

### Layer 1 -- essential resolution core
- A `scope` column on shell_aliases (empty string = global alias).
- Scope-aware resolution in get_alias: when a `scope_name` is active (from fsh enter), an alias
  matching (name, current-scope) wins; else fall back to the global (empty-scope) alias.
- Scoped alias definition: defining an alias while in a scope tags it to that scope.
- This is the minimal-but-real core that delivers genuine isolation (project A's alias != B's).

### Layer 2 -- scope-management surface
1. Scoped-alias management: `alias --scope <name>` explicit targeting; scope-aware `unalias`
   (remove scoped, or `--global`); move/copy an alias between scopes.
2. Scope-aware listing: alias list shows + filters by scope; a `scope` status readout (what
   scope am I in, how many scoped aliases).
3. Project alias manifests: on `fsh enter`, optionally auto-load a project-local alias manifest
   (`.fsh-aliases` or an `[aliases]` block) so a project's command vocabulary is declarative and
   committable -- reuses INT-134's env-export/import manifest pattern.
4. (Evaluate) scoped commands/functions beyond aliases -- adopt or defer against the fsh filter
   (scoped vocabulary is fine; opaque per-dir auto-magic stays cut).

### Layer 3 -- project-specific shell configuration (folded in from roadmap line 75)
Distinct from namespaces (vocabulary) -- this is per-project SETTINGS/behavior:
- A per-project config store keyed by scope_name (prompt style, default editor, theme, project
  env defaults). No mechanism exists today (confirmed: grep for project-config found nothing).
- Loaded/applied on `fsh enter <project>`, restored on `fsh leave`.
- Same architecture as the alias manifest: scope-keyed state loaded on enter from a committable
  project manifest -- so config + aliases share ONE manifest + ONE load path, not two.
- Command: `project-config set/get/list` (or a `[config]` block in the project manifest).
- Why folded here: project config and project namespaces share the exact foundation (scope-keyed
  state on fsh enter from a committable manifest). Building separately = duplicate machinery.

## Success Criteria
- [ ] `scope` column on shell_aliases; scope-aware get_alias (current-scope wins, global fallback) -- demonstrated
- [ ] scoped alias definition (define in a scope -> tagged to it); resolves in-scope, absent/global-fallback outside -- demonstrated live
- [ ] `alias --scope` explicit targeting + scope-aware `unalias`
- [ ] alias listing shows + filters by scope; a `scope` status readout
- [ ] project alias manifest auto-loaded on `fsh enter` (declarative, committable)
- [ ] scoped-vs-global precedence documented and demonstrated
- [ ] (evaluate) scoped commands beyond aliases -- adopt or defer with reason
- [ ] per-project config store keyed by scope_name; `project-config set/get/list`
- [ ] project config loaded on `fsh enter`, restored on `fsh leave` -- demonstrated live
- [ ] config + aliases share one project manifest + one load path (no duplicate machinery)

## Relationship
- Builds on: INT-322 Phase 7 (fsh enter/leave scope, shell_state `scope_name`).
- Reuses: INT-134's env-export/import TOML manifest pattern for the project alias manifest idea.
- Filter: scoped vocabulary deepens understanding + reproducible control (a project declares its
  commands); opaque auto-magic stays cut.

## Notes
Owns two adjacent roadmap items, both split out of INT-134 (fsh Evolution Roadmap) and both left
UNCHECKED deliberately so they complete + check off with integrity here:
- "Per-project isolated command namespaces" (roadmap line 72) -- Layers 1 & 2
- "Project-specific shell configuration" (roadmap line 75) -- Layer 3
They share one foundation (scope-keyed state loaded on fsh enter from a committable manifest), so
they are one intent, not two. NOTHING built yet. Each roadmap checkbox is checked ONLY when this
intent completes.
