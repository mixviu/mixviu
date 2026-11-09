// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lab-only, opt-in, thread-local inclusive stage timings. Never in a binary.
use std::{cell::RefCell, collections::BTreeMap, time::Instant};

thread_local! {
    static ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static STAGES: RefCell<BTreeMap<String, (u128, u64)>> = RefCell::default();
}
pub fn enable(active: bool) { ACTIVE.set(active); }
pub fn take() -> BTreeMap<String, (u128, u64)> {
    STAGES.with(|stages| std::mem::take(&mut *stages.borrow_mut()))
}
pub struct Span(Option<(String, Instant)>);
pub fn span(name: impl Into<String>) -> Span {
    Span(ACTIVE.get().then(|| (name.into(), Instant::now())))
}
pub fn count(name: &str) {
    if ACTIVE.get() {
        STAGES.with(|stages| stages.borrow_mut().entry(name.into()).or_default().1 += 1);
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        if let Some((name, start)) = &self.0 {
            let ns = start.elapsed().as_nanos();
            STAGES.with(|stages| {
                let mut stages = stages.borrow_mut();
                let stage = stages.entry(name.clone()).or_default();
                stage.0 += ns;
                stage.1 += 1;
            });
        }
    }
}
