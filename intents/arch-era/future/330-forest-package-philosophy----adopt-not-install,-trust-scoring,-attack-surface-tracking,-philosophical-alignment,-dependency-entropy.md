---
id: 330
title: "Forest Package Philosophy -- adopt not install, trust scoring, attack surface tracking, philosophical alignment, dependency entropy"
status: planned
date: 2026-05-21
tags: [forest, packages, philosophy, trust, security, adoption, pacman, audit, entropy, alignment]
---

INT-330 -- Forest Package Philosophy -- The Forest Chooses Its Tools
date: 2026-05-21

---
THE PREMISE

`pacman -S helix`

This command installs helix.
It does not ask:
  Why does the forest need helix?
  Who maintains helix and do we trust them?
  What attack surface does helix add?
  Does helix align with the forest philosophy?
  What does helix depend on and do we trust those things?
  What happens when helix is no longer maintained?

pacman answers one question: can I download and install this binary?
The forest needs to answer a different question: should I adopt this tool?

Install is a technical act.
Adopt is a philosophical act.

INT-330 builds the forest's answer to that question.
Not a replacement for pacman.
A layer above pacman that makes adoption deliberate.
---
THE VOCABULARY SHIFT

The forest does not install software.
The forest adopts tools.

The difference:

  pacman -S helix
  -- helix is now on the system. Why? Unknown. Trust? Unassessed.

  forest adopt helix --reason "primary editor, replaces vim, kakoune-style"
  -- helix is now adopted by the forest.
  -- Reason recorded. Trust assessed. Attack surface documented.
  -- Friday knows why helix exists.
  -- Rollback plan exists.
  -- Maintenance status tracked.

Adoption is recorded in state.db.
Every adopted tool has a record:
  Why it was adopted
  Who decided (always: christian)
  When it was adopted
  What it replaced (if anything)
  Its current trust score
  Its attack surface score
  Its philosophical alignment score
  Its maintenance health
  Its dependency entropy score
  Whether it is still justified

The forest can ask at any time:
  "Why do we have this tool?"
  "Is it still earning its place?"
  "What would we lose if we removed it?"
---
THE FIVE SCORES

Every adopted tool is scored on five dimensions:

1. TRUST SCORE (0.0 -- 1.0)
   How much does the forest trust this tool?
   Factors:
     Upstream reputation (well-known project vs obscure)
     Maintainer track record
     Audit history (has the forest reviewed the source?)
     Time in use without incident
     CVE history
     License alignment (MIT/Apache preferred)
   Examples:
     helix: 0.92 (well-maintained, clean history, MIT)
     curl: 0.85 (essential but large attack surface)
     unknown-aur-tool: 0.20 (unreviewed, single maintainer)

2. ATTACK SURFACE SCORE (0.0 -- 1.0, lower is better)
   How much exposure does this tool add?
   Factors:
     Network access (does it phone home?)
     File system access (does it read beyond its scope?)
     Privilege requirements (does it need root?)
     Binary size (more code = more surface)
     Dependency count (each dependency is a trust chain)
     Setuid bits
   Examples:
     helix: 0.15 (terminal editor, no network, no privilege)
     brave: 0.75 (browser, network, large codebase)
     sudo: 0.60 (privilege escalation by design)

3. PHILOSOPHICAL ALIGNMENT SCORE (0.0 -- 1.0)
   Does this tool think the way the forest thinks?
   Factors:
     Written in Rust? (bonus)
     Respects user privacy?
     Minimal dependencies?
     Explicit over implicit?
     Source available and readable?
     Does it follow the forest's "understand over convenience" principle?
   Examples:
     helix: 0.90 (Rust, explicit config, no telemetry)
     atuin: 0.85 (Rust, privacy-respecting, open source)
     brave: 0.50 (good privacy but large, closed components)
     vscode: 0.30 (Microsoft telemetry, heavy, proprietary extensions)

4. MAINTENANCE HEALTH SCORE (0.0 -- 1.0)
   Is this tool being actively cared for?
   Factors:
     Last commit date
     Open issue count and response time
     Release cadence
     Bus factor (one maintainer = risk)
     Upstream dependency health
     Is it in Arch official repos? (signal of health)
   Examples:
     helix: 0.90 (active, multiple maintainers, regular releases)
     atty: 0.10 (unmaintained, RUSTSEC warning)
     zsh: 0.75 (stable, long-maintained, slow cadence)

5. DEPENDENCY ENTROPY SCORE (0.0 -- 1.0, lower is better)
   How much chaos does this tool's dependency tree add?
   Factors:
     Direct dependency count
     Transitive dependency count
     Duplicate dependencies (same crate, different versions)
     Known vulnerable dependencies
     Dependency churn rate
   Examples:
     helix: 0.25 (reasonable dep tree, well-managed)
     electron apps: 0.95 (enormous, chaotic, unmaintainable)
     curl: 0.30 (C library, few deps, well-audited)

COMPOSITE SCORE:
  adoption_score = (trust * 0.30) + (1 - attack_surface) * 0.25
                 + alignment * 0.25 + maintenance * 0.15
                 + (1 - entropy) * 0.05

  Score >= 0.75: Confidently adopted
  Score 0.60-0.74: Adopted with caveats (document concerns)
  Score 0.40-0.59: Provisional adoption (review quarterly)
  Score < 0.40: Do not adopt (or adopt only with explicit exception)
---
THE ADOPTION RECORD

Every tool in the forest has an entry in the adopted_tools table in state.db:

  CREATE TABLE adopted_tools (
      id INTEGER PRIMARY KEY,
      name TEXT NOT NULL,
      pacman_name TEXT,           -- actual pacman package name
      adopted_at INTEGER,         -- unix timestamp
      reason TEXT NOT NULL,       -- WHY we have this
      replaces TEXT,              -- what it replaced (if anything)
      trust_score REAL,
      attack_surface REAL,
      alignment_score REAL,
      maintenance_score REAL,
      entropy_score REAL,
      composite_score REAL,
      last_reviewed INTEGER,      -- when scores were last updated
      status TEXT DEFAULT 'active', -- active/provisional/deprecated/removed
      notes TEXT,
      friday_assessment TEXT,     -- Friday's notes on this tool
  );

Current forest tools that WOULD have adoption records:
  helix -- editor, trust 0.92, alignment 0.90
  atuin -- shell history, trust 0.88, alignment 0.85
  niri -- compositor, trust 0.85, alignment 0.88
  fzf -- fuzzy finder, trust 0.90, alignment 0.80
  ripgrep -- search, trust 0.95, alignment 0.92
  bat -- file viewer, trust 0.90, alignment 0.88
  brave -- browser, trust 0.70, alignment 0.50
  zsh -- shell (temporary), trust 0.90, alignment 0.60
  sqlite3 -- database, trust 0.95, alignment 0.85
---
THE ADOPT COMMAND

  forest adopt <tool> [options]

  forest adopt helix
    -- Interactive adoption wizard
    -- Asks for reason
    -- Runs automated scoring
    -- Shows composite score
    -- Records to state.db

  forest adopt helix --reason "primary editor" --replaces vim
    -- Non-interactive
    -- Reason and replacement recorded

  forest adopt helix --audit
    -- Downloads source, runs cargo audit if Rust
    -- Checks CVE databases
    -- Reviews dependency tree
    -- Updates trust score based on findings

  forest unadopt helix --reason "replacing with forest-native editor"
    -- Records removal reason
    -- Checks what depends on helix
    -- Removes from adopted_tools (marks as removed, never deletes)

  forest tools
    -- Lists all adopted tools with scores
    -- Highlights tools with low scores
    -- Shows tools pending review

  forest tools --audit
    -- Checks maintenance health of all tools
    -- Flags unmaintained tools (RUSTSEC warnings, no commits in 1 year)
    -- Friday generates weekly digest

  forest tools --justify
    -- For each tool: is it still earning its place?
    -- Tools not used in 30 days: flagged for review
    -- Redundant tools (two tools with same purpose): flagged
---
PHILOSOPHICAL ALIGNMENT PRINCIPLES

The forest has 7 alignment principles for tool adoption.
A tool that violates any principle requires explicit exception documentation.

1. UNDERSTANDS OVER CONVENIENCE
   The tool should be understandable. Its behavior should be predictable.
   Violation: tools that "just work" through magic we cannot inspect.

2. EXPLICIT OVER IMPLICIT
   The tool should do what you tell it. No hidden behavior.
   Violation: tools that phone home, auto-update, collect telemetry.

3. RUST PREFERRED
   Rust tools get a trust and alignment bonus.
   Memory safety, explicit error handling, no hidden allocations.
   Exception allowed for: essential system tools (curl, git, sqlite)

4. MINIMAL SURFACE
   The tool should do one thing well.
   Violation: tools that bundle unrelated features, large dependency trees.

5. USER SOVEREIGN
   The tool respects the user's data and decisions.
   Violation: tools that require accounts, cloud sync, or analytics.

6. RECOVERABLE
   The tool's effects should be reversible or at least visible.
   Violation: tools that make silent changes to system state.

7. FOREST NATIVE FIRST
   If the forest can build it, it should.
   External tool adoption is a last resort, not a first instinct.
   Violation: adopting an external tool when a forest-native solution exists.
---
DEPENDENCY ENTROPY IN PRACTICE

The cargo audit output we see on every deploy is dependency entropy made visible.

Currently flagged:
  aws-lc-sys: 5 CVEs -- in faelight-browser via reqwest via rustls
  users 0.10.0: unmaintained -- in faelight-lock via pam
  atty 0.2.14: unmaintained -- in faelight-palette
  drm 0.14.2: yanked -- in faelight-compositor

These are not just security warnings.
They are adoption failures at the dependency level.

INT-330 makes this visible at the tool level:
  faelight-browser has entropy score 0.75 (high) because of aws-lc-sys chain
  faelight-lock has maintenance score 0.40 (low) because of users 0.10.0
  These scores drive the quarterly review

The forest should eventually be able to say:
  "faelight-browser's entropy score has increased 0.15 this quarter.
   Three of its transitive dependencies have new CVEs.
   Consider: remove browser from forest, use system browser instead."

That is a genuinely novel insight.
No package manager today reasons this way.
---
INTEGRATION WITH FRIDAY

Friday monitors the adoption database as part of weekly synthesis:

  -- Weekly: check maintenance health of all adopted tools
  SELECT name, maintenance_score, last_reviewed FROM adopted_tools
  WHERE status = 'active' AND maintenance_score < 0.60
  ORDER BY maintenance_score ASC;

  -- Monthly: flag tools not used in 30 days
  -- Quarterly: full adoption review
  -- On CVE: immediate alert for affected tools

Friday surfaces:
  "atty is unmaintained and has a soundness issue. faelight-palette depends on it.
   Consider: replace atty with std::io::IsTerminal (stable since Rust 1.70)."

  "3 tools have not been used in 30 days: faelight-gen, faelight-forecast, faelight-diff.
   Are they still earning their place in the forest?"

  "faelight-browser's composite adoption score dropped from 0.65 to 0.58 this month.
   It is now in provisional adoption territory. Review recommended."
---
PHASES

Phase 0 -- Audit existing tools:
  Create adopted_tools table in state.db
  Manually score all current forest tools
  Identify the worst offenders (low scores)
  Gate: all forest tools have adoption records with scores

Phase 1 -- forest tools command:
  List adopted tools with scores
  Flag provisional and deprecated tools
  Show tools pending review
  Gate: forest tools shows all tools with composite scores

Phase 2 -- forest adopt command:
  Interactive adoption wizard
  Automated scoring (maintenance check, CVE check, dep count)
  Record to state.db
  Gate: adopt 3 new tools through the wizard, all recorded correctly

Phase 3 -- Friday integration:
  Weekly maintenance health check
  Monthly usage review
  Quarterly full audit
  CVE alert on new findings
  Gate: Friday surfaces one genuine adoption concern from weekly check

Phase 4 -- Dependency entropy tracking:
  cargo audit output parsed and stored per tool
  Entropy score updated on every deploy
  Trends tracked over time
  Gate: entropy score changes visible in forest tools --trend

Phase 5 -- Adoption discipline (ongoing):
  No new tool adopted without forest adopt
  No tool removed without forest unadopt
  Quarterly review run as a forest ritual
  Gate: 3 months of disciplined adoption tracking
---
GATES
[ ] adopted_tools table created in state.db
[ ] All current forest tools scored and recorded
[ ] forest tools command shows all tools with composite scores
[ ] forest adopt wizard works for new tool adoption
[ ] forest unadopt records removal with reason
[ ] Friday weekly maintenance check surfaces real concerns
[ ] Dependency entropy scores update on deploy
[ ] 3 months of disciplined adoption tracking

DEPENDS ON
INT-329 (typed pipes) -- forest tools uses typed query pipeline
INT-327 (self-healing) -- adoption scores feed into service health
INT-261 (fsh vocabulary) -- adopt/unadopt as forest vocabulary
Friday (active) -- weekly synthesis includes adoption health

TIMELINE
Phase 0: any session (just database + scoring work)
Phase 1-2: 2-3 sessions
Phase 3: after Friday synthesis improvements
Phase 4-5: ongoing, no hard deadline
Full adoption discipline: before NY presentation

"pacman installs.
The forest adopts.
The difference is intention.
Every tool in the forest earned its place.
Every tool can justify its existence.
Every tool can be removed without mystery.
The forest is not a collection of software.
It is a set of deliberate decisions." 🌲
