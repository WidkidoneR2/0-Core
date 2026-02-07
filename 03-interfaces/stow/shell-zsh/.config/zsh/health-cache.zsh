# Health cache updater for prompt
# Updates health status before each prompt

# Cache file location
HEALTH_CACHE_FILE="$HOME/.cache/faelight/health-status"

# Update health cache before each prompt
update_health_cache() {
    # Only update occasionally (every 30 seconds max)
    local now=$(date +%s)
    local cache_time=0
    
    if [[ -f "$HEALTH_CACHE_FILE" ]]; then
        cache_time=$(stat -c %Y "$HEALTH_CACHE_FILE" 2>/dev/null || echo 0)
    fi
    
    # Update if cache is older than 30 seconds
    if (( now - cache_time > 30 )); then
        # Run doctor and extract health%
        local health=$(doctor 2>/dev/null | grep "Health:" | awk '{print $2}' | tr -d '%')
        
        # Write to cache file
        if [[ -n "$health" ]]; then
            echo "$health" > "$HEALTH_CACHE_FILE"
        fi
    fi
}

# Add to precmd hooks
autoload -Uz add-zsh-hook
add-zsh-hook precmd update_health_cache
