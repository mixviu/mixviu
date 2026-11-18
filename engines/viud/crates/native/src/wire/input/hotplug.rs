// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reconcile the existing udev context when device nodes change. Containers
//! can see bind-mounted input nodes without receiving netlink udev events.
use smithay::reexports::calloop::{
    LoopHandle,
    timer::{TimeoutAction, Timer},
};
use smithay::reexports::input::Libinput;
use std::{
    collections::BTreeMap,
    os::unix::fs::MetadataExt,
    path::PathBuf,
    time::{Duration, Instant},
};
use world::state::{Loop, StatusSession};

type Nodes = BTreeMap<PathBuf, (u64, u64)>;
thread_local! {
    static KNOWN: std::cell::RefCell<Nodes> = const { std::cell::RefCell::new(BTreeMap::new()) };
    static METADATA_RETRY: std::cell::RefCell<std::collections::BTreeSet<PathBuf>> = const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
    static METADATA_WARNING: std::cell::Cell<Option<std::time::Instant>> = const { std::cell::Cell::new(None) };
    static SCAN_WARNING: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) };
    static INPUT_CHANGED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn metadata_warning_due(last: &mut Option<std::time::Instant>, now: std::time::Instant) -> bool {
    if last.is_some_and(|last| now.duration_since(last) < Duration::from_secs(30)) {
        return false;
    }
    *last = Some(now);
    true
}

fn metadata_failed(path: &std::path::Path, error: std::io::Error) {
    METADATA_WARNING.with(|last| {
        let mut time = last.get();
        if metadata_warning_due(&mut time, std::time::Instant::now()) {
            warn!("input node metadata failed for {}: {error} (warnings limited to one per 30s)", path.display());
        }
        last.set(time);
    });
}

// Called on the same loop thread as the timer, after libinput handled udev.
pub(super) fn observed(sysname: &str, added: bool) {
    let path = PathBuf::from("/dev/input").join(sysname);
    observed_path(path, added);
}

fn observed_path(path: PathBuf, added: bool) {
    INPUT_CHANGED.set(true);
    KNOWN.with_borrow_mut(|known| {
        if added {
            match std::fs::metadata(&path) {
                Ok(meta) => {
                    METADATA_RETRY.with_borrow_mut(|pending| {
                        pending.remove(&path);
                    });
                    known.insert(path, (meta.rdev(), meta.ino()));
                }
                Err(error) => {
                    METADATA_RETRY.with_borrow_mut(|pending| {
                        pending.insert(path.clone());
                    });
                    metadata_failed(&path, error);
                }
            }
        } else {
            known.remove(&path);
            METADATA_RETRY.with_borrow_mut(|pending| {
                pending.remove(&path);
            });
        }
    });
}

#[derive(Default)]
struct Retry {
    failures: u32,
    pending: Option<(Nodes, Nodes)>,
    declined: BTreeMap<PathBuf, Declined>,
    probed: Option<Nodes>,
    grace_restarts: u32,
}
struct Declined {
    identity: (u64, u64),
    retry_at: Option<Instant>,
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
        retry.grace_restarts = 0;
        return false;
    }
    // The scan can precede a queued libinput lifecycle event. Give it one
    // full tick to arrive, and restart the grace period if either map changes.
    let disagreement = (current.clone(), known.clone());
    let confirmed = retry.pending.as_ref() == Some(&disagreement);
    if !confirmed && retry.pending.is_some() {
        retry.grace_restarts = retry.grace_restarts.saturating_add(1);
        if retry.grace_restarts == 8 {
            warn!("input nodes keep changing; reconciliation grace restarted eight times");
        }
    }
    retry.pending = Some(disagreement);
    // Bound grace even when a broken node is replaced on every scan.
    confirmed || retry.grace_restarts >= 8
}

fn reconcile(current: Nodes, retry: &mut Retry, reset: impl FnOnce() -> bool) -> Option<Duration> {
    reconcile_at(current, retry, Instant::now(), reset)
}

fn reconcile_at(
    current: Nodes,
    retry: &mut Retry,
    now: Instant,
    reset: impl FnOnce() -> bool,
) -> Option<Duration> {
    // Retry failed DeviceAdded stats before starting disagreement grace.
    let pending = METADATA_RETRY.with_borrow_mut(std::mem::take);
    for path in pending {
        match std::fs::metadata(&path) {
            Ok(meta) => KNOWN.with_borrow_mut(|known| {
                known.insert(path, (meta.rdev(), meta.ino()));
            }),
            Err(error) => {
                // A lifecycle event already admitted this device. Keep its
                // metadata pending so a transient stat cannot cause a reset.
                METADATA_RETRY.with_borrow_mut(|pending| {
                    pending.insert(path.clone());
                });
                metadata_failed(&path, error);
            }
        }
    }
    // Resume queues lifecycle events. On the following tick dispatch has had
    // a full interval to admit them; the remaining nodes were declined.
    if let Some(probed) = retry.probed.take() {
        KNOWN.with_borrow(|known| {
            for (path, identity) in probed {
                if current.get(&path) == Some(&identity) && known.get(&path) != Some(&identity) {
                    retry.declined.entry(path).or_insert_with_key(|path| {
                        info!("libinput declined {}; one retry in 5s", path.display());
                        Declined {
                            identity,
                            retry_at: Some(now + Duration::from_secs(5)),
                        }
                    });
                }
            }
        });
    }
    KNOWN.with_borrow(|known| {
        retry.declined.retain(|path, declined| {
            current.get(path) == Some(&declined.identity)
                && known.get(path) != Some(&declined.identity)
        })
    });
    let mut retry_declined = false;
    for declined in retry.declined.values_mut() {
        if declined.retry_at.is_some_and(|at| now >= at) {
            declined.retry_at = None;
            retry_declined = true;
        }
    }
    let admitted = current
        .iter()
        .filter(|(path, id)| {
            !METADATA_RETRY.with_borrow(|pending| pending.contains(*path))
                && retry
                    .declined
                    .get(*path)
                    .is_none_or(|declined| declined.identity != **id)
        })
        .map(|(path, id)| (path.clone(), *id))
        .collect();
    if !retry_declined && !KNOWN.with_borrow(|known| needs_reconcile(&admitted, known, retry)) {
        return None;
    }
    let success = reset();
    let delay = retry.completed(success);
    if success {
        retry.pending = None;
        retry.grace_restarts = 0;
        KNOWN.with_borrow_mut(|known| known.retain(|path, id| current.get(path) == Some(id)));
        retry.probed = Some(current);
    }
    Some(delay)
}
fn nodes() -> std::io::Result<Nodes> {
    nodes_at(std::path::Path::new("/dev/input"))
}

fn nodes_at(root: &std::path::Path) -> std::io::Result<Nodes> {
    nodes_at_with(root, |entry| entry.metadata())
}

fn nodes_at_with(
    root: &std::path::Path,
    metadata: impl Fn(&std::fs::DirEntry) -> std::io::Result<std::fs::Metadata>,
) -> std::io::Result<Nodes> {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error),
    };
    entries
        .filter_map(|entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => return Some(Err(error)),
            };
            if !entry.file_name().to_string_lossy().starts_with("event") {
                return None;
            }
            let meta = match metadata(&entry) {
                Ok(meta) => meta,
                Err(error) => {
                    metadata_failed(&entry.path(), error);
                    // Preserve this node's last identity; continue scanning
                    // other nodes so one failed stat cannot stall hotplug.
                    return KNOWN
                        .with_borrow(|known| known.get(&entry.path()).copied())
                        .map(|identity| Ok((entry.path(), identity)));
                }
            };
            Some(Ok((entry.path(), (meta.rdev(), meta.ino()))))
        })
        .collect()
}

fn scanned(result: std::io::Result<Nodes>) -> Option<Nodes> {
    match result {
        Ok(nodes) => Some(nodes),
        Err(error) => {
            SCAN_WARNING.with(|last| {
                let mut time = last.get();
                if metadata_warning_due(&mut time, Instant::now()) {
                    warn!("input node scan failed: {error} (warnings limited to one per 30s)");
                }
                last.set(time);
            });
            None
        }
    }
}

struct Cadence {
    previous: Option<Nodes>,
    quiet_since: Instant,
}
impl Cadence {
    fn delay(&mut self, nodes: &Nodes, changed: bool, now: Instant) -> Duration {
        if changed || self.previous.as_ref() != Some(nodes) {
            self.quiet_since = now;
            self.previous = Some(nodes.clone());
        }
        if now.duration_since(self.quiet_since) >= Duration::from_secs(60) {
            Duration::from_secs(2)
        } else {
            Duration::from_millis(250)
        }
    }
}

// Called by the session path even if pause and resume happen between scans.
pub(crate) fn resumed() {
    INPUT_CHANGED.set(true);
}

pub fn register(
    handle: &LoopHandle<'static, Loop>,
    mut input: Libinput,
    wake: std::rc::Rc<std::cell::Cell<bool>>,
) {
    let mut retry = Retry {
        probed: scanned(nodes()),
        ..Default::default()
    };
    let mut cadence = Cadence {
        previous: None,
        quiet_since: Instant::now(),
    };
    handle
        .insert_source(
            Timer::from_duration(Duration::from_millis(250)),
            move |_, _, state| {
                if matches!(state.inner.status_session, StatusSession::Active)
                    && let Some(current) = scanned(nodes())
                {
                    let changed = INPUT_CHANGED.replace(false) || retry.failures != 0;
                    let delay = cadence.delay(&current, changed, Instant::now());
                    if let Some(delay) = reconcile(current, &mut retry, || {
                        // One context owns every device; resume re-enumerates.
                        super::release_held(state);
                        input.suspend();
                        let success = input.resume().is_ok();
                        wake.set(true);
                        success
                    }) {
                        // A resume starts a new quiet period, even if nodes agree.
                        cadence.quiet_since = Instant::now();
                        return TimeoutAction::ToDuration(delay);
                    }
                    return TimeoutAction::ToDuration(delay);
                }
                cadence.quiet_since = Instant::now();
                TimeoutAction::ToDuration(Duration::from_millis(250))
            },
        )
        .expect("input node reconciliation timer");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn declined_nodes_get_one_delayed_retry_and_can_then_be_admitted() {
        for admitted in [false, true] {
            KNOWN.with_borrow_mut(|known| known.clear());
            let node = Nodes::from([(PathBuf::from("/dev/input/event0"), (1, 2))]);
            let now = Instant::now();
            let mut retry = Retry {
                probed: Some(node.clone()),
                ..Default::default()
            };
            assert_eq!(
                reconcile_at(node.clone(), &mut retry, now, || panic!("too early")),
                None
            );
            assert_eq!(
                reconcile_at(
                    node.clone(),
                    &mut retry,
                    now + Duration::from_secs(4),
                    || panic!("too early")
                ),
                None
            );
            assert!(
                reconcile_at(
                    node.clone(),
                    &mut retry,
                    now + Duration::from_secs(5),
                    || true
                )
                .is_some()
            );
            if admitted {
                KNOWN.with_borrow_mut(|known| *known = node.clone());
            }
            for seconds in [6, 20, 60, 120] {
                assert_eq!(
                    reconcile_at(
                        node.clone(),
                        &mut retry,
                        now + Duration::from_secs(seconds),
                        || panic!("only one delayed retry")
                    ),
                    None
                );
            }
            assert_eq!(retry.declined.is_empty(), admitted);
        }
        KNOWN.with_borrow_mut(|known| known.clear());
    }

    #[test]
    fn failed_stat_preserves_known_node_and_scans_other_nodes() {
        let root = TempDir::new();
        let path = root.0.join("event0");
        std::fs::write(&path, []).unwrap();
        let other = root.0.join("event1");
        std::fs::write(&other, []).unwrap();
        KNOWN.with_borrow_mut(|known| *known = Nodes::from([(path.clone(), (1, 2))]));
        let nodes = nodes_at_with(&root.0, |entry| {
            if entry.path() == path {
                Err(std::io::Error::from_raw_os_error(libc::EACCES))
            } else {
                entry.metadata()
            }
        })
        .unwrap();
        assert_eq!(nodes.get(&path), Some(&(1, 2)));
        assert!(nodes.contains_key(&other));
        KNOWN.with_borrow_mut(|known| known.clear());
        SCAN_WARNING.set(None);
        assert!(scanned(nodes_at(&path)).is_none());
        let first = SCAN_WARNING.get().unwrap();
        assert!(scanned(nodes_at(&path)).is_none()); // read_dir on a file fails too.
        assert_eq!(SCAN_WARNING.get(), Some(first));
        assert!(nodes_at(&root.0.join("absent")).unwrap().is_empty());
    }

    #[test]
    fn quiet_scan_slows_and_change_or_resume_restores_fast_interval() {
        let now = Instant::now();
        let mut cadence = Cadence {
            previous: None,
            quiet_since: now,
        };
        let empty = Nodes::new();
        assert_eq!(
            cadence.delay(&empty, false, now),
            Duration::from_millis(250)
        );
        assert_eq!(
            cadence.delay(&empty, false, now + Duration::from_secs(59)),
            Duration::from_millis(250)
        );
        assert_eq!(
            cadence.delay(&empty, false, now + Duration::from_secs(60)),
            Duration::from_secs(2)
        );
        let changed = Nodes::from([(PathBuf::from("event0"), (1, 2))]);
        assert_eq!(
            cadence.delay(&changed, false, now + Duration::from_secs(61)),
            Duration::from_millis(250)
        );
        assert_eq!(
            cadence.delay(&changed, false, now + Duration::from_secs(121)),
            Duration::from_secs(2)
        );
        resumed();
        assert_eq!(
            cadence.delay(
                &changed,
                INPUT_CHANGED.replace(false),
                now + Duration::from_secs(122)
            ),
            Duration::from_millis(250)
        );
    }

    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "viud-hotplug-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn declined_node_plug_and_unplug_reset_only_once() {
        KNOWN.with_borrow_mut(|known| known.clear());
        let node = Nodes::from([(PathBuf::from("/dev/input/event0"), (1, 2))]);
        let mut retry = Retry::default();
        assert_eq!(
            reconcile(node.clone(), &mut retry, || panic!("grace")),
            None
        );
        assert!(reconcile(node.clone(), &mut retry, || true).is_some());
        assert_eq!(reconcile(node, &mut retry, || panic!("declined")), None);
        for _ in 0..3 {
            assert_eq!(
                reconcile(Nodes::new(), &mut retry, || panic!("declined removal")),
                None
            );
        }
        assert!(retry.declined.is_empty());
    }

    #[test]
    fn metadata_retry_precedes_disagreement_and_warnings_are_throttled() {
        let root = TempDir::new();
        let path = root.0.join("event0");
        KNOWN.with_borrow_mut(|known| known.clear());
        observed_path(path.clone(), true); // First stat fails.
        std::fs::write(&path, []).unwrap();
        let mut retry = Retry::default();
        assert_eq!(
            reconcile(nodes_at(&root.0).unwrap(), &mut retry, || panic!(
                "metadata recovered"
            )),
            None
        );
        assert!(KNOWN.with_borrow(|known| known.contains_key(&path)));
        assert!(METADATA_RETRY.with_borrow(|pending| pending.is_empty()));
        let now = std::time::Instant::now();
        let mut last = None;
        assert!(metadata_warning_due(&mut last, now));
        assert!(!metadata_warning_due(
            &mut last,
            now + Duration::from_secs(1)
        ));
        assert!(metadata_warning_due(
            &mut last,
            now + Duration::from_secs(30)
        ));
        KNOWN.with_borrow_mut(|known| known.clear());
    }

    #[test]
    fn flapping_nodes_reconcile_at_the_once_only_warning_threshold() {
        let mut retry = Retry::default();
        for inode in 0..20 {
            let nodes = Nodes::from([(PathBuf::from("event0"), (1, inode))]);
            assert_eq!(
                needs_reconcile(&nodes, &Nodes::new(), &mut retry),
                inode >= 8
            );
        }
        assert_eq!(retry.grace_restarts, 19);
        assert!(!needs_reconcile(&Nodes::new(), &Nodes::new(), &mut retry));
        assert_eq!(retry.grace_restarts, 0);
    }

    #[test]
    fn repeated_device_added_stat_failure_does_not_reset_admitted_input() {
        let root = TempDir::new();
        let path = root.0.join("event0");
        KNOWN.with_borrow_mut(|known| known.clear());
        observed_path(path.clone(), true);
        // The node exists in the scan, but the lifecycle stat keeps failing.
        let scanned = Nodes::from([(path.clone(), (1, 2))]);
        let mut retry = Retry::default();
        for _ in 0..12 {
            assert_eq!(reconcile(scanned.clone(), &mut retry, || panic!("already admitted")), None);
        }
        std::fs::write(&path, []).unwrap();
        assert_eq!(reconcile(nodes_at(&root.0).unwrap(), &mut retry, || panic!("stat recovered")), None);
        assert!(METADATA_RETRY.with_borrow(|pending| pending.is_empty()));
        KNOWN.with_borrow_mut(|known| known.clear());
    }

    #[test]
    fn chmod_is_not_a_node_replacement() {
        use std::os::unix::fs::PermissionsExt;
        let root = TempDir::new();
        let path = root.0.join("event0");
        std::fs::write(&path, []).unwrap();
        let before = nodes_at(&root.0).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(before, nodes_at(&root.0).unwrap());
    }
    #[test]
    fn scan_before_lifecycle_event_does_not_reset_input() {
        let guard = TempDir::new();
        let root = guard.0.clone();
        let path = root.join("event0");
        std::fs::write(&path, []).unwrap();
        let added = nodes_at(&root).unwrap();
        KNOWN.with_borrow_mut(|known| known.clear());
        let mut retry = Retry::default();
        // DeviceAdded is queued when the first scan sees the new node.
        assert_eq!(
            reconcile(added.clone(), &mut retry, || panic!("suspend/resume")),
            None
        );
        observed_path(path.clone(), true);
        assert_eq!(
            reconcile(added, &mut retry, || panic!("suspend/resume")),
            None
        );
        // Likewise for a queued DeviceRemoved.
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            reconcile(Nodes::new(), &mut retry, || panic!("suspend/resume")),
            None
        );
        observed_path(path, false);
        assert_eq!(
            reconcile(Nodes::new(), &mut retry, || panic!("suspend/resume")),
            None
        );
    }

    #[test]
    fn missed_lifecycle_event_resets_once_on_next_scan() {
        let current = Nodes::from([(PathBuf::from("/dev/input/event0"), (1, 2))]);
        KNOWN.with_borrow_mut(|known| known.clear());
        let mut retry = Retry::default();
        assert_eq!(
            reconcile(current.clone(), &mut retry, || panic!("first scan")),
            None
        );
        let mut resets = 0;
        assert_eq!(
            reconcile(current.clone(), &mut retry, || {
                resets += 1;
                true
            }),
            Some(Duration::from_millis(250))
        );
        assert_eq!(
            reconcile(current, &mut retry, || panic!("already reconciled")),
            None
        );
        assert_eq!(resets, 1);
        KNOWN.with_borrow_mut(|known| known.clear());
    }

    #[test]
    fn changed_disagreement_restarts_grace_period() {
        let first = Nodes::from([(PathBuf::from("/dev/input/event0"), (1, 2))]);
        let replacement = Nodes::from([(PathBuf::from("/dev/input/event0"), (1, 5))]);
        let mut retry = Retry::default();
        assert!(!needs_reconcile(&first, &Nodes::new(), &mut retry));
        assert!(!needs_reconcile(&replacement, &Nodes::new(), &mut retry));
        assert!(needs_reconcile(&replacement, &Nodes::new(), &mut retry));
    }
    #[test]
    fn handled_hotplug_is_noop_and_missed_hotplug_reconciles_once() {
        let guard = TempDir::new();
        let root = guard.0.clone();
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
        assert_eq!(
            reconcile(Nodes::new(), &mut retry, || true),
            Some(Duration::from_millis(250))
        );
        assert_eq!(retry.failures, 0);
        assert!(retry.pending.is_none());
    }
    #[test]
    fn node_snapshot_detects_add_remove_and_same_path_replacement() {
        let guard = TempDir::new();
        let root = guard.0.clone();
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
    }
}
