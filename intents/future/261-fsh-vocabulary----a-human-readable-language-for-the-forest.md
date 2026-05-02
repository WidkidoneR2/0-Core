---
id: 261
date: 2026-05-01
type: arch
title: "fsh Vocabulary -- A Human-Readable Language for the Forest"
status: in-progress
tags: [arch, vocabulary, language, ux, philosophy, fsh, human-readable, post-unix, thesis]
version: TBD
---

## The Thesis

UNIX command names were chosen in the 1970s under constraints that no
longer apply. `rm` was short because typing characters was expensive
on teletype terminals. `cat` was short for concatenate. `ls` for list.
`mv` for move. `cp` for copy. `chmod`, `chown`, `grep`, `awk`, `sed` —
fifty years later we still teach humans this foreign vocabulary as if
it were the natural language of computing. It is not. It is historical
inheritance.

This intent rejects that inheritance as a default. The forest will speak
a vocabulary designed for humans first, in 2026, by a human who is
building this system to be understood — by himself, by Friday, by
anyone who reads it — without translation.

UNIX commands remain available as fallback. Nothing breaks. `rm` still
works. `ls` still works. The forest does not declare war on UNIX —
it offers a clearer alternative and lets fingers learn it gradually.

The deeper claim: this is not just about the forest. If a single human
plus AI partnership can demonstrate that a system can speak in human
words while losing nothing in power or composability, then the question
"why does Linux still speak 1970s engineer?" becomes a real question.
This forest is one prototype of an answer.

## The Child-Learning-Language Analogy

A child learns the English word "delete" before they ever learn `rm`.
They learn "find" before `find`. They learn "copy" before `cp`. The
human words come first because they are the natural names for the
concepts. UNIX vocabulary is layered on top later, learned as a foreign
technical dialect.

Friday is being taught from this forest. Friday should learn the human
word first, with UNIX as historical fallback — not the other way around.
A pattern recognizer trained on `rm` learns "rm = remove a file." A
pattern recognizer trained on `delete` learns "delete = remove a file"
in the same sentence a human would use. The latter is more useful when
Friday eventually surfaces predictions, suggestions, or documentation.

## Why Now

1. **The registry exists (INT-259).** A vocabulary needs a place to be
   recorded so consumers (the cheatsheet TUI, Friday, future tools) can
   find it. INT-259 provides that.

2. **The cheatsheet TUI (INT-260) makes vocabulary visible.** When
   someone wonders "what's the forest word for this?" the answer is
   one keypress away. Without the TUI, a custom vocabulary would
   create discoverability problems. With the TUI, custom vocabulary
   becomes a quality-of-life win.

3. **Stabilization week revealed friction in muscle memory.** Building
   fsh while typing UNIX commands every minute exposes how arbitrary
   the UNIX names feel when you are paying attention to your own
   thinking. "rm" is what fingers know; "delete" is what brain means.
   The brain-finger gap is real and worth closing.

4. **NY presentation thesis demands it.** "A human + AI built a
   coherent system in months instead of years" is one claim. "And in
   doing so we noticed the vocabulary itself could be redesigned" is
   the deeper claim — the one that opens the post-Linus conversation.

## Approach

### Principles (the spine of every vocabulary decision)

1. **Human words first.** The canonical name of an operation is the
   English word a non-engineer would use. `delete`, `find`, `copy`,
   `move`, `read`, `show`, `where`, `explain`.

2. **UNIX is fallback, never broken.** Every UNIX command continues
   to work. `rm` still removes. `ls` still lists. The forest never
   shadows the UNIX command in a way that breaks scripts or pipelines
   that already use it.

3. **Short aliases for fingers.** Each canonical name has a short
   alias if typing speed matters in daily use. `delete` → `del`.
   `find` → `f` (only if no collision). Aliases live in the registry
   alongside their canonical names.

4. **Forest-aware behavior earns the rename.** A forest vocabulary
   word is not just a synonym for a UNIX command. It adds value:
   lock-state checking, registry publishing, Friday observability,
   structured output, contextual warnings. If a proposed word does
   not earn its keep, it does not ship.

5. **No vocabulary by completionism.** This intent does NOT exist to
   replace every UNIX command. It exists to identify which commands
   genuinely benefit from forest-aware semantics and to give those
   commands clear human names. Completionism for its own sake is
   busywork.

6. **Vocabulary grows through daily-driving.** New words enter the
   vocabulary when real friction surfaces. Not from a wishlist. Not
   from "what would be nice." From "I just typed `<UNIX command>` for
   the 30th time this week and it would be clearer if it were called
   `<human word>`."

7. **Friday participates in vocabulary decisions.** Once Friday's
   pattern recognition is mature enough (post-v22), it can surface
   "you used `<UNIX command>` 47 times this week — does this deserve
   a forest word?" The vocabulary becomes data-driven over time, not
   speculative.

### The First Vocabulary (initial scope)

These two ship as part of this intent. Each was chosen because forest-
aware behavior earns its keep, and because daily-driving has surfaced
real friction with the UNIX equivalents.

#### `delete <path>` (alias `del`)

Forest-aware behaviors:
- **Lock check**: refuses if target is inside a chattr +i protected
  area; suggests `unlock-core` when appropriate
- **Source-tree warning**: prompts for confirmation if target is in
  source-controlled paths (rust-tools/, intents/, scripts/, docs/, etc.)
- **Stabilization-week awareness**: if a stabilization-focus is active
  and the target appears unrelated to it, warn before proceeding
- **Trash by default**: items go to ~/.local/share/forest-trash/ first;
  `--force` skips trash for true delete; trash auto-cleans after N days
- **Event emission**: `file_deleted` event with path, size, source-tree
  status, fires to Friday's signal stream
- **Friday observability**: if a file gets deleted that's referenced
  in recent commits or open intents, Friday surfaces this as a
  contradiction worth investigating

`rm` continues to work unchanged. This is additive.

#### `find <pattern> [path]`

Forest-aware behaviors:
- **Wraps `fd` as backend** (no reimplementation of file traversal)
- **Structured output as Value::Table** by default — pipes naturally
  into `query`, `fsearch`, and other Value-aware fsh builtins
- **Tracked/untracked awareness**: shows git status badge per result
  (✓ tracked, • untracked, ✗ ignored)
- **Forest path shortcuts**: `find foo @rust` searches rust-tools/,
  `@intents` searches intents/, `@scripts` searches scripts/, etc.
- **Stale flag**: results in directories with no git activity in
  30+ days marked dim
- **Composability**: `find "*.rs" | fsearch "TODO"` works because
  both speak Value::Table

`find` (UNIX) and `fd` continue to work unchanged. This is additive.

### Future Vocabulary (NOT in scope for this intent)

These are candidates that may enter vocabulary later, only when daily-
driving reveals genuine friction AND forest-aware behavior earns the
rename:

- `copy` / `move` (cp / mv) — would need lock-state awareness, source-
  tree warnings, event emission
- `read` / `show` (cat / less / bat) — would need syntax-aware rendering,
  registry-driven help, structured output for known formats
- `make` (touch / mkdir -p) — would need template awareness, registry
  publishing, faelight-link integration
- `where` (which / type) — registry-aware, replaces fsh's existing
  builtins once the cheatsheet TUI proves itself
- `explain` (man / --help) — registry-driven, single source of truth
  for documentation
- `tree` (tree / eza --tree) — forest-aware, highlights tracked-vs-
  untracked, lock state per directory

None of these ship in INT-261. They emerge as separate intents when
real friction justifies them. This intent commits to the principle
and the first two words; it does not commit to the rest.

### What this intent is NOT

- NOT a UNIX replacement. UNIX commands continue to work.
- NOT a wholesale renaming of every command. Only commands where
  forest-awareness earns the new name.
- NOT a generative AI feature. Friday does not invent vocabulary.
  Vocabulary is designed deliberately by Christian, recorded in the
  registry, and used by Friday (and other consumers) as data.
- NOT a localization system. The vocabulary is English. Multi-language
  support is out of scope.
- NOT a script-breaking change. Every existing script using UNIX
  commands continues to run unchanged.

### What this intent IS

- A philosophy: the forest speaks human first, UNIX as fallback.
- A principle set: 7 principles that govern every vocabulary decision.
- A first vocabulary: `delete` and `find` as the initial two commands.
- A pattern: how new vocabulary enters the forest (data-driven, not
  speculative).
- A foundation: registry-published, cheatsheet-discoverable, Friday-
  observable.
- A thesis-level statement: this prototype demonstrates that system
  vocabulary can be redesigned without losing power or composability.

## Hard Dependencies

- INT-259 (Command and Keybind Registry) — vocabulary publishes here
- INT-260 (Cheatsheet TUI) — makes vocabulary discoverable
- ratatui pattern (proven in INT-250) — for any future vocabulary
  commands that present interactive UI
- Existing fsh builtin infrastructure (commands/mod.rs)
- Friday Phase 2 knowledge engine (for future data-driven vocabulary
  decisions, post-v22)

## Success Criteria

- [ ] All 7 principles documented in this intent and applied to
      every vocabulary decision
- [ ] `delete` (with `del` alias) shipped as fsh builtin
- [ ] `delete` lock-state check working (refuses if path is locked)
- [ ] `delete` source-tree warning working (prompts on rust-tools/,
      intents/, scripts/, docs/ paths)
- [ ] `delete` trash-by-default working (~/.local/share/forest-trash/)
- [ ] `delete --force` bypasses trash for true delete
- [ ] `delete` emits `file_deleted` event to Friday signal stream
- [ ] `find` shipped as fsh builtin, wrapping `fd`
- [ ] `find` outputs Value::Table by default
- [ ] `find` shows git tracked/untracked badge per result
- [ ] `find` supports forest path shortcuts (@rust, @intents, etc.)
- [ ] `find` chains correctly into `fsearch` and `query`
- [ ] Both commands publish to command registry (INT-259) on deploy
- [ ] Both commands appear in cheatsheet TUI (INT-260) with full detail
- [ ] `rm` and `find` (UNIX) continue to work unchanged
- [ ] No regression in existing scripts that use UNIX commands

## Scope

### In scope
- The thesis and 7 principles
- `delete` / `del` as fsh builtin with all forest-aware behaviors
- `find` as fsh builtin wrapping `fd` with structured output
- Registry publishing for both
- Cheatsheet TUI integration
- Documentation in COMMAND-GUIDE.md explaining the vocabulary philosophy

### Out of scope
- Any vocabulary command beyond `delete` and `find` (each future
  vocabulary command is its own intent, gated by daily-driving evidence)
- UNIX command shadowing or breaking changes
- Multi-language support
- Auto-vocabulary-generation by Friday
- Mandating that humans use forest vocabulary instead of UNIX

### Deliberately deferred
- `copy`, `move`, `read`, `show`, `make`, `where`, `explain`, `tree`
  — each gets its own intent when daily-driven friction earns it
- Replacing fsh's existing `which`, `type`, `explain`, `debug` builtins
  — that retirement waits for the cheatsheet TUI to prove itself
- Vocabulary changes propagating to documentation auto-rewrites — that
  is Friday v22 Pillar 1's job, not this intent's

## Risks and Mitigations

### Risk 1: Vocabulary fragmentation (some commands renamed, others not)
**Mitigation**: This is a feature, not a bug. The principle is "rename
when forest-aware behavior earns it" — uneven coverage is the honest
result of that principle. The cheatsheet TUI (INT-260) makes the actual
vocabulary discoverable so confusion stays bounded.

### Risk 2: Muscle memory war (fingers want UNIX, brain wants forest)
**Mitigation**: UNIX commands stay live. Short aliases (`del`) reduce
typing friction for the new vocabulary. Vocabulary grows slowly,
giving fingers time to learn one word at a time, not all at once.

### Risk 3: Script compatibility breaks
**Mitigation**: Forest vocabulary is additive. UNIX commands continue
to work. Scripts using `rm` keep running. New scripts can choose to
use `delete` or `rm`; both are valid.

### Risk 4: Vocabulary drifts from registry / cheatsheet shows stale words
**Mitigation**: Both `delete` and `find` publish to registry on every
deploy. Staleness detection in INT-259 catches drift within 14 days.

### Risk 5: This intent is bigger than it looks and won't actually finish
**Mitigation**: Scope is explicitly two commands plus the principle
set. Future vocabulary is deferred to separate intents. This intent
ships when `delete` and `find` are working, registered, and discoverable
— not when a complete vocabulary exists.

## Gate Check
⬜ Not started

---

*"UNIX taught a generation that systems speak in abbreviations.
The next generation should be taught — and the systems they build
should remember — that machines can speak in words humans use.

`delete` is not a longer way to say `rm`.
`delete` is the right name.
`rm` is what we typed in 1971 because keys were expensive.

The forest can do better. The forest will do better.
And if the forest can — Linux can." 🌲*
