---
id: 136
date: 2026-07-10
type: future
title: "faelight-term v4: a real terminal, and a Nix contract for it"
status: planned
tags: [faelight-term, terminal, nix, alacritty, mars, integration]
version: TBD
---

## Filed during the intent freeze -- an explicit exception
On 2026-07-09 Christian froze new intents until the five in-progress close (056, 086, 087,
130, 135), directing future intents to Friday and fsh only. This charter breaks that freeze
deliberately, because the reconnaissance was live and a note would be a charter with less
discipline. Recorded here rather than quietly. **Work does not start until the five close.**

## The trigger
Christian found Mars (github.com/luccahuguet/mars) and said: "this is exactly what I tried
to build with faelight-term, and failed." The recon showed something better than that.

## What the recon actually found (2026-07-10)

**faelight-term is ALREADY the Mars model.** Mars is a Rio *fork* -- it inherits sugarloaf,
teletypewriter, corcovado, rio-backend, and contributes deltas (font placement, palette,
quit modal, split cursors, Kitty graphics fixes). It tunes; it does not construct.
faelight-term v3 is 1,318 lines of frontend over mature upstream crates:
  - `alacritty_terminal 0.24` -- VTE parsing, grid, scrollback, PTY
  - `cosmic-text 0.12` + `glyphon 0.8` -- shaping, Unicode, GPU text
  - `wgpu 24`, `smithay-client-toolkit`, `wl-clipboard-rs`
That is LESS delta than Mars carries. Architecturally, faelight-term and Rio are siblings.
**There is nothing to fork. The architecture is already right.**

**Rio was already evaluated and rejected** -- `decisions/135-rio-terminal` (2026-05-06):
skim TUIs fail, emoji broken, alt-screen incompatible. Mars inherits Rio and its delta table
does not mention emoji width, skim, or alt screen. Adopting Mars likely reproduces the exact
failure already recorded. (Coincidence worth noting: Mars's author is luccahuguet, whose
Yazelix was studied in `complete/009-study-yazelix`. The same person's work, twice.)

**What actually broke v3 -- the failure inventory, observed live:**
1. **No argument parsing.** `faelight-term --help` launches a terminal and hangs. There is no
   clap, no arg handling. Confirmed by ^C after 13s.
2. **No shell selection.** Opens `bash`, not `fsh`. Nothing in main.rs selects a shell;
   Alacritty declares `terminal.shell` in TOML, faelight-term inherits a fallback.
3. **Wide characters smear the grid.** Powerline separators blur into adjacent text.
   ROOT CAUSE (line ~351): it measures ONE monospace advance width and assumes every glyph
   fits one cell. It does not implement **wcwidth** -- the Unicode rule for whether a
   grapheme occupies one column or two. Powerline glyphs, Nerd Font icons, and emoji are
   WIDE. This is the SAME bug class that got Rio rejected in decisions/135.
4. **No config surface.** `const FONT_SIZE: f32 = 12.0` at line 55. Font, palette, cursor,
   shell are compile-time constants. The terminal cannot be configured without a rebuild.
5. **Zero TODOs in 1,318 lines** -- not a sign of completeness. A sign nobody wrote down
   what was missing. That is why it stalled for 3-4 months.

## The goal -- stated honestly
**NOT "better than Alacritty and Mars."** Alacritty is ~8 years and dozens of contributors,
built to be fast and minimal. Its years were not spent typing -- they were spent absorbing
thousands of bug reports from real software in real terminals, each revealing one place the
model diverged from reality. That corpus cannot be generated; it can only be accumulated.
A goal of "surpass Alacritty" is a gate that never ticks, and INT-130 exists because of
gates that never tick.

**The achievable, closeable goal, on three axes:**
- **PARITY on correctness**, where it can be tested. wcwidth, alt screen, bracketed paste,
  DA1/DA2, mouse protocols, selection, resize reflow.
- **SUPERIORITY on integration** -- the one axis nobody else can copy. Alacritty does not
  know the intent ledger. Mars does not know the health score. faelight-term v11 once had an
  intent-aware title bar and an exit-status indicator. *The forest knows things.* That is the
  differentiator: not speed, not features. Integration.
- **A NIX CONTRACT neither has.** This is Christian's idea and it is the best one here.

## The Nix contract -- the structural insight
Mars's real contribution is not code. Yazelix consumes Mars through a **declared Nix package
surface** (`MARS_PROFILE`, `MARS_APPEARANCE`, cursor presets) -- reading metadata rather than
guessing internals. The terminal exposes an interface; the environment reads it.

The forest currently has `nix/home/christian/alacritty.nix`: a module that copies a TOML file
and comments that font and shell "are set in alacritty.toml". A contract by convention.

**v4 makes it a contract by declaration:**
```nix
programs.faelight-term = {
  enable  = true;
  shell   = "faelight-shell";
  font    = { family = "JetBrainsMono Nerd Font"; size = 12; };
  palette = config.lib.stylix.colors;   # single source of truth -- INT-091
  cursor  = { style = "beam"; healthTint = true; };
  profile = "forest";
};
```
The terminal reads that surface. The terminal becomes swappable. And the palette plugs
straight into the candy-neon / Stylix work (INT-091), where `config.lib.stylix.colors` is
already the intended single source of truth.

## Success Criteria
- [ ] **Gate 1 -- argument parsing.** `--help`, `--version`, `-e <cmd>` behave. None of them
      launch a terminal. Proven on the DEPLOYED binary.
- [ ] **Gate 2 -- shell selection.** Launches `fsh`, declared not hardcoded. Falls back
      sanely. Proven: `faelight-term` opens fsh; `faelight-term -e bash` opens bash.
- [ ] **Gate 3 -- wide characters. THE HARD ONE.** Implement wcwidth-correct cell placement:
      wide glyphs claim two cells; combining marks claim zero. PROOF: the fsh powerline
      prompt renders correctly, emoji do not smear, `htop`/`btm`/`yazi` draw clean borders.
      This is most of the work. Everything else is plumbing.
- [ ] **Gate 4 -- config surface.** Font, size, palette, cursor, shell read from a declared
      source at runtime. No `const FONT_SIZE`. Changing the config changes the terminal
      without a rebuild.
- [ ] **Gate 5 -- the Nix module.** `programs.faelight-term` writes Gate 4's surface.
      Demonstrated: change the palette in Nix, rebuild, see it in the terminal.
- [ ] **Gate 6 -- correctness suite.** A reproducible test the way Mars has
      `tools/mars_perf_gate.py`: alt screen, bracketed paste (heredocs), DA1/DA2, mouse
      tracking, resize reflow, selection. Each either passes or is honestly listed as unmet.
- [ ] **Gate 7 -- integration, the differentiator.** At least one thing no other terminal can
      do: active intent in the title bar, or cursor tinted by health score, or exit status
      surfaced. Small, real, and impossible for Alacritty.
- [ ] **Gate 8 -- daily-drive it for one week.** The honest gate. If Christian reaches for
      Alacritty, it is not done. Log what sent him back.

## On the AI multiplier -- recorded honestly
Christian's argument: "Alacritty took 8 years without an AI; Claude could halve it."
Claude's honest counter, recorded because it belongs in the charter either way:

The multiplier is real on things that can be *specified* -- plumbing, protocols with published
specs (wcwidth has a spec; DA1/DA2 have specs), the Nix module, refactors, test harnesses.
Genuine leverage there, plausibly 2x or better.

It is near zero on **unknown-unknowns**, and a terminal is mostly unknown-unknowns. Alacritty's
eight years bought a bug corpus, not code. Claude cannot generate that corpus. Neither can one
person. Evidence from the two days preceding this charter: Claude was confidently wrong four
times about a *counter* (one word, one file); invented a `templates/` directory that never
existed and wrote it into the ledger; patched two identical-looking lines and broke the correct
one (the compiler caught it, not Claude); and ran `cargo build -p faelight-core` three times --
a no-op on an unrelated crate -- reading "Finished" as success.

"AI will halve it" is a DECLARATION. This forest runs on demonstrated, not declared.
**The multiplier is unknown. Gates 1-3 will measure it.** Gate 3 in particular: wcwidth is
specified, so if the AI multiplier is real anywhere, it is real there. If Gate 3 takes as long
as it would have without an AI, that is the honest answer and it gets recorded.

## Relationship
- Prior art: `decisions/135` (Rio rejected -- emoji, skim TUIs, alt screen).
- Prior art: `complete/009-study-yazelix` (same author as Mars).
- Feeds: INT-091 (Stylix -- `config.lib.stylix.colors` as palette source).
- Related: INT-134 (fsh roadmap -- the terminal and the shell are one experience).
- Supersedes the "v4" fragments in the changelog: "foot removed, Zellij removed, Alacritty
  wired as..." -- v4 was started, never charted, and quietly replaced by Alacritty.

## Notes
- Do NOT fork Alacritty. Alacritty the APP is minimal by design -- no tabs, no splits, no
  graphics protocol -- and its maintainers reject those features. Forking it means BUILDING
  what Mars merely TUNED. `alacritty_terminal` the CRATE is already a dependency. That was
  always the right call.
- WezTerm publishes `termwiz` and `wezterm-term` the same way. Swapping the VTE backend is a
  bounded experiment, not a rewrite -- worth knowing, not worth doing yet.
- The filename generated for this charter contains `:` and `,` -- the slug generator does not
  sanitize. Noted, not fixed. -> INT-135's territory.
