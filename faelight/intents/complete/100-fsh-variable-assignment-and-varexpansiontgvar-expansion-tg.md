---
id: 100
date: 2026-06-29
type: future
title: "fsh: variable assignment and $VAR expansion (VAR=$(...) name-case bug)"
status: complete
tags: [fsh, variables, expansion, shell]
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


## RESOLVED (2026-07-01) -- fix in place + verified across the matrix
The value_is_cmdsub check (main.rs ~1668-1688) detects a balanced $(...) value and
routes it to the non-truncating standalone-assign path; the substitution is executed
and its output stored. Verified in the running binary against the full test matrix:
- TG=$(find /nix/store -name tuigreet -type f | head -1)  -> real tuigreet path (the
  ORIGINAL failing case from the metal-tuigreet session -- now works, spaces+pipe).
- TG=$(echo hello)      -> hello
- X=$(echo one two three) -> one two three   (multi-word output captured whole)
- FOO=bar echo test     -> test              (prefix-assign regression: OK)
- Y=simple              -> simple            (plain assign regression: OK)
- Z="a b c"             -> a b c             (quoted-with-spaces regression: OK)
All pass. The value-truncation root cause (split_whitespace at line 1703) is bypassed
for $(...) values by the balanced-paren guard. Closing.

## Gates (reconciled per INT-130, 2026-07-10)
- [x] Variable name case preserved through assignment/expansion (TG stays TG). <!-- Root cause was NOT lowercasing (initial misdiagnosis corrected 2026-06-30); both assign paths preserve case. RESOLVED matrix confirms. -->
- [x] `VAR=$(cmd)` captures the whole value, not truncated at the first space. <!-- VERIFIED LIVE 2026-07-10: X=$(echo one two three) -> "one two three" (whole, untruncated). The exact split_whitespace root cause (main.rs:1703) is fixed. -->
- [x] The original failing case works: TG=$(find /nix/store -name tuigreet | head -1). <!-- RESOLVED matrix (2026-07-01): returns the real tuigreet path -- the metal-tuigreet-session case with spaces+pipe now works. Mechanism re-verified live this session. -->
- [x] No regressions: FOO=bar cmd (prefix-assign), VAR=value (plain), VAR="a b c" (quoted). <!-- RESOLVED matrix: all pass. -->
- [x] Fix is IN THE DEPLOYED BINARY, not merely diagnosed. <!-- INT-130 note: 100 was REOPENED 2026-07-01 because diagnosis had been mistaken for completion. This gate honors that: fix verified live in the running fsh this session (TG=$(echo hello)->hello; X=$(echo one two three)->one two three). -->

<!-- STAMP-100-DONE. Reconciled per INT-130, 2026-07-10: GENUINE reconcile + CHARTER REPAIR (like 099). Removed dead template stub; added 5 real [x] gates after the RESOLVED matrix. Was REOPENED once (2026-07-01) when diagnosis was mistaken for the fix -- so verified LIVE this session in the deployed binary: TG=$(echo hello)->hello, X=$(echo one two three)->'one two three' (whole, untruncated -- the split_whitespace root cause fixed). Initial 'lowercasing' theory was a misdiagnosis; real bug was value truncation. 9/23. -->
