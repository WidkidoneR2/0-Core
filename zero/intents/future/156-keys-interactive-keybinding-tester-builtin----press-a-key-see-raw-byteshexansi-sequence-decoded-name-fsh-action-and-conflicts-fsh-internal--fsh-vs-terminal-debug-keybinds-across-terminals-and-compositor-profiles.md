---
id: 156
date: 2026-07-13
type: future
title: "keys: interactive keybinding tester builtin -- press a key, see raw bytes/hex/ANSI sequence, decoded name, fsh action, and conflicts (fsh-internal + fsh-vs-terminal). Debug keybinds across terminals and compositor profiles."
status: planned
tags: [fsh, keybinds, tui, debug, dx, terminal, 154, 155]
---

## Vision
A `keys` builtin that drops into an interactive tester: press any key or chord, and fsh shows
exactly what your terminal sent and what fsh does with it -- raw bytes, hex, the ANSI escape
sequence, the decoded key name, the fsh action it is bound to, and any conflicts. Press q/Esc to
exit. The forest tells you the truth about your keyboard.

## Why (the real, personal need)
fsh runs across alacritty in THREE compositor profiles (mango daily, Miracle dark-glass,
Pinnacle frosted). Terminals and compositors send slightly different escape sequences for the
same chord, and a compositor may intercept a chord before the terminal/fsh ever sees it. When
mirroring mango's keybinds into Miracle (INT-154) and Pinnacle (INT-155), "did this bind actually
fire?" and "why does Super+X do nothing here?" are constant questions. `keys` answers them
directly: press the chord, see the bytes, see whether fsh got it and what it mapped to. This is
debugging infrastructure for the keybind-mirroring work, not a toy.

## Core capture + display (first build)
On each keypress, show:
- Raw bytes (e.g. 0x1b 0x5b 0x31 ...) 
- Hex + the ANSI escape sequence rendered readable (e.g. ESC [1;4D)
- Decoded key name if known (e.g. Shift+Alt+Left) -- from a sequence->name table (see scope note)
- fsh action if the sequence is bound (cross-reference fsh's own keybind map)
- Status line (OK / unbound / see conflict)
Mechanism: put the terminal in raw mode (fsh already uses rustyline/raw-mode for the prompt, so
the machinery exists), read the byte sequence for the chord, format it. Standard terminal work.

## Conflict detection (in scope for first build)
Two USEFUL conflict types for a personal shell (NOT other-shell conventions like fish -- not
relevant here):
- fsh-internal: two fsh keybinds want the SAME sequence -> flag the collision + name both binds.
- fsh-vs-terminal: the terminal (or compositor) intercepts a chord BEFORE fsh sees it (e.g.
  terminal eats Ctrl+Shift+C for copy; a compositor eats Super+chord). Detect by: fsh never
  receives bytes for a chord the user pressed -> "intercepted upstream (terminal/compositor),
  fsh never saw it." This is the one that explains cross-profile keybind mysteries.

## Honest scope notes
- Decoded-name requires a sequence->name TABLE (real work -- build incrementally; unknown
  sequences still show raw bytes/hex/ANSI, just labelled "unknown key"). Core value survives even
  with a partial table: raw bytes + ANSI are always shown.
- DEFERRED to later (stretch, not first build): terminal identification (which emulator),
  latency/repeat-rate measurement, a "supported terminals" matrix. Nice, niche, not why you
  reach for this.

## Success criteria
- [ ] `keys` builtin registered (first-class fsh domain, like `vm` / `d`); q or Esc exits cleanly
- [ ] press a key -> correct raw bytes + hex + ANSI sequence shown (verified against several
      chords: Ctrl+C, arrows, Shift+Alt+Left, a plain letter)
- [ ] bound key -> correct fsh action shown (cross-referenced from fsh's keybind map)
- [ ] unbound key -> shown as unbound (not a crash, not silence)
- [ ] fsh-internal conflict -> two binds on one sequence flagged, both named
- [ ] fsh-vs-terminal interception -> a chord fsh never receives is reported as
      intercepted-upstream (demonstrated with a known terminal-eaten chord)
- [ ] decoded-name table covers the common chords; unknowns degrade gracefully to raw display

## Relationships
- INT-154 (Miracle Day) + INT-155 (Pinnacle Day): `keys` is the tool that makes the keybind
  MIRROR tractable -- verify each ported bind actually fires per profile, catch conflicts and
  upstream interceptions. Well-sequenced: build `keys` before/during the compositor Days for
  immediate payoff.
- Personal-shell scope (2026-07-13): fsh is Christian's personal shell, not a distributed
  product -- so this is a PERSONAL debugging tool, not a public UX feature. (Sibling ideas from
  the same brainstorm -- flake integration tests, Home Manager example configs, distributable
  packaging -- were set aside as public-project concerns that do not apply to a personal shell.)
- Doctor (`d`): related but distinct -- doctor is run-and-report health checks; `keys` is an
  interactive live tester. Kept separate deliberately.

## The Rule
"Press the key. See the truth. No more guessing which chord the terminal actually sent." 🌲
