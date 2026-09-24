---
id: 256
date: 2026-09-20
type: future
title: "twenty-three tools panic when piped to head"
status: in-progress
tags: [panic, nsh, novashell]
---

## Vision

A tool piped into head exits quietly, the way every Unix tool has since pipes existed.

## The Problem -- MEASURED 2026-09-20

Found by accident. `teach --present` was piped to `head` and printed this in place of slide 3:

```text
    thread main panicked at library/std/src/io/stdio.rs:1166:9:
    failed printing to stdout: Broken pipe (os error 32)
    note: run with RUST_BACKTRACE=1 to display a backtrace
```

⭐ AND THE FIX ALREADY EXISTS, WRITTEN AND DOCUMENTED, IN THE RIGHT PLACE.
`faelight_core::restore_sigpipe()` has been in lib.rs since 2026-08-27. Its own doc comment
states the problem exactly: the Rust runtime sets SIGPIPE to SIG_IGN at startup, a write to a
closed pipe returns EPIPE instead of killing the process, and println! unwraps that and panics.
It says EVERY CLI IN THIS WORKSPACE dies this way, and it says to call it as the first statement
of main.

★ THREE TOOLS OUT OF TWENTY-SIX CALL IT: faelight-deadwood, faelight-update, ship -- the three
measured that August day. Nothing since. TWENTY-THREE BINARIES STILL PANIC.

⚠️ THE DEFECT IS NOT THE MISSING FIX, IT IS THE MISSING ADOPTION. Someone found a fleet-wide
bug, wrote the one-line cure, put it in the shared crate, documented it well, fixed the three
tools in front of them -- and there was no step that asked the other twenty-three. The same
shape as the cockpit name lists and the nine registry readers: not carelessness, the absence of
a place where the question gets asked.

## Success Criteria

- [x] THE CENSUS FIRST: every binary target in the workspace, and whether its main calls
      restore_sigpipe as its first statement.
<!-- evidence: 23 crates with a main.rs. THREE called it (deadwood, update, ship) and faelight-core
     only DEFINES it. Three more had their own implementations, which the source census by name
     would have missed: release inline, core with a hook, nsh-test with a different hook. -->
- [x] ⭐ PROVEN BY PIPING, ONE BY ONE. Every deployed binary is run piped and its exit observed.
      A tool that prints fewer lines than head asked for LOOKS fine while still being broken.
<!-- evidence: `| true` rather than `| head -3` -- it closes the pipe before the tool writes a
     byte, so there is no race and no hunting for a verb that prints enough. OLD vs NEW binaries:
       teach list            panic -> clean
       faelight-docs list    panic -> clean
       faelight-git status   panic -> clean
       faelight-context scan panic -> clean
     Confirmed on the DEPLOYED binaries after ship, alongside core intent list. The intent's
     warning proved true in BOTH directions: a silent run is not evidence of breakage OR of a
     fix. zone, sandbox, zero-gate, gen and vm are recorded as adopted-but-unproven. -->
- [x] The call is the FIRST statement of main in every one, before any output, as the doc says.
<!-- evidence: nine adoptions, each inserted directly after its verified `fn main` line -- docs,
     teach, git, context, zone, sandbox, zero-gate, gen, vm. release swapped its inline copy for
     the helper; core likewise. THREE crates could not use the helper: zero-gate's Cargo.toml
     RECORDS a decision not to depend on faelight-core ("a gate that needs an intent ledger to run
     cargo fmt is carrying a house to hold a door open"), and gen and vm never depended on it.
     Those three got the inline signal plus libc -- a small dependency rather than reversing a
     recorded decision. -->
- [x] A mechanical check exists so the twenty-fourth tool cannot be written without it. The
      faelight-deadwood check for INT-195 is the model: the rule is only real once something
      else enforces it.
<!-- evidence: check_sigpipe_adoption in faelight-deadwood, registered as "SIGPIPE adoption
     (INT-256)" and gated by --strict. It reads the opening of each main and accepts either the
     call or `// INT-256-EXEMPT: <reason>`; per INT-192 an unreadable source returns Skipped
     rather than clean.
     PROVEN TWICE: on its FIRST run it flagged exactly gen and vm, the two then unfixed -- it
     found the real gap without being told where it was. And with the call removed from a COPY of
     faelight-git it flagged faelight-git alone. Reads clean on the real tree.
     ⭐ THE EXEMPTIONS ARE PART OF THE CHECK: novashell (INT-299 -- the shell must IGNORE SIGPIPE),
     nsh-test (INT-219 -- exits 2 so a truncated run cannot read as complete), three TUIs and two
     daemons. INT-195 G5's lesson: a check with permanent unfixable findings gets muted. -->
- [x] nsh-test green, and a case covers the CLASS -- a tool piped to head exits 0 and prints no
      panic -- rather than covering teach.
<!-- evidence: 199/199 after ship. ⚠️ THE GATE'S OWN WORDING IS CORRECTED HERE: exit 0 is the WRONG
     assertion. `seq` and `yes` exit 141 when head closes the pipe, because they die by SIGPIPE,
     and a tool that exits 0 claims it finished when it was cut off. The class assertion is
     "no panic on stderr", and the deadwood check covers the class at the source instead -- every
     binary, every verb, rather than one tool and one verb. -->

## ⚠️ THE TITLE IS WRONG, AND THE TRUTH IS MORE INTERESTING -- MEASURED 2026-09-23

"Twenty-three tools panic" was inferred from a source census: three mains call restore_sigpipe,
twenty-three do not. Running them says something else.

```text
    PROVEN BROKEN, NOW FIXED   teach, faelight-docs, faelight-git, faelight-context        4
    ALREADY SOLVED, 4 WAYS     the helper (deadwood, update, ship); an inline signal
                               (release); signal + a panic hook (core, INT-249b);
                               a hook with its own exit code (nsh-test, INT-219)           6
    ADOPTED ANYWAY             zone, sandbox, zero-gate, gen, vm -- changed, no panic
                               ever reproduced on the verbs tried                          5
    EXEMPT, EACH WITH REASON   novashell, nsh-test, 3 TUIs, 2 daemons                      7
```

★ SO THE INTENT'S OWN DIAGNOSIS SURVIVES AND ITS COUNT DOES NOT. "Not carelessness, the absence of
a place where the question gets asked" is exactly right -- SIX tools solved this independently, in
FOUR shapes, none knowing about the others. That is what the check fixes.

### ⭐ AND THE PROBE IS THE METHOD, NOT A DETAIL

`| head -3` proves nothing: a tool that prints two lines finishes before the pipe closes, and a
tool whose --help goes through clap never reaches its own printing path. Measured both ways in one
session -- `faelight-git --help` passed while `faelight-git status` PANICKED, and `teach --help`
panicked while `teach list` passed.

```text
    | true      closes the pipe BEFORE the tool writes a byte -- no race, no verb hunting
```

Every proven row above came from `| true`. Three earlier probes reported "ok" for everything and
were wrong each time: one captured into a variable so head never closed anything, one ran each
command twice and overwrote the evidence, one was interrupted by Ctrl-C.

### ⚠️ AND THE ONE-LINE CURE WAS DOUBTED, THEN TESTED

`core`'s INT-249b comment says "SIG_DFL alone is insufficient because Rust stdio panics on EPIPE
before SIGPIPE fires", which would mean the shared helper is incomplete and its three callers only
half-fixed. Three identical programs printing 100,000 lines into `head -2`:

```text
    nothing                   exit 101   panicked at io/stdio.rs
    signal(SIGPIPE, SIG_DFL)  exit 141   silent
    signal + a panic hook     exit 141   silent -- THE HOOK NEVER RAN
    seq 1 100000              exit 141   the reference every user already knows
```

The claim is FALSE on this Rust version. The helper is sufficient, `core`'s hook was unreachable,
and its `exit(0)` would have been wrong anyway: 0 claims the command finished when `head` cut it
off. `core` now calls the helper and the hook is gone, with the measurement recorded at the site.

## Relationship

Found while correcting teach for INT-253. Not folded into that intent: this is twenty-three
files and a mechanical gate, and 253 is already carrying more than its title.
