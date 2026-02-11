# get-version v4.0.0

Simple utility to read 0-Core system version and package versions.

## Features

- ✅ Get system version from VERSION file
- ✅ Get package versions from .dotmeta
- ✅ Health check integration
- ✅ Clean, simple output
- ✅ Exit codes for scripting

## Usage
```bash
# Get system version (default)
get-version
get-version system

# Get package version
get-version package shell-zsh

# Health check
get-version health
```

## Commands

- `system` - Show 0-Core system version (default)
- `package <name>` - Show package version from .dotmeta
- `health` - Run health check

## Output
```bash
$ get-version
9.6.0

$ get-version package editor-nvim
1.2.0
```

## Exit Codes

- `0` - Success
- `1` - Error (file not found, read error)

## Integration

Part of Faelight Forest:
- Reads from `~/0-core/00-meta/VERSION`
- Reads from `~/0-core/03-interfaces/stow/<package>/.dotmeta`
- Used by update scripts
- Version tracking

## Version: 4.0.0
