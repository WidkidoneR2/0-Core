function sync-0-core --description 'Sync 0-core with auto-unlock'
    set -l was_locked false

    echo "🔄 Syncing 0-core..."
    echo ""

    # Check if locked (look for immutable attribute)
    if lsattr ~/0-core 2>/dev/null | head -1 | grep -q -- ----i
        set was_locked true
        echo "🔓 Core is locked, unlocking temporarily..."
        unlock-core
        echo ""
    end

    # Navigate and sync
    cd ~/0-core

    # Pull changes
    echo "⬇️  Pulling latest changes..."
    if git pull
        echo ""
        echo "⬆️  Pushing local changes..."
        git push
    else
        echo ""
        echo "❌ Pull failed - resolve conflicts manually"
        if test "$was_locked" = true
            echo "⚠️  Core left unlocked for conflict resolution"
        end
        return 1
    end

    echo ""

    # Re-lock if it was locked
    if test "$was_locked" = true
        echo "🔒 Re-locking core..."
        lock-core
    end

    echo ""
    echo "✅ Sync complete!"
end
