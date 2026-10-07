---
id: 279
date: 2026-10-06
type: future
title: "d warns of declining health while every check passes (improving health)"
status: planned
tags: [health, checks, doctor, forecast, messaging]
depends_on: []
---

## Vision

When d says health is declining, it is true, and it shows why. When every check passes,
d does not tell anyone to investigate.

## The Problem

On 2026-10-06 one d run printed both of these, two lines apart:

    Passed: 28   Warnings: 0   Failed: 0   Health: 100% (28 of 28 determinable)
    Forecast  24h: 98%  7d: 92%  trend: -4.0
    Declining health with no active work — investigate

The run before it, minutes earlier, was ADVISORY at 92% (two warnings: uncommitted
changes and unpushed commits) and printed "Forecast 24h: 92% 7d: 90% trend: -0.8" with
no warning. The run before that, also 92%, printed "trend: +3.0". So the warning appeared
on the first fully clean run of the evening, after the only two warnings had been
cleared by a commit and a push.

Three things are wrong with that line, whatever the code turns out to say:

1. IT CONTRADICTS THE RUN IT IS PART OF. Every check passed. A reader told to
   "investigate" has nothing to investigate, and learns to ignore the line -- which
   is how a real decline gets missed later.
2. IT GIVES NO EVIDENCE. No window, no sample count, no score it moved from, no check
   that changed. AGENTS.md section 3: a message says what happened, what the tool
   could not do, and leaves a next move. This one leaves an order with no subject.
3. "WITH NO ACTIVE WORK" IMPLIES A CAUSE. INT-278 had been completed minutes before.
   Whether the phrase means "no intent in progress" or something else is unread; as
   printed, it reads as if finishing work made the machine worse.

The forecast numbers have the same shape of problem: a 24h forecast of 98% and a 7d of
92% printed under a 100% run, with no statement of what history they come from or how
many runs that history holds.

## The Solution

First read which code owns the forecast, the trend and the warning, and what history
it reads (G0). Then hold the output to this contract, decided now:

- NO WARNING ON A CLEAN RUN. When the current run has 0 warnings and 0 failures, the
  decline warning does not print. A recovered dip is not a decline.
- A WARNING NAMES ITS EVIDENCE. When it does print, it says what it compared: the
  window, the number of runs in it, the score it moved from and to, and the checks that
  went from passing to warning or failing. Then one next move a reader can run.
- NO IMPLIED CAUSE. "With no active work" goes. If the code reads the ledger, the fact
  may stay as a plain fact ("no intent in progress"), on its own line, never as a reason.
- A FORECAST THAT CANNOT ANSWER SAYS SO. A trend or forecast over a window holding
  fewer than 2 runs prints "not enough history (N runs)" instead of a number: two
  points is the least a trend can be drawn through. Every forecast states its window
  and run count.
- THE ARITHMETIC IS TESTED. The trend and forecast are computed by a function that a
  fixture history can drive, so a test can state the expected numbers for a known
  series and fail if they drift.

## Non-goals

- No change to any health CHECK, its pass/warn/fail logic, or the health score. Only the
  forecast, the trend and the decline message.
- No redesign of d's layout, colours or other lines. The Friday lines printed under d
  are out of scope unless G0 finds Friday owns the decline warning.
- No new dependencies. No change to Omarchy.
- Not the whole messaging effort. This intent fixes one message and the numbers under it;
  the wider aim (better messages across nsh, core, d and ship) stays its own work.

## Scope against neighbours

- INT-199 set the failure-output convention this message must follow; nothing here
  changes that convention.
- INT-222 (the doctor work, complete) owns how checks report; 279 touches only what is
  derived from past runs.
- A future messaging intent may sweep other tools. 279 is the first concrete case, and
  its contract (evidence, no implied cause, say so when it cannot answer) is written so
  that sweep can reuse it.

## Dependencies

depends_on is empty on purpose: nothing has to finish before this can start.

## Success Criteria

Each gate is watched failing before it is watched passing. The line as printed uses an
em dash; any fpatch anchor near it must be ASCII-only (patch_between).

- [ ] G0: recon recorded. fsearch for the warning text and the forecast line names the
  owning file and function for each of: the forecast numbers, the trend, the decline
  warning, and the "no active work" test. Also recorded: where past runs are stored,
  how many are read, over what window, and the trend formula -- each with file:line.
- [ ] G1: RED FIRST. A test drives the owning function with a fixture history shaped
  like 2026-10-06 (runs at 92%, then a run at 100% with 0 warnings and 0 failures) and
  shows the decline warning is produced today. Output saved as evidence before any
  code changes.
- [ ] G2: no warning on a clean run. Same fixture: no decline warning. A second fixture
  whose current run has a failure that earlier runs did not still warns.
- [ ] G3: the warning names its evidence. When it fires, the message carries the window,
  the run count, the score from and to, and each check that moved from pass to warn or
  fail, plus one runnable next move. A test asserts each part is present.
- [ ] G4: no implied cause. The phrase "with no active work" is gone from the warning.
  fsearch for it returns no live source line. If the ledger fact is kept, it prints as
  a separate plain line and a test shows it never appears inside the warning.
- [ ] G5: not enough history. A fixture window holding 0 runs and one holding 1 run each
  print "not enough history" with the count, never a number. Red first: today they
  print numbers (or show what they print, recorded verbatim).
- [ ] G6: the arithmetic is pinned. A fixture series with a hand-computed trend and
  forecast is asserted to the printed precision. Changing the formula without updating
  the test turns it red.
- [ ] G7: every forecast states its basis. The forecast line shows window and run count
  for each number it prints. A test asserts it.
- [ ] G8: wired in, not configured. The new tests run in the owning crate under cargo
  test, and that crate is already covered by the commit gate or nsh-test (whichever G0
  finds owns it). A deliberately broken assertion turns that run red; restored, green.
- [ ] G9: DEPLOYED. One ship at the end. In a new shell, the deployed d on a clean tree
  prints no decline warning and a forecast line with its basis. Then a deliberate
  warning (an uncommitted file) shows the warning path still works and names the
  Git Repository check. The file is removed afterwards.
- [ ] G10: doors. nsh-test all passing against the deployed binaries, d 0 failed,
  commits carry Intent: INT-279.
