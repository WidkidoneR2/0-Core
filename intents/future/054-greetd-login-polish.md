---
id: 054
date: 2026-06-09
type: polish
title: "greetd: green theme, F2 session picker, compositor safety net"
status: planned
tags: [greetd, tuigreet, login, polish, recovery, safety]
priority: medium
---
## Why
The login screen works but lacks polish. Green theme is not rendering.
Session picker keybind needs fixing. More critically: the 2026-06-09
tuigreet incident proved that greetd changes on a live system without
VM pre-flight is unacceptable. This intent enforces that discipline.

## Depends On
- INT-056 (Forest Recovery Protocol) -- MUST complete Phase 1 and 2
  before any work in this intent begins on the real machine
- INT-024 (VM graduation pipeline) -- all changes tested in VM first

## Vision
- Green neon candy theme renders correctly in tuigreet
- F2 opens session picker reliably
- MangoWM and Pinnacle both selectable at login
- Fallback session (fsh) defined for recovery
- Boot generation pinned as rollback safety net
- All changes land via INT-024 graduation pipeline

## Phases

Phase 1 -- VM pre-flight (INT-024 required)
  All greetd config changes built and tested in VM first
  Snapshot created: before-INT-054
  Primary session, fallback session, F2 picker all tested in VM
  Gate: all greetd changes verified working in VM, recovery from
        broken session demonstrated

Phase 2 -- Graduate to real machine
  INT-056 Phase 1+2 gates passed (TTY2 hardened, fallback defined)
  Snapshot/generation checkpoint before applying
  Apply greetd changes from VM-verified config
  Gate: changes on real machine match VM behavior

Phase 3 -- Theme and polish
  Green neon candy colors in tuigreet config
  Clock and hostname rendering verified
  Gate: login screen matches Faelight Forest aesthetic

## Gates
- [ ] INT-056 Phase 1 complete (TTY2 hardened) before any live work
- [ ] INT-056 Phase 2 complete (fallback session defined) before any live work
- [ ] All changes tested in VM via INT-024 before real machine
- [ ] VM snapshot created: before-INT-054
- [ ] Green theme renders correctly on real machine
- [ ] F2 opens session picker reliably
- [ ] MangoWM selectable at login
- [ ] Pinnacle selectable at login
- [ ] Fallback fsh session defined in greetd config
- [ ] Boot generation rollback verified before closing intent

## The Rule
"greetd is the door to the forest.
 We do not change the door on a live house
 without a key in our pocket and the window already open." 🌲
