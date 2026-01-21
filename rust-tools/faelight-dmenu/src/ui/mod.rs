//! Wayland UI rendering

pub mod renderer;

use std::sync::{Arc, Mutex};

/// Shared state between main thread and Wayland event loop
pub struct DmenuState {
    pub items: Vec<String>,
    pub filtered: Vec<String>,
    pub query: String,
    pub selected: usize,
    pub running: bool,
    pub result: Option<String>,
}

impl DmenuState {
    pub fn new(items: Vec<String>) -> Self {
        let filtered = items.clone();
        Self {
            items,
            filtered,
            query: String::new(),
            selected: 0,
            running: true,
            result: None,
        }
    }

    pub fn update_query(&mut self, query: String) {
        self.query = query;
        
        // Filter items by fuzzy match
        if self.query.is_empty() {
            self.filtered = self.items.clone();
        } else {
            self.filtered = crate::search::fuzzy_filter(&self.query, &self.items)
                .into_iter()
                .map(|(_, item)| item)
                .collect();
        }
        
        // Reset selection
        self.selected = 0;
    }

    pub fn select_next(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + 1) % self.filtered.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = if self.selected == 0 {
                self.filtered.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn confirm(&mut self) {
        if let Some(item) = self.filtered.get(self.selected) {
            self.result = Some(item.clone());
            self.running = false;
        }
    }

    pub fn cancel(&mut self) {
        self.running = false;
    }
}

pub type SharedState = Arc<Mutex<DmenuState>>;
