# INT-348 -- Study: Ewwii
# Status: future
# Tags: study, widgets, bar, rust, wayland, gtk4

## Purpose
Study Ewwii as potential foundation for faelight-bar v4 widget layer.
Ewwii is a Rust GTK4 widget system -- closest thing to the forest's stack.

## What Ewwii Is
Fork of Eww (Elkowars Wacky Widgets) rewritten in Rust.
Standalone widget system -- works in any WM/compositor.
GPU accelerated. Scriptable from any language. GPL-3.0.
Has its own package manager (eiipm).

## The Question
Can faelight-bar v4 use Ewwii as its rendering foundation
instead of building raw Wayland layer-shell again?

Pros: GPU accelerated, Rust, Wayland native, scriptable
Cons: GTK4 dependency, not aware of forest concepts, extra layer

## Study Focus
- Widget declaration model -- how widgets are composed
- IPC/scripting interface -- how it reads shell data
- Niri compatibility -- does it work with Niri layer-shell?
- Performance -- startup time, memory footprint
- NixOS packaging -- is there a nixpkg?

## Decision Gate
After study: adopt as faelight-bar v4 foundation, or continue with custom layer-shell?

## Source
https://github.com/Ewwii-sh/ewwii
74 stars, active development, Rust + GTK4
