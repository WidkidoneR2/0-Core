---
id: 121
date: 2026-03-12
type: future
title: "faelight-readme — auto-update README dynamic sections on release"
status: complete
tags: [readme, release, automation, github, docs, v10.8]
version: 10.8.0
priority: medium
---

## Vision

faelight-release currently updates the changelog but the
README static section requires manual updates.

A `faelight-readme` tool or `core readme update` command
that regenerates dynamic README sections automatically
on every release.

## What Gets Auto-Updated

- Version badge and header
- Latest release section (from CHANGELOG)
- Tool count (from registry)
- Commit count (from git)
- Health percentage (from cache)
- Intent stats (from ledger)
- Quick reference commands (from aliases)

## What Stays Manual

- Philosophy section
- Architecture diagrams
- Journey table (curated milestones)
- Acknowledgments

## Integration

faelight-release publish → calls faelight-readme update
                        → commits README changes
                        → pushes to GitHub

## Success Criteria

- [ ] `core readme update` command
- [ ] Auto-updates version, tool count, commit count
- [ ] Called automatically by faelight-release publish
- [ ] Manual sections protected from overwrite
- [ ] Zero double-v version prefix issues

---
*"The forest should document itself."* 🌲
