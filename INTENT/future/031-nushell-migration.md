---
id: 031
date: 2026-01-10
type: future
title: "Nushell Migration (Gradual)"
status: complete
tags: [rust, shell, nushell, migration]
---

## The Vision
Migrate to Nushell - a modern Rust shell - while preserving all current functionality.

## Why
- 100% Rust shell
- Structured data (tables, records)
- Better error messages
- Modern scripting language
- Type safety

## Migration Strategy

### Phase 1: Parallel Installation
- [x] Install Nushell alongside Zsh
- [ ] Create `shell-nushell` package
- [ ] Keep Zsh as default

### Phase 2: Core Configuration
- [ ] Starship prompt (works natively)
- [ ] Environment variables
- [ ] PATH configuration
- [ ] Basic aliases (top 20)

### Phase 3: Alias Conversion
- [ ] Convert navigation aliases
- [ ] Convert git aliases
- [ ] Convert pacman aliases
- [ ] Convert system aliases
- [ ] Test each conversion

### Phase 4: Advanced Features
- [ ] Custom completions
- [ ] Hooks (cd, prompt)
- [ ] Modules for complex functions
- [ ] Pipeline integrations

### Phase 5: Switchover
- [ ] Set Nushell as default
- [ ] Archive shell-zsh (keep for reference)
- [ ] Update documentation

## Alias Conversion Examples
```nu
# Zsh
alias ll="eza -la --icons --git"

# Nushell
def ll [...args] { eza -la --icons --git ...$args }
```

## Keep Compatibility
- Rust tools work unchanged
- Starship works unchanged
- faelight commands work unchanged

## Success Criteria
- [ ] All 188+ aliases converted
- [x] Starship prompt working
- [ ] No workflow regression
- [ ] Faster shell startup

---
_The forest speaks a new language._ 🌲🦀
