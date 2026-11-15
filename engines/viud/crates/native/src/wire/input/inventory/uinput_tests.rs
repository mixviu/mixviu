// SPDX-License-Identifier: MIT OR Apache-2.0
//! Requires writable /dev/uinput, visible /dev/input and host udev classification.
use super::*;
use smithay::reexports::{
    calloop::EventLoop,
    input::{Libinput, LibinputInterface},
};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    path::Path,
    time::{Duration, Instant},
};

struct Interface;
impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
        // SAFETY: a live NUL-terminated path; ownership transfers on success.
        let fd = unsafe { libc::open(path.as_ptr(), flags | libc::O_CLOEXEC) };
        if fd < 0 {
            Err(std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EIO))
        } else {
            // SAFETY: open returned a new owned fd. Grab only our virtual
            // node so the worker's desktop cannot receive the injected key.
            let owned = unsafe { OwnedFd::from_raw_fd(fd) };
            if unsafe { libc::ioctl(fd, 0x40044590, 1) } != 0 {
                return Err(std::io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(libc::EIO));
            }
            Ok(owned)
        }
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(fd);
    }
}

#[repr(C)]
struct Setup {
    id: [u16; 4],
    name: [u8; 80],
    ff: u32,
}
#[repr(C)]
struct Event {
    time: libc::timeval,
    kind: u16,
    code: u16,
    value: i32,
}

fn write_event(file: &mut File, kind: u16, code: u16, value: i32) {
    let event = Event {
        time: libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        kind,
        code,
        value,
    };
    // SAFETY: repr(C), all fields and padding initialised (input_event has no
    // padding on supported Linux targets); bytes are used only for this write.
    let bytes = unsafe {
        std::slice::from_raw_parts(
            (&event as *const Event).cast::<u8>(),
            std::mem::size_of::<Event>(),
        )
    };
    file.write_all(bytes).unwrap();
}

#[test]
fn virtual_keyboard_is_counted_before_human_input_and_delivers_a_key() {
    let mut device = match OpenOptions::new().write(true).open("/dev/uinput") {
        Ok(device) => device,
        Err(error) => {
            eprintln!(
                "NOT RUN uinput lifecycle/event test: cannot open /dev/uinput for writing: {error}; worker must expose writable uinput plus /dev/input and udev classification"
            );
            return;
        }
    };
    let mut setup = Setup {
        id: [3, 0x1234, 0x5678, 1],
        name: [0; 80],
        ff: 0,
    };
    let name = b"viud-regression-keyboard";
    setup.name[..name.len()].copy_from_slice(name);
    // SAFETY: documented uinput ioctls, with scalar bit numbers or live setup storage.
    unsafe {
        assert_eq!(
            libc::ioctl(device.as_raw_fd(), 0x40045564, 1),
            0,
            "UI_SET_EVBIT EV_KEY"
        );
        // A complete ordinary keyboard, so udev/libinput classify it as one.
        for key in 1..=127 {
            assert_eq!(libc::ioctl(device.as_raw_fd(), 0x40045565, key), 0);
        }
        assert_eq!(
            libc::ioctl(device.as_raw_fd(), 0x405c5503, &setup),
            0,
            "UI_DEV_SETUP"
        );
        assert_eq!(libc::ioctl(device.as_raw_fd(), 0x5501), 0, "UI_DEV_CREATE");
    }
    // Closing uinput destroys only this test's virtual device, including on panic.
    let mut sysname = [0u8; 128];
    assert!(unsafe { libc::ioctl(device.as_raw_fd(), 0x8080552c, sysname.as_mut_ptr()) } >= 0);
    let sysname = std::ffi::CStr::from_bytes_until_nul(&sysname)
        .unwrap()
        .to_str()
        .unwrap();
    let sysroot = std::path::PathBuf::from("/sys/devices/virtual/input").join(sysname);
    let until = Instant::now() + Duration::from_secs(5);
    let mut context = Libinput::new_from_path(Interface);
    let mut last_error = String::from("event node has not appeared in sysfs");
    let mut permission_denied = false;
    let node = loop {
        let node = std::fs::read_dir(&sysroot)
            .ok()
            .and_then(|entries| {
                entries
                    .flatten()
                    .find(|e| e.file_name().to_string_lossy().starts_with("event"))
            })
            .map(|e| format!("/dev/input/{}", e.file_name().to_string_lossy()));
        if let Some(node) = node {
            match OpenOptions::new().read(true).open(&node) {
                Ok(_) => {
                    permission_denied = false;
                    if context.path_add_device(&node).is_some() {
                        break node;
                    }
                    last_error = format!(
                        "libinput rejected readable {node}; verify udev keyboard classification and exclusive grab permission"
                    );
                }
                Err(error) => {
                    permission_denied = error.kind() == std::io::ErrorKind::PermissionDenied;
                    last_error = format!("cannot read {node}: {error}");
                }
            }
        }
        if Instant::now() >= until && permission_denied {
            eprintln!(
                "NOT RUN uinput lifecycle/event assertions: created {sysname}, but worker device access blocks the generated evdev node: {last_error}"
            );
            return;
        }
        assert!(
            Instant::now() < until,
            "uinput created {sysname}: {last_error}"
        );
        std::thread::sleep(Duration::from_millis(50));
    };
    let source = kms::input::loop_::libinput::source(context.clone());
    let wake = source.dispatch_waker();
    let mut lp = EventLoop::<Inventory>::try_new().unwrap();
    let mut inventory = Inventory::default();
    let delivered = std::rc::Rc::new(std::cell::Cell::new(false));
    let seen = delivered.clone();
    lp.handle()
        .insert_source(source, move |event, _, inventory| {
            if matches!(&event, InputEvent::Keyboard { .. }) {
                seen.set(true);
            }
            inventory.observe(&event);
        })
        .unwrap();
    for _ in 0..5 {
        lp.dispatch(Some(Duration::from_millis(20)), &mut inventory)
            .unwrap();
    }
    let rows = inventory.snapshot().0;
    assert_eq!(
        rows.len(),
        1,
        "DeviceAdded must drain without a key or fd readiness"
    );
    assert_eq!(rows[0].node, node);
    assert!(rows[0].capabilities.iter().any(|c| c == "keyboard"));
    assert_eq!(inventory.events, 0);
    write_event(&mut device, 1, 30, 1); // KEY_A
    write_event(&mut device, 0, 0, 0); // SYN_REPORT
    let until = Instant::now() + Duration::from_secs(2);
    while !delivered.get() {
        assert!(Instant::now() < until, "virtual key was not delivered");
        lp.dispatch(Some(Duration::from_millis(20)), &mut inventory)
            .unwrap();
    }
    assert!(inventory.events > 0);
    context.suspend();
    wake.set(true);
    for _ in 0..5 {
        lp.dispatch(Some(Duration::from_millis(20)), &mut inventory)
            .unwrap();
    }
    assert!(
        inventory.snapshot().0.is_empty(),
        "closed devices must leave the inventory without another input event"
    );
}
