---
id: 260
date: 2026-05-01
type: feature
title: "Cheatsheet TUI -- Live Reference for Commands and Keybinds"
status: planned
tags: [feature, rust, faelight, tui, ratatui, cheatsheet, reference, commands, keybinds, fsh, ux]
version: TBD
---

## Vision

`Ctrl+/` from anywhere in fsh opens a ratatui-based cheatsheet TUI: a
fuzzy-searchable, live reference for every command, keybind, alias, and
tool the forest knows about. Read directly from the command registry
(INT-259) — never curated, never stale.

The pattern proven by INT-250 (Ctrl+R), INT-253 (gt), INT-254 (it), and
INT-258 (Ctrl+D health) extended one more time. Function key opens
focused TUI, returns cleanly to fsh prompt, shares the forest's color
palette and tone.

This is NOT a tutorial, NOT a help system that explains philosophy, and
NOT auto-categorization-magic. It is a reference — a place to ask "how
do I do X?" and get the answer in three seconds.

The keybind is `Ctrl+/` (the `/` key, which produces `?` when shifted).
Reasoning: `?` is the universal symbol for a question, which is exactly
what this TUI answers. Ergonomic to press as Ctrl+/ (no shift needed),
symbolic as Ctrl+?.

## Why Now

1. **The registry foundation will exist.** INT-259 ships first; this
   intent is its first real consumer. Without the registry, this TUI
   would have to parse 6+ source files at runtime, which is exactly
   the brittle work the registry exists to eliminate.

2. **The ratatui UX language is mature.** Five intents (250, 253, 254,
   258, this one) all share architectural pattern: ConditionalEventHandler
   intercept, ratatui rendering, crossterm event handling, clean exit
   back to fsh. Each new TUI is faster to build than the last because
   the pattern is proven.

3. **Real friction surface.** With 359 aliases, 114 niri keybinds, ~30
   fsh builtins, ~50 tools, and growing — Christian cannot hold all of
   this in working memory. A fast lookup is a daily quality-of-life win.

4. **Replaces redundant tools.** When this TUI ships and proves itself
   in real daily use, fsh's `which`, `type`, `explain`, `debug` builtins
   become candidates for retirement (separate intents — built only after
   the TUI has earned its keep).

## Approach

### Invocation

- `Ctrl+/` from fsh REPL line edit -> opens TUI (ConditionalEventHandler)
- `cheat` from fsh prompt -> opens TUI (alias path for command-line use)
- TUI exits cleanly back to fsh prompt with nothing changed

### Layout (initial)

**Default view:**
┌─ 🌲 Forest Cheatsheet ──────────────────────────── 753 entries ───┐
│                                                                    │
│  Search: [                                                       ] │
│                                                                    │
│  ▸ Niri (114)            window/workspace keybinds                 │
│  ▸ Forest Workflow (47)  cistart/cicomplete, gc/gp/dep, lock-core  │
│  ▸ fsh Builtins (28)     query, fsearch, patch, edit, friday       │
│  ▸ Aliases (359)         all forest aliases                        │
│  ▸ Tools (50)            faelight-shell, faelight-term, ...        │
│  ▸ Faelight-Term (24)    Ctrl+Shift+F, Super+Enter, split panes    │
│  ▸ Git Workflow (15)     gc, gp, fg done, fg sync                  │
│  ▸ System (16)           lock-core, unlock-core, deploy, doctor    │
│                                                                    │
│  / search · Tab category · Enter expand · c copy · q quit          │
└────────────────────────────────────────────────────────────────────┘

**Search-active view (typing 'def'):**
┌─ 🌲 Forest Cheatsheet ──────────────────────── 4 matches ──────────┐
│                                                                    │
│  Search: [def                                                    ] │
│                                                                    │
│  ▸ default-niri      tool       reset niri to default config       │
│  ▸ defer-intent      alias      core intent → planning/            │
│  ▸ define-pattern    builtin    fsh: register a custom pattern     │
│  ▸ deploy            tool       deploy a tool from rust-tools/     │
│                                                                    │
│  Enter to expand, c to copy, Esc to clear search                   │
└────────────────────────────────────────────────────────────────────┘

**Expanded entry view (after pressing Enter on 'deploy'):**
┌─ 🌲 deploy ───────────────────────────────────────── tool ────────┐
│                                                                    │
│  Source:       scripts/deploy                                      │
│  Description:  Build and deploy a tool from rust-tools/ into       │
│                ~/.cargo/bin and the registry                       │
│                                                                    │
│  Usage:        deploy <tool>                                       │
│  Example:      deploy faelight-shell                               │
│                                                                    │
│  Related:      cargo build · cistart · cicomplete · core registry  │
│                                                                    │
│  Last seen:    2026-05-01 (today)                                  │
│                                                                    │
│  [c] copy command   [Esc] back to list   [q] quit                  │
└────────────────────────────────────────────────────────────────────┘

### Search

- Default focus is in the search bar
- Live fuzzy filtering as user types
- Searches across: name, description, source, expansion (for aliases),
  example, related entries
- Case-insensitive
- Results sorted by relevance (exact match > prefix > substring > fuzzy)

### Navigation

- Arrow keys move selection
- `Tab` cycles between category groupings
- `Enter` expands selected entry
- `Esc` clears search OR closes expanded view OR quits TUI (state-dependent)
- `q` always quits cleanly to fsh prompt
- `c` copies the selected entry's command/keybind to clipboard (wl-copy)

### Layout in compact mode

- Total height: ~12-15 lines + search bar — fits below standard
  terminal split
- Categories collapsed by default, count shown
- Tab to expand a category in place; entries appear with one-line
  summary each

### Tone consistency with other forest TUIs

- Forest emoji (🌲) in header
- Same color palette as INT-250 history TUI and INT-258 health TUI
- Same exit behavior (Esc/q returns cleanly)
- Same status-bar pattern at bottom (action keys with descriptions)

### Implementation modules (suggested)

- `rust-tools/faelight-shell/src/cheatsheet_tui/mod.rs` -- entry point
- `rust-tools/faelight-shell/src/cheatsheet_tui/state.rs` -- registry reader,
  search/filter logic
- `rust-tools/faelight-shell/src/cheatsheet_tui/render.rs` -- ratatui rendering
- `rust-tools/faelight-shell/src/cheatsheet_tui/fuzzy.rs` -- fuzzy match scoring

Or as standalone tool `rust-tools/cheat/` if scope grows or if it should
be callable independently of fsh.

## Hard Dependencies

- INT-259 (Command and Keybind Registry) — hard dependency, this TUI
  reads from it. INT-260 cannot ship before INT-259.
- ratatui 0.28 + crossterm 0.28 (already in fsh)
- ConditionalEventHandler pattern (proven in INT-250)
- wl-copy for clipboard (already in system, used by faelight-clipboard)

## Success Criteria

- [ ] `Ctrl+/` from fsh REPL opens a working cheatsheet TUI
- [ ] `cheat` from fsh prompt opens the TUI (alias path)
- [ ] TUI displays all entries from command_registry grouped by category
- [ ] Categories collapsible/expandable with Tab
- [ ] Live fuzzy search filters across name/description/source/expansion
- [ ] Search results ranked by relevance (exact > prefix > substring > fuzzy)
- [ ] Enter on a result shows full expanded entry view
- [ ] Expanded view shows: source, description, usage, example,
      related entries, last_seen timestamp
- [ ] [c] copies the selected entry's name/command to clipboard
- [ ] Esc/q returns cleanly to fsh prompt with no terminal artifacts
- [ ] Stale entries (last_seen >14 days) shown with dim color
- [ ] Forest color palette and tone consistent with other TUIs
- [ ] No regression in existing fsh builtins (which/type/explain/debug
      continue working unchanged during migration period)

## Scope

### In scope

- Ratatui TUI for browsing and searching the registry
- Ctrl+/ keybind via ConditionalEventHandler
- `cheat` command-line entry point
- Fuzzy search across all registry fields
- Category grouping with collapse/expand
- Expanded entry view with full detail
- Copy-to-clipboard for selected entries
- Stale entry indication

### Out of scope

- Modifying registry contents from the TUI (read-only consumer)
- Tutorial or learning-path content (this is reference, not teacher)
- Cross-host sharing
- Live registry refresh during open session (registry queried on TUI
  open; if user wants fresh data, they reopen — keeps logic simple)
- Auto-categorization beyond what the registry provides

### Deliberately deferred

- Voice readout of selected entry (Friday voice integration, separate)
- Linking entries to documentation pages (would require URL field in
  registry; not v1)
- Usage analytics ("you use this command 12x/week") — would need usage
  tracking infrastructure that doesn't exist yet
- Suggested-related-commands ML — registry's `related` field is curated
  by source publishers, not learned

## Tool Retirement Considerations (FUTURE work, not this intent)

After this TUI ships and is daily-driven for 2+ weeks, the following
fsh builtins become candidates for retirement (each requires its own
intent and proof that the TUI fully replaces the use case):

- `which` — answers "where does this command come from?" (registry has
  source field)
- `type` — answers "is this a builtin or external?" (registry has kind
  field)
- `explain` — answers "what does this command do?" (registry has
  description field)
- `debug` — answers "show me everything about this command" (registry
  expanded view shows all fields)

NONE of these retire as part of INT-260. They retire only after the
TUI has proven itself in real daily use, in separate per-tool retirement
intents.

## Gate Check
⬜ Not started

---

*"How do I do X? — should always have an answer in three seconds.
Not in six locations.
Not in stale documentation.
In one place that knows what every tool, alias, and keybind does
right now today."* 🌲
