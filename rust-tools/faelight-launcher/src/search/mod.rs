//! Universal Search - Multi-domain search system
//! Phase 1: Files

pub mod files;

use std::cmp::Ordering;

/// Universal search result types
#[derive(Debug, Clone)]
pub enum SearchResult {
    App {
        name: String,
        exec: String,
        icon: String,
        score: f32,
    },
    File {
        name: String,
        path: String,
        modified: u64,
        score: f32,
    },
}

impl SearchResult {
    pub fn score(&self) -> f32 {
        match self {
            SearchResult::App { score, .. } => *score,
            SearchResult::File { score, .. } => *score,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SearchResult::App { name, .. } => name,
            SearchResult::File { name, .. } => name,
        }
    }
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

impl Eq for SearchResult {}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.score().partial_cmp(&self.score())
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}
