---
id: 043
date: 2026-01-13
type: decisions
title: "paru Migration - Modern AUR Helper"
status: decided
tags: [aur, package-management, tooling]
---

## Decision

Migrate from `yay` to `paru` as the AUR helper for 0-Core.

## Context

- yay development has slowed significantly
- paru is more actively maintained
- Better PKGBUILD review workflow
- Cleaner upgrade news integration
- Improved conflict resolution

## Implementation

**v6.8.1:**
- Installed paru
- Updated topgrade config: `arch_package_manager = "paru"`
- Removed yay: `paru -Rns yay`
- Created tools-topgrade stow package

## Benefits

1. **Active Development**: paru is actively maintained
2. **Better Reviews**: Improved PKGBUILD review interface
3. **News Integration**: Shows Arch news before upgrades
4. **Compatibility**: Drop-in yay replacement (same flags)
5. **Cleaner Output**: Better formatted search/upgrade output

## Migration Path
```bash
# Install paru
yay -S paru

# Test compatibility
paru -Syu

# Remove yay (after verification)
paru -Rns yay
```

## Configuration

**topgrade integration:**
```toml
[linux]
arch_package_manager = "paru"
show_arch_news = true
```

## Alternatives Considered

- **Keep yay**: Rejected - development stalled
- **Use pacman only**: Rejected - AUR access needed
- **Try other helpers**: paru emerged as clear winner

## Validation

- ✅ topgrade detects paru
- ✅ All updates work
- ✅ PKGBUILD review functional
- ✅ News display working

---

_"Choose tools that are maintained, not just familiar."_ 🌲
