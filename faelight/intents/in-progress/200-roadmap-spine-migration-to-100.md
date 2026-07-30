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
<!-- evidence: commit 6b61e673, 2026-07-29. `ParseError::UnsupportedOperator` carries the operator
     kind and commands/mod.rs:612 discarded it with `Err(_)` one line before incrementing the
     counter. Bound, passed, classified into `declined_by_reason`, rendered sorted descending.
     Output above. The prediction going in was wrong and the counts corrected it -- which is the
     whole argument for this gate existing before any of the others. -->
- [ ] Redirects parse, lower into `IoPlan`, and execute on the spine
- [ ] The three redirect buckets fall to ~0 in `spine migrate`
- [ ] Pipelines parse into an AST node, lower, and execute with POSIX exit status (the LAST
      stage), WITHOUT reintroducing INT-143 double execution
- [ ] Pipe bucket falls to ~0, and forest pipelines still work
- [ ] `unexpected` stays at 0 -- checked after EVERY construct, never only at the end
- [ ] `FSH_SPINE=1 fsh-test` stays green throughout
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
