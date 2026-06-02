---
id: 300
date: 2026-05-12
type: arch
title: "fsh Language Layer v1 — The Shell Speaks Human First"
status: complete
tags: [fsh, language, vocabulary, human-readable, grep, awk, builtins, arch]
version: TBD
depends_on: [261, 299]
---

## Vision

fsh is not just a shell. It is a language.

Most shells inherit 1970s vocabulary because they never questioned it.
fsh questions it. The forest speaks human first — `delete` before `rm`,
`find` before `fd`, `search` before `grep`. UNIX commands stay alive as
fallback. Nothing breaks. But the canonical name of every operation is
the word a human would use.

INT-261 established the philosophy and the first two words: `delete` and `find`.
This intent extends that work into a full language layer:
- Audit all 10 existing vocabulary words — are they working?
- Fix the ones that are broken
- Add the next vocabulary words that have earned their place
- Make grep, awk, cat, sed forest-aware builtins, not just passthroughs
- Build the test infrastructure that proves the language works

The goal: a shell where you never have to remember a UNIX abbreviation.
The words you already know are the right words.

---

## The Existing Vocabulary (10 words — audit required)

These words are registered in fsh dispatch at vocab_builtins:
  write, read, list, copy, move, delete, find, db, gt, it

Each needs an audit:
  - Does it execute correctly?
  - Does it have forest-aware behavior beyond just renaming?
  - Does it appear in the cheatsheet TUI (INT-260)?
  - Does it have a test in fsh_audit.sh?
  - Is the UNIX fallback still working?

Suspected broken: copy, move (may not have forest-aware behavior)
Suspected incomplete: write, read (may just be aliases, not enhanced)
Known working: delete, find (INT-261), list (ls wrapper), db, gt, it

---

## Why Now

1. INT-261 proved the concept — delete and find work and are registered.
2. INT-299 fixed the shell integrity — commands are now reliable enough
   to build on. A language layer on a broken shell is useless.
3. The presentation is this summer. "The shell speaks human first" is
   a thesis-level statement. It needs to be demonstrated, not described.
4. The 10 words already exist in dispatch. This is not a greenfield build —
   it is an audit, a fix, and an extension of what already exists.
5. grep is now fixed (INT-299). The builtins are ready to be enhanced.

---

## Approach

### Phase 1 — Audit the 10 Existing Words

For each word: write, read, list, copy, move, delete, find, db, gt, it

Step 1: Run it. Does it produce output?
Step 2: Does it have forest-aware behavior (lock check, event emission,
        structured output, source-tree awareness)?
Step 3: Does the UNIX fallback still work?
Step 4: Add a test to fsh_audit.sh
Step 5: Fix what is broken

### Phase 2 — Enhance Core Builtins as Language Elements

These are not vocabulary words — they are enhanced versions of tools
the forest uses constantly. They earn forest-aware behavior:

SEARCH (unified grep/fsearch):
  - `search pattern [path]` — unified ripgrep wrapper
  - Shows context lines (-C 2) by default
  - `search --rust` searches only .rs files
  - `search --intent` searches intents/ only
  - `search --forest` searches entire 0-core tree
  - Falls back to /usr/bin/grep for complex patterns
  - Forest-aware: highlights matches in forest files differently

AWK:
  - Verify awk passthrough works in all pipe contexts
  - No custom builtin needed — just ensure it works reliably
  - Add pipe tests to fsh_audit.sh

CAT (enhanced read):
  - `cat` with redirect already fixed (INT-298)
  - `show <file>` as the vocabulary word for reading files
  - bat-enhanced display for reading, real cat for redirects

SED:
  - Verify sed passthrough in all contexts
  - Add pipe tests to fsh_audit.sh

### Phase 3 — New Vocabulary Words

These earn their place through demonstrated daily friction.
Each word gets its own section, its own tests, its own gates.

SEARCH (alias: s):
  Priority: HIGH — grep is used constantly, search is more human
  Forest-aware: context lines, file type filters, forest shortcuts

SHOW (alias: sh — if no collision):
  Priority: MEDIUM — replaces cat for reading files
  Forest-aware: syntax highlighting via bat, structured output for known formats

WHERE (alias: w):
  Priority: MEDIUM — replaces which/type
  Registry-aware: knows about forest vocabulary words, not just PATH binaries

### Phase 4 — Test Suite Expansion

Expand fsh_audit.sh from 50 to 75 tests:
  - 5 tests per vocabulary word (10 words = 50 tests covered)
  - 10 tests for enhanced builtins (search, awk, sed, cat)
  - 10 tests for new vocabulary words
  - 5 regression tests for INT-298 fixes

---

## Success Criteria

### Phase 1 — Vocabulary Audit
- [x] All 10 vocabulary words tested — 75/75 tests pass, each word verified 2026-05-13
- [x] copy works -- forest-aware, shows confirmation, tested 2026-05-13
- [x] move works -- forest-aware, shows confirmation, tested 2026-05-13
- [x] write produces correct output -- 'write text to file' syntax working 2026-05-13
- [x] read produces correct output -- line-numbered reader with file info 2026-05-13
- [x] list works as ls wrapper with forest awareness 2026-05-13
- [x] UNIX fallbacks verified -- rm/mv/cp/grep/ls all still work directly 2026-05-13

### Phase 2 — Enhanced Builtins
- [x] search <pattern> works -- unified ripgrep wrapper, removed fd alias conflict 2026-05-13
- [x] search --rust filters to .rs files -- verified 2026-05-13
- [x] search --intent filters to intents/ -- verified 2026-05-13
- [x] awk passthrough verified in pipes -- test in suite 2026-05-13
- [x] sed passthrough verified in pipes -- echo | sed works 2026-05-13
- [x] show <file> works -- line-numbered bat-enhanced reader 2026-05-13

### Phase 3 — New Words
- [~] search registered in command registry (INT-259) — deferred to INT-259/INT-260/ongoing
- [~] show registered in command registry — deferred to INT-259/INT-260/ongoing
- [~] where registered in command registry — deferred to INT-259/INT-260/ongoing
- [~] All new words appear in cheatsheet TUI (INT-260) — deferred to INT-259/INT-260/ongoing

### Phase 4 — Tests
- [x] fsh_audit.sh expanded to 75 tests 2026-05-13
- [x] All 75 tests pass -- deterministic 2026-05-13
- [x] Each vocabulary word has at least one test 2026-05-13
- [x] Each new builtin has at least one test -- search, show, where, fsearch 2026-05-13

### The Standard
- [~] One full day using only fsh vocabulary — no muscle-memory UNIX fallback — deferred to INT-259/INT-260/ongoing
- [~] Friday can describe what each vocabulary word does from context — deferred to INT-259/INT-260/ongoing

---

## Gate Check
⬜ Not started

---

## The Philosophy (from INT-261, extended)

"Most desktops ask: what app do you want to open?
Most shells ask: what command do you want to run?
Faelight Forest asks: what do you want to happen?

The vocabulary is the answer to that question.
Not `rm -rf ./build` but `delete ./build`.
Not `grep -r TODO .` but `search TODO`.
Not `which fsh` but `where fsh`.

The shell is not a foreign language you learn.
It is a native language you already speak.
The forest just finally agrees with you." 🌲
