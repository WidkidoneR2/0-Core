---
id: 164
date: 2026-03-28
type: future
title: "Core Deploy Pipeline — Build Fast, Deploy Correctly, Every Time"
status: planned
tags: [build, deploy, cargo, core, fsh, workflow, reliability]
version: 11.5.0
priority: high
---

## The Problem
Every core change requires 3 manual commands:
```bash
cd ~/0-core && cargo build --release -p core 2>&1 | grep -E "^error|Compiling|Finished"
sudo cp ~/0-core/target/release/core ~/0-core/scripts/core
sudo cp ~/0-core/target/release/core ~/.cargo/bin/core
```

And for faelight-shell, the same 3-command pattern.
Manual steps = human error. Missed deploys. Wrong binary active.
fsh-deploy exists but is a 200-char alias, not a proper script.

## The Current State
```
fsh-deploy   = alias (200 chars, fragile)
core deploy  = no equivalent (3 manual commands)
faelight-*   = each tool has no deploy standard
```

## The Solution
A proper deploy script for each binary.
Consistent. Fast. Verified. One command.

## Phase 1 — core-deploy script
```bash
#!/usr/bin/env bash
# ~/0-core/scripts/core-deploy
# Build and deploy core binary to all required paths

set -euo pipefail

CORE_ROOT="$HOME/0-core"
RELEASE_BIN="$CORE_ROOT/target/release/core"
SCRIPT_PATH="$CORE_ROOT/scripts/core"
CARGO_PATH="$HOME/.cargo/bin/core"

echo "🌲 Building core..."
cd "$CORE_ROOT"

if cargo build --release -p core 2>&1 | grep -E "^error|Compiling|Finished"; then
    if grep -q "Finished" <<< "$(cargo build --release -p core 2>&1)"; then
        echo "✅ Build successful"
    fi
fi

echo "🚀 Deploying core..."
sudo cp "$RELEASE_BIN" "$SCRIPT_PATH"
sudo cp "$RELEASE_BIN" "$CARGO_PATH"

echo "✅ Core deployed to:"
echo "   $SCRIPT_PATH"
echo "   $CARGO_PATH"

# Verify
DEPLOYED_VER=$("$CARGO_PATH" --version 2>/dev/null | head -1)
echo "   Version: $DEPLOYED_VER"

# Quick health check
core doctor run --quiet 2>/dev/null && echo "✅ Health: OK" || echo "⚠️  Run d to check health"
```

## Phase 2 — fsh-deploy as proper script
Replace the 200-char alias with a real script:
```bash
#!/usr/bin/env bash
# ~/0-core/scripts/fsh-deploy
# Build and deploy faelight-shell to all required paths
```

## Phase 3 — Unified deploy command
```bash
core deploy        # deploy core binary
fsh-deploy         # deploy faelight-shell
deploy-all         # deploy everything
```

## Phase 4 — Build optimization
Current: full workspace rebuild on every change.
Better: cargo watch for development, release build for deploy.
```bash
# Development (fast feedback):
cargo watch -x "build -p core"

# Deploy (release, verified):
core-deploy
```

## Phase 5 — Workspace sync verification
After every deploy, verify binaries are in sync:
```bash
core deploy --verify
# Checks: scripts/core == ~/.cargo/bin/core (same hash)
# Checks: version matches /etc/faelight/VERSION
# Checks: all tool binaries present in scripts/
```

## Gate Check
```
⬜ ~/0-core/scripts/core-deploy script created
⬜ ~/0-core/scripts/fsh-deploy script created (replaces alias)
⬜ core-deploy alias in both zsh and fsh configs
⬜ fsh-deploy alias updated to use script
⬜ deploy --verify checks binary hash sync
⬜ cargo watch available for development builds
⬜ deploy-all script for full workspace deploy
⬜ All tool deploys follow same pattern
```

## The Phrase
**"A system that deploys correctly every time
is a system you can trust.
Manual steps are where trust breaks."**

---
*"Build fast. Deploy correctly. Verify always."* 🌲
