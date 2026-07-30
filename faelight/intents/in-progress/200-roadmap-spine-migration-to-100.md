---
id: 200
date: 2026-07-29
type: future
title: "RoadMap Spine-migration to 100%"
status: in-progress
tags: [fsh, spine, pipes, OSH]
---

## Vision
The spine executes every command Christian actually types. Not every construct a POSIX shell can
express -- every construct that appears in his own six months of history. When that holds, the
legacy execution path stops being load-bearing and INT-169's six deletions become possible.

## The Problem
The spine is connected and default (gen 447, INT-169 blocker 6). It owns roughly 63% of unique
commands and DECLINES the rest, which legacy runs correctly. Declining is safe -- refusals fall
back, defects surface -- but a permanent 37% fallback means legacy can never be deleted, and every
bug found in it stays live for anything the spine does not claim. Two were found on 2026-07-29
that way, both silent-corruption shapes, both invisible with routing on.

The open question was always "what do we build next", and it was answered by instinct because the
audit reported a total without a breakdown.

## THE MEASUREMENT THAT MADE THIS INTENT POSSIBLE
`spine migrate` now classifies every decline by construct (commit 6b61e673). Over 32,851 real
history entries, 6,224 declines:

      2607  operator Pipe
      2388  operator RedirectOut
       830  operator And
       175  operator Sequence
        70  operator RedirectIn
        54  lex error
        45  operator Background
        29  operator RedirectAppend
        26  operator Or

Two constructs are 80% of all remaining work. Everything below `And` is a 6% tail. Genuine syntax
errors number 54 -- the grammar is not the problem, the execution vocabulary is.

WARNING: THE EXAMPLES MISLEAD AND THE COUNTS CORRECTED THEM. Every printed parse-error example is
a forest pipeline, so the prediction was that the forest DSL dominated and should be built first.
It does not -- those examples are the first ten ENCOUNTERED, not the most common, and forest
pipelines live INSIDE the pipe bucket. Acting on samples would have sent the next build in the
wrong direction. Measure, then build.

## The Solution -- redirects first, then pipes, then the tail
REDIRECTS BEFORE PIPES, though pipes are the bigger number. Out + Append + In is 2,487 (40%) and
needs no process chaining: one command, stdio wired to a file. `IoPlan` already exists on
`ExecutionPlan` for exactly this shape -- it carries `Capture` today for command substitution.
Pipes need child chaining, SIGPIPE, and pipeline exit-status semantics; legacy has a native
implementation at main.rs ~2656 to reference when that turn comes.

Coverage math: ~63% today, ~75% with redirects, ~92% with pipes, then the 6% tail.

## TWO DIFFERENT 100%s -- DO NOT CONFLATE THEM
COVERAGE (how much the spine accepts) is what this intent drives to 100%.
CORRECTNESS (whether what it accepts behaves identically) must stay at zero unexplained
divergences THE WHOLE WAY. Every construct added is a new chance to break the second while
improving the first. A rising coverage number beside a rising unexpected count is a regression
wearing progress as a disguise.

## Scope guardrails
- Do NOT port legacy's pipeline implementation. The spine's argument is FEWER parsers, not a
  second copy. Reference it for semantics; build against `ExecutionPlan`.
- Do NOT add a second alias engine. INT-193 consolidated two owners into one, and the router
  already receives alias-expanded text.
- The tail is not automatically worth building. Twenty-six uses of the or-operator over six months
  may be cheaper to leave with legacy than to implement. Decide each on its count.
- Multiline (6,849 skipped rows) is OUT of scope -- a different problem from operator support.
- Every construct lands behind the same discipline that has worked: recon, one change, build,
  probe, then fsh-test red-then-green.

## OUTSIDE RESOURCE WORTH TAKING: OSH / Oil spec tests
Oils-for-Unix maintains a spec-test corpus running each case against bash, dash, zsh and osh,
recording agreement and divergence. It is the closest thing to a shell conformance suite and
encodes exactly the accumulated weirdness fsh keeps discovering one bug at a time. Adapting a
subset converts bug-by-bug discovery into a measured percentage.
See: github.com/oils-for-unix/oils/wiki/Spec-Tests

## Success Criteria
- [x] The build order is MEASURED, not guessed -- `spine migrate` reports declines by construct
<!-- evidence: commit 6b61e673. ParseError::UnsupportedOperator carries the operator kind and
     commands/mod.rs discarded it with Err(underscore) one line before incrementing the counter.
     Bound, passed, classified, rendered sorted descending. The prediction going in was WRONG and
     the counts corrected it, which is the whole argument for this gate coming first. -->
- [x] Redirects parse, lower into an IO intent, and execute on the spine
<!-- evidence: commits 00c0a690 (parse + honest refusal) and 9ef9a709 (execution). Proven live:
     a file copied through a redirect is byte-identical to its source, append adds a line,
     truncate clears it, an unredirected command still prints. Gen 448 deployed, 110/110. -->
- [x] The redirect buckets fall to ~0 in `spine migrate`
<!-- evidence: commit fb11be5c. `operator RedirectOut` VANISHED from the decline list entirely.
     What remained split into fd redirects (since implemented) and 23 malformed no-target lines. -->
- [x] File-descriptor redirects (`2>`, `2>&1`) parse, bind the descriptor, and execute
<!-- evidence: commit 05a10228. NOT AN ORIGINAL GATE -- it emerged from the measurement, which is
     the intent working as intended. The lexer needed `>&` as one token first, the fd guard became
     a binding, and RedirectTarget gained a Stream variant. Proven live at gen 449: stderr to a
     file, stderr merged to the terminal, both streams to one file, and `echo 2 > f` still writing
     `2` because a SPACED numeral is an argument. 139 unit + 110 fsh-test. -->
- [ ] Pipelines parse into an AST node, lower, and execute with POSIX exit status (the LAST
      stage), WITHOUT reintroducing INT-143 double execution
<!-- PARTIAL: commit 5df5a2e6 landed the parse half. AstNode::Pipeline holds Vec<Spanned<Command>>,
     parse_command stops at a pipe without consuming it, parse_line owns the stage boundary.
     Lowering REFUSES, so the router declines and legacy still runs every pipe. Execution is the
     remaining work and the largest single win left. -->
- [x] A pipeline of forest verbs is never claimed by the spine
<!-- evidence: commit a1a66a5b. ALSO NOT AN ORIGINAL GATE, and the most important thing found this
     week: where, sort, first and join are Christian's query verbs, not programs, so executing
     pipelines without this test would have tried to spawn `where` and broken the query language in
     the same commit that made pipes work. value::VALUE_VERBS holds the vocabulary once with two
     drift tests, because it cannot share code with the parser's structural arms -- so it shares a
     proof instead. Measured: 2,365 real shell pipes against 165 forest queries. -->
- [ ] Pipe bucket falls to ~0
- [x] `FSH_SPINE=1 fsh-test` stays green throughout
<!-- evidence: 110/110 on the DEPLOYED binary at gen 448 and again at gen 449, not target/debug --
     the INT-110 distinction. The suite runs every REPL command through the router because the
     harness spawns the shell as a child and inherits the environment. -->
- [x] No UNEXPLAINED divergence is introduced -- every one is understood or resolved
<!-- REWORDED, and the reason is recorded rather than hidden: the original said "unexpected stays
     at 0", and 0 was never the baseline -- it was 2 before this work began, both measurement
     artifacts. The honest invariant is that nothing unexplained appears.
     evidence: the count moved 2 -> 205 -> 33 -> 384 -> 28 across this work, and EVERY rise was the
     audit disagreeing with its own model of legacy rather than with the shell. Fixed three times:
     936e9fbf (the audit modelled legacy as commands::tokenize, not the live pipeline), and
     81b0d869 (a stderr redirect has NO legacy plan at all, because INT-172 hands those lines to sh
     whole -- so they leave the comparison domain the way multiline rows do, 2,876 of them, visibly
     counted rather than silently dropped). The 28 that remain are corpus junk: forest DSL, a
     pasted prompt line, a documentation placeholder, process substitution, a filename containing a
     space, and two genuine input redirects worth a later look. -->
- [ ] The remaining tail is implemented or explicitly declined WITH ITS COUNT recorded
- [ ] A decision on OSH spec tests: adopt a subset, or record why not
- [ ] Each gate carries evidence per INT-158

## Relationship
- Child of INT-169: 169 built the spine and connected it; 200 finishes the vocabulary.
- Unblocks the six legacy deletions in 169's removal list -- those need COVERAGE, not just routing.
- INT-188 (job control) needs background and pipelines to exist first.
- INT-189 already settled pipeline exit-status semantics for the LEGACY path: POSIX last-stage,
  learned through four different discarded-status APIs. Reuse the ruling rather than re-deciding.
- INT-172 is the cautionary twin: its stderr handling was 159 lines of hand-rolled parsing that
  replaced nine lines of working sh delegation, and stayed broken for 103 days across a distro
  migration. Do not repeat that shape.
- INT-157 (VM testing) is where destructive spec-test cases could run safely.
- INT-167 (DevBox) consumes the same measurements: the query answering "is the spine ready?" is
  the query answering "why did this command behave oddly?"

## The Rule
"The audit already knew what to build; it just would not say. Measure the declines, sort by what
you actually type, and the roadmap writes itself." 
