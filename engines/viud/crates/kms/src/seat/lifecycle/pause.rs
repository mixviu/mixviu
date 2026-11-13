// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Pause ordering protocol (TTY switch away / session pause).
//!
//! Order is the protocol: pause the display path FIRST (stop touching DRM fds
//! that are about to be revoked), THEN suspend input. The affected pieces are
//! passed in as capabilities — this crate knows nothing about DRM or libinput.

pub fn pause(pause_display: impl FnOnce(), suspend_input: impl FnOnce()) {
    info!("session pause: pausing display path");
    pause_display();
    info!("session pause: suspending input");
    suspend_input();
}

/// Legacy GPU capture entries must retire before *any* KMS buffer/plane release.
/// Bus frame captures own synchronous CPU readbacks and need no retained KMS fd.
pub fn release<D>(
    data: &mut D,
    retire_capture: impl FnOnce(&mut D),
    release_display: impl FnOnce(&mut D),
) {
    retire_capture(data);
    release_display(data);
}

#[cfg(test)]
mod tests {
    #[test]
    fn capture_entries_retire_before_display_buffers_and_owned_pixels_survive() {
        struct Desk {
            kms: Option<Vec<u8>>,
            legacy_capture: bool,
            snapshot: Vec<u8>,
        }
        let mut desk = Desk {
            kms: Some(vec![1, 2, 3]),
            legacy_capture: true,
            snapshot: vec![1, 2, 3],
        };
        super::release(
            &mut desk,
            |desk| desk.legacy_capture = false,
            |desk| {
                assert!(!desk.legacy_capture);
                desk.kms.take();
            },
        );
        assert!(desk.kms.is_none());
        assert_eq!(desk.snapshot, [1, 2, 3]);
    }
}
