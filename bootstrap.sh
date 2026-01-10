#!/bin/bash
# 🌲 Faelight Forest Bootstrap
# Usage: curl -fsSL https://raw.githubusercontent.com/USER/0-core/main/bootstrap.sh | bash

set -e

echo "═══════════════════════════════════════════════════════════"
echo "🌲 Faelight Forest Bootstrap"
echo "═══════════════════════════════════════════════════════════"
echo

# Check Arch
if [[ ! -f /etc/arch-release ]]; then
    echo "❌ Error: This script is for Arch Linux only"
    exit 1
fi

# Check not root
if [[ $EUID -eq 0 ]]; then
    echo "❌ Error: Don't run as root"
    exit 1
fi

# Install minimal deps first
echo "📦 Installing git and base-devel..."
sudo pacman -Sy --needed --noconfirm git base-devel rustup

# Setup Rust
rustup default stable 2>/dev/null || true

# Clone 0-core
CORE_PATH="$HOME/0-core"
if [[ -d "$CORE_PATH" ]]; then
    echo "⚠️  0-core exists, pulling latest..."
    cd "$CORE_PATH" && git pull
else
    echo "📥 Cloning 0-core..."
    git clone https://github.com/ChristianKaworworwa/0-core.git "$CORE_PATH"
fi

# Build and run the Rust bootstrap
cd "$CORE_PATH/rust-tools/faelight-bootstrap"
echo "🔨 Building bootstrap tool..."
cargo build --release --quiet

echo
echo "🚀 Running Faelight Bootstrap..."
echo
./target/release/faelight-bootstrap
