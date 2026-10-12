---
id: 285
date: 2026-10-08
type: future
title: "nsh-test falls back to the deployed nsh in silence when NSH_BIN is unset, so a red-first run can test the wrong binary"
status: complete
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

- [x] G0 census, read-only. Every place nsh-test starts a shell or reads NSH_BIN (fsh_command,
      fsh_bin, the two doors, any other Command::new of the shell): file:line, and what each
      resolves with NSH_BIN unset, set, and set to a missing path. Also how nsh-test is built
      and deployed. Recorded here before any edit.
<!-- demonstrated 2026-10-12, read-only:
LAUNCH PATHS for the shell under test (nsh-test src/ and tests/ scanned):
repl.rs:43 fsh_bin(), the owner, with the one NSH_BIN read at :44 -- unset: paths::bin_dir()/nsh,
exit 2 if absent; set and present: that path; set and absent: exit 2. Through the owner:
repl.rs:398 (pty REPL), main.rs:133 fsh_command (run_fsh and run_fsh_env :103/:115,
run_fsh_closed_stdout :150), main.rs:5052 store_results, :5273 and :5350 the banner.
DOOR 1 main.rs:324 run_fsh_status, own read at :325, default bare "nsh" -- unset: first nsh on
PATH; set and present: that path; set and absent: spawn error, the case fails, nothing refused.
Callers :819, :861, :2736, :2753, :4814, :4982.
DOOR 2 main.rs:2764 dashc_missing_operand_is_a_usage_error: the same line, the same answers.
NOT THE SHELL: main.rs:86 run_deadwood (DEADWOOD_BIN, default "zero-deadwood"); the whoami, sh,
bash, rm, git, core, zero-docs, python3 and zero-gate spawns.
REACHABILITY: main() calls repl::fsh_bin() at :5273 before all_tests() at :5365, so a missing
NSH_BIN target or a missing bin_dir()/nsh exits 2 before any case runs; the doors' set-and-absent
branch is unreachable in a run. --version and --help (:5231, :5235) return earlier, no cases.
bin_dir() (zero-core paths.rs:178) is XDG_BIN_HOME if set and non-empty, else ~/.local/bin.
Measured: XDG_BIN_HOME unset, NSH_BIN unset, the first nsh on PATH is ~/.local/bin/nsh, so the
owner and both doors resolve to the same file today. LATENT: two readers of one fact, diverging
when an earlier PATH entry holds an nsh (~/.cargo/bin is first on PATH) or XDG_BIN_HOME is set.
G1 is structural for that reason; no behaviour case can go red here today, and G2 changes no
case result.
OUT OF SCOPE: novashell reads NSH_BIN (commands/mod.rs:5414, resolve_nsh_binary :12685); G1
counts nsh-test source only.
BUILD AND DEPLOY: ship (zero/tools/ship) runs cargo build --release (main.rs:466) and installs
into paths::bin_dir() (:415) by stage-and-rename (:140-150); the registry decides what ships
(INT-265, :494), nsh-test deployable = true at zero/registry/tools.toml:325. Pre-push: zero-gate
runs .githooks/lib/nsh-test-gate.sh (zero-gate main.rs:517), read at G5 before the push. -->
- [x] G1 red. A suite case, nsh_bin_is_read_in_one_place, counts env::var("NSH_BIN") reads in
      nsh-test source and requires exactly one, inside fsh_bin. Shown FAILING on the current code,
      naming main.rs:325 and :2764, output quoted.
<!-- demonstrated 2026-10-12: nsh_bin_is_read_in_one_place added to nsh-test main.rs right
after the dashc no-operand case, a repo_test under Category::Regression. It walks the crate
zero_core::paths::crate_dir("nsh-test") locates, skipping target/ and comment lines, and counts
env::var and env::var_os reads of the name, its needles built from pieces so it does not count
itself. Ruled 2026-10-12 (option i): the one read must be in src/repl.rs, the owner's file; the
owner's function still carries a retired name, so no function name is matched or printed.
Red on the code before the fix (plan seal b54fd037f7ff4dac): cargo fmt --check -p nsh-test exit 0,
cargo build -p nsh-test, then ./target/debug/nsh-test --failed, exit 1, the only failure of 236:
  NSH_BIN is read 3 times, expected once, in src/repl.rs: src/main.rs:2764 | src/main.rs:325 | src/repl.rs:44
The first attempt (seal 6cf09fce33e20a01) also turned crate_directory_has_one_owner and
no_retired_display_name_in_printed_strings red, with a hardcoded crate path and a retired name in
its literals, and labelled door 2 with the nested fn has; it was replaced before this run, and both
guards pass here. -->
- [x] G2 fix. Both doors call repl::fsh_bin(). G1 green; the cases built on run_fsh_status and
      the dashc no-operand case still pass.
<!-- demonstrated 2026-10-12: both doors now read let fsh = repl::fsh_bin(); -- main.rs:325 in
run_fsh_status and :2764 in dashc_missing_operand_is_a_usage_error -- one edit at count 2 (plan
seal 27ef7706bd393260), each site keeping its indentation and its own spawn. cargo fmt --check
-p nsh-test exit 0, cargo build -p nsh-test, ./target/debug/nsh-test exit 0 with NSH_BIN unset:
Results: 236 / 236 passed. Green: nsh_bin_is_read_in_one_place; the six run_fsh_status cases
regression_281_printing_builtin_pipe_refuses, regression_282_dashboard_feeds_wc,
dashc_exit_code_is_passed_through, dashc_signalled_child_is_not_a_success,
dashc_reaches_the_safety_guard and dashc_guard_refusal_stays_off_stdout; and
dashc_missing_operand_is_a_usage_error. Against the G1 run the only result that changed is G1's
own case: as G0 recorded, the doors already resolved to the same binary on this machine. -->
- [x] G3 doors. nsh-test built and deployed the way G0 found, all passing, with the case count up
      by one; d 0 failed.
<!-- demonstrated 2026-10-12: ship nsh-test --dry-run listed only nsh-test 2.0.1; ship nsh-test
then built the release profile and shipped nsh-test 2.0.1 to ~/.local/bin, exit 0, nsh and every
other tool left as they were. The deployed /home/christian/.local/bin/nsh-test, NSH_BIN unset:
under test 5.3.1, nsh_bin_is_read_in_one_place passed, Results: 236 / 236 passed (235 before this
intent), exit 0. d: Passed 26, Warnings 2, Failed 0 -- the warnings are uncommitted changes and
Update Readiness (commit or stash), both the tree G4 commits.
The banner also printed: the binary above is NOT what cargo last built -- SAME VERSION, DIFFERENT
BYTES. Not this intent: git diff --stat HEAD over novashell, zero-core and engine is empty. The
deployed nsh was installed 2026-10-08 16:04, before e8679d19 (2026-10-08 23:03, INT-284), the last
commit touching novashell or zero-core, which was not shipped; ship's workspace build rebuilt the
release nsh at 2026-10-12 08:36. Why those bytes differ is not established here, and the shell is
outside this intent (Non-goals). -->
- [x] G4 commit. Intent: INT-285, Fixes: F-0020, and the plan Seal with its note under
      refs/notes/seals.
<!-- demonstrated 2026-10-12: e1a19f83 carries Intent: INT-285, Fixes: F-0020 and Seal:
27ef7706bd393260. It holds three paths -- the intent moved from future/ to in-progress/, and
nsh-test src/main.rs -- staged only after a check that the tree held exactly those. Its note under
refs/notes/seals is the fix plan text written byte-exact (blob be902f74cc4d) and reads back equal.
core intent trace e1a19f83 reads intent INT-285, fixes F-0020 and seal 27ef7706bd393260 intact,
exit 0. -->
- [x] G5 the push. The pre-push hook passes; main and refs/notes/seals reach GitHub; trace reads
      the seal intact.
<!-- demonstrated 2026-10-12: git push origin main refs/notes/seals, one push, so the pre-push gate
(zero-gate pre-push, which builds novashell and nsh-test and runs the suite with NSH_BIN at
target/debug/nsh) ran once, and git sent both refs only after it exited 0 -- 43.0s, long enough for
that build and suite; the gate's own lines were not shown in the terminal. main f67ac552..e1a19f83
and refs/notes/seals 8769ec57..93389739. git ls-remote then matched the local refs: main
e1a19f83e549, refs/notes/seals 933897392ead. core intent trace e1a19f83 after the push reads fixes
F-0020 and seal 27ef7706bd393260 intact. -->
- [x] G6 each gate carries evidence per INT-158.
<!-- demonstrated 2026-10-12: G0 to G5 each carry a demonstrated comment, counted by the plan
that writes this one (five present before it, G5's written beside it). -->

## Non-goals

- The default with NSH_BIN unset stays the deployed shell: fsh_bin already rules that, and the
  banner compares version and bytes against the build. This intent makes every door obey it.
- No change to novashell; the shell is not the subject here.
- The dashc no-operand case keeps spawning directly, because it needs no operand; only the binary
  it names changes.
- No new crates.

## Versions
- nsh-test 2.0.1 -> 2.0.2 (patch)
