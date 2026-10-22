---
id: 293
date: 2026-10-17
type: future
title: "no per-tool record states what each active tool must do or proves it works"
status: in-progress
tags: [tools, census, contract, devbox, per-tool]
depends_on: [291]
---

## Vision
Each active tool has one small record, kept where the tool lives, stating what it does, how it
answers for its version, who calls it and what proves it works. The registry, the index and the
census read that record instead of three hand-kept copies. It starts as a census, and only a
contract that earns its keep is added.

## The Problem
Measured 2026-10-17.

- The index lists 17 active tools. Facts about one tool live in separate places with separate owners:
  Cargo.toml (name, version, description), zero/registry/tools.toml (category, status, usage;
  read by more than one tool, which decision 149 reserves to the core), the README (prose), the devbox census file
  (zero/shell/devbox/census/<tool>.toml: description, policy, command, expect), the aliases in
  ~/.config/nsh/config.nsh, and the doc tables in the state database.
- The census file already has the shape of a check. How many of the 17 tools have one is not
  measured.
- zero-deadwood records that by 2026-09-23 four tools still panicked on a closed pipe (teach,
  zero-docs, zero-git, zero-context) and six tools had fixed it in four different shapes
  (zero-deadwood main.rs:656-659). One per-tool checklist would have caught that.
- No intent states, tool by tool, what each must do. Gaps are found by accident, as the README
  staleness was found while closing INT-290.

## The Solution
Census first, contract second. G0 builds one row per tool: which facts exist, where each lives,
who owns it, and what proves the tool works. G1 rules the minimum fields and their home (candidates:
the census file, the registry, or [package.metadata] in Cargo.toml), chosen against decision 149's
one-owner rule. A per-tool intent is filed only where the census shows a gap, and its gates close
that gap. ZERO//PROTOCOL stays an idea; this intent decides whether a small contract is worth
building at all.

## Success Criteria
- [x] G0 census, read-only: a table of the 17 tools against the facts above, with the owner of each
      fact, whether a census file exists, whether `--version` answers in a clean room, whether a
      closed pipe panics, and who calls the tool. Evidence is the table, kept in ~/.cache/zero.
<!-- evidence: demonstrated: 17-row table at ~/.cache/zero/int293-g0-table.txt (sha256 prefix 1d162a55cbfb, measured 2026-10-22 01:04), with the recon outputs int293-recon.txt, int293-recon2.txt and int293-recon3.txt beside it. Columns: version agreement, census record, clean-room version answer, closed-pipe result, source handling, derived callers; the header names the owner of each fact. Verdict column from zero-sandbox verify on zero/shell/devbox/census, 2026-10-22 01:01: 12 passed, 3 failed (db-browse exit 1, friday-chat exit 1, zero-insightd exit 101), 1 undetermined (zero-ade, no answer in 10s); core has no case. Re-run: zero-sandbox verify zero/shell/devbox/census -->
- [x] G1 rulings, each approved by Christian: the minimum fields of a per-tool record, its home, which
      existing readers switch to it, and what a tool with no record is.
<!-- evidence: demonstrated: rulings a to f written below, after the G0 table. Christian approved the six rulings in chat on 2026-10-22 (All approved). The sentence in f that verify is not wired into the pre-push gate was added after that and is approved by applying this reviewed plan. -->
- [x] G2 red. A check that fails today for at least one measured gap from G0 (for example a tool
      whose `--version` fails in a clean room, or a tool with no record).
<!-- evidence: demonstrated: nsh-test case every_active_tool_has_a_census_record (Plan 2, Seal ea8e4d124e81c12b) failed on the debug build, 2026-10-22 01:13: 1 of 17 active tools have no census record: core. Output kept in ~/.cache/zero/int293-red.txt. The same run also failed crate_directory_has_one_owner, because that case wrote the census path as a literal outside zero_core::paths; that was this intent's own mistake and this plan fixes it with zero_core::paths::census_dir() rather than an allowance. -->
- [x] G3 build. G2 green, for the first record only, on the debug build; no new crates.
<!-- evidence: demonstrated: Plan 3 (Seal 0c2123457731f20f) added zero_core::paths::census_dir() and made the nsh-test case use it; zero/shell/devbox/census/core.toml is the first record (command core --version, exit 0, policy devbox). On the debug build, 2026-10-22 01:24: cargo build exit 0; nsh-test --failed exit 0, All tests passed (~/.cache/zero/int293-green.txt), so every_active_tool_has_a_census_record and crate_directory_has_one_owner both pass; zero-sandbox verify shows [core] passing, 13 passed 3 failed 1 undetermined (~/.cache/zero/int293-verify.txt), the three failures and the undetermined being the G4 gaps. git status at the time shows no Cargo.toml or Cargo.lock change or addition, and a new crate needs one: no new crates. -->
- [x] G4 per-tool intents: for every tool whose census row shows a gap, an intent exists whose gates
      close that gap; tools with no gap are listed as such, with the evidence.
<!-- evidence: demonstrated: intent 298 (zero/intents/future/298-six-active-tools-do-not-answer---version-so-the-census-cannot-tell-whether-they-run.md, created with inta 2026-10-22 01:37, the class intent chosen by Christian) has one gate per gap for the six tools db-browse, friday-chat, zero-ade, zero-insightd, ship, zero-gen, with file:line from source reads and the measurements: verify 13 passed 3 failed 1 undetermined, and direct runs ship --version exit 2 and zero-gen --version exit 2 (2026-10-22 01:34). Tools with no gap, each passing its own record in zero-sandbox verify (~/.cache/zero/int293-verify.txt) and the command that record runs: core (core --version); zero-daemon (zero-daemon --version); zero-git (zero-git --version); zero-sandbox (zero-sandbox --version); nsh (nsh --version); teach (teach --version); zero-docs (zero-docs --version); zero-context (zero-context --version); zero-deadwood (zero-deadwood --version); nsh-test (nsh-test help); zero-gate (zero-gate --version). Note: nsh-test (nsh-test help) asks help in its record and answers --version itself, zero/shell/nsh-test/src/main.rs:6136, read 2026-10-22 01:41. -->
- [x] G5 doors: one ship at the end where code changed; the deployed tools tested in a new shell;
      nsh-test all passing; d 0 failed; core intent validate clean. The close offers its bumps.
<!-- evidence: demonstrated: one ship at the end where code changed: ship nsh-test --dry-run named only nsh-test; ship nsh-test shipped nsh-test 2.0.8 (2026-10-22, around 01:45); only nsh-test was deployed. New shell: exec /home/christian/.local/bin/nsh. Doors on the deployed binaries: nsh-test exit 0, 258 / 258 passed, All tests passed, with every_active_tool_has_a_census_record and crate_directory_has_one_owner both passing (~/.cache/zero/int293-doors-suite.txt); core intent validate exit 0, all 334 intents valid (int293-doors-validate.txt); zero-sandbox verify shows [core] passing and the unchanged 13 passed 3 failed 1 undetermined, the remainder being INT-298 (int293-doors-verify.txt); d in the new shell: 26 passed, 0 failed, 2 warnings (uncommitted changes, update readiness, both for the commit to clear), health 92%. The clause The close offers its bumps cannot be shown before cicomplete runs, and cicomplete refuses an intent with open gates (INT-332, zero/engine/src/domains/intent/mod.rs:2541), so it is recorded in the evidence of G7 at the close. -->
- [ ] G6 commits: Intent INT-293 and the plan Seal with its note under refs/notes/seals; pushed
      with the pre-push hook passing; main and refs/notes/seals reach GitHub; core intent trace
      reads every seal intact.
- [ ] G7 each gate carries evidence per INT-158.

## Rulings (G1, approved by Christian 2026-10-22)
Measured first in ~/.cache/zero/int293-g0-table.txt. Each ruling is the one chosen against the alternatives named.

a. HOME. The per-tool record is the census file, zero/shell/devbox/census/<tool>.toml, run by zero-sandbox verify.
   Not tools.toml: it is core-owned (decision 149) and named by at least twelve source files. Not [package.metadata]:
   no crate uses it, and Cargo owns only name, version and description. 16 of the 17 active tools already have a
   census file, in one shape.
b. FIELDS. command, expect, policy and a description of the check. The record carries no version, no tool description
   and no callers. Cargo.toml owns the first two (the version agrees across Cargo.toml, tools.toml and the deployed
   binary for all 17); callers are derived by scan. A copy could only disagree with its owner.
c. CONTRACT. Every active tool answers --version with exit 0, its name and its version, and reads nothing from its
   environment to do it. zero-git.toml already says so and zero-gate (main.rs:467) already follows it. Not a per-tool
   opt-out flag: the six tools that fail today (ship, zero-gen, db-browse, friday-chat, zero-ade, zero-insightd)
   would opt out.
d. READERS. None switch. The readers of tools.toml change only as decision 149 rules. The only census reader the
   recon found is zero-sandbox verify.
e. NO RECORD. An active tools.toml row (deployable true, retired false) with no census file is red. zero-doctor and
   zero-core are deployable false and exempt. Today that is core.
f. ENFORCEMENT. An nsh-test case reads the active rows through zero-core and lists the census directory, so it needs
   no built tool and is fast enough for the pre-push gate. zero-sandbox verify is not wired into that gate by this
   intent: it is red today for four tools and would refuse every push. Its verdict is the closing test for the G4
   intents.

## Non-goals
- No ZERO//PROTOCOL framework, no capability enforcement, no tool generation. They stay in the idea
  bank until G1 shows what a small contract would hold.
- No new crates. No sudo. Omarchy untouched.
- No per-tool intent without a measured gap.
- The readers of tools.toml change only as decision 149 already rules.

## Dependencies
depends_on: 291 (its G1a and G1b decide which documents a tool record may feed).
