# 🔍 get-version

Extract version information from stow package `.dotmeta` files.

## Usage
```bash
# Get package version
get-version shell-zsh
# Output: 3.3.2

# Check health
get-version --health-check

# Show help
get-version --help
```

## Implementation

- Searches `~/0-core/stow/{package}/.dotmeta`
- Fallback to old `~/0-core/{package}/.dotmeta` path
- Returns "unknown" for missing packages

## Version

v2.0.0 - Stow-aware version reader
