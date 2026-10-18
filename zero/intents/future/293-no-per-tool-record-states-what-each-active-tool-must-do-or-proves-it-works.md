---
id: 293
date: 2026-10-17
type: future
title: "no per-tool record states what each active tool must do or proves it works"
status: planned
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
- [ ] G0 census, read-only: a table of the 17 tools against the facts above, with the owner of each
      fact, whether a census file exists, whether `--version` answers in a clean room, whether a
      closed pipe panics, and who calls the tool. Evidence is the table, kept in ~/.cache/zero.
- [ ] G1 rulings, each approved by Christian: the minimum fields of a per-tool record, its home, which
      existing readers switch to it, and what a tool with no record is.
- [ ] G2 red. A check that fails today for at least one measured gap from G0 (for example a tool
      whose `--version` fails in a clean room, or a tool with no record).
- [ ] G3 build. G2 green, for the first record only, on the debug build; no new crates.
- [ ] G4 per-tool intents: for every tool whose census row shows a gap, an intent exists whose gates
      close that gap; tools with no gap are listed as such, with the evidence.
- [ ] G5 doors: one ship at the end where code changed; the deployed tools tested in a new shell;
      nsh-test all passing; d 0 failed; core intent validate clean. The close offers its bumps.
- [ ] G6 commits: Intent INT-293 and the plan Seal with its note under refs/notes/seals; pushed
      with the pre-push hook passing; main and refs/notes/seals reach GitHub; core intent trace
      reads every seal intact.
- [ ] G7 each gate carries evidence per INT-158.

## Non-goals
- No ZERO//PROTOCOL framework, no capability enforcement, no tool generation. They stay in the idea
  bank until G1 shows what a small contract would hold.
- No new crates. No sudo. Omarchy untouched.
- No per-tool intent without a measured gap.
- The readers of tools.toml change only as decision 149 already rules.

## Dependencies
depends_on: 291 (its G1a and G1b decide which documents a tool record may feed).
