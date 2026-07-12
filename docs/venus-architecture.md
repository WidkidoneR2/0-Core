# Venus Architecture -- Living Brainstorm

**Status:** in-progress, always. Lives with INT-136. NOT committed to git -- a thinking
artifact, not a deliverable. Append freely. Nothing here is a promise until it becomes a
gate in an intent.

**Rule of this doc:** demonstrated-not-declared applies to architecture too. Every claim is
flagged for confidence. HIGH = we have evidence. UNCERTAIN = a bet. No hope dressed as plan.

**Decision locked (2026-07-12):** Venus is **Venus-A** -- forest-awareness built ON the
Zellij multi-pane substrate. NOT a from-scratch multiplexer (Venus-B, rejected -- see section 5).

---

## 0. What Venus actually is

Venus is a **forest-native workspace**: a terminal that hosts, in one surface, the frontend
you're working in + the structure of what you're editing + the live state of the system
(including a rebuild in real time) -- and every pane *knows the forest* (intent ledger, health
score, Friday). A terminal that HOSTS thinking, not one that thinks.

**The corrected scope (post-recon):** the hard parts are already solved by others. The
multi-pane surface exists (Zellij). The correct terminal substrate already exists in your repo
(faelight-term v3 -- INT-136). The ONLY part worth building yourself is the part nobody can
copy: **forest-awareness** -- because nobody else has the forest.

Venus = 136 terminal (renders) + Zellij (panes) + forest-awareness (yours alone).

---

## 1. The Layer Map (what / language / WHY)

- **Substrate: terminal** -- VTE parse, grid, PTY, GPU text, wcwidth, Nix contract. Rust.
  WHY: INT-136. faelight-term v3 already IS the Mars model -- less delta than Mars over Rio.
  Nothing to fork; finish it.
- **Substrate: workspace** -- multi-pane, sidebars, command panes, layouts, session-resurrect.
  Rust (Zellij). WHY: Zellij is full-time-maintained, WASM-extensible, terminal-AGNOSTIC by
  design ("a workspace layer on top of your terminal"). Proven by Yazelix. Do NOT rebuild.
- **Frontends** -- fsh / emacs / vim / nixcats as panes/tenants. Each its own language.
  WHY: Zellij hosts any command as a pane. Multi-frontend is a *layout* problem, not new code.
- **Forest-awareness** -- intent bar, live-rebuild pane Friday reads, structure pane,
  health-tint. WASM (any language -> WASM) + fsh. WHY: THE DIFFERENTIATOR. WASM plugins are
  polyglot -> satisfies "beyond Rust, many languages." This is what's yours alone.
- **Reasoning** -- pattern/suggestion engine. Rust (Friday). WHY: a service the shell/plugin
  layer CONSULTS. Never in the terminal or multiplexer. Lets Friday churn for years without
  touching Venus.

**Load-bearing decisions (both HIGH confidence):**
1. Friday sits behind the frontend/plugin layer, never inside terminal or multiplexer -> both
   evolve independently.
2. Zellij is the pane substrate, not our code -> we inherit years of bug corpus + WASM
   extensibility instead of reproducing it (the exact mistake 136 warned about re: Alacritty).

---

## 2. The Three Panes (Christian's core vision, mapped to Zellij primitives)

- **Frontend pane** -- work in fsh/emacs/nixcats. Maps to: `pane command="..."` in a KDL
  layout. Forest-awareness: fsh already forest-aware (intent ledger, Friday).
- **Structure pane** -- "see the structure of the file I'm working at." Maps to: `strider`
  sidebar (file tree) OR a WASM plugin (symbol outline). OPEN: tree vs in-file outline (Q2).
- **Live-system pane** -- "see the rebuild in real time." Maps to: a **command pane** running
  `nixos-rebuild`. Forest-awareness: a WASM plugin parsing `--log-format internal-json` into
  a live progress/derivation view Friday interprets.

The three-pane workspace = a forest-native KDL layout + forest-awareness plugins. Yazelix
already proves the shape (editor center, Yazi tree sidebar, popups). Venus does it
forest-native: the panes know the intent ledger and the rebuild, not just files.

---

## 3. The Nix Contract (generalized from 136)

136's contract was single-frontend (shell = "faelight-shell"). Venus generalizes it to
declare a *workspace*: which frontend, which layout, shared palette, forest-awareness toggles.

BRAINSTORM sketch (not committed):

    programs.venus = {
      enable = true;
      terminal = "faelight-term";              # INT-136 substrate
      workspace = {
        engine = "zellij";                     # the pane substrate
        layout = "forest";                     # a forest-native KDL layout
        frontend = { command = "faelight-shell"; fallback = "bash"; };
        panes = {
          structure = "symbols";               # or "tree"  (Q2)
          liveSystem = true;                   # the rebuild-watch pane
        };
      };
      palette = config.lib.stylix.colors;      # single source of truth -- INT-091
      cursor  = { style = "beam"; healthTint = true; };
      forestAware = {
        intentInStatusBar = true;
        rebuildPane = true;                    # Friday reads the live build
        healthTint = true;
      };
    };

This module *generates* the KDL layout + wires the WASM plugins + sets the 136 terminal
contract. The whole workspace becomes declarative and committable -- Yazelix's flake/HM
pattern, forest-native. (Confidence: HIGH -- Nix codegen over proven pieces.)

---

## 4. The Differentiator (uncopyable)

From 136, stated right: not speed, not features -- integration. Extended by the recon:
Alacritty doesn't know the intent ledger. Mars doesn't know the health score. Zellij doesn't
know your rebuild's meaning. Venus's panes know the forest. Concretely:
- Intent ledger in the workspace status bar (active intent, health, generation)
- A **live-rebuild pane** where Friday interprets nixos-rebuild output as it streams (which
  derivation, what's left, did it fail, what the failure means) -- genuinely novel
- Structure pane tied to *your* repo semantics
- Health-tinted chrome

Nobody can copy this because nobody else has the forest. That's the whole strategy.

---

## 5. Why NOT Venus-B (own multiplexer) -- recorded honestly

Rejected 2026-07-12. Building our own multi-pane engine = reproducing Zellij: full-time
maintained, WASM-extensible, session-resurrection, battle-tested pane management. Years of
unknown-unknowns (136's Alacritty warning, tripled). We'd spend those years to arrive where
Zellij already is, minus Zellij's accumulated bug corpus. "Bigger" must mean *more uncopyable
value*, not *more code reproducing solved problems*. Venus-A puts every hour into
forest-awareness -- the only part that's ours. If Zellij ever blocks us at the substrate, the
fallback is a bounded fork of Zellij (Rust, we can), not a from-scratch build. Not now.

---

## 6. Open questions / honest probability

- Multi-pane substrate is solved (Zellij) -- HIGH. KDL layouts + command panes + WASM plugins.
  Yazelix proves the exact shape.
- 136 terminal is the right base -- HIGH. 136 recon: already the Mars model, less delta.
- Forest-awareness is the uncopyable value -- HIGH. Nobody else has the ledger/health/Friday.
- Live-rebuild pane is feasible -- MEDIUM-HIGH. nix has --log-format internal-json
  (structured, parseable). Specifiable -> groundable, not unknown-unknown.
- Q1: multiplexer in-terminal or on-top? -- RESOLVED: ON-TOP (Zellij), per layer discipline.
- Q2: structure pane = file tree or in-file symbol outline? -- OPEN. Christian said "structure
  of the file I'm working at" -> leans symbol-outline (LSP document-symbols), NOT a file tree.
  Needs confirmation. Different plugin either way.
- Q3: does multi-frontend need NEW code? -- LIKELY NO. emacs/vim/nixcats already run as Zellij
  panes. Multi-frontend ~= a KDL layout + Nix wiring. Big imagined project may collapse to
  "good layouts + modules."
- AI multiplier on plugin/layout work -- UNKNOWN, measured. 136 pre-records this. Best case on
  specifiable pieces (KDL, Nix codegen, json-log parsing); near-zero on unknown-unknowns.
- Depends on Zellij's plugin permission model maturing -- UNCERTAIN. Was "coming"; verify
  current state before deep plugin work.

---

## 7. Sequencing spine (Christian's timeline)

- **MORNINGS / warm-up:** 134 (finish fsh roadmap) -> 144 (per-project scope) -> 118 (Friday
  engine resumption -- "get Friday to think Nix"). fsh IS Venus's primary frontend -> directly
  upstream.
- **WEEKEND WIN:** 027 (VM-native dev -- the safe workshop, "all high-risk work in VMs first")
  + 059 (Lanzaboote / Secure Boot). NOTE: 059 is Everglow phase 0, not a throwaway. The
  weekend seeds the month-later epic.
- **NEXT MONTH / big:** 078 (Everglow/Faelight-boot) -- a generation-aware Secure Boot manager
  as a Rust EFI app that absorbs 059. Wants its OWN architecture doc (its note says so), like
  this one. Symmetry with Venus: Everglow = a bootloader that knows the generation tree; Venus
  = a terminal that knows the intent ledger. Both: a standard thing made forest-aware.
- **THEN / Venus proper:** 136 (finish the 8 gates, Gate 3/wcwidth is most of the work) ->
  forest-native Zellij layout + forest-awareness plugins + the programs.venus contract. Late
  because it's the integration of everything upstream (needs mature fsh + Nix-native Friday +
  the VM workshop to build safely in).

**Related intents that feed Venus:** 014 (faelight-dashboard v2, ratatui forest surface --
likely a forest-awareness plugin or the structure/system pane), 026 (Forest Observatory --
event timeline, feeds the live-system pane), 042 (natural-language-rebuild -- feeds the rebuild
pane), 091 (Stylix palette -- the single color source the contract reads). Not load-bearing
yet: 110/112 (v2 metal-isolated) -- 110 is still a template stub.

---

## 8. Live Log (append-only)

- **2026-07-12** -- Doc created. Venus-A locked (forest-awareness on Zellij, not own
  multiplexer). Prior-art recon decisive: Zellij = KDL layouts + command panes + WASM plugins
  + terminal-agnostic; Yazelix (luccahuguet, same author as Mars) proves the exact three-pane
  forest-IDE shape via flake/HM. Reframed Venus from "build a multi-pane terminal" to
  "forest-awareness on proven substrate." Three panes mapped to Zellij primitives; the
  live-rebuild pane (nix internal-json + Friday) is the novel piece. Contract generalized to
  programs.venus. Everglow (078) recognized as parallel forest-aware epic (absorbs 059),
  wants its own doc. OPEN: Q2 structure-pane = file tree vs symbol outline (leans outline).
  TODO before deep work: verify Zellij plugin permission model current state.
