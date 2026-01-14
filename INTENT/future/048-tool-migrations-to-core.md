---
id: 048
date: 2026-01-14
type: future
title: "Tool Migrations to faelight-core"
status: planned
tags: [architecture, v7.0, refactor, migration]
---

## Overview

Migrate bar, menu, and notify to use faelight-core (Intent 047).

**Goal:** Eliminate duplication, achieve 70%+ performance gain, validate core API

**Week 2 of v7.0.0**

## Migration Order (By Complexity)

1. **faelight-bar** → Simplest, already stable v1.0
2. **faelight-menu** → Small surface area, good test case
3. **faelight-notify** → Medium complexity + architecture changes

**Launcher migrates separately (Intent 049) due to provider refactor**

---

## Migration 1: faelight-bar v2.0 (1 day)

### Current State
- 414 lines
- Manual glyph rasterization
- SHM buffer recreation
- Hardcoded colors

### After Migration
- ~250 lines (40% reduction)
- Uses `GlyphCache`
- Uses `Canvas` API
- Uses `Theme`
- **Result:** Bar logic only, rendering abstracted

### Changes Required

**Cargo.toml:**
```toml
[dependencies]
faelight-core = { path = "../faelight-core" }
# Remove: fontdue (now in core)
```

**main.rs changes:**
```rust
use faelight_core::{Canvas, GlyphCache, Theme, theme};

struct BarState {
    canvas: Canvas,
    glyph_cache: GlyphCache,
    theme: Theme,
    // ... existing fields
}

// Replace all draw_rect/draw_text with:
fn draw(&mut self) {
    self.canvas.clear(self.theme.bg_primary);
    
    // Time section
    let x = self.canvas.draw_text(
        &mut self.glyph_cache,
        &self.time,
        10,
        (BAR_HEIGHT - self.theme.font_size_normal as u32) / 2,
        self.theme.font_size_normal,
        self.theme.text_primary,
    );
    
    // ... rest of bar
}
```

**Benefits:**
- 70%+ CPU reduction
- Consistent theming
- Cleaner code
- No regressions (v1.0 is stable)

---

## Migration 2: faelight-menu v2.0 (1 day)

### Current State
- Manual glyph rasterization
- Hardcoded colors
- No confirmation on dangerous actions ⚠️

### After Migration
- Uses `GlyphCache`
- Uses `Canvas` API  
- Uses `Theme`
- **NEW: Dangerous action confirmation** ✅

### Changes Required

**Add confirmation state:**
```rust
struct MenuState {
    canvas: Canvas,
    glyph_cache: GlyphCache,
    theme: Theme,
    selected: usize,
    confirming: bool,  // NEW
    // ... existing fields
}

enum MenuAction {
    Lock,       // Safe
    Logout,     // Safe
    Suspend,    // Safe
    Reboot,     // DANGEROUS
    Shutdown,   // DANGEROUS
}

impl MenuAction {
    fn is_dangerous(&self) -> bool {
        matches!(self, Self::Reboot | Self::Shutdown)
    }
    
    fn color(&self, theme: &Theme, confirming: bool) -> u32 {
        if self.is_dangerous() && confirming {
            theme.danger  // RED when confirming
        } else if self.is_dangerous() {
            theme.warning // YELLOW normally
        } else {
            theme.text_primary
        }
    }
}
```

**Confirmation flow:**
```rust
fn handle_enter(&mut self) {
    let action = self.actions[self.selected];
    
    if action.is_dangerous() {
        if !self.confirming {
            // First Enter: Request confirmation
            self.confirming = true;
            return;
        }
        // Second Enter: Execute
    }
    
    // Execute action
    self.execute(action);
    self.exit();
}

fn handle_key(&mut self, key: Keysym) {
    match key {
        Keysym::Escape => {
            if self.confirming {
                self.confirming = false;  // Cancel confirmation
            } else {
                self.exit();
            }
        }
        Keysym::Up | Keysym::k => {
            self.selected = self.selected.saturating_sub(1);
            self.confirming = false;  // Reset on navigation
        }
        // ...
    }
}
```

**Visual feedback:**
```rust
fn draw(&mut self) {
    // Title
    self.canvas.draw_text(
        &mut self.glyph_cache,
        "Power Menu",
        x, y,
        self.theme.font_size_large,
        self.theme.text_primary,
    );
    
    // Actions
    for (i, action) in self.actions.iter().enumerate() {
        let is_selected = i == self.selected;
        let is_confirming = is_selected && self.confirming;
        
        let bg = if is_selected {
            if is_confirming {
                self.theme.danger  // RED background
            } else {
                self.theme.accent  // CYAN background
            }
        } else {
            self.theme.bg_secondary
        };
        
        // Draw row background
        self.canvas.draw_rect(x, y, width, height, bg);
        
        // Draw action text
        let text = if is_confirming && action.is_dangerous() {
            "Press Enter again to confirm"
        } else {
            action.label()
        };
        
        self.canvas.draw_text(
            &mut self.glyph_cache,
            text,
            x + 10, y + 10,
            self.theme.font_size_normal,
            action.color(&self.theme, is_confirming),
        );
    }
}
```

**Benefits:**
- 70%+ CPU reduction
- **CRITICAL: Prevents 2am disasters** ⚠️
- Aligns with intent-guard philosophy
- Visual clarity (RED = danger)

---

## Migration 3: faelight-notify v2.0 (2 days)

### Current State
- Manual glyph rasterization
- D-Bus and UI mixed
- Missing protocol features

### After Migration
- Uses `GlyphCache` + `Canvas`
- **Channel-based architecture** (D-Bus → UI events)
- **Protocol correctness** (replaces_id, CloseNotification)
- **Fade animations**

### Architecture Changes

**Event-based system:**
```rust
use std::sync::mpsc::{channel, Sender, Receiver};

enum NotifyEvent {
    Show(Notification),
    Replace { id: u32, notification: Notification },
    Close(u32),
    Tick,  // For animations
}

struct NotifyState {
    canvas: Canvas,
    glyph_cache: GlyphCache,
    theme: Theme,
    notifications: Vec<Notification>,
    event_rx: Receiver<NotifyEvent>,
}

struct Notification {
    id: u32,
    summary: String,
    body: String,
    created: Instant,
    timeout: Duration,
    alpha: f32,  // For fade animation
}
```

**D-Bus handler sends events:**
```rust
impl OrgFreedesktopNotifications for NotifyHandler {
    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, Variant<Box<dyn RefArg>>>,
        expire_timeout: i32,
    ) -> Result<u32, dbus::MethodErr> {
        let id = if replaces_id != 0 {
            // Replace existing
            self.event_tx.send(NotifyEvent::Replace {
                id: replaces_id,
                notification: Notification::new(replaces_id, summary, body, expire_timeout),
            }).ok();
            replaces_id
        } else {
            // New notification
            let id = self.next_id;
            self.next_id += 1;
            self.event_tx.send(NotifyEvent::Show(
                Notification::new(id, summary, body, expire_timeout)
            )).ok();
            id
        };
        
        Ok(id)
    }
    
    fn close_notification(&mut self, id: u32) -> Result<(), dbus::MethodErr> {
        self.event_tx.send(NotifyEvent::Close(id)).ok();
        Ok(())
    }
    
    fn get_capabilities(&mut self) -> Result<Vec<String>, dbus::MethodErr> {
        Ok(vec![
            "body".into(),
            "actions".into(),
            "persistence".into(),
        ])
    }
}
```

**UI loop processes events:**
```rust
fn handle_event(&mut self, event: NotifyEvent) {
    match event {
        NotifyEvent::Show(notif) => {
            self.notifications.push(notif);
            self.show_surface();
        }
        NotifyEvent::Replace { id, notification } => {
            if let Some(existing) = self.notifications.iter_mut().find(|n| n.id == id) {
                *existing = notification;
            }
        }
        NotifyEvent::Close(id) => {
            self.notifications.retain(|n| n.id != id);
            if self.notifications.is_empty() {
                self.hide_surface();
            }
        }
        NotifyEvent::Tick => {
            self.update_animations();
        }
    }
}

fn update_animations(&mut self) {
    let now = Instant::now();
    
    for notif in &mut self.notifications {
        let elapsed = now.duration_since(notif.created);
        let remaining = notif.timeout.saturating_sub(elapsed);
        
        if remaining < Duration::from_millis(300) {
            // Fade out in last 300ms
            notif.alpha = remaining.as_millis() as f32 / 300.0;
        } else {
            notif.alpha = 1.0;
        }
    }
    
    // Remove expired
    self.notifications.retain(|n| {
        now.duration_since(n.created) < n.timeout
    });
}
```

**Fade rendering:**
```rust
fn draw(&mut self) {
    let notif = &self.notifications[0];
    
    // Background with alpha
    let bg_alpha = (notif.alpha * 255.0) as u32;
    let bg = apply_alpha(self.theme.bg_primary, bg_alpha);
    self.canvas.draw_rect(0, 0, width, height, bg);
    
    // Text with alpha
    let text_alpha = (notif.alpha * 255.0) as u32;
    let text_color = apply_alpha(self.theme.text_primary, text_alpha);
    
    self.canvas.draw_text(
        &mut self.glyph_cache,
        &notif.summary,
        x, y,
        self.theme.font_size_normal,
        text_color,
    );
}

fn apply_alpha(color: u32, alpha: u32) -> u32 {
    (color & 0x00FFFFFF) | ((alpha & 0xFF) << 24)
}
```

**Benefits:**
- 70%+ CPU reduction
- Clean architecture (no mutex in UI)
- Protocol correctness
- Smooth animations
- No D-Bus blocking

---

## Testing Strategy

### Per-Tool Testing

**After each migration:**
1. Visual regression test (screenshot comparison)
2. Performance test (CPU usage measurement)
3. Functionality test (all features work)
4. Memory test (no leaks)

**bar:**
- All modules display correctly
- Clicks work
- No visual changes

**menu:**
- All actions listed
- Navigation works
- Confirmation prevents accidents
- Dangerous actions show RED

**notify:**
- Notifications appear
- replaces_id works
- CloseNotification works
- Fade animation smooth
- No D-Bus errors

### Integration Testing

**All tools together:**
```bash
# Start all
faelight-bar &
faelight-notify &

# Test notify
notify-send "Test" "Message"
notify-send -r 1 "Replace" "Updated"

# Test menu
faelight-menu
# Navigate, test confirmation

# Measure CPU
top -b -n 100 | grep faelight
```

---

## Success Criteria

**Per-tool:**
- [ ] Compiles without warnings
- [ ] All features working
- [ ] No visual regressions
- [ ] 70%+ CPU reduction measured
- [ ] No memory leaks

**menu specific:**
- [ ] Dangerous actions require double-Enter
- [ ] RED highlight when confirming
- [ ] Escape cancels confirmation
- [ ] Navigation resets confirmation

**notify specific:**
- [ ] replaces_id works
- [ ] CloseNotification works
- [ ] Fade animation smooth
- [ ] Channel architecture clean
- [ ] Capabilities correct

**Integration:**
- [ ] All tools run simultaneously
- [ ] No conflicts
- [ ] Consistent theming
- [ ] 100% health maintained

---

## Rollback Plan

If migration causes issues:

1. Keep v1.x binaries in `scripts/v1/`
2. Git tag before migration: `v6.9.0-pre-core`
3. Each tool migrates in separate branch
4. Merge only after testing passes

**Safety first. No rushing.**

---

## Timeline

**Week 2 of v7.0.0:**
- Day 1: faelight-bar v2.0
- Day 2: faelight-menu v2.0
- Day 3-4: faelight-notify v2.0
- Day 5: Integration testing + polish

---

_"Migrate deliberately. Test thoroughly. Ship confidently."_ 🌲
