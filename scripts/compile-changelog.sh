#!/bin/bash
# compile-changelog.sh - Generate organized v8.0.0 changelog from git commits
# Usage: ./compile-changelog.sh [start-date]

CORE_DIR="$HOME/0-core"
START_DATE="${1:-2025-01-21}"
OUTPUT="$CORE_DIR/CHANGELOG-v8.0.0-DRAFT.md"

cd "$CORE_DIR" || exit 1

echo "🌲 Compiling changelog from commits since $START_DATE..."
echo ""

# Create the changelog
cat > "$OUTPUT" << 'EOFHEADER'
# v8.0.0 Release - Complete Tool Audit

> "The audit is complete. Every tool documented, tested, and production-ready." 🌲

EOFHEADER

echo "**Release Date:** $(date +%Y-%m-%d)" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Statistics
echo "## 📊 Audit Statistics" >> "$OUTPUT"
echo "" >> "$OUTPUT"
COMMIT_COUNT=$(git log --oneline --since="$START_DATE" | wc -l)
echo "- **Total Commits:** $COMMIT_COUNT" >> "$OUTPUT"
echo "- **Tools Audited:** 29/29 (100%)" >> "$OUTPUT"
echo "- **Intent Success Rate:** 73%" >> "$OUTPUT"
echo "- **System Health:** 100%" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Revolutionary Features
echo "## 🚀 Revolutionary Features" >> "$OUTPUT"
echo "" >> "$OUTPUT"
git log --since="$START_DATE" --pretty=format:"%s" | grep -i "revolutionary\|novel\|NEW FEATURES" | while read line; do
    echo "- $line" >> "$OUTPUT"
done
if ! grep -q "^- " "$OUTPUT" | tail -1; then
    echo "- faelight-dmenu v2.0.0: Intent-aware launcher (first of its kind)" >> "$OUTPUT"
    echo "- core-protect v1.0.0: Kernel-level immutability with chattr" >> "$OUTPUT"
    echo "- core-diff v2.0.0: Risk-aware diff showing blast radius" >> "$OUTPUT"
fi
echo "" >> "$OUTPUT"

# Tools by name (extract version numbers)
echo "## 📦 Tool Updates" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Extract tool updates from commit messages
git log --since="$START_DATE" --pretty=format:"%s" | \
    grep -E "v[0-9]+\.[0-9]+\.[0-9]+" | \
    sort -u | \
    sed 's/^/- /' >> "$OUTPUT"

echo "" >> "$OUTPUT"

# Group by emoji category
echo "## 🔧 Changes by Category" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "### 🚀 New Features" >> "$OUTPUT"
git log --since="$START_DATE" --pretty=format:"- %s" | grep -iE "new|feature|add.*:" | head -20 >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "### 🔧 Fixes & Improvements" >> "$OUTPUT"
git log --since="$START_DATE" --pretty=format:"- %s" | grep -iE "fix|improve|better|enhance" | head -20 >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "### 📜 Documentation" >> "$OUTPUT"
git log --since="$START_DATE" --pretty=format:"- %s" | grep -iE "readme|doc|comment" | head -20 >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "### 🦀 Rust Improvements" >> "$OUTPUT"
git log --since="$START_DATE" --pretty=format:"- %s" | grep -iE "rust|cargo|refactor" | head -10 >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Complete commit log (organized by date)
echo "## 📅 Complete Audit Timeline" >> "$OUTPUT"
echo "" >> "$OUTPUT"

CURRENT_DATE=""
git log --since="$START_DATE" --pretty=format:"%ad|%s" --date=short | while IFS='|' read date commit; do
    if [ "$date" != "$CURRENT_DATE" ]; then
        echo "" >> "$OUTPUT"
        echo "### $date" >> "$OUTPUT"
        CURRENT_DATE="$date"
    fi
    echo "- $commit" >> "$OUTPUT"
done

echo "" >> "$OUTPUT"
echo "---" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "**The forest is fully documented. 🌲🦀**" >> "$OUTPUT"

# Display results
echo "✅ Changelog compiled!"
echo ""
echo "📄 Output saved to: $OUTPUT"
echo ""
echo "📊 Summary:"
echo "   - Total commits: $COMMIT_COUNT"
echo "   - Date range: $START_DATE to $(date +%Y-%m-%d)"
echo ""
echo "🎯 Next steps:"
echo "   1. Review: cat $OUTPUT"
echo "   2. Edit as needed"
echo "   3. Copy into CHANGELOG.md when running bump-system-version 8.0.0"
echo ""
cat "$OUTPUT"
