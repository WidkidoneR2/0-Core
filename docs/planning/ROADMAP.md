# 🗺️ Faelight Forest Development Roadmap

**Current Version:** 2.7.1 - Theme Toggle System  
**Last Updated:** November 28, 2025

---

## 📝 Version 2.7.1 - Polish & Refinement

### Goals: Fine-tune Current Features

**Light Theme Improvements:**
- [ ] Test light theme in different lighting conditions
- [ ] Adjust colors for better readability
- [ ] Fine-tune Waybar in light mode
- [ ] Add medium contrast option?

**Waybar Theme Switching:**
- [ ] Create waybar theme configs (dark/light)
- [ ] Integrate into theme-switch.sh
- [ ] Test all modules in both themes

**Testing:**
- [ ] Use light theme at different times of day
- [ ] Note what needs improvement
- [ ] Gather feedback from usage

**Time Estimate:** 2-4 hours  
**Focus:** Making existing features perfect

---

## 🎨 Version 2.8 - Enhanced Theming & Recovery

### Primary Goals

**1. Enhanced Recovery Guide**
- [ ] Write comprehensive RECOVERY_GUIDE.md
- [ ] Step-by-step GRUB snapshot recovery
- [ ] Manual BTRFS rollback procedures
- [ ] Single file restoration guide
- [ ] Complete reinstall walkthrough
- [ ] Common issues & solutions database
- [ ] Test on VM (optional but recommended)

**2. Time-Based Theme Switching**
- [ ] Auto-switch to dark at night (8 PM)
- [ ] Auto-switch to light at day (8 AM)
- [ ] Configurable times
- [ ] Manual override capability
- [ ] Notification on auto-switch

**3. Theme Variants**
- [ ] Nord-inspired Faelight variant
- [ ] Gruvbox-inspired Faelight variant
- [ ] Keep Faelight aesthetic but adapt palettes

**4. Waybar Full Integration**
- [ ] Waybar configs for all themes
- [ ] Module color coordination
- [ ] Test with VPN status, workspaces, etc.

**Time Estimate:** 1-2 weeks  
**Impact:** Bulletproof recovery + more theme options

---

## 🤖 Version 2.8.1-2.8.5 - Theme Intelligence Engine

### The Big Project: Pywal-Style Custom Engine

**Phase 1 (v2.8.1): Research & Foundation**
- [ ] Study pywal architecture
- [ ] Research color extraction tools (ImageMagick, colorz)
- [ ] Understand color theory (complementary, analogous)
- [ ] Design algorithm for palette generation

**Phase 2 (v2.8.2): Color Extraction**
- [ ] Install/test color extraction tools
- [ ] Write wallpaper color extraction script
- [ ] Generate 16-color palette from wallpaper
- [ ] Validate palette contrast ratios
- [ ] Test with various wallpapers

**Phase 3 (v2.8.3): Template System**
- [ ] Create template versions of configs
- [ ] Variables: {{background}}, {{primary}}, etc.
- [ ] Template engine (Jinja2 or sed-based)
- [ ] Generate test configs

**Phase 4 (v2.8.4): Integration**
- [ ] Integrate with theme-switch.sh
- [ ] Auto-generate theme.json from wallpaper
- [ ] Apply to Hyprland
- [ ] Apply to Kitty
- [ ] Apply to Waybar
- [ ] Apply to Mako
- [ ] Apply to LazyVim (optional)

**Phase 5 (v2.8.5): Polish & Features**
- [ ] Add wallpaper detection on change
- [ ] Cache generated themes
- [ ] Allow manual palette tweaking
- [ ] Add presets (vibrant, muted, pastel)
- [ ] Live preview before applying
- [ ] Create wallpaper gallery with generated themes

**Commands:**
```fish
set-wallpaper ~/Pictures/forest.jpg    # Auto-themes everything
theme-from-wallpaper                   # Manual trigger
theme-preview                          # Preview before apply
```

**Tools to Evaluate:**
- pywal - reference implementation
- wpgtk - GUI version
- Chameleon - advanced extraction
- ImageMagick - color analysis
- colorz - Python color extraction

**Time Estimate:** 4-6 weeks (iterative)  
**Impact:** 🤯 Your system adapts to any wallpaper  
**Complexity:** High (but worth it!)

---

## 🔐 Version 2.9 - Security & Backup Infrastructure

### Cloud Integration
- [ ] Set up Filen.io sync workflow
- [ ] Create ~/Filen/FilenBackups/ structure
- [ ] Automate KeePassXC backup to Filen
- [ ] Automate dotfiles backup to Filen
- [ ] Optional: Export BTRFS snapshots to Filen

### KeePassXC Integration
- [ ] Document vault structure in README
- [ ] Add keybinds for quick KeePassXC access
- [ ] Set up auto-type for common workflows
- [ ] Browser integration for web logins
- [ ] TOTP 2FA entries

### Notesnook Integration  
- [ ] Add keybind for quick notes
- [ ] Sync workflow (if needed)
- [ ] Document notes organization

### Backup Automation
- [ ] Create sync-filen.sh script
- [ ] Weekly automated backups
- [ ] Backup verification script
- [ ] Recovery testing procedure

## 🚀 Version 3.0 - Complete Ecosystem

### Multi-Machine Support (if needed)
- [ ] Per-host overrides in dotfiles
- [ ] Secrets distribution strategy
- [ ] Theme sync across machines

### Advanced Features
- [ ] Restic versioned backups via Filen WebDAV
- [ ] Automated disaster recovery script
- [ ] Health monitoring dashboard
- [ ] Complete recovery walkthrough
```

---

## 💡 My Professional Opinion

**Your security model:**
```
Local:  LUKS2 → BTRFS Snapshots → KeePassXC
Cloud:  Filen.io E2EE → Zero-knowledge backup
Git:    Clean, no secrets → Public-safe

## 🌲 Version 3.0 - "Faelight Forest Evolved"

### Major Features

**1. Seasonal Theme System ("Seasons of Faelight")**

**Spring (Mar-May):** 🌸
```
Background: #f0f5f0  (soft green-white)
Primary:    #7cb342  (fresh green)
Accent:     #ffb74d  (daffodil yellow)
Vibe:       Gentle, growing, hopeful
Animation:  Smooth, flowing
```

**Summer (Jun-Aug):** ☀️
```
Background: #fffbf0  (warm white)
Primary:    #ffb300  (golden sun)
Accent:     #4db6ac  (ocean teal)
Vibe:       Bright, energetic, clear
Animation:  Fast, vibrant
```

**Autumn (Sep-Nov):** 🍂
```
Background: #1a1410  (dark bark)
Primary:    #d84315  (burnt orange)
Accent:     #8d6e63  (warm brown)
Vibe:       Cozy, warm, contemplative
Animation:  Gentle, settling
```

**Winter (Dec-Feb):** ❄️
```
Background: #0d1117  (deep night)
Primary:    #64b5f6  (ice blue)
Accent:     #b0bec5  (frost gray)
Vibe:       Minimal, calm, focused
Animation:  Slow, crystalline
```

**Implementation:**
- [ ] Design all 4 seasonal palettes
- [ ] Create theme.json for each season
- [ ] Write seasonal-auto.sh (detects month)
- [ ] Add to cron (auto-rotate monthly)
- [ ] Manual season override
- [ ] Seasonal wallpaper packs
- [ ] Seasonal notification sounds?

**2. Hero Mode / Focus Mode** 🦸

**Features:**
- [ ] Dim all unfocused windows (opacity 0.3)
- [ ] Main window zooms slightly
- [ ] Background blur increases with distance
- [ ] Waybar collapses to dot indicator
- [ ] All notifications muted (except critical)
- [ ] Gaps increase for breathing room
- [ ] Keybind: SUPER + SHIFT + H

**Script:**
```bash
~/dotfiles/scripts/hero-mode.sh [on|off|toggle]
```

**Hyprland Integration:**
```conf
decoration {
    dim_inactive = 0.7      # Dim unfocused
    blur {
        passes = 3          # Extra blur
    }
}
```

**3. Advanced Animations**
- [ ] Seasonal-specific animations
- [ ] Window entrance effects
- [ ] Workspace transition styles
- [ ] Notification animations

**4. Performance Dashboard**
- [ ] Waybar widget for CPU/RAM/Disk
- [ ] GPU usage (if applicable)
- [ ] Network traffic
- [ ] Battery (if laptop)
- [ ] Click for detailed view

**Time Estimate:** 2-3 months  
**Impact:** Legendary, poetic, productive system

---

## 🎯 Development Timeline

### Immediate (This Week)
- LazyVim IDE Mode (2-3 hours)

### Short-term (Next 2 Weeks)
- Version 2.7.1 - Polish (2-4 hours)
- Version 2.8 - Enhanced features (1-2 weeks)

### Medium-term (Next 2 Months)
- Version 2.8.1-2.8.5 - Theme Intelligence (4-6 weeks)

### Long-term (3+ Months)
- Version 3.0 - Seasonal + Hero Mode (2-3 months)

---

## 💡 Future Ideas (Post-3.0)

**Version 3.1+:**
- Multi-monitor theme variations
- Per-workspace themes
- Voice-activated theme switching
- Theme marketplace/sharing
- Video tutorials
- System performance optimizations
- Custom widget creation

**Dream Features:**
- AI-assisted theme generation
- Gesture-based theme switching (touchpad)
- Music-reactive themes (for fun!)
- Weather-based theme adaptation
- Productivity metrics dashboard

---

## 📊 Complexity Matrix

| Feature | Complexity | Time | Impact | Priority |
|---------|-----------|------|--------|----------|
| LazyVim IDE | ⭐⭐ | 2-3h | High | NOW |
| v2.7.1 Polish | ⭐ | 2-4h | Medium | Soon |
| v2.8 Recovery | ⭐⭐⭐ | 1w | High | Next |
| Theme Engine | ⭐⭐⭐⭐⭐ | 4-6w | Massive | Later |
| Seasonal | ⭐⭐⭐ | 1w | High | v3.0 |
| Hero Mode | ⭐⭐⭐ | 1d | Medium | v3.0 |

---

## 🎓 Learning Objectives

**Through This Roadmap You'll Master:**
- Advanced Lua (LazyVim configs)
- Color theory & palette generation
- Template engines & code generation
- Advanced Hyprland features
- Shell scripting mastery
- Git workflow perfection
- Technical writing
- System architecture design

---

## 🌲 Philosophy

**Faelight Forest grows organically:**
- Each version adds depth
- Features build on each other
- Never rush perfection
- Enjoy the journey
- Share what you learn

**The goal isn't just a cool system.**  
**The goal is mastery, creativity, and joy.**

---

*"Your desktop should breathe with you, adapt to you, and inspire you."*

**Current Status:** Version 2.7.0 Complete ✅  
**Next Action:** LazyVim IDE Mode  
**Vision:** Version 3.0 - A Living, Breathing Forest 🌲✨

---

*Last Updated: November 27, 2025*  
*Roadmap Version: 3.0*
