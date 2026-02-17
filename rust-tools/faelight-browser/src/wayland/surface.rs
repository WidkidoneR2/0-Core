//! Wayland surface management
//! Based on faelight-bar patterns

use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::wlr_layer::{LayerShell, LayerSurface},
};
use wayland_client::{protocol::wl_surface, QueueHandle};

pub struct BrowserSurface {
    surface: wl_surface::WlSurface,
    layer_surface: Option<LayerSurface>,
}

impl BrowserSurface {
    pub fn new(compositor: &CompositorState, qh: &QueueHandle<crate::BrowserState>) -> Self {
        let surface = compositor.create_surface(qh);
        
        Self {
            surface,
            layer_surface: None,
        }
    }
    
    pub fn wl_surface(&self) -> &wl_surface::WlSurface {
        &self.surface
    }
}
