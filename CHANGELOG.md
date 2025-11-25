# 🌲 Faelight Forest Dotfiles - Changelog

## Version 2.5 (2025-11-25) - 🎨 The Theming & Documentation Update

### Fixed
- Browser workspace rule - Brave now correctly opens in workspace 2
- Fixed Brave class name (`brave-browser` instead of `Brave-browser`)
- Added `silent` flag to prevent workspace stealing

### Added
- **Brave Browser Theming** - Faelight Forest Stylus CSS for new tab page
- **Mako Notifications** - Beautifully themed forest notifications with urgency levels
- **Comprehensive Documentation:**
  - `MELD_GUIDE.md` - Visual diff workflows and verification aliases
  - `KEYBINDINGS.md` - Complete keyboard shortcut reference (100+ bindings)
  - `brave/THEMING.md` - Browser customization guide
- **Help Keybind** - `SUPER + /` opens keybindings reference
- **Notification Controls:**
  - `SUPER + I` - Toggle Do Not Disturb mode
  - `SUPER + SHIFT + I` - Clear all notifications

### Changed
- Updated Mako config with forest green backgrounds, cyan borders, and lime accents
- Organized keybindings guide by category (Apps, Workspaces, Windows, System)
- Improved documentation structure in `/docs` directory

### Documentation
- Created visual comparison workflows for Meld
- Added Thunar + Meld integration instructions
- Comprehensive keybinding tables with legends and pro tips
- Browser theming with color palette reference

## Version 2.1 (2025-11-24)

### Fixed
- Resolved SUPER+TAB keybind conflict (window cycling vs group switching)
- Moved group navigation to SUPER+G/SUPER SHIFT+G  
- Fixed YouTube keybind to SUPER SHIFT+U
- Fixed Thunar closing when terminal closes (background function)

### Added
- **Thunar** file manager (replaced Nautilus)
- **Meld** visual diff tool with Thunar integration
- **Papirus-Dark** icon theme with sunset-colored folders
- **nwg-look** GTK theme manager for Wayland
- GTK 3.0/4.0 configuration files
- Meld verification aliases (verify-hypr, verify-waybar, etc.)
- Updated workspace icons in Waybar (terminal, browser, files, code, media)
- Gitleaks secret scanning with pre-commit hook
- `.gitleaks.toml` configuration
- Comprehensive keybinding documentation

### Changed
- Updated COMPLETE_GUIDE.md with workflow improvements
- Better keybinding organization with descriptive `bindd` labels
- Improved Fish config with Thunar/Meld support
- GTK theming now matches Tropical Sunset color scheme

### Removed
- Nautilus file manager and dependencies

## Version 2.0 (2025-11-XX)
- Initial Faelight Forest theme release
- 5 themed workspaces
- Walker launcher integration  
- Comprehensive keybindings
- Tokyo Night color scheme
