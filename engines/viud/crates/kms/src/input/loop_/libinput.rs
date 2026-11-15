// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The libinput backend as a loop source. Registration and the closures that
//! touch `Loop` live in `native.wire/wire.input` (Law 4).

use smithay::backend::libinput::LibinputInputBackend;
use smithay::reexports::input::Libinput;

pub use smithay::backend::libinput::LibinputInputBackend as LibinputSource;

pub fn source(context: Libinput) -> LibinputSource {
    let source = LibinputInputBackend::new(context);
    source.dispatch_waker().set(true);
    source
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::{calloop::{Dispatcher, EventLoop, EventSource}, input::LibinputInterface};
    struct NoDevices;
    impl LibinputInterface for NoDevices {
        fn open_restricted(&mut self, _: &std::path::Path, _: i32) -> Result<std::os::fd::OwnedFd, i32> { Err(libc::EACCES) }
        fn close_restricted(&mut self, _: std::os::fd::OwnedFd) {}
    }
    #[test]
    fn assignment_and_resume_wake_without_kernel_fd_readiness() {
        let context = Libinput::new_from_path(NoDevices);
        let backend = source(context);
        let wake = backend.dispatch_waker();
        let dispatcher = Dispatcher::new(backend, |_, _, _: &mut ()| {});
        let lp = EventLoop::<()>::try_new().unwrap();
        lp.handle().register_dispatcher(dispatcher.clone()).unwrap();
        for first in [true, false] {
            let mut backend = dispatcher.as_source_mut();
            if !first { wake.set(true); }
            let (ready, token) = backend.before_sleep().unwrap().expect("queued lifecycle events must synthesize readiness");
            assert!(!wake.get());
            backend.process_events(ready, token, |_, _| {}).unwrap();
            assert!(backend.before_sleep().unwrap().is_none(), "no busy loop once the queue is drained");
        }
    }
}
