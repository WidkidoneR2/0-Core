---
id: 091
date: 2026-06-25
type: future
status: complete
title: "Evaluate Stylix: declarative system-wide theming (vs the hand-crafted forest visual language)"
tags: [evaluation, stylix, theming, base16, visual-language, everglow, philosophy]
priority: low
---
## Why
Found Stylix via awesome-nix (2026-06-25): a NixOS module for system-wide colorscheming and
typography, declaratively. Set a palette + fonts once and it themes participating programs
(terminals, GTK apps, editors, etc.) consistently from one source. Related: base16.nix
(theme programs in base16 colorschemes with mustache templates).
Relevant because the forest vision wants the candy-neon aesthetic EVERYWHERE -- login,
logout, boot (Everglow/INT-078), the whole session boundary speaking one visual language.
Stylix is the community's declarative answer to "one theme, applied system-wide."
## The real tension (this is an EVALUATION, not a commitment)
There is a genuine conflict to resolve here, NOT a clear yes:
- PRO: declarative system-wide theming fits the forest's values; one source of truth for
  colors/fonts; could enforce the candy-neon palette consistently across every app; saves
  per-app theming effort; ties naturally to the Everglow "forest aesthetic everywhere" goal.
- CON (the big one): the forest's visual language is HAND-CRAFTED and deliberate. faelight-logout,
  faelight-bar, faelight-fm, the candy-neon system (INT-033), the forest color system -- these
  are bespoke, intentional, OWNED. Stylix imposes ITS model (base16 16-color slots) on theming.
  Adopting it could mean ceding fine control to a generalized framework, and base16's 16-slot
  palette may not capture the specific candy-neon forest language you designed. This is the
  same own-vs-adopt tension as nixvim (INT-090) -- you built fsh from scratch rather than adopt;
  is system THEMING something to own, or to delegate to Stylix?
- SUBTLE RISK: Stylix wants to theme MANY programs; it could fight your existing hand-rolled
  theming (faelight-* tools already have their own colors). Could create conflicts, not harmony.
## Priority honesty
NOT one of the three priorities (0-Core, faelight-shell, Friday). Theming-adjacent. LOW
priority. Most relevant AROUND the Everglow/visual-language work (INT-078) and AFTER the
login/compositor weekend -- not before. Don't let it pull focus from VM/login/Miracle.
## What "evaluate" means (gates about DECIDING, not adopting)
- [x] Read how Stylix's base16 model works; map it against the existing candy-neon/forest
      color system (INT-033) -- does the 16-slot model express the forest palette faithfully? <!-- INT-130 2026-07-10: DONE -- decisions/091 maps candy-neon slot-by-slot onto base16 (~10/16 fit, 6 gaps: no blue/orange/brown, green-tinted greys). -->
- [x] Try Stylix in the VM ONLY (never the daily driver first) on a throwaway config; see what
      it themes and whether it FIGHTS the hand-rolled faelight-* theming or complements it. <!-- INT-130 2026-07-10: VM trial genuinely NOT performed -- decisions/091 (L94-98) documents it as NOT REQUIRED: the base16 mapping is decisive alone (forest has no blue/orange/brown, base16 requires them), and the decision is to DECLINE wholesale adoption, so there is nothing to visually spike. Closed analytically, not by VM test. Reason recorded. -->
- [x] Assess: does system-wide declarative theming SERVE the forest's bespoke visual language,
      or flatten/override it? Harmony or conflict? <!-- INT-130 2026-07-10: DONE -- decisions/091: CONFLICT for owned faelight-* tools (base16 fabricates non-forest colors), ACCEPTABLE for the external long tail. -->
- [x] DECISION recorded (adopt wholesale / adopt for SOME apps only / base16.nix instead /
      keep hand-crafted / leave as-is) with reasoning. <!-- INT-130 2026-07-10: DONE -- decisions/091-stylix-evaluation.md, status:decided. HYBRID-NARROW: own the forest's candy-neon for faelight-* tools; optionally base16.nix ONLY the external long tail. Commits c2cd7acc, 26620f96. -->
## Notes
- base16.nix is the lighter-weight alternative (just base16 theming, mustache templates) if
  full Stylix is too heavy/opinionated.
- Evaluate in the VM -- theming changes are visible and reversible there, zero risk to the
  daily driver's carefully-built look.
- Connects to: INT-033 (candy-neon color system, done), INT-078 (Everglow), the whole
  "forest aesthetic everywhere" vision.
## The Rule
"The forest's colors were chosen, not generated. Evaluate honestly whether a framework
 can speak the forest's language -- or whether it would only make it speak base16's." 🌲

<!-- Gates reconciled per INT-130, 2026-07-10: GENUINE reconcile -- evaluation was DONE and well-documented (decisions/091-stylix-evaluation.md, status:decided), the intent's checkboxes just never got ticked (the exact 130 disease). Decision: HYBRID-NARROW (own candy-neon for faelight-* tools; optionally base16.nix the external long tail). HONEST NOTE: the 'try in VM' gate was NOT performed -- decisions/091 documents it as not required (declining wholesale adoption => nothing to spike), closed analytically. Commits c2cd7acc, 26620f96. 5/23. -->
