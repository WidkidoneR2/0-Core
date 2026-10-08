---
id: 281
date: 2026-10-07
type: future
title: "printing builtins feed a pipeline nothing -- dashboard | wc -l prints 0"
status: complete
tags: [nsh, pipeline, builtins, fix]
depends_on: []
---

## Vision

A builtin at the head of a pipeline feeds the next stage what it printed. dashboard | wc -l
counts the dashboard's lines; it does not print the dashboard and then 0.

## The Problem

Observed 2026-10-07 at a top-level prompt, typed one line at a time (INT-243 G1):

    dashboard | wc -l     -> the whole dashboard on the terminal, then 0
    dashboard | head -4   -> the whole dashboard on the terminal; head shows nothing

No error, no message. A wrong answer with no signal -- the code's own comment at
commands/mod.rs:9844 calls this worse than an error.

By reading: peel_builtin_first_stage (commands/mod.rs:9775) runs a builtin that leads a
pipeline inside the shell (execute_impl, 9840). A builtin that RETURNS text feeds the pipe:
Output at 9842, Value through to_pipe_text at 9855. A builtin that PRINTS with println! has
already written to the shell's own stdout -- the terminal -- and returns Empty, which 9843
pipes on as empty text. The next stage gets nothing.

It is the same split that made history | head drop its pipe, fixed 2026-08-21 for table
commands by giving them to_pipe_text. That fix covered builtins that return a Value. This
intent covers the ones that still print.

Moved here from INT-243 under its 2026-10-07 re-scope: the census of printing builtins and
the ruling on how to fix them (243's old G2 and G3).

## The Solution

The census decides the shape. Three candidates, ruled in G3:

(a) Printing builtins return Output instead of printing. Consistent with to_pipe_text.
    Cost grows with the census count, and builtins that print incrementally by design
    change behaviour.
(b) The dispatcher captures a leading builtin's stdout and pipes what it printed. No
    per-builtin change, but novashell has no fd-level redirection today (zero dup/dup2
    calls), and redirecting the interactive shell's own stdout in-process is exactly the
    kind of change INT-277 has to trust. Weighed against that before it is chosen.
(c) A printing builtin leading a pipeline refuses with a clear message and a non-zero
    exit, the way quoted arguments already do (9823-9836). Honest at once; pipes nothing.

(c) can also be a first step under (a) or (b): stop the silent wrong answer now, then fix
builtins as the census allows.

## Non-goals

- The EPIPE behaviour INT-243 locked in (exit 141 into a closed stdout). Not touched.
- Making every builtin return a structured Value. This is about pipes receiving text.
- The quoted-argument refusal at 9823-9836.
- External pipelines. Both stages already run as real children.

## Scope against neighbours

INT-243 closed the crash and kept it closed with two tests. 281 fixes the wrong answer
those tests do not assert on. INT-277 is the reason (b) needs care: a shell that redirects
its own stdout and fails to restore it would be the next lockout.

## Dependencies

depends_on is empty on purpose: INT-243 is complete and nothing else must finish first.

## Success Criteria

Each gate is watched failing before it is watched passing. Anchors stay ASCII-only.

- [x] G0: recon recorded with file:line -- the peel path (commands/mod.rs:9775-9858), which
      builtins already return Output or Value, and nsh-test's existing pipeline cases.
<!-- evidence: recon 2026-10-08, three read-only python scripts (sha256 1e6c3da3e2ae, c0dbf677e152, c5db8e03c628). Peel path zero/shell/novashell/src/commands/mod.rs:9775-9858 as stated: Output 9842, Value 9855 via to_pipe_text (value.rs:90), Empty 9843 piped as an empty string. dashboard_cmd 15373 calls dashboard_system 15387 and dashboard_overview 15487: 16 println! lines, Empty returned at 15484 and 15580. Returners: 148 functions construct CommandResult::Output (345 sites) or ::Value (61 sites); value sources are VALUE_SOURCES (value.rs:914); execute_dispatch spans mod.rs:1188-5290. No dup, dup2, dup3 or freopen call in novashell src (one comment, mod.rs:10026); expand_subshells (expand.rs:508) and run_capture (exec.rs:1008) capture child processes only. nsh-test pipeline cases: INT-243 closed stdout exits 141 (nsh-test main.rs:780) and REPL dashboard | head -4 (main.rs:799), which asserts the shell answers, not the count; external pipelines at 750, 775, 2592, 3179, 5021. -->
- [x] G1 RED FIRST, BOTH DOORS: dashboard | wc -l captured verbatim in the REPL (dashboard
      on the terminal, then 0) and through nsh -c, the -c result recorded as observed
      rather than assumed.
<!-- evidence: demonstrated 2026-10-08 on the deployed nsh (mtime 2026-10-07 21:17:26; source unchanged since, later commits touch intents, findings and the version only). REPL: dashboard | wc -l printed the whole dashboard on the terminal, then 0. dashboard | head -4 printed the whole dashboard; head showed nothing. nsh -c through python subprocess: exit 0, stderr empty, stdout is the whole dashboard followed by 0 (the builtin wrote to the stdout nsh was given; wc -l counted an empty pipe). -->
- [x] G2 CENSUS: every builtin that writes to stdout directly -- println!, print!, or a
      write to an io::stdout() handle -- enumerated by fsearch across builtin code paths,
      each marked prints or returns. The count is recorded.
<!-- evidence: census 2026-10-08. fsearch println!, fsearch print! and fsearch stdout(), each with type rs, piped as text into a read-only classifier (sha256 826abbeabbc5) that checked fsearch against the disk: 465, 22 and 20 novashell rows, equal to disk (stdout() recounted literally, sha256 2bd07ce10499). Call graph from every top-level execute_dispatch arm (158): 33 reach a stdout write; 8 are on PATH and spawn the real program before the peel (nsh, git, fd, grep, watch, find, cat, env); 25 can lead a pipeline. Per handler (sha256 dfd85f05a8dd), marked: RETURNS 4 (last_command|lc, trace, terminate, select); PRINTS 12 (dashboard|dash, snap-diff, histogram, run, query, goto, fdiff, clear|c|cls, zsh|bash, edit, replace, reload); MIXED 9 (on, command, dev, let, chart, logs, patch-multi, clean|fix, time). In every handler the printing branches return Empty and the returning branches return Output or Value. execute_impl mod.rs:718 prints only in text mode (allows_text_transforms), never under the peel. echo returns Output always (mod.rs:1653). -->
- [x] G3 THE RULING among (a), (b) and (c) is recorded here with its reason, the census
      count, and, for (b), how the interactive shell's stdout is guaranteed restored.
<!-- evidence: ruled 2026-10-08. RULING (c), keyed on the result, not on a list of names: when a builtin peeled off the head of a pipeline returns Empty, the rest of the pipeline does not run; nsh says so and exits 1, through the existing Peeled::Finished arm (its doc: the builtin ran and produced something that is not pipeable text). Reasons: 1) the census (25 arms: 12 print, 9 mixed, 4 return) shows printing branches always return Empty, so the result identifies them and no list can drift; 2) Output and Value are untouched, so logs, history, alias and every table command keep piping; 3) (b) rejected: novashell has no dup, dup2, dup3 or freopen call, and redirecting the interactive shell stdout in-process is the INT-277 lockout class, so no restore guarantee is needed; 4) (a) for dashboard is out of 281: G4 accepts the refusal, and converting it carries its own decision about colour codes in a pipe; it is filed with core intent find before cicomplete. -->
- [x] G4 on the DEPLOYED binary, through both doors, dashboard | wc -l prints the
      dashboard's line count -- or, under (c), a clear refusal with a non-zero exit.
      Never the dashboard and then 0.
<!-- evidence: demonstrated 2026-10-08 on the deployed nsh (shipped 14:11:23, nsh 5.2.0, reloaded with exec /home/christian/.local/bin/nsh). REPL: dashboard | wc -l printed the dashboard, then the refusal (dashboard: the rest of the pipeline did not run, dashboard gave it no text), and the prompt showed exit 1; no 0 line. nsh -c through python subprocess: exit 1, the refusal on stderr, the dashboard on stdout and no 0 line. Fix: zero/shell/novashell/src/commands/mod.rs, the Empty arm of peel_builtin_first_stage now returns Peeled::Finished(CommandResult::Error(.., 1)); fpatch Seal e1dea7b79115b8af. -->
- [x] G5 NO REGRESSION: both INT-243 cases stay green, history | head still feeds head,
      and an external pipeline (printf | head) is unchanged.
<!-- evidence: demonstrated 2026-10-08. Suite 231/231 on the debug build after the fix (/tmp/suite-281-green.txt) and on the deployed nsh-test (/tmp/suite-281-deployed.txt), with regression_243_closed_stdout_exits_141 and repl_243_builtin_pipe_leaves_shell_alive green, and the printf into head -3 case green. nsh -c on the deployed binary: history | head -3 gave three rows, exit 0; seq 1 5 | head -2 gave 1 and 2, exit 0; logs | head -2 gave two rows, exit 0 (a mixed builtin on its Value branch still pipes). d: 26 passed, 0 failed, 2 warnings for the uncommitted change. -->
- [x] G6 regression tests in nsh-test beside the INT-243 cases: a printing builtin piped
      into wc -l through both doors (run_fsh and a Category::Repl case), asserting the
      count is not 0 (or the refusal), watched failing first.
<!-- evidence: zero/shell/nsh-test/src/main.rs, regression_281_printing_builtin_pipe_refuses (run_fsh_status) and repl_281_printing_builtin_pipe_refuses (Category::Repl, run_repl_lines), inserted beside the INT-243 cases by fpatch Seal 052965131657e9b7. Watched failing first: 229/231 on the unfixed debug build with exactly these two red (/tmp/suite-281-red.txt); then 231/231 after the fix, debug and deployed. -->
- [x] G7 each gate carries evidence per INT-158.
<!-- evidence: G0 to G6 each carry an evidence comment in INT-158 form, naming the scripts by sha256, the fpatch seals (red 052965131657e9b7, fix e1dea7b79115b8af) and the suite files. -->

## Versions
- novashell 5.2.0 -> 5.3.0 (minor)
