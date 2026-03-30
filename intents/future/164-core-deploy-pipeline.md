---
id: 164
date: 2026-03-28
type: future
title: "Core Deploy Pipeline — Versioned, Immutable, Rollback-Safe"
status: planned
tags: [build, deploy, cargo, versioned, symlink, rollback, reliability]
version: 11.5.0
priority: high
---

## The Problem
Every core change requires 3 manual commands:
```bash
cd ~/0-core && cargo build --release -p core
sudo cp ~/0-core/target/release/core ~/0-core/scripts/core
sudo cp ~/0-core/target/release/core ~/.cargo/bin/core
```
Two copies = drift risk. No history. No rollback.
If something breaks after deploy: rebuild from scratch or stay broken.

## The Architecture

### Directory Layout (final form)
```
~/0-core/
├── bin/                        # ALL real binaries live here
│   ├── core                    → core@2.0.0-7477a3b  (active symlink)
│   ├── core@2.0.0-7477a3b      # immutable binary
│   ├── core@2.0.0-abc1234      # previous version
│   ├── fsh                     → fsh@0.6.0-def5678
│   ├── fsh@0.6.0-def5678
│   └── fsh@0.6.0-abc9999
├── scripts/
│   ├── deploy                  # ONE deploy script
│   └── rollback                # ONE rollback script
└── target/                     # cargo build output

~/.cargo/bin/
├── core  → ~/0-core/bin/core   # symlink to active pointer
├── fsh   → ~/0-core/bin/fsh    # symlink to active pointer
```

### The Symlink Chain
```
~/.cargo/bin/core
    → ~/0-core/bin/core          (active pointer)
        → ~/0-core/bin/core@VERSION-HASH  (immutable binary)
```

Three levels. Two symlinks. One real binary. Can never drift.

### Version Naming
```
core@VERSION-GITHASH
Example: core@2.0.0-7477a3b
```
Version from Cargo.toml + git short hash.
Every build is uniquely identified and traceable to its commit.

## The Deploy Script
```bash
#!/usr/bin/env bash
# ~/0-core/scripts/deploy
# Usage: deploy [core|fsh|all] [--dev]
set -euo pipefail

ROOT="$HOME/0-core"
BIN_DIR="$ROOT/bin"
CARGO_BIN="$HOME/.cargo/bin"
KEEP_VERSIONS=5

mkdir -p "$BIN_DIR"

build() {
    local pkg="$1"
    local mode="${2:---release}"
    echo "🌲 Building $pkg ($mode)..."
    if [[ "$mode" == "--dev" ]]; then
        (cd "$ROOT" && cargo build -p "$pkg")
        echo "$ROOT/target/debug/$pkg"
    else
        (cd "$ROOT" && cargo build --release -p "$pkg")
        echo "$ROOT/target/release/$pkg"
    fi
}

get_version() {
    local bin="$1"
    local version
    version=$("$bin" --version 2>/dev/null | awk "{print \$2}" | head -1)
    local hash
    hash=$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")
    echo "${version:-0.0.0}-${hash}"
}

install_bin() {
    local name="$1"
    local src="$2"
    local version
    version=$(get_version "$src")
    local versioned="$BIN_DIR/${name}@${version}"
    local active="$BIN_DIR/$name"
    local global="$CARGO_BIN/$name"

    echo "📦 Installing ${name}@${version}..."
    cp "$src" "$versioned"
    chmod +x "$versioned"

    echo "🔗 Updating active pointer..."
    ln -sfn "$versioned" "$active"

    echo "🔗 Ensuring global symlink..."
    ln -sfn "$active" "$global"

    echo "🧪 Verifying..."
    "$global" --version 2>/dev/null | head -1 && echo "✅ ${name}@${version} deployed"

    # Auto-clean old versions (keep last N)
    ls -t "$BIN_DIR/${name}@"* 2>/dev/null | tail -n +$((KEEP_VERSIONS + 1)) | xargs rm -f 2>/dev/null || true
    echo "🧹 Kept last $KEEP_VERSIONS versions"
}

deploy_one() {
    local name="$1"
    local dev_flag="${2:-}"
    local src
    src=$(build "$name" "$dev_flag")
    install_bin "$name" "$src"
}

case "${1:-}" in
    core) deploy_one core "${2:-}" ;;
    fsh)  deploy_one fsh "${2:-}" ;;
    all)
        deploy_one core
        deploy_one fsh
        ;;
    *)
        echo "Usage: deploy [core|fsh|all] [--dev]"
        exit 1
        ;;
esac
```

## The Rollback Script
```bash
#!/usr/bin/env bash
# ~/0-core/scripts/rollback
# Usage: rollback [core|fsh]
set -euo pipefail

BIN_DIR="$HOME/0-core/bin"
NAME="$1"

echo "📜 Available versions of $NAME:"
ls -t "$BIN_DIR/${NAME}@"* 2>/dev/null | while read -r v; do
    version=$(basename "$v")
    current=$(readlink "$BIN_DIR/$NAME")
    if [[ "$v" == "$current" ]]; then
        echo "  → $version  (current)"
    else
        echo "    $version"
    fi
done

echo ""
read -rp "Enter version to activate (e.g. core@2.0.0-abc1234): " VERSION
TARGET="$BIN_DIR/$VERSION"

if [[ ! -f "$TARGET" ]]; then
    echo "❌ Version not found: $VERSION"
    exit 1
fi

ln -sfn "$TARGET" "$BIN_DIR/$NAME"
echo "🔄 Rolled back to $VERSION"
"$BIN_DIR/$NAME" --version
```

## forest-status Command
```bash
forest-status
# 🌲 Forest Status
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#   core    → core@2.0.0-7477a3b   ✅ symlink valid
#   fsh     → fsh@0.6.0-def5678    ✅ symlink valid
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#   Lock:   🔒 LOCKED
#   Health: 100%
#   Dirty:  0 files
```

## Why This Is Better
```
Before:                    After:
core (overwritten)         core@2.0.0-7477a3b (immutable)
no history                 full version history in bin/
no rollback                rollback in milliseconds
drift possible             drift impossible (two symlinks)
debug = rebuild            debug = switch version pointer
```

## Implementation Order (careful, one step at a time)
```
Step 1: Create ~/0-core/bin/ directory
Step 2: Write scripts/deploy — test on core ONLY first
Step 3: Verify symlink chain works correctly
Step 4: Test rollback with a second deploy
Step 5: Add fsh to deploy
Step 6: Write forest-status
Step 7: Update aliases in zsh and fsh config
Step 8: Add deploy/rollback/forest-status to COMMAND-GUIDE.md
```

## Gate Check
```
⬜ ~/0-core/bin/ directory created
⬜ scripts/deploy written and tested on core
⬜ core@VERSION-HASH naming working
⬜ Symlink chain: ~/.cargo/bin/core → bin/core → bin/core@VERSION
⬜ rollback command works (tested)
⬜ Auto-clean keeps last 5 versions
⬜ deploy all works (core + fsh)
⬜ forest-status shows active versions
⬜ aliases updated: deploy/rollback/forest-status in zsh + fsh
⬜ No binary ever manually copied again
⬜ Test: deploy → verify → rollback → verify previous works
```

## The Phrase
**"You are no longer deploying binaries.
You are publishing versions.
Every version is immutable.
Every rollback is instant.
The forest always knows what is running."**

---
*"One symlink chain. Zero drift. Infinite rollback."* 🌲
