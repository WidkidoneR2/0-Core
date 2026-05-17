mod compositor;
mod xdg_shell;

use crate::FaelightCompositor;

use smithay::{
    delegate_data_device, delegate_output, delegate_seat,
    input::{Seat, SeatHandler, SeatState},
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Resource},
    wayland::{
        output::OutputHandler,
        selection::{
            data_device::{
                set_data_device_focus, DataDeviceHandler, DataDeviceState, WaylandDndGrabHandler,
            },
            SelectionHandler,
        },
    },
};

impl SeatHandler for FaelightCompositor {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<FaelightCompositor> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let dh = &self.display_handle;
        let client = focused.and_then(|s| dh.get_client(s.id()).ok());
        set_data_device_focus(dh, seat, client);

        // Emit window.focus into the forest ledger
        let payload = serde_json::json!({
            "focused": focused.is_some(),
        })
        .to_string();
        self.emit("window.focus", payload);
    }
}

delegate_seat!(FaelightCompositor);

impl SelectionHandler for FaelightCompositor {
    type SelectionUserData = ();
}

impl DataDeviceHandler for FaelightCompositor {
    fn data_device_state(&mut self) -> &mut DataDeviceState {
        &mut self.data_device_state
    }
}

impl WaylandDndGrabHandler for FaelightCompositor {}

delegate_data_device!(FaelightCompositor);

impl OutputHandler for FaelightCompositor {}
delegate_output!(FaelightCompositor);

use smithay::{
    delegate_primary_selection, delegate_xdg_decoration, delegate_xdg_activation,
    wayland::{
        selection::primary_selection::{
            PrimarySelectionHandler, PrimarySelectionState,
            set_primary_focus,
        },
        shell::xdg::decoration::{XdgDecorationHandler, XdgDecorationState},
        xdg_activation::{XdgActivationHandler, XdgActivationState, XdgActivationToken},
    },
    reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode,
};

impl PrimarySelectionHandler for FaelightCompositor {
    fn primary_selection_state(&mut self) -> &mut PrimarySelectionState {
        &mut self.primary_selection_state
    }
}

delegate_primary_selection!(FaelightCompositor);

impl XdgDecorationHandler for FaelightCompositor {
    fn new_decoration(&mut self, toplevel: smithay::wayland::shell::xdg::ToplevelSurface) {
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(Mode::ServerSide);
        });
        toplevel.send_configure();
    }
    fn request_mode(&mut self, _toplevel: smithay::wayland::shell::xdg::ToplevelSurface, _mode: Mode) {}
    fn unset_mode(&mut self, _toplevel: smithay::wayland::shell::xdg::ToplevelSurface) {}
}

delegate_xdg_decoration!(FaelightCompositor);

impl XdgActivationHandler for FaelightCompositor {
    fn activation_state(&mut self) -> &mut XdgActivationState {
        &mut self.xdg_activation_state
    }
    fn token_created(&mut self, _token: XdgActivationToken, _data: smithay::wayland::xdg_activation::XdgActivationTokenData) -> bool { true }
    fn request_activation(&mut self, _token: XdgActivationToken, _data: smithay::wayland::xdg_activation::XdgActivationTokenData, _surface: smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) {}
}

delegate_xdg_activation!(FaelightCompositor);

use smithay::{
    delegate_cursor_shape, delegate_fractional_scale,
    wayland::{
        cursor_shape::CursorShapeManagerState,
        fractional_scale::{FractionalScaleHandler, FractionalScaleManagerState},
    },
};

impl FractionalScaleHandler for FaelightCompositor {
    fn new_fractional_scale(&mut self, _surface: smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) {}
}

delegate_cursor_shape!(FaelightCompositor);
delegate_fractional_scale!(FaelightCompositor);

use smithay::wayland::tablet_manager::TabletSeatHandler;
use smithay::backend::input::TabletToolDescriptor;
use smithay::input::pointer::CursorImageStatus;
impl TabletSeatHandler for FaelightCompositor {
    fn tablet_tool_image(&mut self, _tool: &TabletToolDescriptor, _image: CursorImageStatus) {}
}
