# Changelog - latest-update

## [4.0.0] - 2026-02-11

### 🎉 PRODUCTION READY

**New Features:**
- JSON output (`--json`)
- Quiet mode (`--quiet` - names only)
- Configurable count (`-n <count>`)
- Show all packages (`--all`)

**Features:**
- Scan .dotmeta files for package info
- Sort by last updated time
- Clean formatted output
- Multiple output formats

**Code Quality:**
- Clean 129+ lines
- Zero clippy warnings
- No unsafe operations
- Proper error handling

**Usage:**
```bash
latest-update              # Show last 10
latest-update -n 20        # Show last 20
latest-update --all        # Show all
latest-update --quiet      # Names only
latest-update --json       # JSON format
```

---

## [3.0.0] - Earlier

Package update tracker.

---

**Version Format:** MAJOR.MINOR.PATCH
