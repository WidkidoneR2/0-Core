# INT-221 — faelight-git v4 — Smarter Commits, Auto-Intent Detection, Rollback Command
Status: [planned]
Date: 2026-04-10
Tags: faelight-git, commits, rollback, intelligence, auto-intent, diff-summary, v4
faelight-git v3 is a governance layer. It stages, verifies, commits, pushes.
It asks you which intent this commit belongs to — every single time.
It computes nothing about the commit itself.
It cannot roll back.
It does not warn you when committing during risky conditions.
v4 makes the commit itself intelligent.
The most repeated friction in every session:
"Intent reference (INT-0XX or skip):"
v4 eliminates this prompt in most cases.
How it works:
1. Read active intents from state.db (cistart records)
2. If exactly one intent is active — auto-attach silently
3. If multiple intents active — show ranked suggestions based on:
   - Which files changed match which intent directory
   - Which intent was most recently worked on (last cistart)
   - Which intent has the most commits already
4. If no intent active — ask as before
fg commit
→ Auto-detected: INT-208 (only active intent)
→ Commit message:
No prompt. No friction. The forest already knows.
Every commit records a structured diff summary:
files_changed:   3
lines_added:     47
lines_removed:   12
domains_touched: ["faelight-shell", "engine"]
test_files:      0
Stored in commit_patterns (already exists from INT-208).
Friday uses this to understand: what kind of work produces what kind of commits.
Before committing, v4 checks:
**High velocity warning:**
⚠️  You have committed 8 times in the last hour
High velocity sessions have a 23% higher rollback rate
Proceed? (y/n)
**Low health warning:**
⚠️  Health is at 95% — uncommitted changes in flight
Consider running d before committing
Proceed anyway? (y/n)
**Large change warning:**
⚠️  This commit touches 847 lines across 12 files
Consider splitting into smaller commits
Proceed? (y/n)
These are warnings, not blocks. You always decide.
Current state: to rollback you manually find the commit hash and git reset.
That is too slow and too risky.
v4 adds:
fg rollback                    — interactive rollback picker
fg rollback --list             — show last 10 commits with risk scores
fg rollback <hash>             — rollback to specific commit
fg rollback --dry-run          — show exactly what would change
fg rollback --intent 208       — rollback all commits from INT-208
Rollback picker shows:
Recent commits:
[1] abc1234  INT-208: gate 3 complete            2m ago   LOW risk
[2] def5678  fsh: fix echo quote stripping        8m ago   MEDIUM risk
[3] ghi9012  INT-208: gate 1 complete             1h ago   LOW risk
Select commit to restore to (1-3) or Ctrl+C to cancel:
Risk score is computed from:
- How many files changed
- Whether any deploys happened after this commit
- Whether health dropped after this commit
v4 makes push smarter:
fg push                        — push with pre-push health check
fg push --force-with-lease     — safe force push
fg push --dry-run              — show what would be pushed
Pre-push health check:
- Health must be >= 95% to push
- No uncommitted changes
- If pushing to main — shows summary of all commits being pushed
v4 tracks velocity per session and warns:
📊 Session velocity: 12 commits/hour (unusually high)
Last 3 high-velocity sessions had 2 rollbacks
Consider a brief review before pushing
This data comes from commit_patterns (INT-208).
The warning threshold is learned — not hardcoded.
INT-208 Tool Intelligence L2 — commit_patterns table
faelight-git v3.3.1 — current base
state.db commit_patterns with velocity_per_hour and session_depth
Phase 1 — Auto-intent detection (single active intent)
Phase 2 — Diff summary on every commit
Phase 3 — Risk assessment warnings (velocity, health, size)
Phase 4 — fg rollback interactive picker
Phase 5 — fg rollback --intent (rollback all commits from intent)
Phase 6 — fg push intelligence with health gate
Phase 7 — Multi-intent auto-detection with ranking
⬜ auto-intent detection — single active intent auto-attached
⬜ auto-intent detection — multi-intent ranked suggestion
⬜ diff summary recorded on every commit (files, lines, domains)
⬜ high velocity warning fires at threshold
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
v4 is not stricter.
It is smarter.
The forest remembers every commit you have ever made.
It uses that memory to protect the next one." 🌲
