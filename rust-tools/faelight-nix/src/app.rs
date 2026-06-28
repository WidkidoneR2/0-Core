//! app.rs -- faelight-nix application state (INT-076 Phase 1b).

use crate::search::Package;
use ratatui::widgets::ListState;

#[derive(PartialEq)]
pub enum Mode {
    Editing,
    Browsing,
    Confirm,
}

pub struct App {
    pub query: String,
    pub results: Vec<Package>,
    pub list_state: ListState,
    pub mode: Mode,
    pub status: String,
    pub should_quit: bool,
    pub pending_diff: String,
    pub pending_content: String,
    pub pending_pkg: String,
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
            pending_diff: String::new(),
            pending_content: String::new(),
            pending_pkg: String::new(),
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

    pub fn plan_add_selected(&mut self) {
        let pkg = match self.selected() {
            Some(p) => p.attr.clone(),
            None => { self.status = String::from("no package selected"); return; }
        };
        let path = home_nix_path();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => { self.status = format!("cannot read home.nix: {e}"); return; }
        };
        match crate::config_edit::plan_add(&content, &pkg) {
            Ok(plan) => {
                self.pending_diff = plan.diff;
                self.pending_content = plan.new_content;
                self.pending_pkg = plan.pkg;
                self.status = format!("add '{}' to home.packages?  y / n", self.pending_pkg);
                self.mode = Mode::Confirm;
            }
            Err(e) => { self.status = format!("{e}"); }
        }
    }

    pub fn confirm_add(&mut self) {
        let path = home_nix_path();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let bak = format!("{path}.bak-{stamp}");
        if let Err(e) = std::fs::copy(&path, &bak) {
            self.status = format!("backup failed, NOT writing: {e}");
            self.mode = Mode::Browsing;
            return;
        }
        match std::fs::write(&path, &self.pending_content) {
            Ok(_) => {
                self.status = format!("added '{}' -- run rebuild to apply", self.pending_pkg);
            }
            Err(e) => { self.status = format!("write failed: {e}"); }
        }
        self.pending_diff.clear();
        self.pending_content.clear();
        self.pending_pkg.clear();
        self.mode = Mode::Browsing;
    }

    pub fn cancel_add(&mut self) {
        self.pending_diff.clear();
        self.pending_content.clear();
        self.pending_pkg.clear();
        self.status = String::from("add cancelled");
        self.mode = Mode::Browsing;
    }
}

fn home_nix_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".into());
    format!("{home}/0-core/users/christian/home.nix")
}
