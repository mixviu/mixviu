// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reconcile the existing udev context when device nodes change. Containers
//! can see bind-mounted input nodes without receiving netlink udev events.
use smithay::reexports::calloop::{
    LoopHandle,
    timer::{TimeoutAction, Timer},
};
use smithay::reexports::input::Libinput;
use std::{collections::BTreeMap, os::unix::fs::MetadataExt, path::PathBuf, time::Duration};
use world::state::{Loop, StatusSession};

type Nodes = BTreeMap<PathBuf, (u64, u64, i64, i64)>;
fn nodes() -> std::io::Result<Nodes> {
    nodes_at(std::path::Path::new("/dev/input"))
}

fn nodes_at(root: &std::path::Path) -> std::io::Result<Nodes> {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error),
    };
    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.file_name().to_string_lossy().starts_with("event") {
                return None;
            }
            let meta = entry.metadata().ok()?;
            Some(Ok((
                entry.path(),
                (meta.rdev(), meta.ino(), meta.ctime(), meta.ctime_nsec()),
            )))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn node_snapshot_detects_add_remove_and_same_path_replacement() {
        let root = std::env::temp_dir().join(format!("viud-input-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let empty = nodes_at(&root).unwrap();
        std::fs::write(root.join("event0"), []).unwrap();
        let added = nodes_at(&root).unwrap();
        assert_ne!(empty, added);
        assert_eq!(
            added,
            nodes_at(&root).unwrap(),
            "unchanged nodes do not reopen input"
        );
        std::fs::rename(root.join("event0"), root.join("old")).unwrap();
        std::fs::write(root.join("event0"), []).unwrap();
        assert_ne!(added, nodes_at(&root).unwrap());
        std::fs::remove_file(root.join("event0")).unwrap();
        assert_eq!(empty, nodes_at(&root).unwrap());
        std::fs::remove_file(root.join("old")).unwrap();
        std::fs::remove_dir(&root).unwrap();
    }
}

pub fn register(handle: &LoopHandle<'static, Loop>, mut input: Libinput) {
    let mut known = nodes().unwrap_or_default();
    let mut retry = false;
    handle
        .insert_source(
            Timer::from_duration(Duration::from_millis(250)),
            move |_, _, state| {
                if matches!(state.inner.status_session, StatusSession::Active)
                    && let Ok(current) = nodes()
                    && (retry || current != known) {
                            // One context owns every device, on both netlink and node
                            // changes. Resume re-enumerates; it cannot double-open one.
                            seat::modifier::clear::clear_held_modifiers(state);
                            input.suspend();
                            retry = input.resume().is_err();
                            if !retry {
                                known = current;
                            }
                }
                TimeoutAction::ToDuration(Duration::from_millis(250))
            },
        )
        .expect("input node reconciliation timer");
}
