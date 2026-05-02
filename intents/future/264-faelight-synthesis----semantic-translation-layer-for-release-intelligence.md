---
id: 264
date: 2026-05-02
type: arch
title: \"faelight-synthesis -- Semantic Translation Layer for Release Intelligence\"
status: planned
tags: [architecture, rust, design, release, docs, translation, public-voice, faelight-release, faelight-docs]
supersedes: INT-250
version: TBD
---

## Vision
faelight-release today produces technically correct changelogs that are
spiritually wrong. The v11.9.0 changelog reads:
  INT-217 Friday Phase 1 -- Friday finds her voice (repeated 6 times)
  INT-241 fixes (2 commits)
  48 commits total
This is ledger truth presented as human meaning. They are not the same
thing. The INT number is an audit unit -- it belongs in debugging logs and
internal tooling, never in a public changelog. "48 commits" is a process
metric, not a signal of value. "2 fix commits" erases the actual meaning
of what was fixed.
faelight-synthesis is the missing layer between ledger truth and human
meaning. It sits between faelight-release (which counts and groups) and
faelight-docs (which renders), and performs semantic translation: turning
what the system knows internally into what a human should understand
externally.
The architecture becomes:
  INTENTS + COMMITS
      ↓
  faelight-release  (structural truth -- counts, groups, labels)
      ↓
  faelight-synthesis  (semantic translation -- meaning, not metadata)
      ↓
  faelight-docs  (rendering -- formats for README, CHANGELOG, welcome)
      ↓
  PUBLIC OUTPUT
What v12.0.0 should look like when this exists:
  12.0.0 -- The Forest Speaks Human
  The forest gained a voice this release -- not just Friday's voice,
  but the shell's own vocabulary. Commands now speak English first,
  UNIX as fallback.
  What changed:
  - The terminal can now copy across its entire scrollback history
  - The shell introduced its first human words: delete, find, db
  - Git workflow moved into a visual TUI -- stage, diff, commit, push
  - Health is now a keypress away: Ctrl+D shows system status at a glance
  - The shell can now query its own memory natively
  Theme: Human first. UNIX as fallback. The forest speaks.
No INT numbers. No commit counts. Meaning, not structure.
1. v12.0.0 is the first release where the theme should emerge from what
   actually shipped, not from a template pool. The vocabulary words
   (delete, find, db), the TUIs (gt, health, history), the terminal fixes
   -- these tell a coherent story. faelight-synthesis extracts that story.
2. INT-250 (release tool intelligence) originally scoped this as "smarter
   release tooling." The real insight from reviewing v11.9.0 is that the
   problem is not formatting -- it is translation. faelight-synthesis is
   what INT-250 was pointing toward.
3. The three-layer separation (structural / semantic / rendering) prevents
   the coupling bug where INT-NNN is accidentally used as a narrative unit.
   Once the layers are clean, each one can evolve independently.
- NOT an LLM. No generative text. No API calls. Fully local, fully
  deterministic, fully auditable.
- NOT a formatting tool. faelight-docs handles formatting.
- NOT a changelog generator. It produces structured meaning that
  faelight-docs then renders into whatever format is needed.
A weighted signal extraction and compression engine that:
1. Takes as input: completed intents in release range, commit metadata,
   intent tags, dependency graph, changed file domains.
2. Produces as output: structured semantic summary:
  {
    "theme": "The forest learned to speak human",
    "headline": "Commands now speak English first, UNIX as fallback",
    "changes": [
      {
        "domain": "shell",
        "impact": "high",
        "summary": "The shell gained human-readable vocabulary: delete, find, db"
      },
      {
        "domain": "terminal",
        "impact": "high",
        "summary": "The terminal can now copy text across its entire scrollback"
      }
    ],
    "fix_meaning": "Terminal rendering, shell parsing, and database connection stability improved",
    "philosophy_shift": "Forest vocabulary principle: human words first, UNIX as fallback"
  }
3. faelight-docs then renders this into README, CHANGELOG, welcome message,
   health headline -- each format gets the semantic truth, not the ledger.
**1. Intent → Human meaning**
Each intent title becomes a one-sentence human description. The mapping
is rule-based, not generative:
- Tags drive domain classification (shell, terminal, friday, infra, etc.)
- Title + description are compressed to one human-readable sentence
- INT number is dropped from all external output
**2. Commit count → Work significance**
Instead of "48 commits" or "2 fix commits":
  fix_weight = issues_resolved + subsystems_impacted + severity_corrected
A single commit that fixes a 6-month database connection leak scores
higher than 10 commits of formatting changes. The weighting is defined
in a config table, not hardcoded.
**3. Philosophical weighting**
Some changes outweigh their commit count:
- A vocabulary word added (delete, find) changes how the system is used
- A philosophy document updated changes how the system is understood
- A TUI shipped changes how the human interacts with the system daily
These are detected from tags: "vocabulary", "philosophy", "ux", "tui"
all trigger high-impact classification regardless of commit count.
**4. Theme generation**
The theme is NOT selected from a template pool. It is derived from the
highest-impact changes in the release:
- Identify top 3 domains by weighted impact
- Find the connecting narrative (what do these changes share?)
- Express as a one-sentence theme
For v12.0.0: vocabulary + terminal + TUIs = "the forest speaks human"
Rust module in rust-tools/faelight-synthesis/ or embedded in
faelight-release as a synthesis subcommand.
Input: intent ledger range (start..end commit), state.db
Output: structured JSON that faelight-docs consumes
The weighting table lives in a config file (not hardcoded) so Christian
can tune it without recompiling.
- faelight-release (provides structural input)
- faelight-docs (consumes semantic output)
- state.db intent ledger (source of truth)
- Intent tag taxonomy (tags drive domain classification)
- [ ] faelight-synthesis produces structured JSON from intent range
- [ ] INT numbers absent from all synthesis output
- [ ] Theme derived from weighted signals, not template pool
- [ ] fix_meaning describes what was fixed, not how many commits
- [ ] Philosophical changes (vocabulary, TUI, philosophy docs) weighted high
- [ ] faelight-docs consumes synthesis output for CHANGELOG, README, welcome
- [ ] v12.0.0 changelog reads as human meaning, not ledger metadata
- [ ] Weighting table in config file, tunable without recompile
- [ ] Fully local -- no LLM, no API calls, fully deterministic
- [ ] No regression in existing faelight-release structural output
- The synthesis engine and its four transformations
- Structured JSON output schema
- Integration with faelight-docs rendering
- Weighting config table
- v12.0.0 as the first release produced by this system
- Replacing faelight-release (it still produces structural truth)
- LLM or generative text of any kind
- Cross-project synthesis (single repo only)
- Automatic publishing to external channels
- Voice readout of release notes (Friday voice, separate intent)
- Multi-language output (English only in v1)
- Historical reanalysis of past releases (forward-only in v1)
⬜ Not started
---
*"INT-NNN is an audit unit.
It belongs in debugging logs and internal tooling.
It should never appear in a public changelog.
The gap between ledger truth and human meaning
is exactly the gap faelight-synthesis exists to close." 🌲*
