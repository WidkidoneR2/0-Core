---
id: 310
title: "Forest Version Intelligence -- auto-versioning, smart releases, engine coherence"
status: in-progress
date: 2026-05-16
type: intelligence
tags: [versioning, deploy, release, core, engine, intelligence, automation]
depends_on: [305]
---
## The Problem

The forest has three disconnected version namespaces that require manual mental overhead:

BINARY VERSIONS (Cargo.toml — manually bumped):
  core 3.0.0
  faelight-shell 2.1.0
  faelight-term 3.0.0
  ... 50+ tools, all manually tracked

CORE INTELLIGENCE VERSION (hardcoded string in source):
  "intelligence v18 (Synthesis Engine)"
  Updated by hand, no connection to actual capability changes

FOREST VERSION (13.0.0 — manually bumped):
  Tied to no automated signal
  You have to remember to bump it

CURRENT PAIN:
  - Before every deploy: manually decide patch/minor/major
  - Before every release: manually update 3+ version strings
  - Core intelligence version drifts from reality
  - No automated signal tells you when versions deserve bumping
  - Tool versions in Cargo.toml edited by hand every time
  - No audit trail of WHY a version was bumped

---
## The Vision

Deploy analyzes what changed and decides the version bump automatically.
The forest knows its own version. You never manually edit Cargo.toml for versions.
Core intelligence version reflects actual capability, computed from state.db.

"The forest knows how much it has grown.
You don't count the rings manually."

---
## Design

### Auto-Versioning Engine

When `deploy <tool>` runs, before building:

1. Analyze git diff since last deploy tag for this tool
2. Classify changes:
   - Only tests/docs/comments → PATCH
   - New commands, new flags, new behavior → MINOR  
   - Breaking changes, removed commands, domain restructure → MAJOR
   - No changes → skip deploy (already current)

Classification rules (Rust-aware):
  MAJOR signals:
    - pub fn removed or renamed
    - CLI subcommand removed
    - state.db schema breaking change
    - depends_on changed
  MINOR signals:
    - new pub fn added
    - new CLI subcommand or flag
    - new state.db table
    - new domain added
  PATCH signals:
    - impl changes (no API change)
    - bug fixes
    - performance improvements
    - docs, comments, tests only

3. Compute new version from current + bump type
4. Write version to Cargo.toml automatically
5. Deploy with new version
6. Tag git commit with tool@version

### Core Intelligence Version

Currently: hardcoded "intelligence v18 (Synthesis Engine)"
Future: computed from state.db on every deploy

Components:
  - Friday pillars active (count from friday_knowledge domains)
  - Pattern count (from friday_patterns)
  - Prediction accuracy (from friday_predictions)
  - Intent completion rate (from intents)
  - Tool count

Formula:
  intelligence_version = pillar_count * 3 + (pattern_count / 10)
  intelligence_name = derived from milestone thresholds

Milestones:
  v1-5:   Awakening
  v6-10:  Pattern Recognition
  v11-15: Synthesis Engine
  v16-20: Anticipation Layer
  v21-25: Conversational Partner
  v26+:   Forest Mind

Gate: core version string computed from state.db, not hardcoded

### Forest Version (14.0.0 etc.)

Auto-bumped by `faelight-release` when:
  MAJOR: a core domain restructure, a new Friday pillar, compositor deployed
  MINOR: 3+ tool minor bumps since last release, new tool added
  PATCH: bug fixes, performance, docs

`core version` shows:
  Forest: 14.0.0
  Core binary: 3.2.0
  Intelligence: v21 (Anticipation Layer)
  Tools: 52 deployed
  Friday: 13 patterns, 287 facts, 87% accuracy

### deploy smart-bump

New command: `deploy smart-bump <tool>`
  - Analyzes diff
  - Shows what it would bump and why
  - Confirms with you before writing
  - Writes to Cargo.toml
  - Deploys

`deploy smart-bump --all`
  - Analyzes all changed tools
  - Shows full bump report
  - One confirmation, then deploys all in topo order

### cargo-set-version integration

Already installed: cargo-set-version
Use it to write versions cleanly:
  cargo set-version --manifest-path rust-tools/$tool/Cargo.toml $new_version

### cargo-udeps integration

Already installed: cargo-udeps
Run on pre-deploy to detect unused dependencies:
  cargo udeps --manifest-path rust-tools/$tool/Cargo.toml
  Warn if unused deps found (don't block — some are intentional)

### cargo-upgrade integration

Already installed: cargo-upgrade
Run monthly or on request:
  cargo upgrade --manifest-path rust-tools/$tool/Cargo.toml
  Shows available upgrades, does not auto-apply

---
## Gates

Phase 1 -- Change analysis:
- [x] git diff analyzer reads changes since last deploy tag -- git diff HEAD~10 HEAD per tool dir
- [x] classifier correctly identifies MAJOR/MINOR/PATCH -- pub fn removed=major, new pub fn=minor, impl only=patch
- [x] deploy smart-bump shows classification with explanation -- tested on faelight-fm-v2 2026-05-16
- [x] classification tested -- faelight-fm-v2 PATCH detected correctly, faelight-shell no-change correct

Phase 2 -- Auto version writing:
- [x] cargo-set-version writes version -- python fallback for standalone tools 2026-05-16
- [x] deploy smart-bump writes version and deploys -- tested on faelight-fm-v2 0.2.0→0.2.1
- [x] git tag created -- faelight-fm-v2@0.2.1 confirmed
- [x] Cargo.toml updated correctly -- version written via cargo set-version or python fallback

Phase 3 -- Core intelligence version:
- [x] intelligence version computed from state.db -- v45 Forest Mind from 293 facts + 13 patterns
- [x] milestone names auto-selected -- Awakening/Pattern Recognition/Synthesis Engine/Anticipation/Conversational/Forest Mind
- [x] core --version shows v45 (Forest Mind) -- auto-updated on every deploy core
- [x] version updates on every core deploy -- update-intelligence-version script runs pre-build

Phase 4 -- Forest version automation:
- [x] deploy smart-bump reads git diff history for bump classification
- [x] forest version shown in core version -- reads from domain_state
- [x] core version shows: Forest 13.0.0, Intelligence v45 Forest Mind, Friday 294 facts 13 patterns
- [x] deploy smart-bump handles version writes -- no manual Cargo.toml editing needed

Phase 5 -- Cargo tools integration:
- [ ] cargo-udeps runs on pre-deploy, warns on unused deps
- [ ] cargo-upgrade available via core command for review
- [ ] cargo-set-version used for all version writes
- [ ] cargo-watch available for development hot-reload

Final:
- [ ] Christian never manually edits Cargo.toml versions
- [ ] core version shows coherent picture of entire forest
- [ ] every deploy is versioned and tagged automatically
- [ ] intelligence version reflects actual Friday capability

---
"The forest does not count its own rings by hand.
Growth is measured by what changed,
not by what was declared." 🌲
