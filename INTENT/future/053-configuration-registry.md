---
id: 053
date: 2026-01-14
type: future
title: "Configuration Registry - Know What Changed"
status: planned
tags: [v8.0, infrastructure, config, tracking]
version: 8.0.0
---

## Vision
Central registry of all configs with checksums, ownership, and change tracking.

**Philosophy:** "If you can't track it, you can't recover it."

## The Registry

### Structure
```json
{
  "configs": [
    {
      "path": "~/.config/sway/config",
      "owner": "sway",
      "stow_package": "sway",
      "intent": "035",
      "checksum": "sha256:...",
      "last_modified": "2026-01-14T10:30:00Z",
      "last_scanned": "2026-01-14T14:00:00Z"
    }
  ]
}
```

### Commands
```bash
config-registry scan          # Rebuild registry
config-registry diff          # What changed?
config-registry blame FILE    # Which intent modified?
config-registry deps FILE     # What depends on this?
config-registry history FILE  # Change timeline
```

## Features

### 1. Drift Detection
```bash
$ config-registry diff

Changed since last scan:
  ~/.config/sway/config (modified 2h ago)
    - Intent 050 expected
    - Manual edit detected

New configs:
  ~/.config/faelight/new.toml (not tracked)
```

### 2. Intent Attribution
```bash
$ config-registry blame ~/.config/sway/config

Created by: Intent 035 (Sway Migration)
Modified by:
  - Intent 041 (Menu keybind) - 2026-01-10
  - Intent 044 (Keybind conflicts) - 2026-01-12
  - Manual edit - 2026-01-14
```

### 3. Dependency Tracking
```bash
$ config-registry deps ~/.config/sway/config

Depends on:
  - faelight-bar (workspace display)
  - faelight-launcher (Super+Space)
  - faelight-menu (Super+Shift+E)

Used by:
  - scripts/sway-reload
  - doctor (health check)
```

## Success Criteria
- [ ] CLI tool: `config-registry`
- [ ] Storage: `~/.local/state/0-core/config-registry.json`
- [ ] Detects drift automatically
- [ ] Tracks intent attribution
- [ ] Shows dependencies
- [ ] Integration with doctor
- [ ] Integration with entropy-check

## Timeline
**v8.0.0**

---

_"Know what changed. Know why. Know when."_ 🌲
