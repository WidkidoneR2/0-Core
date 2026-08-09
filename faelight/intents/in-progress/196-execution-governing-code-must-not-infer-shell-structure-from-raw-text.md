---
id: 196
date: 2026-07-25
type: arch
title: "execution-governing code must not infer shell structure from raw text"
status: in-progress
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

## FINDING 2026-08-09: A DERIVATION BOUNDARY, NOT A DUPLICATION SITE

The investigation does NOT establish command_word or split_into_segments as duplication sites.

command_word is already explicitly the single owner of command-word extraction -- INT-171 gate 2 --
and its three callers are callers of one owner rather than three independent derivations. Its own
doc enumerates which split_whitespace calls elsewhere are NOT command-word extraction and why each
is safe. split_into_segments is likewise a deliberate single owner with two callers, and its
documented history shows why that consolidation mattered: when the REPL and the migration audit each
split inline, both cut an if-then-fi containing && at the &&, and sh reported a syntax error. Four
days live, missed because every chain test used simple commands.

⭐ THE REMAINING PROBLEM IS THEREFORE ONE OF DERIVATION BOUNDARY.

Both helpers derive parser-owned structure from raw source text BEFORE the parser has established
the authoritative structure. In particular, command_word invokes tokenize itself. So although
command-word extraction has been consolidated to one function, that function is still performing a
SECOND TOKENIZATION before the scanner and parser produce the AST.

The AST already contains the authoritative answer: the command word is the first Word of a Command
node. The same principle applies to segments -- they should be obtained from the parsed structure
rather than reconstructed from the original text.

⚠️ SO THE RELEVANT CHANGE IS NOT ANOTHER DEDUPLICATION OR CENSUS SWEEP. It is a boundary and
ordering change: parse first; derive command words and segments from the resulting AST; do not
reconstruct parser-owned structure from raw text before parsing.

This is materially larger and riskier than consolidating existing helpers. It changes the order in
which the shell's processing pipeline establishes facts, and potentially affects callers that
currently depend on the pre-parser representations.

⚠️⚠️ ACCORDINGLY THIS INTENT RECORDS THE FINDING AND DOES NOT BEGIN IMPLEMENTATION. The next step is
to establish SEPARATE evidence for the proposed parse-first boundary: identify all current consumers
of command_word and split_into_segments, determine what AST information each actually requires, and
verify that the parser and AST can supply equivalent information across the affected command forms.

NO IMPLEMENTATION SHOULD BE INFERRED FROM THE CENSUS OR CONSOLIDATION WORK ALONE.

## CENSUS CORRECTION AND EVIDENCE BOUNDARY (2026-08-09)

command_word has approximately TWENTY-FIVE real call sites, not five. Its documentation's reference
to "the five sites that needed the user's command word to ACT on it" describes the five
dispatch/action sites, not the total consumer count. The documentation is corrected so a reader does
not mistake the dispatch count for the complete census.

split_into_segments has exactly two callers, consistent with its documentation.

tokenize has six additional callers beyond command_word. A parse-first design would therefore not
simply reorder two helpers -- it would displace a tokenizer that currently has its own independent
consumer set. That makes a blanket migration premature.

⭐ SO THE CENSUS ESTABLISHES THE NEXT EVIDENCE STEP AS CONSUMER CLASSIFICATION, NOT MIGRATION. The
command_word callers visibly fall into at least three categories:

  EXECUTION-GOVERNING -- safety_guard.rs:19, dispatch, routing. These need the parser's authoritative
  structure and are the actual subject of this intent. Candidates for AST-based replacement.

  CLASSIFICATION -- comparisons such as == "jobs", == "fg", == "kill". These are not necessarily
  asking for an execution command word in the AST sense, and command_word's own documentation already
  establishes that a quoted word fails these comparisons and safely falls through. Their semantics
  must be examined before treating them as migration targets.

  TELEMETRY AND LABELLING -- record_failure, db.rs:592, the routing instruments. These consume the
  derived word while recording or labelling an outcome. A text-derived value can be defensible here,
  and replacing it with an AST-derived one does not inherently provide a correctness benefit.

This is the same evidence discipline INT-210 used for the quote-state machines: classify each
consumer by what information it actually needs, rather than assuming every caller of a shared helper
should migrate when the helper's implementation changes.

⚠️ THE STEP IS THEREFORE: classify the ~25 command_word consumers by semantic role and required
authority, and DO NOT MIGRATE THEM YET. Do not treat the census as a migration plan. The
execution-governing consumers establish the parse-first correctness case; classification and
telemetry consumers may have different and valid reasons to retain their current representation.

The same analysis applies to the six independent tokenize callers before any proposal to remove or
relocate that path. The purpose is to determine which consumers actually require parser authority --
not to make every consumer AST-based by association.

## THE CLASSIFICATION (2026-08-09) -- nineteen sites read, three kinds

EXECUTION-GOVERNING -- the word decides WHAT RUNS or WHETHER IT MAY RUN. These need parser authority
and are this intent's actual subject.

  safety_guard.rs:19      the allow/deny lists. A mis-read word means a dangerous command is not
                          blocked. Highest stakes of the nineteen, and its own comment records the
                          near-miss: the bare form was CHALLENGED and blocked while the quoted form
                          produced no guard output at all.
  commands/mod.rs:270     the alias-expansion loop, with cycle detection over `seen`. Decides what
                          the line BECOMES before anything runs it.
  engine.rs:1536          expand_aliases. Same question, the other owner.
  engine.rs:760           try_query_executor's forest-pipeline detection. This decides WHICH LANGUAGE
                          runs -- the two-languages boundary itself.
  engine.rs:1502          try_shell_construct. Decides whether the line is handed to sh whole.
  main.rs:184             is_repl_state_command. Decides routing exclusion, and its own doc says one
                          rule here is what INT-193 existed to end.
  main.rs:1618            the yazi / faelight-fm check, which selects a cwd-handoff execution path.

CLASSIFICATION -- the word is only COMPARED to a literal. command_word's own doc already argues a
quoted word fails the compare and falls through safely, so these are not automatically migration
targets and their semantics must be examined before treating them as such.

  engine.rs:850           `!= "jobs"` -- an early return guard.
  main.rs:1029            `== "cargo"` for Friday's power switching. Not execution.
  main.rs:1060            `== "flow"` -- ⚠️ READ THIS ONE AGAIN BEFORE DECIDING. It may dispatch
                          rather than merely compare, which would move it to execution-governing.

TELEMETRY AND LABELLING -- the word labels a record or a message AFTER the fact. A text-derived value
is defensible; an AST-derived one offers no correctness benefit.

  commands/mod.rs:9220    the not-found SUGGESTION message.
  commands/mod.rs:9362    the builtin not-found check, for the same message.
  commands/mod.rs:9435    record_failure.
  db.rs:592               snapshot naming. ⚠️ Its comment says it is reached solely from the
                          execution path and to revisit if a second caller appears -- so it is
                          telemetry TODAY by a stated assumption, not by nature.
  engine.rs:1655          the legacy-executor instrument added during this work. Temporary.
  engine.rs:1888          slow-command telemetry.
  main.rs:1009            snapshot capture. ⚠️ Safety-ADJACENT: it decides whether a destructive
                          command gets a snapshot, so a mis-read word means no snapshot before an
                          `rm`. Classified telemetry because it records rather than gates, but it is
                          the one telemetry site whose failure has a cost.
  main.rs:2943            Friday's consecutive-failure hint.
  main.rs:2986            Friday's suggestion filter.
  exec.rs:467             telemetry key normalisation -- its own comment says so.

⭐ THE SHAPE: SEVEN execution-governing, THREE classification, NINE telemetry. So roughly a third of
the call sites are this intent's subject and two thirds are not -- which is exactly why the census
was not a migration plan.

THE TWO FLAGGED SITES WERE READ, AND BOTH MOVED TO EXECUTION-GOVERNING. My first classification was
wrong on both, which is why they were flagged rather than filed.

  main.rs:1060  DISPATCH, not comparison. `if ftok == "flow"` then matches a subcommand and
                EXECUTES -- set_focus_intent, database writes, printed output. A builtin dispatched
                on a text-derived word, which is exactly this intent's subject.

  main.rs:1009  GATES, not records. `if _is_destructive { capture_snapshot(...) }` -- the word
                decides whether a snapshot is taken BEFORE a destructive command runs. Its own
                comment records the failure: on gen 432 the quoted form of rmdir produced NO
                snapshot row. The underscore-prefixed names read as unused and are not; that
                convention is misleading here and worth fixing when the site is touched.

REVISED SHAPE: NINE execution-governing, TWO classification, EIGHT telemetry. Both sites that moved
are PROTECTIVE -- one dispatches, one decides whether a safety net is deployed. Reading them
mattered more than the count did.

## GUARD EVIDENCE: ONE DECISION, TWO CALLS (2026-08-09)

Inspection of the two safety_guard::check(&line) call sites resolves the guard's control flow.

main.rs:2340 is the UNIVERSAL guard. It runs unconditionally after history is saved and before the
multi-line branch, the heredoc branch, the ? prefix, the ||| operator and run_input at ~2462. No
parsing has occurred; the guard receives only raw line text.

main.rs:2383 is inside the HEREDOC branch. That branch is reached only after the universal guard has
already run, and it passes the same unmodified line. The heredoc path therefore invokes the guard
twice with identical input and obtains the same decision twice.

So the two sites are NOT two independent guard situations. They are one decision expressed in two
places, with the second invocation redundant.

⭐ AND IT ESTABLISHES SOMETHING THE DESIGN MUST ANSWER: the universal guard intentionally sees inputs
that are not necessarily complete single commands. Multi-line pastes and heredoc continuations reach
it before execution parsing. So the parse-first design must EXPLICITLY define guard behaviour for
non-executable parser outcomes -- Incomplete and Refused -- rather than assuming every line presented
to the guard yields an executable AST. Blocking everything unparseable would be unusable; passing
everything unparseable is how the quoted rm got through.

THE STRONGEST SMALL-SCOPE DESIGN NOW SUPPORTED BY EVIDENCE:
  1. Parse once at the appropriate point before the universal guard.
  2. When an executable AST is available, obtain the first Word from the AST.
  3. Give that first-word FACT to the guard rather than asking it to reconstruct one from raw text.
  4. Remove the redundant heredoc guard call at main.rs:2383.
  5. Explicitly specify what the guard does for Incomplete and Refused BEFORE implementation.

## SEPARATE FINDINGS -- RECORDED, NOT FOLDED IN

⚠️ Neither belongs to this intent's change, and neither should expand it before the evidence boundary
above is settled.

DUPLICATE normalize_input. It is called twice consecutively at main.rs:2332-2333, which appears
accidental. Verify idempotence and side effects before removing either call. Not INT-196's subject.

HEREDOC DELIMITER RE-DERIVATION. The heredoc branch derives its delimiter at ~main.rs:2365 with a raw
split(" << "), then separately derives quoting at ~2373 with starts_with. This duplicates information
find_heredoc_intro already exposes -- but do NOT replace it merely because that function exists.
First establish whether its returned representation fully supplies the delimiter and quoting semantics
this branch requires. Another text-derivation finding, its own analysis.

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
