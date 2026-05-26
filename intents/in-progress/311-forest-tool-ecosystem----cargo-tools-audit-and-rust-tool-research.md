---
id: 311
title: "Forest Tool Ecosystem -- cargo tools audit, unused removal, new Rust tools research"
status: in-progress
date: 2026-05-16
type: maintenance
tags: [cargo, tools, audit, ecosystem, rust, research, cleanup]
depends_on: []
---
## Current Cargo Tools Inventory

FULLY WIRED INTO FOREST:
  cargo-audit    — vulnerability scanning (INT-305, deploy pipeline)
  cargo-deny     — license compliance (INT-305, deploy pipeline)
  cargo-bloat    — binary size reporting (INT-305, deploy pipeline)
  cargo-clippy   — linting (pre-push hooks)
  cargo-fmt      — formatting (pre-push hooks)
  cargo-nextest  — faster test runner (installed, NOT wired yet)

PARTIALLY USED:
  cargo-binstall      — binary install (used manually, not in pipeline)
  cargo-install-update — update installed cargo tools (manual only)
  cargo-set-version   — version writing (installed, NOT wired yet — INT-310)
  cargo-upgrade       — dependency upgrades (installed, NOT wired yet — INT-310)
  cargo-udeps         — unused dep detection (installed, NOT wired yet — INT-310)
  cargo-watch         — hot reload dev (installed, not in forest workflow)
  cargo-add           — add deps (manual use only)
  cargo-rm            — remove deps (manual use only)
  cargo-cache         — cache management (manual use only)
  cargo-flamegraph    — performance profiling (manual use only)
  cargo-miri          — unsafe code checking (rarely used)

SYSTEM TOOLS (non-cargo, in ~/.cargo/bin):
  difft          — structural diff (used in faelight-diff)
  flamegraph     — perf profiling (manual only)
  bump-system-version — unclear purpose, needs audit
  test-intent    — unclear purpose, needs audit

---
## The Problem

1. UNUSED TOOLS: Several installed tools serve no purpose in the current forest workflow
2. DISCONNECTED TOOLS: Some powerful tools (nextest, udeps, watch) are installed but never called
3. UNKNOWN TOOLS: bump-system-version, test-intent — what are these?
4. MISSING TOOLS: There may be valuable Rust ecosystem tools we don't have

---
## Phase 1: Audit Unknown Tools

Investigate:
  bump-system-version — is this a forest script? a cargo tool? what does it do?
  test-intent — is this a forest script? what does it test?

Action: identify, document, and either integrate or remove each.

---
## Phase 2: Wire Disconnected Tools

cargo-nextest:
  Faster, better test output than cargo test
  Drop-in replacement for cargo test
  Wire into fsh-test or as deploy pipeline gate
  Command: cargo nextest run --manifest-path rust-tools/$tool/Cargo.toml

cargo-watch:
  Hot reload during development
  Start with: cargo watch -x "build -p faelight-shell"
  Add as forest dev command: core dev watch <tool>
  Useful during fsh shell stabilization sprints

cargo-udeps:
  Detect unused dependencies — forest has 785 crate dependencies
  Run quarterly or before major releases
  Add to: core dev audit-deps
  May reveal significant cleanup opportunities

cargo-miri:
  Detect undefined behavior in unsafe Rust
  Run on: faelight-shell (has unsafe in PTY handling)
  Run on: faelight-term (has unsafe in rendering)
  Add as optional gate: deploy --strict runs miri

---
## Phase 3: Evaluate Removal

Tools to consider removing:
  cargo-flamegraph / flamegraph — only useful for perf profiling sprints
    Keep if: active perf work planned
    Remove if: not used in 6 months
  cargo-miri — only useful for unsafe code audits
    Keep if: planning unsafe audit
    Remove if: no unsafe code planned

Rule: if a tool has not been used in 3 months and has no planned integration, remove it.

---
## Phase 4: New Rust Tools Research

Tools worth evaluating for the forest:

DEVELOPMENT:
  cargo-expand     — macro expansion viewer (useful for debugging derive macros)
  cargo-hakari     — workspace dependency unification (reduce build times)
  cargo-machete    — another unused dep finder (simpler than udeps)
  cargo-semver-checks — verify semver compliance (INT-310 support)

ANALYSIS:
  cargo-geiger     — count unsafe code (know your risk surface)
  cargo-llvm-lines — see which functions contribute most to binary size
  cargo-modules    — visualize module structure
  cargo-graph      — dependency graph visualization

PERFORMANCE:
  samply           — modern sampling profiler (better than flamegraph)
  dhat             — heap profiling

FOREST-SPECIFIC POTENTIAL:
  tokei            — already have it (code statistics)
  fd               — already have it (find replacement)
  bat              — already have it (cat replacement)
  ripgrep (rg)     — already have it (grep replacement)
  delta            — git diff pager (worth adopting?)
  zoxide           — smart cd (worth adopting in fsh vocabulary?)
  tealdeer (tldr)  — quick command help (forest docs integration?)
  hyperfine        — benchmarking (fsh-test performance baseline?)
  ast-grep         — structural code search (forest code analysis?)
  bacon            — background cargo checker (better than cargo-watch?)

---
## Phase 5: Forest Tool Philosophy Alignment

For each tool, apply the 0-Core philosophy test:
  - Does it increase understanding? (not just convenience)
  - Is it intentional? (not just "nice to have")
  - Does it have a clear home in the forest workflow?
  - Does it respect manual control over automation?

If a tool fails this test, it does not belong in the forest.

---
## Gates

Phase 1 -- Audit unknowns:
- [ ] bump-system-version identified and documented
- [ ] test-intent identified and documented
- [ ] both either integrated with clear purpose or removed

Phase 2 -- Wire disconnected tools:
- [ ] cargo-nextest wired as faster test runner in fsh-test or deploy
- [ ] cargo-watch available as core dev watch <tool>
- [ ] cargo-udeps available as core dev audit-deps
- [ ] each tool has a documented command in the forest

Phase 3 -- Cleanup:
- [ ] tools unused for 3+ months identified
- [ ] removal decision documented for each
- [ ] removed tools uninstalled cleanly (cargo uninstall)
- [ ] no orphaned binaries in ~/.cargo/bin

Phase 4 -- New tools research:
- [ ] each candidate tool evaluated against 0-Core philosophy
- [ ] evaluation documented in state.db or friday_knowledge
- [ ] adopted tools have clear integration path
- [ ] rejected tools have documented reason

Phase 5 -- Ecosystem coherence:
- [ ] every cargo tool in ~/.cargo/bin has a documented purpose
- [ ] every tool either wired into forest or marked manual-only
- [ ] tools.toml updated to reflect tool ecosystem
- [ ] forest has no mystery binaries

Final:
- [ ] ~/.cargo/bin is intentional — every binary has a reason
- [ ] cargo tools meaningfully improve forest quality
- [ ] no tool installed "just in case"
- [ ] Christian can explain why every tool exists

---
"The forest does not collect tools.
It wields them with intention.
Every binary in the path
earns its place." 🌲
