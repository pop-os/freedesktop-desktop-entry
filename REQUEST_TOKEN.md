
```rs
platform_specific::wayland::activation::Action::RequestToken { app_id, window, message } => {
    if let Some(activation_state) = self.state.activation_state.as_ref() {
        let (seat_and_serial, surface) = if let Some(id) = window {
            let surface = self.state.windows.iter().find(|w| w.id == id)
                .map(|w| w.window.wl_surface().clone())
                .or_else(|| self.state.layer_surfaces.iter().find(|l| l.id == id)
                    .map(|l| l.surface.wl_surface().clone())
                );
            let seat_and_serial = surface.as_ref().and_then(|surface| {
                self.state.seats.first().and_then(|seat| if seat.kbd_focus.as_ref().map(|focus| focus == surface).unwrap_or(false) {
                    seat.last_kbd_press.as_ref().map(|(_, serial)| (seat.seat.clone(), *serial))
                } else if seat.ptr_focus.as_ref().map(|focus| focus == surface).unwrap_or(false) {
                    seat.last_ptr_press.as_ref().map(|(_, _, serial)| (seat.seat.clone(), *serial))
                } else {
                    None
                })
            });

            (seat_and_serial, surface)
        } else {
            (None, None)
        };

        activation_state.request_token_with_data(&self.state.queue_handle, IcedRequestData::new(
            RequestData {
                app_id,
                seat_and_serial,
                surface,
            },
            message,
        ));
    } else {
        // if we don't have the global, we don't want to stall the app
        sticky_exit_callback(
            IcedSctkEvent::UserEvent(message(None)),
            &self.state,
            &mut control_flow,
            &mut callback,
        )
    }
},
```