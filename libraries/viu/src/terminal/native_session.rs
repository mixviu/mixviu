// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native pane lease seam. The issuer owns verified broker attachment; the
//! terminal owns only the issued descriptor and ordered lifecycle callbacks.
//! This interface creates no broker, credentials, tabs or pane layout policy.
use std::sync::Arc;

use super::session_fd::LaunchFd;

pub trait SessionIssuer: Send + Sync {
    fn prepare(&self, pane_id: u64) -> Option<(PaneSession, LaunchFd)>;
}

/// Issuer callbacks are in-process, never reconstructed from Mix JSON.
pub struct PaneSession {
    exited: Arc<dyn Fn() + Send + Sync>,
    revoke: Arc<dyn Fn() + Send + Sync>,
    cleanup: Arc<dyn Fn() + Send + Sync>,
    rejected: Arc<dyn Fn(&str) + Send + Sync>,
}

impl PaneSession {
    pub fn new(
        exited: Arc<dyn Fn() + Send + Sync>,
        revoke: Arc<dyn Fn() + Send + Sync>,
        cleanup: Arc<dyn Fn() + Send + Sync>,
        rejected: Arc<dyn Fn(&str) + Send + Sync>,
    ) -> Self {
        Self {
            exited,
            revoke,
            cleanup,
            rejected,
        }
    }

    pub fn exit_notifier(&self) -> impl Fn() + Send + Sync + use<> {
        let exited = self.exited.clone();
        move || exited()
    }

    pub fn reject_launch(self, reason: &str) {
        let rejected = self.rejected.clone();
        drop(self);
        rejected(reason);
    }

    pub fn revoke_before_cleanup(self) {
        // Local authority must be invalid before the cleanup callback can
        // wait for remote acknowledgement or release the PTY. Drop repeats
        // invalidation, so issuer revocation remains idempotent as in gold.
        (self.revoke)();
        (self.cleanup)();
    }
}

impl Drop for PaneSession {
    fn drop(&mut self) {
        // Covers failed PTY launches as well as ordinary close. As in gold,
        // issuer invalidation must be idempotent.
        (self.revoke)();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn local_authority_is_revoked_before_cleanup_waits() {
        let live = Arc::new(AtomicBool::new(true));
        let revoked = live.clone();
        let cleaned = live.clone();
        let cleanup_seen = Arc::new(AtomicBool::new(false));
        let observed = cleanup_seen.clone();
        let pane = PaneSession::new(
            Arc::new(|| {}),
            Arc::new(move || {
                revoked.store(false, Ordering::Release);
            }),
            Arc::new(move || {
                assert!(
                    !cleaned.load(Ordering::Acquire),
                    "cleanup observed live pane authority"
                );
                observed.store(true, Ordering::Release);
            }),
            Arc::new(|_| {}),
        );
        pane.revoke_before_cleanup();
        assert!(!live.load(Ordering::Acquire));
        assert!(cleanup_seen.load(Ordering::Acquire));
    }

    #[test]
    fn rejected_launch_revokes_before_reporting_failure() {
        let live = Arc::new(AtomicBool::new(true));
        let revoked = live.clone();
        let rejected = live.clone();
        let pane = PaneSession::new(
            Arc::new(|| {}),
            Arc::new(move || {
                revoked.store(false, Ordering::Release);
            }),
            Arc::new(|| panic!("failed launch must not wait for PTY cleanup")),
            Arc::new(move |reason| {
                assert_eq!(reason, "spawn refused");
                assert!(!rejected.load(Ordering::Acquire));
            }),
        );
        pane.reject_launch("spawn refused");
        assert!(!live.load(Ordering::Acquire));
    }
}
