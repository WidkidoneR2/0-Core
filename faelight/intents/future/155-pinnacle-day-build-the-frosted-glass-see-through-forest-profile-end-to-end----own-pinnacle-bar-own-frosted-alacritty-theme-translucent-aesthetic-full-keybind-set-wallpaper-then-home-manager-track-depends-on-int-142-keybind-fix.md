---
id: 155
date: 2026-07-12
type: future
title: "Pinnacle Day: build the frosted-glass see-through forest profile end-to-end -- own pinnacle-bar, own frosted Alacritty theme, translucent aesthetic, full keybind set, wallpaper, then home-manager track (depends on INT-142 keybind fix)"
status: planned
depends_on: [142]
tags: [compositor, pinnacle, theme, bar, alacritty, frosted-glass, 142, 067]
---

## Vision
Pinnacle as the FROSTED-GLASS SEE-THROUGH FOREST -- the same forest as mango (fsh, keybinds,
tooling) wearing a lighter, translucent skin: frosted glass, see-through panels, a brighter airy
scenery you can almost look through. The counterpoint to Miracle's dark glass. A change of mood,
not of bones. Mango stays the primary WORK profile; Pinnacle is chosen for a light, open,
frosted change of pace. Dedicated full-day build, unhurried, tuned until it looks right.

## HARD PREREQUISITE -- INT-142 must land FIRST
Unlike Miracle (whose config already loads), Pinnacle's custom Lua config has NEVER loaded under
greetd -- it falls back to the DEFAULT config, so custom forest keybinds (Super+B, Super+E,
Super+P) do NOT fire. INT-142 is the fix (protobuf definitions via system XDG_DATA_DIRS from the
client-api package, plus any API-drift reconcile against pinnacle 0.2.3). NONE of the styling
below matters until 142 makes the custom config actually load. So: LAND 142 first, confirm
custom keybinds fire in a real Pinnacle greetd session, THEN this day begins. (Note: Brave is
already reachable in Pinnacle by typing `brave` in a terminal -- brave is on PATH -- so the
communicate-goal has a fallback even pre-142; 142 is about the KEYBINDS.)

## Build checklist (rough order -- after 142 lands, each verified before the next)
1. CONFIRM 142: custom config loads under greetd, custom keybinds (Super+B brave, Super+E broot,
   Super+P bar) fire live. This is the foundation gate -- do not proceed until green.
2. FULL KEYBIND SET: the June-13 init.lua already contains brave/broot/bar binds, tags, layouts,
   media keys, borders. Audit it against mango's keybinds; fill gaps so muscle memory carries
   across profiles. Reconcile any API drift (pinnacle 0.2.3) surfaced by 142's Layer 3.
3. FROSTED-GLASS AESTHETIC: translucent/see-through look -- lighter palette, blur/opacity on
   panels and borders, airy. Pinnacle (Snowcap/Smithay-based, Rust) has its own theming surface
   (borders, gaps, decorations via the Lua API). Design Q: what does Pinnacle's API expose for
   opacity/blur, and how far can "frosted" go natively vs needing a compositor-side effect.
4. OWN FROSTED ALACRITTY THEME: build a Pinnacle-specific Alacritty theme (light frosted bg,
   translucent, high-legibility) DISTINCT from both mango and Miracle's dark-glass. Same
   per-profile-theming design question as Miracle Day (how Pinnacle gets its own alacritty config
   without disturbing mango's tracked one) -- resolve consistently with whatever Miracle Day chose.
5. OWN PINNACLE-BAR: build a SEPARATE bar for Pinnacle (pinnacle-bar, genuinely distinct from
   miracle-bar -- Christian wants separate bars per profile). Styled frosted/translucent to match.
   Pinnacle's config already references a faelight-bar bind (Super+P) -- recon what that launches
   and whether it fits the frosted look or needs its own build. Ties INT-067.
6. WALLPAPER: bright airy frosted forest scenery (light, translucent-feeling), set at session
   start.
7. HOME-MANAGER TRACK (final settling step): the entire ~/.config/pinnacle/ (pinnacle.toml +
   lua/init.lua) is currently UNTRACKED -- 142 flags this same reproducibility gap. Once dialed
   in, track the whole Pinnacle config + frosted alacritty theme + pinnacle-bar into
   home-manager (xdg.configFile pattern). Retire the stale ~/.local/share/pinnacle/protobuf
   symlink (142 Layer 2 makes it obsolete).

## Design questions to resolve ON the day (do not assume)
- What Pinnacle's Lua API exposes for translucency/blur/frosted effects (native vs not).
- Per-profile Alacritty theming mechanism (resolve consistently with Miracle Day's choice).
- pinnacle-bar: what the existing Super+P faelight-bar bind launches; reusable or fresh build?
  Recon INT-067 state.
- API-drift reconcile scope (any init.lua vs pinnacle 0.2.3 mismatches from 142 Layer 3).
- How "see-through" is achievable -- may be limited by Pinnacle/Snowcap's current capabilities.

## Success criteria
- [ ] INT-142 landed: custom config loads under greetd, custom keybinds fire (foundation)
- [ ] Full keybind set mirrored from mango -- each fires in a real Pinnacle session
- [ ] Frosted-glass aesthetic renders: translucent/see-through borders + panels, light airy palette
- [ ] Own frosted Alacritty theme renders in Pinnacle; mango's + Miracle's themes UNCHANGED
- [ ] pinnacle-bar shows under Pinnacle (own bar, frosted-styled) -- ties INT-067
- [ ] Frosted forest wallpaper set at session start
- [ ] Entire Pinnacle config tracked in home-manager; survives rebuild + fresh login; stale
      protobuf symlink retired
- [ ] Daily-drivable as a scenery profile: switch in, communicate, switch out, nothing lost

## Dependencies / relationships
- HARD dep on INT-142 (Pinnacle custom config loads under greetd) -- must land first.
- INT-067 (faelight-bar under secondary compositor) -- the pinnacle-bar piece lives here.
- Sibling: the "Miracle Day" intent (INT-154, dark glassy forest) -- same structure, opposite
  mood (dark glass vs frosted glass). Resolve shared design questions (per-profile alacritty
  theming, bar tech) consistently across both.
- Mango (primary work profile) is NOT touched -- guard against changes leaking into mango's
  tracked config.

## The Rule
"Same forest, different light. Miracle is the dark glass; Pinnacle is the frost you see through." 🌲

## Dependencies

**INT-142** -- Pinnacle custom config does not load under greetd until the protos and
XDG_DATA_DIRS reconcile lands. Stated in this intent title since filing. The frosted-glass
profile is a full keybind set, and the keybinds are what 142 fixes.
