# Next Session Mission (planned 2026-06-26)

Sequence -- close-before-open (honors Friday's focus>speed flag):

1. **Close INT-092** (cheatsheet v2) -- finish remaining: Phase 2 (optional liveness
   polish -- decide build-or-drop) + description curation decision. Then cicomplete.
2. **Close INT-088** (Nix Inspector) -- assess Phase 3 (themed TUI): build / carve to
   new intent / close on the CLI being complete. Then cicomplete.
3. **Then open the VM WEEKEND MISSION** -- clean, focused in-progress list:
   - INT-024 (R&D VM, complete) = linchpin, unlocks the cluster
   - Forest-native login: INT-005 (ReGreet), INT-056 (recovery), candy-neon session
     boundary; INT-086 (Pinnacle removal); if it sings → INT-078 (Everglow)
   - INT-043 (Cachix/crane cache) -- VM may unblock its closure (safe throwaway env to
     prove cache config). VERIFY 043's actual parked-reason vs what the VM provides.

Discipline: VM/login is lockout-class -- VM-first, one verified step, facts over guesses.
Helix stays primary if nixvim (090) curiosity surfaces -- it's a side-channel.

## Idea to formalize later: focus-rotation cadence
Christian's proposal -- dedicated ~2-week focus blocks, one domain at a time, to enforce
sustained immersion over context-switching (the structural answer to Friday's focus>speed flag):
  VM/system (this weekend) -> Friday (deep RL/world-models work) -> shell/fsh -> Faelight
  Forest system -> repeat.
Why: deep work (esp. Friday's theory) needs immersion, not scattered hours. Rotation = full
presence in one domain at a time.
Caution agreed: keep it a DEFAULT rhythm, not a rigid law -- flex when work is mid-flow or a
domain isn't ready. Don't cut singing work short for the calendar.
DECISION DEFERRED: try the VM block first, see how sustained focus actually feels, THEN decide
if this earns a formal decisions/ record. Demonstrated-not-declared.

## RESUME HERE: virtio-gpu swap for the login-test VM (INT-056)
State at break: greetd+mango committed to hosts/vm/configuration.nix (031b72c7). VM boots
real login flow (greetd -> tuigreet --cmd mango), login works, mango runs on tty1 + gets a
seat. SSH rescue path proven. BLOCKER: qxl GPU doesn't route wlroots output/input through
SPICE -> black screen.
NEXT STEP: switch the VM's virtual GPU qxl -> virtio-gpu (no install -- it's a QEMU device
flag). In pkgs/faelight/scripts/vm, cmd_gui builds QOPTS with `-vga qxl -spice ...` (~line 166).
Plan: (1) read cmd_gui fully, (2) try a ONE-OFF launch with virtio (-vga virtio or
-device virtio-vga-gl) instead of qxl -- nothing committed, (3) check SPICE window: does mango
paint? does Super+Return spawn alacritty? (4) if yes -> make permanent (script edit/option);
if no -> keep diagnosing. Guest already has virtio drivers (kernel default). qxl stays right
for the headless watching loop (INT-080) -- this may need a separate graphical-GPU path/flag.

## Working convention (adopted 2026-06-26): human-legible abort/skip messages
Edit-script abort/skip messages must state the SITUATION and its IMPLICATION, not raw match
counts. They are for Christian reading the output, to diagnose without a full file search.
  BAD:  "ABORT: found 0x, already=True"
  GOOD: "SKIP: INT-089 warning already in main.rs (applied by a prior run). No change needed --
         verify with cargo check."
Pattern: say WHAT stopped it, WHETHER that's a problem, and the NEXT step to confirm.
Same spirit as INT-089 itself: an error that blames the wrong thing (or is cryptic) is worse
than none. Aborts should help locate the issue, not force reverse-engineering the logic.

## Convention correction (2026-06-26, from INT-096): reload picks up deploys
The "close + reopen terminal after a bundled-crane-tool deploy" step is NOT required.
`reload` re-execs into the newly-deployed binary correctly (proven INT-096) AND now reports
what it loaded: "New fsh build detected -- was <hash> / new <hash>" on a real change, or
"Already on the current fsh build ... nothing new" when unchanged. So the deploy loop is:
  commit -> rebuild -> deploy -> reload   (reload, not close+reopen).
(Claude repeatedly told Christian to close+reopen today before this was understood -- that was
unnecessary friction; reload alone suffices.)
