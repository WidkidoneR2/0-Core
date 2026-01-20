# 📅 latest-update

Find the most recently updated package in 0-Core.

## Usage
```bash
# Show latest updated package
latest-update
# Output: 📦 wm-sway v1.2.0 (1 weeks ago)

# Show all packages (sorted by update date)
latest-update --all

# Health check
latest-update --health-check
```

## Features

- Scans `~/0-core/stow/` packages
- Human-readable time formatting
- Sorted by most recent first
- Shows package name, version, and update time

## Version

v2.0.0 - Stow-aware package scanner
