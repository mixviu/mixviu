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
    observed_path(path, added);
}

fn observed_path(path: PathBuf, added: bool) {
    KNOWN.with_borrow_mut(|known| {
        if added {
            match std::fs::metadata(&path) {
                Ok(meta) => {
                    known.insert(
                        path,
                        (meta.rdev(), meta.ino(), meta.ctime(), meta.ctime_nsec()),
                    );
                }
                Err(error) => debug!("input DeviceAdded metadata failed for {}: {error}", path.display()),
            }
        } else {
            known.remove(&path);
        }
    });
}

#[derive(Default)]
struct Retry {
    failures: u32,
    pending: Option<(Nodes, Nodes)>,
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

fn needs_reconcile(current: &Nodes, known: &Nodes, retry: &mut Retry) -> bool {
    // A failed resume must retry even if the nodes have since converged.
    if retry.failures != 0 {
        return true;
    }
    if current == known {
        retry.pending = None;
        return false;
    }
    // The scan can precede a queued libinput lifecycle event. Give it one
    // full tick to arrive, and restart the grace period if either map changes.
    let disagreement = (current.clone(), known.clone());
    let confirmed = retry.pending.as_ref() == Some(&disagreement);
    retry.pending = Some(disagreement);
    confirmed
}

fn reconcile(current: Nodes, retry: &mut Retry, reset: impl FnOnce() -> bool) -> Option<Duration> {
    if !KNOWN.with_borrow(|known| needs_reconcile(&current, known, retry)) {
        return None;
    }
    let success = reset();
    let delay = retry.completed(success);
    if success {
        retry.pending = None;
        KNOWN.with_borrow_mut(|known| *known = current);
    }
    Some(delay)
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
                    && let Some(delay) = reconcile(current, &mut retry, || {
                        // One context owns every device; resume re-enumerates.
                        super::release_held(state);
                        input.suspend();
                        let success = input.resume().is_ok();
                        wake.set(true);
                        success
                    })
                {
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
    fn scan_before_lifecycle_event_does_not_reset_input() {
        let root = std::env::temp_dir().join(format!("viud-scan-first-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("event0");
        std::fs::write(&path, []).unwrap();
        let added = nodes_at(&root).unwrap();
        KNOWN.with_borrow_mut(|known| known.clear());
        let mut retry = Retry::default();
        // DeviceAdded is queued when the first scan sees the new node.
        assert_eq!(reconcile(added.clone(), &mut retry, || panic!("suspend/resume")), None);
        observed_path(path.clone(), true);
        assert_eq!(reconcile(added, &mut retry, || panic!("suspend/resume")), None);
        // Likewise for a queued DeviceRemoved.
        std::fs::remove_file(&path).unwrap();
        assert_eq!(reconcile(Nodes::new(), &mut retry, || panic!("suspend/resume")), None);
        observed_path(path, false);
        assert_eq!(reconcile(Nodes::new(), &mut retry, || panic!("suspend/resume")), None);
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    fn missed_lifecycle_event_resets_once_on_next_scan() {
        let current = Nodes::from([(PathBuf::from("/dev/input/event0"), (1, 2, 3, 4))]);
        KNOWN.with_borrow_mut(|known| known.clear());
        let mut retry = Retry::default();
        assert_eq!(reconcile(current.clone(), &mut retry, || panic!("first scan")), None);
        let mut resets = 0;
        assert_eq!(reconcile(current.clone(), &mut retry, || { resets += 1; true }), Some(Duration::from_millis(250)));
        assert_eq!(reconcile(current, &mut retry, || panic!("already reconciled")), None);
        assert_eq!(resets, 1);
        KNOWN.with_borrow_mut(|known| known.clear());
    }

    #[test]
    fn changed_disagreement_restarts_grace_period() {
        let first = Nodes::from([(PathBuf::from("/dev/input/event0"), (1, 2, 3, 4))]);
        let replacement = Nodes::from([(PathBuf::from("/dev/input/event0"), (1, 5, 6, 7))]);
        let mut retry = Retry::default();
        assert!(!needs_reconcile(&first, &Nodes::new(), &mut retry));
        assert!(!needs_reconcile(&replacement, &Nodes::new(), &mut retry));
        assert!(needs_reconcile(&replacement, &Nodes::new(), &mut retry));
    }
    #[test]
    fn handled_hotplug_is_noop_and_missed_hotplug_reconciles_once() {
        let root = std::env::temp_dir().join(format!("viud-handled-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("event0");
        KNOWN.with_borrow_mut(|known| known.clear());
        let mut retry = Retry::default();
        let mut suspends = 0;
        let mut resumes = 0;
        let mut reset = || {
            suspends += 1;
            resumes += 1;
            true
        };
        std::fs::write(&path, []).unwrap();
        observed_path(path.clone(), true); // DeviceAdded before the scan.
        let handled = nodes_at(&root).unwrap();
        assert_eq!(reconcile(handled, &mut retry, &mut reset), None);
        std::fs::remove_file(&path).unwrap();
        observed_path(path.clone(), false); // DeviceRemoved before the scan.
        let handled = nodes_at(&root).unwrap();
        assert_eq!(reconcile(handled, &mut retry, &mut reset), None);
        std::fs::write(&path, []).unwrap(); // No DeviceAdded: CT lost udev.
        let current = nodes_at(&root).unwrap();
        for _ in 0..4 {
            reconcile(current.clone(), &mut retry, &mut reset);
        }
        assert_eq!((suspends, resumes), (1, 1));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
        KNOWN.with_borrow_mut(|known| known.clear());
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
    fn successful_resume_rearms_fast_scan_even_after_maximum_backoff() {
        let mut retry = Retry::default();
        for _ in 0..32 {
            retry.completed(false);
        }
        KNOWN.with_borrow_mut(|known| known.clear());
        assert_eq!(reconcile(Nodes::new(), &mut retry, || true), Some(Duration::from_millis(250)));
        assert_eq!(retry.failures, 0);
        assert!(retry.pending.is_none());
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
