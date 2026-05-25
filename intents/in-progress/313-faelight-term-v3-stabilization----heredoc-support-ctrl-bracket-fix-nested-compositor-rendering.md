---
id: 313
title: "faelight-term v3 stabilization -- heredoc support, Ctrl+[ fix, nested compositor rendering"
status: in-progress
date: 2026-05-17
tags: faelight-term, stabilization, heredoc, kitty, compositor, wgpu, nested
depends_on: []
blocks: []
relates: [296, 308, 286]
---

## Why This Intent Exists

faelight-term v3 is the daily driver terminal.
It works well but has known stability gaps that affect daily use.
This intent formally tracks and closes them -- no workarounds, real fixes.

---

## Known Issues

### Issue 1: Heredoc not supported
- fsh heredoc syntax (`<< 'EOF'`) fails inside faelight-term
- Multi-line input is not handled by the PTY layer
- Workaround: use foot for heredoc commands
- Fix: implement multi-line PTY input handling in faelight-term

### Issue 2: Ctrl+[ not working as Escape
- In helix/evil-helix, Ctrl+[ should send ESC (0x1b)
- bracketleft keysym handler added but not firing
- Debug: check if ctrl modifier is set when bracketleft keysym arrives
- Possible fix: handle in key_press before the main match arm

### Issue 3: wgpu nested compositor rendering
- faelight-term uses wgpu/Vulkan for rendering
- When nested inside faelight-compositor (winit backend), wgpu fails
  ERROR_SURFACE_LOST_KHR on Vulkan surface
- Root cause: wgpu needs direct DRM/KMS access for Vulkan
- Fix path 1: use softbuffer/CPU fallback when nested detected
- Fix path 2: implement DRM backend in faelight-compositor (INT-308 Phase 4)
- Fix path 3: detect WAYLAND_DISPLAY=wayland-2 and switch to EGL

### Issue 4: fsh heredoc in term
- fsh splits multi-line commands -- this is an fsh bug not a term bug
- Tracked separately in fsh stabilization
- faelight-term should pass multi-line input as-is to PTY

---

## Gates

Phase 1 -- Ctrl+[ fix:
- [x] Ctrl+[ confirmed sending ESC to PTY -- helix exits insert mode correctly 2026-05-20
- [x] bracketleft + ctrl modifier correctly detected -- keysym=XK_bracketleft ctrl=true utf8=ESC confirmed via debug log 2026-05-20

Phase 2 -- Heredoc support:
- [x] Multi-line input (paste + heredoc) works in faelight-term -- bracketed paste protocol implemented 2026-05-17
[x] fsh heredoc commands work inside faelight-term -- python3 << EOF confirmed working

Phase 3 -- Nested compositor rendering:
- [ ] faelight-term renders correctly inside faelight-compositor
- [ ] No ERROR_SURFACE_LOST_KHR when nested
- [ ] Option: detect nested and use EGL/softbuffer fallback

Final:
- [ ] faelight-term is the sole daily driver terminal (foot retired)
- [x] helix + evil-helix fully functional inside faelight-term -- confirmed 2026-05-17
- [ ] faelight-term runs inside faelight-compositor cleanly

---

"The terminal that thinks must first work reliably.
Stability before intelligence.
The forest does not rush a tree to grow." 🌲
