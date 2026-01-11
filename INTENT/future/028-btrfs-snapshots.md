---
id: 028
date: 2026-01-10
type: future
title: "Btrfs Automatic Snapshots"
status: planned
tags: [rust, btrfs, safety, backup]
---

## The Problem
System updates can break things. No LUKS encryption means data protection via snapshots is even more important.

## The Solution
Automatic btrfs snapshots before system updates with easy rollback.

## Implementation
`faelight-snapshot` tool:
- `faelight snapshot create [name]` - Manual snapshot
- `faelight snapshot list` - List all snapshots
- `faelight snapshot rollback <name>` - Restore snapshot
- `faelight snapshot prune` - Remove old snapshots

## Integration
- Hook into `pacu` alias to auto-snapshot before updates
- Keep last 5 snapshots by default
- Store in `/.snapshots` subvolume

## Technical
- Use btrfs subvolume snapshot commands
- Track metadata in `~/.local/state/faelight/snapshots.json`

## Success Criteria
- [ ] Create snapshots before updates
- [ ] List existing snapshots
- [ ] Rollback to previous state
- [ ] Auto-prune old snapshots

---
_The forest remembers its past._ 🌲
