---
id: 299
date: 2026-10-22
type: future
title: "Fix Health in Welcoming, needs to stop lying"
status: complete
tags: [nsh, novashell, health, welcome]
---

## Vision
When a terminal opens, the welcome shows a health number that was measured, or says it does not know. It never shows a number nobody measured. Opening a terminal and typing nsh show the same health, because every place that displays health asks one owner. This is the first fix held to the standard written in INT-300: unknown is a word, never a zero.

## The Problem
Seen 2026-10-22 about 22:30. Opening a terminal printed `258/258 shell 0% (red cross) health`. Typing nsh printed 89% with the warning mark. Running d reported 89%. One machine, one minute, two answers.

Measured the same evening, before any change:
- The banner row reads health from the file ~/.cache/zero/health-status through core_integration::health(), which is paths::read_health() (main.rs:3447 and 3713). That reader returns None for an empty or unparseable file and the banner then prints health unknown. A file reader alone cannot print 0 unless the file holds 0.
- That file has one writer: doctor/mod.rs:752, a plain std::fs::write. It truncates and then writes, with no temporary file and no rename, and it writes whatever health holds with no check that the number was measured.
- 22,011 doctor runs are recorded in state.db (health_patterns). The lowest score ever recorded is 38. No run scored 0, no run had 0 passed, and the newest ten all read 89 (25 passed, 3 warned, 0 failed, 0 unknown). So no recorded run explains a 0 in the file, and the 0% is NOT yet attributed. This intent does not guess.
- Zero collapses that can print 0 as a fact exist today: doctor/mod.rs:319 turns determinable == 0 into a score of 0 and line 752 would write it; prompt.rs:269 is db.health_score().unwrap_or(0) and feeds the prompt badge (INT-230 G4 named it as remaining work and left it).
- health_score() (db.rs:549) takes the newest event with domain doctor of ANY action and reads detail.health. 106 of the newest 3000 doctor events have none (151 health_check_failed events exist, the last on 2026-10-01), and each of those, if it were newest, makes the prompt show 0.
- Two sources of health in one startup: the banner reads the cache file, while the prompt badge and the status commands read state.db events. They agree today only because both hold 89.
- Terminal-open nsh starts a detached core doctor run when the newest doctor event is older than this boot (main.rs:67 to 119). The banner shows the last known value for that launch and the file is rewritten while the session runs.
- goals/mod.rs:31 reads health_cache() (cache_dir()/health.txt), which has no writer found in the tree, and falls back to 95. Four more domains use the same accessor.
- Existing protection: nsh-test asserts that core doctor check does not write the health cache (nsh-test main.rs:2145).

## The Solution
Measure first, then fix only what the measurement names. One concern per change.

Rulings, written before any code (change one by editing this section and saying so in the commit):
- R1 Reproduce before fixing. G1 captures what the file holds, its age and what the banner printed at terminal open, at a typed nsh and after d. G2 states the cause from evidence, or states that it did not reproduce and files that as a finding. No fix gate is ticked on a guess.
- R2 One owner for what the screen says: core_integration::health(), returning Option, is the only source for the banner, the prompt badge and the health status commands. None is shown as unknown.
- R3 The writer tells the truth: write to a temporary file in the same directory and rename it over health-status, so no reader ever sees a partial file; when no check was determinable, write no number and remove the stale file, so readers say unknown.
- R4 health_score() only reads events whose action is run, so a newer check event cannot turn into a score.
- R5 A number older than this boot is shown as stale, or as unknown, never as current.
- R6 The dead health.txt readers are filed as findings with core intent find and are not fixed here.

## Success Criteria
- [x] G1 REPRODUCE: with a read-only observer, record for three fresh terminal opens the contents and mtime of health-status, what the banner printed at open, what it printed at a typed nsh, and what d reported; say plainly whether the 0 came back
      <!-- evidence: observer run 2026-10-24 07:58 to 07:59 against the deployed pre-fix nsh (log /tmp/g1-observer.txt), three pty-driven fresh opens, each with a typed nsh and d. health-status held 89 before and after every open (mtimes 07:46:15, 07:58:31, 07:58:52, 07:59:16). Banner at open: 89 percent, banner at typed nsh: 89 percent, d: Health 89 percent (25 of 28 determinable), in all three. The 0 did NOT come back. Limit: pty-driven opens, not a terminal emulator window. -->
- [x] G2 ATTRIBUTE: the cause of the 0 is named in one sentence with evidence tied to a file and line, or, if it did not reproduce, that is stated and a finding is filed with core intent find; no later fix gate is ticked on a guess
      <!-- evidence: did not reproduce (G1), stated plainly; finding F-0068 filed for the unattributed 0 percent. Routes measured instead, each with its own red test: in-place rewrite of health-status gave 4339 empty reads in 10000 (paths.rs write_health); determinable 0 wrote 0 (doctor/mod.rs health_percent); db health_score and the refresh check read the newest doctor event of any action, 151 health_check_failed events carry no detail.health, 81.3 h across windows, last 2026-10-01; prompt.rs unwrap_or(0). These explain the historic prompt 0 percent windows, not necessarily the recent sighting. -->
- [x] G3 RED FIRST: a test names each way the screen can show 0 for health nobody measured (empty file, a 0 written by an unmeasured run, newest doctor event without detail.health, determinable == 0) and each fails before the fix, shown failing
      <!-- evidence: red plans shown failing before each fix, 2026-10-24. Seals: writer A 794c0ed9d1f23e5d and B 665bbe0aa943744d; owner CA a17ac3ebfa5c7fb1; badge CB1 0e65e9e45d8540a0 and CB2 b24436fe8b6ad02d; stale R1 37ec0f536afc3623 and R2 dd805cde18ddd259; refresh 0451b86fb7bd269e; class 41c9ffec2b0d10f8 (29 sites listed failing). -->
- [x] G4 ONE OWNER: the banner, the prompt badge and the health status commands all take health from core_integration::health() as an Option; prompt.rs:269 no longer uses unwrap_or(0) and shows unknown; a search lists every caller that displays health and shows no second source
      <!-- evidence: db.rs health_score is core_integration::health().map(i64::from); prompt.rs health_badge(Option) shows unknown. Tests health_owner_tests (2) and health_badge_tests (3) green; cargo test -p novashell 253 passed. Seals: red CA a17ac3ebfa5c7fb1, fix FD 3cb0468b22fc50d0 and FP 00dbd1afd0e3ea71. Other readers searched with fsearch; every second source is filed (F-0069 to F-0080) and listed in the G11 table. -->
- [x] G5 health_score() reads only events with action run; a test with a newer health_check_failed event proves the score does not change
      <!-- evidence: db.rs health_owner_tests g5_health_score_does_not_read_the_event_stream forbids any events read in health_score, so no newer event of any action can change the score; this is stronger than reading only action run. refresh_health_if_stale now selects action run only (main.rs, health_refresh_tests). Seals: red CA a17ac3ebfa5c7fb1 and refresh 0451b86fb7bd269e, fix FD 3cb0468b22fc50d0 and refresh 9420e1d29f53dba0. An event-injecting test was not built: it needs process-wide env changes that race the other db.rs tests. -->
- [x] G6 THE WRITER: health-status is written to a temporary file and renamed; when determinable is 0 no number is written and the stale file is removed; one test runs a reader against the writer for 10,000 iterations with zero empty or unparseable reads, and one test makes every check unknown or blocked and shows the reader says unknown
      <!-- evidence: paths.rs write_health is atomic (tmp then rename) and None removes the file; doctor/mod.rs health_percent returns None when determinable is 0. health_write_tests (6) green including g6 reader against writer for 10,000 iterations; cargo test -p core 163 passed. Measured before the fix: 4339 of 10000 reads empty. Seals: red A 794c0ed9d1f23e5d and B 665bbe0aa943744d, fix FA e8b385f8cb0b376b and FB 626f11252ab07c2f. -->
- [x] G7 STALE: a health-status older than the current boot is shown as stale or unknown by the banner, proven by a test that injects the boot time and the file time
      <!-- evidence: core_integration.rs health() hides a number written before this boot (health_if_current with injected boot and file times); 7 g7_ tests in health_stale_tests green. Seals: red R1 37ec0f536afc3623 and R2 dd805cde18ddd259, fix F1 81c12a4f4c19ee1a and F2 61ad01db82cada29. Trade-off kept: the first launch after a boot shows unknown until the doctor finishes. -->
- [x] G8 PARITY: for the same file state the terminal-open path and the typed nsh path print the same health, proven by a test that drives both entry paths, then demonstrated in a fresh terminal on this machine
      <!-- evidence: automated: nsh-test welcome_299_parity_every_route_to_the_same_file_prints_the_same_health (print_welcome has exactly one call site; explicit XDG_CACHE_HOME route and HOME default route print the same for a measured 73 and for no file; seal 8d97084a59d373f0). Demonstrated 2026-10-24 08:02 to 08:03 in fresh terminals on this machine with the shipped nsh 5.3.6: health-status held 89, the terminal-open banner showed 89 percent, the typed nsh banner showed 89 percent, identical. Limit: one file state shown live (89); number versus unknown parity is proven by the test. -->
- [x] G9 UNKNOWN STAYS UNKNOWN: with no health-status in a scratch cache directory (never the real one) the banner prints health unknown, and the existing behaviour is kept
      <!-- evidence: nsh-test case welcome_299_unknown_stays_unknown_and_a_measured_number_shows runs the real banner on a scratch cache and a scratch state db; its control run with a fresh 61 shows the number. Seal b3a6570f8dd96fb5. nsh-test 260 of 260 passed 2026-10-24. -->
- [x] G10 FILED, NOT FIXED: goals, prioritize, tradeoffs, reaction and snapshot reading health.txt with a made-up fallback is filed as findings with core intent find, one per site, naming the class (unknown is not a number)
      <!-- evidence: filed with core intent find, class unknown is not a number: F-0069 goals, F-0070 prioritize, F-0071 tradeoffs, F-0072 reaction, F-0073 snapshot; F-0068 the unattributed 0 percent; further same-class sites F-0074 events, F-0075 predict, F-0076 reaction history, F-0077 decisions, F-0078 daemon, F-0079 doctor trend, F-0080 shell listing. -->
- [x] G11 THE CLASS: a test scans the display code for health readers that fall back to a number (unwrap_or(0), unwrap_or(95), unwrap_or(100)) and fails on any, with the known filed sites listed as exceptions so the exception list is visible
      <!-- evidence: core test g11_no_health_read_falls_back_to_a_number_except_the_known_filed_sites (doctor/mod.rs health_class_tests) scans engine and shell sources; the KNOWN table lists 29 filed sites, each with its finding id. Red with an empty table failed on 29 sites (seal 41c9ffec2b0d10f8); green with the table (seal 0c6033fd2d522fbc); cargo test -p core 163 passed. -->
- [x] G12 doors: nsh-test all passing, d 0 failed, core intent validate exit 0; one ship at the end and the deployed nsh tested in a new terminal
      <!-- evidence: nsh-test 260 of 260 passed (after the G8 and G9 cases were added); d: 0 failed, Health 89 percent (25 of 28 determinable); core intent validate exit 0 (337 intents valid); cargo test -p core 163 passed; cargo test -p novashell 253 passed. One ship 2026-10-24 08:02: 15 shipped, 0 failed (nsh 5.3.6, core 4.4.5). Deployed nsh tested in new terminals (see G8). Commits 1bc6ed9b and e64ce78f. -->
- [x] G13 each gate carries evidence per INT-158; INT-300 G13 (the welcome bug line) waits for this intent to complete
      <!-- evidence: gates G1 to G12 above each carry an evidence comment, checked by listing the gate lines 2026-10-24. INT-300 G13 (the welcome bug line) can now proceed once this intent is completed. -->

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->

## Versions
- core 4.4.5 -> 4.4.6 (patch)
- nsh 5.3.6 -> 5.3.7 (patch)
- nsh-test 2.0.9 -> 2.0.10 (patch)
- zero-core 1.0.3 -> 1.0.4 (patch)
