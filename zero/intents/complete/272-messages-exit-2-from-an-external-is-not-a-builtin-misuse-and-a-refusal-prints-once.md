---
id: 272
date: 2026-10-03
type: future
title: "Messages: exit 2 from an external is not a builtin misuse, and a refusal prints once"
status: complete
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
- [x] Phase 0: the nsh category table and its caller recorded with file:line, the builtin-or-external signal named, and the close refusal path plus core's error printer recorded
<!-- evidence: read 2026-10-04 at HEAD 62f9e40b with numbered-line reads. nsh: explain_exit_code commands/mod.rs:9006-9019 (2 arm 9010, 128 arm 9013), reached from explain_exit_code_for 8980-9003 for every code but 1; callers 9851 (pipeline), 10084, 10125, 10174 (direct spawn) and 10419 (run_external), all external paths. The signal is the call site: a builtin returns CommandResult::Error(msg, code) and prints only msg. core: complete_intent intent/mod.rs:1520-1527 prints refused: and nothing was written, then returns CoreError::Runtime(msg); main.rs:63-71 prints it again as Runtime error: and exits 1 (Runtime at errors/mod.rs:12-13). write_versions 1297-1322 has the same shape for a write failure. -->
- [x] Red baseline on deployed nsh: an external exiting 2 (`/usr/bin/ls --int272-bogus`) is labelled "misuse of shell builtin", captured verbatim
<!-- evidence: demonstrated 2026-10-04 on deployed nsh (built 2026-10-03 22:08:40) in the REPL: /usr/bin/ls --int272-bogus printed exited 2 -- misuse of shell builtin. Same class in the same session: git -C on a missing directory printed exited 128 -- invalid exit argument. bump-versions novashell printed only its usage, exit 2, no category. -->
- [x] After, on the debug then deployed nsh: the same external exiting 2 carries no builtin label; `bump-versions novashell` (a builtin exiting 2) keeps it; the other categorised statuses for externals and builtins are unchanged, checked as a class in a test
<!-- evidence: commit ca900401, 2026-10-04. The 2 and 128 arms removed from explain_exit_code. Red first: cargo test -p novashell exit_label failed on bare 2 (left misuse of shell builtin, right non-zero exit), then 1 passed; exit_label_tests covers 2 and 128 bare and for /usr/bin/ls, git and an unknown external, with 1, 126, 127, 130, 137, 139 and grep exit 1 unchanged. rustfmt --check silent. Debug nsh, then deployed nsh (built 01:07:44), in the REPL: ls exiting 2 and git exiting 128 each read non-zero exit. The builtin clause rested on a false premise: no builtin path reaches this table, so bump-versions novashell had no label to keep; it printed its own usage, exit 2, no category, identical before and after. nsh-test 215 of 215 on debug and deployed. -->
- [x] Red baseline on deployed core: a close refusal on a scratch HOME prints its reason twice, captured verbatim
<!-- evidence: demonstrated 2026-10-04 on deployed core (built 2026-10-03 22:32:05), scratch HOME /tmp/int272-home, a git repo whose INT-999 commit touches novashell 1.0.0 and engine 2.0.0: core intent complete 999 --bump novashell=ptach printed refused: --bump novashell=ptach: ptach is not patch, minor, major or skip, then nothing was written and the intent was not moved, then Runtime error: with the same reason. Exit 1; git status on the scratch repo empty; intent still in in-progress. -->
- [x] After, on the debug then deployed core: the refusal prints its reason once, still states that nothing was written and the intent was not moved, exits non-zero, and the scratch intent and Cargo.toml files are unchanged
<!-- evidence: commit 146baa1e, 2026-10-04. CoreError::Reported and errors::top_level_line, the one owner of what main prints; both close sites return Reported. Red first: cargo test -p core top_level_line failed (left Some(r), right None), then 1 passed; the test covers Reported silent and Runtime, Registry, CapabilityDenied and Io printed, with the Runtime wording unchanged. cargo fmt --check silent. Debug core, then deployed core (built 01:21:22), same scratch probe: refused: and nothing was written and the intent was not moved each printed once, no Runtime error line, exit 1, git status on the scratch repo empty (the intent and both Cargo.toml files unchanged), intent still in in-progress. Same class on debug core: a read-only engine Cargo.toml made the write fail; its three lines printed once with no trailing Runtime error, exit 1; the scratch tree was restored by git checkout and is clean. -->
- [x] Doors per commit: the result on PATH after ship, nsh-test all passing, d 0 failed, plus that commit's own door above
<!-- evidence: ca900401 (nsh): ship 1 shipped 0 failed, nsh built 01:07:44, deployed probes as in gate 3, nsh-test 215 of 215, d 0 failed. 146baa1e (core): ship 1 shipped 0 failed, core built 01:21:22, deployed refusal probe as in gate 5, nsh-test 215 of 215, d 0 failed. Each ledger commit ran nsh-test (215 of 215) and d (0 failed) first with only the intent staged. The d warnings each time were uncommitted or unpushed work only. -->

## Notes
- Found during INT-271 on 2026-10-03 and recorded there as found, not fixed.
- The scratch-HOME method from INT-271 (HOME pointing at a throwaway 0-core with an INT-999 commit)
  reproduces a close refusal without touching the real ledger.
- Related: INT-271 (where both were seen), AGENTS.md section 3 Messages and tool output.
- Found, not fixed (2026-10-04, INT-272 probes): after git -C on a missing directory (cannot change to ..., No such file or directory), Friday printed a 95% confidence hint to use python3 to write files (core knowledge fsh_echo_redirect). The hint does not match the failure, and the confidence claims a certainty nobody measured.
- Found, not fixed (2026-10-04, INT-272 write-failure probe): write_versions tells the reader to rerun cicomplete and answer skip for crates already bumped. That holds only in a terminal; an unattended rerun needs --bump <tool>=skip, so the message assumes a terminal it did not check for. The wording is from INT-271.

## The Rule
"A message that names a cause it did not see is a guess wearing a uniform."

## Versions
- novashell 5.0.2 -> 5.0.3 (patch)
- engine 4.1.4 -> 4.1.5 (patch)
