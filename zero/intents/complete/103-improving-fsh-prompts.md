---
id: 103
date: 2026-07-01
type: future
title: "Improving Fsh Prompts"
status: complete
tags: [prompt, fsh]
---


## Why
The fsh prompt is functional but visually flat -- it does NOT sing the way
faelight-launcher and faelight-logout do. Now that the forest is on NixOS with the
INT-033 semantic candy-neon color system in place (theme.rs/prompt.rs), this is the
moment to bring the prompt into that visual family BOLDLY. Not cosmetic tinkering:
make the prompt a standout piece of the forest's identity.

## Vision -- candy-neon prompt, forest-family
Bring the fsh prompt into the same candy-neon palette as faelight-launcher and
faelight-logout:
- Launcher/logout family: electric lime (#A6E22E / #97C459), aqua (#36E0D0),
  rose (#ED93B1), lavender (#AFA9EC), gold (#F4D06F), near-black-green base
  (#0a0f0c / #0c1404). Lime = structure, others = accents.
- The prompt should feel unmistakably part of THIS forest -- a person seeing a
  screenshot should know it's Faelight.

## Scope -- what "better" means (NOT just recolor)
- Use the INT-033 semantic color system (prompt.rs already consumes it) -- push it
  HARD, not timidly.
- Consider the prompt's information design too: the segments (cwd, git branch/state,
  health %, intent, the ❄/🌲 markers, exit-code) -- are they arranged for maximum
  clarity AND candy pop? Standout means legible + beautiful, not busy.
- The multi-line prompt structure (path line, status line, the ❯ input line) is a
  canvas -- each zone can carry candy-neon accents that mean something (e.g. health
  color-coded, git-dirty in a specific accent, intent in another).

## Companion intent (Christian, 2026-07-01)
A SEPARATE bar intent will push the same envelope for faelight-bar ("Friday's face
on the desktop", the 3-zone v2 concept). Prompt (103) + bar together define the
candy-neon desktop identity. Keep them visually consistent (same palette, same
accent meanings) so prompt and bar read as one system.

## Approach (rough -- recon first)
- Read prompt.rs: how it currently builds the prompt, which semantic colors it
  already uses, where the flat spots are.
- Map each prompt zone to a candy-neon role (lime structure, accents with meaning).
- Iterate on the LOOK -- this is a visual/taste loop like the candy-tuigreet arc
  (dial it until it's "chef's kiss"). Screenshots/observation drive it.
- Preserve prompt performance (it renders every keystroke-ish; keep it fast).

## Gates (demonstrated, not declared)
- [x] Prompt uses the candy-neon palette consistent with launcher/logout <!-- STAMP-103-DONE / INT-130 2026-07-10: shipped -- commits a001b8e8 (candy-neon powerline) + 24f451a9 (~70% neon across 17 zone/accent colors: lime, rust-orange, lavender, ice-blue, aqua, gold, rose). Same palette family as launcher/logout. NixOS-era 103 (NOT the Arch-era faelight-idle INT-103). -->
- [x] Each prompt zone's color has MEANING (not random rainbow) <!-- INT-130 2026-07-10: meanings documented in a001b8e8 -- lime=forest, orange=rust, lavender=intents, ice-blue=nix, gold=dirty-git, aqua=devshell. 'Directory color tells you what KIND of place you're in.' Semantic, not rainbow. -->
- [x] Legible + fast (no performance regression, no clutter) <!-- INT-130 2026-07-10: attested by author -- prompt renders fast, legible, uncluttered in daily use. Health/rebuild line was removed (lives on bar + d), reducing clutter. -->
- [x] Christian's eye test: "it stands out, it's unmistakably Faelight" <!-- INT-130 2026-07-10: PASSED -- author confirmed live this session ("yes it works"); eye-tested live per commit 94df029b. -->
- [~] Visually consistent with the planned faelight-bar candy pass <!-- INT-130 2026-07-10: CANNOT be met as written -- the 'planned faelight-bar candy pass' (named as a companion intent in this charter, 2026-07-01) was NEVER created as an intent, and the bar is not candy-neon. The prompt cannot be consistent with an unbuilt thing. Deferred [~]. FOLLOW-UP: file a faelight-bar candy-neon intent (author wants the bar as magical as launcher/logout) -- that would give this gate a real referent. To be done AFTER the 130 audit + 137, per author's sequencing. -->

## Notes
Christian 2026-07-01: "the fsh prompt needs to be better improved, not just by
cosmetics. Now that we are Nix this is a good time to really give it that neon
candy look like we have with faelight-launcher, faelight-logout. I want it to stand
out." Pure userspace visual work -- no login/boot risk. A good taste-loop intent.
