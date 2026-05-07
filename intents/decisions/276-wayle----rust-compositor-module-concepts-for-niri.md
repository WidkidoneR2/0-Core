---
id: 276
title: "Wayle -- Rust Compositor Module Concepts for Niri"
status: decided
date: 2026-05-06
decided: 2026-05-06
verdict: monitor
tags: [wayle, niri, compositor, rust, gtk4, relm4, notifications, decided]
---
Wayle studied on 2026-05-06. Monitored but not adopted.
- Wayle: Rust-based desktop environment targeting Wayland compositors
- Compositor-specific modules for Niri and Hyprland
- Notification system using GTK4 + Relm4
Wayle has compositor-specific modules for Niri -- the same integration
pattern faelight-niri-bridge uses, potentially more efficiently.
Wayle's notification system uses GTK4 + Relm4 (Elm architecture for GTK).
This gives proper layer-shell Wayland surfaces, animation, and font rendering
that faelight-notify's fontdue approach cannot easily replicate.
Wayle is young and not mature enough to evaluate concretely today.
The GTK4 dependency conflicts with the 99% Rust / minimal dependency philosophy.
However: the Niri compositor module pattern is worth revisiting when
faelight-niri-bridge v2 is planned. And if faelight-notify ever needs
proper layer-shell popup surfaces, Relm4 is the reference to study.
Monitor Wayle's development. Revisit post-presentation.
"The forest watched what Wayle was building.
It learned from the pattern.
It will build its own way." 🌲
