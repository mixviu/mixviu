// SPDX-License-Identifier: MIT OR Apache-2.0
use std::time::Instant;
use graphics::surface::WgpuGlContext;
use std::sync::Arc;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::utils::{Physical, Point, Rectangle, Size};
use crate::{HandleId, Transform, UiSpace, UiRenderElement, UiItemView};
use crate::input::{Input, Modifiers};
use crate::error::{DispatchError, ResizeError};
#[derive(Clone, Copy, Debug)]
pub struct PaneView { pub transform: Transform, pub rect: Rectangle<i32, Physical> }
/// World-facing compositor surface operations. No typed programs or toolkit values.
pub trait UiSurfaces {
    fn allocate_id(&mut self) -> HandleId;
    fn gpu(&self) -> Arc<WgpuGlContext>;
    fn instance_scale(&self) -> f32;
    fn insert(&mut self, source: Box<dyn crate::producer::UiProducer>, space: UiSpace, layer: u64);
    fn request_redraw(&mut self, id: HandleId);
    fn items(&self) -> Vec<UiItemView>;
    fn get(&self, id: HandleId) -> Option<UiItemView> { self.items().into_iter().find(|item| item.id == id) }
    fn set_dealloc_enabled(&mut self, enabled: bool);
    fn set_draw_order(&mut self, topmost_first: &[HandleId]);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn contains(&self, id: HandleId) -> bool;
    fn set_instance_scale(&mut self, scale: f32);
    fn destroy_by_id(&mut self, id: HandleId) -> bool;
    fn set_location_by_id(&mut self, id: HandleId, location: Point<i32, Physical>) -> bool;
    fn raise(&mut self, id: HandleId);
    fn lower(&mut self, id: HandleId);
    fn location_of(&self, id: HandleId) -> Option<Point<i32, Physical>>;
    fn size_of(&self, id: HandleId) -> Option<Size<i32, Physical>>;
    fn space_of(&self, id: HandleId) -> Option<UiSpace>;
    fn request_resize_by_id(&mut self, id: HandleId, new_size: Size<i32, Physical>) -> bool;
    fn set_zoom_lock_by_id(&mut self, id: HandleId, lock: Option<f32>) -> bool;
    fn request_resize_scaled_by_id(
        &mut self,
        id: HandleId,
        new_size: Size<i32, Physical>,
        scale_factor: f32,
    ) -> bool;
    fn apply_pending_resizes(
        &mut self,
        render_node: &str,
        gles: &mut GlesRenderer,
    ) -> Result<usize, ResizeError>;
    fn dispatch_input(&mut self, id: HandleId, input: Input) -> Result<(), DispatchError>;
    fn route_pointer_to(
        &mut self,
        target: Option<HandleId>,
        screen_point: Point<f64, Physical>,
        transform: &Transform,
        output_size: Size<f64, Physical>,
    );
    fn dispatch_pointer_at(
        &mut self,
        point: Point<f64, Physical>,
        transform: &Transform,
        output_size: Size<f64, Physical>,
    ) -> Option<HandleId>;
    fn pointer_global_leave(&mut self);
    fn pointer_target(&self) -> Option<HandleId>;
    fn set_keyboard_focus(&mut self, target: Option<HandleId>);
    fn keyboard_focus(&self) -> Option<HandleId>;
    fn release_all_modifiers(&mut self);
    fn modifier_changed(&mut self, modifier: Modifiers, pressed: bool);
    fn effective_modifiers(&self) -> Modifiers;
    fn dispatch_axis(
        &mut self,
        target: Option<HandleId>,
        discrete_x: i32,
        discrete_y: i32,
        pixel_x: f64,
        pixel_y: f64,
        source: crate::input::ScrollSource,
    );
    fn dispatch_button(&mut self, target: Option<HandleId>, button_code: u32, pressed: bool);
    fn pointer_grab(&self) -> Option<HandleId>;
    fn hit_test(
        &self,
        point: Point<f64, Physical>,
        transform: &Transform,
        output_size: Size<f64, Physical>,
    ) -> Option<HandleId>;
    fn process_frame(&mut self, render_node: &str, gles: &mut GlesRenderer);
    fn manage_backings(
        &mut self,
        render_node: &str,
        gles: &mut GlesRenderer,
        output_size: Size<f64, Physical>,
        panes: &[PaneView],
    );
    fn release_all_backings(&mut self) -> usize;
    fn elements(
        &self,
        transform: &Transform,
        output_size: Size<f64, Physical>,
        layer: u64,
    ) -> Vec<UiRenderElement>;
    fn render_all(
        &mut self,
        render_node: &str,
        gles: &mut GlesRenderer,
        transform: Transform,
        output_size: Size<f64, Physical>,
        layer: u64,
    ) -> Result<Vec<UiRenderElement>, ResizeError>;
    fn prepare_frame(
        &mut self,
        render_node: &str,
        gles: &mut GlesRenderer,
        transform: Transform,
        output_size: Size<f64, Physical>,
    ) -> Result<(), ResizeError>;
    fn element_of(
        &self,
        id: HandleId,
        transform: &Transform,
        output_size: Size<f64, Physical>,
    ) -> Option<UiRenderElement>;
    fn is_visible(&self, id: HandleId) -> bool;
    fn set_visible_by_id(&mut self, id: HandleId, visible: bool) -> bool;
    fn retain_backing_by_id(&mut self, id: HandleId, retain: bool) -> bool;
    fn set_output_affinity_by_id(&mut self, id: HandleId, output: Option<String>) -> bool;
    fn output_affinity(&self, id: HandleId) -> Option<String>;
    fn set_opaque_occluder_by_id(&mut self, id: HandleId, opaque: bool) -> bool;
    fn is_passthrough(&self, id: HandleId) -> bool;
    fn set_passthrough_by_id(&mut self, id: HandleId, passthrough: bool) -> bool;
    fn is_keyboard_transparent(&self, id: HandleId) -> bool;
    fn set_keyboard_transparent_by_id(&mut self, id: HandleId, v: bool) -> bool;
    fn show_tooltip_by_id(&mut self, id: HandleId, at: Point<i32, Physical>) -> bool;
    fn hide_tooltip_by_id(&mut self, id: HandleId) -> bool;
    fn wants_frame(&self) -> bool;
    fn has_queued(&self) -> bool;
    fn has_input(&self) -> bool;
    fn tick_queued(&mut self);
    fn next_deadline(&self) -> Option<Instant>;
}
