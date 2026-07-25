---
id: 196
date: 2026-07-25
type: arch
title: "execution-governing code must not infer shell structure from raw text"
status: planned
tags: [architecture, rust, design]
---

## Vision
Execution-governing code learns shell structure from the parser. It does not
rediscover redirects, pipes or operators by scanning the raw line.

## ⚠️ BLOCKED -- GATE ZERO IS A PREREQUISITE, NOT A QUESTION
This intent CANNOT be worked until the parser owns execution. There is no
canonical replacement to point violations at today: the answer for structure is
"ask the parser", and the parser is not authoritative until INT-169's flip. Do
not start this by hand-writing a better scanner -- that would add a fifth
interpretation of shell structure to a codebase whose documented bug class is
having several. Split out of INT-195 for exactly this reason: an intent must not
be responsible for enforcing something the codebase cannot yet satisfy.

## The Problem
INT-172 is the worst instance. detect_redirect scanned the raw line for '>' and
dropped everything after `2>` -- silent every time, three sightings before it was
caught, and physical evidence survived on disk in a file literally named after
the pipeline that was swallowed. INT-143 was the same shape at a different site:
four tokenizers with no pipeline between them, six bugs in one day.

Known instances, enumerated 2026-07-25 and not yet fixable:
  - main.rs, the cat_with_redirect block: `line.contains(" > ")`,
    `line.contains(" >> ")`, and `line.split_whitespace()` scanning for
    bat-unsupported flags. Decides routing for every `cat` invocation.
  - main.rs, detect_redirect itself. The whole function is a raw-line scanner;
    INT-172 turned it from a parser into a router, which made INT-171's job
    smaller, but it still reads structure out of text.
That list is a starting point, not a census. The census belongs to this intent
and should be taken when the intent starts, since the sites will have moved.

## Scope
The same boundary INT-195 established: the execution path, plus any component
whose decision directly governs execution (a privileged execution consumer).
Independent consumers of raw user text are exempt. See INT-195 for the full
statement -- this intent inherits that scope rather than restating it.

## Relationship to INT-195 and INT-169
INT-195 owns the command word: canonical derivation, enforceable today because
commands::command_word() exists. THIS intent owns shell structure: enforceable
only once the parser is authoritative. Same architectural property, different
prerequisites -- which is why they are separate numbers rather than one intent
that is half-actionable. INT-169's flip is the unblocking event, and this intent
is one of the things the flip pays for.

## Success Criteria
- [ ] GATE ZERO: the parser is authoritative for execution, or this intent stays
      blocked. Do not tick the rest before this one
- [ ] Every execution-governing site that infers shell structure from raw text is
      enumerated with file:line, taken fresh at start rather than inherited from
      the 2026-07-25 list above
- [ ] Each enumerated site either consumes parser-owned structure, or is recorded
      as a known exception with a stated reason
- [ ] detect_redirect is resolved: replaced by parser-owned structure, or its
      remaining role is stated and bounded
- [ ] A check exists that returns the violations, runnable on demand
- [ ] RETRO-VALIDATION: the check is confirmed to catch INT-172's original
      detect_redirect truncation, watched failing against the pre-fix shape. A
      check that would have missed the bug it exists to prevent is the wrong check
