## Suckless Philosophy

**Source:** https://suckless.org/
**Interest:** Minimalist design philosophy
**Status:** Philosophy study, not tool adoption

**Key Principles to Apply:**
- Keep code simple and readable
- Minimize dependencies
- Configuration should be straightforward
- Avoid feature creep

**What NOT to do:**
- Switch to X11 (backwards)
- Use suckless tools on Wayland (incompatible)
- Abandon themes for minimalism (conflicts with goals)

**What TO do:**
- Apply minimalist thinking to scripts
- Keep the theme engine focused and simple
- Avoid unnecessary complexity in FCM
- Write readable, maintainable code

**Status:** Philosophy absorbed, tools not applicable

---

## AETHER SHELL - Material Design on Hyprland

**Date Researched:** December 7, 2025  
**Source:** Custom research compilation  
**Type:** Complete desktop environment redesign  
**Complexity:** HIGH (20-30 hours)

### Concept
Material Shell aesthetic on Hyprland using:
- Hyprland (layout engine)
- EWW (side panel with workspace/window list)
- Waybar (top bar, minimal)
- Rofi (launcher, material theme)

### Architecture
```
~/.config/aether-shell/
├── hypr/          # Layout zones (tile/stack/max)
├── panel/         # EWW side panel
├── bar/           # Waybar config
└── rofi/          # Material theme
```

### Key Features
- Material Shell–like workspaces (1-9)
- Side panel showing windows
- Layout zones (tile/stack/maximize)
- Glassmorphism aesthetic
- Unified Material Design

### Implementation Notes
[Your full config pasted here]

---

### Analysis

**Pros:**
- Material Design aesthetic
- Interesting workspace model
- Side panel is cool

**Cons:**
- Conflicts with Faelight Forest theme
- Requires EWW (new tool to learn)
- Complete system redesign
- Abandons current progress
- 20-30 hours of work
- May not be better than current setup

**Decision:** DEFER until v4.0+

**Why:**
1. Current system works perfectly
2. In middle of theme engine development
3. Would lose Faelight Forest aesthetic
4. FCM atomic packages (v3.0) would make this easier to test
5. Can create `aether-shell` profile LATER

**Timeline:**
- NOW: Focus on v2.8.2 (color extraction)
- v3.0: Complete FCM with atomic packages
- v4.0+: THEN experiment with Aether Shell as alternate profile

**Status:** Captured, deferred, will revisit after FCM complete

---
