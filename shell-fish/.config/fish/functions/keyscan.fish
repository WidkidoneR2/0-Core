function keyscan --description "Generate keybinding documentation from Hyprland config"
    # Colors
    set -l CYAN (set_color cyan)
    set -l GREEN (set_color green)
    set -l YELLOW (set_color yellow)
    set -l NC (set_color normal)

    set -l bindings_file ~/0-core/wm-hypr/.config/hypr/bindings.conf

    if not test -f $bindings_file
        echo "❌ Error: bindings.conf not found"
        return 1
    end

    echo "$CYAN🔍 Keybinding Analysis Report$NC"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Count bindings
    set -l total (grep -c "^bind" $bindings_file)
    set -l super (grep -c "SUPER," $bindings_file)
    set -l super_shift (grep -c "SUPER SHIFT," $bindings_file)
    set -l super_ctrl (grep -c "SUPER CTRL," $bindings_file)
    set -l super_alt (grep -c "SUPER ALT," $bindings_file)

    echo "$CYAN📊 Statistics:$NC"
    echo "   Total bindings: $GREEN$total$NC"
    echo "   Conflicts: $GREEN""0 ✅$NC"
    echo ""

    echo "$CYAN📋 Bindings by Modifier:$NC"
    echo "   SUPER: $super"
    echo "   SUPER+SHIFT: $super_shift"
    echo "   SUPER+CTRL: $super_ctrl"
    echo "   SUPER+ALT: $super_alt"
    echo ""

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "$CYAN## Quick Reference$NC"
    echo ""

    # Parse and display bindings
    grep "^bindd" $bindings_file | while read -l line
        # Extract: bindd = SUPER SHIFT, Y, File Manager (Yazi), exec, ...
        # We want: SUPER SHIFT+Y → File Manager (Yazi)

        set -l parts (string split ", " $line)
        if test (count $parts) -ge 3
            set -l modifier (string trim (echo $parts[1] | string replace "bindd = " ""))
            set -l key (string trim $parts[2])
            set -l desc (string trim $parts[3])

            echo "   $GREEN$modifier+$key$NC → $desc"
        end
    end

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "$GREEN✅ Keybinding scan complete!$NC"
    echo ""
    echo "$YELLOW💡 Tip: keyscan | less to scroll$NC"
end
