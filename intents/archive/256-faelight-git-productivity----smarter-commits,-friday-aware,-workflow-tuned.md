---
id: 256
date: 2026-04-28
type: arch
title: "faelight-git Productivity -- Smarter Commits, Friday-Aware, Workflow-Tuned"
status: complete
tags: [arch, faelight-git, friday, workflow, productivity, intent, pre-push]
version: TBD
---

## Vision

faelight-git is already a real tool: git2-rs based, Git Risk Score engine,
push-main confirmation gate, signal emission to the forest. This intent
makes it more productive without rewriting it.

The pattern: faelight-git knows what it's seeing (commit content, file
diffs, intent context, push direction). Today it uses that knowledge for
risk scoring. Tomorrow it should use that knowledge for:

- Smart commit message scaffolding (from active intent + diff analysis)
- Workflow shortcuts that match Christian's actual habits
- Friday integration so common patterns become predictions
- Pre-push intelligence beyond just "is this main?"
- Branch hygiene helpers
- Better human-readable signal output

This is companion work to INT-253 (gt-tui). Where gt provides the visual
interface, faelight-git provides the underlying intelligence. Both layered
on top of regular git.

## Why Now

Three observations from sustained daily use:

1. **Commit messages are the most common friction.** INT-245 #12-15 all
   trace back to commit-message authoring. We've documented workarounds
   (single quotes, editor flow, gt-tui) but the deeper question is:
   should faelight-git PROPOSE commit messages from context?

2. **Pattern emergence.** Christian's commits follow stable patterns
   (deploy + commit + push, INT-prefixed messages, intent-aligned scope).
   Friday already records these. faelight-git should consume Friday's
   pattern knowledge to streamline the common case.

3. **The push-main gate proves the model works.** That gate caught real
   mistakes by raising friction at the right moment. More gates of this
   kind (with human-in-the-loop confirmation) would prevent a wider
   class of mistakes without being annoying.

## Approach

### Smart commit scaffolding
- `fg commit` (with no -m) consults active intent ID + diff
- Pre-fills editor with: `INT-XXX: ` subject prefix
- Suggests scope from changed files (e.g. "rust-tools/faelight-shell/src/main.rs"
  -> suggested scope: "fsh: ")
- Offers Friday-recorded message templates from past commits in this area

### Workflow shortcuts
- `fg done` = stage all + commit (editor) + push (with confirm if main)
- `fg sync` = pull + rebase + status report
- `fg blame-here <file:line>` = blame with surrounding history context
- All shortcuts emit signals so Friday tracks the pattern

### Friday integration
- After `fg commit`, faelight-git records: intent_id, scope, timestamp
- Friday correlates commits with deploys, intent gate transitions, health
- Predictions surface inline: "you usually run `deploy core` after this commit"
- Signals fire to faelight-context so other tools react

### Pre-push intelligence (beyond push-main)
- Detect: large file additions (likely accidental)
- Detect: commits without intent prefix when intent is active
- Detect: commits touching `runtime/` (should never be source-controlled)
- Detect: commits during stabilization windows (warn with stabilization context)
- Each detection asks for confirmation, doesn't block

### Branch hygiene
- `fg branches` lists with: ahead/behind main, last commit date, intent ID
- `fg cleanup` removes merged branches with confirmation
- Discourages branch creation during stabilization weeks (info only)

### Better signal output
- Signals carry structured payload (intent_id, scope, file count, magnitude)
- faelight-context can surface signals as Friday-driven nudges
- Reduces "hey something happened" generic notifications

## Hard Dependencies

- Existing faelight-git v3+ as foundation (no rewrites)
- Friday Phase 2 knowledge engine (already shipped)
- Active intent tracking (already in place via core intent focus)
- INT-253 (gt-tui) is parallel work; both reference each other but neither blocks

## Success Criteria

- [ ] `fg commit` with no -m pre-fills INT-XXX prefix from active intent
- [ ] Commit message editor shows suggested scope from changed file paths
- [ ] Friday-recorded patterns surface as message template suggestions
- [ ] `fg done` runs stage + commit + push as one workflow
- [ ] `fg sync` runs pull + rebase + status report
- [ ] Pre-push detects: large file additions, missing intent prefix, runtime/ touches
- [ ] Each pre-push detection asks for confirmation, doesn't auto-block
- [ ] Signal payloads include structured fields (intent_id, scope, magnitude)
- [ ] No regression in existing faelight-git operations

## Scope

### In scope
- Commit message intelligence
- Workflow shortcuts (done, sync)
- Pre-push detection layer
- Friday integration
- Structured signals

### Out of scope
- TUI for git operations (INT-253)
- Cross-repo orchestration
- Custom git protocol
- AI-generated commit messages (no LLM in faelight-git)
- Replace built-in git binary

### Deliberately deferred
- Conflict resolution helpers (large topic, separate intent if needed)
- Interactive rebase wrapper (gt-tui's territory)
- Tag management beyond release tooling

## Gate Check
⬜ Not started

---

*"The forest knows what you've done.
faelight-git should know enough to help you do the next part."* 🌲
