---
id: 200
date: 2026-07-29
type: future
title: "RoadMap Spine-migration to 100%"
status: complete
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

## MEASURED STATE (gen 457 + 90db9f84, 2026-08-03)
Applicable to comparison: 21,735   Equivalent: 21,394  98.4%
Skipped: 10,125                    Safe improvements: 16
  multiline: 7,185                 Feature gaps: 167  0.8%
  stderr-delegated: 2,940          Unexpected: 30  0.1%
Spine parse errors: 107            Pipelines owned: 2,456
Declined by construct:
  148 forest value pipeline (legacy's permanently)
   39 lex: unterminated quote  ·  3 lex: unterminated $( )
   27 comparison, not a redirect -- DELIBERATE DIVERGENCE
   19 unlowerable: sequence (atomic control structures)
   23 redirect with no target (malformed)
   11 operator Background (structural -- see below)
    4 empty
THE DECLINE LIST NOW CONTAINS NO ACCOUNTING ARTIFACTS, which is the first time that has been true.
Every entry is legacy's by design, a deliberate divergence, malformed input, or a structural limit
that is stated rather than merely unbuilt. What remains as genuine construct work is roughly 19 rows.
THE ELEVEN BACKGROUND DECLINES ARE STRUCTURAL AND CORRECT. A trailing `&` after a boolean list has
lost its true operand before the parser sees anything, because main.rs splits the list upstream, and
wrapping what does arrive would background only the tail -- plausible output, wrong semantics. The
parser declines instead. If the splitter ever learns that `&` binds looser than `&&`, that check
simply stops firing and the parser is unchanged.
PIPELINES OWNED IS A POSITIVE CATEGORY, not a gap and not a skip. Legacy builds no single plan for a
pipeline -- its live path routes one to the native implementation or to sh, never through an
execution context -- so there is nothing to compare against. But the spine LOWERS AND RUNS these, so
counting them as gaps would have been the opposite of the truth.
⚠️ AND BACKGROUND IS DELIBERATELY *NOT* SUCH A CATEGORY. Its operand is an ordinary single command
that both engines model completely, so once the wrapper is unwrapped it belongs in equivalent
alongside everything else. A second positive bucket would have been an explanation offered where a
comparison was available -- 56 rows that briefly read as declines for commands the shell was already
executing.
HOW THE NUMBERS MOVED, and why a snapshot alone would mislead. Declines were 6,224 across nine
constructs when this intent opened, and are 274 now, of which 245 are deliberate or malformed.
Redirects, file-descriptor redirects, pipelines, boolean chains and background have all been
implemented or unblocked, and each moved rows OUT of the decline list into equivalence or the owned
category. The percentage dipped to 88.1 in between, which looked like a regression and was a
denominator effect: pipelines became comparable, so thousands of rows entered the applicable count at
once.
UNEXPECTED HELD AT 28-30 THROUGHOUT, which is the number that would have signalled real trouble. Its
full history is 2, 205, 33, 384, 28, 70, 29, 30 -- and EVERY rise was the audit disagreeing with its
own model of legacy rather than with the shell. SEVEN times. A future reader seeing a spike should
check the audit's model, and the DENOMINATOR, before treating it as a defect.

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
- [x] Pipelines parse into an AST node, lower, and execute with POSIX exit status (the LAST
      stage), WITHOUT reintroducing INT-143 double execution
<!-- evidence: commits 5df5a2e6 (parse) and f3695af8 (execute), deployed gen 450, 110/110 on the
     deployed binary. Proven live: a three-stage chain counts correctly, exit status propagates from
     the last stage, and a forest query pipeline still declines and renders its table.
     THE WIRING: stdin from the previous stage or inherited, stdout piped except on the last stage,
     and the last child carrying the status per INT-189's ruling. Pipe wiring is a DEFAULT that a
     stage's own redirect overrides, which legacy never had to reconcile because it had no per-stage
     IO plan. Every child is waited on even after a mid-pipeline spawn failure -- an unreaped child
     with an open pipe end blocks the reader forever, and that failure mode hangs rather than errors.
     NOT COPIED FROM LEGACY: its native pipeline opens with forty lines of hand-rolled quote-aware
     tokenizing per stage, which is the five-parsers problem in the flesh. The process semantics were
     borrowed; the tokenizer is what the spine exists to delete. -->
- [x] A pipeline of forest verbs is never claimed by the spine
<!-- evidence: commit a1a66a5b. ALSO NOT AN ORIGINAL GATE, and the most important thing found this
     week: where, sort, first and join are Christian's query verbs, not programs, so executing
     pipelines without this test would have tried to spawn `where` and broken the query language in
     the same commit that made pipes work. value::VALUE_VERBS holds the vocabulary once with two
     drift tests, because it cannot share code with the parser's structural arms -- so it shares a
     proof instead. Measured: 2,365 real shell pipes against 165 forest queries. -->
- [x] Pipe bucket falls to ~0
<!-- evidence: commit 87d174ba. `unlowerable: pipeline` is GONE from the decline list entirely --
     2,372 rows moved into a new `Pipelines owned` category. That required fixing the audit too: it
     was calling the single-plan lowering entry, which correctly refuses a pipeline, so it reported
     2,371 declines for commands the shell was already running. Fourth time the model diverged from
     the live path, and the first where the divergence was an ENTRY POINT rather than a data shape.
     Equivalence 88.1 -> 98.5 percent with no execution code changed. -->
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
- [x] Background (`cmd &`) parses, lowers, and executes on the spine, registered with the SAME
      job table the REPL owns
<!-- evidence: commits dd19eb9c (parse), 1462c970 (execute), 90db9f84 (audit), and the redirect
     composition, deployed gen 457, 117/117 fsh-test.
     NOT AN ORIGINAL GATE -- it emerged from the measurement, like the fd-redirect and forest-verb
     gates above, which is the intent working as designed.
     THE SHAPE: `AstNode::Background(Box<Spanned<AstNode>>)` WRAPS the operand rather than flagging
     a command, because `cmd &`, `cmd > out &`, `a || b &` and `(a; b) &` are one meaning with four
     operands -- a bool on Command would leak the moment the operand is not a command, which two of
     those already are. The parser DECLINES whenever it cannot see the true operand, which keeps the
     limitation structural rather than semantic: every Background node means "this entire subtree
     runs in the background", and the AST never claims a scope it did not observe.
     ⚠️ IT FIXED TWO LIVE BUGS, NOT JUST COVERAGE. Legacy re-derived argv with splitn and
     split_whitespace, so `bash -c "foo bar" &` reached the child as fragments; and a redirected
     background command never reached that path at all, because the redirect branch claims any line
     containing `>` and continues six hundred lines earlier -- no file, no job, exit zero, nothing
     reported. Both are now covered by fsh-test regressions.
     ⚠️ AND THE CHEAP FIX WAS TRIED FIRST AND REVERTED, which is worth recording because it is the
     obvious idea: moving legacy's background check above the redirect branch. It failed twice over
     -- the redirect arrived as argv (`uname: extra operand`) and the block below it stopped being
     reachable, breaking `jobs`. PHASE ORDERING ENCODES WHAT EACH PHASE CANNOT HANDLE, so moving a
     phase up moves its blind spots up with it. Only one structure parsing both operators together
     fixes it.
     ★ THE IO WIRING WAS EXTRACTED, NOT COPIED: `configure_file_io` is the single owner of the
     clone-not-reopen dup, truncate-versus-append, and missing-input-file rules, each learned from a
     live bug. The spawn stayed with the callers, which is what lets the foreground path wait and the
     background path register.
     ⚠️ DEFERRED ON PURPOSE: a background job gets no stderr tee, so the knowledge engine sees
     nothing from its failures. That matches legacy; giving them telemetry is its own decision. -->
- [x] The remaining tail is implemented or explicitly declined WITH ITS COUNT recorded
<!-- evidence: REWRITTEN 2026-08-03. The 2026-07-31 version of this gate said the 588 And/Sequence/Or
     rows were BLOCKED UPSTREAM and would stay blocked "until the routing point moves". They are not
     blocked any more, and the routing point never moved -- the SPLITTER did.
     WHAT HAPPENED (7db111fa): main.rs split a line on `;` and `&&`/`||` and ran multi-part segments
     through its own reduced dispatch, which predated the spine. That executor skipped variable
     expansion, alias resolution, `export` and the router. It was not merely incomplete: it had been
     hand-patched over time for whatever got noticed, so `cd` worked inside a chain and nothing else
     did. Three live bugs from one defect, the variable case silent. The repair flattened the two
     splitters into one list of command-and-operator pairs and deleted the parallel executor, so
     every logical part now flows through the same path a standalone command does -- and that path
     CONTAINS the routing point. Proven live: `echo one && echo two` produced NO router trace line
     before, and TWO `claimed` lines after.
     THEN THE AUDIT HAD TO CATCH UP (4e939bf4), because it was still feeding whole chained lines to a
     parser the live shell no longer feeds whole -- 586 declines for commands already running, the
     SIXTH instance of that class. A sequence is neither a skip nor an owned category: each part is
     an ordinary command both engines handle, so the row is split and each part compared. Sequence
     declines 586 -> 17, equivalent rows +1,353, and every control number held at baseline.
     ⚠️ The first attempt split ABOVE the applicability check and manufactured 14,000 rows from
     pasted code blocks. A jump in the denominator is the tell -- check it before reading any
     percentage above it.
     THE TAIL AS IT ACTUALLY STANDS (gen 457), and it is now smaller than any single piece built:
       19  unlowerable: sequence -- the ATOMIC constructs (`if ...; then ...; fi`, `for`/`while`)
                                    that split_semicolons deliberately keeps whole for sh
     DELIBERATELY DECLINED, WITH COUNTS -- 255 rows, none of them work:
       148  forest value pipelines -- legacy's permanently; `where`/`sort`/`first` are query verbs
        42  lex errors -- an odd quote or an unclosed `$(`; not commands
        27  the comparison guard firing on real history -- the divergence that keeps `> 0.5` working
        23  redirects with no target -- malformed input
        11  background after a boolean list -- STRUCTURAL, the parser cannot see the true operand
         4  empty
     BACKGROUND LEFT THIS LIST ENTIRELY (2026-08-03). It was 50 rows and is now parsed, lowered and
     executed: `AstNode::Background(Box<Spanned<AstNode>>)` wraps the operand rather than flagging a
     command, because `cmd &`, `cmd > out &`, `a || b &` and `(a; b) &` are one meaning with four
     operands and a bool on Command would leak the moment the operand is not a command. The router
     unwraps it and hands a configured Command to the SAME JobTable the REPL owns -- a second
     registry would have broken `jobs`, the completion notices and the prompt count.
     ⚠️ AND IT FIXED TWO LIVE BUGS RATHER THAN MOVING COVERAGE. Legacy's `&` path re-derived argv by
     splitting on spaces, so `bash -c "foo bar" &` reached the child as fragments; and a redirect
     never reached that path at all, because the redirect branch claims any line containing `>` and
     continues six hundred lines earlier -- no file, no job, exit zero, nothing reported.
     ⚠️ ONE THING DEFERRED ON PURPOSE: a backgrounded command gets NO stderr tee, so the knowledge
     engine sees nothing from its failures. That matches legacy exactly. Giving background jobs
     telemetry is a real improvement and its own decision; smuggling it inside a redirect fix would
     have made both harder to judge. -->
- [x] A decision on OSH spec tests: adopt a subset, or record why not
<!-- evidence: DECIDED AND BUILT -- `spine conform`, spine/conform.rs. The decision is MINE THE
     METHOD, NOT THE CORPUS. Their value is asking what a real shell actually does rather than what
     a shell should do; importing their cases would drag in historical bash warts as requirements,
     and fsh has deliberately diverged from bash at least twice. Bash is already on the box, so the
     reference is real rather than transcribed, and the cases are fsh's own -- scoped to what the
     spine owns: redirects, fd redirects, pipelines, exit status, quoting.

     THREE VERDICTS, NOT TWO, and that is the design: agrees, diverges-as-declared, unexplained. A
     declared divergence is a PASS, and a declared divergence that starts MATCHING bash again is a
     FAILURE, because deliberate behaviour was silently lost. Only unexplained is a defect -- so a
     NEW divergence announces itself the first time it appears.

     ⚠️ AND IT FOUND A REAL DEFECT ON ITS FIRST RUN: 14 agree, 2 unexplained, both digit-guard
     cases. The filesystem settled it -- fsh created the files exactly as bash did. THE DIGIT GUARD
     WORKS AT THE INTERACTIVE PROMPT AND NOT THROUGH `fsh -c`. Same binary, two doors, two
     behaviours: INT-173's two-doors problem in a new place, on behaviour proven live the day before
     and therefore believed correct. Recorded below as open work rather than closed here. -->
- [x] Each gate carries evidence per INT-158
<!-- evidence: the twelve blocks above. Same dogfood shape as INT-158's own final gate. -->

## CLOSED AFTER CONFORMANCE (both items resolved 2026-08-03/04)
1. **`fsh -c` does not apply the digit guard.** NOT FIXED, AND NO LONGER THIS INTENT'S. The cause
   turned out to be larger than the guard: `fsh -c` hands the whole string to `sh`, so no alias,
   router, guard or job table applies to it at all. Routing it through fsh needs a reusable executor
   that does not exist -- the entire pipeline lives inside main.rs's REPL loop, coupled through
   mutable session state. Re-homed to **INT-201**, which owns the extraction. `-c` stays on `sh`
   meanwhile, with the reasoning recorded at the handler.
2. **The conformance harness cannot see file effects.** FIXED. The suite moved into fsh-test and now
   drives a real pty, so it measures the interactive shell rather than `sh` -- which is what it had
   actually been comparing against bash all along. Cases read files back with `sed` rather than the
   aliased `cat`, and both shells run under /tmp so the reference implementation stops creating
   `0.5` and `=` in the repository. 15 cases, both declared divergences passing as divergences.

## WHAT "100%" TURNED OUT TO MEAN
Not 100% of parseable shell -- 100% of what the spine can own without breaking the query language.
Forest value pipelines are legacy's permanently, because `where`, `sort` and `first` are query verbs
with no programs behind them. The honest end state is 98.4% equivalence with a decline list holding
no accounting artifacts, which is what this intent reached.
⚠️ THE VISION'S SECOND HALF IS NOT DONE AND IS NOT THIS INTENT'S TO FINISH: "the legacy execution
path stops being load-bearing and INT-169's six deletions become possible." Legacy still runs the
forest pipelines and the 19 atomic constructs. Those deletions belong to INT-169, and they need the
executor extraction (INT-201) before the last of them is even reachable.

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
