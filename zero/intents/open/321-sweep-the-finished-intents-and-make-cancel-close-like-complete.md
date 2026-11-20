---
id: 321
date: 2026-11-19
type: future
title: "Sweep the finished intents out of open/ and make cancel close like complete"
status: planned
tags: [ledger, intents, cleanup]
format: 3
---

## Vision
open/ holds only work that is planned or in progress. Every finished or
cancelled intent is a row in closed.tsv and a file in git history, and the
counts Christian sees do not move when the files go. Cancelling an intent
closes it the same way completing one does. This is the clean ledger the
Quickshell ledger panel will read.

## The Problem
Read against main b4f5fa36.
1. Eleven intents with status complete are still files in open/: 308, 309,
   310, 313, 314, 315, 316, 317, 318, 319 and 327. Two with status
   cancelled are too: 039 and 118. INT-313 decided finished and cancelled
   intents are not files, and INT-320 gave them closed.tsv, but only 320,
   328 and 329 are rows there.
2. `core intent cancel` leaves the file in open/ and writes no closed.tsv
   row (mod.rs cancel_intent, "The file stays in open/"), so every future
   cancel recreates problem 1.
3. Help text names folders that do not exist: parser.rs "Category
   (directory): future, decisions, ..." on `intent new`, and "moves to
   cancelled/" on `intent cancel`.
4. Statuses that do not match their gates: 169 is planned with 14 of 15
   gates ticked, 231 with 7 of 8, 191 with 4 of 12, 186 with 2 of 8, 048
   with 1 of 5 and 236 with 1 of 6.
5. Dependencies: 041 depends on 039 (cancelled), so 041 is always blocked;
   167 depends on 214 (planned). These are the banner's 2 blocked.
6. AGENTS.md still names Claude as author and Jarvis as reviewer, and
   points at zero/intents/in-progress/300-bug-report.md, which does not
   exist.

## The Solution
Record the counts first. Then one commit adds a closed.tsv row for each of
the thirteen files, with the last commit that touched it, and git rm's
exactly those files. Then `core intent cancel` gets the same close as
cicomplete: a cancelled row and a staged removal. Then the two help strings.
Then Christian rules on the planned triage (G5) from his phone, and each
ruling is applied as its own small change.

## Limits
- In scope: the thirteen files and their closed.tsv rows; cancel_intent in
  intent/mod.rs; two help strings in cli/parser.rs; the triage rulings
  Christian gives; AGENTS.md lines only if he rules yes.
- Out of scope: rewriting old frontmatter or renaming old files
  (forward-only); moving old intents to format 3; renumbering the Arch-era
  INT-320+ citations (INT-231 owns that); `core intent list --json` and the
  Quickshell panel (next intent); the double Versions sections in 316 and
  317 (the files are leaving).
- How deep: one sweep commit (closed.tsv plus deletions), one Rust change
  in cancel_intent with a red test first, one help-text commit, and one
  small commit per triage ruling. No new dependencies.
- Stop and ask: a file whose status is not complete or cancelled; a count
  that moves after the sweep (done, blocked, the next number, or trace
  INT-318); a cancel change that needs a second production file; any
  triage item without a ruling.
- Done means: G1 to G6 ticked with evidence.
- Must not change: planned and in-progress files (except by ruling), the
  done and blocked counts, the next intent number, live state.db, the nsh
  prompt path, anything outside the repo, root.

## Success Criteria
- [ ] G1 the counts before. On the laptop: core intent validate count, the
      banner's planned, done and blocked, core intent trace INT-318, and the
      next number.
      ask: the before numbers are written into this gate
      ruling:
      evidence:
- [ ] G2 the sweep. One commit adds closed.tsv rows for the eleven
      complete files and 039 and 118 as cancelled, each with the last
      commit that touched it, and git rm's exactly those thirteen files.
      Afterwards validate passes with 13 fewer intents, done and blocked
      are unchanged, trace INT-318 still answers, the next number is
      unchanged, and core intent show 316 reads the closed row.
      ask: done and blocked unchanged; validate passes; files gone on GitHub
      ruling:
      evidence:
- [ ] G3 cancel closes too (intent/mod.rs). core intent cancel writes a
      cancelled row and stages the removal the way cicomplete does, with
      the same fallback when the last commit is unknown. Red test in a temp
      git repo first. The reason is kept as a sixth closed.tsv column (Christian 2026-11-19).
      ask: cancel stages the removal and a cancelled row
      ruling:
      evidence:
- [ ] G4 the help text names only real folders (cli/parser.rs). The new and
      cancel help strings stop naming future and cancelled/.
      ask: the help names only folders that exist
      ruling:
      evidence:
- [ ] G5 the planned triage is ruled. Every planned intent has keep,
      cancel, merge or fix-status from Christian; each ruling is applied as
      its own small commit; anything unruled stays as it is.
      ask: every planned intent has a ruling or is left as it is
      ruling: Christian 2026-11-19: the triage table as written
      evidence:
- [ ] G6 doors and d green on the laptop.
      ask: doors and d are green
      ruling:
      evidence:

## Job order
- Wave 1, side by side: the sweep (closed.tsv and deletions), the cancel
  red test, the help strings.
- Wave 2: the cancel fix.
- Wave 3: one commit per triage ruling, after Christian rules.
- G1 before wave 1 and G6 at the end, on the laptop.

## Open for Christian
- The triage table below (G5).
- Whether AGENTS.md should drop Claude and Jarvis and fix the 300 path.

## Triage for G5 (suggestions only, each needs a ruling)
| id | suggestion | why |
|----|------------|-----|
| 026 | merge into 167 | events and the observatory overlap 167 |
| 041 | keep, drop the 039 edge | Friday-first; 039 is cancelled so 041 is always blocked |
| 042 | keep, rescope | Friday-first; the action set predates Omarchy |
| 048 | keep (small) | 1 of 5 ticked; title and filename disagree |
| 132 | keep | Friday-first |
| 144 | keep, defer | large, no current demand |
| 145 | measure, then decide | zero-ade exists; unverified since the migration |
| 156 | keep (low) | small builtin |
| 167 | keep, split later | instrumentation half stays |
| 169 | fix status | 14 of 15 ticked; the spine is in the tree |
| 170 | keep, defer | plugins are not a current need |
| 181 | cancel | the inquire wizard; core intent add exists |
| 186 | fix status | 2 of 8 ticked |
| 187 | cancel | its own title says not a felt need |
| 191 | fix status | 4 of 12 ticked |
| 214 | keep, do soon | small, and it unblocks 167 |
| 216 | keep | NFlow leans on byte offsets |
| 218 | keep | deadwood correctness |
| 223 | keep, high | Core must never depend on Intelligence |
| 224 | keep | naming |
| 226, 232, 233, 234, 235 | keep, group | the ledger-checkable family |
| 229 | keep | three-outcome service checks |
| 231 | fix status | 7 of 8 ticked |
| 236, 238, 239 | keep | shell correctness |
| 240 | keep | paths owner |
| 242, 244, 248 | keep | real bugs filed as intents |
| 249 | rescope | verify is built; the census and cases remain |
| 288 | keep | SafeShell |
| 297 | measure | cicomplete already bumps versions |
| 298 | keep (small) | --version census |
| 302 | keep | terminal follow-ups |

## Versions
- core patch (cancel closes; help text)
