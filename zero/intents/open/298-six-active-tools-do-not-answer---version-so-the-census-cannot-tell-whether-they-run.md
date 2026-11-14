---
id: 298
date: 2026-10-22
type: future
title: "six active tools do not answer --version, so the census cannot tell whether they run"
status: planned
tags: [tools, census, contract, version]
depends_on: [293]
---

## Vision
Every active tool answers one ordinary question the same way: asked for its version, it says its name and version on one line, exits 0, and reads nothing from its environment to do it. Then the census (INT-293) can put the same question to all 17 tools and the answer means this tool starts and knows what it is, not that its real action happened to succeed on the author machine.

## The Problem
Measured 2026-10-22: INT-293 G0 table; zero-sandbox verify 13 passed, 3 failed, 1 undetermined (~/.cache/zero/int293-verify.txt); direct runs and source reads below. Six active tools do not answer --version.
- db-browse (zero/tools/db-browse/src/main.rs:603): takes argument 1 as an initial table name, then enters raw mode and the alternate screen. Clean room: exit 1.
- friday-chat (main.rs:725): handles only chat QUERY; anything else starts the TUI. Clean room: exit 1.
- zero-ade (main.rs:619): main reads no arguments and starts the TUI loop. Clean room: no answer in 10 seconds, undetermined.
- zero-insightd (main.rs:11 and 266): every unknown argument means status, which opens the state database read-write and calls expect on it. Clean room with no database: panic, exit 101. On the author machine the same call succeeds, so its record passes for the wrong reason, and it sets WAL pragmas on the live database to do it.
- ship (main.rs:23-24): clap derive with no version attribute. Direct run with HOME in an empty directory: exit 2, unexpected argument --version. Its census record asks a different question on purpose: ship builds the repo and cannot build in a clean room (ship.toml, measured 2026-09-15).
- zero-gen (main.rs:9-10): the same, exit 2. Its census record asks help instead (zero-gen.toml, measured 2026-09-14).
The G0 table shows yes in its version column for ship and zero-insightd. The direct run (ship, exit 2) and the source read (zero-insightd main.rs:11 and 266) govern.

## The Solution
One owner for the answer, as zero_core::paths is the one owner of locations.
- zero-core gets answer_version!(): a macro that expands at the call site to the calling crate own CARGO_PKG_NAME and CARGO_PKG_VERSION, so it can never print zero-core version for another tool. It is the first statement of main in db-browse, friday-chat, zero-ade and zero-insightd (all four depend on zero-core), before raw mode and before any database or terminal call. As first argument, --version or -V prints NAME VERSION and exits 0.
- ship and zero-gen (clap): one attribute each, command(version). zero-gen keeps its choice not to depend on zero-core.
- ship.toml and zero-gen.toml ask --version like the other fifteen. What each proved before is kept as its own case where the census keeps cases, not dropped: ship reports nothing to build with no repo; zero-gen help exits 0.
- Closing test: zero-sandbox verify zero/shell/devbox/census reads 17 passed, 0 failed, 0 undetermined, run against the deployed tools after the one ship.

## Success Criteria
- [ ] G0 red. ship.toml and zero-gen.toml ask --version; zero-sandbox verify on the deployed tools reads 11 passed, 5 failed, 1 undetermined (db-browse, friday-chat, zero-insightd, ship, zero-gen failed; zero-ade undetermined). Output kept in ~/.cache/zero.
- [ ] G1 owner. zero_core::answer_version!() exists, with a unit test for the pure part (which arguments count as a version request); no new crates.
- [ ] G2 the four zero-core tools. db-browse, friday-chat, zero-ade and zero-insightd call it as the first statement of main. Each, on the debug build with HOME in an empty directory and stdin closed, exits 0 within one second printing its own name and version, and creates nothing in HOME; zero-insightd touches no database.
- [ ] G3 the two clap tools. ship and zero-gen carry command(version); each, on the debug build, prints name and version and exits 0.
- [ ] G4 nothing lost. The property each old record proved (ship: reports nothing to build with no repo; zero-gen: help exits 0) is kept as its own case, or the evidence says why not.
- [ ] G5 doors. One ship at the end; the deployed tools tested in a new shell; zero-sandbox verify reads 17 passed, 0 failed, 0 undetermined (G0 red, now green); nsh-test all passing; d 0 failed; core intent validate clean; the close offers version bumps for zero-core and the six tools.
- [ ] G6 commits. Intent: INT-298 and the plan Seal, with the plan text as a note under refs/notes/seals, pushed with the pre-push hook; core intent trace reads the seals intact.
- [ ] G7 evidence per INT-158 on every ticked gate.

## Non-goals
- No framework for tool command lines, no --help contract, no change to anything a tool does beyond answering --version first.
- zero-gen does not gain a zero-core dependency.
- No new crates; no sudo; Omarchy untouched.
- No per-tool intent: six tools, one contract, one intent (Christian, INT-293, 2026-10-22).

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
