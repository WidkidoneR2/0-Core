//! Fuzzy search for items

/// Calculate fuzzy match score
pub fn fuzzy_score(query: &str, text: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(0);
    }

    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();
    
    let mut score = 0;
    let mut last_idx = 0;
    
    for ch in query_lower.chars() {
        if let Some(idx) = text_lower[last_idx..].find(ch) {
            score += 1;
            last_idx += idx + 1;
        } else {
            return None; // No match
        }
    }
    
    // Bonus for consecutive matches
    Some(score)
}

/// Filter and sort items by fuzzy match
pub fn fuzzy_filter(query: &str, items: &[String]) -> Vec<(usize, String)> {
    let mut results: Vec<(usize, String)> = items
        .iter()
        .filter_map(|item| {
            fuzzy_score(query, item).map(|score| (score, item.clone()))
        })
        .collect();
    
    // Sort by score (higher is better)
    results.sort_by(|a, b| b.0.cmp(&a.0));
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_score() {
        assert!(fuzzy_score("int", "intent").is_some());
        assert!(fuzzy_score("066", "INT-066").is_some());
        assert!(fuzzy_score("xyz", "intent").is_none());
    }
}
