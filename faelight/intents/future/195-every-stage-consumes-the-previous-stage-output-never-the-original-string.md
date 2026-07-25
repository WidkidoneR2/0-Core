---
id: 195
date: 2026-07-25
type: arch
title: "every stage consumes the previous stage output, never the original string"
status: planned
tags: [architecture, rust, design]
---

## Vision
Within the execution pipeline there is ONE canonical derivation of the command
word. Nobody re-derives it.

## Scope
IN: the execution path -- main.rs, commands/mod.rs, exec.rs, expand.rs -- and any
component whose decision directly GOVERNS execution, even if it is not adjacent
in the pipeline. safety_guard.rs is the motivating case: architecturally outside
execution, semantically inside it, because it decides whether execution may
proceed. A PRIVILEGED EXECUTION CONSUMER is in scope.

OUT: independent consumers of raw user text -- completion, natural-language
parsing, semantic analysis, health UI, scripting utilities, value and db helpers.
They are not successive stages of one pipeline; completion runs on a partial line
before any parse exists. They may benefit from command_word(), but using
split_whitespace() there is not automatically a violation. This exemption is what
keeps the intent actionable instead of "replace every split_whitespace in the
repository".

CANONICAL: commands::command_word(). It is quote-aware per INT-171 gate 2, so
`"ll" foo` resolves to `ll`. Execution-governing code must not derive the command
word by other means.

TESTS: a test may bypass the canonical helper when it deliberately generates
malformed input or checks equivalence between implementations, but it must say so
in a comment -- otherwise the next reader treats it as precedent.

## The Problem
fsh's documented bug class is not "the parser was wrong". Twice the parser was
RIGHT and something downstream ignored it: INT-143 was four tokenizers with no
pipeline between them, six bugs on 2026-07-16; INT-171 consolidated to one
parsing entry point precisely because the entry points were never the problem,
the bypasses were.

A live instance, found 2026-07-25 while scoping this intent. safety_guard.rs:12
derives the command word with `trimmed.split_whitespace().next()`, which is not
quote-aware, while the executor uses the quote-aware command_word(). On
`"rm" -rf /` the guard sees `"rm`, matches no deny entry, no allow entry, no safe
entry, and fails its own `first_word == "rm"` test, so it returns None and gates
nothing -- while the executor sees `rm` and runs it. The guard's first-word-only
design is deliberate and correct; only the instrument is wrong.

## The Solution
Name the scope, name the canonical derivation, enumerate the sites inside the
scope that bypass it, and make the check mechanical. The fix at each site is a
call substitution, not new logic, because the correct implementation already
exists -- which is INT-143's lesson applied rather than restated.

## Narrowed 2026-07-25, and why (recorded, not silently rewritten)
This intent was FILED covering four banned calls: ad-hoc splitting, a second
tokenizer, raw-text operator scans, and re-parsing. Scoping recon showed the
first has a canonical replacement TODAY while the others do not -- the answer for
operator and structure derivation is "the parser", and the parser does not own
execution until the spine flips. An intent must not be made responsible for
enforcing something the codebase cannot yet satisfy, so the clauses were split by
their PREREQUISITES rather than their similarity:
  - THIS intent: nobody re-derives the command word. Enforceable now.
  - Operator/structure derivation: nobody re-derives shell structure. Enforceable
    only once the parser is authoritative. Its own intent, tied to the flip.
  - Guard placement: whether safety evaluates the typed, canonical or expanded
    command. A policy question, independent of how the word is derived. Its own
    intent.

## Success Criteria
- [ ] The scope above is recorded where code can be checked against it: execution
      path plus privileged execution consumers IN, independent consumers OUT,
      command_word() named as canonical, test bypasses requiring justification
- [ ] Every execution-governing site that derives the command word independently
      is enumerated with file:line -- a census, not a fix
- [ ] Each enumerated site either routes through command_word(), or is recorded
      as a known exception with a stated reason
- [ ] safety_guard.rs uses command_word(). Named explicitly because it is the
      privileged consumer that motivated the scope, and because a guard that
      disagrees with the executor about what is running is the worst instance of
      the class
- [ ] A check exists that returns the violations, runnable on demand
- [ ] The check runs somewhere it will be seen (pre-commit or fsh-test), not only
      by hand
- [ ] RETRO-VALIDATION: the check is confirmed to catch safety_guard.rs's
      split_whitespace derivation, watched failing before it is trusted. A check
      that would have missed the violation it was written for is the wrong check.
      NOTE: the original wording named INT-172's detect_redirect, which is an
      OPERATOR scan and moved out of scope in the narrowing above
