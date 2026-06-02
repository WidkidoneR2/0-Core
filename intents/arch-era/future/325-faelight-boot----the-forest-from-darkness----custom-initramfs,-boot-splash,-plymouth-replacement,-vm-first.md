---
id: 325
title: "faelight-boot -- The Forest from Darkness"
status: planned
date: 2026-05-20
tags: [boot, initramfs, splash, plymouth, dracut, mkinitcpio, QEMU, vm, kernel, framebuffer]
---
---
THE PREMISE

Every piece of the forest stack is owned.
The shell. The terminal. The bar. The compositor. The login screen.
The file manager. The notification system.

But the machine still wakes up wearing someone else's clothes.

The moment power is pressed: darkness.
Then: firmware logo.
Then: a spinning Plymouth spinner that belongs to no one.
Then: faelight-login appears -- and suddenly it is ours.

faelight-boot closes the gap.

The machine wakes into the forest from the very first pixel.
No Plymouth. No spinner. No generic splash.
A single green pulse from the center of the screen.
The forest becoming conscious.
By the time it reaches the edges, faelight-login is ready.

This is the most ambitious visual statement in the entire stack.
The forest owns the machine from power-on.
---
WHAT HAPPENS BETWEEN POWER AND LOGIN

Understanding the boot sequence before changing anything:

  Power → UEFI firmware (cannot touch this)
  UEFI → systemd-boot (bootloader -- do not touch, it works)
  systemd-boot → kernel + initramfs (THIS IS WHERE WE WORK)
  initramfs → mounts root filesystem
  initramfs → hands off to systemd (PID 1)
  systemd → starts services including greetd
  greetd → launches faelight-login (already ours)

The window of opportunity:
  initramfs phase: the kernel is running, hardware is initialized,
  but the root filesystem is not yet mounted.
  This is where Plymouth lives.
  This is where faelight-boot lives.

What initramfs can do:
  Display graphics via kernel framebuffer (DRM/KMS -- no Wayland yet)
  Play audio (if audio drivers are in initramfs -- skip for now)
  Run arbitrary Rust binaries (statically compiled)
  Unlock encrypted disks (LUKS -- already handled, do not break this)
  Show progress as the system initializes

What initramfs cannot do:
  Run Wayland (no compositor yet)
  Use systemd services (not started yet)
  Access the internet (no network configured yet)

The rendering layer:
  Direct framebuffer: /dev/fb0 or DRM/KMS via /dev/dri/card0
  Resolution: whatever the display reports before Wayland
  Format: usually XRGB8888 or ARGB8888 at native resolution
  No GPU acceleration: just pixel writes to framebuffer memory
  This is how Plymouth works. We do the same, but forest-green.
---
THE BOOT ANIMATION

The vision:

  t=0ms:   Screen is dark. Machine just started.
  t=200ms: A single point of forest green appears at screen center.
           Exact color: #00E580 (the forest's active green)
  t=200ms-800ms: The point expands as a circle.
           The edge of the circle glows bright, fades inward.
           The expansion speed matches the feeling of waking.
           Not instant. Not slow. Deliberate.
  t=800ms: The circle reaches the screen edges and fades.
           Left behind: subtle forest green tint on the screen center.
  t=800ms+: If the system needs more time: a second smaller pulse.
            Amplitude decreases each pulse. The forest is breathing.
  t=final:  faelight-login fades in from black.
            The splash fades out simultaneously.
            The transition is invisible -- one continuous wake.

The framebuffer animation:
  Each frame: iterate over all pixels.
  For each pixel: calculate distance from screen center.
  Color = f(distance, time) -- a wave function.
  The wave travels outward at constant speed.
  At the wave front: full brightness green.
  Behind the wave front: exponential decay to dark.
  Ahead of the wave front: black.

  This is pure math. No image files. No assets.
  The animation is generated entirely from the wave equation.
  It looks the same at any resolution.
  It is impossible to copyright-infringe -- it is a formula.

The Rust implementation:
  Binary: faelight-splash
  Statically compiled (musl target for initramfs compatibility)
  Opens /dev/dri/card0 or /dev/fb0
  Gets display dimensions via EDID or ioctl
  Runs animation loop at ~30fps (33ms per frame)
  Receives signal from mkinitcpio hook when root is mounted
  On signal: begins fade-out transition
  Exits: faelight-login takes over display
---
REFERENCE ARCHITECTURES TO STUDY

Plymouth (what we replace):
  Language: C
  Source: gitlab.freedesktop.org/plymouth/plymouth
  Key files:
    src/main.c              -- boot process integration
    src/plugins/renderers/  -- framebuffer and DRM renderers
    src/plugins/splash/     -- splash themes
  Study: how Plymouth hooks into initramfs, how it draws to framebuffer
  Key insight: Plymouth uses DRM/KMS for modern rendering
               Falls back to /dev/fb0 for compatibility
  Study specifically: the DRM renderer code -- this is our model

drm-framebuffer rendering in Rust:
  Crate: drm-rs (github.com/Smithay/drm-rs)
  Already in workspace via faelight-compositor
  Study: how to open /dev/dri/card0, set mode, write pixels
  This is the same DRM code as compositor but simpler (no Wayland)

mkinitcpio (Arch Linux initramfs builder):
  Documentation: wiki.archlinux.org/title/Mkinitcpio
  Key concepts:
    Hooks: scripts that run during initramfs build and boot
    BINARIES: array of binaries to include in initramfs
    FILES: array of files to include
    MODULES: kernel modules to preload
  Study: how to write a custom hook
  Key file: /etc/mkinitcpio.conf (we have a .pacnew to review)
  Custom hook structure:
    /etc/initcpio/install/faelight-splash  -- build hook (adds files)
    /etc/initcpio/hooks/faelight-splash    -- runtime hook (runs binary)

Bootsplash alternatives (study for ideas):
  fbsplash: legacy framebuffer splash, educational
  systemd-boot: already our bootloader, study plymouth integration
  refind-theme: UEFI-level splash (before kernel -- not our target)

QEMU/libvirt (VM environment for testing):
  Never used before. Start simple.
  Install: qemu-base libvirt virt-manager
  Create: VM with Arch Linux, same disk layout as real machine
  Boot the VM: test initramfs changes without risking real machine
  Key commands:
    qemu-system-x86_64 -enable-kvm -m 4G -drive file=arch.qcow2
    virsh console arch-test
  The VM is the safety net. Every initramfs change is tested there first.
  Only after VM confirms: apply to real machine.
---
ARCHITECTURE

Components:

  faelight-splash binary:
    Language: Rust, statically compiled with musl
    Target: x86_64-unknown-linux-musl
    Size target: < 500KB (initramfs space is limited)
    Dependencies: none external (pure Rust, direct syscalls)
    Rendering: DRM/KMS via drm-rs OR direct /dev/fb0 mmap
    Animation: wave equation, float math, no_std compatible
    IPC: listens on a Unix socket or reads a file for "ready" signal
    Exit: clean, releases framebuffer before faelight-login takes over

  mkinitcpio hook (install hook):
    /etc/initcpio/install/faelight-splash
    Adds to initramfs:
      /usr/bin/faelight-splash (the binary)
      Any required kernel modules (DRM, display driver)
    Declares position: after base, before filesystems

  mkinitcpio hook (runtime hook):
    /etc/initcpio/hooks/faelight-splash
    Runs during boot:
      Starts faelight-splash in background
      Continues normal boot process (LUKS unlock, root mount)
      On root mounted: signals faelight-splash to fade out
      Waits for splash to exit before handing off to init

  Coordination with faelight-login:
    faelight-login detects if splash is still running.
    If yes: faelight-login waits for splash fadeout signal.
    If no: faelight-login renders immediately.
    This prevents a visible flash between splash and login.

  LUKS compatibility:
    If LUKS encryption is active: splash must yield the screen
    for the password prompt, then resume after unlock.
    The wave pauses at the LUKS prompt.
    After unlock: wave resumes.
    This requires coordination with the encrypt hook in mkinitcpio.

VM testing workflow:
  1. Build faelight-splash binary
  2. Copy to VM
  3. Update VM initramfs: mkinitcpio -p linux (inside VM)
  4. Reboot VM: watch the splash
  5. Iterate until perfect
  6. Copy hook and binary to real machine
  7. Update real initramfs: mkinitcpio -p linux (on real machine)
  8. Test real boot
  9. If broken: boot recovery USB, restore previous initramfs
---
THE WAVE EQUATION

The animation is mathematically defined.
No assets. No images. Pure computation.

  For each pixel at position (px, py):
  For each frame at time t (seconds):

    cx = screen_width / 2
    cy = screen_height / 2
    dist = sqrt((px - cx)^2 + (py - cy)^2)
    max_dist = sqrt(cx^2 + cy^2)  // corner distance

    wave_speed = max_dist / 0.6   // crosses full screen in 600ms
    wave_pos = t * wave_speed     // current wave front position
    wave_width = 80.0             // pixels wide

    // Distance from wave front (negative = behind, positive = ahead)
    delta = dist - wave_pos

    intensity = if delta > 0 {
        0.0  // ahead of wave: dark
    } else if delta > -wave_width {
        // On the wave front: bright
        let phase = -delta / wave_width  // 0.0 to 1.0
        phase * phase  // quadratic falloff
    } else {
        // Behind wave front: exponential decay
        let age = (-delta - wave_width) / wave_speed
        exp(-age * 3.0)  // fast decay
    }

    // Global fade: start bright, fade at end
    global = if t > 0.7 { 1.0 - (t - 0.7) / 0.3 } else { 1.0 }

    final_intensity = intensity * global

    r = (0x00 as f32 * final_intensity) as u8
    g = (0xE5 as f32 * final_intensity) as u8
    b = (0x80 as f32 * final_intensity) as u8

    write_pixel(px, py, r, g, b)

This renders a clean expanding ring that fades as it expands.
The forest waking from darkness. Pure math.
---
PHASES

Phase 0 -- VM environment setup (1 session):
  Install QEMU + libvirt + virt-manager
  Create Arch Linux VM (minimal, same packages as real machine)
  Practice booting and accessing console
  Practice building and testing initramfs in VM
  Gate: VM boots Arch, can rebuild initramfs inside VM
        understand mkinitcpio hook structure

Phase 1 -- faelight-splash binary:
  Write the wave equation renderer in Rust
  Compile to x86_64-unknown-linux-musl (static, no deps)
  Test in VM: run faelight-splash manually, verify animation
  Gate: binary runs in VM, displays expanding green ring
        binary size < 500KB
        animation runs at 30fps without frame drops

Phase 2 -- Framebuffer integration:
  Switch from any test surface to actual DRM/KMS or /dev/fb0
  Handle resolution detection
  Handle the case where DRM is not available (fb0 fallback)
  Gate: animation renders at native VM display resolution
        correct framebuffer format (XRGB8888 or detected format)

Phase 3 -- mkinitcpio hook:
  Write install hook: adds binary + modules to initramfs
  Write runtime hook: starts binary, signals on root mount
  Test in VM: does it appear during boot? Does boot complete?
  Gate: VM boots with splash visible during initramfs phase
        normal boot completes after splash exits

Phase 4 -- LUKS compatibility:
  Test with encrypted VM disk
  Splash yields for password prompt
  Splash resumes after unlock
  Gate: LUKS-encrypted VM boots with splash, password prompt visible

Phase 5 -- faelight-login coordination:
  faelight-login detects active splash, waits for exit signal
  Smooth transition: splash fades as login appears
  No visible flash between splash and login
  Gate: transition from splash to login is invisible
        appears as one continuous animation

Phase 6 -- Real machine deployment:
  Full backup of current initramfs
  Apply hooks and binary to real machine
  Build new initramfs: mkinitcpio -p linux
  Reboot real machine with fingers ready on recovery USB
  Gate: real machine boots with forest splash
        if failure: recovery USB restores previous initramfs in < 5 minutes

Phase 7 -- Polish:
  Tune wave speed, intensity, colors
  Pulse behavior if boot takes > 1 second
  Verify on different resolutions (external display)
  Gate: looks identical to the vision described in this intent
        the wave is beautiful at any resolution
---
GATES
[ ] Phase 0: VM environment working, mkinitcpio understood
[ ] Phase 1: faelight-splash binary runs, animation correct, < 500KB static
[ ] Phase 2: renders to actual framebuffer at native resolution
[ ] Phase 3: mkinitcpio hook works, splash appears during boot, boot completes
[ ] Phase 4: LUKS-encrypted boot works with splash
[ ] Phase 5: faelight-login transition is invisible
[ ] Phase 6: real machine boots with forest splash, recovery plan tested
[ ] Phase 7: wave is beautiful, tuned, resolution-independent
Final:
[ ] The machine wakes into the forest from the very first pixel
[ ] No Plymouth, no spinner, no logos except the forest
[ ] The animation is pure math -- no assets, no image files
[ ] The transition to faelight-login is invisible -- one continuous experience
[ ] Linus Torvalds sees this machine boot and sees a forest wake up
---
DEPENDS ON
drm-rs crate -- already in workspace via faelight-compositor
mkinitcpio -- already installed (Arch Linux)
QEMU + libvirt -- to be installed (Phase 0)
faelight-login v2 -- COMPLETE -- coordination target

CRITICAL SAFETY RULE
Never modify real machine initramfs without:
  1. Testing the identical change in VM first
  2. Having a recovery USB with working initramfs ready
  3. Knowing exactly how to boot recovery and restore
  Breaking initramfs = unbootable machine.
  The VM is not optional. It is mandatory.

TIMELINE
Phase 0 (VM setup): start any time, prerequisite for everything else
Phase 1-3: after VM is working
Phase 4-5: can run in parallel with other intents
Phase 6 (real machine): last, after everything passes in VM
Target: real machine boot splash before NY presentation (mid-July 2026)
        the first thing Linus sees is the forest waking up

"Power is pressed.
Darkness.
Then a point of green at the center.
The forest is waking.
The wave expands.
The forest is alive.
The login screen appears.
The forest has always been here." 🌲
