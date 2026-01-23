# 🌲 faelight-fetch

Canonical system information for Faelight Forest.

## Overview

A minimal system information tool that displays exactly what matters for 0-Core:
- System version
- Profile state  
- Core lock status (using lsattr to check immutable flag)
- Health percentage (from dot-doctor)
- Essential system info (WM, terminal, shell, kernel, uptime, host)

## Philosophy

No ASCII art. No bloat. No configuration. Just facts.

## Output Format
```
🌲 Faelight Forest v8.0.0
────────────────────────
profile   DEF
core      🔓 unlocked
health    🟢 92%
wm        sway
term      foot
shell     zsh
kernel    6.18.6-arch1-1
uptime    33m
host      fealight
```

## Usage
```bash
faelight-fetch
```

## Health Icons

- 🟢 100% or ≥90%
- 🟡 70-89%
- 🔴 <70%

## Core States

- 🔓 Unlocked (no immutable attribute)
- 🔒 Locked (chattr +i on /home/christian/0-core)

## Dependencies

- `dot-doctor` - For health checks
- `lsattr` - For checking core lock status
- `sysinfo` - For system information (Rust crate)

## Implementation

- **Lines of code:** ~150
- **Build time:** ~2.5 hours from concept to working
- **Binary size:** 560KB
- **No external config needed**

## Replaces

Designed to replace fastfetch as a cleaner, 0-Core-aware alternative.

## Version

1.0.0 - Initial release
