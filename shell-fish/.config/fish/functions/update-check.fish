function update-check --description "Check for updates (no installation)"
    echo ""
    echo "🔍 Checking for available updates..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Check official repos
    echo "📦 Official repositories:"
    sudo pacman -Sy > /dev/null 2>&1
    set -l updates (checkupdates 2>/dev/null | wc -l)
    
    if test $updates -gt 0
        echo "   ⚠️  $updates updates available"
    else
        echo "   ✅ System is up to date"
    end
    
    echo ""
    
    # Check AUR
    echo "📦 AUR packages:"
    set -l aur_updates (yay -Qua 2>/dev/null | wc -l)
    
    if test $aur_updates -gt 0
        echo "   ⚠️  $aur_updates AUR updates available"
    else
        echo "   ✅ AUR packages up to date"
    end
    
    echo ""
    
    if test $updates -gt 0 -o $aur_updates -gt 0
        echo "💡 Run 'safe-update' or 'weekly-check' to update"
    else
        echo "🎉 Everything is up to date!"
    end
    
    echo ""
end
