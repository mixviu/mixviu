// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! LibSeatSession + notifier construction. (Ex wire.rs `new()` step 1.)
//! Failure policy: a compositor without a session cannot run — panic.
//!
//! viud: on a VT the session is opened only once that VT is the active one
//! ([`crate::seat::session::vt_wait`]): seatd re-reads the current VT only when a
//! client is ADDED (or on a VT signal it receives only for a VT it put in
//! process mode, which it does only for an active client), so a session
//! opened on an inactive VT is never enabled and its device opens are
//! refused (EPERM). Refusing to start on an inactive VT and leaving the
//! restart to the unit would also work; viud waits for the switch as an
//! event instead.

use smithay::backend::session::Session;
use smithay::backend::session::libseat::{LibSeatSession, LibSeatSessionNotifier};

/// Never wait on the compositor thread. A switch racing this attempt leaves
/// the offscreen desk running until the next activation notification.
pub fn try_create() -> Option<(LibSeatSession, LibSeatSessionNotifier)> {
    if let Some(own) = crate::seat::session::vt_wait::own_vt()
        && crate::seat::session::vt_wait::active_vt() != Some(own)
    {
        return None;
    }
    let (session, notifier) = match LibSeatSession::new() {
        Ok(session) => session,
        Err(error) => {
            warn!("seat creation deferred; keeping the offscreen desk alive: {error}");
            return None;
        }
    };
    if crate::seat::session::vt_wait::own_vt().is_some() && !session.is_active() {
        return None;
    }
    Some((session, notifier))
}

pub fn create() -> (LibSeatSession, LibSeatSessionNotifier) {
    let own_vt = crate::seat::session::vt_wait::own_vt();
    loop {
        crate::seat::session::vt_wait::wait_for_own_vt();
        let (session, notifier) = LibSeatSession::new().expect("libseat session creation failed");
        if let Some(own) = own_vt
            && !session.is_active()
        {
            if !matches!(crate::seat::session::vt_wait::active_vt(), Some(active) if active != own)
            {
                panic!(
                    "libseat session opened inactive although VT{own} is active: seatd is bound to a different tty than standard input; check TTYPath/StandardInput"
                );
            }
            info!(
                "libseat session opened inactive after switching away from VT{own}; dropping it and waiting for VT activation before retrying"
            );
            drop(notifier);
            drop(session);
            continue;
        }
        info!(
            "libseat session created (seat: {}, active: {})",
            session.seat(),
            session.is_active()
        );
        return (session, notifier);
    }
}
