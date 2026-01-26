# faelight-stow v0.3.0

🔗 **Stow symlink verification and management for 0-Core**

Part of the Faelight Forest ecosystem - ensures all dotfile packages are properly symlinked via GNU Stow.

---

## Features

- ✅ **Auto-discovery** - Scans `~/0-core/stow/` for all packages
- ✅ **Verification** - Checks all symlinks are valid and point to correct targets
- ✅ **Auto-fix** - Repairs broken symlinks with `stow -R`
- ✅ **Selective stowing** - Install individual packages on demand
- ✅ **Integration** - Used by `dot-doctor` for system health checks
- ✅ **Silent mode** - `--quiet` for scripting (exit code only)

---

## Usage

### Verify all packages
```bash
faelight-stow
```

### Verify silently (for scripts)
```bash
faelight-stow --quiet
echo $?  # 0 = healthy, 1 = issues found
```

### Auto-fix broken symlinks
```bash
faelight-stow --fix
```

### Stow a new package
```bash
faelight-stow shell-zsh
```

### Send notification on issues
```bash
faelight-stow --notify
```

---

## Philosophy

**Auto-discover packages, verify integrity, stay in control.**

Unlike running `stow` manually for each package, `faelight-stow`:
- Discovers all packages automatically
- Verifies existing symlinks are correct
- Only fixes what's actually broken
- Provides clear feedback about system state

---

## Integration

Used by `dot-doctor` as a health check:
```rust
✅ Stow Symlinks: All 12/12 packages properly stowed
```

---

## Exit Codes

- `0` - All packages healthy
- `1` - Issues found (or with `--fix`: issues couldn't be fixed)

---

## Technical Details

**Auto-discovery:**
- Scans `~/0-core/stow/` for directories
- Each directory = one package
- Verifies symlinks in `$HOME` point to package contents

**Verification:**
- Checks symlink exists
- Checks symlink target is valid
- Checks symlink points to correct stow package

**Auto-fix:**
- Runs `stow -R <package>` to restow
- Only attempts fix if package directory exists
- Reports success/failure clearly

---

## Examples

**Daily verification (integrated with dot-doctor):**
```bash
doctor  # Includes stow verification
```

**Manual check before system update:**
```bash
faelight-stow || echo "Fix symlinks before updating"
```

**CI/CD health check:**
```bash
faelight-stow --quiet && echo "✅ Dotfiles healthy"
```

---

## Part of 0-Core

One of 30+ Rust tools in the Faelight Forest ecosystem.

See: https://github.com/WidkidoneR2/0-Core
