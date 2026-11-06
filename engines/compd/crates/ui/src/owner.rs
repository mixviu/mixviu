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
use ui_surface::UiSurfaces;
use crate::{UiHandle, UiSpace, UiSnapshot};
use crate::error::{CreateError, DispatchError};
use crate::instance::build_instance;
use graphics::surface::UiSurface;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::utils::{Physical, Point, Size};
/// Typed control operations are provided by the UI owner only.
pub trait TypedSurfaces: UiSurfaces {
    fn create<U: UiProgram>(
        &mut self,
        render_node: &str,
        ui: U,
        gles: &mut GlesRenderer,
        location: Point<i32, Physical>,
        size: Size<i32, Physical>,
        layer: u64,
    ) -> Result<UiHandle<U>, CreateError> {
        self.create_in_space(render_node, ui, gles, location, size, UiSpace::World, layer)
    }
    fn create_screen<U: UiProgram>(
        &mut self,
        render_node: &str,
        ui: U,
        gles: &mut GlesRenderer,
        location: Point<i32, Physical>,
        size: Size<i32, Physical>,
        layer: u64,
    ) -> Result<UiHandle<U>, CreateError> {
        self.create_in_space(
            render_node,
            ui,
            gles,
            location,
            size,
            UiSpace::Screen,
            layer,
        )
    }
    fn create_in_space_snapshot<U: UiSnapshot>(
        &mut self,
        render_node: &str,
        ui: U,
        gles: &mut GlesRenderer,
        location: Point<i32, Physical>,
        size: Size<i32, Physical>,
        space: UiSpace,
        layer: u64,
    ) -> Result<UiHandle<U>, CreateError> {
        self.create_with(render_node, ui, gles, location, size, space, layer)
    }
    fn create_in_space<U: UiProgram>(
        &mut self,
        render_node: &str,
        ui: U,
        gles: &mut GlesRenderer,
        location: Point<i32, Physical>,
        size: Size<i32, Physical>,
        space: UiSpace,
        layer: u64,
    ) -> Result<UiHandle<U>, CreateError> {
        self.create_with(render_node, ui, gles, location, size, space, layer)
    }
fn create_with<U: UiProgram>(
        &mut self,
        render_node: &str,
        ui: U,
        gles: &mut GlesRenderer,
        location: Point<i32, Physical>,
        size: Size<i32, Physical>,
        space: UiSpace,
        layer: u64,
    ) -> Result<UiHandle<U>, CreateError> {
        let id = self.allocate_id();
        let gpu = self.gpu();
        let surface = UiSurface::allocate(render_node, &gpu, gles, size)?;
        let instance = build_instance(id, ui, surface, gpu, location, self.instance_scale());
        let control = crate::UiControl::new(instance);
        self.insert(Box::new(crate::instance::SharedInstance(control)), space, layer);
        Ok(UiHandle::new(id))
    }
    fn destroy<U: UiProgram>(&mut self, handle: UiHandle<U>) -> bool {
        self.destroy_by_id(handle.id)
    }
    fn set_location<U: UiProgram>(
        &mut self,
        handle: UiHandle<U>,
        location: Point<i32, Physical>,
    ) -> bool {
        self.set_location_by_id(handle.id, location)
    }
    fn request_resize<U: UiProgram>(
        &mut self,
        handle: UiHandle<U>,
        new_size: Size<i32, Physical>,
    ) -> bool {
        self.request_resize_by_id(handle.id, new_size)
    }
fn dispatch_message<U: UiProgram>(
        &mut self,
        handle: UiHandle<U>,
        message: U::Message,
    ) -> Result<(), DispatchError> {
 let instance = self.instance_mut(handle).ok_or(DispatchError::UnknownHandle(handle.id))?;
 instance.runtime_mut().queue_message(message);
 Ok(())
 }
    fn set_message_handler<U: UiProgram>(
        &mut self,
        handle: UiHandle<U>,
        handler: impl FnMut(&U::Message) + Send + 'static,
    ) -> bool {
        match self.instance_mut(handle) {
            Some(inst) => {
                inst.runtime_mut().set_message_handler(handler);
                true
            }
            None => false,
        }
    }
    fn snapshot<U: UiSnapshot>(&self, handle: UiHandle<U>) -> Option<U::Snapshot> {
        self.instance(handle).map(|i| i.ui().snapshot())
    }
fn instance_mut<U: UiProgram>(
        &mut self,
        handle: UiHandle<U>,
    ) -> Option<crate::UiControl<U>> { if !self.contains(handle.id) { return None; } crate::UiControl::lookup(handle.id) }
fn instance<U: UiProgram>(&self, handle: UiHandle<U>) -> Option<crate::UiControl<U>> { if !self.contains(handle.id) { return None; } crate::UiControl::lookup(handle.id) }
    fn create_tooltip<U: UiProgram>(&mut self, render_node: &str, program: U, gles: &mut GlesRenderer, size: Size<i32, Physical>, layer: u64) -> Result<UiHandle<U>, CreateError> {
        let handle = self.create_screen(render_node, program, gles, Point::from((0, 0)), size, layer)?;
        self.set_passthrough_by_id(handle.id, true);
        self.set_visible_by_id(handle.id, false);
        Ok(handle)
    }
}
impl<T: UiSurfaces + ?Sized> TypedSurfaces for T {}

use crate::TypedSurfaces as _;
