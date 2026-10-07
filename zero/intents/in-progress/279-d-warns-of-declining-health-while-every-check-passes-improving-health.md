---
id: 279
date: 2026-10-06
type: future
title: "d warns of declining health while every check passes (improving health)"
status: in-progress
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
- NO IMPLIED CAUSE, ANYWHERE IN THE ADVISORY CHAIN (doctor/mod.rs 534-569). "With no
  active work", "expected pattern" and "recovery expected on completion" all assert a
  cause the code never established. Every advisory states facts only. The intents in
  progress may be printed as a plain fact, on its own line, never as a reason.
- THE INTENT COUNT ASKS THE LEDGER. Today the doctor scans intents/future/ for
  "status: in-progress" itself; cistart moves intents to intents/in-progress/, so the
  count is always 0. The count comes from the intent domain's own loader -- one owner
  of what is in progress -- and no second scan of the ledger directories is added.
- UNREADABLE HISTORY IS SKIPPED, NOT INVENTED. A doctor event whose health cannot be
  read is left out and counted as skipped; it is never replaced with the current
  run's health (doctor/mod.rs:462 does that today).
- A FORECAST THAT CANNOT ANSWER SAYS SO. The trend compares the newest 3 runs with
  the older ones, so it needs at least 4 runs: one on each side of the comparison.
  Fewer prints "not enough history (N runs)" instead of a number. Every forecast
  states its window and run count.
- THE ARITHMETIC IS TESTED. The trend and forecast are computed by a function that a
  fixture history can drive, so a test can state the expected numbers for a known
  series and fail if they drift.

## Non-goals

- No change to any health CHECK, its pass/warn/fail logic, or the health score. Only the
  forecast, the trend and the decline message.
- No redesign of d's layout, colours or other lines. The Friday lines printed under d
  are out of scope unless G0 finds Friday owns the decline warning.
- No new dependencies. No change to Omarchy.
- The forecast.declining reaction rule (reaction/mod.rs:316-343) reads
  ~/.cache/zero/forecast.txt, which nothing writes, so it never fires. That is filed
  as a finding with core intent find, not fixed here.
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

- [x] G0: recon recorded. fsearch for the warning text and the forecast line names the
  owning file and function for each of: the forecast numbers, the trend, the decline
  warning, and the "no active work" test. Also recorded: where past runs are stored,
  how many are read, over what window, and the trend formula -- each with file:line.
<!-- evidence: demonstrated 2026-10-07. Owner: zero/engine/src/domains/doctor/mod.rs, the inline forecast block 448-570. It reads the last 10 doctor events (452). trend = mean of the newest 3 minus mean of the rest (476-480); 24h = health + 0.5 x trend, 7d = health + 2 x trend (482-485). Fewer than 3 events prints nothing (471). With exactly 3, the older set is empty, older_avg is 0, and the trend equals the recent average. An event with no readable health counts as the current health (462). The intent count (504-524) scans intents/future/ for status: in-progress, but cistart moves intents to intents/in-progress/ (279 is there), so the count is always 0: the warning (546-549) fires on any trend below -1.0, and the advisories at 541 and 559 can never print. 2026-10-06 checks out: trend -4.0 gives 98 and 92. core forecast (1093) is a separate computation with its own query. forecast_cache (paths.rs:413) has one reader, reaction/mod.rs:316-343, and no writer; forecast.txt does not exist. -->
- [x] G1: RED FIRST, AFTER A PURE EXTRACTION. The forecast and advisory logic moves out
  of the print path into a pure function (runs, current health, warnings, failures,
  intents in progress in; lines out) with NO behaviour change, in its own commit, proven
  by d printing the same lines before and after. Then a test drives that function with a
  fixture shaped like 2026-10-06 (runs at 92%, then 100% with 0 warnings and 0 failures,
  no intents counted) and shows the decline warning is produced. Output saved as evidence
  before any behaviour changes.
<!-- evidence: commit 9d808f0f, 2026-10-07. forecast_lines(runs, health, intents) is a pure function; the inline block calls it. No behaviour change, shown two ways: the payload checked before writing that every literal (emoji, messages, format strings) already existed verbatim in mod.rs, and five characterisation tests hand-computed from the old inline code pass (cargo test forecast_tests: 5 passed, 0 failed). The red: g1_clean_run_after_a_dip_warns_today drives the 2026-10-06 shape (100, 92, 92 newest, 98s older, no intents) and the decline warning is produced. Method changed from the gate as first written: comparing d before and after was dropped, because every d run adds a reading, so two runs never read the same history. -->
- [ ] G2: no warning on a clean run. Same fixture: no decline warning. A second fixture
  whose current run has a failure that earlier runs did not still warns.
- [ ] G3: the warning names its evidence. When it fires, the message carries the window,
  the run count, the score from and to, and each check that moved from pass to warn or
  fail, plus one runnable next move. A test asserts each part is present.
- [ ] G4: no implied cause, across the whole advisory chain. "with no active work",
  "expected pattern" and "recovery expected on completion" are gone; fsearch for each
  returns no live source line. A test runs every advisory branch and asserts none of
  them contains a causal claim; the intents in progress, when printed, are a separate
  plain line.
- [ ] G5: not enough history. Fixtures holding 0, 1, 2 and 3 runs each print "not
  enough history (N runs)" and no number. Red first: today 0-2 print nothing at all, and
  3 prints a trend equal to the recent average (older_avg is 0), which clamps both
  forecasts to 100.
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
- [ ] G10: the intent count asks the ledger. Red: with an intent in intents/in-progress/
  the doctor counts 0 today. Green: the count comes from the intent domain's loader and
  matches core intent list; fsearch shows no other "status: in-progress" scan in the
  doctor.
- [ ] G11: unreadable history is skipped. Red: a fixture event with no readable health
  is counted as the current health today. Green: it is left out, the run count drops by
  one, and the skipped count is reported.
- [ ] G12: the dead reaction rule is filed. core intent find records forecast.declining
  reading a file nothing writes, and the finding id is cited here.
- [ ] G13: doors. nsh-test all passing against the deployed binaries, d 0 failed,
  commits carry Intent: INT-279.
