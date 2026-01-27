## [8.5.0] - 2026-01-26

### Added
- **Hybrid Wayland bar architecture** with integrated application launcher
  - Wayland layer-shell keyboard mode switching (first implementation of its kind)
  - Compact 400px dropdown overlay (doesn't disrupt window positions)
  - Real-time fuzzy search with nucleo (500+ applications)
  - Single-process design using compositor-mediated input modes

### Changed
- **faelight-bar**: Complete modular rewrite from v1.0.0 to v2.0.0
  - State machine architecture (bar/menu mode transitions)
  - Separate render pipeline (bar.rs, menu.rs with transparent overlays)
  - Input handling subsystem (keyboard navigation, pointer events)
  - Menu subsystem (fuzzy filtering, desktop app discovery)
  - Reduced codebase by 589 lines while adding functionality

### Documentation
- **FAELIGHT-CLI.md**: Refreshed for current CLI tool ecosystem
- **FAELIGHT-CONFIG.md**: Updated configuration patterns and examples
- **HEALTH-ENGINE.md**: Synchronized health check documentation
- **MANUAL_INSTALLATION.md**: Revised installation workflows
- **PHILOSOPHY.md**: Refined 0-Core principles and design philosophy
- **POLICIES.md**: Updated governance and contribution guidelines
- **QUICK_REFERENCE.md**: Enhanced quick reference for daily operations
- **TESTING.md**: Expanded testing strategies and tooling
- **TOOL_REFERENCE.md**: Comprehensive tool catalog refresh
- **WORKFLOWS.md**: Modernized common workflow documentation

### Fixed
- Click regions now properly tracked during rendering (profile, VPN, volume)
- Profile cycling preserved from v1.0.0
- Exclusive zone handling prevents window jumping during menu activation
