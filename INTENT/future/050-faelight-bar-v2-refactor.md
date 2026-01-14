---
id: 050
date: 2026-01-14
type: future
title: "faelight-bar v2.0 - Architectural Refactor"
status: planned
tags: [rust, bar, architecture, performance, v7.0]
---

## The Vision

Explore architectural improvements to faelight-bar that may improve performance, maintainability, and visual polish. These are IDEAS to evaluate, not mandates - each must prove its value against 0-Core philosophy.

**Core Question:** Can we beat waybar's performance while staying simpler to understand?

---

## 10 Potential Improvements (To Evaluate)

### 1. Cached Model Architecture (HIGH VALUE)

**Problem:** Every `draw()` calls external commands synchronously on Wayland thread:
- `swaymsg`
- `iwctl`
- `wpctl`
- `mullvad`
- filesystem reads
- git commands

**Proposed Solution:**
```rust
struct BarModel {
    profile: String,
    workspaces: Vec<i32>,
    active_ws: i32,
    window_title: String,
    volume: (u8, bool),
    wifi: (bool, String),
    battery: (u8, bool),
    vpn: (bool, String),
    health: u8,
    locked: bool,
    time: String,
    last_update: Instant,
}
```

- Update model every 500-1000ms (or staggered)
- `draw()` becomes pure rendering
- Pointer events never block

**Philosophy Check:** Does this hide system state or make it clearer?

**Decision:** Prototype first, measure impact

---

### 2. JSON Parsing Instead of String Search

**Problem:** Brittle substring searches:
```rust
if resp.contains("\"num\":3") { ... }
```

Breaks on:
- JSON formatting changes
- Named workspaces
- Scratchpad
- Multi-output

**Proposed Solution:**
```rust
let v: serde_json::Value = serde_json::from_str(&resp)?;
for ws in v.as_array().unwrap() {
    let num = ws["num"].as_i64().unwrap_or(0);
    let focused = ws["focused"].as_bool().unwrap_or(false);
}
```

**Benefits:**
- More robust
- Easier to extend (icons per app, marks, urgency)

**Philosophy Check:** Does this increase understanding or hide behavior?

---

### 3. Glyph Cache (HIGH VALUE)

**Problem:** `font.rasterize()` called every frame, every glyph.

**Proposed Solution:**
```rust
HashMap<(char, u32), (Metrics, Vec<u8>)>
```

Keyed by `(char, font_size)`.

**Measure text width correctly:**
```rust
fn text_width(font: &Font, text: &str, size: f32) -> i32 {
    text.chars()
        .map(|c| font.metrics(c, size).advance_width as i32)
        .sum()
}
```

Fixes centering and proportional fonts.

**Philosophy Check:** Pure optimization, no behavior change

---

### 4. Semantic Click Handling

**Problem:** Current approach:
```rust
Vec<(x_start, x_end, String)>
```

Issues:
- No Y bounds
- String matching
- Easy to break layout

**Proposed Solution:**
```rust
struct ClickRegion {
    rect: (i32, i32, i32, i32),
    action: ClickAction,
}

enum ClickAction {
    ToggleVpn,
    ToggleMute,
    CycleProfile,
}
```

**Benefits:**
- Safer
- Faster
- Compiler-checked
- Easier to add hover effects

**Philosophy Check:** Makes intent explicit

---

### 5. Module-Level Update Frequencies

**Problem:** Everything updates at 500ms.

**Proposed Solution:**

| Module | Interval |
|--------|----------|
| Clock | 1s |
| Volume | Event-based or 500ms |
| Workspaces | Event-based |
| VPN/WiFi | 3-5s |
| Battery | 10-30s |
| Health | 30-60s |
```rust
if now - last_battery_update > Duration::from_secs(30) {
    update_battery();
}
```

**Philosophy Check:** Reduces unnecessary polling = respects system resources

---

### 6. Wayland / SCTK Improvements

**A. Respect scale factor**

Currently ignored:
- `scale_factor_changed`

Should:
- Multiply font size
- Multiply `BAR_HEIGHT`
- Adjust buffer size

**B. Avoid recreating buffers every frame**

Currently:
```rust
self.pool.create_buffer(...)
```

Better:
- Keep one buffer
- Recreate only when size changes

**Philosophy Check:** Proper Wayland citizenship

---

### 7. External Command Hygiene

**A. Avoid shelling out when possible**

Examples:
- Battery → upower DBus
- Volume → PipeWire via libspa/wireplumber
- WiFi → netlink or iw ioctl

**B. Timeout external commands**

A hung `mullvad status` will freeze the bar.
```rust
Command::new(...)
    .stdout(Stdio::piped())
    .spawn()
```

Or worker thread with timeout.

**Philosophy Check:** Makes failure modes explicit

---

### 8. Visual Polish Ideas

**Low cost, high impact:**
- Underline active workspace instead of color only
- Animated accent bar when profile changes
- Battery bar (▁▂▃▄▅▆▇█)
- Icons instead of text (already using Nerd Font)
- Urgent workspace color

Example:
```
󰖩 VPN   42%  󰁹 87%+  
```

**Philosophy Check:** Clarity over aesthetics, but clarity can be beautiful

---

### 9. Configuration Without Recompile

**Problem:** Hardcoded values everywhere.

**Proposed Solution:**

Minimal TOML config:
```toml
bar_height = 32
font_size = 14
update_rate_ms = 500
```

Load once at startup.

**Philosophy Check:** User control over defaults = good

---

### 10. Overall Architecture Assessment

Current strengths:
- Manual layer-shell
- Pointer handling
- Custom rendering
- Font rasterization
- Clickable regions

**Top 5 Priority (If Proven Valuable):**
1. Cached model (#1)
2. JSON parsing (#2)
3. Glyph cache (#3)
4. Module update intervals (#5)
5. Typed click actions (#4)

---

## Philosophy Alignment

**Key Question:** Does each change:
- Increase understanding? ✅
- Reduce hidden behavior? ✅
- Make failures explicit? ✅
- Respect user control? ✅

**OR:**

- Hide complexity? ❌
- Add magic? ❌
- Reduce visibility? ❌

**Evaluation Process:**
1. Prototype each change
2. Measure impact (CPU, latency, code clarity)
3. Document tradeoffs
4. Decide: keep, iterate, or reject

---

## Success Criteria

**Must prove:**
- [ ] Measurably faster (benchmarks)
- [ ] Easier to understand (subjective but honest)
- [ ] More maintainable (fewer brittle patterns)
- [ ] No regressions (functionality preserved)

**Nice to have:**
- [ ] Beats waybar in performance
- [ ] Matches or exceeds waybar in features
- [ ] Maintains 0-Core philosophy

---

## Out of Scope

- ❌ Wayland protocols we don't need
- ❌ Features for features' sake
- ❌ Complexity without benefit

---

## Related Intents

- Intent 047: faelight-core (shared foundation)
- Intent 046: v7.0.0 architectural milestone

---

_"Performance matters. Clarity matters more. Both is best."_ 🌲
