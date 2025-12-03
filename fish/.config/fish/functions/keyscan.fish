function keyscan --description "Scan and analyze Hyprland keybindings"
    echo "🔍 Keybinding Analysis Report"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    set -l bindings_file ~/dotfiles/hypr/.config/hypr/bindings.conf

    if not test -f $bindings_file
        echo "❌ Error: bindings.conf not found at $bindings_file"
        return 1
    end

    # Parse bindings
    set -l bindings (_parse_bindings $bindings_file)

    # Count by modifier
    set -l super_count 0
    set -l super_shift_count 0
    set -l super_ctrl_count 0
    set -l super_alt_count 0

    for binding in $bindings
        if string match -q "*SHIFT*" $binding
            set super_shift_count (math $super_shift_count + 1)
        else if string match -q "*CTRL*" $binding
            set super_ctrl_count (math $super_ctrl_count + 1)
        else if string match -q "*ALT*" $binding
            set super_alt_count (math $super_alt_count + 1)
        else if string match -q "SUPER+*" $binding
            set super_count (math $super_count + 1)
        end
    end

    # Detect conflicts (same binding appears twice)
    set -l conflicts (_detect_conflicts $bindings)

    # Statistics
    echo "📊 Statistics:"
    echo "   Total bindings: "(count $bindings)
    if test (count $conflicts) -gt 0
        echo "   Conflicts: "(count $conflicts)" ⚠️"
    else
        echo "   Conflicts: 0 ✅"
    end
    echo ""

    echo "📋 Bindings by Modifier:"
    echo "   SUPER: $super_count"
    echo "   SUPER+SHIFT: $super_shift_count"
    echo "   SUPER+CTRL: $super_ctrl_count"
    echo "   SUPER+ALT: $super_alt_count"
    echo ""

    # Show conflicts if any
    if test (count $conflicts) -gt 0
        echo "⚠️  Conflicts Detected:"
        for conflict in $conflicts
            echo "   🔴 $conflict"
        end
        echo ""
    end

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
end

# Helper: Parse bindings from config
function _parse_bindings
    set -l file $argv[1]

    # Extract all bind/bindd lines and parse them
    grep -E '^\s*bind[d]?\s*=' $file | while read -l line
        # Extract modifier and key
        # Format: bindd = MODIFIER, KEY, Description, command
        if string match -qr 'bind[d]?\s*=\s*([^,]+)\s*,\s*([^,]+)' $line
            set -l modifier (string trim (string match -r 'bind[d]?\s*=\s*([^,]+)' $line)[2])
            set -l key (string trim (string match -r ',\s*([^,]+)' $line)[2])
            echo "$modifier+$key"
        end
    end
end

# Helper: Detect duplicate bindings
function _detect_conflicts
    set -l bindings $argv
    set -l seen
    set -l conflicts

    for binding in $bindings
        if contains $binding $seen
            if not contains $binding $conflicts
                set conflicts $conflicts $binding
            end
        else
            set seen $seen $binding
        end
    end

    echo $conflicts
end
