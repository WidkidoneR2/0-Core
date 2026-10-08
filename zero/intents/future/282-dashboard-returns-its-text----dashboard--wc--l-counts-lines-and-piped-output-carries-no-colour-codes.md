---
id: 282
date: 2026-10-08
type: future
title: "dashboard returns its text -- dashboard | wc -l counts lines, and piped Output carries no colour codes"
status: planned
tags: [nsh, pipeline, builtins, dashboard, ansi, finding, fix]
---

## Vision

dashboard is a builtin that feeds a pipe. dashboard | wc -l counts the dashboard's lines,
dashboard | grep Health finds the line, and what crosses a pipe is plain text whichever door
the command came through -- the same rule to_pipe_text already keeps for Value.

## The Problem

F-0022 (c65db701). Since INT-281 (d191e2ac), dashboard leading a pipeline refuses with exit 1:
it prints to the terminal and returns Empty, so the pipe would get nothing. The refusal is
honest; the builtin is still not a pipe source.

Measured 2026-10-08 (recon, read-only):
- dashboard_cmd mod.rs:15387 dispatches system / overview / full; dashboard_system 15401 and
  dashboard_overview 15501 hold 17 println! calls and return Empty (15498, 15594). Their only
  callers are inside dashboard_cmd; the dispatch arm is mod.rs:1383.
- Every place an Output reaches the terminal prints it with println!("{}", out): engine.rs:231,
  776, 803, 986, 1582, 1687, 2151 and main.rs:3021. A String holding the same lines displays
  the same.
- colored 2.2.0 (control.rs:83-105) decides once per process: CLICOLOR (default on) AND the
  shell's own stdout is a terminal; NO_COLOR and CLICOLOR_FORCE override. In the REPL colour
  is on, so an Output string carries escape codes into the pipe; under nsh -c with a piped
  stdout it is off. The same line pipes different bytes through the two doors.
- Value already pipes uncoloured text (to_pipe_text, value.rs:90). Output does not:
  Peeled::Piped(&plans[1..], text) passes it as is (mod.rs:9842).
- strip_ansi exists twice, privately: health_tui.rs:194 and pty_exec.rs:109.
- nsh-test uses dashboard as its printing stand-in four times: INT-243 at 785 (closed stdout,
  exit 141) and 799 (dashboard | head -4, shell alive); INT-281 at 818 and 836 (the refusal).
  The two INT-281 cases stop guarding the refusal the moment dashboard pipes.

## The Solution

1. dashboard_system and dashboard_overview build a String (writeln! where println! was) and
   dashboard_cmd returns Output. Display is unchanged because every Output site println!s it.
2. The two INT-281 tests move to run --list (mod.rs:16066-16100): it prints, returns Empty,
   changes nothing, carries no quoted argument. The refusal keeps a test that can fail.
3. Colour, ruled in G3 among:
   (a) leave it: piped Output carries escapes in the REPL and not under -c.
   (b) strip at the peel: the Output arm of peel_builtin_first_stage pipes strip_ansi(text);
       the two private copies become one shared helper. Class-wide: every Output builtin.
       Recommended -- one place, matches to_pipe_text, makes the two doors agree.
   (c) something narrower found by the G3 census.

## Non-goals

- The other printing builtins from INT-281's census (snap-diff, histogram, run, query, ...).
  Each stays refused until its own change.
- Redirecting a builtin (dashboard > file). A different path; not measured here.
- Any global colour switch, colored::control override, or change to NO_COLOR handling.
- How Value pipes. to_pipe_text is the model, not the subject.
- Any fd-level redirection of the shell's stdout. Nothing here touches a descriptor (INT-277).

## Scope against neighbours

INT-281 stays: Empty still refuses; only dashboard stops returning Empty. INT-243's two cases
keep dashboard and must stay green -- the closed-stdout case now reaches EPIPE through the
Output println! instead of the builtin's own. INT-277 is untouched: no stdout redirection.

## Dependencies

depends_on is empty on purpose: INT-281 is complete and F-0022 is filed.

## Success Criteria

Each gate is watched failing before it is watched passing. Anchors stay ASCII-only.

- [ ] G0: recon recorded with file:line -- the three dashboard functions and their callers,
      the Output display sites, colored 2.2.0 control.rs, both strip_ansi copies, the four
      nsh-test uses of dashboard, and run --list as the stand-in.
- [ ] G1 RED FIRST, BOTH DOORS: on the current build, dashboard | wc -l refuses with exit 1,
      captured verbatim in the REPL and through nsh -c, and the new count tests (G7) are
      watched failing against it.
- [ ] G2 STAND-IN: the two INT-281 tests use run --list instead of dashboard, watched red
      against a pre-281 build (a worktree at 1c0b19f2, its own target dir, removed after)
      and green on the current one -- so the refusal keeps a test that can fail.
- [ ] G3 THE COLOUR RULING among (a), (b) and (c), recorded with its reason and a census of
      the Output builtins whose text carries escape codes in the REPL.
- [ ] G4 DISPLAY UNCHANGED: dashboard, dashboard system and dashboard overview show the same
      lines and labels on the terminal before and after, compared with the volatile numbers
      masked (load, memory, processes, disk, commits, events), through both doors.
- [ ] G5 on the DEPLOYED binary, through both doors: dashboard | wc -l prints a count equal to
      the dashboard's own line count with exit 0, and under (b) dashboard | cat -v shows no
      escape sequence in the REPL.
- [ ] G6 NO REGRESSION: both INT-243 cases green (785 still exits 141, 799 shell alive), the
      INT-281 refusal holds through run --list on both doors, history | head, logs | head,
      alias | head and printf | head unchanged in count, nsh-test all passing, d 0 failed.
- [ ] G7 regression tests in nsh-test beside the INT-243 and INT-281 cases: dashboard | wc -l
      equals the dashboard's line count through run_fsh and a Category::Repl case; under (b),
      a piped Output carries no escape byte with colour forced (CLICOLOR_FORCE=1 through
      run_fsh_env), watched failing before the strip lands.
- [ ] G8 each gate carries evidence per INT-158.
