---
id: 206
date: 2026-08-06
type: future
title: "fsh always starts in the forest home -- there is no way to spawn it in a chosen directory"
status: complete
tags: [fsh, shell, faelight-shell, forest]
---

## Vision
A shell you spawn in a directory should start in that directory. fsh preferring the forest home is a
good default; a default that cannot be overridden is not a default.

## The Problem
    cd /tmp && fsh          -> the prompt comes up in ~/0-core
    Command::new(fsh).current_dir("/tmp")  -> the same

Two separate overrides, both deliberate, neither consulting the directory fsh was actually spawned in:

  main.rs ~847   set_current_dir(&core_root), commented "Start in ~/0-core by default". Until
                 2026-08-07 this call was there TWICE, identically, with the comment between the two
                 copies. One was dead and has been deleted.

  main.rs ~2660  the session-memory restore. It takes mem.last_dir, then falls back to core_root
                 unless the path exists, is a directory, and is not under engine/src or rust-tools --
                 commented "Always restore to core_root, keep work in forest home".

Neither is a bug on its own. Together they mean the launch directory is unreachable.

## Why it matters more than it sounds
IT SILENTLY DEFEATS TEST ISOLATION, and it did so unnoticed for months. fsh-test spawns the shell with
current_dir("/tmp") specifically so that conformance cases which write files do not write them into
the repo. That call has never taken effect. The comment beside it explains that bash is run in /tmp
because two cases create files named 0.5 and = wherever they run, and that those files "landed in the
repo root and were committed before anyone noticed" -- and concluded that fsh was the shell that
refused them. Half of that was true. Only bash was ever protected.

On 2026-08-07 the digit guard was narrowed so fsh executes those two cases as bash does. The suite
promptly wrote both files into the repo root, which is how this was found.

## The design question
An explicitly chosen launch directory and an inherited one look identical from inside the process: a
spawned program sees a working directory and cannot tell whether the caller meant it. So "honour the
launch directory when it was chosen" is not implementable as stated, and the mechanism has to be an
explicit signal.

  ENV OPT-OUT     something like FSH_KEEP_CWD=1 suppressing both overrides. The harness sets it, daily
                  use never sees it. Smallest change; adds a second way to configure the shell.

  ARGUMENT        a flag. Explicit and discoverable, but fsh is a login shell and flags arrive from
                  places that did not mean them.

  NARROW BY ROLE  keep the forest-home default for an INTERACTIVE login session and honour the launch
                  directory otherwise. Closest to what a reader would expect, and it needs the shell
                  to know which role it is in -- which is a question INT-201 is already answering.

## A STOPGAP IS IN PLACE, AND IT IS PART OF THIS INTENT'S WORK TO REMOVE (2026-08-07)
Since the conformance corpus stopped declaring its two divergences, fsh EXECUTES those redirects --
so the files named 0.5 and = now land in the repository root on EVERY suite run, not occasionally.
They have been committed by accident before.

.gitignore therefore carries a root-anchored entry for both, with a comment naming this intent. It
hides the symptom and does not touch the cause. ⚠️ THE BLOCK IS DELETED AS PART OF CLOSING THIS
INTENT -- an ignore that outlives its bug is one nobody remembers the reason for, and the next person
to see those files appear would have no thread to pull.

## THE DECISION (2026-08-07): an explicit signal, set harness-wide, with a guardian case
THE OTHER TWO OPTIONS ARE DEAD, and one of them died on evidence rather than taste.

  Narrow by role -- keep the forest-home default for an interactive login session and honour the
  launch directory otherwise -- read best on paper. It is not implementable: fsh-test drives a REAL
  PTY, so stdin is a terminal inside the harness too. An is_terminal check cannot separate a login
  shell from a test, and the test is the case this intent exists for.

  A flag was rejected because fsh is a login shell, and flags arrive from places that did not mean
  them.

SO THE MECHANISM IS AN ENVIRONMENT VARIABLE, following the convention already in the code --
FSH_CONFIG, FSH_SPINE, FSH_SESSION_ID. When set, both overrides are suppressed and the shell stays
where it was spawned. Unset, nothing changes: the forest-home default is deliberate, documented in
the code, and what its owner wants for daily use.

TWO SITES ARE SUPPRESSED, AND A THIRD IS DELIBERATELY NOT. The startup default and the session-memory
restore both move the shell to the forest home. The third place that changes directory is the file
manager handoff -- fsh follows yazi out to wherever you left it -- and that is a feature with nothing
to do with this intent.

THE HARNESS SETS IT ONCE, FOR EVERY CASE, and that is a choice with a cost worth stating. Isolation
becomes the default, so a case added later cannot pollute the repository by forgetting to opt in --
which is the failure mode this intent is about, reintroduced through omission. The cost is that every
case then runs a shell configuration daily use never runs, which is the same silent divergence that
made INT-204's isolation accidental.

⭐ SO ONE GUARDIAN CASE UNSETS IT AND ASSERTS THE FOREST-HOME DEFAULT. The behaviour daily use gets is
covered by a case that says out loud what it is testing, rather than by a hundred cases inheriting an
override nobody mentions. That is the INT-204 lesson applied on purpose instead of discovered later.

## Success Criteria
- [x] RED FIRST, RECORDED: `cd /tmp && fsh` lands in the forest home, captured before any change, and
      the same for a spawned process with an explicit working directory.
<!-- evidence: captured 2026-08-07 before any change. A process spawned with an explicit working
     directory of /tmp reported /home/christian/0-core. ⚠️ Use an ABSOLUTE binary path when setting
     cwd -- a relative one resolves against the new directory, which cost one attempt. -->
- [x] The mechanism is CHOSEN AND WRITTEN DOWN before implementation, with the reason the other two
      were not.
<!-- evidence: the decision section above. FSH_KEEP_CWD, following FSH_CONFIG and FSH_SPINE. A flag
     was rejected because fsh is a login shell and flags arrive from places that did not mean them.
     Narrow-by-role was rejected on EVIDENCE rather than taste: fsh-test drives a real pty, so stdin
     is a terminal in the harness too and is_terminal cannot separate a login shell from a test. -->
- [x] The forest-home default is UNCHANGED for an ordinary interactive session. This intent is about
      adding a way out, not removing the behaviour.
<!-- evidence: demonstrated 2026-08-07. Spawned in /tmp: unset -> /home/christian/0-core, and the
     literal string 0 -> /home/christian/0-core. An ordinary session still WRITES its directory to
     session_state, confirmed by query -- suppressing that for everyone would have been a different
     bug wearing this fix's clothes. -->
- [x] fsh-test's current_dir actually takes effect: a conformance case that writes a file writes it
      where the harness said, and the repo stays clean across a full run.
<!-- evidence: FSH_KEEP_CWD=1 spawned in /tmp reported /tmp AND left session_state.last_dir with a
     row count of zero, so the opt-out session both stayed put and wrote nothing. Full suite run
     afterwards: git status shows no 0.5 and no = in the repository. -->
- [x] ⭐ PROVEN BY THE CASE THAT FOUND IT: run the suite with the two redirect-writing cases and show
      the repo has no untracked 0.5 or = afterwards. Watch it fail first against the current build.
<!-- evidence: repl_206_forest_home_is_still_the_default was added to cover the default the harness
     now opts out of, and immediately caught a bug in a different part of this change -- the opt-out
     session was still SAVING its directory, so an ordinary session restored it afterwards. 136/136
     with it green. The gitignore stopgap was then DELETED and the suite re-run: still clean, so the
     proof is not resting on an ignore that was covering for it. -->
- [x] The session-memory restore and the startup default agree about precedence -- one place decides
      where the shell starts, not two that happen to reach the same answer.
<!-- evidence: one predicate decides -- keep_launch_cwd in the crate root -- and all three affected
     sites ask it: the startup default, the session-memory restore, and the session-memory SAVE. The
     save was the one that mattered and was missed on the first pass. The file manager handoff in
     engine.rs is deliberately NOT guarded: following yazi out is a feature, not an override. -->
- [x] Each gate carries evidence per INT-158.
<!-- evidence: every gate above cites a demonstration or a query. The behavioural ones were watched
     failing first -- the red capture, and the guardian case going red before green. -->

## Scope guardrails
- NOT a removal of the forest-home default. It is deliberate, it is documented in the code, and it is
  what the shell's owner wants for daily use.
- NOT a fix at the call site. Making fsh-test cd somewhere and hope, or passing a path the shell then
  ignores, would leave the next caller with the same surprise.
- The duplicate set_current_dir call is already deleted; that was housekeeping found on the way, not
  this intent's work.

## Relationship
- Found 2026-08-07 while narrowing the digit guard: the conformance corpus stopped declaring its two
  divergences, both cases began executing as redirects, and the files appeared in the repo root.
- INT-201 owns the question of which role the shell is running in, which the third option depends on.
- The fsh-test comment that half-protected against this is at main.rs ~1106 and now says so.
