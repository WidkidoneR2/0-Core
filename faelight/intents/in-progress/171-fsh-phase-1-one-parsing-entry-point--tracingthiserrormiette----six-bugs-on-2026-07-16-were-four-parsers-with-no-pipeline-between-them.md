---
id: 171
date: 2026-07-16
type: arch
title: "fsh Phase 1: ONE parsing entry point + tracing/thiserror/miette -- six bugs on 2026-07-16 were four parsers with no pipeline between them"
status: in-progress
tags: [fsh, architecture, parser, tracing, diagnostics, phase1, 134]
---

## Vision
ONE path from input to execution. Every feature goes through it. When a bug appears, there is ONE
place to fix it.

    Input -> Lexer -> Parser -> AST -> Executor -> Builtins / External Commands / Plugins

That sentence is the whole intent. Everything below is evidence for it.

## Why this is Phase 1 and not Phase 3
This does NOT replace the parser. It makes the existing one SINGULAR. No new grammar, no AST redesign,
no new parsing dependency. Three well-understood crates (tracing, thiserror, miette) and a
consolidation of code that already exists and is already correct.

INT-169 is the "should we replace the parser?" question. This is the "stop having four of them"
answer. They would fight if merged. 171 runs FIRST, and 171s OUTCOME IS 169s EVIDENCE: once there is
a single entry point, the current parser either proves itself fine or its limits become obvious. That
is the advisory sequence -- Phase 1 before Phase 3 -- and it is also just honest: you cannot judge a
parser you have four of.

## THE MEASURED CASE -- 2026-07-16, INT-143 (six bugs, one root)
Not a hypothesis. Every one of these was reproduced on demand and fixed in a SEPARATE commit, because
they lived in SEPARATE parsers.

  bfe25bc9  `cmd > file` ran every external command TWICE. main.rs:2357 asked execute() whether a
            line was a builtin; execute() RUNS. Proven: mkdir into a dir that did not exist ->
            "File exists", because run 1 made it.
  968c7be5  fsh reported SUCCESS for commands that never ran. `nosuchcommand123 && echo DANGER` ->
            DANGER printed. run_external returned Empty (= success) after printing "command not
            found". `mkae build && rm -rf dist` would have deleted dist.
  d5a52c1c  `VAR="a b" cmd` -- THREE parsers wrong in one code path: split_whitespace() that does
            not know quotes; no POSIX scoping (the var persisted forever); and a
            starts_with(quote) && ends_with(quote) check that misread the whole LINE as a value.
            That is the QEMU_OPTS incident: an hour lost on 2026-07-15, blamed on the firmware.
  c5086945  python3 swallowed every flag -- `--version` was evaluated as Python source.
  5cba096d  `bash script.sh` dropped into interactive bash; the script never ran.
  56aa0798  `env VAR=x cmd` printed fshs environment table instead of running cmd.

THE COUNT THAT MATTERS: FOUR SEPARATE PARSERS were involved.
  1. commands/mod.rs `tokenize_args` (inside execute_impl)        -- CORRECT, quote-aware
  2. exec.rs `tokenize`                                            -- CORRECT, quote-aware, and a
                                                                      BYTE-FOR-BYTE DUPLICATE of #1
  3. main.rs:2411 redirect branch: splitn(2,\' \') + split_whitespace() -- WRONG, mangled every
                                                                      redirected command
  4. main.rs:1956 inline-VAR loop: split_whitespace()              -- WRONG, the QEMU_OPTS bug
  (+ expand.rs detect_redirect, is_complete_command, split_logical, expand_globs_in_segment --
     each doing its own scanning)

FSH ALREADY HAD A CORRECT TOKENIZER. TWICE. The bugs were code that did not call it.
That is the finding, and it is why "add chumsky" is the wrong first move: you can fail to call
chumsky exactly as easily as you can fail to call tokenize(). A fifth parser does not fix
"four parsers, nobody uses the good one".

WHY nobody called it: there is no pipeline to call it FROM. execute_impl parses. mains redirect
branch parses. The inline-VAR loop parses. detect_redirect parses. Four parsers because there are
four entry points. The absence of the diagram at the top of this file, itemized as six bugs.

## In scope
- ONE tokenizer. Delete the duplicate (exec.rs tokenize vs mod.rs tokenize_args are the same
  function). Every consumer calls the survivor.
- ONE parsing entry point. Nothing outside it may call split_whitespace() on a user-typed line.
- `tracing` -- structured spans over lexer / parse / expansion / dispatch / execution.
- `thiserror` -- typed errors instead of CommandResult::Error(String). Note 968c7be5: the message
  was honest and the TYPE was the lie. A shell whose errors are Strings cannot be asked what went
  wrong; it can only be read.
- `miette` -- rich diagnostics. Target: not "unexpected token" but
      error: expected command after '|'
        echo hello |
                   ^
- The existing tests keep passing (18/18 today) and the six INT-143 regressions get real tests.

## Explicitly OUT of scope -- these are other intents, deliberately
- Replacing the parser or lexer with logos/chumsky/nom/winnow -> INT-169. This intent must not
  smuggle in a rewrite.
- An explicit AST (Command / Pipeline / Redirect / VariableAssignment / If / While / Function) ->
  INT-169. It is the right long-term shape and it is NOT Phase 1. Phase 1 makes ONE parser produce
  todays representation; Phase 3 decides whether that representation should become an AST.
- reedline -> INT-168.
- Plugin runtimes -> INT-170.
- `ariadne` -- miette covers the diagnostic need first; ariadne only if miette proves insufficient.
  Two diagnostic crates for one shell is the duplication this intent exists to remove.

## Relationship to INT-167 (DevBox)
`tracing` here IS DevBoxs P1. Do not build it twice. 167 measured that fshs event spine already
exists (112,643 rows, 8 domains, 4 indexes) and that its tracing columns are DEAD (correlation_id:
475 rows, all the empty string, since 2026-05-22). 167s P0 is "make the existing instrumentation
honest"; this intents tracing work is the same seam approached from the shell side.
DECIDE ONE OWNER before starting. Two intents adding tracing to the same binary is the exact failure
both were written to prevent.

## Success Criteria
- [x] ONE tokenizer exists. Prove it: grep the tree, exactly one quote-aware tokenizer function, and
      the duplicate is DELETED (not deprecated)
<!-- DONE 2026-07-19, commit 3abc454e, deployed gen 397. Promoted one copy to
`pub fn tokenize` at module level in commands/mod.rs; deleted the nested tokenize_args
(execute_impl) and tokenize (exec.rs::from_line), repointed both callers. `grep -rn "fn
tokenize" src/` returns exactly one function. pty receipt on the deployed binary: 5/5
quote-aware cases pass, 172 stayed fixed. -22 lines net. -->
- [x] No code outside the parsing entry point calls split_whitespace() on a user-typed line. Prove
      it with the grep, and with each remaining call site justified in a comment
<!-- DONE 2026-07-20, commit c1fc7d69. Full triage: `grep -rn split_whitespace src/` = 92 sites;
filtered to split_whitespace().next() on a line-like variable = 27 candidates; classified each by
what it does with the token. Result: 5 sites extract a USER COMMAND WORD to dispatch/look-up
(main.rs alias-expansion 2229, forest-route 1420, forest-detect 2254; commands/mod.rs run_external
not-found 7776, builtin not-found 7876) -- all routed through the new command_word(line), the ONE
quote-aware extractor. The rest are justified in a rule-block above command_word(): output/telemetry
parsing, completion (a partial being typed, never dispatched), and classify-only checks that compare
the token and fall through safely on a quoted word.
HONEST SCOPE: this is CONSOLIDATION, not a live-bug fix. A probe of the pre-change deployed binary
confirmed quoted commands already resolved (quotes are stripped upstream of dispatch), so no site was
reaching a mis-parse in practice. The value is structural -- one home means the INT-143 quote-blind
bug cannot return piecemeal. A contract unit test (command_word_tests) guards quote-awareness, proven
red-under-revert / green-on-restore. cargo test 19/19, fsh-test 94/94. A REPL test that was green on
the pre-change binary (proved nothing) was removed. -->
- [x] The six INT-143 regressions have tests that FAIL on the pre-171 parser and pass after --
      demonstrated, not declared (INT-158)
<!-- DONE 2026-07-19, commit 6bff0f91, deployed gen 399, fsh-test 94/94. Six Category::Repl
tests (repl_143_*) drive the REAL REPL via pty -- NOT `fsh -c`, which bypasses fsh dispatch to
/bin/sh and would pass on a broken shell. Five proven by surgically reintroducing each bug and
watching ONLY its test go red (python3/bash/env/typo-&&/var-scope). The sixth (double-exec,
bfe25bc9) proven BY CONSTRUCTION: the fix is type-level robust -- swapping try_builtin->execute
does not reintroduce the double-run (measured 3 ways, append-counter = 1 tick), because
CommandResult::NotBuiltin routes around it. The test stays green because there is no bug, not
because it is blind. "pre-171 parser" read as "pre-143 broken behavior" -- the only binary
where these fail; interpretation agreed 2026-07-19. -->
- [x] `tracing` spans cover lexer / parse / expansion / dispatch / execution, and ONE typed command
      can be traced end to end. Owner agreed with INT-167 first, in writing
<!-- RESOLVED 2026-07-20 by ownership decision (the written agreement this gate required). Tracing is
INT-167's, not 171's: 167's principle 2 ("EVERY FUNCTION GETS TRACING -- ENTER parser.parse / ENTER
lexer.next ... a tree shell_start->parser->lexer->tokenize") IS this gate, verbatim in intent. 167
sequences ENTER/EXIT spans at its P2, built ON TOP OF its P0 substrate (a real per-command
correlation_id, one payload format, libdebug). That P0 does not exist yet -- correlation_id is still
the inert empty-string column. Building spans in 171 now would sit on an inert foundation, which is
167's own explicitly-named anti-pattern ("building more on an inert foundation makes the lie bigger").
DE-SCOPED to 167 (recorded in 167's Relationship section same day). 171 does NOT build tracing; it
completes on its parser-consolidation and error-typing gates. Gates 5 (thiserror) and 6 (miette)
remain real work -- this de-scope does not complete 171. -->
- [ ] `thiserror` types the errors that control flow depends on. The 968c7be5 class -- a result whose
      TYPE contradicts its MESSAGE -- is unrepresentable, not merely fixed
- [ ] `miette` renders one real fsh syntax error with a caret under the offending character
- [x] 18/18 existing tests still pass; fsh still boots, still logs in, still deploys
<!-- DONE 2026-07-19. The suite has grown 18 -> 94; all 94 pass on the deployed binary at
gen 399. fsh booted, logged in, and deployed cleanly across gens 397/398/399 during this work. -->
- [x] NOTHING from INT-168/169/170 landed here. If a change needs an AST, it is 169s change
<!-- DONE 2026-07-19. No reedline (168), no AST/logos/chumsky (169), no plugin runtime (170).
The AST was fenced to 169 explicitly: value.rs:292 PipeOp was confirmed to be the structured-DATA
pipeline (INT-162), not a command-AST seed, and the "make redirect emit a Redirect variant" idea
was scrapped after reading source. Gates 1-3 consolidated the EXISTING parser only. -->

## The Rule
"fsh already had a correct tokenizer. Twice. The bugs were the code that did not call it. Adding a
better parser to a shell with four parsers gives you five." 🌲
