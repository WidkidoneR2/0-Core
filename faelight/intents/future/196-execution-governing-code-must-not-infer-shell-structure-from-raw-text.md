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

## Census, first entries -- found by extraction, not by grep (2026-08-06)
INT-201's gate-4 work walked the segment loop line by line to extract it, and that walk found five
sites in one region of main.rs. They are recorded here because this intent owns the census.

FOUR DERIVATIONS OF THE SAME THING. The command to execute is derived four times between the router
and the executor: once after the expansions, again after a second pipe analysis, again when the file
manager needs its own form, and again for tilde expansion. Each derives from the line rather than
from the previous derivation.

TWICE FOR THE PIPE ANALYSIS TOO. Whether the line contains a pipe, and what its stages are, is worked
out once above and again below, the second set named with a trailing two. Nothing consumes the first
result; the second simply redoes it.

AND A HAND-WRITTEN QUOTE-AWARE SCANNER. One of those re-analyses is a closure that walks the bytes of
the line looking for an unquoted pipe, tracking whether it is inside double quotes as it goes. This
intent warns against opening the work by writing a better scanner, on the grounds that it would add a
fifth interpretation of shell structure. That warning is retrospective: the scanner is already there.

WHY THESE WERE NOT FIXED WHEN FOUND. Extraction would have moved them into the engine unchanged --
the same code in a tidier place, still deriving structure from raw text. That would have made the
violation harder to see without making it smaller. They stay where they are until there is something
correct to point them at, which is what gate zero is waiting for.

⭐ AND THAT IS THE USEFUL PART: gate zero says this intent is blocked until the parser is
authoritative over execution. INT-201 spent two days making it so -- the routing decision, the
assignment handling, the expansions and the streaming path all now live behind the engine, and
main.rs went from 3,913 lines to 3,060. The blocker is not theoretical any more; it is being lifted,
and these five sites are what will be waiting when it lifts.

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
