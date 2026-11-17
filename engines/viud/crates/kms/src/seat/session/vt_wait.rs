// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Wait, as an event, for this process's VT to become the active one before
//! the seat is opened (see `factory`).
//!
//! The VT is the one standard input is (a unit with `TTYPath=/dev/ttyN` and
//! `StandardInput=tty`, as viud's VT unit runs): character major
//! 4, minor 1–63. Anything else (a pipe, a pty, no tty: a nested or
//! dev run) does not wait. The kernel notifies `/sys/class/tty/tty0/active`
//! (POLLPRI) on every console switch — the attribute systemd-logind watches
//! the same way. Before attachment, a bounded poll also covers missing
//! notifications in containers. Nothing here switches VT; only a human does.

use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::io::AsRawFd;

const ACTIVE: &str = "/sys/class/tty/tty0/active";

/// The VT number standard input is, if it is one.
pub(super) fn own_vt() -> Option<u32> {
    let meta = std::fs::metadata("/dev/stdin").ok()?;
    if !meta.file_type().is_char_device() {
        return None;
    }
    vt_of(meta.rdev())
}

/// A character device number's VT, if it is one (major 4, minor 1..=63; the
/// minors from 64 are serial ports, 0 is the current console).
pub fn vt_of(rdev: u64) -> Option<u32> {
    let major = ((rdev >> 8) & 0xfff) | ((rdev >> 32) & !0xfff);
    let minor = (rdev & 0xff) | ((rdev >> 12) & !0xff);
    (major == 4 && (1..=63).contains(&minor)).then_some(minor as u32)
}

/// `tty0/active`'s content (`tty5\n`) as a VT number.
pub fn parse_active(text: &str) -> Option<u32> {
    text.trim().strip_prefix("tty")?.parse().ok()
}

/// Read the active VT once, if the attribute can be read and parsed.
pub(super) fn active_vt() -> Option<u32> {
    // Query the session tty directly: containers need not expose /dev/tty0
    // and may expose a sysfs mount without working switch notifications.
    if let Some(active) = ioctl_active() {
        return Some(active);
    }
    parse_active(&std::fs::read_to_string(ACTIVE).ok()?)
}

fn ioctl_active() -> Option<u32> {
    let mut state = [0u16; 3]; // struct vt_stat
    // SAFETY: VT_GETSTATE writes three unsigned shorts to valid storage.
    let result = unsafe { libc::ioctl(0, 0x5603, state.as_mut_ptr()) };
    (result == 0 && (1..=63).contains(&state[0])).then_some(u32::from(state[0]))
}

pub fn source_available() -> bool {
    own_vt().is_none() || active_vt().is_some()
}

/// Where the active VT is read from and waited on.
pub trait ActiveVt {
    /// The active VT now.
    fn read(&mut self) -> Option<u32>;
    /// Block until it may have changed.
    fn wait(&mut self) -> std::io::Result<()>;
}

/// Read until `own` is the active VT, waiting on the source between reads.
/// Returns how many waits it took (0: active already). A source that cannot
/// be read or waited on ends the wait: the seat open then fails or succeeds on
/// its own, as before this existed.
pub fn wait_until_active(own: u32, source: &mut impl ActiveVt) -> usize {
    let mut waits = 0;
    loop {
        match source.read() {
            Some(active) if active == own => return waits,
            None => return waits,
            Some(_) => {}
        }
        if let Err(error) = source.wait() {
            warn!("VT activation wait failed ({error}); opening the seat anyway");
            return waits;
        }
        waits += 1;
    }
}

/// `/sys/class/tty/tty0/active`, read from the start each time and polled
/// for POLLPRI (sysfs notification).
struct SysfsActive(std::fs::File);

/// Notify each activation without switching consoles. The initial
/// read also notifies an already active VT. Libseat takes over after attach.
pub fn watch_own_vt(
    mut activated: impl FnMut(),
    ready: impl FnOnce(),
    stop: &std::sync::atomic::AtomicBool,
) -> std::io::Result<()> {
    let Some(own) = own_vt() else {
        ready();
        activated();
        return Ok(());
    };
    let file = std::fs::File::open(ACTIVE).ok();
    if file.is_none() && ioctl_active().is_none() {
        return Err(std::io::Error::other(
            "no active VT source: expose the session tty (VT_GETSTATE) or /sys/class/tty/tty0/active; /dev/tty0 is not required",
        ));
    }
    let mut source = Monitor(file);
    if source.read().is_none() {
        return Err(std::io::Error::other(
            "VT source unreadable: expose the session tty or /sys/class/tty/tty0/active",
        ));
    }
    ready();
    watch_until_stopped(own, &mut source, stop, activated)
}

struct Monitor(Option<std::fs::File>);
impl ActiveVt for Monitor {
    fn read(&mut self) -> Option<u32> {
        active_vt()
    }
    fn wait(&mut self) -> std::io::Result<()> {
        if let Some(file) = &self.0 {
            let mut fd = libc::pollfd {
                fd: file.as_raw_fd(),
                events: libc::POLLPRI,
                revents: 0,
            };
            poll_monitor(&mut fd, |fd| {
                // SAFETY: a valid owned fd; the deadline covers lost sysfs edges.
                let rc = unsafe { libc::poll(fd, 1, 200) };
                if rc < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            })?;
            // Clear the sysfs notification before the next wait.
            let mut text = String::new();
            let mut file = file;
            let _ = file.seek(SeekFrom::Start(0));
            let _ = file.read_to_string(&mut text);
        } else {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        Ok(())
    }
}

fn poll_monitor(
    fd: &mut libc::pollfd,
    mut poll: impl FnMut(&mut libc::pollfd) -> std::io::Result<()>,
) -> std::io::Result<()> {
    loop {
        fd.revents = 0;
        match poll(fd) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => result?,
        }
        // kernfs reports a changed attribute as POLLERR | POLLPRI.
        if fd.revents & (libc::POLLNVAL | libc::POLLHUP) != 0
            || (fd.revents & libc::POLLERR != 0 && fd.revents & libc::POLLPRI == 0)
        {
            return Err(std::io::Error::other(
                "VT monitor poll error; re-arm watcher",
            ));
        }
        return Ok(());
    }
}

/// A failed read is a failed watcher, not an immediately-readable poll loop.
pub fn watch_changes(
    own: u32,
    source: &mut impl ActiveVt,
    activated: impl FnMut(),
) -> std::io::Result<()> {
    watch_until_stopped(
        own,
        source,
        &std::sync::atomic::AtomicBool::new(false),
        activated,
    )
}

fn watch_until_stopped(
    own: u32,
    source: &mut impl ActiveVt,
    stop: &std::sync::atomic::AtomicBool,
    mut activated: impl FnMut(),
) -> std::io::Result<()> {
    let mut previous = None;
    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
        let active = source
            .read()
            .ok_or_else(|| std::io::Error::other("VT source unreadable"))?;
        if active == own && previous != Some(active) {
            activated();
        }
        previous = Some(active);
        source.wait()?;
    }
    Ok(())
}

/// A retry timer may attempt a seat open only while this VT is active.
pub fn own_vt_active() -> bool {
    own_vt().is_none_or(|own| active_vt() == Some(own))
}

impl ActiveVt for SysfsActive {
    fn read(&mut self) -> Option<u32> {
        let mut text = String::new();
        self.0.seek(SeekFrom::Start(0)).ok()?;
        self.0.read_to_string(&mut text).ok()?;
        parse_active(&text)
    }

    fn wait(&mut self) -> std::io::Result<()> {
        let mut fd = libc::pollfd {
            fd: self.0.as_raw_fd(),
            events: libc::POLLPRI | libc::POLLERR,
            revents: 0,
        };
        loop {
            // SAFETY: one valid pollfd for an fd this struct owns; no timeout.
            let ready = unsafe { libc::poll(&mut fd, 1, -1) };
            if ready >= 0 {
                return Ok(());
            }
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }
}

/// Block until this process's VT is the active one (no-op off a VT).
pub fn wait_for_own_vt() {
    let Some(own) = own_vt() else { return };
    let Ok(file) = std::fs::File::open(ACTIVE) else {
        warn!("no {ACTIVE}: cannot wait for VT{own} activation; opening the seat anyway");
        return;
    };
    let mut source = SysfsActive(file);
    if source.read() != Some(own) {
        info!(
            "waiting for VT activation (seat inactive): VT{own} is not the active VT; nothing here switches it"
        );
    }
    let waits = wait_until_active(own, &mut source);
    if waits > 0 {
        info!("VT{own} is active (after {waits} console switch notification(s)): opening the seat");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_accepts_kernfs_priority_edges_with_or_without_pollerr() {
        for bits in [libc::POLLPRI, libc::POLLERR | libc::POLLPRI] {
            let mut fd = libc::pollfd {
                fd: -1,
                events: libc::POLLPRI,
                revents: 0,
            };
            poll_monitor(&mut fd, |fd| {
                fd.revents = bits;
                Ok(())
            })
            .unwrap();
        }
    }

    #[test]
    fn monitor_retries_eintr_and_rejects_poll_error_bits() {
        for bit in [
            libc::POLLERR,
            libc::POLLNVAL,
            libc::POLLHUP,
            libc::POLLNVAL | libc::POLLPRI,
            libc::POLLHUP | libc::POLLPRI,
        ] {
            let mut fd = libc::pollfd {
                fd: -1,
                events: libc::POLLPRI,
                revents: 0,
            };
            let mut calls = 0;
            assert!(
                poll_monitor(&mut fd, |fd| {
                    calls += 1;
                    if calls == 1 {
                        return Err(std::io::Error::from_raw_os_error(libc::EINTR));
                    }
                    fd.revents = bit;
                    Ok(())
                })
                .is_err()
            );
            assert_eq!(calls, 2);
        }
    }

    #[test]
    fn repeated_active_reads_notify_only_on_changes() {
        let mut source = Script {
            readings: vec![Some(4), Some(4), Some(1), Some(4), Some(4), None],
            at: 0,
            waits: 0,
        };
        let mut notices = 0;
        assert!(watch_changes(4, &mut source, || notices += 1).is_err());
        assert_eq!(notices, 2);
    }

    #[test]
    fn lost_notification_recheck_attaches_and_stops_after_handover() {
        let stop = std::sync::atomic::AtomicBool::new(false);
        let mut source = Script {
            readings: vec![Some(1), Some(4)],
            at: 0,
            waits: 0,
        };
        let mut attached = 0;
        watch_until_stopped(4, &mut source, &stop, || {
            attached += 1;
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
        })
        .unwrap();
        assert_eq!(attached, 1);
        assert_eq!(
            source.at, 2,
            "a timed recheck can replace the lost sysfs edge"
        );
    }

    #[test]
    fn watcher_waits_for_edges_and_returns_source_failures_for_rearming() {
        struct Source {
            at: usize,
        }
        impl ActiveVt for Source {
            fn read(&mut self) -> Option<u32> {
                [Some(1), Some(4), None][self.at]
            }
            fn wait(&mut self) -> std::io::Result<()> {
                self.at += 1;
                Ok(())
            }
        }
        let mut source = Source { at: 0 };
        let mut activated = 0;
        assert!(watch_changes(4, &mut source, || activated += 1).is_err());
        assert_eq!(activated, 1);
        assert_eq!(source.at, 2, "failed reads must not spin on poll");
    }

    /// A scripted active-VT source: each wait advances to the next reading.
    struct Script {
        readings: Vec<Option<u32>>,
        at: usize,
        waits: usize,
    }

    impl ActiveVt for Script {
        fn read(&mut self) -> Option<u32> {
            self.readings[self.at.min(self.readings.len() - 1)]
        }
        fn wait(&mut self) -> std::io::Result<()> {
            self.waits += 1;
            self.at += 1;
            Ok(())
        }
    }

    /// The seat is opened only once the session's own VT is the active one:
    /// at once when it already is; after the switch notifications otherwise
    /// (other VTs in between do not end the wait).
    #[test]
    fn the_seat_opens_only_when_its_vt_is_active() {
        let mut already = Script {
            readings: vec![Some(4)],
            at: 0,
            waits: 0,
        };
        assert_eq!(wait_until_active(4, &mut already), 0);
        let mut later = Script {
            readings: vec![Some(5), Some(1), Some(5), Some(4)],
            at: 0,
            waits: 0,
        };
        assert_eq!(wait_until_active(4, &mut later), 3);
        assert_eq!(
            later.waits, 3,
            "one wait per non-matching reading, none after the match"
        );
        let mut unreadable = Script {
            readings: vec![Some(5), None],
            at: 0,
            waits: 0,
        };
        assert_eq!(
            wait_until_active(4, &mut unreadable),
            1,
            "an unreadable source ends the wait"
        );
    }

    #[test]
    fn device_numbers_and_the_active_attribute_parse() {
        // tty4: major 4, minor 4 (old-style encoding).
        assert_eq!(vt_of((4 << 8) | 4), Some(4));
        assert_eq!(
            vt_of(4 << 8),
            None,
            "tty0 is the current console, not a VT of its own"
        );
        assert_eq!(vt_of((4 << 8) | 64), None, "ttyS0 shares major 4");
        assert_eq!(vt_of((136 << 8) | 1), None, "a pty");
        assert_eq!(parse_active("tty5\n"), Some(5));
        assert_eq!(parse_active("ttyS0"), None);
    }
}
