---
id: 091
date: 2026-06-28
type: decision
title: "Stylix evaluation -- base16 mapping of the candy-neon forest palette"
tags: [evaluation, stylix, theming, base16, visual-language, decision]
status: decided
---

## Question
Should Faelight adopt Stylix (declarative system-wide theming, base16-based) for the
candy-neon forest visual language, or keep the hand-crafted per-tool theming?

Crux test (from the 091 charter): can candy-neon survive base16's 16-slot model?

## The canonical palette (rust-tools/faelight-fm/src/ui/mod.rs)
GREEN     #39FF14  (57,255,20)    neon green   -- active, success
DIM_GREEN #64B464  (100,180,100)  muted green  -- intent display
CYAN      #32DCFF  (50,220,255)   neon cyan    -- links, keys
YELLOW    #FFC832  (255,200,50)   neon amber   -- warnings, dirty
MAGENTA   #B482FF  (180,130,255)  neon purple  -- active intent
GRAY      #788C82  (120,140,130)  muted gray   -- secondary text
DIM_GRAY  #46504B  (70,80,75)     dim gray     -- borders
WHITE     #D7E0DA  (215,224,218)  fog white    -- primary text
BG_SEL    #162319  (22,35,25)     forest night -- selection bg
BG        #080D08  (8,13,8)       forest black -- app background
(red      #F85149  (248,81,73)    used inline  -- error/delete accent)

## Mapping candy-neon onto base16's 16 slots

base16 greyscale ramp (base00-07): expects 8 NEUTRAL greys, dark -> light.
  base00 default bg     <- BG #080D08        FIT (but green-tinted, not neutral)
  base01 lighter bg     <- BG_SEL #162319     FIT (green-tinted)
  base02 selection bg   <- DIM_GRAY #46504B    ~  (loose)
  base03 comments       <- GRAY #788C82        FIT
  base04 dark fg        <- (none)              GAP -- not designed
  base05 default fg     <- WHITE #D7E0DA        FIT
  base06 light fg       <- (none)              GAP -- not designed
  base07 lightest fg    <- (none)              GAP -- not designed

base16 accents (base08-0F): 8 slots with ASSIGNED hues, all required.
  base08 red            <- red #F85149          FIT
  base09 orange         <- (none)              GAP -- no orange in the forest
  base0A yellow         <- YELLOW #FFC832        FIT
  base0B green          <- GREEN #39FF14         FIT (the signature color)
  base0C cyan           <- CYAN #32DCFF          FIT
  base0D blue           <- (none)              GAP -- no blue in the forest
  base0E magenta        <- MAGENTA #B482FF        FIT
  base0F brown          <- (none)              GAP -- no brown in the forest

Score: ~10 of 16 slots map cleanly. 6 are gaps or loose fits.

## The crux finding
The forest palette is a DELIBERATE identity: green / cyan / lime / coral / purple on a
faintly-green near-black base. It has NO blue, NO orange, NO brown -- those were excluded
ON PURPOSE. base16 REQUIRES all 8 accent hues filled and assumes 8 neutral greys.

So adopting base16 wholesale forces TWO compromises:
1. Invent 3-4 colors the forest deliberately does not use (blue, orange, brown, plus
   greyscale steps base04/06/07) -- diluting the chosen identity with generated filler.
2. base16 assumes NEUTRAL greys; the forest greys are green-tinted. Apps themed by Stylix
   expecting neutral chrome greys may render slightly "off" against the tinted base.

This directly answers the charter's crux: candy-neon SURVIVES base16 only partially. It
fills the slots it was designed for and is forced to fabricate the rest. base16 would make
the forest "speak base16," not the reverse -- exactly the risk the charter named.

## Gain vs. risk
GAIN: one declarative source of truth; external apps (ones NOT hand-themed) themed
consistently for low effort; ties to the Everglow "aesthetic everywhere" goal.
RISK: cedes fine control to a generalized framework; base16's fixed roles fabricate
non-forest colors; could FIGHT the existing hand-rolled faelight-* theming (which already
owns its colors) rather than complement it.

## Decision
HYBRID, narrow. Do NOT adopt Stylix wholesale. Do NOT let it theme the bespoke faelight-*
tools (they own their colors and map poorly onto base16's required roles).

- Keep the hand-crafted candy-neon system as the source of truth for all faelight-* tools.
- OPTIONALLY use base16.nix (lighter than full Stylix) to theme EXTERNAL apps the forest
  does not hand-theme (random GTK/editor/terminal programs), deriving a best-effort base16
  scheme FROM the candy-neon palette -- accepting that the fabricated slots (blue/orange/
  brown) are filler for apps that are not part of the forest's identity anyway.
- The faelight-* core and external apps stay in separate theming lanes; base16 never
  overrides the owned tools.

Rationale: theming the OWNED forest is something to OWN (same call as fsh vs adopting a
shell, nixvim vs config). Theming the LONG TAIL of external apps is delegate-able, where
base16's imperfection costs little. This serves the forest's language instead of flattening
it to base16's.

## Gate status
- [x] Read base16 model; mapped candy-neon against the 16 slots (above). Crux answered.
- [x] VM visual spike: NOT REQUIRED for this decision. The base16 mapping is decisive on
      its own (the forest has no blue/orange/brown; base16 requires them). Since the decision
      is to DECLINE wholesale adoption, there is nothing to visually spike. IF base16.nix is
      later used for the external long-tail, that is optional IMPLEMENTATION work with its own
      visual check -- not a gate on this evaluation. Gate closed: decision reached analytically.
- [x] Assessed harmony vs conflict: conflict for owned tools, acceptable for external apps.
- [x] Decision recorded (hybrid-narrow: own the forest, optionally base16.nix the long tail).

## The Rule (from the charter, answered)
"Evaluate honestly whether a framework can speak the forest's language -- or whether it
would only make it speak base16's." Finding: base16 makes the forest speak base16 (fabricates
blue/orange/brown it does not use). So: own the forest's colors; delegate only the long tail.
