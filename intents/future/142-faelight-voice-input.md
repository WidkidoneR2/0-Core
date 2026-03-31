---
id: 142
date: 2026-03-21
type: future
title: "faelight-voice-input — Voice Input to Forest Pipelines (Whisper)"
status: planned
tags: [voice, whisper, nlp, shell, rust, v12]
version: 12.0.0
priority: medium
depends_on: [139]
---

## The Vision

Speak to the forest. The forest listens.
```
"Hey forest, what's using my memory"
→ ?memory hogs
→ ps | sort memory desc | first 5
→ confirmed, executed
```

Not a general AI assistant.
Forest-specific voice commands mapped to structured pipelines.
The same pattern matching engine as INT-139, but with voice input.

## The Stack
```toml
whisper-rs = "0.11"   # Rust bindings for whisper.cpp
cpal = "0.15"         # Cross-platform audio capture
```

whisper.cpp runs locally — no network, no cloud, no privacy concerns.
Uses the small.en model (~150MB) for fast transcription.

## How It Works
```
1. faelight-voice daemon runs in background
2. Activation: hotkey (Super+V) or always-on VAD
3. Audio captured via cpal
4. Transcribed via whisper-rs (local)
5. Text sent to faelight-shell as ?<transcribed text>
6. NL engine (INT-139) translates to pipeline
7. User confirms before execution
```

## Integration Points

- faelight-shell `?` prefix — voice text becomes NL query
- faelight-notify v4 — shows what was heard before executing
- faelight-niri-bridge — hotkey registration
- Core v9 goals — "what should I work on" → goal list

## Models
```
whisper tiny.en   — fastest, lowest accuracy (~75MB)
whisper small.en  — balanced (~150MB) — RECOMMENDED
whisper medium.en — highest accuracy (~500MB)
```

## Dependency Requirements (investigate before starting)
```
whisper.cpp     — AUR only, out-of-date (1.8.3-1), depends on libggml-git (unstable)
libggml-git     — AUR git package, unstable dependency chain
python-pytorch  — 2GB+ if using python-openai-whisper (not recommended)
alsa-lib        — already installed ✅
```

### Recommended Install Path
Wait for whisper.cpp AUR package to be updated, OR:
Build whisper.cpp from source directly — more stable than AUR git dependency:
```bash
git clone https://github.com/ggerganov/whisper.cpp
cd whisper.cpp && make
./models/download-ggml-model.sh small.en
```
Then use whisper-rs with WHISPER_CPP_PATH pointing to local build.
Do NOT install on a healthy system until dependency chain is verified stable.

## Gate Check
```
⬜ whisper-rs compiles in workspace
⬜ Audio capture via cpal
⬜ Transcription accuracy > 90% on forest commands
⬜ Hotkey activation via faelight-niri-bridge
⬜ Text piped to faelight-shell as ?query
⬜ faelight-notify v4 shows transcription
⬜ Latency < 2 seconds end-to-end
```

## The Phrase

**"The forest that listens
does not need to be commanded.
It understands."**
