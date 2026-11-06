// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Typed per-program controls. The compositor registry itself is never shared.
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use crate::{UiProgram, UiInstance, UiRuntime, HandleId};
pub struct UiControl<U: UiProgram> { pub(crate) inner: Rc<RefCell<UiInstance<U>>> }
impl<U: UiProgram> Clone for UiControl<U> { fn clone(&self) -> Self { Self { inner: self.inner.clone() } } }
impl<U: UiProgram> UiControl<U> {
    pub(crate) fn new(instance: UiInstance<U>) -> Self {
        let id = instance.handle_id();
        let inner = Rc::new(RefCell::new(instance));
        U::with_controls(|controls| { controls.retain(|_, v| v.strong_count() != 0); controls.insert(id, Rc::downgrade(&inner)); });
        Self { inner }
    }
    pub fn lookup(id: HandleId) -> Option<Self> { U::with_controls(|controls| controls.get(&id).and_then(|v| v.upgrade()).map(|inner| Self { inner })) }
    pub fn runtime(&self) -> Ref<'_, UiRuntime<U>> { Ref::map(self.inner.borrow(), |i| i.runtime()) }
    pub fn runtime_mut(&self) -> RefMut<'_, UiRuntime<U>> { RefMut::map(self.inner.borrow_mut(), |i| i.runtime_mut()) }
    pub fn ui(&self) -> Ref<'_, U> { Ref::map(self.inner.borrow(), |i| i.ui()) }
    pub fn ui_mut(&self) -> RefMut<'_, U> { RefMut::map(self.inner.borrow_mut(), |i| i.ui_mut()) }
}
#[macro_export]
macro_rules! program_owner {
    ($program:ty) => {
        fn with_controls<R>(f: impl FnOnce(&mut std::collections::HashMap<$crate::HandleId, std::rc::Weak<std::cell::RefCell<$crate::UiInstance<Self>>>>) -> R) -> R {
            thread_local! { static CONTROLS: std::cell::RefCell<std::collections::HashMap<$crate::HandleId, std::rc::Weak<std::cell::RefCell<$crate::UiInstance<$program>>>>> = std::cell::RefCell::new(std::collections::HashMap::new()); }
            CONTROLS.with(|controls| f(&mut controls.borrow_mut()))
        }
    };
}