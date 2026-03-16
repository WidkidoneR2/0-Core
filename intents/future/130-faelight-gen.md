---
id: 130
date: 2026-03-16
type: future
title: "faelight-gen — Forest-Native Password & Secret Generator Suite"
status: planned
tags: [security, generator, password, tui, ratatui, rust, v10.9]
version: 10.9.0
priority: medium
---

## Vision

A beautiful, colorful, forest-native secret generator.
12 generator types in one tool. Interactive TUI menu.
Color-coded output — instantly scannable at a glance.

Not just a password generator.
A cryptographic toolbox that speaks Faelight Forest.

## The 12 Generators
```bash
faelight-gen                # interactive TUI menu
faelight-gen random         # random character password
faelight-gen passphrase     # diceware wordlist passphrase
faelight-gen uuid           # UUID v4
faelight-gen username       # name-based username
faelight-gen pin            # numeric PIN
faelight-gen apikey         # API key format
faelight-gen base64         # base64 secret
faelight-gen base32         # base32 secret
faelight-gen cryptokey      # cryptographic key (AES-256)
faelight-gen seed           # 12-word mnemonic seed phrase
faelight-gen pronounceable  # human-readable password
faelight-gen token          # session/token ID
```

## The Colored Output

Every character type gets its own color — instantly scannable:
```
🔐 Generated Password
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Tr@4kX9#mQ2$vB7!nP5&wL1^jH8*cF6
  🟢 letters  🔴 numbers  🟡 symbols
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Strength:  ████████████░░░░  STRONG
  Entropy:   94.3 bits
  Type:      Random Character
```

- Letters    → bright green
- Numbers    → bright red
- Symbols    → yellow
- Uppercase  → bold

## The Interactive TUI Menu

Launch with no args for the full menu:
```
╭─ 🔐 faelight-gen ─────────────────────────────╮
│  1  Random Character    — Tr@4kX9#mQ2$         │
│  2  Passphrase          — correct-horse-battery │
│  3  UUID                — 550e8400-e29b...      │
│  4  Username            — swift_falcon_42       │
│  5  PIN                 — 8472                  │
│  6  API Key             — sk_live_x9mQ2$vB7    │
│  7  Base64 Secret       — dGhpcyBpcyBh...       │
│  8  Base32 Secret       — JBSWY3DPEB3W...       │
│  9  Cryptographic Key   — 256-bit AES key       │
│  10 Seed Phrase         — 12-word mnemonic      │
│  11 Pronounceable       — tremoviko             │
│  12 Token               — sess_8f3kQ9xP         │
│                                                  │
│  [1-12] select  [r] regenerate  [q] quit         │
╰──────────────────────────────────────────────────╯
```

## Strength & Entropy Display

Real cryptographic entropy calculation:
```
Strength:  ████████████░░░░  STRONG (82%)
Entropy:   94.3 bits
Crack time: ~1 billion years at 10B guesses/sec
```

## Forest Integration

Every generation emits to state.db:
```
domain: security
action: generate
detail: { type: "random", entropy: 94.3, length: 32 }
```

In faelight-shell:
```
et | where domain == security | where action == generate
```

core security advise surfaces patterns:
```
→ 8 API keys generated this week — consider rotation schedule
```

## Dependencies
```toml
rand          — cryptographic random number generation
crossterm     — terminal control
ratatui       — TUI menu
colored       — color output
sha2          — cryptographic hashing
base64        — base64 encoding
base32        — base32 encoding
zeroize       — secure memory clearing
```

## Success Criteria

- [ ] All 12 generator types working
- [ ] Color-coded output per character type
- [ ] Entropy and strength display
- [ ] Interactive TUI menu
- [ ] Events emitted to state.db
- [ ] Copy to clipboard support
- [ ] core security advise integration

---
*"Security through randomness. Beauty through color."* 🌲
