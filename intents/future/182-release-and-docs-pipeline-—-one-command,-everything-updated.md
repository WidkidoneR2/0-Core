---
id: 182
date: 2026-03-30
type: feature
title: "Release and Docs Pipeline — One Command, Everything Updated"
status: in-progress
tags: [release, docs, automation, faelight-release, faelight-docs, github, readme]
version: 11.6.0
---

## The Problem
Currently releasing Faelight Forest requires:
1. faelight-release publish VERSION --theme "THEME"
2. faelight-docs sync
3. fg sync (or fg commit)
4. Manual README link verification
5. Manual changelog verification
6. /etc/faelight/ manual sync

That is too many steps. Too many places where something can go wrong.
The GitHub README changelog link was broken for an entire release cycle.
That should never happen.

## The Vision
One command. Everything correct.
```
faelight-release publish 11.6.0 --theme "The Forest Grows"
→ ✅ Version bumped — VERSION, .zshrc, README
→ ✅ Changelog generated and inserted
→ ✅ README links verified — all paths correct
→ ✅ /etc/faelight/ synced
→ ✅ Docs synced — faelight-docs sync called automatically
→ ✅ Commit created — "release: Faelight Forest 11.6.0"
→ ✅ Pushed to origin
Done. Nothing manual. Nothing forgotten.
```

## What Needs to Change

### faelight-release improvements
- Auto-call faelight-docs sync after publish (already partially done)
- Verify all README links before publishing — fail loudly if broken
- Auto-sync /etc/faelight/ VERSION and COMMITS
- Generate a structured release summary in docs/

### faelight-docs improvements  
- Know which sections it owns vs faelight-release owns
- Never overwrite faelight-release sections
- Update COMMAND-GUIDE.md on every release automatically
- Verify its own output — no broken links, no stale data

### Single Release Commit
Instead of: release commit + docs commit + sync commit
Just one: "release: Faelight Forest VERSION — THEME"
Everything staged and committed in one shot.

### Link Verification
Before any release commit:
- Scan README for all markdown links
- Verify each link resolves correctly
- Block release if any link is broken
- Report exactly which links need fixing

## Gate Check
```
⬜ faelight-release publish does full pipeline in one command
⬜ README link verification — blocks release on broken links
⬜ /etc/faelight/ auto-synced on every release
⬜ faelight-docs sync called automatically — no manual step
⬜ COMMAND-GUIDE.md updated on every release
⬜ Single release commit — no manual fg sync needed
⬜ Release summary generated in docs/RELEASE-NOTES.md
⬜ Zero manual steps after faelight-release publish
```

## The Phrase
**"A release pipeline that requires manual steps
is a release pipeline that will fail.
One command. Everything correct.
The forest releases itself."**

---
*"If you have to remember to do it,
automate it."* 🌲
