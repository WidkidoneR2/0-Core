---
id: 147
date: 2026-03-23
type: future
title: "faelight-voice-output — The Forest Speaks Aloud (Piper TTS)"
status: planned
tags: [voice, tts, piper, speech, ai, forest-personality, v12, v13]
version: 12.0.0
priority: medium
depends_on: [133, 146]
---

## The Vision

The forest has always had a voice — in its prompts, its narratives,
its autobiography. faelight-voice gives that voice sound.

Not a novelty. Not a demo. A forest that speaks to you
the way Jarvis speaks to Tony Stark — with context, purpose,
and knowledge of your world.

**Critically: the forest only speaks when it has something worth saying.**
No noise. No filler. Every utterance is intentional.

---

## Why Piper TTS

Piper is the right choice for the forest:
- **Fully local** — no cloud, no API key, no subscription
- **Neural TTS** — sounds human, not robotic
- **C/C++ core with C API** (`libpiper`) — callable from Rust via FFI
- **Multiple voices** — male, female, neutral — you choose
- **GPL-3.0** — open, auditable, fits forest philosophy
- **Battle tested** — used by Home Assistant, NVDA, 448+ projects
- **Fast** — designed for real-time use on embedded hardware

Source: https://github.com/OHF-Voice/piper1-gpl

---

## The Rust Integration Strategy

Piper is C++ but exposes a clean C API via `libpiper`.
Rust calls C with `unsafe` FFI — this is standard practice.
```
libpiper (C++ compiled to .so)
    ↓ C API
faelight-voice/src/ffi.rs (unsafe Rust bindings)
    ↓ safe wrapper
faelight-voice/src/voice.rs (ForestVoice struct)
    ↓
faelight-voice/src/main.rs (CLI + event listener)
    ↓ audio output
pipewire / alsa (system audio)
```

Build strategy:
- `build.rs` compiles and links libpiper via `cc` crate
- `bindgen` generates Rust FFI from piper C headers
- Safe wrapper handles initialization, synthesis, cleanup
- Audio output via `rodio` crate (pure Rust, cross-platform)

---

## What The Forest Says

### On events (Core v10 integration)
```
health drops below 95%
  → "Health advisory. Stability goals have been activated."

background job completes
  → "Cargo build complete. No errors detected."

intent accepted
  → "Intent 147 accepted. The forest has a new goal."

morning startup (INT-143 digest)
  → "Good morning. Health is 100%. You have 3 active goals.
     171 commits this week. The forest is growing fast."
```

### On demand
```bash
faelight-voice say "the forest is healthy"
faelight-voice read <file>          # reads a file aloud
faelight-voice narrate              # speaks core autobiography
faelight-voice status               # current voice config
```

### Forest personality phrases (rotating)
```
"The forest remembers."
"Nothing runs without explicit human authorization."
"Understanding over convenience. Always."
"The roots hold. The branches grow."
```

---

## Voice Identity

The forest's voice is its own. Choices to make on Wednesday:

**Gender:** Male / Female / Neutral
**Tone:** Calm, measured, deliberate — not cheerful, not robotic
**Speed:** Slightly slower than default — the forest thinks before it speaks

Recommended starting voice models to evaluate:
- `en_US-lessac-medium` — neutral American English, clear
- `en_US-ryan-high` — male, natural, high quality
- `en_GB-alan-medium` — male, British, distinctive

Voice model is a config file — changeable without recompiling.

---

## Architecture
```
rust-tools/faelight-voice/
├── Cargo.toml
├── build.rs           — compile + link libpiper
└── src/
    ├── main.rs        — CLI dispatch + event listener
    ├── ffi.rs         — unsafe C bindings to libpiper
    ├── voice.rs       — safe ForestVoice wrapper
    ├── audio.rs       — rodio audio output
    ├── phrases.rs     — forest personality phrase library
    └── events.rs      — state.db event watcher
```

## Configuration
```toml
# ~/.config/faelight-shell/voice.toml
[voice]
enabled = true
model = "en_US-ryan-high"
speed = 0.9
volume = 0.8
speak_on_health_drop = true
speak_on_job_complete = true
speak_morning_digest = true
gender = "male"
```

---

## The Five Phases

### Phase 1 — Core TTS
`faelight-voice say "text"` works.
Piper linked via FFI, audio plays through system output.
Gate: `faelight-voice say "the forest awakens"` produces clear audio.

### Phase 2 — Forest Phrases
Rotating personality phrases on startup.
`faelight-voice narrate` reads core autobiography aloud.
Gate: Morning startup speaks the forest digest.

### Phase 3 — Event Integration
Listens to state.db events — speaks on health drop, job complete.
Runs as a background daemon.
Gate: `cargo build &` completes → forest announces it aloud.

### Phase 4 — Shell Integration
faelight-shell triggers voice on key events.
`d` runs → health spoken if below 95%.
Gate: Health advisory spoken automatically when health drops.

### Phase 5 — Voice Identity
Fine-tuned voice model, speed, tone locked in.
Voice config in `voice.toml`.
Forest has a consistent, recognizable voice.
Gate: Voice is indistinguishable from a deliberate design choice.

---

## Gate Check
```
⬜ Phase 1 — Core TTS (libpiper FFI, say command)
⬜ Phase 2 — Forest phrases (narrate, morning digest)
⬜ Phase 3 — Event integration (daemon, health/job events)
⬜ Phase 4 — Shell integration (automatic voice on key events)
⬜ Phase 5 — Voice identity (model, speed, tone locked in)
```

## Relationship to Other Intents
```
INT-133 Core v9  — autobiography content that voice narrates
INT-140 Core v10 — reaction events that trigger voice
INT-143 digest   — morning summary that voice speaks
INT-146 shell    — shell events that trigger voice
INT-142 (old)    — superseded by this intent (more detailed)
```

## The Phrase

**"A forest that speaks aloud
is a forest that cannot be ignored.
Every word intentional.
Every silence meaningful."**

---
*"The voice is not the personality.
The personality was always there.
The voice just lets you hear it."* 🌲
