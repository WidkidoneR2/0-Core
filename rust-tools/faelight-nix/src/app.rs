//! app.rs -- faelight-nix application state (INT-076 Phase 1b).

use crate::search::Package;
use ratatui::widgets::ListState;

#[derive(PartialEq)]
pub enum Mode {
    Editing,
    Browsing,
}

pub struct App {
    pub query: String,
    pub results: Vec<Package>,
    pub list_state: ListState,
    pub mode: Mode,
    pub status: String,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: vec![],
            list_state: ListState::default(),
            mode: Mode::Editing,
            status: String::from("type a package name, press Enter to search"),
            should_quit: false,
        }
    }

    pub fn run_search(&mut self) {
        let q = self.query.trim().to_string();
        if q.is_empty() {
            self.status = String::from("enter a search term first");
            return;
        }
        self.status = format!("searching nixpkgs for '{q}' ...");
        match crate::search::search(&q) {
            Ok(pkgs) => {
                let n = pkgs.len();
                self.results = pkgs;
                if n == 0 {
                    self.status = String::from("no matches");
                    self.list_state.select(None);
                } else {
                    self.status = format!("{n} result(s)");
                    self.list_state.select(Some(0));
                    self.mode = Mode::Browsing;
                }
            }
            Err(e) => {
                self.status = format!("search failed: {e}");
                self.results.clear();
                self.list_state.select(None);
            }
        }
    }

    pub fn selected(&self) -> Option<&Package> {
        self.list_state.selected().and_then(|i| self.results.get(i))
    }

    pub fn next(&mut self) {
        if self.results.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.results.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        if self.results.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
