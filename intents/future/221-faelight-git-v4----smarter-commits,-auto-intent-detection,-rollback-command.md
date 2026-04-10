---
id: 221
date: 2026-04-10
type: feature
title: \"faelight-git v4 -- Smarter Commits, Auto-Intent Detection, Rollback Command\"
status: planned
tags: [feature, rust, faelight]
version: TBD
---
faelight-git v3 is a governance layer. It stages, verifies, commits, pushes.
It asks you which intent this commit belongs to every single time.
It computes nothing about the commit itself.
It cannot roll back.
It does not warn you when committing during risky conditions.
v4 makes the commit itself intelligent.
The most repeated friction in every session: "Intent reference (INT-0XX or skip):"
v4 eliminates this prompt in most cases.
1. Read active intents from state.db (cistart records)
2. If exactly one intent is active -- auto-attach silently
3. If multiple intents active -- show ranked suggestions based on files changed
4. If no intent active -- ask as before
Every commit records a structured diff summary in commit_patterns:
files_changed, lines_added, lines_removed, domains_touched.
Friday uses this to understand what kind of work produces what kind of commits.
High velocity warning: fires when 8+ commits in last hour (23% higher rollback rate).
Low health warning: fires when health < 95%.
Large change warning: fires when touching 800+ lines across 10+ files.
These are warnings, not blocks. You always decide.
Current state: rollback is manual and painful.
fg rollback                 -- interactive rollback picker (last 10 commits with risk scores)
fg rollback faelight-shell  -- rollback specific tool to previous version
fg rollback --dry-run       -- show exactly what would change
fg rollback --intent 208    -- rollback all commits from INT-208
Risk score per commit: files changed + deploys after + health drops after.
fg push              -- push with pre-push health check (>= 95% required)
fg push --dry-run    -- show commits being pushed
Velocity warnings are learned from commit_patterns, not hardcoded.
High velocity sessions with history of rollbacks generate stronger warnings.
INT-208 Tool Intelligence L2 -- commit_patterns table with velocity_per_hour
faelight-git v3.3.1 -- current base
Phase 1 -- Auto-intent detection (single active intent)
Phase 2 -- Diff summary on every commit
Phase 3 -- Risk assessment warnings (velocity, health, size)
Phase 4 -- fg rollback interactive picker
Phase 5 -- fg rollback --intent (rollback all commits from intent)
Phase 6 -- fg push intelligence with health gate
Phase 7 -- Multi-intent auto-detection with ranking
⬜ auto-intent detection -- single active intent auto-attached
⬜ auto-intent detection -- multi-intent ranked suggestion
⬜ diff summary recorded on every commit (files, lines, domains)
⬜ high velocity warning fires at learned threshold
⬜ low health warning fires when health < 95%
⬜ large change warning fires at threshold
⬜ fg rollback --list shows last 10 commits with risk scores
⬜ fg rollback interactive picker works
⬜ fg rollback --dry-run shows changes without executing
⬜ fg rollback --intent rolls back all commits from an intent
⬜ fg push pre-push health gate (>= 95% required)
⬜ fg push --dry-run shows commits being pushed
⬜ commit velocity warning learned from commit_patterns
⬜ d passes 100% after full implementation
"A commit that knows nothing about itself
cannot tell you when you are about to make a mistake.
v4 is not stricter. It is smarter.
The forest remembers every commit you have ever made.
It uses that memory to protect the next one." 🌲