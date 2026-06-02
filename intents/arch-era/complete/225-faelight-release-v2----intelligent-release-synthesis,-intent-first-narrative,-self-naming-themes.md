---
id: 225
date: 2026-04-13
type: feature
title: \"faelight-release v2 -- Intelligent Release Synthesis, Intent-First Narrative, Self-Naming Themes\"
status: complete
tags: [feature, rust, faelight]
version: TBD
---
faelight-release v1 records what happened.
faelight-release v2 understands what it means.
v1 lists commits. v2 synthesizes capability.
v1 asks you for a theme. v2 earns its own name.
v1 shows stats. v2 tells a story.
v1 is a changelog generator. v2 is the forest writing its own autobiography.
Every release is a chapter.
Every chapter should read like one.
At the top of every release, before any commit list, before any intent list:
one paragraph. Written by the release tool. Not by you.
The paragraph is synthesized from:
- Completed intents and their descriptions
- Dominant commit categories (what kind of work dominated this release?)
- Velocity data from commit_patterns (how many sessions, what intensity?)
- Health trajectory from health_patterns (was this a stable build or a battle?)
- New tools or capabilities that did not exist before this release
Example output for v11.8.0:
"This release delivers the fsh native execution layer: query, fsearch, patch, edit,
and run builtins that eliminate temp file workflows entirely. Pattern learning now
flows through all four major tools into state.db, giving Friday its first real data
foundation. Shell quoting, heredoc safety, and redirect handling were hardened across
17 targeted fixes. 12 sessions. 89 commits. Health held at 100% throughout."
That paragraph took zero manual effort.
The data was already there.
v1 structure:
  - INT-223: query builtin live
  - INT-223: fsearch builtin live
  - INT-208: gate 3 complete
  ... 20 more lines
v2 structure:
  Delivered: query, fsearch, patch, edit, run builtins. Zero-/tmp workflow verified.
  13 of 15 gates complete. Native pipes deferred to next release.
  Delivered: Pattern learning across faelight-update, faelight-git, fsh, core doctor.
  10 of 13 gates complete. 30-day clock started.
The intent IS the story. Commits are the evidence.
Readers see capability, not implementation detail.
v1: "What is the theme for this release?"
You type something. Maybe it is good. Maybe it is tired.
v2: The release tool analyzes what was built and proposes three theme options:
  Suggested themes for v11.8.0:
  [1] "The Shell Remembers"   — dominant work: fsh intelligence + pattern learning
  [2] "Roots and Signals"     — dominant work: state.db data flows + engine signals
  [3] "The Living Toolkit"    — dominant work: new builtins, zero-/tmp workflow
  Choose (1-3) or type your own:
Theme generation rules:
- Analyze dominant intent categories (shell, intelligence, tools, architecture)
- Generate three distinct options covering different framings of the same work
- Never repeat a theme from CHANGELOG.md history
- If you type your own — check it against history and warn if similar exists
- The chosen theme feeds the welcome message and doctor header as always
Friday inherits this theme logic. When Friday eventually writes release notes,
it will know not just what changed but how to name the chapter.
v1 stats: Health: 100% · Commits: 89 · Tools: 49 deployed · Intents: 168 complete
v2 stats pull from the pattern tables INT-208 built:
  Sessions this release:    12
  Total commits:            89
  Peak velocity:            8.2 commits/hour (session 7)
  Average session health:   99.1%
  Deploys:                  47
  Files changed:            234
  Lines added:              8,847
  Lines removed:            3,201
  Health at release:        100%
  Intents completed:        6
  Gates closed:             31
These numbers come from commit_patterns, health_patterns, deploy_patterns.
No manual counting. No approximation. Exact data from the sessions themselves.
faelight-release v2 continues to own:
- The welcome message shown when fsh starts
- The doctor header version and theme line
- The 00-meta/README.md dynamic section
These update automatically as part of the release ceremony.
The theme chosen (or generated) propagates to all three.
No separate fdocs sync needed for these — release handles them.
bump preview 11.8.0 shows the full generated release note
before writing anything. You read it. You adjust the theme if needed.
You confirm. Then it writes.
The preview is the review.
The review is the approval.
The approval is the release.
The changelog module gains:
- synthesize_narrative() — reads intents + patterns + commits, returns paragraph
- suggest_themes() — returns 3 theme options, checks against history
- load_release_stats_from_db() — reads commit_patterns, health_patterns, deploy_patterns
- group_by_intent() — restructures commits under their parent intent
The TUI gains:
- Theme suggestion screen with 3 options + custom entry
- History check before accepting custom theme
- Narrative preview panel (scrollable)
- Richer stats panel
INT-208 Tool Intelligence L2 — pattern tables in state.db
faelight-release v1 — current base (changelog, TUI, readme, rollback modules)
commit_patterns, health_patterns, deploy_patterns tables populated
Phase 1 — load_release_stats_from_db (reads commit/health/deploy patterns)
Phase 2 — group_by_intent (restructure features/fixes under parent intents)
Phase 3 — synthesize_narrative (one paragraph from data)
Phase 4 — suggest_themes (3 options, history dedup)
Phase 5 — TUI theme selection screen (3 options + custom + history check)
Phase 6 — TUI narrative preview panel
Phase 7 — richer stats display in TUI and CHANGELOG
Phase 8 — welcome message + doctor header still updated correctly
✅ load_release_stats_from_db reads commit_patterns correctly (2026-04-13)
✅ load_release_stats_from_db reads health_patterns correctly (2026-04-13)
✅ load_release_stats_from_db reads deploy_patterns (when available) (2026-04-13)
✅ group_by_intent restructures commits under parent intent IDs (2026-04-13)
✅ completed intents show title + one-line delivery summary (2026-04-13)
✅ synthesize_narrative generates coherent one-paragraph summary (2026-04-13)
✅ narrative reflects dominant work categories accurately (2026-04-13)
✅ suggest_themes generates 3 distinct options (2026-04-13)
✅ suggest_themes never repeats a theme from CHANGELOG history (2026-04-13)
✅ TUI shows theme selection with 3 suggestions -- deferred to future intent
✅ custom theme entry warns if similar to existing theme -- deferred to future intent
✅ richer stats section shows sessions, velocity, deploys, lines (2026-04-13)
✅ welcome message updated with chosen theme (2026-04-13)
✅ doctor header updated with chosen theme (2026-04-13)
✅ 00-meta/README.md dynamic section updated (2026-04-13)
✅ bump preview shows full release note before writing (2026-04-13)
✅ d passes 100% after full implementation (2026-04-13)
"The forest does not need to be told what happened.
It was there.
It felt every commit, every deploy, every session.
faelight-release v2 does not record the past.
It remembers it.
And when the chapter closes,
it writes the title itself." 🌲
