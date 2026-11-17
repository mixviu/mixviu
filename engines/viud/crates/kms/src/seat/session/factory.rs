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
use std::sync::atomic::{AtomicU8, Ordering};

static SEAT_ERROR: AtomicU8 = AtomicU8::new(0);
static LAST_FAILURE: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub fn seat_error() -> Option<&'static str> {
    match SEAT_ERROR.load(Ordering::Relaxed) {
        1 => Some("libseat creation failed; check backend and device permissions"),
        2 => Some("libseat remained inactive while this VT was active; check backend VT binding"),
        3 => Some(
            "libseat creation failed and /dev/tty0 is missing; check seatd console access or use a supported logind session",
        ),
        4 => Some(
            "libseat remained inactive while this VT was active and /dev/tty0 is missing; check seatd console access",
        ),
        _ => None,
    }
}

fn remember_failure(last: &mut Option<String>, message: String) -> bool {
    if last.as_ref() == Some(&message) { return false; }
    *last = Some(message);
    true
}

fn failed(code: u8, detail: &str) -> bool {
    let code = code
        + if std::path::Path::new("/dev/tty0").exists() {
            0
        } else {
            2
        };
    SEAT_ERROR.store(code, Ordering::Relaxed);
    let message = format!("{detail}; {}", seat_error().unwrap());
    remember_failure(&mut LAST_FAILURE.lock().unwrap(), message)
}

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
            if failed(1, &error.to_string()) {
                warn!(
                    "seat creation deferred; keeping the offscreen desk alive: {error}; {}",
                    seat_error().unwrap()
                );
            }
            return None;
        }
    };
    if crate::seat::session::vt_wait::own_vt().is_some() && !session.is_active() {
        if failed(2, "inactive") {
            warn!("{}", seat_error().unwrap());
        }
        return None;
    }
    SEAT_ERROR.store(0, Ordering::Relaxed);
    *LAST_FAILURE.lock().unwrap() = None;
    Some((session, notifier))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distinct_libseat_errors_are_not_deduped() {
        let mut last = None;
        assert!(remember_failure(&mut last, "permission denied; creation failed".into()));
        assert!(!remember_failure(&mut last, "permission denied; creation failed".into()));
        assert!(remember_failure(&mut last, "connection refused; creation failed".into()));
        assert!(last.unwrap().contains("connection refused"));
    }
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
