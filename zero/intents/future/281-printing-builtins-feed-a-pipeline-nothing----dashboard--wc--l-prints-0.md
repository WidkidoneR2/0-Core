---
id: 281
date: 2026-10-07
type: future
title: "printing builtins feed a pipeline nothing -- dashboard | wc -l prints 0"
status: planned
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

- [ ] G0: recon recorded with file:line -- the peel path (commands/mod.rs:9775-9858), which
      builtins already return Output or Value, and nsh-test's existing pipeline cases.
- [ ] G1 RED FIRST, BOTH DOORS: dashboard | wc -l captured verbatim in the REPL (dashboard
      on the terminal, then 0) and through nsh -c, the -c result recorded as observed
      rather than assumed.
- [ ] G2 CENSUS: every builtin that writes to stdout directly -- println!, print!, or a
      write to an io::stdout() handle -- enumerated by fsearch across builtin code paths,
      each marked prints or returns. The count is recorded.
- [ ] G3 THE RULING among (a), (b) and (c) is recorded here with its reason, the census
      count, and, for (b), how the interactive shell's stdout is guaranteed restored.
- [ ] G4 on the DEPLOYED binary, through both doors, dashboard | wc -l prints the
      dashboard's line count -- or, under (c), a clear refusal with a non-zero exit.
      Never the dashboard and then 0.
- [ ] G5 NO REGRESSION: both INT-243 cases stay green, history | head still feeds head,
      and an external pipeline (printf | head) is unchanged.
- [ ] G6 regression tests in nsh-test beside the INT-243 cases: a printing builtin piped
      into wc -l through both doors (run_fsh and a Category::Repl case), asserting the
      count is not 0 (or the refusal), watched failing first.
- [ ] G7 each gate carries evidence per INT-158.
