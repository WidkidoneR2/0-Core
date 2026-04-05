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
