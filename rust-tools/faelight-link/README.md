# faelight-link v2.0.0

Zone-aware symlink manager for Faelight Forest.

## Features

- ✅ Stow/unstow packages
- ✅ Conflict resolution with backups
- ✅ Link health auditing
- ✅ Dry-run mode (`--dry-run`)
- ✅ JSON output (`--json`)
- ✅ Helpful error hints
- ✅ Automatic cleanup of broken links

## Usage
```bash
# Stow a package
faelight-link stow shell-zsh

# Dry-run (preview changes)
faelight-link stow editor-nvim --dry-run

# Unstow a package
faelight-link unstow fm-yazi

# List available packages
faelight-link list

# Check link health
faelight-link audit

# Clean broken links
faelight-link clean

# Show status
faelight-link status
```

## Commands

- `stow <package>` - Create symlinks for package
  - `--force` - Skip verification prompts
  - `--dry-run` - Preview without changes
- `unstow <package>` - Remove symlinks
- `list` - Show all available packages
- `status` - Display link status
- `audit` - Check for broken/orphaned links
- `clean` - Remove broken links
  - `--force` - Skip confirmation

## Global Flags

- `--dry-run` - Preview mode (no changes)
- `--json` - JSON output format
- `-h, --help` - Show help
- `-V, --version` - Show version

## Package Structure

Packages live in `~/0-core/03-interfaces/stow/`:
```
stow/
├── shell-zsh/
│   └── .zshrc
├── editor-nvim/
│   └── .config/nvim/
└── fm-yazi/
    └── .config/yazi/
```

## Error Hints

faelight-link provides helpful guidance:

- Unknown package → Shows available packages
- No packages found → Explains expected location
- Broken links → Suggests cleanup command

## Integration

Part of the Faelight Forest ecosystem:
- Zone-aware operations
- Health monitoring integration
- Conflict resolution with backups

## Version: 2.0.0
