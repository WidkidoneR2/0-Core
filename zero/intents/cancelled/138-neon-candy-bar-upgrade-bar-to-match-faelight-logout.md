---
id: 138
date: 2026-07-10
type: future
title: "Neon-Candy Bar (Upgrade Bar to match faelight-logout)"
status: cancelled
tags: [bar, gtk4, candy, neon]
---

## Why
The forest's candy-neon identity is almost complete. faelight-logout sings. faelight-launcher
sings. The fsh prompt sings (INT-103: 2-line candy-neon powerline, zone-colored semantic
palette). faelight-bar is the ONE surface left that is merely functional -- it works, but it
does not feel like the forest the way the others do. This intent brings faelight-bar into the
same candy-neon family so the whole desktop, top to bottom, is unmistakably Faelight.

The bar is "Friday's face on the desktop." It should be as beautiful and magical as
faelight-logout -- not cosmetic tinkering, a real elevation into the forest's visual language.

## The tie to INT-103 (why this intent exists)
INT-103 (candy-neon fsh prompt) has a gate that is currently [~]:
  "Visually consistent with the planned faelight-bar candy pass"
That gate is deferred BECAUSE this companion intent was never created -- the prompt cannot be
consistent with a bar that isn't candy-neon yet. Completing 138 gives that gate its referent.
Prompt + bar were always meant to read as ONE system (same palette, same accent meanings).
Closing 138 lets INT-103 gate 5 flip [~] -> [x].

## Current bar (faelight-bar-gtk/main.py -- Python + GTK4 + gtk4-layer-shell)
Same stack as faelight-logout, so the candy pass builds on proven ground. Existing zones:
- LEFT:   workspaces (numbered labels + middot separators), health (H:NN%, already thresholded
          green/amber/red), git
- CENTER: active intent (already recolored purple/lavender -- matches the prompt's intent zone)
- RIGHT:  CPU  RAM  battery  wifi  clock (middot dim separators)
- Base text: #D7E0DA (fog white -- forest primary text token). Separators use a "dim" class.
The bar ALREADY has semantic hooks (health thresholds, intent-in-purple). This intent elevates
those into the full palette and gives EVERY zone a meaningful candy-neon color -- it is a
visual/taste elevation, not a rewrite.

## The palette (INT-033, shared with launcher / logout / prompt)
- electric lime   #A6E22E / #97C459  -- STRUCTURE (the signature forest color)
- aqua            #36E0D0            -- nix / devshell family (ties to the snowflake)
- rose            #ED93B1            -- alerts / attention
- lavender        #AFA9EC            -- intents (matches the prompt's intent zone)
- gold            #F4D06F            -- dirty git / warnings
- near-black-green base  #0a0f0c / #0c1404
Lime is structure; the accents carry MEANING. This is the exact family faelight-logout and
faelight-launcher already use.

## Zone -> candy-neon mapping (make prompt and bar speak ONE language)
Mirror the prompt's semantic mapping so a person reads the two as one system:
- workspaces      -- lime for the active workspace (structure), dim for inactive
- health H:NN%    -- keep the threshold logic, but in candy tones: lime >=95, gold 80-94,
                     rose <80 (rose = the alert accent, consistent with logout)
- git             -- gold when dirty (matches the prompt's gold-dirty-git), lime/dim when clean
- active intent   -- lavender (already is -- keep, it matches the prompt's intent zone exactly)
- clock / system  -- restrained: fog-white / dim so the accents pop where they mean something;
                     candy where it earns it (e.g. battery low -> rose, charging -> aqua)
- separators      -- keep the dim middots so the candy zones breathe (legible, not busy)

## Approach
- Python + GTK4 + gtk4-layer-shell (existing stack -- no rewrite).
- Pull colors from the INT-033 candy-neon tokens (one source of truth, same as prompt/logout)
  rather than hardcoding hexes, so prompt + bar + logout stay in lockstep.
- Iterate on the LOOK as a taste loop (like the candy-tuigreet / prompt arc): dial it until
  it is "as magical as faelight-logout." Screenshots / live observation drive it.
- Preserve performance: the bar redraws on a timer + on workspace/intent events; keep it fast
  and flicker-free. Candy pop must not cost redraw smoothness.
- Keep it LEGIBLE: standout = beautiful + readable, never busy. The dim separators and
  restrained system zone are what let the meaningful accents sing.

## Gates (demonstrated, not declared)
- [ ] Bar uses the candy-neon palette, consistent with launcher / logout / prompt (INT-033 tokens)
- [ ] Each bar zone's color has MEANING, mirroring the prompt's mapping (health, git-dirty=gold,
      intent=lavender, workspace-active=lime) -- prompt and bar read as ONE system
- [ ] Legible + performant -- no redraw lag, no flicker, not cluttered
- [ ] Christian's eye test: "as beautiful and magical as faelight-logout"
- [ ] Colors pulled from the shared INT-033 token source (not hardcoded) so prompt/bar/logout
      stay in lockstep
- [ ] Closing this flips INT-103 gate 5 from [~] to [x] (prompt <-> bar consistency achieved)

## Depends On / Relates To
- INT-033 (candy-neon semantic palette -- the shared token source)
- INT-103 (candy-neon prompt -- the sibling this must match; its gate 5 waits on this)
- faelight-logout / faelight-launcher (the visual bar to meet -- "as magical as")

## The Rule
"The forest should speak one language, from prompt to bar to the door you leave by.
 The bar is Friday's face -- let it be as bright and magical as the rest of the forest." 🌲

## Gate Check
🚫 138 -- cancelled: Upgrades the GTK bar, whose Friday zone read a wrong path and was silently blank and whose intent zone never had a focus set. Superseded by Quickshell, which already delivers more than that bar did. -- approved by: christian 2026-08-27
