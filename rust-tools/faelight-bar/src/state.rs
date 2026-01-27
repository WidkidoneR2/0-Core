//! State machine for hybrid bar/menu architecture

use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity, LayerSurface};
use smithay_client_toolkit::shell::WaylandSurface;

const BAR_HEIGHT: u32 = 32;
const MENU_HEIGHT: u32 = 100;
const TOTAL_HEIGHT: u32 = BAR_HEIGHT + MENU_HEIGHT; // 132px

/// Application mode - explicit state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Bar,
    Menu,
}

/// Mode-specific state
pub enum ModeState {
    Bar(BarState),
    Menu(MenuState),
}

pub struct BarState {
    pub last_update: std::time::Instant,
}

pub struct MenuState {
    pub input: String,
    pub items: Vec<String>,
    pub filtered: Vec<usize>,
    pub selected: usize,
}

impl MenuState {
    pub fn new(items: Vec<String>) -> Self {
        let filtered = (0..items.len()).collect();
        Self {
            input: String::new(),
            items,
            filtered,
            selected: 0,
        }
    }
    
    pub fn refilter(&mut self, matcher: &mut nucleo::Matcher) {
        use nucleo::pattern::{Pattern, CaseMatching};
        use nucleo::Utf32String;
        
        if self.input.is_empty() {
            self.filtered = (0..self.items.len()).collect();
        } else {
            let pattern = Pattern::parse(
                &self.input,
                CaseMatching::Smart,
                nucleo::pattern::Normalization::Smart,
            );
            
            let mut scored: Vec<_> = self.items
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let haystack = Utf32String::from(item.as_str());
                    pattern.score(haystack.slice(..), matcher)
                        .map(|score| (idx, score))
                })
                .collect();
            
            // Sort by score (highest first)
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered = scored.into_iter().map(|(idx, _)| idx).collect();
        }
        
        self.selected = 0;
    }
    
    pub fn get_selected_item(&self) -> Option<String> {
        self.filtered.get(self.selected)
            .and_then(|&idx| self.items.get(idx))
            .cloned()
    }
}

/// Mode transition logic
pub struct ModeTransition;

impl ModeTransition {
    /// Configure layer surface for bar mode
    pub fn to_bar(layer: &LayerSurface) {
        layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
        layer.set_size(0, BAR_HEIGHT);
        layer.set_exclusive_zone(BAR_HEIGHT as i32);  // Always reserve 32px
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.wl_surface().commit();
    }
    
    /// Configure layer surface for menu mode
    pub fn to_menu(layer: &LayerSurface) {
        layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
        layer.set_size(0, TOTAL_HEIGHT);  // 132px tall surface
        layer.set_exclusive_zone(BAR_HEIGHT as i32);  // STILL reserve 32px - don't change!
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        layer.wl_surface().commit();
    }
}

/// Application state wrapper
pub struct AppState {
    pub mode: ModeState,
    pub width: u32,
    pub matcher: nucleo::Matcher,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: ModeState::Bar(BarState {
                last_update: std::time::Instant::now(),
            }),
            width: 0,
            matcher: nucleo::Matcher::new(nucleo::Config::DEFAULT),
        }
    }
    
    pub fn current_height(&self) -> u32 {
        match self.mode {
            ModeState::Bar(_) => BAR_HEIGHT,
            ModeState::Menu(_) => TOTAL_HEIGHT,
        }
    }
    
    pub fn is_menu_mode(&self) -> bool {
        matches!(self.mode, ModeState::Menu(_))
    }
    
    pub fn enter_menu(&mut self, items: Vec<String>, layer: &LayerSurface) {
        self.mode = ModeState::Menu(MenuState::new(items));
        ModeTransition::to_menu(layer);
    }
    
    pub fn exit_menu(&mut self, layer: &LayerSurface) {
        self.mode = ModeState::Bar(BarState {
            last_update: std::time::Instant::now(),
        });
        ModeTransition::to_bar(layer);
    }
}
