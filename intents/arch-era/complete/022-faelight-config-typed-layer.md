---
id: 022
date: 2026-01-10
type: future
title: "faelight-config - Typed Configuration Layer"
status: complete
tags: [rust, config, serde, v6.4.0]
---
## The Vision
Move all configuration to typed TOML. Configs are read-only inputs, not procedural scripts.

## Why
- Kill magic env vars
- Kill undocumented defaults
- Kill drift
- Rust enforces schema validity

## Directory Structure
```
~/.config/faelight/
├── config.toml      # Global settings
├── profiles.toml    # Profile definitions
├── themes.toml      # Color schemes
└── versions.toml    # Tool versions
```

## Example: profiles.toml
```toml
[profile.default]
vpn = true
cpu = "balanced"
notifications = "all"

[profile.gaming]
vpn = false
cpu = "performance"
gpu = "max"
notifications = "minimal"

[profile.work]
vpn = true
cpu = "balanced"
notifications = "focused"
apps = ["slack", "tutanota"]
```

## Technical
- `serde` + schema structs
- Refuse to apply invalid state
- Schema versioning built-in
- Migration helpers for schema changes

## Success Criteria
- [ ] All profiles defined in TOML
- [ ] All themes defined in TOML
- [ ] Rust structs enforce schema
- [ ] `faelight config validate`
- [ ] `faelight config migrate`

---
_Configuration as data, not code._ 🌲
