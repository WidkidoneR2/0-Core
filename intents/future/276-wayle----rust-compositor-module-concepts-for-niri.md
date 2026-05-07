---
id: 276
title: "Wayle -- Rust Compositor Module Concepts for Niri"
status: planned
date: 2026-05-06
tags: [wayle, niri, compositor, rust, gtk4, relm4, notifications, study, evaluation]
---
Wayle is a Rust-based desktop environment targeting Wayland compositors.
Source: https://github.com/wayle-rs/wayle
The forest does not need Wayle.
But Wayle has ideas the forest has not explored yet.
Two specific things worth studying:
1. Compositor-specific modules for Niri (and Hyprland)
2. Notification system written in Rust with GTK4 and Relm4
---
OBSERVATION 1: COMPOSITOR MODULES FOR NIRI
Wayle builds compositor-specific integration modules.
faelight-niri-bridge does this too -- but as a single binary polling niri msg.
What does Wayle do differently?
Does it use niri's IPC more efficiently?
Does it expose compositor state that faelight-niri-bridge currently misses?
Does its Niri module pattern suggest improvements to faelight-niri-bridge v2?
The forest talks to Niri through:
  niri msg -j focused-window (polled by faelight-bar)
  niri msg -j workspaces (polled by faelight-bar)
  niri msg event-stream (faelight-niri-bridge)
Is there a better pattern? Wayle may show us.
---
OBSERVATION 2: NOTIFICATIONS WITH GTK4 + RELM4
faelight-notify: D-Bus + zbus + custom ratatui rendering
Wayle notifications: GTK4 + Relm4 (Elm architecture for GTK)
Relm4 brings:
  Elm-like message/update/view architecture (similar to iced)
  GTK4 widgets with first-class Wayland support
  gtk4-layer-shell for proper overlay positioning
  Proper animation support
  Native font rendering via Pango
The tradeoff:
  GTK4 is a C library with Rust bindings -- not pure Rust
  Adds a significant dependency
  But delivers capabilities faelight-notify cannot easily replicate
Key question: does faelight-notify need to be more capable?
  Currently: popups render in a terminal-adjacent overlay
  With GTK4+Relm4: popups would be proper native Wayland surfaces
This is the gap Wayle reveals.
The forest decides whether to close it.
---
WHAT THIS IS NOT
This is not a plan to adopt GTK4 or Relm4.
The forest philosophy: understanding over convenience.
Study first. Decide with evidence.
---
GATES
[ ] Read Wayle source: compositor module architecture for Niri
[ ] Compare Wayle Niri IPC pattern vs faelight-niri-bridge approach
[ ] Document: what does Wayle expose that faelight-niri-bridge misses?
[ ] Read Wayle notification system: GTK4 + Relm4 architecture
[ ] Understand Relm4: how Elm architecture maps to GTK4
[ ] Evaluate: would GTK4 notifications improve faelight-notify meaningfully?
[ ] Evaluate: does gtk4-layer-shell solve problems faelight-notify has?
[ ] Document findings -- what (if anything) the forest should adopt
[ ] Move to decisions/ with clear conclusion
Possible outcomes:
  A) Wayle confirms faelight-niri-bridge is on the right path -- no change
  B) Wayle reveals a better Niri IPC pattern -- faelight-niri-bridge v2 adopts it
  C) Relm4 notifications are compelling -- faelight-notify v2 evaluates GTK4
  D) GTK4 dependency conflicts with 99% Rust goal -- confirmed, stay the course
"The forest watches what others build.
It learns from every approach.
Then it builds its own way." 🌲
