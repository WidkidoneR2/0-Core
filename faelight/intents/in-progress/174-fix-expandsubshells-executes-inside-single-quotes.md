---
id: 174
date: 2026-07-18
type: fix
title: "Fix expand_subshells executes inside single quotes"
status: in-progress
tags: [fsh, quotes, faelight-shell]
---

## Vision
Single quotes mean literal, everywhere -- including command substitution. `echo '$(date)'`
prints the literal string, it does not run `date`. Standard POSIX quoting semantics.

## The Problem
MEASURED 2026-07-20 on the deployed binary: fsh EXECUTES `$(...)` inside single quotes.
  echo '$(date +%Y)'  -> 2026        (WRONG -- should be the literal $(date +%Y))
  echo '$(whoami)'    -> christian   (WRONG -- should be the literal $(whoami))
Double quotes were fine (they SHOULD expand: echo "$(whoami)" -> christian, correct), and
the backtick form inside single quotes was already literal (echo '`whoami`' -> `whoami`).
So the bug was specific to `$(...)`.

ROOT CAUSE: expand_subshells() (expand.rs) walked the line looking for `$(` and expanded it
UNCONDITIONALLY -- it tracked no quote state at all. It could not tell it was inside single
quotes because it never looked. The backtick form only stayed literal by accident: backticks
are handled elsewhere, by code that does check quotes; `$()` went through this quote-blind
function.

Beyond correctness, this is a SAFETY issue: a single-quoted string is the one construct a
user reaches for to guarantee no execution. Executing it anyway violates the one promise
single quotes make.

## The Solution
Track single/double quote state while walking the line, and expand `$(` only when NOT inside
single quotes. Standard shell semantics: single quotes suppress ALL expansion; double quotes
and unquoted still allow command substitution. Only in_single gates the expansion, so
`"$(...)"` and bare `$(...)` keep working.

## Success Criteria
- [x] The bug is CONFIRMED on the deployed binary before the fix, with the specific cases.
      <!-- DONE 2026-07-20. Confirmed on deployed gen 404: echo '$(date +%Y)' -> 2026, echo
'$(whoami)' -> christian (both wrong, should be literal). Double-quoted correctly expanded,
backtick-in-single correctly literal -- so the bug was specific to $() in single quotes. -->
      (Done 2026-07-20: `'$(date +%Y)'` -> 2026, `'$(whoami)'` -> christian, both wrong.)
- [x] expand_subshells tracks quote state and skips `$(` inside single quotes. Prove with the
      <!-- DONE 2026-07-20, commit 2fbf13c8. expand_subshells (expand.rs) now tracks in_single/
in_double while walking; the $( expansion is gated on !in_single. Previously it tracked NO quote
state and expanded unconditionally. -->
      diff (in_single / in_double added; expansion gated on !in_single).
- [x] Single-quoted `$()` is literal AND double-quoted / unquoted `$()` STILL EXPAND -- the
      <!-- DONE 2026-07-20, deployed gen 405. Six cases in one run: '$(date +%Y)' -> literal,
'$(whoami)' -> literal, "$(whoami)" -> christian, $(whoami) -> christian, '`whoami`' -> literal,
'plain' -> plain. Single-quoted suppressed, double/unquoted preserved -- no regression. -->
      regression guard. Proven on the DEPLOYED binary: `'$(whoami)'` -> literal, `"$(whoami)"`
      -> christian, `$(whoami)` -> christian, all correct in one run.
- [x] A REPL test (Category::Repl, per INT-173) pins both directions: single-quoted stays
      <!-- DONE 2026-07-20, commit 71606776. repl_174_single_quote_no_subshell: single-quoted
`echo '$(echo INNER174)'` stays literal AND double-quoted expands, in one test. Proven real:
RED under the quote-fix revert ($() unconditional -> 96/97), GREEN on restore -> 97/97. -->
      literal, double-quoted expands. Green on HEAD, red if the quote-check is reverted.
- [x] fsh still boots, deploys, fsh-test green on the deployed binary.
      <!-- DONE 2026-07-20, gen 405. Deployed clean, reloaded, fsh-test = 97/97 on the DEPLOYED
binary (bare fsh-test, not cargo run). -->
- [x] Each gate carries evidence per INT-158.
      <!-- DONE 2026-07-20. Every gate carries measured cases / commit hash / demonstrated proof. -->

## The Rule
"Single quotes make one promise: nothing runs. A shell that breaks that promise cannot be
trusted with the next quoted string." 🌲
