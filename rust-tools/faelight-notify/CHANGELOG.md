# Changelog - faelight-notify

All notable changes to this project will be documented in this file.

## [2.0.0] - 2026-02-11

### 🎉 Production Ready

**Comprehensive Documentation:**
- 195-line README with complete usage guide
- Installation instructions (standalone + systemd service)
- D-Bus protocol explanation
- Examples for system/app integration
- Technical details and architecture
- Comparison with dunst/mako
- CHANGELOG.md added

**Code Quality:**
- Zero clippy warnings
- 8 error messages with helpful output
- Proper D-Bus interface implementation
- Wayland layer-shell integration

**Features:**
- Urgency-based color coding (RED/GREEN/DIM)
- Wayland-native rendering
- D-Bus org.freedesktop.Notifications compatible
- Click to dismiss
- Configurable timeouts
- Multi-line text rendering

### Technical
- Lines of code: 627
- Memory: ~5MB per notification
- Startup: <20ms
- Dependencies: zbus, faelight-core, smithay-client-toolkit

---

## [1.x.x] - Earlier

Previous versions with basic notification functionality.

---

**Version Format:** MAJOR.MINOR.PATCH
- MAJOR: Breaking changes or major feature additions
- MINOR: New features, non-breaking
- PATCH: Bug fixes only
