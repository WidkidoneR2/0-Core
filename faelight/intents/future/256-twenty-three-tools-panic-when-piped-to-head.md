---
id: 256
date: 2026-09-20
type: future
title: "twenty-three tools panic when piped to head"
status: planned
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

- [ ] THE CENSUS FIRST: every binary target in the workspace, and whether its main calls
      restore_sigpipe as its first statement.
- [ ] ⭐ PROVEN BY PIPING, ONE BY ONE. Every deployed binary is run piped to `head -3` and its
      exit observed. A tool that prints fewer lines than head asked for LOOKS fine while still
      being broken -- the August note says exactly that -- so a silent run is not evidence.
- [ ] The call is the FIRST statement of main in every one, before any output, as the doc says.
- [ ] A mechanical check exists so the twenty-fourth tool cannot be written without it. The
      faelight-deadwood check for INT-195 is the model: the rule is only real once something
      else enforces it.
- [ ] nsh-test green, and a case covers the CLASS -- a tool piped to head exits 0 and prints no
      panic -- rather than covering teach.

## Relationship

Found while correcting teach for INT-253. Not folded into that intent: this is twenty-three
files and a mechanical gate, and 253 is already carrying more than its title.
