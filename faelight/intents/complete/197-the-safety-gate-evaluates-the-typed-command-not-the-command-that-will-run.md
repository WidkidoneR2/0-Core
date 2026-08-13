---
id: 197
date: 2026-07-25
type: arch
title: "the safety gate evaluates the typed command, not the command that will run"
status: complete
tags: [architecture, rust, design]
---

## Vision
The safety gate evaluates the representation that will actually execute.

## The Problem
safety_guard::check has two call sites, main.rs:1175 and main.rs:1217. Alias
expansion is at main.rs:2311. Both calls are far above it, so the guard inspects
the line as TYPED -- after comment-stripping, history expansion, normalize and
brace expansion, but before variables, subshells, globs and aliases.

So an alias whose expansion is a gated command is not gated. Given
`alias zap='rm -rf /'`, the guard's first word is `zap`, which matches no deny
entry, no allow entry, no safe entry, and fails its own `first_word == "rm"`
test. It returns None. The executor then expands and runs it.

PROPORTION, stated so future readers do not over-scope this: fsh is a
single-user personal shell and this gate is an "are you sure" confirmation, not a
boundary defending against an attacker. No alias in the current 285 expands to
`rm -rf`. This is a real gap worth closing, not an emergency.

## Why this is NOT INT-195
INT-195 owns HOW the command word is derived -- an implementation inconsistency,
fixed by routing through commands::command_word(). This intent owns WHICH
REPRESENTATION the gate evaluates, and WHEN it sees it. That is a policy
decision, not a parsing one, and folding them together would let the policy
question be quietly answered by whoever happened to be fixing the derivation.

## ⚠️ Moving the call is not the obvious fix
The allow, deny and safe lists all match BARE NAMES. `safe` contains entries like
core, git, cargo, cat, ls, cd. If the gate ran after expansion it would see
expanded text, and `d` expands to `/run/current-system/sw/bin/core doctor run`,
whose first word is an absolute PATH -- so it would fall out of the safe list and
start gating a harmless doctor run. Any placement change has to reconcile the
lists with whatever representation is chosen. That reconciliation IS most of the
work; the call move is the small part.

## The actual question
Which representation should the gate evaluate?
  - AS TYPED: what it does today. Honest about intent, blind to aliases.
  - CANONICAL COMMAND WORD: quote-aware, still blind to aliases.
  - POST-EXPANSION: sees what will run, but breaks bare-name list matching and
    raises the same question again for plugins.
  - THE EXECUTION PLAN: the eventual answer once INT-169's spine owns execution,
    since the plan is precisely "what will run" in structured form.
Choosing among these is the intent. Do not open by moving the call.

## THE RULING (2026-08-12)

POST-EXPANSION REPRESENTATION. The gate evaluates the alias-expanded segment, and the executable
identity is NORMALIZED ONCE at the policy boundary. The three lists stay bare-name, because by the
time they see the word it already is one.

WHAT IS GIVEN UP, stated as the intent demands:
  Policy identity is the post-expansion executable, normalized to its basename at the policy
  boundary. Arguments are NOT resolved recursively -- including scripts passed to interpreters.
  So `rebuild`, which expands to bash running a script, is identified as `bash`. That is the
  INTENDED boundary rather than an accidental lossy case: if the guard starts interpreting scripts,
  wrappers or aliases semantically, it has crossed from executable policy into command
  interpretation. Distinguishing `bash <known-script>` from arbitrary `bash` is a NEW CAPABILITY
  requiring a richer command identity, not something to smuggle into basename normalization.

WHY NOT THE OTHER OPTIONS, measured rather than argued:
  AS TYPED is today and is blind to aliases -- the gap this intent exists to close.
  CANONICAL COMMAND WORD is INT-195/196 and already landed. It fixes quoting, not aliases.
  A CANONICAL EXECUTABLE IDENTITY consumed by the lists would be better, and it has NOWHERE TO LIVE:
  a full scan found no such abstraction. Every file_name() call in the shell is a directory-entry
  walk for completion, listings or digests, and program_on_path returns a BOOL -- it answers
  reachability, not identity. Building one is a real abstraction gap, the same shape as INT-216.
  TEACHING THE THREE LISTS to recognise absolute paths is rejected outright: three copies of one
  normalization rule, which is the disease this codebase has spent a month removing.

SIZE, measured from shell_aliases rather than the rendered alias table: 284 aliases, THREE expand to
an absolute path -- d, core, and rebuild. The reconciliation the intent calls most of the work is
three rows, because normalization happens once rather than per list.

## Success Criteria
- [x] The representation is CHOSEN and RECORDED with reasoning, including what is
      given up. A decision, not a default
<!-- evidence: the ruling above. Post-expansion, normalized once at the policy boundary, lists stay
     bare-name, and rebuild is intentionally identified as bash. The alternatives are recorded with
     the measurement that rejected each rather than a preference. -->
- [x] The allow, deny and safe lists are reconciled with that representation, or
      the mismatch is recorded as a stated limitation
<!-- evidence: reconciled by NORMALIZING ONCE rather than teaching three lists to recognise paths.
     policy_identity at main.rs:217 takes the basename when the word contains a separator.
     Measured from shell_aliases, NOT the rendered alias table: 284 aliases, THREE expand to an
     absolute path -- d, core, rebuild. All three normalize to a bare name the lists already
     hold, so NO LIST CHANGED AT ALL. The reconciliation the intent called most of the work is
     three rows, because normalization happens once instead of per list. -->
- [x] PROVEN: an alias whose expansion is a gated command is gated. Watched
      failing first, on a throwaway alias, never on a real destructive one
<!-- evidence: red-first, gen 493 against this tree, throwaway alias, payload mkfs.zzz.
     DEPLOYED: alias then invoke reports command-not-found with NO guard output -- the gap.
     THIS TREE: the same input produces CHALLENGE and blocks.
     The payload is deliberate: the guard matches on the word alone and no such binary exists,
     so a failed guard costs command-not-found rather than damage. -->
- [x] REGRESSION PROVEN: commands that are safe today are still not gated --
      specifically the bare-name entries reached through aliases, `d` being the
      known hard case
<!-- evidence: `d` runs the doctor with NO challenge. It expands to a store path ending in core, so
     without normalization it would fall out of the safe list and start prompting on a harmless
     doctor run -- exactly what the intent warned about. `core` and `ls` also produce zero
     challenges. -->
- [x] The interaction with INT-195 is stated: derivation and placement touch the
      same function, and whichever lands second must not silently undo the first
<!-- evidence: INT-195 and INT-196 landed FIRST, and this intent builds on them rather than
     colliding. INT-195 fixed HOW the word is derived, routing it through the canonical quote-aware
     command_word. INT-196 M5 then moved the derivation OUT of safety_guard entirely: the signature
     is check(cmd, first_word), so the guard performs no tokenization at all and grep proves it.
     THAT MADE THIS INTENT SMALLER, not harder. Because the caller already supplies the word, the
     placement question changed from "move the call" -- which the intent warned against -- to
     "derive the word from a different string". The guard site does not move, so INT-196 M8s
     multi-line-paste property is untouched, and INT-195 derivation is not undone: the expanded copy
     still goes through guard_command_word, which is the same authority hierarchy.
     ⚠️ THE ORDER THAT WOULD HAVE HURT is the reverse one. Had this landed first, moving the call
     below expansion, INT-196 M5 would then have been changing the signature of a guard whose
     placement had just shifted -- two changes to the highest-stakes function with no way to tell
     which broke what. Recorded because the intent asked for the interaction, not just the outcome. -->
- [x] The gate behaviour is covered by a REPL test, since this is interactive behaviour
      and the dash-c door does not reach fsh own dispatch at all
<!-- ⏸ BLOCKED ON THE HARNESS, and deliberately NOT ticked. State, precisely:
     CORRECTED 2026-08-12, AFTER this note was written. The gate IS covered by a REPL test:
     repl_197_an_alias_expanding_to_a_gated_command_is_gated, 152 of 152, two lines through
     run_repl_answered_after -- alias defined on one line, invoked on the next.
     GHOST-CHECKED: reverting the guard to judge the TYPED line turns that case RED at the wait_for
     timeout while both repl_195 guard cases and all seven repl_193 alias cases stay GREEN. That
     split proves it tests the alias fix rather than the guard in general.
     AND THE GUARD SAID SO ITSELF. A log at the decision point recorded, for the invocation line:
     typed=zzq219 expanded=mkfs.zzz word=Some(mkfs.zzz). The definition line recorded word=alias,
     correctly not gated, and an operator-leading line recorded None, which is INT-196 M4 as ruled.
     WHY IT WAS RECORDED AS BLOCKED, and the cause was NOT the harness. Every symptom -- the case
     timing out, a trace printing nothing, log files that did not exist, a ghost-check that failed
     to discriminate -- came from PIPING THE SUITE THROUGH head OR grep. Closing the pipe kills
     fsh-test partway with a broken-pipe panic, so runs that looked complete had stopped before
     reaching the cases in question. Measured to a FILE instead, everything resolved at once.
     The original note is kept below per INT-027 rather than deleted.
     SUPERSEDED FROM HERE:
     BEHAVIOUR: proven MANUALLY through the REPL. Piping `alias zzq197=mkfs.zzz` then `zzq197` to
     the shell produces the alias confirmation, then CHALLENGE, then blocked. The property holds.
     AUTOMATED DOOR: retained but UNCALLED. run_repl_answered_after exists, carries
     #[allow(dead_code)] and a provenance comment, and sits on an independently justified fix --
     run_session applies the answer to the LAST line rather than every line, which is the limitation
     the single-command door already documented and which also blocks INT-196 M8.
     HARNESS LIMITATION: the same two lines TIME OUT at the wait_for boundary inside the pty, at
     20.9 seconds, while succeeding when piped by hand. A trace at the submit loop produced NO
     output despite the site being present in source and the binary rebuilt, so the cause is not
     located. Three investigative cycles were spent and stopped there deliberately.
     NO CLAIM OF AUTOMATED VERIFICATION IS MADE. This gate stays open until the harness can drive
     the case, which is its own piece of work and blocks two intents rather than one. -->
