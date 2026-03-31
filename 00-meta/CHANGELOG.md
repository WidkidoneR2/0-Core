# Changelog

## [11.5.0] — The Shell Awakens (2026-03-30)

### 🎯 Completed Intents
- **INT-120** — faelight-shell — Forest-Native Shell Environment
- **INT-126** — Core v8 — Evolution: The Forest Refines Itself
- **INT-136** — Faelight Forest — Visual Identity & Niri Cosmetics
- **INT-140** — Core v10 — Reaction: The Forest Responds Without Being Asked
- **INT-146** — faelight-shell v2 — The Shell Becomes the OS
- **INT-148** — Core v11 — Prediction: The Forest Anticipates
- **INT-149** — Tool Retirement Sprint — Clean What the Core Has Absorbed
- **INT-151** — Core v12 — Strategy: The Forest Plans Across Horizons
- **INT-153** — Intent Genealogy — The Forest Remembers How It Grew
- **INT-155** — faelight-shell Prompt Themes — The Shell Has a Face
- **INT-162** — Shell Architecture Hardening — The Foundation Must Be Solid
- **INT-163** — Alias Audit — One Concept, One Command
- **INT-164** — Core Deploy Pipeline — Versioned, Immutable, Rollback-Safe
- **INT-165** — fsh Welcome Screen — Truth Only, No Stale Data
- **INT-166** — state.db Backup and Recovery — Protect the Forest's Memory
- **INT-171** — Pre-Command Decision Layer — The Shell That Understands Before It Executes
- **INT-172** — Shell Config Stow — config.fsh Under Version Control
- **INT-173** — Command Registry — The Shell Knows What It Can Do
- **INT-174** — Structured Errors — The Shell Explains Its Failures

### ✨ Features
- fsh — core subcommand shortcuts, predict/react/stress/doctor/goals native, no prefix needed
- fsh — jarvis theme Phase 22, prediction inline in prompt, INT-136 + health state visible
- fsh Phase 23 — session persistence, directory restored on startup
- fsh Phase 25 — faelight-term launches fsh by default, falls back to zsh
- faelight-release — docs section added, newest first, live health, scope grouping

### 🔧 Fixes
- README changelog link — full GitHub URL to 00-meta/CHANGELOG.md
- fsh welcome — reads live health from ~/.cache/faelight/health-status
- INT-163 — aliases.zsh 446→412, zero duplicates, 21 stale aliases removed (dot-doctor, sway, v9.3.0)
- INT-163 — alias-audit removes retired dot-doctor + bump-system-version, array size 34→32
- INT-163 — synonym cleanup, 446→368 aliases, zero duplicates, one concept one command
- d alias points to core doctor run directly, doctor synonym removed INT-163

### 📚 Documentation
- README rewritten — core v10+v11 complete, accurate tool count, journey updated, chaos testing documented
- INT-157 faelight-docs v2, INT-158 Partner Vision — the forest becomes a genuine collaborator
- INT-159 faelight-context, INT-160 faelight-memory, INT-161 build order — path to partnership
- INT-146 Phase 21+22+23 gates updated
- INT-146 Phase 25 gate updated, phase priority order documented
- INT-162 Shell Architecture Hardening + INT-163 Alias Audit — foundation integrity before v12
- INT-163 updated — 100 alias target, COMMAND-GUIDE.md deliverable added
- INT-164 Core Deploy Pipeline + INT-165 fsh Welcome Truth — build and accuracy improvements
- INT-166/167/168/169 — state.db backup, prediction feedback loop, test suite, niri autostart audit
- INT-164 updated — symlink strategy, forest-status command, simplified deploy pipeline
- INT-164 versioned deploy architecture + INT-171 Pre-Command Decision Layer

### 🔩 Internal (52 commits)
- faelight-forest 11.4.0 live
- faelight-docs sync — v11.4.0 The Bloom
- INT-163: add COMMAND-GUIDE.md, clean config.fsh dead alias
- INT-163: add COMMAND-GUIDE.md, clean config.fsh, update gate checks
- INT-164: deploy pipeline, rollback, forest-status, clean warnings
- ...and 47 more internal changes

### 📊 Stats
- Health: 100%  ·  Commits: 1779  ·  Tools: 50 deployed  ·  Intents: 115 complete

---

## [11.4.0] — The Bloom (2026-03-26)

### 🎯 Completed Intents
- **INT-137** — Architectural Horizons — Known Future Limits
- **INT-141** — faelight-notify v4 — Freedesktop Spec, zbus, Wayland Native
- **INT-143** — faelight-digest — Morning Forest Summary
- **INT-148** — Core v11 — Prediction: The Forest Anticipates
- **INT-152** — Core v11 Stress Test — Verify Before v12 Builds On Top
- **INT-154** — Core Health Stress Test — Chaos Engineering for the Forest

### ✨ Features
- INT-146 Phase 21 — context-aware completion v2, dynamic intent IDs, core predict/react/stress, aliases
- INT-154 COMPLETE — health chaos stress test 5/5 PASS, forest is chaos-resilient
- fsh — theme command, minimal/forest/classic/jarvis themes, echo/cat/type builtins, INT-155 Phase 1
- fsh — echo builtin, cat builtin, type command, quote stripping fixed
- fsh — env command, clear builtin, c alias now forest-native
- fsh — which builtin, pwd builtin, welcome shows real in-progress intents
- fsh — pwd builtin, audit_scores deduped, faelight-search removed from scores
- INT-152 COMPLETE — Core v11 stress test 5/5 PASS, v12 foundation verified solid
- INT-136 Phase 1 — forest visual identity, gaps 12, focus-ring #a3e36b, border, rounded corners 8px, inactive opacity 0.92
- INT-141 COMPLETE — faelight-notify v4, freedesktop spec, all urgency levels, display_start fix
- INT-148 COMPLETE — Core v11 all 5 phases + INT-151 refined with v11 foundations, Jarvis score 65/100
- INT-148 COMPLETE — Core v11 Prediction Engine, all 5 phases, forest anticipates
- INT-148 Phase 3 — Intent Velocity, completion rate, backlog prediction, next intent
- INT-148 Phase 1+2 — Core v11 Prediction Engine: session patterns, health trajectory, cadence
- fsh — zoxide z command, cd feeds zoxide, grep/df pass through, tilde expansion, eza ls/ll, core PATH fix
- debug last/reactions/preexec + usage report — transparency commands answer complexity concerns
- INT-146 Phase 20b — git guardrail, yazi cd-on-quit, faelight-fm cwd-file support
- INT-143 complete — faelight-digest v1.0.0, replaces faelight-fetch, system+forest context
- INT-149 — faelight-search retired, source+binary+registry+aliases cleaned, 44/44 path resilience

### 🔧 Fixes
- INT-152 and INT-154 status corrected to complete
- INT-141 and INT-148 status corrected to complete in ledger
- doctor health % excludes core_protect — lock state is operational not a health issue
- fsh — grep/df/du pass through, tilde expansion in external args, forest_map trimmed

### 🔩 Internal (3 commits)
- remove niri config backup after INT-136 Phase 1 verified
- faelight-docs sync — v11.3.0 The Forest Grows, README and welcome updated
- update to 11.3.0

### 📊 Stats
- Health: 95%  ·  Commits: 1701  ·  Tools: 50 deployed  ·  Intents: 100 complete

---

## [11.3.0] — The Forest Grows (2026-03-25)

### 🎯 Completed Intents
- **INT-140** — Core v10 — Reaction: The Forest Responds Without Being Asked
- **INT-145** — faelight-docs — Living Documentation Engine

### ✨ Features
- faelight-shell — since command, comment stripping, fsh-deploy now builds release
- INT-140 COMPLETE — Core v10 Reaction Engine, all 6 phases, forest responds without being asked
- INT-140 Phase 5 — reaction narrative, story command, today arc with goal context
- INT-140 Phase 4 — reaction boundaries, health gates, bounds/audit commands, 4 guardrails
- INT-140 Phase 3 — goal-scoped reactions, active goal context enrichment on react run
- INT-140 Phase 2 — TOML reaction rules, enable/disable/add, human-editable config
- INT-140 Phase 1 — reaction engine, 6 rules, cooldown/discipline, history, explain
- INT-146 Phase 20 — zsh retirement audit, 28 aliases ported to config.fsh, stale binaries removed
- INT-146 Phase 18b — flow mode, conscious intent focus, prompt live-updates
- INT-145 complete — faelight-docs v1.0.0, status/check now agree, TOOLS.md deferred
- INT-146 Phase 17 — prompt v2, two-line forest prompt, git branch, alias recursion guard fixed
- INT-148 Core v11 Prediction, INT-149 Tool Retirement, INT-150 Docs Audit
- INT-147 faelight-voice — The Forest Speaks Aloud, Piper TTS via Rust FFI

### 🔧 Fixes
- faelight-shell — comment stripping, # handled correctly at line start and inline
- clippy — resolve all workspace warnings, faelight-shell/vault/gen/sandbox/wallpaper

### 🔩 Internal (13 commits)
- INT-146: Phase 18 complete — script arguments $1 $2 $#
- faelight-shell: Phase 18 — script arguments $1 $2 $# working in .fsh scripts
- INT-146: Phase 12 complete — pkg package helpers
- faelight-shell: Phase 12 — pkg package helpers (list, search, install, remove, update)
- faelight-shell: fix pkgs — remove take(100) limit, show all packages
- ...and 8 more internal changes

### 📊 Stats
- Health: 95%  ·  Commits: 1669  ·  Tools: 51 deployed  ·  Intents: 94 complete

---

## [11.2.0] — Will and Motion (2026-03-22)

### 🎯 Completed Intents
- **INT-109** — faelight-compositor — Rust Wayland Compositor on Smithay
- **INT-120** — faelight-shell — Forest-Native Shell Environment
- **INT-132** — faelight-vault — Forest-Native Credential Manager
- **INT-135** — faelight-shell Phase 11 — Forest Personality & Adaptive Intelligence
- **INT-139** — faelight-shell — Natural Language Pipeline Translation
- **INT-144** — v11.1.0 Release Gate — The Forest Speaks
- **INT-145** — faelight-docs — Living Documentation Engine

### ✨ Features
- INT-146 Phase 11 — pipes to external commands, forest data flows into less/grep/wc
- INT-146 Phase 10 — shell variables, let/export, dollar sign expansion
- INT-146 Phase 9 — signal handling, Ctrl+C kills foreground process cleanly, shell survives
- INT-146 Phase 8 — job control, background jobs, jobs/fg/kill, forest announces completion
- INT-146 Phase 16 — interactive improvements, editor config, history dedup, emacs mode
- INT-146 Phase 15 — config file, aliases and settings load from config.fsh on startup
- INT-133 Core v9 Phase 5 COMPLETE — intent autobiography, core autobiography narrate
- INT-133 Core v9 Phase 4 — dynamic prioritization, core prioritize run/explain
- INT-146 Phase 13 — redirection (> and >>), file output from any command or pipeline
- INT-146 Phase 14 — multi-command input (cmd1; cmd2; cmd3), d built-in alias
- INT-146 Phase 7 — external command execution, PATH passthrough, forest-aware suggestions
- INT-133 Core v9 Phase 3 — tradeoff engine, core tradeoff analyze/history/balance
- INT-133 Core v9 Phase 2 — task planning engine, core plan generate/review/simulate/list
- INT-133 Core v9 Phase 1 — goal engine, core goals generate/list/accept/reject/show
- INT-145 faelight-docs v1.0.0 — living docs engine, auto-sync on release, boundary rule enforced
- INT-132 complete — faelight-vault registered, aliases added (vault/fv/fva/fvl/fvg)
- INT-132 faelight-vault v1.0.0 — forest-native credential manager, Argon2id encryption, health scores, audit
- INT-146 faelight-shell v2 — Phase 7-32 defined, 10% to 100% daily driver path
- INT-109 Session 8 — auto chvt 7 after render, clean return to Niri
- INT-109 Session 8 — auto-set XDG_RUNTIME_DIR, no more RuntimeDirNotSet panic
- INT-109 Session 7 — VT switching via libseat session.change_vt(), session stored in state
- INT-109 Session 7 — VT switching Ctrl+Alt+F1-F7, clean exit Ctrl+Alt+Q
- Phase 10/15/16 complete — chart command, git.commits/files/branches aliases, history duration, histogram
- Phase 6 — .fsh scripting language, let bindings, if/when/emit/warn/confirm, run <file.fsh>
- INT-145 faelight-docs, ARCHITECTURE-FUTURE — tool retirement, core continuity, shell independence, self-building vision

### 🔧 Fixes
- INT-145 faelight-docs — fix README path to root, deploy correct binary, 68 tools/33 domains live
- INT-109 — chvt 1 (Niri on TTY1 not TTY7), clean return after render
- faelight-notify v4 systemd user service — auto-restart, remove Niri autostart

### 🔩 Internal (8 commits)
- ledger: INT-120 complete — faelight-shell Phase 1-32, remaining phases deferred to INT-146
- ledger: INT-109 COMPLETE — compositor renders forest green, returns to Niri cleanly
- ledger: INT-109 Sessions 5+6 complete — forest green on real hardware 2560x1600@165Hz AMD Radeon 780M
- remove faelight-notify v3 backup — v4 stable and running
- ledger: INT-135 complete — shell personality 5/6 criteria, Core v9 integration deferred to INT-133
- ...and 3 more internal changes

### 📊 Stats
- Health: 95%  ·  Commits: 1630  ·  Tools: 51 deployed  ·  Intents: 93 complete

---

## [11.1.0] — The Forest Speaks (2026-03-21)

### 🎯 Completed Intents
- **INT-126** — Core v8 — Evolution: The Forest Refines Itself

### ✨ Features
- INT-141 faelight-notify v4.0.0 — fontdue::layout renderer, clean text, D-Bus compliant, urgency levels
- Phase 25 — NL auto-diagnose, pipeline execution in diagnose, warning cleanup
- INT-143 Phase 1 — forest digest, morning summary on long gaps
- INT-139 custom TOML patterns — load from 01-registry/shell-patterns.toml
- INT-144 v11.1.0 release gate — The Forest Speaks, requirements documented
- Phase 22 — observability dashboard, system+forest panels
- Phase 21 — SQL query language, select/from/where/order by/limit
- Phase 18 — time travel, snapshot/timeline/snap-diff
- Phase 17 — event system, shell triggers, on/list/remove/enable/disable
- Core v8 Phase 6 — future simulation, risk analysis, impact analysis
- Core v8 Phase 5 — evolution proposals, evolve-propose/list/accept/reject
- INT-141 faelight-notify v4, INT-142 faelight-voice, INT-143 faelight-digest
- faelight-term — Ctrl+R atuin fix, full ctrl key map, ft alias, daily driver ready
- group pipe op — find | group ext, gchurn | group ext, et | group domain
- Phase 16 — history analytics, hstats command frequency, hpattern time-of-day
- Core v8 Phase 4 — architecture suggestions, CLI hotspot, coupling detection
- INT-140 Core v10 — Reaction, event bus, reaction rules, guided instinct
- Phase 15 — git churn engine (gchurn), branch table (gbr), hotspot detection
- INT-135 Pillar 3 — momentum detection, feat commits today, weekly streak
- Phase 14 — persistent file index, find command, 1009 files indexed
- Phase 15 — gc powered by faelight-git native git2 bindings, fully joinable
- faelight-shell — live join system, ps | join ports on pid, ad-hoc relational queries
- Core v8 Phase 3 — decision patterns, friction, reversal detection
- INT-135 Pillar 2 — adaptive shell modes, Recovery/Streak/Idle/Milestone/Focused
- session memory — show INT numbers instead of full titles on welcome
- INT-135 Pillar 1 — session memory, active intents, welcome back message
- INT-139 Layer 2 — fuzzy token matching, short aliases ?mem ?cpu ?commits ?ports
- faelight-shell v0.6.0 — status line on clear, ? pattern list, fsh-deploy alias, clean prompt
- faelight-shell — add c/cls as hardcoded clear aliases
- INT-139 Layer 1 — natural language pipeline translation, 35+ patterns, ?prefix
- faelight-shell Phase 11 — schema-aware Tab completion, pipe column hints
- faelight-shell Phase 11a — schema registry, 10 system tables, typed columns
- Core v8 Phase 1+2 — core evolution map/tools, architecture observation domain
- INT-126 Core v8 — layered build order, evidence-first phases, horizon monitoring
- INT-120 — four-layer architecture, Phase 11a schema system, corrected phase order
- INT-132 vault expanded, INT-138 compositor v2 EGL, INT-139 shell natural language
- INT-136 Phase 1 — forest green borders, rounded corners 8px, shadows, animations, 12px gaps
- INT-137 architectural horizons, INT-126 evidence rule added

### 🔧 Fixes
- INT-109 faelight-compositor — status corrected to in-progress, Sessions 5-8 remain
- faelight-term — bracketed paste mode, paste wraps with 200~/201~, remove premature 2004h
- faelight-term — force correct PTY size on first configure, btm/TUI apps now render correctly

### 🔩 Internal (2 commits)
- doc: INT-126 complete
- cistart 126 — Core v8 in-progress

### 📊 Stats
- Health: 95%  ·  Commits: 1586  ·  Tools: 49 deployed  ·  Intents: 86 complete

---

## [v11.0.0] — Where the Forest Becomes Whole (2026-03-17)

### 🎯 Completed Intents
- **INT-109** — faelight-compositor — Rust Wayland Compositor on Smithay
- **INT-122** — Core v7 — The Resilient Forest
- **INT-125** — faelight-sandbox v3 — Full Policy Engine & Deep Isolation
- **INT-134** — faelight-shell Phase 10 — Shell Personality & Living Welcome

### ✨ Features
- faelight-shell v0.5.0 — all INT-134 criteria complete, quote DB tracking, today's focus, exit fixed
- faelight-shell v0.5.0 — Ctrl+C, exit fixed, logs --follow, autocomplete, today's focus (INT-134)
- faelight-shell v0.5.0 — Ctrl+C proper, logs --follow streaming, schema autocomplete, double Ctrl+C exit (INT-134)
- INT-125 complete — seccomp syscall filtering, all isolation levels working
- INT-125 — disk I/O tracking, doctor sandbox check (24th), advise integration
- INT-125 complete — disk I/O tracking, doctor sandbox check (24th), advise integration
- faelight-shell Phase 10 — living welcome, graceful exit, forest quotes, v0.4.0 (INT-134)
- INT-134 shell Phase 10 personality, INT-135 Phase 11 adaptive intelligence
- faelight-shell Phase 9 — streaming pipelines, ps | watch, v0.3.0 (INT-120)
- INT-133 — Core v9 Intent, goal engine, task planning, tradeoff engine
- INT-109 Session 5 — FIRST RENDER COMPLETE, forest green #11140f on real hardware 2560x1600@165Hz
- INT-109 Session 4 — GBM device created, eDP 2560x1600@165Hz connector+CRTC found
- INT-109 Session 2 — DRM device enumeration, probe mode, AMD Radeon 780M identified
- Core v7 Phase 7 — deterministic rebuild, core doctor rebuild (INT-122
- Core v7 Phase 6 — snapshot narrative, two voices (markdown+JSON), faelight-snapshot retired (INT-122)
- Core v7 Phase 5 — forest narrative, core narrative/--intent commands (INT-122)
- faelight-shell Phase 8 — system tables, ps/ports/services/files/net/pkgs pipeable (INT-120)
- faelight-sandbox v3 Phase 3+4 — deep isolation (net/full), resource profiling (INT-125)
- Core v7 Phase 4 — dependency intelligence, graph/risk/audit, zero warnings (INT-122)
- Core v7 Phase 3 — security simulate command, CVE and package simulation (INT-122)
- faelight-release — auto-update tool counts on publish, remove archived tools

### 🔧 Fixes
- faelight-release auto-updates /etc/faelight/COMMITS on publish, faelight-login fix

### 🔩 Internal (16 commits)
- ledger: INT-134 complete — faelight-shell Phase 10 personality done
- ledger: INT-125 complete — faelight-sandbox v3 all 11 criteria met
- ledger: INT-109 complete — faelight-compositor first render, forest green on real hardware
- ledger: INT-109 Session 4 complete — eDP 2560x1600@165Hz, ready for first render
- ledger: INT-109 Session 3 complete — DRM device opened, hardware enumerated
- ...and 11 more internal changes

### 📊 Stats
- Health: 95%  ·  Commits: 1513  ·  Tools: 49 deployed  ·  Intents: 86 complete

---

## [v10.9.0] — Roots and Branches (2026-03-16)

### 🎯 Completed Intents
- **INT-127** — Schema Layer — Registry and Policy Validation
- **INT-128** — Domain Restructuring — Subdirectory Per Domain
- **INT-129** — Event Log Directory — File-Based JSONL Alongside SQLite
- **INT-130** — faelight-gen — Forest-Native Password & Secret Generator Suite
- **INT-131** — faelight-teach upgrade — Interactive faelight-shell Tutorial

### ✨ Features
- teach v5.0.0 — faelight-shell tutorial, 5 lessons, interactive prompt (INT-131)
- faelight-gen v1.0.0 — 12 generator types, colored output, entropy display (INT-130)
- INT-129 complete — JSONL event log, lifecycle policy, core events status/archive
- Core v7 Phase 2 — bootstrap intelligence, plan/verify/diff commands (INT-122)
- INT-128 — doctor domain restructured into subdirectories, checks/cockpit/schema split (INT-128)
- Core v7 Phase 1 — anomaly detection, core anomaly scan/history/alert (INT-122)
- INT-127 complete — 04-schema/ layer, JSON schemas for all registry files, doctor schema validation check
- faelight-shell Phase 5 — plugin system, .fsh plugin files, forest-utils plugin shipped (INT-120)
- faelight-shell — alias system with persistence, pipeline-aware expansion, 29 commands (INT-120)
- faelight-shell Phase 3 — security audit log, watch mode, histogram, git-commits/files, clear fix (INT-120)

### 🔧 Fixes
- faelight-release preview double v bug
- alias-audit tool count 43→50, reads from registry (INT-120)
- faelight-shell — ? alias conflict, panic guard, fmt_time helper, domain icons, did-you-mean suggestions (INT-120)

### 🔩 Internal (11 commits)
- ledger: INT-131 complete — teach shell tutorial
- ledger: INT-130 complete — faelight-gen shipped
- ledger: INT-129 complete — event log directory
- ledger: INT-122 Phase 2 complete — bootstrap intelligence
- ledger: INT-122 Phase 1 complete — anomaly detection
- ...and 6 more internal changes

### 📊 Stats
- Health: 95%  ·  Commits: 1469  ·  Tools: 51 deployed  ·  Intents: 82 complete

---

## [v10.8.0] — The Forest Between Worlds (2026-03-12)

### 🎯 Completed Intents
- **INT-118** — doctor facelift — cockpit-style health dashboard
- **INT-119** — core security advise — judgment layer for security decisions
- **INT-121** — faelight-readme — auto-update README dynamic sections on release
- **INT-123** — faelight-audit — Tool Intelligence Layer (core audit domain)
- **INT-124** — faelight-sandbox v2 — Forest-Aware Isolation Environment

### ✨ Features
- faelight-shell v0.2.0 — Phase 2 complete, full data pipeline, 26 commands (INT-120)
- faelight-shell — ht, ct, domains, select fix, 26 commands (INT-120)
- faelight-shell — fix select pipe, multi-stage pipeline working (INT-120)
- faelight-shell — decisions-table, count pipe, 22 commands total (INT-120)
- faelight-sandbox v3 Phase 2 — policy enforcement wired, network isolation active, policy logged to ledger (INT-125)
- faelight-shell Phase 2 — Value type system, data pipeline, where/select/sort/first/last, tt/et/at commands (INT-120)
- faelight-sandbox v3 Phase 1 — policy engine, 5 policies, --policy flag (INT-125)
- faelight-shell — search command, history search, 18 native commands (INT-120)
- core audit Phase 3 — stale tools surface in core advise (INT-123)
- faelight-shell — checkpoint and git commands, 17 native commands (INT-120)
- core audit Phase 2 — expected_usage calibration in tools.toml, 51 tools scored (INT-123)
- faelight-shell — forecast, sandbox commands, 15 total native commands (INT-120)
- faelight-shell polish — health version, events today filter, cd navigation, registry updated
- INT-120 Phase 1 — faelight-shell v0.1.0, forest REPL with 12 native commands (INT-120)
- INT-123 Phase 1 — core audit domain, scan/show/stale/coverage, audit_scores table (INT-123)
- faelight-sandbox v2 — ledger integration, audit trail, duration tracking (INT-124)
- faelight-compositor DRM/udev backend — running on real hardware, libseat+libinput initialized (INT-109)
- add DRM backend dependencies, seatd enabled, groups configured (INT-109)
- INT-119 complete — core security advise, judgment layer for security decisions
- INT-118 — doctor cockpit facelift, grouped sections, summary header (INT-118)

### 🔧 Fixes
- sandbox v3 — clean network display, no duplicate Network line when policy controls it
- remove duplicate audit alias, commit INT-123 intent update from cistart
- faelight-release syncs /etc/faelight/VERSION on publish — faelight-login stays current
- INT-118 complete — alias coverage clean message, ADVISORY status, single separator
- faelight-fetch double v prefix in forest header, deploy updated binary

### 🔩 Internal (13 commits)
- ledger: INT-122 — add structural considerations, organized growth principle before Core v7
- ledger: cistart INT-120 — faelight-shell in-progress
- ledger: add development philosophy to INT-120 — parallel track, no rushing, craft over speed
- ledger: INT-120 expanded — beyond NixOS vision, interactive prompt, .fsh language, full architecture
- ledger: expand INT-120 to full faelight-shell vision — Nu-inspired, security-native, structured data
- ...and 8 more internal changes

### 📊 Stats
- Health: 95%  ·  Commits: 1438  ·  Tools: 50 deployed  ·  Intents: 77 complete

---

## [v10.7.0] — The Forest Remembers (2026-03-12)

### 🎯 Completed Intents
- **INT-99** — Niri Migration & faelight-compositor — The Forest Grows Its Own Roots
- **INT-103** — faelight-idle — Rust Idle Daemon
- **INT-117** — Fix atuin zsh interactive parse warning on terminal open

### ✨ Features
- Core v6 complete — all 5 phases, INT-116 closed, aliases added (INT-116)
- Core v6 aliases — decide, advise, story, lessons, heuristics, simulate scenario
- Core v6 Phase 5 — extended simulation, core simulate scenario (INT-116)
- Core v6 Phase 4 — heuristics engine, core lessons, core story (INT-116)
- Core v6 Phase 3 — core advise judgment assist, context-aware advisory (INT-116)
- Core v6 Phase 2 — outcome correlation, decision show/stats, context hash matching (INT-116)
- Core v6 Phase 1 — decision ledger foundation, core decide/outcome/hindsight (INT-116)
- faelight-fm visual overhaul — rounded borders, accent highlights, rich status bar (v2.3.0)
- input handling wired, auto-focus on new_toplevel (INT-109)
- faelight-compositor now writes events to state.db — forest participant (INT-109)
- register faelight-compositor in registry, add fc/fcomp aliases (INT-109)
- faelight-compositor v0.1.0 — winit backend, proof of life achieved (INT-109)
- add faelight-compositor v0.1.0 — the last sibling comes home (INT-109)
- (idle) INT-103 complete — faelight-idle v1.0.0, ext-idle-notify-v1, replaces swayidle, event ledger

### 🔧 Fixes
- remove faelight-core from tools registry (library not binary), deploy verify-bootstrap
- (zsh) disable promptsubst during load, remove y alias conflict with ya() function
- (zsh) move functions.zsh before starship init — eliminates functions parse warning on terminal open
- (zsh) rename alias-help to alias_help — hyphenated function names cause parse errors
- (update) use core doctor run instead of dot-doctor, fix alias-help function definition
- (notify) improved layout, fractional scale support, JetBrainsMono font, baseline fix
- (aliases) doctor alias points to core doctor run instead of old dot-doctor binary
- (doctor) security hardening shows explicit UFW/fail2ban status instead of count

### 🔩 Internal (8 commits)
- config: fix nu home-dir variable in nushell config
- ledger: close INT-117, cancel INT-108, complete INT-099 Phase 1
- restore to 497e9ff
- revert(zsh): restore original functions.zsh — parse warning was pre-existing, not our bug
- retire faelight-dashboard — archived source, removed binary, aliases, audit fix
- ...and 3 more internal changes

### 📊 Stats
- Health: 95%  ·  Commits: 1396  ·  Tools: 49 deployed  ·  Intents: 72 complete

---

## [10.6.0] — The Judgment Layer (2026-03-08)

### 🎯 Completed Intents
- **INT-104** — faelight-wallpaper — Rust Wallpaper Daemon
- **INT-105** — core intent dashboard — Terminal Intent Overview
- **INT-107** — faelight-search — Unified Rust Search
- **INT-110** — core why visual — Workspace Topology in Event Ledger
- **INT-111** — faelight-bar — Fractional Scaling Support (wp_fractional_scale_v1)
- **INT-113** — Core v5 — The Intelligent System

### ✨ Features
- (core) Core v5 complete — all 5 phases, ledger foundation, forecasting, causality, patterns, compositor intelligence
- (core) Core v5 Phase 4 — pattern recognition, correlate domains, suggest based on learned history
- (core) Core v5 Phase 3 — causality engine, why health-since, causal domain analysis, causal chain
- (core) Core v5 Phase 2 — forecast line integrated into core doctor output
- (core) Core v5 Phase 1 — ledger foundation, indexed queries, stats/query/export commands
- (bar) INT-111 complete — fractional scaling via wp_fractional_scale_v1 + wp_viewport, sharp at 1.5x
- (intent) INT-105 complete — faelight-intent v1.0.0, intent dashboard with focus/checkpoints/planned
- (core) INT-110 — core why visual + attention, workspace topology in event ledger
- (search) INT-107 complete — faelight-search v1.0.0, unified search across files/intents/commits/events/aliases
- (wallpaper) INT-104 complete — faelight-wallpaper v0.1.0, health-reactive, replaces swaybg

### 🔧 Fixes
- (doctor) exclude Notesnook Singleton symlinks from broken symlink check
- (faelight-git) hardcoded version 3.2.0 → 3.3.0 in clap command
- (dashboard) ufw check reads /etc/ufw/ufw.conf — no sudo needed, accurate status
- (zshrc) update welcome message to v10.5.0 The Intelligent Forest

### 🔩 Internal (2 commits)
- add cwv/cwa aliases for core why visual/attention
- retire bump-system-version — faelight-release is the forest's release system. README updated to v10.5.0, 47 tools

### 📊 Stats
- Health: 95%  ·  Commits: 1363  ·  Tools: 50 deployed  ·  Intents: 69 complete

---

## [10.5.0] — The Intelligent Forest (2026-03-07)

### 🎯 Completed Intents
- **INT-93** — Core v3 — The Living System
- **INT-94** — faelight-term — VTE Refactor & Stability
- **INT-95** — faelight-browser — Stability & Feature Improvements
- **INT-100** — core pulse — Live Event Stream Terminal View
- **INT-101** — faelight-login — Rust Display Manager
- **INT-102** — faelight-clipboard — Rust Clipboard Manager
- **INT-106** — doctor forecasting v2 — Predictive Health Intelligence

### ✨ Features
- (release) INT-114 Phase 6 — learning layer, theme suggestions, anomaly detection, pattern analysis
- (release) INT-114 Phase 5 — rollback with generation diff, health check, event ledger
- (release) INT-114 Phase 4 — smart README writer, dynamic section auto-updated from manifest
- (release) INT-114 Phase 3 — faelight-release TUI, inline theme editing, publish flow
- (release) INT-114 Phase 2 — faelight-release v0.1.0, smart changelog engine, history, diff, preview
- (releases) INT-114 Phase 1 — generation structure, 10.3.0 + 10.4.0 manifests, generation pointer
- (forecast) INT-106 complete — faelight-forecast v1.0.0, predictive health intelligence
- (niri) add faelight-niri-bridge to Niri autostart — compositor events on every session
- (niri-bridge) INT-099 — faelight-niri-bridge v0.1.0, compositor events in event ledger
- (core) INT-099 WM abstraction — ControlSway→ControlWM, Niri detection in fetch/lock/entropy
- (pulse) INT-100 — faelight-pulse v1.0.0, live event stream TUI, health sparkline, domain filtering

### 🔧 Fixes
- (aliases) remove duplicate clipboard/pulse/niri-bridge block
- (aliases) resolve ff conflict — faelight-fetch→fae, remove duplicate forecast block
- (aliases) add forecast, pulse, clipboard, niri-bridge aliases to aliases.zsh
- (foot) rename [colors] → [colors-dark] — remove deprecation warning on boot
- (login) panel height 18 — no excess padding, pixel perfect
- (login) remove all warnings — unused imports, variables, constants cleaned up
- (doctor) keybind check now Niri-aware — detects niri/sway automatically

### 🔩 Internal (7 commits)
- replace cliphist with faelight-clipboard watch — zero C clipboard deps
- fix all status frontmatter — correct format, 099 in-progress, 100/101/102 complete
- INT-067 archived, 093/094/095 complete, 111 niri-keybind → incidents/112
- INT-102 complete — faelight-clipboard v0.2.0, native wlr-data-control, zero C dependencies
- INT-102 in-progress — v0.1.0 shipped, 3/5 criteria met
- ...and 2 more internal changes

### 📊 Stats
- Health: 95%  ·  Commits: 1344  ·  Tools: 47 deployed  ·  Intents: 63 complete

---

## v10.4.0 - 🌲 Niri Version — The Forest Finds Its Roots (2026-03-03)

- Core v4 complete — checkpoint, recovery, intent discipline, security debt, analytics
- Niri 25.11 as primary compositor — INT-099 Phase 1
- faelight-login v1.0.0 — Rust greeter replaces tuigreet
- faelight-notify v3.0.0 — Unix IPC socket, DND control, dismiss
- faelight-notifyctl — new IPC controller tool
- Full keybind migration Sway → Niri
- eDP-2 output, 2560x1600 @ 165Hz, 1.5x scale
- Brave browser native Wayland
- Fn media keys working
- Niri Version greeting in shell

**Tools Updated:** faelight-login, faelight-notify

**Statistics:**
- Commits: 1305
- Tools: 35 deployed, 43 with aliases
- Health: 95%
- Intents: 112 total, 69 complete
- Rust tools: 42 custom binaries
- Lines of code: 118000+

- System Health: 95%
- Commits: 24
- Files Changed: 13

---


## v10.3.0 - 🧠 Core v3 — The Living System (2026-02-26)

- Core v3 Phase 1: Event Ledger — doctor, git, security, update write structured events to SQLite
- Core v3 Phase 2: Causality Engine — cw/cwh/cwd/ctr/ctrd trace why system is in current state
- Core v3 Phase 3: Simulation Engine — csd/csu dry-run predictions without mutation
- Core v3 Phase 4: Event Bus — faelight-daemon v3.0.0 SQLite polling + live broadcast (cew)
- Core v3 Phase 5: Plugin Registry — ecosystem manifest with domain mapping (cpl/cpa/cps)
- Core v3 Phase 6: Health Forecasting — trend analysis + trajectory prediction (cdt/cdf)
- bump-system-version v9.4.0 — cache health, real rollback, fixed commit count, event ledger write
- faelight-sandbox v1.1.0 — correct change count, reflink=auto, session history ring buffer
- faelight-fetch v2.3.0 — live resources, cache health, term detection, 0-core stats
- faelight-update v3.3.0 — LazyVim support, safe pacman checker, smart health gate
- faelight-git v3.3.0 — lazygit retired, native commit, file-level status
- teach v4.0.0 — live system narrator, persona detection, presentation mode

**Tools Updated:** faelight-daemon, faelight-fetch, faelight-git, faelight-update, teach, faelight-sandbox, bump-system-version

**Statistics:**
- 343 aliases across 43 tools
- 1280 commits indexed
- 19 commits since v10.2.0
- 6 new core commands: cew, cpl, cpa, cps, cdt, cdf
- faelight-daemon v3.0.0 with pub/sub event broadcast
- 5 plugins registered in ecosystem registry

- System Health: 95%
- Commits: 19
- Files Changed: 38

> "The forest doesn't just run — it thinks, remembers, and forecasts." — Intent 093

---


## v10.2.0 - 🌲 Two Tools Out of WIP — faelight-term & faelight-browser production ready (2026-02-26)

- ENDfaelight-term v10.3.0 — OUT OF WIP
- - vte crate parser (same as Alacritty/Zellij)
- - Alternate screen buffer (nvim/fm full size)
- - Dynamic resize via TIOCSWINSZ
- - Backspace, delete, escape, return key handling
- - DSR cursor position response (atuin inline history)
- - Production ready — actively replacing foot
- faelight-browser v0.4.0 — OUT OF WIP
- - w3m-style inline link navigation
- - Forward/back navigation (Shift+F/B)
- - Reader mode strips nav/ads (Shift+R)
- - In-page search Ctrl+F with match count
- - Unicode panic fixes
- - Brave web search integration
- System
- - 9 new tools: atuin, tokei, hyperfine, tealdeer, ouch, difftastic, cargo-flamegraph, bottom, onefetch
- - Removed: qutebrowser, fuzzel, mako (replaced by faelight tools)
- - Added helix to packages
- - Fixed infinite loop in update domain
- - Intents 094/095 marked complete
- - 19 commits since v10.1.0

**Tools Updated:** faelight-browser, faelight-term

**Statistics:**
- Tools: 43 active (2 out of WIP this release)
- Commits: 1260 total (19 since v10.1.0)
- Intents: 62 complete
- Packages: 97 system packages
- Health: 95%
- Lines of Rust: 30,976+ across 205 files

- System Health: 85%
- Commits: 19
- Files Changed: 10

---


## v10.1.0 - 🌲 The Forest Matures — 0-Core v2 Complete (2026-02-22)

- END0-Core v2 architecture migration complete — all 6 phases done
- Phase 5: alias-audit, bin-doctor, entropy-check absorbed into doctor domain
- Phase 6: capability model enforced across all 15 domains — every operation logged
- faelight-menu v4.0.0 — forest palette aesthetic matching faelight-palette
- faelight-palette v3 — split layout, real 0-Core stats, health cache integration
- faelight-fm — rich preview panel with zone/git/intent metadata
- faelight-link v3 — adopt command, GNU Stow dir-folding awareness
- Health cache — single source of truth across bar, palette, prompt, doctor
- All hardcoded paths removed from engine — fully portable
- Cold start: 3ms

**Tools Updated:** faelight-menu

**Statistics:**
- Tools: 34/34 deployed (100%)
- Aliases: 318 total
- Commits: 1240
- Health: 95%
- Domains: 15 native Rust domains
- Sub-tools absorbed: 3 (alias-audit, bin-doctor, entropy-check)
- Cold start: 3ms

- System Health: 90%
- Commits: 37
- Files Changed: 27

---

## v10.1.0 - 🔧 Phase 3 Complete — Engine Hardened (2026-02-22)

### 0-Core v2 Phase 3 — Cutover Complete
- faelight-link fully native: adopt command, GNU Stow dir-folding awareness
- faelight-palette v3: split layout, real 0-Core stats, health cache integration
- faelight-fm: clean filelist, rich preview panel with zone/git/intent metadata
- faelight-fm: SCR scratch zone configured (~~/scratch), UTF-8 panic fixed
- faelight-launcher + faelight-dmenu: fully removed, superseded by palette
- core update: scripts path fix, allow_hyphen_values flag passthrough
- doctor: writes health cache after every run — bar/prompt/palette now in sync
- All hardcoded /home/christian paths removed from engine — fully portable
- All target/release paths removed from scripts — thin wrappers only
- Cold start measured: 3ms (target was <50ms) ✅
- runtime/ isolation complete: gitignored, state.db, logs, cache, locks, snapshots
- Root VERSION file created: 0-Core v2 engine = 2.0.0
- ARCHITECTURE.md rewritten to reflect v2 reality

### Phase 4 — Cleanup
- docs/ARCHITECTURE.md fully updated for v2 layer model
- Intent 092 updated: Phase 3 marked complete, Phase 4 NEXT

**Statistics:**
- Tools: 34/34 deployed (100%)
- Aliases: 318 total
- Commits: 1232
- Health: 95% (core unlocked warning only)


## v10.0.0 - 🏛️ v10.0.0 — The Migration Complete (2026-02-21)

- END- core v2.0.0 single orchestrator binary replaces all v1 tool delegation
- - All 15 domains implemented natively in Rust (no shell scripts with logic)
- - Phase 1-6 migration complete: scaffold, wiring, native, runtime, cleanup, capabilities
- - Runtime locking prevents concurrent core processes
- - Capability model enforces domain permissions at dispatch time
- - JSONL audit log at runtime/logs/capabilities.jsonl
- - 36/36 tools deployed at 100% path resilience
- - Removed deprecated rust-tools: dot-doctor, security-audit, recent-files
- - Consolidated runtime to runtime/ (removed legacy 04-runtime/)
- - 0c alias for cd ~/0-core, core alias now correctly points to binary

**Tools Updated:** recent-files, security-audit, dot-doctor

**Statistics:**
- Tools: 36/36 deployed (100%)
- Aliases: 318 total
- Intents: 92 total (55 complete, 8 planned)
- Domains: 15/15 native
- Health: 95% locked / 90% unlocked

- System Health: 100%
- Commits: 29
- Files Changed: 24

---


## v9.9.0 - 🌲 The Forest Grows — Visual Intelligence Update (2026-02-20)

- Real-time status bar with profile-aware color theming
- Profile system: DEF/GAME/WORK with live color propagation across bar
- Zone detection: automatic directory-based zone display
- Core lock status: direct lsattr verification, color-coded
- System health: cached gradient display (green/amber/red)
- Battery charging indicator with level-based color grading
- WiFi: live SSID display, green connected red disconnected
- Volume: actual percentage with mute detection
- VPN: mullvad status, green on red off
- faelight-hooks v10.2.0: status command, graceful tool detection
- core-protect v2.3.0: verify command, audit log, speed fix
- faelight-fm v2.3.0: persistent preview, full-width highlight
- dot-doctor v4.1.0: core-protect integration, path resilience
- faelight-update v3.2.0: path fixes, dead code removed
- faelight-notify v2.1.0: border urgency fix, close notification
- faelight-fetch v2.2.0: double-call bug eliminated
- faelight-palette v2.2.0: command palette with app launching
- faelight-menu v3.0: complete ratatui rewrite
- 43 stale markdown files cleaned from repository

**Tools Updated:** dot-doctor, faelight-fm, faelight-fetch, faelight-launcher, faelight-notify, faelight-bar, faelight-term, faelight-update

**Statistics:**
- 43 commits since v9.8.0
- 8 tools improved with bug fixes
- 1 tool completely redesigned (faelight-bar)
- 43 stale markdown files removed
- 38/38 tools deployed at 100% path resilience
- 20/20 health checks passing
- 118000+ lines of Rust across 42+ tools

- System Health: 95%
- Commits: 43
- Files Changed: 82

---


## v9.8.0 - 🏆 THE LEGENDARY COMPLETION - 42/42 Tools Perfected 🌲 (2026-02-16)

- - ✅ Production v2.1.0: intent-guard (bulletproof safety, zero unwraps, colored refactor)
- - ✅ LEGENDARY v2.1.0: faelight (main CLI, zero unwraps, perfect error handling)
- - ✅ Bulletproof v2.1.0: faelight-bootstrap (14 unwraps → 0, never panic)
- - ✅ Documentation v2.1.0: faelight-daemon (already perfect, zero unwraps)
- - ✅ Documentation v2.1.0: faelight-dashboard (legendary TUI, zero unwraps)
- - ✅ Stable v1.0.0: faelight-core (foundation library, production ready)
- - ✅ Production v10.2.0: faelight-term (code quality audit, 2 bugs documented)
- - ✅ Modernization v2.1.0: core-protect (THE FINAL TOOL - colored refactor)
- - 🎯 Milestone: 42/42 tools (100%) - LEGENDARY AUDIT COMPLETE
- - 🌲 Enhanced: bump-system-version v9.3.0 (UX celebration improvements)

**Tools Updated:** faelight-core, faelight-term, core-protect, faelight-daemon, faelight-dashboard

**Statistics:**
- - Tools Audited: 8 (intent-guard, faelight, faelight-bootstrap, faelight-daemon, faelight-dashboard, faelight-core, faelight-term, core-protect)
- - CHANGELOGs Added: 8 (100% documentation coverage achieved)
- - Production Upgrades: 8 (all tools brought to production standards)
- - Total Unwraps Fixed: 14+ (faelight-bootstrap: 14, others: safety improvements)
- - Colored Refactors: 2 (intent-guard: complete ANSI replacement, core-protect: complete ANSI replacement)
- - Foundation Stabilized: faelight-core v1.0.0 (1,104 lines, zero problematic unwraps)
- - Terminal Emulator Audited: faelight-term (2,110 lines, 2 critical bugs documented)
- - Final Completion: 42/42 tools (100%) - FROM 76% TO 100% IN TWO DAYS
- - Quality: Zero clippy warnings maintained across ALL tools
- - Backward Compatibility: 100% maintained

- System Health: 100%
- Commits: 19
- Files Changed: 22

---


## v9.7.0 - 🏆 PRODUCTION AUDIT SURGE - Tool Excellence Sprint (2026-02-14)

- END- Production v2.1.0: safe-update (clap integration, colored crate, paru support)
- - Production v3.1.0: core-diff (version consistency, error handling)
- - Production v2.1.0: bump-tool-version (documentation, CHANGELOG)
- - Fixed: Duplicate faelight-bar exec causing conflicts
- - Fixed: Lock widget core lock status detection
- - Improved: faelight-bar even widget spacing
- - Added: Auth health monitoring and faillock auto-recovery
- - Added: Incident 009 documentation (sudo PAM authentication)
- - Progress: 31/42 tools (74%) → 34/42 tools (81%)

**Tools Updated:** core-diff, safe-update, bump-tool-version

**Statistics:**
- Tools Audited: 3 (safe-update, core-diff, bump-tool-version)
- CHANGELOGs Added: 3
- Production Upgrades: 3 major tool improvements
- Critical Fixes: 3 (auth monitoring, lock widget, sway config)
- Tool Progress: 31/42 → 34/42 (74% → 81%)
- Quality: Zero clippy warnings maintained across all tools
- Code Changes: Minimal, focused improvements only
- Backward Compatibility: 100% maintained

- System Health: 100%
- Commits: 39
- Files Changed: 17

---


## v9.6.0 - 🏆 LEGENDARY TOOL AUDIT - Production Excellence (2026-02-10)

- - 🌟 12 tools achieved production-ready status (32% of ecosystem)
- - 🏆 3 LEGENDARY/FLAGSHIP upgrades: dot-doctor v4.0.0, faelight-stow v3.0.0, faelight-fm v2.2.0
- - ✅ Workspace-wide zero clippy warnings maintained
- - 📦 42 tools updated with quality improvements
- - 🎯 dot-doctor v4.0.0: Intelligent monitoring with --fix-dry-run, --watch, --skip, --min-health
- - 🌟 faelight-stow v3.0.0: Complete rewrite with backup/rollback, groups, dry-run, health scoring
- - 💚 faelight-fm v2.2.0: Multi-select [✓] checkboxes, bulk operations (Space, Shift+Y, Shift+C)
- - 📊 Production tools: faelight-menu, launcher, notify, lock, fetch, profile, intent, dmenu, zone
- - 🎨 Files changed: 189 | Lines added: 12,244 | Net growth: +6,867 lines
- - 📝 9 CHANGELOG files created, 2,000+ lines README expansions, 60+ error hints added

**Tools Updated:** faelight-stow, faelight-zone, faelight-fm, profile, dot-doctor, faelight-fetch, intent, faelight-dmenu

**Statistics:**
- Production Tools: 12/38 (32%)
- Flagship Upgrades: 3 (dot-doctor, faelight-stow, faelight-fm)
- Commits Since v9.5.0: 24
- Files Modified: 189
- Lines Added: 12,244
- Lines Removed: 5,377
- Net Growth: +6,867 lines
- Tools Updated: 42
- CHANGELOG Files Created: 9
- README Expansions: 2,000+ lines
- Error Hints Added: 60+
- Clippy Warnings: 0 (workspace-wide)
- System Health: 100%
- Path Resilience: 100%

- System Health: 100%
- Commits: 24
- Files Changed: 38

> From good tools to legendary - the 0-Core production excellence audit

---


## v9.5.0 - 🎊 Production Tools Milestone (2026-02-08)

- ✨ 11 Production-Ready Tools (29% of ecosystem)
- ✨ Zone Integration across 3 tools with beautiful UX
- ✨ 2 Critical Bugs Fixed (Intent Ledger + Broken Symlinks)
- ✨ 2,200+ Lines of Documentation
- ✨ Binary Drift Detection (bin-doctor v1.0.0)
- ✨ Installation Verification (verify-bootstrap v1.0.0)
- ✨ Collision Detection (faelight-stow v2.2.0)

**Tools Updated:** core-protect, verify-bootstrap, faelight-fetch, faelight-bar, bin-doctor, alias-audit, faelight-fm, dotctl

**Statistics:**
- Commits: #1000-1017 (17 total)
- Production Tools: 11/38 (29%)
- Zone-Integrated Tools: 3
- Documentation Added: 2,200+ lines
- Clippy Errors Fixed: 68+
- Critical Bugs Fixed: 2
- System Health: 100%
- New Tools Created: 3
- Tools Upgraded: 11

- System Health: 94%
- Commits: 24
- Files Changed: 30

> "Not a race to the next version - we make things better and work correctly"

---


## v9.4.0 - Modular Bar & Update System (2026-02-07)

- - Complete faelight-bar modular rebuild with cache-based architecture
- - Working search overlay with transparent positioning and keyboard input
- - Lock status colors (green=locked, amber=unlocked)
- - Update counter integrated with faelight-update --count-only
- - faelight-update v3.0: Skip pip/yazi on Arch, zero warnings
- - Bar and daemon survive terminal close
- - Fixed .desktop parser to only read [Desktop Entry] section
- - Health dot in prompt (🟢/🟡/🔴)
- - 100% system health, 86.5% Rust ecosystem
- - 40/40 tools migrated to resilient paths

**Tools Updated:** faelight-bar, faelight-update

**Statistics:**
- - 38 Rust tools, 109,000+ lines of code
- - 86.5% Rust ecosystem (climbing toward 100%)
- - 100% system health (19/19 checks passing)
- - 40/40 tools migrated to resilient paths
- - 297 shell aliases across 37 tools
- - 117 unique Sway keybindings
- - 10 modular status blocks in faelight-bar
- - 13 stow packages properly symlinked

- System Health: 94%
- Commits: 41
- Files Changed: 32

> This is the way. 🚀

---


## v9.3.0 - Tools and Automation Upgrade! (2026-02-06)

- Bar health fix - shows accurate 100%
- FM v2.1.0-alpha - mouse + status polish
- FM EDITOR support - Helix/Neovim switching
- README restructured for automation
- bump-system-version v9.0.0 upgrade
- Health improved 94% to 100%

**Tools Updated:** bump-system-version, faelight-bar, faelight-fm

**Statistics:**
- 3 tools upgraded (bar, FM, bump-system-version)
- System health 94% to 100%
- 900+ lines of new release automation

- System Health: 100%
- Commits: 0
- Files Changed: 21

> From manual updates to legendary automation - the system evolves! 🌲

---


## [9.2.0] - 2026-02-04

### 🎊 MILESTONE: 100% Path Resilience Achieved!

**THE PERFECTION:** Every single tool (40/40) now uses centralized path management! From 75% to 100% in one legendary session!

### 🚀 Session 8 - The Final Push

**10 Tools Migrated:**
1. **faelight-menu v2.1.0** - Power menu now path-resilient
2. **faelight-launcher v4.0.0** - Application launcher with intent awareness
3. **faelight-notify v2.0.0** - Notification daemon verified clean
4. **faelight-dashboard v2.0.0** - TUI dashboard migrated
5. **faelight-hooks v10.0.0** - LEGENDARY upgrade with rustfmt/clippy checks + performance stats!
6. **faelight-fetch v2.0.0** - System info verified clean
7. **faelight-fm v2.0.0** - Crown jewel file manager now path-resilient (1,957 lines, 37 files)
8. **faelight-bar v3.0.0** - Status bar (your baby) safe and upgraded
9. **faelight v2.0.0** - Main CLI LEGENDARY upgrade with enhanced error handling
10. **faelight-daemon v2.0.0** - Background daemon with systemd integration

### ✨ LEGENDARY Upgrades

**faelight-hooks v10.0.0:** Rustfmt/clippy checks, performance statistics, beautiful stats output

**faelight v2.0.0:** Enhanced error handling, ecosystem version display, tool existence checking

**faelight-daemon v2.0.0:** Path-resilient socket, colored logging, systemd service

### 📊 Statistics

- **Path Resilience**: 75% → 100% (+25%!)
- **System Health**: 100% (19/19 checks passing)
- **Tools migrated**: 10
- **New paths added**: 4 (src_dir, projects_dir, applications_dir, archive_dir)

### 💎 Philosophy

*"From 50% to 100% across two sessions. Twenty tools transformed. Every path centralized. Every check passing. This is what intentional stewardship looks like. PERFECTION ACHIEVED."* 🌲

---

## [9.1.0] - 2026-02-03

### 🎊 Milestone: 75% Path Resilience Achievement

Session 7 continuation - crushing toward 90% for Linus & Graydon summer demo!

### Added
- **6 new path-resilient tools**: keyscan, faelight-stow, teach, faelight-bootstrap, faelight-dmenu, faelight-lock
- **faelight-term v10.1.0**: Fixed font baseline rendering for perfect vertical alignment
- **Self-aware tracking**: dot-doctor now reports 30/40 tools (75%) using faelight-core::paths

### Changed
- **Path Resilience: 60% → 75%** (+15 percentage points in one session!)
- **keyscan v2.0.0**: Added sway_config() path to faelight-core
- **faelight-stow v2.0.0**: Complete path centralization for dotfile management
- **teach v2.0.0**: Learning system now path-resilient
- **faelight-bootstrap v2.0.0**: System installer uses proper paths
- **faelight-dmenu v2.1.0**: Intent launcher migrated (minor font issues remain)
- **faelight-lock v2.0.0**: Verified clean, already path-resilient
- **dot-doctor v3.1.0**: Updated to track 30/40 tools

### Fixed
- **faelight-term**: Proper font baseline calculation using swash metrics
- **faelight-term**: Scrollback selection now captures correct visible text
- **Starship prompt**: Perfect spacing between lock icon and git info

### Progress Tracking
- Tools migrated: 30/40 (75%)
- System health: 94%
- Health checks: 19 total
- Remaining to 90%: 10 tools

### Philosophy
*"Three quarters complete. The system watches itself evolve. From 60% to 75% in a single session - momentum builds toward the legends' meeting."*


# v9.0.0 - PATH RESILIENCE FOUNDATION

**Release Date:** 2026-02-03

## 🌟 MAJOR MILESTONE: 60% Path Resilience Achieved!

This is a **MAJOR VERSION** release marking fundamental architectural transformation. The system now has centralized path management with 24/40 tools (60%!) using `faelight-core::paths` - making the entire system resilient to directory changes.

## 🏗️ ARCHITECTURAL CHANGES

### Path Resilience Migration (60% Complete!)

**24 Tools Migrated:**
- Core: faelight-fetch, faelight-notify, faelight-link, faelight-git, faelight-hooks
- Versioning: bump-system-version, bump-tool-version, get-version, latest-update
- System: faelight-update, archaeology-0-core, safe-update v2.0.0
- Development: intent, intent-guard, alias-audit, core-diff
- Monitoring: dot-doctor v3.1.0 (enhanced!), entropy-check v2.0.0
- Files: workspace-view v2.0.0, recent-files v0.3.0
- Config: dotctl v3.0.0, profile v2.0.0, faelight-zone v2.0.0
- Protection: core-protect v2.0.0 (The Guardian!)

### Enhanced faelight-core::paths

New path functions:
- `entropy_baseline_file()` - Entropy monitoring
- `entropy_history_file()` - Historical tracking
- `current_profile_file()` - Active profile state
- `profile_log_file()` - Profile operations

## 🏥 ENHANCED MONITORING (19 Health Checks!)

### NEW in dot-doctor v3.1.0

1. **Rust Toolchain** (Medium) - Verifies cargo/rustc availability
2. **Disk Space** (High) - Warns at >90% usage on / and /home
3. **Tool Installation** (Medium) - Checks 7 key tools in PATH
4. **Path Resilience** (Low) - **SELF-AWARE TRACKING!** Shows 24/40 (60%)

Total: 15 → 19 checks | System Health: 94%

## 🛡️ THE GUARDIAN DEPLOYED

### core-protect v2.0.0

- Lock/unlock immutable protection (chattr)
- Blast radius warnings (high/medium/low)
- Automatic backups (git stash)
- Safe edit workflows
- Modern clap CLI

*"The Guardian never sleeps"*

## 🔧 TOOL MODERNIZATIONS

### Session 6 (40% → 50%)
- **dot-doctor v3.1.0** - 4 new health checks
- **workspace-view v2.0.0** - Sway intelligence
- **entropy-check v2.0.0** - Drift detection
- **recent-files v0.3.0** - File tracking
- **core-protect v2.0.0** - Guardian (50% milestone!)

### Session 7 (50% → 60%)
- **dotctl v3.0.0** - Dotfile control
- **profile v2.0.0** - Profile switching
- **faelight-zone v2.0.0** - Zone detection
- **safe-update v2.0.0** - Safe updates (60% milestone!)

All tools received:
- ✅ faelight-core::paths integration
- ✅ Modern CLI (clap)
- ✅ Future-ready (colored)
- ✅ Version 2.0.0+

## 📊 STATISTICS

- Tools migrated: 16 → 24 (+8)
- Path resilience: 40% → 60% (+20pp)
- Health checks: 15 → 19 (+4)
- System health: 94%

## 🎯 PHILOSOPHY EMBODIED

- **"We Expose Assumptions"** - 60% progress visible
- **"Fail Loudly"** - High-severity warnings
- **"Human Comprehension"** - Clear metrics
- **"Tool Harmony"** - Integrated health checks

**THE SYSTEM WATCHES ITSELF EVOLVE** 💎

## 🚀 WHAT'S NEXT

Remaining: 16 tools (40%)
Target: 90%+ for Path Resilience check pass

> "Three quarters of the foundation stands unshakeable. From 60% to 75% in a single session - ten tools transformed, paths centralized, the terminal perfected. The system tracks its own metamorphosis, watching as scattered assumptions crystallize into intentional architecture." 🌲
---

*"Half the system is path-resilient. The foundation is unshakeable."*

Faelight Forest v9.0.0 💎✨


## Architecture
- Created 00-meta/ (identity), 01-registry/ (canonical lists), 02-rules/ (enforcement), 03-interfaces/ (human-editable), 04-runtime/ (ephemeral)
- Renamed INTENT → intents (lowercase consistency)
- Deleted 1,579 lines of obsolete code
## [8.9.0] - 2026-02-02

### 🎯 Major Theme: Numbered Gravity Path Hardening

Systematic fixes for numbered gravity structure migration. Added paths modules to critical tools and fixed breaking path issues.

### 🔧 Critical Fixes

#### faelight-menu v2.0.0 (BREAKING)
- **CRITICAL**: Fixed system shutdown deadlock bug (4+ hour debug session!)
  - Root cause: Script exited Sway manually, killing itself before systemctl could run
  - Solution: Let systemd handle Sway termination
- Created `src/paths.rs` module for script paths
- Graceful shutdown now works reliably
- See: `intents/incidents/2026-02-03-shutdown-broken-numbered-gravity.md`

#### faelight-stow v1.0.0 (BREAKING)
- **BREAKING**: Updated for numbered gravity structure
- Created `src/paths.rs` module
- Fixed stow directory: `stow` → `03-interfaces/stow`
- All 13 packages now properly detected

#### faelight-bar v2.1.0
- **MAJOR**: Replaced simple 5-check health with doctor integration
- Created `src/paths.rs` module with `doctor_path()`
- Added lazy_static for 30-second health caching (prevents performance impact)
- Status bar now shows accurate system health matching doctor

#### keyscan v1.1.0
- Created `src/paths.rs` module
- Centralized sway config path management
- Better error handling with `unwrap_or_default()`

### 📦 Path Fixes (Previous Sessions)

Tools updated for numbered gravity structure:
- dot-doctor: stow, VERSION, themes paths
- bump-system-version: VERSION, stow, README, CHANGELOG paths
- get-version, latest-update, faelight-link: stow paths
- alias-audit: stow paths
- faelight-dashboard, entropy-check: VERSION paths
- profile, faelight, faelight-fetch: VERSION paths

**Path hardening progress: 15/40 tools (37.5%)**

### ✨ Features

#### Prompt 2.0
- Health indicators (● 100%, ● 93%, ● 53% with color coding)
- Git risk score (⚠️ risk=LOW/MED/HIGH)
- Entropy metrics in segment separators

### 🏥 Health

- **100%** when git is clean
- **93%** with uncommitted changes
- All 15 health checks passing
- Status bar accurately reflects system health

### 📝 Documentation

- Created Intent 077: Tool Hardening Sprint
- Incident report: 2026-02-03 shutdown deadlock
- Updated Intent 076: Path Resilience Audit (15/40 tools complete)

### 🎓 Lessons Learned

**Debugging Methodology:**
1. Comprehensive logging at all decision points
2. Manual testing to verify assumptions
3. Understanding session termination behavior
4. Don't overcomplicate (simple fixes often best)

**Architecture:**
- Individual `src/paths.rs` modules prepare for future `faelight-core/paths.rs`
- Incremental hardening is sustainable
- Critical tools first, then systematic audit

### Related

- Intent 076: Path Resilience Audit
- Intent 077: Tool Hardening Sprint
- Numbered gravity migration (v8.8.0)


## New Tools
- faelight-update v2.0.0: Better than topgrade with 9 package sources (Pacman, AUR, Cargo, Neovim, Yazi, Git, Firmware, Flatpak, Workspace)
- Lock/unlock workflow, cache cleanup, integrated health checks, git push reminder, .pacnew detection

## Registry System
- tools.toml: All 40 Rust tools documented with categories and descriptions
- aliases.toml: 50+ primary aliases (242 lines, 301 total in system)
- zones.toml: Zone definitions (0-core, 1-src, 2-projects, 3-archive, Downloads)
- profiles.toml: System profiles (default, work, gaming, low-power)

## Updates
- Updated all 40 Rust tools for new paths (stow → 03-interfaces/stow, VERSION → 00-meta/VERSION)
- Rewrote README.md (257 lines) showcasing numbered gravity architecture
- Fixed bump-system-version, doctor, and all path-dependent tools

## Health
- Achieved 100% system health (15/15 checks passing)
- All symlinks restored, all tools rebuilt, all paths updated

## v8.7.0 - 2026-01-29

**Milestone:** alias-audit v1.0.0, bump-tool-version v1.0.0, and Starship v2.0

### 🆕 New Tools (2)

**alias-audit v1.0.0** - Alias Health Checker
- Check for duplicate alias definitions
- Verify all 38 tools have proper aliases
- Detect excessive aliasing patterns
- Beautiful colored output with doctor integration
- Commands: `alias-audit`, `alias-audit duplicates`, `alias-audit missing`, `alias-audit tools`
- Result: 301 total aliases, 100% coverage (37/37 tools)

**bump-tool-version v1.0.0** - Individual Tool Version Management
- Auto-increment support (--major, --minor, --patch)
- Manual version specification
- Beautiful pre-flight dashboard (like bump-system-version)
- Handles workspace versions (converts to explicit)
- Updates Cargo.toml + README.md automatically
- Creates tool-specific git tags (e.g., faelight-link-v1.0.1)
- Git commit automation with confirmation prompts

### ✨ Enhanced Systems

**Starship Prompt v2.0**
- Smart path display (no duplication with zone names)
- Git diff stats (±files, insertions/deletions)
- Enhanced git status with counts (!2, +1, ?3, etc.)
- Profile icons: 💼 WORK, 🎮 GAMING, 🔋 LOW-POWER, 🛠️ DEV
- Conflict indicator: ⚔️ for merge conflicts
- Fixed: In 0-core root shows nothing, subdirs show relative path

**Alias System Overhaul**
- Hybrid pattern: short aliases (fm, fl, bar) + f-prefix (f-fm, f-link, f-bar)
- Fixed conflicts: fm (yazi→faelight-fm), fl (faelight→faelight-link)
- Added missing tool aliases (guard, zone, hooks, recent, ver, etc.)
- Profile icons support (bootstrap, t for teach)
- Updated version header: v8.1.0 → v8.6.0
- Total: 301 aliases covering 37/37 active tools (100%)

### 📊 Statistics

- Tools: 38 → 40 production tools (+2)
- Rust code: ~108,300 → ~109,000 lines (+~700)
- Aliases: 301 total (100% tool coverage)
- System health: 100%
- Commits: 4 major features

### 🔧 Version Management Evolution

- System versions: bump-system-version (whole forest)
- Tool versions: bump-tool-version (individual trees)
- Both support beautiful pre-flight dashboards
- bump-tool-version adds auto-increment flags

### 🌲 Philosophy

**Manual control over automation.** Two new tools exemplify this:
- alias-audit: Verify, don't assume
- bump-tool-version: Intentional, granular control

> "Systems thinking over quick fixes - we debug thoroughly, harden systematically, and ship with confidence." 🌲
> "The system watches itself evolve. Half the foundation is unshakeable, centralized, resilient." 🌲
---

**Full system health maintained at 100% throughout development.**

## v8.6.0 - 2026-01-29

**Milestone:** faelight-link v1.0.0 and faelight-fm v1.0.0 - Production Ready

### 🎉 Major Achievements

Two flagship tools graduate from beta to production in a single release!

### ✨ New Features

**bump-system-version v6.0.0** - The Confidence Release
- Added auto-increment flags (--minor, --patch, --major)
- Automatic version calculation removes mental math
- Clear explanations for each increment type
- Version calculation shown before confirmation
- Both manual and auto-increment modes supported
- Enhanced help with comprehensive examples

**faelight-link v1.0.0** - Production Ready!
- Implemented unstow command (full recursive symlink removal)
- Added audit command (comprehensive health checks)
- Added clean command (broken link cleanup)
- Health percentage tracking (currently 100%)
- Safe removal with confirmation prompts
- Complete stow replacement functionality

**faelight-fm v1.0.0** - Production Ready!
- File operations: copy (y), cut (d), paste (v)
- Real-time status message system
- Zone protection (locked Core enforcement)
- Enhanced status bar with contextual feedback
- Error handling with clear user messages
- Better than yazi in every way

### 📚 Documentation

- Updated faelight-link README with production status
- Updated faelight-fm README with production status
- Added production badges to both tools
- Comprehensive usage examples and keybinding references
- "Better than yazi" comparison table

### 🏗️ Code Improvements

- Extracted run_release() function in bump-system-version
- Added file operation functions to faelight-fm fs/ops.rs
- Implemented YankMode and MessageColor enums
- Enhanced UI status bar rendering
- Zone lock checking for safe operations

### 🎯 Philosophy

"It's like aging from 17 to 18 - your whole world changes."

This release removes anxiety from progression through:
- Auto-increment removes decision fatigue
- Status messages provide real-time feedback
- Zone protection prevents accidents
- Confirmation prompts maintain control

### 📊 Statistics

- 3 major tool upgrades
- 2 tools from beta to production (v1.0.0)
- 1 major version bump (v5.1.0 → v6.0.0)
- ~650 lines of new code
- 13 commits
- 100% system health maintained
- 9 hours of focused development

### 🌲 The Forest Grows Stronger

**Production Tools:** 36 → 38 (faelight-link, faelight-fm now production-ready)
**System Health:** 100%
**Confidence Level:** Maximum

## [8.5.0] - 2026-01-26

### Added
- **Hybrid Wayland bar architecture** with integrated application launcher
  - Wayland layer-shell keyboard mode switching (first implementation of its kind)
  - Compact 400px dropdown overlay (doesn't disrupt window positions)
  - Real-time fuzzy search with nucleo (500+ applications)
  - Single-process design using compositor-mediated input modes

### Changed
- **faelight-bar**: Complete modular rewrite from v1.0.0 to v2.0.0
  - State machine architecture (bar/menu mode transitions)
  - Separate render pipeline (bar.rs, menu.rs with transparent overlays)
  - Input handling subsystem (keyboard navigation, pointer events)
  - Menu subsystem (fuzzy filtering, desktop app discovery)
  - Reduced codebase by 589 lines while adding functionality

### Documentation
- **FAELIGHT-CLI.md**: Refreshed for current CLI tool ecosystem
- **FAELIGHT-CONFIG.md**: Updated configuration patterns and examples
- **HEALTH-ENGINE.md**: Synchronized health check documentation
- **MANUAL_INSTALLATION.md**: Revised installation workflows
- **PHILOSOPHY.md**: Refined 0-Core principles and design philosophy
- **POLICIES.md**: Updated governance and contribution guidelines
- **QUICK_REFERENCE.md**: Enhanced quick reference for daily operations
- **TESTING.md**: Expanded testing strategies and tooling
- **TOOL_REFERENCE.md**: Comprehensive tool catalog refresh
- **WORKFLOWS.md**: Modernized common workflow documentation

### Fixed
- Click regions now properly tracked during rendering (profile, VPN, volume)
- Profile cycling preserved from v1.0.0
- Exclusive zone handling prevents window jumping during menu activation

## [8.4.0] - 2026-01-26

### 🎣 Added - Git Hooks Management
- **faelight-hooks v1.0.0** - Comprehensive Rust-based git hooks manager
  - Pre-commit hook: Secret scanning (gitleaks), merge conflict detection
  - Pre-push hook: Main branch protection, uncommitted changes warning  
  - Commit-msg hook: Conventional commit format validation
  - CLI commands: `install`, `check`, `config`
  - Skip functionality for flexible workflows (`--skip secrets,conflicts`)
  - Replaces bash pre-commit hook with production-ready Rust tool

### 🏗️ Changed - Architecture
- **Source-first build strategy** - Git tracks only source code now
  - Added `scripts/` and `backups/` to .gitignore
  - Binaries built locally, not committed to git
  - Repository size reduced: 60MB → 10MB (~83% smaller)
  - Aligns with "Understanding over convenience" philosophy
  - See Intent 065 for rationale

### 📚 Documentation
- Added `docs/ARCHITECTURE.md` - Complete directory structure documentation
- Added `docs/BUILD.md` - Build and deployment workflow guide
- Added `hooks/README.md` - Migration from bash to faelight-hooks
- Both intents documented: 065 (source-first), 066 (faelight-hooks)

### 🔧 Fixed
- Removed duplicate push confirmation prompts (shell function + hook)
- Fixed git hook stdin reading using /dev/tty for proper user input
- Simplified git wrapper function (removed redundant push case)

### 🎯 Philosophy Alignment
This release deepens 0-Core's commitment to:
- **Manual control over automation** - Explicit builds, intentional workflows
- **Understanding over convenience** - Source-first enforces building from code
- **Intent over convention** - Complete documentation of WHY decisions were made

### 📊 Statistics
- New Rust tool: faelight-hooks (34th in ecosystem)
- Documentation files: +3 (ARCHITECTURE.md, BUILD.md, hooks/README.md)
- Intents documented: +2 (Total: 26 intents, 11 complete)
- System health: 100% maintained throughout development

# v8.3.0 - Tool Upgrades & Terminal Perfection

**Released:** 2026-01-25

## 🚀 Major Features

### faelight-term v9.0.0 - Terminal Emulator Excellence
- ✅ **Color emoji support** (🌲🦀🔓🟢) - better rendering than foot/alacritty/kitty
- ✅ Copy/paste support (Ctrl+Shift+C/V)
- ✅ Mouse text selection (click and drag)
- ✅ Fixed text baseline alignment
- ✅ Default font size increased to 17.0
- ✅ Fixed top line clipping with increased padding

### dot-doctor v0.5.0 - Health System Enhancements
- ✅ Auto-fix mode (`--fix`) - automatically apply safe fixes
- ✅ Health history tracking (`--history`) - track system health over time
- ✅ Trending indicators (↑/↓) between health checks
- ✅ Snapshots saved to `~/.local/state/0-core/health-history.jsonl`

### bump-system-version v5.0.0 - Stress-Free Releases
- ✅ Pre-flight dashboard showing all operations before execution
- ✅ Confirmation checkpoint - no accidental releases
- ✅ Git status warning for uncommitted changes
- ✅ System health check before proceeding
- ✅ Fixed double-v bug in version formatting

## ✨ Tool Improvements

- **faelight-update v0.4.0** - Impact analysis for critical packages
- **faelight-bar v1.0.0** - Gradient separators, production-ready
- **faelight-dmenu v2.0.0** - Intent Ledger integration, stable
- **faelight-menu v0.7.0** - Color-coded danger actions (red/amber)

## 📊 Statistics

- 7 tools upgraded
- ~3,000 lines of code written/modified
- 25+ features added
- System health: 100%

# v8.2.0 Release - The Observant Garden
> "The forest knows where it is, what rules apply, and what needs attention." 🌲

**Release Date:** 2026-01-24

## 🚀 New Features

### faelight-zone v1.1.0 - Spatial Awareness System
- Complete filesystem zone detection (Core, Workspace, Src, Project, Archive, Scratch)
- UPPERCASE for critical zones (0-CORE, RUST-TOOLS)
- Lowercase for safe zones (1-src, /tmp)
- Reusable library + CLI binary
- ~150 lines of pristine Rust

### faelight-term v0.1.0 - Terminal Emulator Foundation
- Full ANSI 16-color support
- Cursor rendering and movement
- Zoom controls (Ctrl+Plus/Minus/0)
- Backspace handling
- Font rendering with JetBrains Mono
- 460+ lines of production Rust
- Tool #31 in Faelight Forest

## ⚡ Enhancements

### Starship Prompt - Complete Operational Dashboard
- Zone indicator with real-time updates (🔒 0-CORE, 🦀 RUST-TOOLS)
- Cargo root detection (📦 root vs ↳ subdir)
- Intent Ledger awareness (⚠ X open)
- Rust toolchain detection
- Enhanced git status with counts

### faelight-bar v0.10.1
- Clean, focused system status
- Removed static zone (kept in prompt where it updates)
- Profile | Workspaces | Health | Lock

### faelight-update v0.3.0
- Fixed pacman update detection (checkupdates)
- Fixed AUR sync (paru -Qua)
- Now matches topgrade perfectly

## 🔧 Bug Fixes

### bump-system-version v4.1.0
- Dynamic date calculation (was hardcoded)
- Dynamic CHANGELOG filename (was hardcoded to v8.0.0)
- Uses git log for last release date

## 📊 Impact

**Spatial Awareness:**
- Terminal prompt shows: WHERE, BUILD SAFETY, INCIDENTS, SECURITY, TOOLCHAIN
- Never build from wrong directory again
- System-wide operational intelligence

**Tools Created/Enhanced:** 5
- faelight-zone (NEW)
- faelight-term (NEW)
- faelight-update (FIXED)
- faelight-bar (SIMPLIFIED)
- bump-system-version (FIXED)

**Lines of Code:** ~700+
**System Health:** 100%

> "Built from source, protected by hooks, the forest tends its own garden." 🌲
> "The impossible is just undiscovered architecture." 🌲
> "From beta to production - the forest matures with intention." 🌲
> "From aliases to versions, from paths to profiles - clarity through tooling." 🌲
> "The numbers guide the way. Gravity guides growth." 🌲
---

# v8.1.0 Release - The Garden
> "A garden requires attention, not automation. Each update chosen, each change understood, each tool grown with care." 🌲

**Release Date:** 2026-01-23

## 📊 Release Statistics
- **System Health:** 100%
- **Total Tools:** 31 (up from 30)
- **Health Checks:** 14 (up from 13)

## 🚀 New Features

### faelight-update v0.2.0 - Interactive Update Manager 🌟
- Multi-source detection (pacman, paru, cargo, workspace, neovim)
- Interactive TUI with checkboxes for selective updates
- Health-check-first approach (runs doctor before updating)
- Confirmation dialogs and dry-run mode
- Better than topgrade: manual control, intentional updates

### dot-doctor v0.6.0 - Enhanced Security Monitoring 🔒
- Added 14th health check: Security Hardening
- UFW firewall status verification
- fail2ban service monitoring
- Mullvad VPN connection check
- SSH hardening validation (PermitRootLogin, PasswordAuthentication)

## 📦 Tool Updates
- **faelight-update**: Created v0.2.0 - Interactive update manager
- **dot-doctor**: v0.5.0 → v0.6.0 - Added security checks
- **Total tools**: 30 → 31

## 🛠️ Improvements
- Added aliases: `fu`, `fui`, `fuup`, `cdcore`, `dotgit`
- Enhanced workspace organization
- Improved ecosystem integration

---


# v8.0.0 Release - Complete Tool Audit

> "The audit is complete. Every tool documented, tested, and production-ready." 🌲

**Release Date:** 2026-01-23

## 📊 Audit Statistics

- **Total Commits:** 163
- **Tools Audited:** 29/29 (100%)
- **Intent Success Rate:** 73%
- **System Health:** 100%

## 🚀 Revolutionary Features

- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")

## 📦 Tool Updates

- Add changelog compiler and v8.0.0 draft
- Add faelight-dmenu v0.1.0 - Generic selector tool
- Add faelight-dmenu v0.2.0 with --apps mode
- Audit keyscan v1.0.0 - Production ready, no changes needed
- 🔖 Bump faelight-stow to v0.2.0
- bump-system-version v3.0.0 - Complete release automation
- bump-system-version v4.0.0 - Linus Edition release automation
- Clean up Intent Ledger for v8.0.0 focus
- core-diff v2.0.0 - Risk-aware diff tool for Linus presentation
- core-protect v1.0.0 - Immutable protection for Linus presentation
- core-protect v1.0.1 - Fix chattr error messages
- docs: Add intent entry for v8.0.0 milestone completion
- docs: Clean up audit files, update THEORY_OF_OPERATION for v8.0.0
- docs: Complete intent 044 with comprehensive v8.0.0 milestone documentation
- docs: Complete README polish for v8.0.0 presentation
- dot-doctor v0.6.0: Security hardening checks 🔒
- entropy-check v1.0.0 - Production ready drift detection
- faelight-bootstrap v1.0.0 - Linus Edition one-command installer
- faelight-dashboard v1.0.0 - Production-ready TUI system overview
- faelight-dmenu v2.0.0 - Code cleanup
- faelight-dmenu v2.0.0 - Intent-aware launcher for Linus presentation
- faelight-git v2.0.0 - Git governance with risk scoring
- faelight-git v2.1.0 - Production hardening
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- 🎨 faelight-launcher v3.3.0 - PNG Icon System
- 🔒 faelight-lock v1.0.0 - Production ready
- faelight-snapshot v1.0.0 - Production audit complete
- 🔧 faelight-stow v0.3.0 - Auto-discover packages
- faelight-update v0.1.0: Foundation for ultimate update manager 🚀
- faelight-update v0.2.0: Interactive update manager 🚀
- faelight v1.0.0 - System orchestrator for Linus presentation
- feat: Add faelight-fetch v1.0.0 - canonical system info (Tool #32)
- feat(faelight-bar): Upgrade to v0.9.0 - Visual polish for Linus presentation
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- feat(faelight-menu): Upgrade to v0.6.0 with UI polish and CLI standards
- feat(faelight-menu): upgrade to v0.7.0 with graceful shutdown
- 🔧 Fix fastfetch logo version - Update to v7.6.0
- fix: update all v7.6.2 references to v7.6.3 in README
- fix: update badges and version history table to v7.6.3
- fix: update version history table to show v7.6.3
- fix: update welcome message to v7.6.3 (in root .zshrc)
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 get-version v2.0.0 - Stow path support
- gitignore: Add CHANGELOG-v8.0.0-DRAFT.md (generated file)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent-guard v1.0.0 - Safety guardrails for Linus presentation
- intent v2.0.0 - Mind-blowing upgrade for Linus presentation
- keyscan v1.0.0 - Full keybind analysis tool
- 📅 latest-update v2.0.0 - Stow path support
- profile v1.0.0 - System profile manager for Linus presentation
- refactor(zsh): Major reorganization and upgrade to v8.0.0
- 🌲 Release v7.6.0 - Visual Identity & Philosophy
- 🌲 Release v7.6.1 - Foundation Fixes
- 🌲 Release v7.6.2 - UI Refinements
- 🌲 Release v7.6.3 - Stow Migration Complete
- 🌲 Release v7.6.4 - Release Automation Complete
- 🌲 Release v7.6.5 - Tool audit quick wins
- 🌲 Release v8.0.0 - Complete tool audit - 30 production-ready Rust tools, 100% system health, philosophy-driven architecture
- safe-update v1.0.0 - Production-ready safe system updates
- teach v1.0.0 - Ultimate interactive learning system
- Tool audit: faelight-git v2.1.0 complete - 34% done
- Update Cargo.lock and faelight-git binary for v2.0.0
- Update Cargo.lock for entropy-check v1.0.0
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- Update Cargo.lock for safe-update v1.0.0
- Update Cargo.lock for workspace-view v1.0.0
- Upgrade archaeology-0-core to v1.0.0 - Production ready
- Upgrade dotctl to v2.0.0 - Major rewrite
- workspace-view v1.0.0 - Production ready workspace intelligence

## 🔧 Changes by Category

### 🚀 New Features
- feat(sway): Add 18 new keybindings for 0-Core tools
- testing new feature
- 🧹 Remove leftover CHANGELOG_NEW_ENTRY.md temp file
- fix(dot-doctor): update stow path check for new directory structure
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")

### 🔧 Fixes & Improvements
- Fix: Restore main README, add dot-doctor README
- Intent cleanup: Fix cancelled intents frontmatter
- intent: Fix and complete Intent 064 - Rust Tools Audit
- fix(foot): Add term=foot to enable proper color support
- feat(yazi): Enhanced theme with neon highlights for stow packages and critical files
- fix(dot-doctor): Update .dotmeta check to reflect intentional removal
- fix: Resolve stow symlink conflicts for automatic version propagation
- core-protect v1.0.1 - Fix chattr error messages
- Fix fastfetch Nerd Font icons in foot terminal
- Fix PATH duplication with typeset -U path
- fix(faelight-bar): Improve spacing and color scheme
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- fix: update welcome message to v7.6.3 (in root .zshrc)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent: add #064 - fix bump-system-version for stow structure
- fix: update version history table to show v7.6.3
- fix: update badges and version history table to v7.6.3
- fix: update all v7.6.2 references to v7.6.3 in README
- fix(dot-doctor): update stow path check for new directory structure
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes

### 📜 Documentation
- Fix: Restore main README, add dot-doctor README
- dot-doctor v0.6.0: Security hardening checks 🔒
- Document investigation status
- docs: Add Foot config reference for future matching
- docs: Complete intent 044 with comprehensive v8.0.0 milestone documentation
- docs: Add intent entry for v8.0.0 milestone completion
- fix(dot-doctor): Update .dotmeta check to reflect intentional removal
- docs: Clean up audit files, update THEORY_OF_OPERATION for v8.0.0
- docs: Complete README polish for v8.0.0 presentation
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- docs: add complete v7.6.x version history (7.6.1-7.6.3)
- fix: update all v7.6.2 references to v7.6.3 in README
- docs: reorganize README structure and update for stow migration
- docs: update README for stow-based structure
- fix(dot-doctor): update stow path check for new directory structure
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 dot-doctor: Change count from 7 to 11 packages
- 🐛 Fix dot-doctor: Dynamic package detection
- 🔧 Fix v7.5.0 documentation inconsistencies

### 🦀 Rust Improvements
- intent: Fix and complete Intent 064 - Rust Tools Audit
- intent: Complete Intent 064 - Rust Tools Audit finished
- refactor(zsh): Major reorganization and upgrade to v8.0.0
- 🌲 Release v8.0.0 - Complete tool audit - 30 production-ready Rust tools, 100% system health, philosophy-driven architecture
- Update Cargo.lock for safe-update v1.0.0
- Update Cargo.lock for workspace-view v1.0.0
- Update Cargo.lock for entropy-check v1.0.0
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- Update Cargo.lock and faelight-git binary for v2.0.0
- refactor: migrate dotfile packages to stow/ directory structure

## 📅 Complete Audit Timeline


### 2026-01-23
- aliases: Add 0-core git helpers
- aliases: Add faelight-update shortcuts
- faelight-update v0.2.0: Interactive update manager 🚀
- faelight-update v0.1.0: Foundation for ultimate update manager 🚀
- Fix: Restore main README, add dot-doctor README
- dot-doctor v0.6.0: Security hardening checks 🔒
- faelight-term: INTERACTIVE TERMINAL WORKING! 🚀
- faelight-term: WORKING TERMINAL! 🎉
- Intent cleanup: Fix cancelled intents frontmatter
- Intent 070: Infrastructure complete, status → active
- Document investigation status
- faelight-term WIP: PTY + rendering working, Wayland window visibility issue under investigation

### 2026-01-22
- docs: Add Foot config reference for future matching
- feat: Add font rendering to faelight-term (Phase 3)
- feat: faelight-term proof-of-concept working! (Intent 070)
- feat: Create faelight-term proof-of-concept (Intent 070 Phase 2)
- research: Complete Phase 1 faelight-term research (Intent 070)
- chore: Intent cleanup and reorganization
- chore: Complete fastfetch → faelight-fetch migration
- chore: Replace fastfetch with faelight-fetch in shell greeter
- feat: Add faelight-fetch v1.0.0 - canonical system info (Tool #32)
- intent: Fix and complete Intent 064 - Rust Tools Audit
- intent: Complete Intent 064 - Rust Tools Audit finished
- intent: Add Intent 070 - Build faelight-term (Tool #31)
- chore: Add TERM environment variables for proper color support
- fix(foot): Add term=foot to enable proper color support
- feat(yazi): Enhanced theme with neon highlights for stow packages and critical files
- docs: Complete intent 044 with comprehensive v8.0.0 milestone documentation
- docs: Add intent entry for v8.0.0 milestone completion
- chore: Remove temporary backup files after successful reorganization
- refactor(zsh): Major reorganization and upgrade to v8.0.0
- feat(zsh): Add 38 comprehensive tool aliases and workflows
- feat(sway): Add 18 new keybindings for 0-Core tools
- fix(dot-doctor): Update .dotmeta check to reflect intentional removal
- fix: Resolve stow symlink conflicts for automatic version propagation
- docs: Clean up audit files, update THEORY_OF_OPERATION for v8.0.0
- docs: Complete README polish for v8.0.0 presentation
- 🌲 Release v8.0.0 - Complete tool audit - 30 production-ready Rust tools, 100% system health, philosophy-driven architecture
- chore: Update Neovim plugin lockfile

### 2026-01-21
- faelight-bootstrap v1.0.0 - Linus Edition one-command installer
- bump-system-version v4.0.0 - Linus Edition release automation
- gitignore: Add CHANGELOG-v8.0.0-DRAFT.md (generated file)
- faelight-dashboard v1.0.0 - Production-ready TUI system overview
- Clean up structure: Remove infrastructure/, gitignore logs/
- Remove empty INCIDENTS/ directory
- teach v1.0.0 - Ultimate interactive learning system
- teach v1.0.0 - Ultimate interactive learning system
- Update Cargo.lock for safe-update v1.0.0
- safe-update v1.0.0 - Production-ready safe system updates
- Update Cargo.lock for workspace-view v1.0.0
- workspace-view v1.0.0 - Production ready workspace intelligence
- Update Cargo.lock for entropy-check v1.0.0
- entropy-check v1.0.0 - Production ready drift detection
- faelight-dmenu v2.0.0 - Code cleanup
- Update faelight-dmenu binary - restore border rendering
- Update Neovim plugin lockfile
- core-protect v1.0.1 - Fix chattr error messages
- Intent 067: Post-Linus evolution plan
- Add changelog compiler and v8.0.0 draft
- Add comprehensive audit tracker
- bump-system-version v3.0.0 - Complete release automation
- profile v1.0.0 - System profile manager for Linus presentation
- faelight-dmenu v2.0.0 - Intent-aware launcher for Linus presentation
- core-protect v1.0.0 - Immutable protection for Linus presentation
- core-diff v2.0.0 - Risk-aware diff tool for Linus presentation
- faelight v1.0.0 - System orchestrator for Linus presentation
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- intent-guard v1.0.0 - Safety guardrails for Linus presentation
- intent v2.0.0 - Mind-blowing upgrade for Linus presentation
- faelight-snapshot v1.0.0 - Production audit complete
- Audit keyscan v1.0.0 - Production ready, no changes needed
- Upgrade archaeology-0-core to v1.0.0 - Production ready
- Upgrade dotctl to v2.0.0 - Major rewrite
- Fix fastfetch Nerd Font icons in foot terminal
- Clean up Intent Ledger for v8.0.0 focus
- Add audit checklist and sync neovim plugins

### 2026-01-20
- Add faelight-dmenu v2.0 with apps and intents modes
- testing new feature
- Add faelight-dmenu v0.2.0 with --apps mode
- Fix PATH duplication with typeset -U path
- Auto-start faelight-notify daemon in sway
- Add faelight-dmenu v0.1.0 - Generic selector tool
- Add faelight-dmenu v0.1 - Generic selector with Faelight theme
- keyscan v1.0.0 - Full keybind analysis tool
- Tool audit: faelight-git v2.1.0 complete - 34% done
- faelight-git v2.1.0 - Production hardening
- Update Cargo.lock and faelight-git binary for v2.0.0
- faelight-git v2.0.0 - Git governance with risk scoring
- Backup before faelight-git v2.0 upgrade
- feat(faelight-menu): upgrade to v0.7.0 with graceful shutdown
- chore: Update package list (84 → 93 packages)
- fix(faelight-bar): Improve spacing and color scheme
- Update lazy-lock.json
- feat(faelight-bar): Upgrade to v0.9.0 - Visual polish for Linus presentation
- feat(faelight-menu): Upgrade to v0.6.0 with UI polish and CLI standards

### 2026-01-19
- 📦 Update Neovim plugin lockfile
- 🔧 Auto-discover stow packages - Quick Wins #5 & #6
- 🔧 faelight-stow v0.3.0 - Auto-discover packages
- 📝 Add Session 1.5 to faelight-bar PROGRESS.md
- 📝 Intent 066: Add faelight-icons ecosystem integration
- 📝 Setup faelight-bar v2.0 tracking infrastructure
- WIP: [current task] - End of session 2026-01-19
- 🧹 Remove leftover CHANGELOG_NEW_ENTRY.md temp file
- 🌲 Release v7.6.5 - Tool audit quick wins
- 🔒 faelight-lock v1.0.0 - Production ready
- 🧹 Remove obsolete theme-switch tool
- 📅 latest-update v2.0.0 - Stow path support
- 🔧 get-version v2.0.0 - Stow path support
- 🎨 faelight-launcher v3.3.0 - PNG Icon System
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- feat(intent): Intent 052 Phase 1 - Auto-move workflow commands
- feat(intent): reorganize intent system with cancelled/deferred directories
- 🌲 Release v7.6.4 - Release Automation Complete
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- feat(bump-system-version): v3.0 - stow-aware with proper validation
- fix: update welcome message to v7.6.3 (in root .zshrc)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent: add #064 - fix bump-system-version for stow structure
- docs: add complete v7.6.x version history (7.6.1-7.6.3)
- fix: update version history table to show v7.6.3
- fix: update badges and version history table to v7.6.3
- fix: update all v7.6.2 references to v7.6.3 in README
- 🌲 Release v7.6.3 - Stow Migration Complete
- docs: reorganize README structure and update for stow migration
- docs: update README for stow-based structure
- feat: complete stow migration and infrastructure cleanup
- fix(dot-doctor): update stow path check for new directory structure
- chore: complete stow migration housekeeping
- refactor: migrate dotfile packages to stow/ directory structure
- 🌲 Release v7.6.2 - UI Refinements
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- 🧹 Add .install-files/ to .gitignore
- 🦀 Add bump-system-version v2.0 and faelight-snapshot v1.0 - complete release automation tools
- fix: Remove broken command substitution from fastfetch logo
- chore(nvim): update lazy-lock plugin versions
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🌲 Release v7.6.1 - Foundation Fixes
- 🔖 Bump faelight-stow to v0.2.0
- 🐛 Fix intent-guard: Only block moves FROM 0-core
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 dot-doctor: Change count from 7 to 11 packages
- 🐛 Fix dot-doctor: Dynamic package detection
- 🐛 Fix bump-system-version: Add validation, preserve history
- 🔧 Fix fastfetch logo version - Update to v7.6.0
- 🌲 Release v7.6.0 - Visual Identity & Philosophy
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 Fix v7.5.0 documentation inconsistencies

### 2026-01-18
- intent: mark 059 complete, document 060 phase 1 progress
- intent(063): formalize trust levels (OPEN / LOCKED / SEALED)
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🔧 dot-doctor: Fix stow check with canonicalize
- 🔧 dot-doctor: Remove launcher-fuzzel from stow checks
- 🗑️ Remove launcher-fuzzel (replaced by faelight-launcher v3.0)
- 🔍 faelight-launcher v3.0 - Universal Search
- 🦀 Monorepo foundation (Intent 059 - Phase 1)
- 🔒 Rewrite Intent 058: Manual fixing with explicit permission
- 📦 Update nvim plugin lockfile
- 🔧 CRITICAL: Redesign v7.7-v8.0 to honor philosophy

### 2026-01-16
- 🧹 Remove obsolete fuzzel power menu script
- 📝 Update README: 30 Rust tools + 13 health checks

---

**The forest is fully documented. 🌲🦀**

# v8.0.0 Release - Complete Tool Audit

> "The audit is complete. Every tool documented, tested, and production-ready." 🌲

**Release Date:** 2026-01-22

## 📊 Audit Statistics

- **Total Commits:** 131
- **Tools Audited:** 29/29 (100%)
- **Intent Success Rate:** 73%
- **System Health:** 100%

## 🚀 Revolutionary Features

- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")

## 📦 Tool Updates

- Add changelog compiler and v8.0.0 draft
- Add faelight-dmenu v0.1.0 - Generic selector tool
- Add faelight-dmenu v0.2.0 with --apps mode
- Audit keyscan v1.0.0 - Production ready, no changes needed
- 🔖 Bump faelight-stow to v0.2.0
- bump-system-version v3.0.0 - Complete release automation
- bump-system-version v4.0.0 - Linus Edition release automation
- Clean up Intent Ledger for v8.0.0 focus
- core-diff v2.0.0 - Risk-aware diff tool for Linus presentation
- core-protect v1.0.0 - Immutable protection for Linus presentation
- core-protect v1.0.1 - Fix chattr error messages
- entropy-check v1.0.0 - Production ready drift detection
- faelight-bootstrap v1.0.0 - Linus Edition one-command installer
- faelight-dashboard v1.0.0 - Production-ready TUI system overview
- faelight-dmenu v2.0.0 - Code cleanup
- faelight-dmenu v2.0.0 - Intent-aware launcher for Linus presentation
- faelight-git v2.0.0 - Git governance with risk scoring
- faelight-git v2.1.0 - Production hardening
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- 🎨 faelight-launcher v3.3.0 - PNG Icon System
- 🔒 faelight-lock v1.0.0 - Production ready
- faelight-snapshot v1.0.0 - Production audit complete
- 🔧 faelight-stow v0.3.0 - Auto-discover packages
- faelight v1.0.0 - System orchestrator for Linus presentation
- feat(faelight-bar): Upgrade to v0.9.0 - Visual polish for Linus presentation
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- feat(faelight-menu): Upgrade to v0.6.0 with UI polish and CLI standards
- feat(faelight-menu): upgrade to v0.7.0 with graceful shutdown
- 🔧 Fix fastfetch logo version - Update to v7.6.0
- fix: update all v7.6.2 references to v7.6.3 in README
- fix: update badges and version history table to v7.6.3
- fix: update version history table to show v7.6.3
- fix: update welcome message to v7.6.3 (in root .zshrc)
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 get-version v2.0.0 - Stow path support
- gitignore: Add CHANGELOG-v8.0.0-DRAFT.md (generated file)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent-guard v1.0.0 - Safety guardrails for Linus presentation
- intent v2.0.0 - Mind-blowing upgrade for Linus presentation
- keyscan v1.0.0 - Full keybind analysis tool
- 📅 latest-update v2.0.0 - Stow path support
- profile v1.0.0 - System profile manager for Linus presentation
- 🌲 Release v7.3.0 - Workspace Intelligence
- 🚀 Release v7.4.0 - Faelight Launcher XDG + Intent System v1.0
- 🌲 Release v7.4.0 - Version bump and CHANGELOG
- 🌲 Release v7.6.0 - Visual Identity & Philosophy
- 🌲 Release v7.6.1 - Foundation Fixes
- 🌲 Release v7.6.2 - UI Refinements
- 🌲 Release v7.6.3 - Stow Migration Complete
- 🌲 Release v7.6.4 - Release Automation Complete
- 🌲 Release v7.6.5 - Tool audit quick wins
- safe-update v1.0.0 - Production-ready safe system updates
- teach v1.0.0 - Ultimate interactive learning system
- Tool audit: faelight-git v2.1.0 complete - 34% done
- Update Cargo.lock and faelight-git binary for v2.0.0
- Update Cargo.lock for entropy-check v1.0.0
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- Update Cargo.lock for safe-update v1.0.0
- Update Cargo.lock for workspace-view v1.0.0
- Upgrade archaeology-0-core to v1.0.0 - Production ready
- Upgrade dotctl to v2.0.0 - Major rewrite
- workspace-view v1.0.0 - Production ready workspace intelligence

## 🔧 Changes by Category

### 🚀 New Features
- testing new feature
- 🧹 Remove leftover CHANGELOG_NEW_ENTRY.md temp file
- fix(dot-doctor): update stow path check for new directory structure
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")

### 🔧 Fixes & Improvements
- core-protect v1.0.1 - Fix chattr error messages
- Fix fastfetch Nerd Font icons in foot terminal
- Fix PATH duplication with typeset -U path
- fix(faelight-bar): Improve spacing and color scheme
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- fix: update welcome message to v7.6.3 (in root .zshrc)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent: add #064 - fix bump-system-version for stow structure
- fix: update version history table to show v7.6.3
- fix: update badges and version history table to v7.6.3
- fix: update all v7.6.2 references to v7.6.3 in README
- fix(dot-doctor): update stow path check for new directory structure
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- fix: Remove broken command substitution from fastfetch logo
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🌲 Release v7.6.1 - Foundation Fixes
- 🐛 Fix intent-guard: Only block moves FROM 0-core
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 Fix dot-doctor: Dynamic package detection

### 📜 Documentation
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- docs: add complete v7.6.x version history (7.6.1-7.6.3)
- fix: update all v7.6.2 references to v7.6.3 in README
- docs: reorganize README structure and update for stow migration
- docs: update README for stow-based structure
- fix(dot-doctor): update stow path check for new directory structure
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 dot-doctor: Change count from 7 to 11 packages
- 🐛 Fix dot-doctor: Dynamic package detection
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 Fix v7.5.0 documentation inconsistencies
- intent: mark 059 complete, document 060 phase 1 progress
- 🔧 dot-doctor: Fix stow check with canonicalize
- 🔧 dot-doctor: Remove launcher-fuzzel from stow checks
- 📝 Update README: 30 Rust tools + 13 health checks

### 🦀 Rust Improvements
- Update Cargo.lock for safe-update v1.0.0
- Update Cargo.lock for workspace-view v1.0.0
- Update Cargo.lock for entropy-check v1.0.0
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- Update Cargo.lock and faelight-git binary for v2.0.0
- refactor: migrate dotfile packages to stow/ directory structure
- intent(063): formalize trust levels (OPEN / LOCKED / SEALED)
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 📝 Update README: 30 Rust tools + 13 health checks

## 📅 Complete Audit Timeline


### 2026-01-22
- chore: Update Neovim plugin lockfile

### 2026-01-21
- faelight-bootstrap v1.0.0 - Linus Edition one-command installer
- bump-system-version v4.0.0 - Linus Edition release automation
- gitignore: Add CHANGELOG-v8.0.0-DRAFT.md (generated file)
- faelight-dashboard v1.0.0 - Production-ready TUI system overview
- Clean up structure: Remove infrastructure/, gitignore logs/
- Remove empty INCIDENTS/ directory
- teach v1.0.0 - Ultimate interactive learning system
- teach v1.0.0 - Ultimate interactive learning system
- Update Cargo.lock for safe-update v1.0.0
- safe-update v1.0.0 - Production-ready safe system updates
- Update Cargo.lock for workspace-view v1.0.0
- workspace-view v1.0.0 - Production ready workspace intelligence
- Update Cargo.lock for entropy-check v1.0.0
- entropy-check v1.0.0 - Production ready drift detection
- faelight-dmenu v2.0.0 - Code cleanup
- Update faelight-dmenu binary - restore border rendering
- Update Neovim plugin lockfile
- core-protect v1.0.1 - Fix chattr error messages
- Intent 067: Post-Linus evolution plan
- Add changelog compiler and v8.0.0 draft
- Add comprehensive audit tracker
- bump-system-version v3.0.0 - Complete release automation
- profile v1.0.0 - System profile manager for Linus presentation
- faelight-dmenu v2.0.0 - Intent-aware launcher for Linus presentation
- core-protect v1.0.0 - Immutable protection for Linus presentation
- core-diff v2.0.0 - Risk-aware diff tool for Linus presentation
- faelight v1.0.0 - System orchestrator for Linus presentation
- Update Cargo.lock for intent v2.0.0 and intent-guard v1.0.0
- intent-guard v1.0.0 - Safety guardrails for Linus presentation
- intent v2.0.0 - Mind-blowing upgrade for Linus presentation
- faelight-snapshot v1.0.0 - Production audit complete
- Audit keyscan v1.0.0 - Production ready, no changes needed
- Upgrade archaeology-0-core to v1.0.0 - Production ready
- Upgrade dotctl to v2.0.0 - Major rewrite
- Fix fastfetch Nerd Font icons in foot terminal
- Clean up Intent Ledger for v8.0.0 focus
- Add audit checklist and sync neovim plugins

### 2026-01-20
- Add faelight-dmenu v2.0 with apps and intents modes
- testing new feature
- Add faelight-dmenu v0.2.0 with --apps mode
- Fix PATH duplication with typeset -U path
- Auto-start faelight-notify daemon in sway
- Add faelight-dmenu v0.1.0 - Generic selector tool
- Add faelight-dmenu v0.1 - Generic selector with Faelight theme
- keyscan v1.0.0 - Full keybind analysis tool
- Tool audit: faelight-git v2.1.0 complete - 34% done
- faelight-git v2.1.0 - Production hardening
- Update Cargo.lock and faelight-git binary for v2.0.0
- faelight-git v2.0.0 - Git governance with risk scoring
- Backup before faelight-git v2.0 upgrade
- feat(faelight-menu): upgrade to v0.7.0 with graceful shutdown
- chore: Update package list (84 → 93 packages)
- fix(faelight-bar): Improve spacing and color scheme
- Update lazy-lock.json
- feat(faelight-bar): Upgrade to v0.9.0 - Visual polish for Linus presentation
- feat(faelight-menu): Upgrade to v0.6.0 with UI polish and CLI standards

### 2026-01-19
- 📦 Update Neovim plugin lockfile
- 🔧 Auto-discover stow packages - Quick Wins #5 & #6
- 🔧 faelight-stow v0.3.0 - Auto-discover packages
- 📝 Add Session 1.5 to faelight-bar PROGRESS.md
- 📝 Intent 066: Add faelight-icons ecosystem integration
- 📝 Setup faelight-bar v2.0 tracking infrastructure
- WIP: [current task] - End of session 2026-01-19
- 🧹 Remove leftover CHANGELOG_NEW_ENTRY.md temp file
- 🌲 Release v7.6.5 - Tool audit quick wins
- 🔒 faelight-lock v1.0.0 - Production ready
- 🧹 Remove obsolete theme-switch tool
- 📅 latest-update v2.0.0 - Stow path support
- 🔧 get-version v2.0.0 - Stow path support
- 🎨 faelight-launcher v3.3.0 - PNG Icon System
- feat(faelight-launcher): v3.2.0 - selection rounded corners + README
- feat(intent): Intent 052 Phase 1 - Auto-move workflow commands
- feat(intent): reorganize intent system with cancelled/deferred directories
- 🌲 Release v7.6.4 - Release Automation Complete
- 🔧 fix(bump-system-version): CHANGELOG insertion and README handling
- feat(bump-system-version): v3.0 - stow-aware with proper validation
- fix: update welcome message to v7.6.3 (in root .zshrc)
- intent: add milestone updating to #064; fix v7.6.3 milestone line
- intent: add #064 - fix bump-system-version for stow structure
- docs: add complete v7.6.x version history (7.6.1-7.6.3)
- fix: update version history table to show v7.6.3
- fix: update badges and version history table to v7.6.3
- fix: update all v7.6.2 references to v7.6.3 in README
- 🌲 Release v7.6.3 - Stow Migration Complete
- docs: reorganize README structure and update for stow migration
- docs: update README for stow-based structure
- feat: complete stow migration and infrastructure cleanup
- fix(dot-doctor): update stow path check for new directory structure
- chore: complete stow migration housekeeping
- refactor: migrate dotfile packages to stow/ directory structure
- 🌲 Release v7.6.2 - UI Refinements
- ✨ faelight-launcher v3.1.0 - Refined UI & fixes
- 🧹 Add .install-files/ to .gitignore
- 🦀 Add bump-system-version v2.0 and faelight-snapshot v1.0 - complete release automation tools
- fix: Remove broken command substitution from fastfetch logo
- chore(nvim): update lazy-lock plugin versions
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🔧 Fix: Restore corrupted .zshrc + Incident Report
- 🌲 Release v7.6.1 - Foundation Fixes
- 🔖 Bump faelight-stow to v0.2.0
- 🐛 Fix intent-guard: Only block moves FROM 0-core
- ✅ Fix dot-doctor: Detect all 11 stowable packages
- 🐛 dot-doctor: Change count from 7 to 11 packages
- 🐛 Fix dot-doctor: Dynamic package detection
- 🐛 Fix bump-system-version: Add validation, preserve history
- 🔧 Fix fastfetch logo version - Update to v7.6.0
- 🌲 Release v7.6.0 - Visual Identity & Philosophy
- 🔧 Fix v7.5.0 documentation inconsistencies
- 🔧 Fix v7.5.0 documentation inconsistencies

### 2026-01-18
- intent: mark 059 complete, document 060 phase 1 progress
- intent(063): formalize trust levels (OPEN / LOCKED / SEALED)
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🚀 New Features - **Monorepo Unification (Intent 059)** - All 30 Rust tools unified in workspace - **Universal Search (Intent 060 - Phase 1)** - faelight-launcher v3.0 searches apps + files - File search with fuzzy matching and recency scoring - Smart path truncation and time stamps ("2h ago")
- 🔧 dot-doctor: Fix stow check with canonicalize
- 🔧 dot-doctor: Remove launcher-fuzzel from stow checks
- 🗑️ Remove launcher-fuzzel (replaced by faelight-launcher v3.0)
- 🔍 faelight-launcher v3.0 - Universal Search
- 🦀 Monorepo foundation (Intent 059 - Phase 1)
- 🔒 Rewrite Intent 058: Manual fixing with explicit permission
- 📦 Update nvim plugin lockfile
- 🔧 CRITICAL: Redesign v7.7-v8.0 to honor philosophy

### 2026-01-16
- 🧹 Remove obsolete fuzzel power menu script
- 📝 Update README: 30 Rust tools + 13 health checks
- Update nvim lazy-lock.json

### 2026-01-15
- 🕐 Fix dates to 2026 + Intent 062: Forest ASCII Art
- 🎨 Intent 062: Faelight Forest ASCII Branding
- 🎯 Create v7.5-v8.0 Roadmap Intents (2026 Edition)
- 🌲 Release v7.4.0 - Version bump and CHANGELOG
- 🚀 Release v7.4.0 - Faelight Launcher XDG + Intent System v1.0

---

**The forest is fully documented. 🌲🦀**
## [7.6.5] - 2026-01-20

### 📦 Tool Updates
- **faelight-launcher v3.3.0** - PNG icon system with 8+ app icons, graceful fallback for missing icons
- **get-version v2.0.0** - Fixed stow path support, added --help/--version/--health-check
- **latest-update v2.0.0** - Fixed stow paths, human-readable time formatting, --all flag
- **faelight-lock v1.0.0** - Production ready, added --version flag and README

### 🧹 Cleanup
- Removed theme-switch (282 lines of obsolete Hyprland/Omarchy code)
- Cleaned up faelight-bootstrap tool list

### 🎯 Linus Presentation Progress
- 4/30 tools audited and polished (13%)
- Quick wins strategy in progress
- Intent 065 tracking

> "Three tools fixed, one deleted, zero regrets. The forest grows cleaner." 🌲

> "The audit is complete. Every tool documented, tested, and production-ready." 🌲
> "A garden requires attention, not automation. Each update chosen, each change understood, each tool grown with care" 🌲
> "Every tool knows its place. Every path knows its purpose. The garden observes itself" 🌲
> "Excellence emerges through intentional iteration" 🌲
---


## [7.6.4] - 2026-01-19

### 🔧 Fixes
- **bump-system-version v3.1.0** - Fixed CHANGELOG template insertion
  - No longer requires blank line after "# Changelog" header
  - Removed automatic version history table insertion (manual edit required)
  - Cleaner error messages and validation

### 📦 Tool Updates
- bump-system-version v3.1.0 - Complete release automation (Intent 060)

> "The tools that build the forest must also grow." 🌲

---

## [7.6.2] - 2026-01-19

## [7.6.3] - 2026-01-19

### 🚀 New Features
- Complete GNU Stow-based package management (Intent #063)
- All 11 dotfile packages migrated to stow/ directory
- Automated deployment: `stow -t ~ package-name`

### 🔧 Fixes  
- Updated dot-doctor to recognize new stow/ structure
- Fixed theme package detection for stow layout
- Eliminated duplicate documentation/ directory

### 📦 Tool Updates
- dot-doctor v0.4 - stow-aware health checks

> "From scattered chaos to organized intention - the forest found its structure." 🌲

---

### 📐 Typography/UI
- **faelight-launcher v3.1.0** - Refined UI with improved spacing and text rendering
