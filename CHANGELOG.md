# 🌲 Faelight Forest Dotfiles - Changelog

All notable changes to Omarchy dotfiles.

---

## [2.8.6] - 2025-12-09

### 📚 Documentation Overhaul

**README.md - Complete Rewrite:**
- New structure: Philosophy, Quick Start, System Structure, Core Tools
- Detailed tool explanations (WHY each tool was chosen)
- Faelight Theme Engine documentation
- dot-doctor and keyscan usage guides
- Stow-based management explanation
- Package philosophy section (what was removed and why)
- Installation guide
- Comprehensive keybinding reference

**Files Updated:**
- `README.md` - Complete rewrite from scratch
- `CHANGELOG.md` - Full v2.8.x history documented

### 🎯 Summary

Complete documentation reflecting all v2.8.x changes. System is fully documented, tested, and production-ready.

---

## [2.8.5] - 2025-12-09

### 🚀 Complete System Optimization - Native Wayland Stack

**Major Migration: Rofi → Fuzzel**
- Removed rofi-wayland (had click/focus issues)
- Installed fuzzel (native Wayland, instant response)
- Created fuzzel-theme-dark package with Faelight colors
- Configured: width=50, lines=8, line-height=28, size=12
- Power menu: `power-menu-fuzzel.sh`
- Omarchy menu: `omarchy-menu-fuzzel.sh`
- Theme engine: Added `fuzzel.ini.j2` template

**Terminal Migration: Kitty → Foot + Ghostty**
- Removed Kitty (60MB freed) - X11-based, heavier
- Installed Foot as primary (native Wayland, 2ms startup, 5-10MB RAM)
- Configured with JetBrainsMono Nerd Font (perfect spacing)
- Ghostty remains as backup terminal
- Created foot-theme-dark package
- Theme engine: Added `foot.ini.j2` template
- All keybindings updated to use Foot
- Scripts updated: omarchy-menu, theme-switch

**File Manager Optimization**
- Removed Thunar + plugins (20MB freed)
- Yazi configured with Meld integration (SUPER ALT+cm)
- Added PCManFM-Qt (3MB) as GUI backup
- No GUI file manager needed for daily use!

**New Tools Installed:**
- `cliphist` - Clipboard history (SUPER+V)
- `slurp` - Area selection for screenshots
- `hyprpicker` - Color picker (SUPER SHIFT+C)
- `zathura` - Lightweight PDF viewer (5MB vs Evince's 40MB)
- `foot` - Primary terminal
- `pcmanfm-qt` - GUI file manager backup
- `topgrade` - System updater

**Removed (Total: ~106MB freed):**
- `rofi` (1MB)
- `kitty` + plugins (60MB)
- `thunar` + plugins (20MB)
- `evince` (15MB)
- `gvfs` family (6MB)
- `smbclient`, `libmtp`, etc. (4MB)

**Configuration Updates:**
- All Hyprland keybindings updated
- Fish shell functions updated
- Workspace rules updated
- theme-switch.sh: Graceful stow error handling
- Autostart: cliphist integration

**Theme Engine:**
- Updated render.py: Added fuzzel and foot templates
- Updated apply-to-dotfiles.sh: Fuzzel and Foot support
- Generates themes for 5 apps: Kitty, Ghostty, Foot, Fuzzel, Waybar

**Package Lists:**
- pkglist.txt: Removed old packages, added new tools
- packages/official.txt: Updated
- Added missing essentials: topgrade, btop, lazydocker, lazygit, meld

**Files Modified:**
- `hypr/.config/hypr/bindings.conf` - Updated terminal and file manager
- `hypr/.config/hypr/workspaces.conf` - PCManFM-Qt rules
- `hypr/.config/hypr/autostart.conf` - cliphist startup
- `fish/.config/fish/config.fish` - PCManFM function
- `scripts/omarchy-menu-fuzzel.sh` - Fuzzel integration
- `scripts/power-menu-fuzzel.sh` - Created
- `scripts/theme-switch` - Updated for Foot/Fuzzel
- `fuzzel-theme-dark/.config/fuzzel/fuzzel.ini` - Faelight colors
- `foot-theme-dark/.config/foot/foot.ini` - Faelight colors

**Files Added:**
- `foot-theme-dark/` - Complete Foot configuration
- `fuzzel-theme-dark/` - Complete Fuzzel configuration
- `scripts/power-menu-fuzzel.sh` - Power menu
- `~/faelight-theme-engine/templates/foot.ini.j2` - Template
- `~/faelight-theme-engine/templates/fuzzel.ini.j2` - Template

**Files Removed:**
- `kitty-theme-dark/`
- `kitty/`
- `scripts/omarchy-menu-rofi.sh`
- `scripts/power-menu-rofi.sh`

**Results:**
- ✅ 106MB system bloat removed
- ✅ 100% native Wayland stack
- ✅ Faster performance across the board
- ✅ All tools themed and working perfectly
- ✅ No redundancy (1 primary terminal, 1 backup)
- ✅ Professional workflow maintained

---

## [2.8.4] - 2025-12-08

### 🔗 Theme Engine Integration

**Dotfiles Workflow:**
- Created `apply-to-dotfiles.sh` script
- Renders templates to temp directory
- Applies to dotfiles stow packages automatically
- Updates kitty-theme-dark, ghostty-theme-dark, rofi-theme-dark
- Preserves base configs, only updates colors
- Git workflow guidance included

**Structure:**
- `~/faelight-theme-engine/scripts/apply-to-dotfiles.sh`
- Renders all templates with one command
- Color-coded output (red/green/yellow/blue)
- Automatic cleanup (trap on EXIT)

**Integration Testing:**
- Created kitty-theme-dark with current-theme.conf
- Updated ghostty-theme-dark colors
- Waybar preview generated
- theme-switch.sh integration

**Files:**
- `scripts/apply-to-dotfiles.sh` - Created and tested
- Fixed kitty stow conflict (removed current-theme.conf from base)
- Updated theme-switch script name (removed .sh extension)

---

## [2.8.3] - 2025-12-08

### 🎨 Template Rendering System

**Jinja2 Templates Created:**
- `templates/kitty.conf.j2` - Complete ANSI colors, cursor, selection
- `templates/ghostty.conf.j2` - Colors without # prefix (Ghostty format)
- `templates/rofi.rasi.j2` - CSS variables, complete styling
- `templates/waybar.css.j2` - Bar styling, module colors, animations

**Rendering Engine:**
- `faelight_theme/render.py` - Template rendering logic
- `render_template()` - Single template rendering
- `render_to_file()` - Render and save
- `render_all()` - Batch rendering
- Jinja2 environment with FileSystemLoader
- Template directory auto-detection
- Output directory creation

**CLI Integration:**
- `faelight-theme render palette.json --target kitty`
- `faelight-theme render palette.json --all`
- Available targets: kitty, ghostty, rofi, waybar

**Testing:**
- Generated 4 perfect configs from test-palette.json
- Verified all color formats correct
- Ghostty colors properly formatted (no # prefix)

**Troubleshooting:**
- Fixed multiple Python syntax errors
- Removed incomplete code blocks
- Removed duplicate function definitions

**Files:**
- `templates/` - All 4 templates created
- `faelight_theme/render.py` - Complete rendering system
- Updated `cli.py` with render command

---

## [2.8.2] - 2025-12-08

### 🎨 Color Extraction Core

**Color Extraction (extract.py):**
- `extract_colors()` - Uses colorgram.py for extraction
- `rgb_to_hex()`, `hex_to_rgb()` - Color format conversion
- `calculate_brightness()` - Weighted: 0.299*R + 0.587*G + 0.114*B
- `detect_theme()` - Auto dark/light detection (<128 = dark)
- `sort_by_brightness()` - Color organization

**Palette Generation (palette.py):**
- `generate_palette()` - Creates 16 ANSI colors from extracted colors
- Hue-based color mapping: Red(0°), Yellow(60°), Green(120°), Cyan(180°), Blue(240°), Magenta(300°)
- `find_closest_hue()` - 60° tolerance matching
- Bright variants: 30% brighter for colors 8-15
- Accent selection: Highest saturation + contrast optimization
- HSV color space manipulation

**Contrast Validation (contrast.py):**
- `contrast_ratio()` - WCAG calculation (1-21 scale)
- `validate_palette()` - Returns valid/warnings/ratios
- `auto_adjust_contrast()` - Iterative brightness adjustment
- Requirements: BG/FG ≥7:1 (AAA), BG/Accent ≥4.5:1 (AA)
- Gamma correction for relative luminance

**CLI Commands:**
- `faelight-theme extract <image> [--output palette.json]`
- `faelight-theme validate <palette.json>`

**Testing:**
- 5/5 unit tests passing
- Test wallpaper: Extracted 8 colors, auto-detected dark theme
- Background: #191c37 (dark blue, 51.8%)
- Foreground: #7cc6b0 (teal, auto-adjusted to 8.36:1 contrast)
- Accent: #ca7f99 (pink, 5.55:1 contrast)
- All 16 ANSI colors generated with proper bright variants

**Accent Optimization Fix:**
- Initial accent had 1.0:1 contrast (too low)
- Fixed accent selection algorithm to balance saturation and contrast
- Result: 5.55:1 contrast (passes AA standard)

**Files:**
- `faelight_theme/extract.py` - 150 lines
- `faelight_theme/palette.py` - 180 lines
- `faelight_theme/contrast.py` - 150 lines
- `faelight_theme/cli.py` - Updated with commands
- `tests/test_extract.py` - Unit tests

---

## [2.8.1] - 2025-12-08

### 🎨 Faelight Theme Engine - Project Setup

**Phase 1A - Package Structure:**
- Created `~/faelight-theme-engine/` project directory
- Set up Python package structure with `pyproject.toml`
- Configured Click CLI framework
- Added dependencies: Pillow, Jinja2, colorgram.py
- Created skeleton CLI: `faelight-theme --help`

**Phase 1B - Research & Planning:**
- Researched color extraction libraries
- Analyzed terminal color format requirements
- Documented theme file structures (Kitty, Ghostty, Rofi, Waybar)
- Created comprehensive `DESIGN.md` with architecture

**Documentation:**
- `DESIGN.md` - Complete system architecture
- Module breakdown: extract, palette, contrast, render, apply
- Input/output flow documented
- Color format specifications
- Template examples

**Files:**
- `pyproject.toml` - Project configuration
- `faelight_theme/__init__.py` - Package init
- `faelight_theme/cli.py` - CLI framework
- `DESIGN.md` - Complete design document
- `README.md` - Project overview

---

## [2.8.0] - 2025-12-06

### 🛠️ Rofi Migration & System Utilities

**Walker → Rofi Migration:**
- Removed Walker (335MB freed - unmaintained, flaky)
- Installed rofi-wayland as launcher
- Created `omarchy-menu-rofi.sh` - Central system menu
- Created `power-menu-rofi.sh` - Power options
- Enhanced Rofi theme with glassmorphism and glow effects
- Custom theme: `rofi/.config/rofi/faelight-forest.rasi`

**Ghostty Terminal:**
- Added Ghostty as backup terminal
- Complete configuration with Faelight Forest theme
- ghostty-theme-dark package structure

**System Health Tools:**
- `dot-doctor` - Comprehensive system health checker
  - Validates packages, configs, scripts
  - Checks stow deployments
  - Reports issues with fixes
- `keyscan` - Auto-generates keybinding documentation
  - Scans bindings.conf
  - Formats as readable markdown
  - Perfect for learning your system

**Cleanup:**
- Removed elephant folders (old backup directories)
- Fixed dot-doctor checks
- Updated all package references

**Files:**
- `scripts/dot-doctor` - System validator
- `scripts/keyscan` - Keybinding documentation
- `scripts/omarchy-menu-rofi.sh` - Main menu
- `scripts/power-menu-rofi.sh` - Power menu
- `rofi/.config/rofi/faelight-forest.rasi` - Enhanced theme
- `ghostty-theme-dark/` - Ghostty configuration

---

## [Earlier Versions]

See git history for v2.7.x and earlier changes.

---

**Note:** Version 2.8.x represents a major system optimization focused on native Wayland tools, bloat removal, and the addition of the Faelight Theme Engine for automated theming workflows.
