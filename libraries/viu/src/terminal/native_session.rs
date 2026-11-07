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
    rejected: Arc<dyn Fn(&str) + Send + Sync>,
}

impl PaneSession {
    pub fn new(
        exited: Arc<dyn Fn() + Send + Sync>,
        revoke: Arc<dyn Fn() + Send + Sync>,
        rejected: Arc<dyn Fn(&str) + Send + Sync>,
    ) -> Self {
        Self { exited, revoke, rejected }
    }

    pub fn exit_notifier(&self) -> impl Fn() + Send + Sync + use<> {
        let exited = self.exited.clone();
        move || exited()
    }

    pub fn reject_launch(&self, reason: &str) {
        (self.rejected)(reason);
    }

    pub fn revoke_before_cleanup(self) {
        (self.revoke)();
    }
}
