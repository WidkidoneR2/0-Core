---
id: 100
date: 2026-06-29
type: future
title: "fsh: variable assignment and $VAR expansion (VAR=$(...) name-case bug)"
status: in-progress
tags: [fsh, variables, expansion, shell]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---

## Why
fsh mishandles shell variable assignment + expansion. Observed 2026-06-29:
`TG=$(find /nix/store -name tuigreet -type f | head -1)` then `$TG --help`
failed with "command not found: $tg" -- fsh LOWERCASED the variable name
($TG -> $tg). So uppercase variable names aren't preserved through
assignment/expansion. Forces avoiding shell variables entirely (we used full
paths instead all session).

## Desired behaviour
- `VAR=value` and `VAR=$(cmd)` assignments preserve the variable name exactly
  (case-sensitive: TG stays TG, not tg).
- `$VAR` / `${VAR}` expansion resolves the correctly-cased name.
- Matches POSIX/bash case-sensitivity for variable names.

## Approach (rough)
- Find fsh's variable handling (rust-tools/faelight-shell/src/ -- the
  assignment parser + expansion path). Locate where the name is normalised
  (likely an unintended .to_lowercase() or case-folding somewhere).
- Preserve case through assignment storage and lookup.
- Test: TG=$(...) ; $TG runs the stored value; lowercase and mixed-case names
  also work; existing lowercase usage unaffected.

## Notes
Distinct from INT-099 (multi-line blocks). This is variable name-case handling.
Workaround until fixed: use full paths, avoid shell vars in fsh.
Surfaced during the metal-tuigreet session.


## ROOT CAUSE FOUND (2026-06-30) -- it is NOT lowercasing
Investigated main.rs. BOTH assignment paths preserve name case correctly:
- Standalone-assign (main.rs ~1659-1696): `let name = parts[0];` -- no lowercase.
- Inline-assign (main.rs ~1697-1760): `let name = &maybe_var[..eq];` -- no lowercase.
So the "$TG -> $tg" symptom was a MISLEAD; the real bug is VALUE TRUNCATION:

The failing case `TG=$(find /nix/store -name tuigreet -type f | head -1)` contains
SPACES + a command substitution `$(...)` + a pipe. Trace:
- The standalone-assign guard (line 1664-1668) requires no-space-after OR quoted.
  The $(...) value has spaces and is unquoted -> guard FALSE -> falls through.
- The inline-assign path (line 1703) does `rest.split_whitespace().next()` ->
  grabs only the FIRST whitespace token `TG=$(find`. So:
    * name = "TG" (correct)
    * val  = "$(find"   <-- TRUNCATED at the first space
    * the rest ("/nix/store ... | head -1") is then executed as a command.
  TG ends up set to the broken fragment, and the remainder runs as garbage.

ROOT CAUSE: the inline-assignment parser tokenizes with split_whitespace(), which
does not respect command-substitution `$(...)`, quotes, or pipes spanning spaces.

## Real fix (scoped -- needs a clear head, NOT a rushed one-liner)
Teach the assignment parser to treat a value as spanning spaces when it is a
command substitution `$(...)` (balanced parens, may contain pipes/spaces) or a
quoted string. Likely: detect `NAME=$(` and capture through the matching `)`,
running the substitution, before falling back to whitespace tokenization. Must NOT
regress: `KEY=val cmd` prefix-assign, plain `VAR=value`, multiple inline assigns.
Touches fsh's core command parser -> do deliberately with full test matrix.
Test cases: TG=$(echo hi); TG=$(find ... | head -1); A=1 B=2 cmd; VAR="a b c".

## Status
Root cause located + documented (the hard part). REOPENED 2026-07-01: status
corrected to in-progress -- diagnosis complete but the CODE FIX is not yet done,
so 100 is not complete until TG=$(...) actually works. Fix deferred to a focused session
-- rushing a change to the central command parser is how new bugs are born.
Workaround stands: avoid $(...) in fsh assignments; use full paths or a /tmp file.
