---
id: 138
date: 2026-03-18
type: future
title: "faelight-compositor v2 — EGL/OpenGL First Real Frame"
status: planned
tags: [compositor, egl, opengl, rendering, smithay, drm, v12]
version: 12.0.0
priority: high
depends_on: [109]
---

## Where We Are (v0.1.0)

Session 5 was a landmark — forest green painted on real hardware.
But it was a dumb buffer — raw pixel fill, no GPU rendering.
```
v0.1.0 accomplished:
  ✅ DRM device opened (AMD Radeon 780M)
  ✅ GBM device created
  ✅ eDP connector found (2560x1600@165Hz)
  ✅ CRTC selected (Handle 363)
  ✅ Dumb buffer created and filled (#11140f)
  ✅ set_crtc SUCCESS — pixels on screen
```

v2.0.0 goal: replace the dumb buffer with a real EGL/OpenGL
rendering pipeline. GPU-accelerated. Shader-capable.
The foundation for everything visual the forest will ever do.

## Why This Matters

A dumb buffer is CPU-rendered — every pixel written by the CPU.
EGL/OpenGL uses the GPU — 780M cores doing the work.

This unlocks:
```
Smooth animations     — GPU handles compositing
Blur effects          — shader-based
Custom visual effects — GLSL shaders
Text rendering        — GPU-accelerated fonts
Window compositing    — GPU blending
Future: custom shaders unique to Faelight Forest
```

## The Rendering Stack
```
DRM device (card2)
  └── GBM device
       └── EGL display
            └── EGL context (OpenGL ES 3.2)
                 └── EGL surface (GBM-backed)
                      └── OpenGL framebuffer
                           └── Render frame
                                └── Present (page flip)
```

## Session Plan

### Session 1 — EGL Setup
Goal: Create EGL display and context from GBM device
```rust
// Key calls:
egl.get_display(gbm_device)
egl.initialize(display)
egl.choose_config(display, attribs)
egl.create_context(display, config, attribs)
egl.create_window_surface(display, config, gbm_surface)
egl.make_current(display, surface, surface, context)
```

Success: EGL context created, no errors.

### Session 2 — OpenGL Clear Color
Goal: Clear screen to forest green using OpenGL
```rust
gl::ClearColor(0.067, 0.078, 0.059, 1.0); // #11140f
gl::Clear(gl::COLOR_BUFFER_BIT);
egl.swap_buffers(display, surface);        // present
```

Success: Forest green rendered via GPU (not CPU).
Visually identical to dumb buffer but GPU-rendered.

### Session 3 — Triangle
Goal: Render the classic "Hello GPU" — a colored triangle
```glsl
// vertex shader
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
// fragment shader — forest green
void main() {
    FragColor = vec4(0.639, 0.890, 0.420, 1.0); // #a3e36b
}
```

Success: Forest green triangle on screen. GPU confirmed working.

### Session 4 — Texture Rendering
Goal: Render a Wayland client surface as a texture

This is where the compositor becomes real — clients draw to
wl_buffer, compositor samples as texture, renders to screen.

Success: A Wayland client window visible through faelight-compositor.

### Session 5 — Page Flip & VBlank
Goal: Proper frame synchronization
```rust
// Queue frame, wait for vblank event
drm.page_flip(crtc, framebuffer, PageFlipFlags::EVENT)
// Handle DRM vblank event
// Present next frame at correct time
```

Success: Tear-free rendering at 165Hz.

## Smithay Integration

Smithay has a full EGL/GBM/OpenGL pipeline in:
```
smithay::backend::egl::*
smithay::backend::renderer::gles::*
smithay::backend::drm::compositor::DrmCompositor
```

DrmCompositor (from Session 4 exploration) wraps the entire
render pipeline. Sessions 1-3 build understanding.
Session 4 uses DrmCompositor properly.

## AMD Radeon 780M Notes
```
GPU:        AMD Radeon 780M (radeonsi, phoenix)
Driver:     Mesa 26.0.2 / ACO
DRM:        3.64
OpenGL ES:  3.2
EGL:        1.5
Connector:  eDP (embedded display)
Resolution: 2560x1600
Refresh:    165Hz
CRTC:       Handle(363)
GBM format: XRGB8888 confirmed working
```

## Success Criteria

- ⬜ EGL display created from GBM device
- ⬜ OpenGL ES 3.2 context created
- ⬜ EGL surface created (GBM-backed)
- ⬜ Forest green cleared via OpenGL (GPU-rendered)
- ⬜ Triangle rendered in forest green (#a3e36b)
- ⬜ Wayland client surface rendered as texture
- ⬜ Page flip with VBlank synchronization
- ⬜ 165Hz tear-free rendering

## The Phrase

**"The dumb buffer was the forest touching the screen.
EGL is the forest seeing through the screen.
OpenGL is the forest painting with light."**

---
*"Session 5 proved the forest can reach the hardware.
v2.0.0 proves the forest can render with it."* 🌲
