---
id: 150
date: 2026-03-25
type: future
title: "Docs Audit — The Forest Documents Itself Accurately"
status: complete
tags: [docs, audit, cleanup, readme, accuracy, v12]
version: 12.0.0
priority: low
depends_on: [145]
---

## The Problem

The forest has grown faster than its documentation.
Documents exist that are stale, duplicate, or no longer relevant.
The README was recently rewritten but other docs remain untouched.

A forest that values understanding over convenience
must keep its own records honest.

## What Needs Auditing

### 00-meta/
- `CHANGELOG.md` — auto-generated, accurate ✅
- `VERSION` — auto-updated, accurate ✅
- `README.md` — recently rewritten ✅
- `TOOLS.md` — may be stale, needs review

### intents/
- Complete intents — all accurate by definition ✅
- Future intents — some may be superseded or merged
- INT-142 vs INT-147 naming collision — needs resolution

### 03-interfaces/
- Any stale config documentation
- Shell guide — does not exist yet (needed)

### Root docs
- `docs/` directory — contents unknown, needs inventory

## The Shell Guide (Priority)

faelight-shell is now capable enough to deserve its own guide.
Written for a human who has never seen it before.
```
docs/faelight-shell-guide.md
  What is faelight-shell?
  Core concepts (structured data, pipelines, forest awareness)
  Command reference
  Pipeline examples
  Config file (config.fsh)
  Aliases and shortcuts
  Background jobs
  Shell variables
  Pipes to external commands
  Redirection
  NL queries (?prefix)
  Scripting (.fsh files)
```

## Audit Process
```
1. Inventory every document in the repository
2. Classify: accurate / stale / duplicate / missing
3. Update stale documents
4. Remove duplicates
5. Create missing documents (shell guide first)
6. Update faelight-docs to track doc health
```

## Gate Check
```
✅ TOOLS.md updated (2026-03-26) — version header corrected to v11.3.0
✅ docs/ inventoried (2026-03-26) — 15 docs, headers updated to v11.3.0
✅ Naming resolved (2026-03-26) — 142=voice-input (Whisper), 147=voice-output (Piper TTS)
✅ faelight-shell guide written (2026-03-26) — docs/FAELIGHT-SHELL-GUIDE.md, 198 lines
✅ Stale docs updated (2026-03-26) — FAELIGHT-SHELL.md, TOOL_REFERENCE.md headers corrected
✅ faelight-docs sync run — README and welcome accurate
```

## The Phrase

**"A forest whose map is wrong
leads travellers astray.
Accuracy is not vanity —
it is respect for those who follow."**

---
*"Document what exists. Not what you wish existed."* 🌲
