---
id: 300
date: 2026-10-22
type: future
title: "Bug Report"
status: planned
tags: [Bug report, nsh, novashell]
---

## Vision
`core bug` turns the F-#### findings into a bug report that tells its own story. Every bug has a number, a day it was found, a severity (High, Medium or Low) and an honest state: open, solved or closed. A solved bug shows the day it was solved, the commit that solved it and the Seal on that commit, and `core bug promote` can turn a bug into a real intent in one reviewed step. This is the first Project 0 tool built to the mature standard written below, and the standard is the second deliverable: the older tools are held to it afterwards, one finding at a time.

## The Problem
Findings (INT-266) record what was found and not fixed, and they work, but they stop short of a bug report. Measured 2026-10-22:

- 67 records. Every record holds the same six fields: id, what, at, found_by, found_on, closed_without_fix. No record has a severity.
- The closed_without_fix field is overloaded: 65 records hold `no` and 2 hold a free-text reason, so one field is a yes/no and a sentence at once.
- 17 of the 67 records have no location (`at` is unknown).
- Fixed is already derived, never stored (INT-192): a commit trailer `Fixes: F-NNNN` names the finding it fixes. 18 findings have such a commit, 1 of them (F-0046) has two commits, and every one of those commits carries a Seal. 130 commits carry a Seal trailer in all.
- One Fixes line carries prose after the id (commit 23c7310e, which only filed a finding). A reader that takes any id it sees would mark a finding as fixed by a filing commit.
- The surface is thin: `core intent find` files a finding, `core intent trace` follows one, `core intent seals` re-checks seals. Nothing lists the findings, rates them, or shows one as a whole story from found to solved to sealed.
- By the derivation above the ledger should read 18 solved, 2 closed and 47 open. That is an expectation, not a fact: the tool must reproduce it, an independent recount must agree, and any difference is itself a finding.

## The Solution
A new `core bug` command (nsh alias `bug`) over the same F-NNNN records in zero/intents/findings/. No record moves, no id changes, no migration. A new domain module reads findings.rs and trace.rs and adds the derivation and the display; it never gets a parser of its own.

Stored is only what a person saw or decided: the finding, where, who found it, the day it was found, a severity, and a closure reason when it is closed without a fix. Everything else is derived from git, every time, and explains itself: state, solved day, time to fix, the fixing commit, the Seal and whether that Seal still checks out.

Rulings, written here before any code (change one by editing this section and saying so in the commit):
- R1 Home: `core bug`, with list, show, rate, check and promote, and a dashboard when run with no arguments. Records stay in zero/intents/findings/. The ids stay F-NNNN.
- R2 Severity is one stored field, `severity`, with the values unrated, low, medium and high. A filing gets unrated, never an invented medium. High: it can make a tool or a gate tell a lie, lose data, or break a promise another gate relies on (calibration example F-0054, validate exits 0 with issues). Medium: wrong or missing behaviour that has a workaround, or a second source of truth that can drift (F-0067, cicomplete attributes a commit to the wrong intent). Low: naming, cosmetics, or a note for later (F-0064, the nsh completion list lacks the gates subcommand). The 67 old records start unrated and are rated in a triage pass.
- R3 State is derived: solved when at least one valid Fixes trailer names the finding on HEAD, and the solved day and commit are those of the latest such commit; closed when closed_without_fix holds a reason and no fix exists; otherwise open. A fix wins over a closure, and `core bug check` reports the conflict.
- R4 A Fixes trailer counts only when each id in it is exactly F-NNNN. A value with prose is reported as malformed and counts for nothing, so commit 23c7310e fixes nothing and F-0046 has one fix commit, af4c9435.
- R5 Writes are plans: `rate` and `promote` print what they will write and a Seal, and write only on `apply SEAL`, like fpatch.
- R6 Promote never runs by itself. It reuses the code behind `core intent new` and `inta`.

## The Standard
Seven rules this tool is built to, and the older tools are held to them afterwards. Each maps to a gate below.
1. Store decisions, derive facts (G2, G3).
2. Unknown is a word, never a zero (G7).
3. One owner per parse: reuse the trailer reader, never write a second (G4).
4. Red first: no behaviour without a failing test that names its case (every code gate).
5. Writes are plans: dry run, Seal, apply (G9, G11).
6. Every number explains itself and an independent recount reproduces it (G5, G6, G12).
7. Old data keeps working: forward-only, the 67 records parse unchanged (G3).
Exit codes mean something as well: 0 clean, 1 findings, 3 could not answer (G8).

## Success Criteria
- [ ] G1 BASELINE, measured before any code and kept as the reference: the counts above (67 records, 65 and 2 closure values, 17 unknown locations, 18 findings fixed, 130 sealed commits) are re-measured by script, and for each of the 67 records the found_on date is compared with the git date of the commit that added the record, so we know how far found_on can be trusted
- [ ] G2 RED FIRST: a pure function derives the state of a finding from its record and the valid Fixes commits, as open, solved (day, commit, Seal) or closed (reason); its tests use the real edge cases (F-0046 with the filing commit that carries prose, a finding with no fix, a closed one, a closed one that also has a fix) and fail against a stub before the real function exists
- [ ] G3 the record gains exactly one stored field, severity, written by filing as unrated; all 67 existing records parse to the same values as before, the missing field reading unrated, proved by a test over a copy of the real records
- [ ] G4 trailer reading has one owner: the bug module calls the reader in trace.rs and adds none; a Fixes value with prose is malformed and counts for nothing (R4), proved on commit 23c7310e, and a search of the new code finds no second Fixes parser
- [ ] G5 `core bug list` prints one line per bug (F-id, severity, state, found day, solved day or age, short text), sorts open and high first, filters by open, solved, closed and severity, and ends with a totals line; the totals equal an independent recount from the files and git, which is run and shown
- [ ] G6 `core bug show F-NNNN` prints the whole story: the record, a timeline from found to solved to sealed, the fixing commit with its subject, the Seal, and whether that Seal checks out through the existing seals check; shown on F-0049, whose fix commit is 825e4803 with Seal eb4f6f578bcc9321
- [ ] G7 UNKNOWN IS A WORD: an unrated bug reads unrated, a bug with no fix has no solved day, a fix without a Seal reads unsealed, and git that cannot be read reads unknown and exits 3; each of these is a test, and none prints a zero, a medium or a default
- [ ] G8 `core bug check` reports malformed Fixes values, a Fixes naming a finding with no record, a closure that also has a fix, and records with missing fields or no location; it exits 0 clean, 1 with findings, 3 when it could not answer; run on the real ledger, with the list it prints
- [ ] G9 `core bug rate F-NNNN <level>` is a plan: it prints the change and a Seal, refuses a changed Seal or an unknown level, and writes only on apply; the record then shows the level and a second apply of the same plan is refused
- [ ] G10 `core intent find` takes a severity, defaulting to unrated, and still files exactly as before when the flag is absent; the existing find and trace tests still pass
- [ ] G11 `core bug promote F-NNNN` prints the intent it would create (title, future category, fix template, the finding as The Problem, and a first gate that a sealed commit carries Fixes: F-NNNN) and a Seal; on apply it creates a real intent through the code behind `core intent new`, records the intent on the bug, refuses a bug already promoted, and core intent validate still passes; demonstrated on a bug filed for the demonstration
- [ ] G12 the dashboard (`core bug` with no arguments) shows open bugs by severity, the oldest open bug and its age, the median time to fix, and the bugs solved in the last 7 days, each number reproduced by the independent recount of G5
- [ ] G13 the welcome line `bugs: N open (H high)` reads from the same derivation, shows unknown when it cannot answer, and is only shipped after INT-299 is complete so that the welcome cannot lie
- [ ] G14 triage and doors: the 67 existing bugs are rated in one reviewed pass (the ratings are the decision, written through G9), then doors: nsh-test all passing, d 0 failed, core intent validate exit 0, and the standard above is proposed to AGENTS.md through a reviewed plan for your approval, not edited
- [ ] G15 each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
