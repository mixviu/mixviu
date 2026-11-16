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
thread_local! {
    static KNOWN: std::cell::RefCell<Nodes> = const { std::cell::RefCell::new(BTreeMap::new()) };
}

// Called on the same loop thread as the timer, after libinput handled udev.
pub(super) fn observed(sysname: &str, added: bool) {
    let path = PathBuf::from("/dev/input").join(sysname);
    KNOWN.with_borrow_mut(|known| {
        if added {
            if let Ok(meta) = std::fs::metadata(&path) {
                known.insert(path, (meta.rdev(), meta.ino(), meta.ctime(), meta.ctime_nsec()));
            }
        } else {
            known.remove(&path);
        }
    });
}

#[derive(Default)]
struct Retry {
    failures: u32,
}
impl Retry {
    fn completed(&mut self, success: bool) -> Duration {
        if success {
            if self.failures >= 3 {
                info!("input reconciliation recovered");
            }
            self.failures = 0;
            Duration::from_millis(250)
        } else {
            self.failures = self.failures.saturating_add(1);
            if self.failures == 3 {
                warn!("input resume failed three times; reconciliation will retry with backoff");
            }
            Duration::from_millis(250 * (1u64 << self.failures.min(5)))
        }
    }
}

fn needs_reconcile(current: &Nodes, known: &Nodes, retry: &Retry) -> bool {
    retry.failures != 0 || current != known
}
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

pub fn register(
    handle: &LoopHandle<'static, Loop>,
    mut input: Libinput,
    wake: std::rc::Rc<std::cell::Cell<bool>>,
) {
    KNOWN.with_borrow_mut(|known| *known = nodes().unwrap_or_default());
    let mut retry = Retry::default();
    handle
        .insert_source(
            Timer::from_duration(Duration::from_millis(250)),
            move |_, _, state| {
                if matches!(state.inner.status_session, StatusSession::Active)
                    && let Ok(current) = nodes()
                    && KNOWN.with_borrow(|known| needs_reconcile(&current, known, &retry))
                {
                    // One context owns every device, on both netlink and node
                    // changes. Resume re-enumerates; it cannot double-open one.
                    super::release_held(state);
                    input.suspend();
                    let success = input.resume().is_ok();
                    let delay = retry.completed(success);
                    wake.set(true);
                    if success {
                        KNOWN.with_borrow_mut(|known| *known = current);
                    }
                    return TimeoutAction::ToDuration(delay);
                }
                TimeoutAction::ToDuration(Duration::from_millis(250))
            },
        )
        .expect("input node reconciliation timer");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handled_hotplug_is_noop_and_missed_hotplug_reconciles_once() {
        let mut known = Nodes::new();
        let mut current = Nodes::new();
        current.insert(PathBuf::from("/dev/input/event0"), (1, 2, 3, 4));
        // DeviceAdded updates the remembered identity before the scan.
        known.clone_from(&current);
        let retry = Retry::default();
        assert!(!needs_reconcile(&current, &known, &retry));
        current.clear();
        known.clear(); // DeviceRemoved handled by libinput.
        assert!(!needs_reconcile(&current, &known, &retry));
        current.insert(PathBuf::from("/dev/input/event1"), (2, 3, 4, 5));
        let mut resets = 0;
        for _ in 0..4 {
            if needs_reconcile(&current, &known, &retry) {
                resets += 1;
                known.clone_from(&current);
            }
        }
        assert_eq!(resets, 1);
    }

    #[test]
    fn resume_failures_back_off_and_recovery_resets_delay() {
        let mut retry = Retry::default();
        assert_eq!(retry.completed(false), Duration::from_millis(500));
        assert_eq!(retry.completed(false), Duration::from_secs(1));
        for _ in 0..32 {
            assert!(retry.completed(false) <= Duration::from_secs(8));
        }
        assert_eq!(retry.completed(true), Duration::from_millis(250));
        assert_eq!(retry.failures, 0);
    }
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
