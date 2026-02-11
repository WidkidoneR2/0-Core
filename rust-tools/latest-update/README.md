# latest-update v4.0.0

Show the most recently updated packages from .dotmeta files.

## Features

- ✅ Scan .dotmeta files for update times
- ✅ Sort by most recent updates
- ✅ Configurable output count
- ✅ Multiple output formats (normal, quiet, JSON)
- ✅ Clean, simple interface

## Usage
```bash
# Show last 10 updates (default)
latest-update

# Show last 20 updates
latest-update -n 20

# Show all packages
latest-update --all

# Quiet mode (names only)
latest-update --quiet

# JSON output
latest-update --json
```

## Output Formats

**Normal:**
```
📦 Latest 3 package updates:

  shell-zsh 2.1.0 - 2026-02-11 13:00
  editor-nvim 3.2.0 - 2026-02-10 15:30
  fm-yazi 1.5.0 - 2026-02-09 09:15
```

**Quiet (names only):**
```
shell-zsh
editor-nvim
fm-yazi
```

**JSON:**
```json
[
  {"name":"shell-zsh","version":"2.1.0","updated":"2026-02-11T13:00:00Z"},
  {"name":"editor-nvim","version":"3.2.0","updated":"2026-02-10T15:30:00Z"}
]
```

## Integration

Part of Faelight Forest:
- Reads from `~/0-core/03-interfaces/stow/<package>/.dotmeta`
- Shows package update activity
- Helps track maintenance work
- Used in update workflows

## Version: 4.0.0
