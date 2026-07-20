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
- [ ] ONE tokenizer exists. Prove it: grep the tree, exactly one quote-aware tokenizer function, and
      the duplicate is DELETED (not deprecated)
- [ ] No code outside the parsing entry point calls split_whitespace() on a user-typed line. Prove
      it with the grep, and with each remaining call site justified in a comment
- [ ] The six INT-143 regressions have tests that FAIL on the pre-171 parser and pass after --
      demonstrated, not declared (INT-158)
- [ ] `tracing` spans cover lexer / parse / expansion / dispatch / execution, and ONE typed command
      can be traced end to end. Owner agreed with INT-167 first, in writing
- [ ] `thiserror` types the errors that control flow depends on. The 968c7be5 class -- a result whose
      TYPE contradicts its MESSAGE -- is unrepresentable, not merely fixed
- [ ] `miette` renders one real fsh syntax error with a caret under the offending character
- [ ] 18/18 existing tests still pass; fsh still boots, still logs in, still deploys
- [ ] NOTHING from INT-168/169/170 landed here. If a change needs an AST, it is 169s change

## The Rule
"fsh already had a correct tokenizer. Twice. The bugs were the code that did not call it. Adding a
better parser to a shell with four parsers gives you five." 🌲
