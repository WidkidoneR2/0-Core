---
id: 193
date: 2026-07-24
type: fix
title: "alias expansion has two owners so every alias expands twice"
status: complete
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

### SECOND SYMPTOM, PROVEN 2026-07-25: expansion DESTROYS QUOTING
`cin` = `core intent new`. Two lines carrying an identical quoted title:
    cin future arch "every stage consumes the previous stage output, ..."
        -> error: unexpected argument 'stage' found
    core intent new future arch "every stage consumes the previous stage output, ..."
        -> created intents/future/195-....md
Same arguments. One path through alias expansion, one not. The alias path lost
the quoting.
MECHANISM, found by reading both sites rather than reasoning about them:
  - Site 1 (main.rs) is INNOCENT. It takes the remainder with
    `line.split_once(' ').map(|x| x.1)` off the RAW line and concatenates it
    verbatim, so quotes survive that pass.
  - Site 2 (execute_impl) builds `format!("{} {}", aliased, args.join(" "))`.
    `args` are ALREADY TOKENIZED, so the quotes are gone before that line runs.
    Joining them with spaces and re-tokenizing splits one quoted argument into N
    bare ones. The comment directly above it says so: alias expansion produced
    new text, so it is re-tokenized here.
WHY cin HITS IT AND core intent new DOES NOT: after the REPL pass, cin has become
`core ...`, and `core` is itself an alias, so site 2 fires. Typing `core`
directly leaves the absolute path as the command word, which is not an alias, so
site 2 never runs.
NESTED CHAINS ARE THEREFORE WHERE QUOTES DIE. The gate below calling nested
aliases "work by accident" is too generous: they work for argument-free chains
and corrupt quoted arguments. Every c* and int* alias routing through `core` is
affected.
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
- [x] EXPANSION PRESERVES QUOTED ARGUMENTS. Reproducer, proven 2026-07-25:
      `cin future arch "a b c"` must behave identically to
      `core intent new future arch "a b c"`. Today the first fails and the
      second succeeds. Consolidation probably fixes this, but exactly-once does
      NOT guarantee it (a single-site implementation could still join and
      re-tokenize), so it is gated separately
      <!-- DONE 2026-07-25 gen 432. repl_193_nested_alias_preserves_quoting was RED on gen 431
      (a.b. for one quoted argument), GREEN after 9d533787. By hand on the deployed shell:
      alias zzq1='printf %s.'; alias zzq2='zzq1'; zzq2 "a b" -> a b. -->

- [x] ★ INVARIANT: every alias expansion occurs exactly once. Verified from
      OUTSIDE the code: `alias echo="echo MARK"; echo` prints `MARK` exactly once
      <!-- DONE 2026-07-25 gen 432. repl_193_expansion_happens_exactly_once holds the original
      reproducer -- alias echo='echo MARK193'; echo must not print the marker twice. -->
- [x] Exactly one code path performs alias expansion (enumerated, not grepped)
      <!-- DONE 2026-07-25, ENUMERATED. ONE definition: commands::expand_aliases. ONE caller:
      main.rs's prompt path where site 1 stood. execute_impl's block is DELETED, not disabled.
      try_builtin has one caller and no longer expands, since Probe reached the block that is
      gone. fsh -c is not a third path: both handlers spawn sh and never enter fsh execution. -->
- [x] INT-057's protection still holds -- a self-referential alias does not
      recurse infinitely and does not crash the shell
      <!-- DONE 2026-07-25 gen 432. repl_193_self_referential_alias_survives: zzloop='zzloop -h'
      expands once, the guard stops it, terminates as command-not-found. The guard is an owned
      Vec<String> inside expand_aliases, checked every round. -->
- [x] Nested aliases still resolve. ⚠️ These currently work BY ACCIDENT: the REPL
      does one pass and `execute_impl` does the rest. Consolidation must preserve
      chains DELIBERATELY (`cistart` -> `core intent start` -> `core` is itself an
      alias to the absolute path) rather than let the behaviour emerge from
      having two sites
      <!-- DONE 2026-07-25 gen 432. Now DELIBERATE: expand_aliases loops until the command word
      is not an alias, so chains resolve in one owner rather than emerging from two.
      repl_193_alias_chain_resolves proves a three-deep chain. -->
- [x] `try_builtin` (`ExecutionMode::Probe`) still answers the same question.
      ⚠️ Text transforms are ON in Probe deliberately, because aliases are part of
      an honest answer to "would this line hit a builtin?". If expansion moves
      upstream, its callers must pass already-expanded text or the probe's ANSWER
      CHANGES MEANING -- and its caller is main.rs's redirect path, where INT-143's
      double-execution scars are
      <!-- DONE 2026-07-25. YES, WITH ONE DELIBERATE EXCEPTION. The precondition was already
      met: cmd_part reaches try_builtin ALREADY EXPANDED (no `let line` rebinding between
      main.rs 2323 and 2441), so removing the probe's expansion is a no-op for its answer.
      EXCEPTION: `cat` under redirect is both an alias and a builtin. The probe used to expand
      cat->bat and answer NotBuiltin, so /bin/cat ran BY ACCIDENT; unexpanded it matches the
      builtin, whose returned string gained a trailing newline (9 bytes in, 10 out). BUG-298-4's
      bypass now skips the builtin probe too. repl_193_cat_redirect_output_matches_source was
      baselined GREEN before the fix and is GREEN after -- bounded and tested, not discovered. -->
- [x] The `expanded_names` cycle guard moves with the expansion (it is currently an
      `execute_impl` parameter threaded through the recursion)
      <!-- DONE 2026-07-25. The ALIAS guard moved: an owned Vec<String> local to expand_aliases,
      no longer threaded through execute_impl's recursion. The expanded_names PARAMETER stays,
      because PLUGIN expansion uses the same INT-057 guard and plugins are untouched (INT-170). -->
- [x] REPL and non-REPL execution share the same expansion logic
      <!-- DONE 2026-07-25 -- and recon changed what this gate means. There IS no non-REPL fsh
      execution: both -c handlers (main.rs 588, repl_main 651) spawn sh and exit, so fsh's parser,
      dispatch and aliases never run there. Satisfied because exactly one expansion path exists
      and every path that executes fsh commands goes through it. NOTE: the two -c handlers use
      DIFFERENT matching rules (positional vs position-anywhere) -- the two-owner shape one layer
      up, deliberately untouched because -c is login-shell compatibility. Its own intent. -->
- [x] fsh-test still 97/97, including `repl_173_alias_expands_at_prompt`
      <!-- DONE 2026-07-25 gen 432. 104/104 via bare fsh-test -- the DEPLOYED harness against the
      DEPLOYED shell, per INT-110; cargo run does not count as the gate. Includes
      repl_173_alias_expands_at_prompt. 97 became 104 because this intent added seven tests. -->
- [x] Regression test added for the exactly-once invariant, so this cannot silently
      return
      <!-- DONE 2026-07-25. Seven Category::Repl tests: expansion_happens_exactly_once,
      nested_alias_preserves_quoting, direct_alias_preserves_quoting, alias_chain_resolves,
      self_referential_alias_survives, redirect_from_alias_value,
      cat_redirect_output_matches_source. Committed RED first (cdae8111, 999270fb) so the history
      shows the bug reproduced before the fix -- INT-158's watch-it-fail-first. -->
