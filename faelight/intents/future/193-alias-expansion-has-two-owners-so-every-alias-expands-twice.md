---
id: 193
date: 2026-07-24
type: fix
title: "alias expansion has two owners so every alias expands twice"
status: planned
tags: [fix, bugfix, fsh, aliases, int-169, int-057]
---

## Vision

Exactly one phase owns alias expansion.

    Every alias expansion occurs exactly once.

That invariant is testable from outside the code, survives refactors, and defines
ownership without prescribing an implementation.

## The Problem

fsh expands aliases in TWO places, neither aware of the other:

  1. **main.rs ~2313** (the REPL). `db.get_alias(&first_word)` then
     `format!("{}{}", aliased, rest)`. ONE pass, NO recursion, NO cycle guard.
     Runs after glob expansion, before pipeline detection.

  2. **commands/mod.rs ~670** (`execute_impl`'s preamble). Recursive, WITH
     INT-057's `expanded_names` cycle guard. Starts at `&[]` every time, because
     the REPL's pass left no trace of itself.

So a self-referential alias expands twice. Trace for `df` = `df -h`:

    typed          df
    REPL 2313  ->  df -h
    execute_impl:  cmd=df, get_alias(df) -> "df -h", guard is EMPTY -> expands AGAIN
               ->  df -h -h
    recurse:       guard now holds "df", falls through
    runs           df -h -h

### Proven live, with a test that could fail

    alias echo="echo MARK"
    echo            ->  MARK MARK        <- two expansions
    unalias echo

Single expansion prints `MARK` once. It printed twice.

Affected aliases that exist today: `df` = `df -h`, `du` = `du -h`,
`free` = `free -h`. All harmless, because the duplicated argument is an
idempotent flag -- which is exactly why this survived unnoticed. An alias whose
argument was not idempotent would have surfaced years ago.

### ⚠️ THIS IS NOT A REGRESSION. INT-057 IS NOT AT FAULT.

INT-057 was "fsh crashes (closes terminal) on df" -- a STABILITY intent, tagged
`stability, crash`, whose vision was "fsh runs stock external commands without
ever crashing". It fixed a SIGSEGV caused by infinite recursion on a
self-referential alias, and that fix is correct and still holds.

Its guard cannot see the REPL's pass because the guard lives INSIDE
`execute_impl`. Different mechanism, adjacent bug.

And the crash MASKED this one: `df -h -h` was unobservable while `df` killed the
terminal before printing anything.

### ⚠️ An earlier diagnostic was invalid and is recorded here so it is not repeated

Running `df` and then checking `ht` showed `df` -> `df -h`, which LOOKS like
single expansion and proves nothing. `postexec` records `ctx.raw` from the OUTER
`execute_with_context` call, while `execute_impl`'s alias recursion calls itself
DIRECTLY -- so a second expansion happens entirely below the postexec boundary
and never reaches history. That evidence was silent, not negative.

## The Solution

Consolidate to ONE owner. Not yet decided WHERE -- that is design work, and the
invariant above is what the design must satisfy.

Strong prior, worth stating: alias expansion is an INPUT transformation, not an
execution concern. Aliases are defined as TEXT by the user, the implementation is
string-concat plus re-tokenize, and **bash expands aliases during tokenization,
before parsing** -- bash's alias quirks (trailing space triggering next-word
expansion) exist precisely because it is a lexical substitution. That puts it in
the same category as `!!` history expansion, which fsh already treats as
text-world.

If that holds, the single owner is the INPUT PHASE, above the point where an
executor is chosen -- and every executor then receives already-expanded text.

## Advances INT-169 blocker 6

INT-169's blocker 6 is "alias expansion" and this is that work. The spine
deliberately does NOT expand aliases today (`ExecutionMode::Spine` skips text
transforms, which is why `spine exec ll` correctly fails). Consolidating
expansion into the input phase means the spine never needs to know aliases exist
-- the architecture falls out of the fix rather than being the goal.

## Success Criteria

- [ ] ★ INVARIANT: every alias expansion occurs exactly once. Verified from
      OUTSIDE the code: `alias echo="echo MARK"; echo` prints `MARK` exactly once
- [ ] Exactly one code path performs alias expansion (enumerated, not grepped)
- [ ] INT-057's protection still holds -- a self-referential alias does not
      recurse infinitely and does not crash the shell
- [ ] Nested aliases still resolve. ⚠️ These currently work BY ACCIDENT: the REPL
      does one pass and `execute_impl` does the rest. Consolidation must preserve
      chains DELIBERATELY (`cistart` -> `core intent start` -> `core` is itself an
      alias to the absolute path) rather than let the behaviour emerge from
      having two sites
- [ ] `try_builtin` (`ExecutionMode::Probe`) still answers the same question.
      ⚠️ Text transforms are ON in Probe deliberately, because aliases are part of
      an honest answer to "would this line hit a builtin?". If expansion moves
      upstream, its callers must pass already-expanded text or the probe's ANSWER
      CHANGES MEANING -- and its caller is main.rs's redirect path, where INT-143's
      double-execution scars are
- [ ] The `expanded_names` cycle guard moves with the expansion (it is currently an
      `execute_impl` parameter threaded through the recursion)
- [ ] REPL and non-REPL execution share the same expansion logic
- [ ] fsh-test still 97/97, including `repl_173_alias_expands_at_prompt`
- [ ] Regression test added for the exactly-once invariant, so this cannot silently
      return
