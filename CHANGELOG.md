# Changelog

## [7.6.4] - 2026-01-19

### 🔧 Fixes
- **bump-system-version v3.1.0** - Fixed CHANGELOG template insertion
  - No longer requires blank line after "# Changelog" header
  - Removed automatic version history table insertion (manual edit required)
  - Cleaner error messages and validation

### 📦 Tool Updates
- bump-system-version v3.1.0 - Complete release automation (Intent 060)

> "The tools that build the forest must also grow." 🌲

---

## [7.6.2] - 2026-01-19

## [7.6.3] - 2026-01-19

### 🚀 New Features
- Complete GNU Stow-based package management (Intent #063)
- All 11 dotfile packages migrated to stow/ directory
- Automated deployment: `stow -t ~ package-name`

### 🔧 Fixes  
- Updated dot-doctor to recognize new stow/ structure
- Fixed theme package detection for stow layout
- Eliminated duplicate documentation/ directory

### 📦 Tool Updates
- dot-doctor v0.4 - stow-aware health checks

> "From scattered chaos to organized intention - the forest found its structure." 🌲

---

### 📐 Typography/UI
- **faelight-launcher v3.1.0** - Refined UI with improved spacing and text rendering
