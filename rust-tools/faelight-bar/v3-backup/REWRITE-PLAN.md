# Faelight-Bar v4.0.0 - Complete Rewrite Plan

## 🎯 Goals

**Primary:** Rock-solid stability, zero crashes
**Secondary:** Modern architecture, low CPU, beautiful UI

## 📊 Current Analysis (v3.0.0)

**What Works:**
- ✅ Click regions system
- ✅ Menu/launcher integration
- ✅ Profile switching
- ✅ VPN toggle (Mullvad)
- ✅ Volume control
- ✅ Faelight Forest colors
- ✅ Wayland/Sway integration

**Stability Issues:**
- ⚠️  Polling at 500ms (blocking)
- ⚠️  2 .unwrap() calls
- ⚠️  No error logging
- ⚠️  No crash recovery
- ⚠️  Synchronous I/O blocks UI
- ⚠️  No widget isolation
- ⚠️  Memory usage not monitored

**Missing Features:**
- ❌ Battery widget
- ❌ Network widget
- ❌ Lock status display
- ❌ Zone indicator
- ❌ Health status
- ❌ Event-driven updates

---

## 🏗️ New Architecture (v4.0.0)

### Component Structure
```
faelight-bar/
├── src/
│   ├── main.rs              # Entry point + event loop
│   ├── bar.rs               # Bar state management
│   ├── config.rs            # Configuration
│   ├── logger.rs            # Error logging
│   │
│   ├── widgets/             # Widget system
│   │   ├── mod.rs           # Widget trait
│   │   ├── clock.rs         # Time display
│   │   ├── battery.rs       # Battery status
│   │   ├── network.rs       # Network status
│   │   ├── vpn.rs           # Mullvad VPN
│   │   ├── volume.rs        # Volume/mute
│   │   ├── profile.rs       # Profile switcher
│   │   ├── lock.rs          # Lock status
│   │   ├── zone.rs          # Zone indicator
│   │   ├── health.rs        # System health
│   │   └── search.rs        # Search/launcher
│   │
│   ├── render/              # Rendering system
│   │   ├── mod.rs           # Core renderer
│   │   ├── colors.rs        # Faelight palette
│   │   └── icons.rs         # Nerd Font icons
│   │
│   └── events/              # Event handling
│       ├── mod.rs           # Event types
│       ├── click.rs         # Click handling
│       └── keyboard.rs      # Keyboard handling
│
├── REWRITE-PLAN.md          # This document
└── v3-backup/               # Current version backup
```

### Widget Trait System
```rust
pub trait Widget: Send {
    // Widget lifecycle
    fn name(&self) -> &'static str;
    fn update(&mut self) -> Result<(), WidgetError>;
    fn render(&self, ctx: &RenderContext) -> WidgetOutput;
    
    // Click handling
    fn click_region(&self) -> Option<(i32, i32)>;
    fn on_click(&mut self) -> Result<(), WidgetError>;
    
    // Error recovery
    fn error_state(&self) -> Option<&str>;
    fn reset(&mut self);
}
```

### Event-Driven Updates

**No More Polling!**
```rust
// Clock: Timer-based (1s)
tokio::time::interval(Duration::from_secs(1))

// Battery: udev events
tokio_udev::MonitorBuilder::new()?
    .match_subsystem("power_supply")?

// Network: netlink events  
rtnetlink::new_connection()?

// VPN: File watcher on Mullvad status
notify::Watcher for /var/run/mullvad

// Volume: PulseAudio events
libpulse_binding::context::subscribe

// Profile: File watcher
notify::Watcher for ~/.local/share/profile/current

// Lock: core-protect status file watcher
```

---

## 🎨 Visual Design (Modernized)

### Color Palette (Faelight Forest)
```rust
// Keep your current colors!
const BG: u32 = 0xFF1a1a2e;        // Dark background
const FG: u32 = 0xFFe8e8f0;        // Light text
const ACCENT: u32 = 0xFF4ecca3;    // Teal accent
const WARNING: u32 = 0xFFf39c12;   // Orange
const ERROR: u32 = 0xFFe74c3c;     // Red
const SUCCESS: u32 = 0xFF2ecc71;   // Green

// Modern additions
const BG_HOVER: u32 = 0xFF2a2a3e;  // Hover state
const SEPARATOR: u32 = 0xFF3a3a4e; // Widget separator
```

### Widget Layout
```
┌─────────────────────────────────────────────────────────────────┐
│ 🔍 Search  │  🌲 Zone  │  🏥 Health  │  🔒 VPN  │  🔊 Vol  │  ⚡ Profile  │  🔐 Lock  │  🔋 Batt  │  📶 Net  │  🕐 14:35 │
└─────────────────────────────────────────────────────────────────┘
   ^clickable   ^indicator  ^indicator  ^clickable ^clickable ^clickable   ^clickable ^indicator ^indicator  ^clock
```

### Icons (Nerd Font)
```rust
// VPN states
const VPN_CONNECTED: &str = "󰦝";     // 
const VPN_DISCONNECTED: &str = "󰅛";  //

// Lock states  
const LOCKED: &str = "";           // 
const UNLOCKED: &str = "";         // 

// Profile icons
const PROFILE_DEFAULT: &str = "";  // 
const PROFILE_GAMING: &str = "󰊗";   // 󰊗
const PROFILE_WORK: &str = "";     // 
const PROFILE_LOWPOWER: &str = "󰾆";// 󰾆

// Battery states
const BATTERY_FULL: &str = "󰁹";    // 󰁹
const BATTERY_CHARGING: &str = "󰂄";// 󰂄
const BATTERY_LOW: &str = "󰁺";     // 󰁺

// Network states
const WIFI_ON: &str = "󰖩";         // 󰖩
const WIFI_OFF: &str = "󰖪";        // 󰖪
const ETHERNET: &str = "󰈀";        // 󰈀

// Volume states
const VOLUME_HIGH: &str = "󰕾";     // 󰕾
const VOLUME_MUTE: &str = "󰖁";     // 󰖁

// Zones
const ZONE_RUST: &str = "🦀";
const ZONE_FOREST: &str = "🌲";
const ZONE_CODE: &str = "💻";
const ZONE_LEARNING: &str = "📚";
const ZONE_HOME: &str = "🏠";

// Health
const HEALTH_GOOD: &str = "";     // 
const HEALTH_WARNING: &str = "";  // 
const HEALTH_ERROR: &str = "";    // 
```

---

## 🔧 Implementation Phases

### Phase 1: Foundation (Session 1)
**Goal:** Core architecture + basic widgets

**Tasks:**
1. ✅ Backup current v3.0.0
2. ✅ Create new Cargo workspace
3. ✅ Set up error logging system
4. ✅ Implement Widget trait
5. ✅ Port rendering system
6. ✅ Basic clock widget (test)
7. ✅ Ensure it compiles and runs

**Deliverable:** Minimal working bar with clock

---

### Phase 2: Core Widgets (Session 2)
**Goal:** Port existing functionality

**Tasks:**
1. ✅ VPN widget (Mullvad, clickable)
2. ✅ Volume widget (clickable)
3. ✅ Profile widget (clickable)
4. ✅ Search/launcher widget
5. ✅ Click region system
6. ✅ Test all interactions

**Deliverable:** Feature parity with v3.0.0

---

### Phase 3: New Widgets (Session 3)
**Goal:** Add missing features

**Tasks:**
1. ✅ Battery widget
2. ✅ Network widget
3. ✅ Lock status widget
4. ✅ Zone display widget
5. ✅ Health status widget
6. ✅ Modern color transitions

**Deliverable:** Complete widget set

---

### Phase 4: Event System (Session 4)
**Goal:** Replace polling with events

**Tasks:**
1. ✅ Set up tokio runtime
2. ✅ Battery events (udev)
3. ✅ Network events (netlink)
4. ✅ File watchers (profile, lock, vpn)
5. ✅ Volume events (PulseAudio)
6. ✅ Performance testing

**Deliverable:** Event-driven, low CPU

---

### Phase 5: Stability (Session 5)
**Goal:** Production hardening

**Tasks:**
1. ✅ Remove all .unwrap() calls
2. ✅ Comprehensive error logging
3. ✅ Widget crash isolation
4. ✅ Memory leak testing
5. ✅ Systemd auto-restart config
6. ✅ Stress testing (24h uptime)

**Deliverable:** Rock-solid stability

---

### Phase 6: Polish (Session 6)
**Goal:** Production ready

**Tasks:**
1. ✅ README (500+ lines)
2. ✅ CHANGELOG
3. ✅ Configuration file
4. ✅ Theme customization
5. ✅ Performance benchmarks
6. ✅ Documentation

**Deliverable:** v4.0.0 release

---

## 📦 Dependencies (Additions)
```toml
[dependencies]
# Existing
smithay-client-toolkit = "0.19"
wayland-client = "0.31"

# New for stability
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# New for events
tokio-udev = "0.9"
rtnetlink = "0.14"
notify = "6.0"
libpulse-binding = "2.28"

# Existing utilities
chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

---

## 🎯 Success Criteria

**Stability:**
- ✅ Zero crashes in 7-day uptime test
- ✅ Zero .unwrap() calls
- ✅ All errors logged
- ✅ Graceful widget degradation

**Performance:**
- ✅ <1% CPU usage (idle)
- ✅ <50MB RAM
- ✅ <10ms render time
- ✅ Instant click response

**Features:**
- ✅ All v3.0.0 features
- ✅ Battery widget
- ✅ Network widget
- ✅ Lock status
- ✅ Zone display
- ✅ Health indicator

**Code Quality:**
- ✅ Modular architecture
- ✅ Comprehensive tests
- ✅ Full documentation
- ✅ Clean separation of concerns

---

## 🚀 Next Steps

**Immediate:**
1. Review this plan
2. Backup v3.0.0
3. Create v4 branch
4. Start Phase 1 (next session)

**Timeline:**
- Phase 1-2: Week 1 (foundation + core)
- Phase 3-4: Week 2 (widgets + events)
- Phase 5-6: Week 3 (stability + polish)

**Total:** ~6 sessions (3 weeks)

---

## 💡 Notes

- Keep v3.0.0 running during development
- Test each phase thoroughly
- Get feedback after each phase
- Don't rush stability testing

This is a critical component - better to take time and do it right!

---

**Ready to start Phase 1?** 🚀
