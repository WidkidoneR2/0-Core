#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════
# 🚀 v3.0 PACKAGE RENAMING - ATOMIC PACKAGE SYSTEM
# Semantic naming for development powerhouse
# ═══════════════════════════════════════════════════════════
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔄 v3.0 Package Renaming - Development Focus"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Window Manager
echo "📦 Renaming: hypr → wm-hypr"
git mv hypr wm-hypr 2>/dev/null || echo "   Already renamed"

# Status Bar
echo "📦 Renaming: waybar → bar-waybar"
git mv waybar bar-waybar 2>/dev/null || echo "   Already renamed"

# Notifications
echo "📦 Renaming: mako → notif-mako"
git mv mako notif-mako 2>/dev/null || echo "   Already renamed"

# Shell
echo "📦 Renaming: fish → shell-fish"
git mv fish shell-fish 2>/dev/null || echo "   Already renamed"

# Editor
echo "📦 Renaming: nvim → editor-nvim"
git mv nvim editor-nvim 2>/dev/null || echo "   Already renamed"

# File Manager
echo "📦 Renaming: yazi → fm-yazi"
git mv yazi fm-yazi 2>/dev/null || echo "   Already renamed"

# Version Control
echo "📦 Renaming: git → vcs-git"
git mv git vcs-git 2>/dev/null || echo "   Already renamed"

# Prompt
echo "📦 Renaming: starship → prompt-starship"
git mv starship prompt-starship 2>/dev/null || echo "   Already renamed"

# Browser
echo "📦 Renaming: brave → browser-brave"
git mv brave browser-brave 2>/dev/null || echo "   Already renamed"

# GTK
echo "📦 Renaming: gtk → theme-gtk"
git mv gtk theme-gtk 2>/dev/null || echo "   Already renamed"

# Terminal Themes
echo "📦 Renaming: foot-theme-dark → theme-term-foot-dark"
git mv foot-theme-dark theme-term-foot-dark 2>/dev/null || echo "   Already renamed"

echo "📦 Renaming: foot-theme-light → theme-term-foot-light"
git mv foot-theme-light theme-term-foot-light 2>/dev/null || echo "   Already renamed"

echo "📦 Renaming: ghostty-theme-dark → theme-term-ghostty-dark"
git mv ghostty-theme-dark theme-term-ghostty-dark 2>/dev/null || echo "   Already renamed"

echo "📦 Renaming: ghostty-theme-light → theme-term-ghostty-light"
git mv ghostty-theme-light theme-term-ghostty-light 2>/dev/null || echo "   Already renamed"

# Launcher Theme
echo "📦 Renaming: fuzzel-theme-dark → theme-launch-fuzzel-dark"
git mv fuzzel-theme-dark theme-launch-fuzzel-dark 2>/dev/null || echo "   Already renamed"

echo ""
echo "✅ Package renaming complete!"
echo ""
echo "📋 NEXT STEPS:"
echo "1. Check renamed packages: ls -la"
echo "2. Update VERSION to v3.0.0"
echo "3. Commit changes"
