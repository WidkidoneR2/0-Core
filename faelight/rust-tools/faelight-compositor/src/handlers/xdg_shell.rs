use crate::FaelightCompositor;
use smithay::{
    delegate_xdg_shell,
    desktop::{PopupKind, Window},
    reexports::wayland_server::protocol::wl_seat,
    utils::Serial,
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    },
};

impl XdgShellHandler for FaelightCompositor {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        // INT-308 Phase 3: auto-tiling -- place windows side by side
        let output_size = self.space.outputs().next()
            .and_then(|o| self.space.output_geometry(o))
            .unwrap_or(smithay::utils::Rectangle::from_size((2560i32, 1600i32).into()));
        let win_count = self.space.elements().count();
        let x = if win_count == 0 {
            0
        } else {
            output_size.size.w / 2
        };
        // Configure window with half-screen size
        let half_w = output_size.size.w / 2;
        let full_h = output_size.size.h;
        window.toplevel().unwrap().with_pending_state(|state| {
            state.size = Some(smithay::utils::Size::from((half_w, full_h)));
        });
        self.space.map_element(window, (x, 0), false);
        self.health.windows_open = self.space.elements().count();

        // Give keyboard focus to the new window automatically
        let serial = smithay::utils::SERIAL_COUNTER.next_serial();
        if let Some(window) = self.space.elements().last().cloned() {
            let wl_surface = window.toplevel().unwrap().wl_surface().clone();
            self.seat
                .get_keyboard()
                .unwrap()
                .set_focus(self, Some(wl_surface), serial);
            window.set_activated(true);
            window.toplevel().unwrap().send_pending_configure();
            // Deactivate other windows (forest: only one active at a time)
            let active_surf = window.toplevel().and_then(|t| Some(t.wl_surface().clone()));
            let others: Vec<_> = self.space.elements().filter(|w| {
                w.toplevel().and_then(|t| Some(t.wl_surface().clone())) != active_surf
            }).cloned().collect();
            for w in others { w.set_activated(false); w.toplevel().map(|t| t.send_pending_configure()); }
        }

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

    fn move_request(&mut self, _surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
    }
    fn resize_request(
        &mut self,
        _surface: ToplevelSurface,
        _seat: wl_seat::WlSeat,
        _serial: Serial,
        _edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    ) {
    }
    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}
}

delegate_xdg_shell!(FaelightCompositor);
