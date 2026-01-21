# keyscan v1.0.0 - Sway Keybind Analyzer

**Zero conflicts. Maximum visibility.**

## What It Does

Analyzes Sway keybindings to find conflicts, show usage patterns, and suggest available combinations.

## Commands
```bash
keyscan                    # Check for conflicts
keyscan --list             # Show all keybindings
keyscan --stats            # Usage statistics
keyscan --find firefox     # Search keybindings
keyscan --category         # Group by function
keyscan --cheatsheet       # Generate markdown reference
keyscan --suggest          # Find unused combos
keyscan --health           # Health check
```

## Example Output
```
📊 Keybind Statistics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Overview:
  Total keybindings: 99
  Unique bindings:   99
  Conflicts:         0

Modifier Usage:
  super           84
  shift           26

Most Used Keys:
  k               5
  h               4
  l               4

Categories:
  Applications         35
  Window Management    31
  Workspaces           21
```

## Features

- ✅ Conflict detection (finds duplicate bindings)
- ✅ Search by pattern (find all workspace bindings)
- ✅ Category grouping (apps, windows, system, etc.)
- ✅ Usage statistics (most used modifiers/keys)
- ✅ Unused combo suggestions (Super+F1-F10 available!)
- ✅ Markdown cheatsheet generation
- ✅ Health check integration (dot-doctor compatible)

## Use Cases

**Find conflicts:**
```bash
keyscan  # Exit code 1 if conflicts found
```

**Search bindings:**
```bash
keyscan --find workspace  # All workspace-related bindings
```

**Generate reference card:**
```bash
keyscan --cheatsheet > ~/keybinds.md
```

**Find free keybinds:**
```bash
keyscan --suggest  # Shows 10 unused combinations
```

---

**Built for Faelight Forest 0-Core**  
Part of the 30-tool Rust ecosystem
