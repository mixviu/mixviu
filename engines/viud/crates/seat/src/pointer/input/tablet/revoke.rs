// SPDX-License-Identifier: MIT OR Apache-2.0
//! Complete dispatcher tablet revocations through normal seat emulation.
pub fn flush(lp: &mut world::state::Loop, time: u32) {
    for button in lp.state.tablet.take_revoked_buttons() {
        if button == 0x110 {
            crate::pointer::input::touch::emulate::release(lp, time);
        } else {
            crate::pointer::input::touch::emulate::button(lp, button, false, time);
        }
    }
    if lp.state.tablet.take_revoked_cursor() {
        lp.state.seat.pointer_status = smithay::input::pointer::CursorImageStatus::default_named();
        lp.state.schedule_redraw(dispatcher::state::RedrawReason::Input);
    }
}
