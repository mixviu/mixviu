// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Erased GPU surface source. Typed program storage never crosses this seam.
use crate::{HandleId, input::Input};
use graphics::surface::{SurfaceError, WgpuGlContext};
use smithay::backend::renderer::{element::Id, gles::GlesRenderer, utils::CommitCounter};
use smithay::utils::{Physical, Point, Size};
use std::time::Instant;
pub trait UiProducer {
    fn capture_setup(&self) -> Option<crate::capture::CaptureSetup> {
        None
    }
    fn capture_save_choice(&self) -> Option<crate::capture::SaveChoice> {
        None
    }
    fn capture_message(&mut self, _message: crate::capture::CaptureMessage) {}

    fn handle_id(&self) -> HandleId;
    fn smithay_id(&self) -> Id;
    fn commit(&self) -> CommitCounter;
    fn bump_commit(&mut self);
    fn location(&self) -> Point<i32, Physical>;
    fn size(&self) -> Size<i32, Physical>;
    /// The UI viewport scale factor (logical = physical / scale_factor). For
    /// `World` items this is the zoom counter-scale (`1/zoom`); for `Screen`
    /// items it's the instance scale (currently always 1.0).
    fn scale_factor(&self) -> f32;
    fn set_location(&mut self, p: Point<i32, Physical>);
    fn queue_input(&mut self, input: Input);
    /// Events or messages wait for the next tick (`UiRuntime::has_queued`).
    fn has_queued(&self) -> bool;
    /// Input or messages wait (`UiRuntime::has_input`); a repaint request
    /// alone does not count.
    fn has_input(&self) -> bool;
    fn tick(&mut self) -> bool;
    /// Run what queued input owes outside a frame, GPU untouched
    /// (`UiRuntime::tick_offscreen`).
    fn tick_offscreen(&mut self);
    /// Advance frame bookkeeping WITHOUT rasterizing: clears the runtime's dirty
    /// flag so an off-screen surface stops pinning the redraw loop, while its
    /// animation/message driving (in `tick`) continues untouched. Paired with a
    /// `stale` flag on the item so it re-renders once when it becomes visible.
    fn acknowledge_frame(&mut self);
    /// Off screen (`true`) or shown again (see `UiRuntime::set_suspended`).
    fn set_suspended(&mut self, suspended: bool);
    /// Force a pass and paint on the next frame (`UiRuntime::request_redraw`).
    fn request_redraw(&mut self);
    /// True if the runtime still wants to be rendered next frame — dirty or
    /// mid-animation. Drives the host's "keep scheduling frames" decision.
    fn wants_frame(&self) -> bool;
    /// The future instant an animation asked to be redrawn at (UI
    /// `RedrawRequest::At`), which `wants_frame` deliberately does not cover.
    fn deadline(&self) -> Option<Instant>;
    fn render(&mut self);
    /// Retire a pipelined frame if the GPU finished it, WITHOUT rasterizing, and
    /// bump the commit when one becomes visible. Must run every frame for every
    /// item: UI rasterizes only when dirty, so a deferred publish has no render
    /// call of its own to ride along with and would otherwise never land.
    fn poll_publish(&mut self);
    /// Whether the GPU backing is currently allocated. Off-screen surfaces are
    /// released to reclaim memory while their runtime keeps ticking.
    fn is_resident(&self) -> bool;
    /// Free the GPU backing (the GL texture ring). The `UiRuntime` keeps
    /// running; call `ensure_backing` before the next render.
    fn release_backing(&mut self);
    /// Re-allocate the backing at the current size if it was released.
    fn ensure_backing(
        &mut self,
        render_node: &str,
        wgpu_ctx: &WgpuGlContext,
        gles: &mut GlesRenderer,
    ) -> Result<(), SurfaceError>;
    /// Match the ring to the live setting. Per-frame, so the knob applies
    /// without a restart; a no-op while released or once the depth matches.
    fn sync_depth(
        &mut self,
        render_node: &str,
        wgpu_ctx: &WgpuGlContext,
        gles: &mut GlesRenderer,
        want: usize,
    ) -> Result<(), SurfaceError>;
    /// The GL texture UI last published, or `None` while the backing is
    /// released. The only output: UI draws into this texture on
    /// the compositor's own context.
    fn texture_handle(&self) -> Option<smithay::backend::renderer::gles::GlesTexture>;
    fn apply_pending_resize(
        &mut self,
        render_node: &str,
        wgpu_ctx: &WgpuGlContext,
        gles: &mut GlesRenderer,
    ) -> Result<bool, SurfaceError>;
    fn request_resize(&mut self, new_size: Size<i32, Physical>, scale_factor: f32);
    fn pending_resize(&self) -> Option<(Size<i32, Physical>, f32)>;
    fn pointer_leave(&mut self);
}
