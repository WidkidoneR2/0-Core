//! State machine for hybrid bar/menu architecture
use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity, LayerSurface};
use smithay_client_toolkit::shell::WaylandSurface;
use crate::menu::MenuItem;

const BAR_HEIGHT: u32 = 32;
const MENU_HEIGHT: u32 = 100;
const TOTAL_HEIGHT: u32 = BAR_HEIGHT + MENU_HEIGHT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Mode {
    Bar,
    Menu,
}

pub enum ModeState {
    #[allow(dead_code)]
    Bar(BarState),
    Menu(MenuState),
}

pub struct BarState {
    #[allow(dead_code)]
    pub last_update: std::time::Instant,
}

pub struct MenuState {
    pub input: String,
    pub items: Vec<MenuItem>,
    pub filtered: Vec<usize>,
    pub selected: usize,
}

impl MenuState {
    pub fn new(items: Vec<MenuItem>) -> Self {
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
                    let haystack = Utf32String::from(item.display.as_str());
                    pattern.score(haystack.slice(..), matcher)
                        .map(|score| (idx, score))
                })
                .collect();
            
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered = scored.into_iter().map(|(idx, _)| idx).collect();
        }
        
        self.selected = 0;
    }
    
    pub fn get_selected_item(&self) -> Option<String> {
        self.filtered.get(self.selected)
            .and_then(|&idx| self.items.get(idx))
            .map(|item| item.exec.clone())
    }
}

pub struct ModeTransition;

impl ModeTransition {
    pub fn to_bar(layer: &LayerSurface) {
        layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
        layer.set_size(0, BAR_HEIGHT);
        layer.set_exclusive_zone(BAR_HEIGHT as i32);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.wl_surface().commit();
    }
    
    pub fn to_menu(layer: &LayerSurface) {
        layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
        layer.set_size(0, TOTAL_HEIGHT);
        layer.set_exclusive_zone(BAR_HEIGHT as i32);
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        layer.wl_surface().commit();
    }
}

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
    
    #[allow(dead_code)]
    pub fn is_menu_mode(&self) -> bool {
        matches!(self.mode, ModeState::Menu(_))
    }
    
    pub fn enter_menu(&mut self, items: Vec<MenuItem>, layer: &LayerSurface) {
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
