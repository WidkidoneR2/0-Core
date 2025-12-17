function weekly-check --description "Weekly maintenance check (MANUAL PROMPT)"
    echo ""
    echo "🗓️  Weekly Maintenance Check"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "This will:"
    echo "  1. Create pre/post snapshots"
    echo "  2. Run system updates (with auto-recovery)"
    echo "  3. Check for .pacnew files"
    echo "  4. Run health check"
    echo ""
    echo "⚠️  This requires user interaction"
    echo "⚠️  You control when this runs (no automation)"
    echo ""
    
    read -P "Continue? (y/N): " -n 1 response
    echo ""
    
    if test "$response" = "y" -o "$response" = "Y"
        echo "🚀 Starting maintenance..."
        echo ""
        ~/0-core/scripts/safe-update
    else
        echo "❌ Cancelled - no changes made"
    end
end
