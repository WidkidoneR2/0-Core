---
id: 273
title: "faelight-maintain -- The Forest Stays Current"
status: in-progress
date: 2026-05-06
tags: [maintenance, dependencies, rust, updates, health, longevity, audit, technical-debt]
---
Code does not rot overnight.
It drifts. Slowly. Quietly.
A warning here. A deprecated API there.
Until one day the forest cannot build.
faelight-maintain prevents that day from ever coming.
The forest is 96.3% Rust. Every crate has a version.
Every version has a shelf life.
Every deprecation warning is a debt payment deferred.
faelight-maintain is the forest's immune system.
It scans. It surfaces. It suggests.
It never acts without you.
---
THE PROBLEM
Every cargo build shows this:
  "wl-clipboard-rs v0.8.1 -- will be rejected by a future Rust version"
That warning has appeared for weeks.
Nobody acted on it because there was no system to act on it.
faelight-maintain creates that system.
Currently unknown:
  How many crates are outdated across 50+ rust-tools?
  Which dependencies have security advisories?
  Which APIs are deprecated and will break on next Rust edition?
  Which tools have not been rebuilt in 90+ days?
  What is the total technical debt of the forest?
faelight-maintain makes all of this visible. Always.
---
WHAT IT DOES
Three modes. One command.
MODE 1 -- SCAN (default)
  faelight-maintain scan
  Reads every Cargo.toml in rust-tools/
  Checks current version vs latest on crates.io
  Flags: outdated, deprecated, security advisory, future-incompat
  Shows: summary table with severity levels
  Output example:
    crate                  current   latest   status
    wl-clipboard-rs        0.8.1     0.9.0    ⚠️  outdated + future-incompat
    smithay-client-toolkit 0.19      0.19     ✅ current
    rusqlite               0.31      0.32     📦 minor update available
    ratatui                0.29      0.29     ✅ current
  Total: 3 outdated, 1 critical, 47 current
MODE 2 -- AUDIT
  faelight-maintain audit
  Runs: cargo audit (security advisories from RustSec)
  Runs: cargo outdated (version comparison)
  Runs: cargo +nightly report future-incompatibilities
  Consolidates into one forest health report
  Stores results in state.db (maintain_audit table)
  Friday reads audit history to spot recurring debt patterns
MODE 3 -- FIX
  faelight-maintain fix wl-clipboard-rs
  Shows current version, available updates, breaking changes
  Asks: "Update wl-clipboard-rs from 0.8.1 to 0.9.0? (y/n)"
  If yes: updates Cargo.toml, runs cargo build, runs d
  If build fails: git restore Cargo.toml -- automatic rollback
  If health drops: git restore -- automatic rollback
  Never leaves the forest in a broken state
---
FRIDAY INTEGRATION
Friday watches maintain_audit history.
Patterns it learns:
  "wl-clipboard-rs has been flagged for 3 consecutive audits.
   Technical debt accumulating. Recommend scheduling fix."
  "smithay-client-toolkit has a new major version.
   4 tools depend on it. Upgrade requires coordination.
   Suggest: create intent for coordinated upgrade."
  "Last forest-wide audit: 47 days ago.
   Recommend monthly audits before they become emergencies."
Friday surfaces maintain insights in the bar center zone
when confidence >= 0.85:
  "3 dependencies need attention. Run: faelight-maintain scan"
---
SCHEDULED AWARENESS (not automation)
faelight-maintain does NOT auto-update anything.
Forest philosophy: nothing runs without human authorization.
Instead: awareness on a schedule.
  Monthly: Friday surfaces a reminder
  "It has been 30 days since the last maintenance audit.
   Run faelight-maintain scan when ready."
  On every deploy: check if the deployed tool has outdated deps
  "faelight-bar deployed. Note: wl-clipboard-rs is outdated.
   Run faelight-maintain fix wl-clipboard-rs when ready."
  On cistart: check if intent involves affected tools
  "INT-239 involves faelight-bar. 1 dependency needs attention."
---
TECHNICAL ARCHITECTURE
New binary: faelight-maintain (Rust, pure)
Reads: all Cargo.toml files in rust-tools/
Calls: cargo outdated, cargo audit (if installed)
Writes: state.db maintain_audit table
Integrates: Friday knowledge engine for pattern detection
New state.db table: maintain_audit
  id INTEGER PRIMARY KEY
  scanned_at INTEGER
  tool TEXT
  crate TEXT
  current_version TEXT
  latest_version TEXT
  severity TEXT (critical/warning/info/ok)
  advisory TEXT
  resolved INTEGER DEFAULT 0
  resolved_at INTEGER
New core commands:
  core maintain scan      -- full forest dependency scan
  core maintain audit     -- security + compatibility audit
  core maintain history   -- past audit results
  core maintain debt      -- total technical debt summary
---
THE wl-clipboard-RS FIX (immediate action)
This intent was created because of a real warning seen every build.
First act after building faelight-maintain: fix this warning.
  faelight-maintain fix wl-clipboard-rs
  Current: 0.8.1  Latest: 0.9.0
  Confirm update? y
  → Update Cargo.toml
  → cargo build --all
  → d (health check)
  → fg done "fix: bump wl-clipboard-rs to clear future-incompat warning"
---
GATES
Phase 1 -- Foundation:
[ ] maintain_audit table created in state.db
[ ] faelight-maintain scan reads all Cargo.toml files in rust-tools/
[ ] Version comparison works: current vs crates.io latest
[ ] Future-incompat warnings parsed from cargo report
[ ] Summary table renders cleanly in terminal
Phase 2 -- Audit:
[ ] cargo audit integration (security advisories)
[ ] cargo outdated integration
[ ] Results written to maintain_audit table
[ ] core maintain history shows past audits
[ ] core maintain debt shows total forest debt score
Phase 3 -- Fix mode:
[ ] faelight-maintain fix <crate> updates Cargo.toml
[ ] Automatic rollback on build failure
[ ] Automatic rollback on health drop
[ ] Confirmation required -- nothing auto-applies
Phase 4 -- Friday Integration:
[ ] Friday reads maintain_audit for pattern detection
[ ] Recurring debt surfaces as Friday signal
[ ] Deploy hook surfaces outstanding debt for deployed tool
[ ] Monthly reminder surfaced by Friday
Phase 5 -- Immediate debt resolution:
[ ] faelight-maintain scan run -- full forest picture
[ ] wl-clipboard-rs bumped to clear future-incompat warning
[ ] All critical advisories resolved or documented
[ ] Forest builds clean with zero future-incompat warnings
Presentation Gate -- MUST PASS before summer presentation:
[ ] Every tool in rust-tools/ builds with zero warnings
[ ] cargo build --workspace shows Finished with nothing above it
[ ] No dead code warnings anywhere in the forest
[ ] No future-incompat warnings anywhere in the forest
[ ] wl-clipboard-rs warning resolved
[ ] Build is clean and silent -- Graydon sees only the result
Final Validation:
[ ] faelight-maintain scan shows complete dependency picture
[ ] wl-clipboard-rs warning is gone from every cargo build
[ ] Friday surfaces a maintenance reminder at the right moment
[ ] Christian says: "I know exactly where the technical debt lives"
[ ] The forest builds clean. Every time.
"The forest does not drift into obsolescence.
It tends itself.
Not automatically -- intentionally.
One crate at a time.
Always with understanding.
Never blindly." 🌲
