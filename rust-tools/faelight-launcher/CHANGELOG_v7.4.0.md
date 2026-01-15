# 0-Core v7.4.0 Release Notes

## 🚀 Faelight Launcher v2.0 - XDG Desktop Integration

### Major Features
- **XDG Desktop Entry Scanner**: Automatically discovers all installed applications
- **Smart App Filtering**: Removes system utilities, duplicates, and non-user apps
- **Alphabetical Sorting**: Clean, organized app list
- **Scrolling Support**: Navigate through all apps with j/k or arrow keys
- **Frecency Tracking**: Most-used apps ranked higher in results
- **Fuzzy Search**: Fast app filtering as you type
- **Icon Configuration**: Customizable app icons via ~/.config/faelight/icons.toml

### Improvements
- Transparency background (91% opacity)
- Improved spacing and layout
- Better font sizing
- 11 visible apps with smooth scrolling

### Technical Details
- Dynamic app loading from /usr/share/applications
- Deduplication by app name
- System app blacklist (cmake, avahi, etc.)
- Launch history persisted to ~/.local/share/faelight-launcher/

### Statistics
- Discovered: 15 user-facing applications
- Total Lines: ~900 (main.rs) + ~160 (desktop module)
