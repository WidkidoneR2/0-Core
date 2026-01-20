---
intent_id: 066
title: "faelight-bar v2.0 - The Sentient Branch"
status: planned
priority: high
category: tool-improvement
target_version: 7.7.0
created: 2026-01-19
updated: 2026-01-19
---

## Description
Transform faelight-bar from functional to legendary through architectural improvements, 
visual polish, and ecosystem-wide icon unification.

## Goals

### 1. Icon System Foundation (NEW!)
Create `faelight-icons` shared crate that:
- Reads icon design tokens from `starship.toml`
- Provides cached, type-safe icon access
- Enables single source of truth across entire Faelight ecosystem
- Allows icon customization without recompilation

### 2. Model/View Separation
Separate data collection from rendering to achieve 70-90% CPU reduction:
- `BarModel` struct caches all system state
- Background updates at intelligent intervals
- Pure rendering in draw loop (no IO)

### 3. Animated Transitions
Visual state changes that feel alive:
- Lock/unlock icon fade transition
- VPN connection glow effect
- Battery low warning pulse
- Workspace slide animations
- Profile change accent ripple

### 4. Production Robustness
- Serde JSON parsing for Sway workspace data
- Typed `ClickAction` enum (no string matching)
- Comprehensive error handling with fallbacks
- Command timeouts and graceful degradation

## Success Criteria

### Phase 0: Icon System (NEW!)
- [x] PROGRESS.md tracking file created
- [ ] `faelight-icons` crate created in rust-tools/
- [ ] `starship.toml` extended with `[faelight_icons]` section
- [ ] Icon loading with caching implemented
- [ ] Test suite verifies icon loading
- [ ] faelight-bar integrated with shared icons

### Phase 1: Foundation
- [ ] BarModel struct implemented
- [ ] Model update separated from draw()
- [ ] Per-module update intervals working
- [ ] Bar still functional after refactor

### Phase 2: Performance
- [ ] Glyph cache implemented
- [ ] CPU usage <5% average (from ~15%)
- [ ] Serde JSON parsing for workspaces
- [ ] Intelligent redraw triggers

### Phase 3: Visual Magic
- [ ] Lock/unlock icons ( / ) working
- [ ] Lock toggle animation smooth
- [ ] VPN glow animation on connection
- [ ] Battery pulse warning <15%
- [ ] Color-coded states (health/lock/vpn)
- [ ] All icons from faelight-icons crate

### Phase 4: Polish
- [ ] Typed ClickAction enum
- [ ] Robust error handling
- [ ] All edge cases tested
- [ ] README.md documentation
- [ ] ARCHITECTURE.md documenting design
- [ ] v2.0.0 released and tagged

## Architecture

### Icon System Flow
```
starship.toml (design tokens)
      ↓
faelight-icons (Rust crate, cached)
      ↓
┌─────────┬──────────┬─────────────┬──────────┐
│ faelight│ faelight │   faelight  │   CLI    │
│  -bar   │ -launcher│   -notify   │  tools   │
└─────────┴──────────┴─────────────┴──────────┘
```

### Bar Model Structure
```rust
struct BarModel {
    // State
    profile: String,
    workspaces: Vec<WorkspaceInfo>,
    lock: LockState,
    vpn: VpnState,
    // ... etc
    
    // Animation state
    animations: AnimationState,
    
    // Timing
    last_update: HashMap<String, Instant>,
}
```

### Update Frequencies
- Workspaces: 100ms (event-driven)
- Window title: 100ms
- Volume: 500ms
- Time: 1s
- WiFi: 5s
- VPN: 3s
- Battery: 30s
- Health: 60s

## Dependencies
- faelight-core (Theme)
- **faelight-icons (new shared crate)**
- Hack Nerd Font
- serde_json
- once_cell (for icon caching)

## Estimated Effort
**Phase 0:** 1-2 sessions (icon system foundation)
**Phase 1-4:** 9-12 sessions (1-2 hours each)
**Total:** 2-3 weeks

## Session Tracking
Progress tracked in `rust-tools/faelight-bar/PROGRESS.md`

Each session:
- Has clear, committable goal
- Updates PROGRESS.md
- Leaves working code
- Safe to pause/resume

## Benefits Beyond faelight-bar

### Ecosystem Impact
Once `faelight-icons` exists:
- faelight-launcher uses same icons for apps
- faelight-notify uses same status icons
- dot-doctor uses same health indicators
- CLI tools can use icons consistently
- Future TUIs automatically match

### User Customization
Users can theme entire system by editing `starship.toml`:
```toml
[faelight_icons]
lock = "🔒"  # Use emoji instead
unlock = "🔓"
# ... etc
```

No recompilation needed!

## Notes
This is the flagship tool - the centerpiece that ties Faelight Forest together.

The icon system addition transforms this from "just a bar refactor" into 
ecosystem-wide infrastructure that benefits all current and future tools.

Multi-session epic tracked across multiple chat sessions.

## Related Intents
- Intent 065: Tool audit quick wins (feeds into this)
- Future: faelight-launcher icon improvements
- Future: faelight-notify icon unification
