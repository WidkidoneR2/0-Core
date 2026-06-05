---
id: 304
title: "fsh test infrastructure v1 -- permanent regression suite, Friday-aware"
status: complete
date: 2026-05-14
type: arch
tags: [fsh, testing, regression, friday, deploy, infrastructure]
depends_on: [299, 300]
---
## The Problem
fsh_audit.sh is a bash script with 75 tests.
It works but it is not a permanent infrastructure:
  - Lives outside the Rust ecosystem
  - Results not stored anywhere
  - No regression tracking over time
  - Not wired into deploy pipeline
  - No performance timing per test
  - Friday cannot see test history
  - No coverage reporting
  - No test categories or modules

A shell that is a daily driver deserves a test suite
that is as intentional as the shell itself.

---
## The Vision
fsh-test: a Rust binary that tests fsh permanently.

Not just "does it pass today" but:
  - "did it pass yesterday?"
  - "which commit broke this test?"
  - "how long does this operation take vs last week?"
  - "which shell paths have zero test coverage?"
  - "Friday, what is the test health trend?"

The test suite is a first-class forest citizen.
It has its own deploy, its own registry entry,
its own Friday integration.

---
## Architecture

### Binary: fsh-test
Location: rust-tools/fsh-test/
Deployed to: scripts/fsh-test, ~/.cargo/bin/fsh-test

### Test structure
Each test is a struct implementing a Test trait:
  name: &str
  category: Category
  run() -> TestResult

Categories:
  Tilde       -- expansion in all contexts
  Pipes       -- pipe chains, SIGPIPE, truncation
  Vocabulary  -- all 10+ forest vocabulary words
  Heredoc     -- heredoc collection and dispatch
  Parallel    -- parallel{} block execution
  Signals     -- Ctrl+C, SIGPIPE, SIGTERM handling
  FdLeaks     -- file descriptor stability
  Zombies     -- no zombie processes after commands
  Performance -- execution time tracking
  Regression  -- specific bugs that were fixed (never regress)

### Results stored in state.db
Table: fsh_test_results
  id, test_name, category, passed, duration_ms,
  commit_hash, timestamp, fsh_version

Friday can query:
  "which tests have regressed in the last 7 days?"
  "what is the slowest test category?"
  "how many tests pass on this commit?"

### Deploy gate
deploy faelight-shell runs fsh-test first.
If any Regression category test fails: deploy blocked.
If >5% of tests fail: deploy blocked with report.
If performance degrades >20%: warning, not block.

### Coverage reporting
fsh-test --coverage shows:
  which vocabulary words have tests
  which execution paths are tested
  which error paths are untested
  coverage % per category

### Friday integration
After each deploy, Friday records:
  test pass rate
  slowest tests
  any new failures
Friday can say:
  "test coverage dropped 3% after your last commit"
  "the heredoc tests have been failing intermittently"

---
## Migration from fsh_audit.sh
Phase 1: port all 75 existing tests to Rust
Phase 2: add regression tests for every bug fixed in INT-298/299
Phase 3: add performance tests
Phase 4: wire into deploy pipeline
Phase 5: Friday integration

---
## Gates
Phase 1:
- [x] fsh-test binary builds and deploys -- v1.0.0 deployed 2026-05-16
- [x] all 75+ fsh_audit.sh tests ported to Rust -- 81 tests total
- [x] fsh-test shows pass/fail per test with timing
- [x] results stored in state.db -- test_name, category, passed, duration_ms, commit, timestamp

Phase 2:
- [x] regression category covers INT-298/299 bug fixes -- SIGPIPE, fsh -c, grep/awk, pipes
- [x] fsh_audit.sh retired to tests/archived/ 2026-05-16

Phase 3:
- [x] performance tracking per test -- --perf flag shows avg/max per category
- [x] baseline established -- heredoc 3ms, pipes 3ms, regression 3ms, tilde 3ms, vocabulary 6ms
- [x] Friday can query state.db -- friday_knowledge updated, full trend analysis deferred to Friday v3

Phase 4:
- [x] deploy faelight-shell runs fsh-test automatically -- regression gate active
- [x] regression failures hard block deploy -- exit 1 on any regression failure 2026-05-16
- [x] deploy shows test summary -- 18 regression tests shown after deploy

Phase 5:
- [x] Friday knows test history -- friday_knowledge updated after each run
- [x] Friday stores regression alerts in friday_knowledge -- session brief integration is INT-246 scope
- [x] coverage reporting implemented -- --coverage flag shows per-category coverage

Final:
- [x] fsh-test is the permanent regression suite -- fsh_audit.sh retired
- [x] process documented -- enforcement via code review, cistart gate is INT-247 scope
- [x] no deploy without passing tests -- regression gate blocks faelight-shell deploy

---
"A shell that cannot test itself
cannot know if it is still itself.
The test suite is not a safety net.
It is the forest checking its own health." 🌲
