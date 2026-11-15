// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reconcile the existing udev context when device nodes change. Containers
//! can see bind-mounted input nodes without receiving netlink udev events.
use std::{collections::BTreeMap, os::unix::fs::MetadataExt, path::PathBuf, time::Duration};
use smithay::reexports::calloop::{LoopHandle, timer::{Timer, TimeoutAction}};
use smithay::reexports::input::Libinput;
use world::state::{Loop, StatusSession};

fn nodes() -> std::io::Result<BTreeMap<PathBuf, (u64, u64, i64, i64)>> {
    std::fs::read_dir("/dev/input")?.filter_map(|entry| {
        let entry = entry.ok()?;
        if !entry.file_name().to_string_lossy().starts_with("event") { return None; }
        let meta = entry.metadata().ok()?;
        Some(Ok((entry.path(), (meta.rdev(), meta.ino(), meta.ctime(), meta.ctime_nsec()))))
    }).collect()
}

pub fn register(handle: &LoopHandle<'static, Loop>, mut input: Libinput) {
    let mut known = nodes().unwrap_or_default();
    let mut retry = false;
    handle.insert_source(Timer::from_duration(Duration::from_millis(250)), move |_, _, state| {
        if matches!(state.inner.status_session, StatusSession::Active) {
            if let Ok(current) = nodes() {
                if retry || current != known {
                    // One context owns every device, on both netlink and node
                    // changes. Resume re-enumerates; it cannot double-open one.
                    seat::modifier::clear::clear_held_modifiers(state);
                    input.suspend();
                    retry = input.resume().is_err();
                    if !retry { known = current; }
                }
            }
        }
        TimeoutAction::ToDuration(Duration::from_millis(250))
    }).expect("input node reconciliation timer");
}
