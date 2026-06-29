---
id: 100
date: 2026-06-29
type: future
title: "fsh: variable assignment and $VAR expansion (VAR=$(...) name-case bug)"
status: planned
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
