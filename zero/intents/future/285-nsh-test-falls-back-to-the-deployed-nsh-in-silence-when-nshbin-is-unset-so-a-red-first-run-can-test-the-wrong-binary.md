---
id: 285
date: 2026-10-08
type: future
title: "nsh-test falls back to the deployed nsh in silence when NSH_BIN is unset, so a red-first run can test the wrong binary"
status: planned
tags: [nsh-test, tests, nsh, red-first, integrity, finding]
---

## START HERE

Begin at G0, read-only. Filed 2026-10-08 from F-0020 (found by INT-243). Measured the same night:
the second site F-0020 records at main.rs:2646 is now main.rs:2764.

## Vision

nsh-test runs the shell it says it runs. Every case spawns the binary that one owner resolved, so
NSH_BIN means the same thing at every door, a missing NSH_BIN target is refused everywhere, and a
red-first run proves what was built rather than what was deployed.

## The Problem

zero/shell/nsh-test/src/repl.rs:43 fsh_bin() is the owner of the shell under test. Its doc names
the disease: a dropped NSH_BIN quietly tested the deployed shell and the suite reported green
(INT-110, again 2026-08-21). It refuses an NSH_BIN that names a missing binary (exit 2), and with
NSH_BIN unset it uses paths::bin_dir()/nsh and refuses if that is missing.

Two doors bypass it (read 2026-10-08):
- main.rs:325, run_fsh_status: std::env::var("NSH_BIN").unwrap_or_else(|_| "nsh".to_string())
- main.rs:2764, the dashc_missing_operand_is_a_usage_error case: the same line.

At both, an unset NSH_BIN runs whatever "nsh" PATH finds, not the owner's bin_dir path, and an
NSH_BIN naming a missing file is not refused. Every case built on run_fsh_status can therefore
measure a different binary from the rest of the suite while the banner names the owner's.

## The Solution

Both doors ask repl::fsh_bin(). A suite case guards the class: NSH_BIN is read by one function, so
a third door goes red the day it is written. G0 confirms every other launch path already goes
through the owner before anything changes.

## Success Criteria

- [ ] G0 census, read-only. Every place nsh-test starts a shell or reads NSH_BIN (fsh_command,
      fsh_bin, the two doors, any other Command::new of the shell): file:line, and what each
      resolves with NSH_BIN unset, set, and set to a missing path. Also how nsh-test is built
      and deployed. Recorded here before any edit.
- [ ] G1 red. A suite case, nsh_bin_is_read_in_one_place, counts env::var("NSH_BIN") reads in
      nsh-test source and requires exactly one, inside fsh_bin. Shown FAILING on the current code,
      naming main.rs:325 and :2764, output quoted.
- [ ] G2 fix. Both doors call repl::fsh_bin(). G1 green; the cases built on run_fsh_status and
      the dashc no-operand case still pass.
- [ ] G3 doors. nsh-test built and deployed the way G0 found, all passing, with the case count up
      by one; d 0 failed.
- [ ] G4 commit. Intent: INT-285, Fixes: F-0020, and the plan Seal with its note under
      refs/notes/seals.
- [ ] G5 the push. The pre-push hook passes; main and refs/notes/seals reach GitHub; trace reads
      the seal intact.
- [ ] G6 each gate carries evidence per INT-158.

## Non-goals

- The default with NSH_BIN unset stays the deployed shell: fsh_bin already rules that, and the
  banner compares version and bytes against the build. This intent makes every door obey it.
- No change to novashell; the shell is not the subject here.
- The dashc no-operand case keeps spawning directly, because it needs no operand; only the binary
  it names changes.
- No new crates.
