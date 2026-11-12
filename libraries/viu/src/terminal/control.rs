// SPDX-License-Identifier: MIT OR Apache-2.0
//! Write-boundary permit mechanism extracted from gold. Admission is the
//! verified session owner's responsibility. A JSON request cannot mint one.
use bus::native_client::session::{Deadline, Hello};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Instant;

pub struct Permit {
    pub connection: Arc<dyn Fn() -> bool + Send + Sync>,
    pub live: AtomicBool,
    pub parent: Deadline,
    pub hello: Hello,
    pub until: Instant,
    pub pane: Arc<dyn Fn() -> bool + Send + Sync>,
    pub written: AtomicU64,
    /// A bounded, nonblocking notification supplied by the issuer. Gold's
    /// actor queues its typed notice here and wakes its native actor.
    pub notice: Mutex<Option<Box<dyn FnOnce(u64) + Send>>>,
}

impl Permit {
    pub fn valid(&self) -> bool {
        self.live.load(Ordering::Acquire)
            && (self.connection)()
            && Instant::now() < self.until
            && self.parent.is_live(&self.hello).unwrap_or(false)
            && (self.pane)()
    }

    pub fn revoke(&self) {
        if self.live.swap(false, Ordering::AcqRel)
            && let Some(notice) = self.notice.lock().unwrap().take()
        {
            notice(self.written.load(Ordering::Acquire));
        }
    }
}
