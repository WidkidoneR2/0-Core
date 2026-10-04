---
id: 272
date: 2026-10-03
type: future
title: "Messages: exit 2 from an external is not a builtin misuse, and a refusal prints once"
status: planned
tags: [nsh, core, messages]
---

## Vision
A message says what happened and nothing it cannot know. Two messages break that today. nsh calls
any exit status 2 "misuse of shell builtin", even when the command was an external program that
never touched a builtin. And core prints a refusal twice: once as the close's own `refused:` line
and again as its generic `Runtime error:` wrapper. After this intent, nsh names a builtin misuse
only for a builtin, and a refusal reaches the reader once, with the fact that nothing was written.

## Why Now
- 2026-10-03, INT-271 forwarding check: `cicomplete 999 --bump novashell=patch` reached core, clap
  refused the flag with exit 2, and nsh printed "exited 2 -- misuse of shell builtin". core is an
  external binary. The category was a guess sounding certain, which AGENTS.md section 3 (Messages
  and tool output) names as the thing a message must not do.
- The same label was right minutes later for `bump-versions novashell`, a builtin exiting 2. The
  label is keyed on the number, not on what ran.
- 2026-10-03, INT-271 probes: every close refusal printed `refused: <reason>` and then
  `Runtime error: <reason>` with the same reason, because complete_intent prints and also returns
  the error, and core's error path prints it again.

## What
1. nsh: the exit-2 category is said only when the command that exited was a builtin. An external
   exiting 2 is reported as exited 2 with no category, unless nsh actually knows one.
2. core: a refusal from the close prints its reason once, still says nothing was written and the
   intent was not moved, and keeps its non-zero exit.

## Approach
- Phase 0 reads before anything changes: where nsh maps exit statuses to categories and how it
  knows builtin from external at that point; how complete_intent's refusal and core's top-level
  error printing meet.
- One concern per commit: the nsh label, then the core refusal.
- Red first for both, on the deployed binaries, before any edit.
- The class, not the example: the nsh check covers builtin and external, exit 2 and the other
  categorised statuses, so no other number keeps the same mistake unseen.
- No Omarchy code, no sudo. AGENTS.md is not edited.

## Phases
Phase 0 -- recon, reading only: the category table and its caller in nsh, with file:line; the
  builtin-or-external signal available there; the close refusal path and core's error printer.
Phase 1 -- nsh: the exit-2 category only for builtins.
Phase 2 -- core: a refusal prints once.
Phase 3 -- ship; doors.

## Gates
- [ ] Phase 0: the nsh category table and its caller recorded with file:line, the builtin-or-external signal named, and the close refusal path plus core's error printer recorded
- [ ] Red baseline on deployed nsh: an external exiting 2 (`/usr/bin/ls --int272-bogus`) is labelled "misuse of shell builtin", captured verbatim
- [ ] After, on the debug then deployed nsh: the same external exiting 2 carries no builtin label; `bump-versions novashell` (a builtin exiting 2) keeps it; the other categorised statuses for externals and builtins are unchanged, checked as a class in a test
- [ ] Red baseline on deployed core: a close refusal on a scratch HOME prints its reason twice, captured verbatim
- [ ] After, on the debug then deployed core: the refusal prints its reason once, still states that nothing was written and the intent was not moved, exits non-zero, and the scratch intent and Cargo.toml files are unchanged
- [ ] Doors per commit: the result on PATH after ship, nsh-test all passing, d 0 failed, plus that commit's own door above

## Notes
- Found during INT-271 on 2026-10-03 and recorded there as found, not fixed.
- The scratch-HOME method from INT-271 (HOME pointing at a throwaway 0-core with an INT-999 commit)
  reproduces a close refusal without touching the real ledger.
- Related: INT-271 (where both were seen), AGENTS.md section 3 Messages and tool output.

## The Rule
"A message that names a cause it did not see is a guess wearing a uniform."
