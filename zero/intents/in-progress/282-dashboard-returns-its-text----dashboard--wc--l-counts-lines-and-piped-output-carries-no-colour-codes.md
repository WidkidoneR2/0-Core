---
id: 282
date: 2026-10-08
type: future
title: "dashboard returns its text -- dashboard | wc -l counts lines, and piped Output carries no colour codes"
status: in-progress
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
4. Piped Output ends with a newline. Measured 2026-10-08: alias prints 244 lines but
   alias | wc -l says 243. The peel writes Output text as returned (mod.rs:9737-9739), and
   Output text has no final newline because the display sites println! it. The Output arm
   terminates the text, the convention to_pipe_text keeps (value.rs:108, 123). Added
   2026-10-08, ruled by Christian.

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

- [x] G0: recon recorded with file:line -- the three dashboard functions and their callers,
      the Output display sites, colored 2.2.0 control.rs, both strip_ansi copies, the four
      nsh-test uses of dashboard, and run --list as the stand-in.
<!-- evidence: recon 2026-10-08, read-only (script sha256 becaa256d277, the colored check inline). zero/shell/novashell/src/commands/mod.rs: dashboard_cmd 15387 (system, overview, full), dashboard_system 15401, dashboard_overview 15501; 17 println! calls; Empty at 15396, 15498 and 15594; the only callers are inside dashboard_cmd, dispatch arm 1383. Output display sites, all println!: engine.rs:231, 776, 803, 986, 1582, 1687, 2151 and main.rs:3021. colored 2.2.0 (Cargo.lock): control.rs:83 lazy_static SHOULD_COLORIZE; 102 to 108 CLICOLOR default on AND io::stdout().is_terminal(), NO_COLOR and CLICOLOR_FORCE override. strip_ansi, private: health_tui.rs:194, pty_exec.rs:109. nsh-test uses of dashboard: main.rs:785, 799, 818, 836. Stand-in: run with its list flag, mod.rs:16066 to 16100, prints and returns Empty at 16100; run with no argument returns Error at 16065. -->
- [x] G1 RED FIRST, BOTH DOORS: on the current build, dashboard | wc -l refuses with exit 1,
      captured verbatim in the REPL and through nsh -c, and the new count tests (G7) are
      watched failing against it.
<!-- evidence: demonstrated 2026-10-08 on the deployed nsh (shipped 14:11:23 with the INT-281 fix). REPL at 14:12: dashboard | wc -l printed the dashboard, then the refusal, and the prompt showed exit 1. nsh -c through python subprocess at 14:44: exit 1, the dashboard on stdout, the refusal on stderr. New tests regression_282_dashboard_feeds_wc and repl_282_dashboard_feeds_wc, added by fpatch Seal c45a607d0e8c55f4, watched failing on the debug build: 231/233 with exactly these two red (/tmp/suite-282-red.txt). -->
- [x] G2 STAND-IN: the two INT-281 tests use run --list instead of dashboard, watched red
      against a pre-281 build (a worktree at 1c0b19f2, its own target dir, removed after)
      and green on the current one -- so the refusal keeps a test that can fail.
<!-- evidence: the two INT-281 tests now pipe run with its list flag into wc (fpatch Seal c45a607d0e8c55f4). Green on the current debug build (/tmp/suite-282-red.txt). Red against a pre-281 build: git worktree at 1c0b19f2 built into its own target dir; that nsh piped dashboard into wc and printed 0 with exit 0; the current nsh-test against it gave 229/233 with exactly the two 281 and two 282 cases red (/tmp/suite-282-g2-pre281.txt). Worktree and target dir removed after. -->
- [x] G3 THE COLOUR RULING among (a), (b) and (c), recorded with its reason and a census of
      the Output builtins whose text carries escape codes in the REPL.
<!-- evidence: ruled 2026-10-08 by Christian: (b), strip at the peel. Census, read-only (sha256 4ec24df44e57): 131 functions in novashell src construct CommandResult::Output and 98 of them use colored styling or literal escape codes in their own body (a function-level scan; one that builds its text through helpers, such as dashboard_cmd, counts as plain). colored 2.2.0 colours whenever the shell stdout is a terminal, so in the REPL those 98 carry escape codes into a pipe and under nsh -c they do not. Reasons for (b): one place, the Output arm of peel_builtin_first_stage; it matches to_pipe_text, which is uncoloured; the two doors pipe the same bytes. The shared helper takes the pty_exec.rs:109 form (ESC to the first ASCII letter, the end of a CSI sequence), which covers the health_tui.rs:194 form (stops at m); both private copies become one-line wrappers. Nothing redirects stdout, so there is nothing to restore (INT-277). -->
- [x] G4 DISPLAY UNCHANGED: dashboard, dashboard system and dashboard overview show the same
      lines and labels on the terminal before and after, compared with the volatile numbers
      masked (load, memory, processes, disk, commits, events), through both doors.
<!-- evidence: demonstrated 2026-10-08. nsh -c, the deployed 5.2.0 binary (before) against the debug build with the conversion (after), volatile values masked (numbers, the memory bar, process and event names): dashboard 23 and 23 lines, dashboard system 13 and 13, dashboard overview 10 and 10, the same shape each. REPL on the deployed 5.3.0 binary (mtime 15:23:54): dashboard shows the same two boxes in colour. Conversion by fpatch Seal 4a93c26e8db0bfa6: the helpers write into a String instead of printing, and dashboard_cmd returns Output with its last newline taken off for the display sites to put back. -->
- [x] G5 on the DEPLOYED binary, through both doors: dashboard | wc -l prints a count equal to
      the dashboard's own line count with exit 0, and under (b) dashboard | cat -v shows no
      escape sequence in the REPL.
<!-- evidence: demonstrated 2026-10-08 on the deployed nsh 5.3.0 (mtime 15:23:54). REPL: dashboard | wc -l printed 22, the line count of dashboard itself, exit 0; dashboard | od -An -c | head -4 showed no 033 with colour on (od in place of cat, which is bat at this prompt). nsh -c: dashboard 22 lines and dashboard | wc -l 22; with CLICOLOR_FORCE=1, dashboard | od -An -c held 0 escape bytes; alias 244 lines and alias | wc -l 244. -->
- [x] G6 NO REGRESSION: both INT-243 cases green (785 still exits 141, 799 shell alive), the
      INT-281 refusal holds through run --list on both doors, history | head, logs | head,
      alias | head and printf | head unchanged in count, nsh-test all passing, d 0 failed.
<!-- evidence: demonstrated 2026-10-08. Suite 235/235 on the debug build (/tmp/suite-282-green.txt) and on the deployed nsh-test (/tmp/suite-282-deployed.txt): both INT-243 cases green (regression_243 still exits 141, repl_243 shell alive) and both INT-281 cases green on run with its list flag. Deployed nsh -c: history | head -3 three rows, logs | head -2 two rows with exit 0, alias | head -3 three rows, seq 1 5 | head -2 gave 1 and 2; the printf into head -3 suite case green. d: 26 passed, 0 failed, 2 warnings for the uncommitted change. -->
- [x] G7 regression tests in nsh-test beside the INT-243 and INT-281 cases: dashboard | wc -l
      equals the dashboard's line count through run_fsh and a Category::Repl case; under (b),
      a piped Output carries no escape byte with colour forced (CLICOLOR_FORCE=1 through
      run_fsh_env), watched failing before the strip lands.
      The newline: alias | wc -l equals the line count of alias itself through run_fsh,
      watched failing before the peel terminates Output.
<!-- evidence: zero/shell/nsh-test/src/main.rs, beside the INT-243 and INT-281 cases. regression_282_dashboard_feeds_wc and repl_282_dashboard_feeds_wc (fpatch Seal c45a607d0e8c55f4): red at 231/233 before the conversion, green after. regression_282_piped_output_carries_no_escape (Seal 5b83a46047d7ba1d): red with 033 crossing the pipe under CLICOLOR_FORCE=1, green after the strip. regression_282_piped_output_ends_with_newline: first written against alias line counts, which passed on the unfixed build because run_fsh trims and alias opens with a blank line; rewritten (Seal 2d16d8715515f2e9) to assert the last piped byte of help, red with 200, green after the peel fix (Seal 49055188f13bc1ad). Suite 233/235 red, then 235/235. -->
- [x] G8 each gate carries evidence per INT-158.
<!-- evidence: G0 to G7 each carry an evidence comment in INT-158 form, naming the scripts by sha256, the fpatch seals and the suite files. -->
