//! High-performance fuzzy filtering using nucleo

use nucleo::{Matcher, Utf32String};

/// Fuzzy filter with scoring
#[allow(dead_code)]
pub fn fuzzy_filter(query: &str, items: &[String], matcher: &mut Matcher) -> Vec<(usize, u32)> {
    use nucleo::pattern::{Pattern, CaseMatching, Normalization};
    
    if query.is_empty() {
        return items.iter().enumerate().map(|(i, _)| (i, 0)).collect();
    }
    
    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    
    let mut results: Vec<_> = items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            let haystack = Utf32String::from(item.as_str());
            pattern.score(haystack.slice(..), matcher)
                .map(|score| (idx, score))
        })
        .collect();
    
    // Sort by score descending
    results.sort_by(|a, b| b.1.cmp(&a.1));
    results
}
