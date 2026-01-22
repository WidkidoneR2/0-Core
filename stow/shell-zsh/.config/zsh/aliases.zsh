alias arch='archaeology-0-core'
alias arch0='archaeology-0-core'

# Archaeology aliases
alias archtime='archaeology-0-core --timeline'
alias archwk='archaeology-0-core --this-week'
alias archint='archaeology-0-core --by-intent'
alias archsince='archaeology-0-core --since'

# Workspace Intelligence aliases
alias ws='workspace-view'
alias wsa='workspace-view --active'
alias wss='workspace-view --summary'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Faelight Git Workflow Aliases
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
alias fg='faelight-git'
alias fgc='faelight-git commit'
alias fgs='faelight-git status'
alias fga='faelight-git add'
alias fgp='faelight-git push'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Version & Update Info Aliases
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
alias latest='latest-update'
alias lastup='latest-update'
alias ver='echo "🌲 Faelight Forest v8.0.0"'
alias sysver='uname -r'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Keybinding & Config Analysis Aliases
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
alias keys='keyscan'
alias keybinds='keyscan'
alias conflicts='keyscan'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Stow Management Aliases (dotctl)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
alias dot='dotctl'
alias dotadd='dotctl add'
alias dotrem='dotctl remove'
alias dotlist='dotctl list'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Changelog & Documentation Aliases
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
alias changelog='bat ~/0-core/CHANGELOG.md'
alias changelog-draft='bat ~/0-core/CHANGELOG-v8.0.0-DRAFT.md'
alias compile-log='~/0-core/scripts/compile-changelog.sh'
alias mklog='~/0-core/scripts/compile-changelog.sh'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Intent Ledger Shortcuts
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
alias int='intent'
alias intl='intent list'
alias ints='intent show'
alias inta='intent add'
alias intc='intent complete'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Composite Workflow Aliases (Multi-tool Chains)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Pre-commit safety check
alias pre-commit='echo "🔍 Pre-commit checks..." && gitleaks protect --staged -v && dot-doctor && echo "✅ Safe to commit!"'

# Quick system audit (health + drift + security)
alias audit='echo "🏥 Running full audit..." && dot-doctor && entropy-check && security-score'
alias full-audit='dot-doctor && entropy-check && security-check'

# Release workflow (bump version, compile changelog, show status)
alias release-prep='echo "📦 Preparing release..." && bump-system-version && compile-changelog.sh && git status'

# Quick health + git status
alias status='dot-doctor && echo "" && git status'

# System overview (all key metrics)
alias overview='fastfetch && echo "" && dot-doctor && echo "" && git -C ~/0-core status -s'

# Snapshot before risky operations
alias snap-now='faelight-snapshot create "Manual snapshot at $(date +%Y%m%d_%H%M%S)"'
alias snap-before='echo "📸 Creating safety snapshot..." && snap-now && echo "✅ Snapshot created! Proceed with operation."'

# Update workflow (snapshot + update + health check)
alias safe-up='snap-now && safe-update && dot-doctor'

# Development quick-commit (staged files + auto message)
alias qc='git commit -m "Quick update: $(date +%Y-%m-%d)"'
alias qcp='git commit -m "Quick update: $(date +%Y-%m-%d)" && git push'

# Show system status card
alias card='echo "╔════════════════════════════════════════╗" && echo "║  🌲 FAELIGHT FOREST v8.0.0            ║" && echo "║  🏥 Health: $(dot-doctor | grep "Health:" | awk "{print \$2}")                        ║" && echo "║  📦 Tools: 30 Production Ready         ║" && echo "║  🔒 Security: Hardened                 ║" && echo "╚════════════════════════════════════════╝"'

