# Health cache updater for prompt
# Updates health status EVERY prompt for real-time feedback

# Cache file location
HEALTH_CACHE_FILE="$HOME/.cache/faelight/health-status"

# Update health cache before each prompt
update_health_cache() {
    # Run doctor and extract health% (fast, runs every prompt)
    local health=$(doctor 2>/dev/null | grep "Health:" | awk '{print $2}' | tr -d '%')
    
    # Write to cache file
    if [[ -n "$health" ]]; then
        mkdir -p ~/.cache/faelight
        echo "$health" > "$HEALTH_CACHE_FILE"
    fi
}

# Add to precmd hooks (runs before every prompt)
autoload -Uz add-zsh-hook
add-zsh-hook precmd update_health_cache
