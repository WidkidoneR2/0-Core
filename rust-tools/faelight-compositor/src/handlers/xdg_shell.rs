use crate::FaelightCompositor;
use smithay::{
    delegate_xdg_shell,
    desktop::{Space, Window, PopupManager, PopupKind},
    reexports::wayland_server::protocol::wl_seat,
    utils::Serial,
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ToplevelSurface,
        XdgShellHandler, XdgShellState,
    },
};

impl XdgShellHandler for FaelightCompositor {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        self.space.map_element(window, (0, 0), false);
        self.health.windows_open = self.space.elements().count();

        // Emit window.open into the forest ledger
        let payload = serde_json::json!({
            "workspace": self.health.active_workspace,
            "windows_open": self.health.windows_open,
        })
        .to_string();
        self.emit("window.open", payload);
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        let _ = self.popups.track_popup(PopupKind::Xdg(surface));
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        surface.with_pending_state(|state| {
            state.geometry = positioner.get_geometry();
            state.positioner = positioner;
        });
        surface.send_repositioned(token);
    }

    fn move_request(&mut self, _surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}
    fn resize_request(&mut self, _surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial, _edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge) {}
    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}
}

delegate_xdg_shell!(FaelightCompositor);
