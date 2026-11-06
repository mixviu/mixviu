// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! A read-only snapshot, in registry draw order (back to front).
//! Bounds retain their original world-times-scale / physical coordinate marker.
//! Project only with the caller's pane camera; never cache a global camera here.
use crate::{HandleId, Transform, UiSpace};
use smithay::utils::{Physical, Point, Rectangle, Size};
#[derive(Clone, Debug)]
pub struct UiItemView {
    pub id: HandleId,
    pub space: UiSpace,
    pub location: Point<i32, Physical>,
    pub size: Size<i32, Physical>,
    pub scale_factor: f32,
    pub zoom_lock: Option<f32>,
    pub pending_resize: Option<(Size<i32, Physical>, f32)>,
    pub output: Option<String>,
    pub layer: u64,
    pub draw_order: usize,
    pub visible: bool,
    pub passthrough: bool,
    pub keyboard_transparent: bool,
}
impl UiItemView {
    pub fn pending_resize(&self) -> Option<(Size<i32, Physical>, f32)> {
        self.pending_resize
    }
    pub fn handle_id(&self) -> HandleId {
        self.id
    }
    pub fn space(&self) -> UiSpace {
        self.space
    }
    pub fn location(&self) -> Point<i32, Physical> {
        self.location
    }
    pub fn size(&self) -> Size<i32, Physical> {
        self.size
    }
    pub fn output(&self) -> Option<&str> {
        self.output.as_deref()
    }
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    pub fn is_passthrough(&self) -> bool {
        self.passthrough
    }
    pub fn is_keyboard_transparent(&self) -> bool {
        self.keyboard_transparent
    }
    pub fn world_zoom(&self, transform: &Transform) -> f64 {
        match (self.space, self.zoom_lock) {
            (UiSpace::Screen, _) => 1.0,
            (UiSpace::World, Some(ss)) => 1.0 / (ss.max(0.01) as f64),
            (UiSpace::World, None) => transform.zoom,
        }
    }

    /// Pin this world item's on-screen size, optionally supersampled (see
    /// `zoom_lock`). The caller is responsible for allocating a buffer `ss` times
    /// the intended on-screen size with a matching UI scale factor.
    pub fn set_zoom_lock(&mut self, lock: Option<f32>) {
        self.zoom_lock = lock;
    }

    pub fn zoom_lock(&self) -> Option<f32> {
        self.zoom_lock
    }

    pub fn screen_location(
        &self,
        transform: &Transform,
        output_size: Size<f64, Physical>,
    ) -> Point<i32, Physical> {
        match self.space {
            UiSpace::Screen => self.location,
            UiSpace::World => {
                let world = self.location;
                let s = transform
                    .world_to_screen(output_size, Point::from((world.x as f64, world.y as f64)));
                Point::from((s.x as i32, s.y as i32))
            }
        }
    }

    /// On-screen size: the buffer scaled by `world_zoom` (1.0 for Screen items
    /// and for zoom-locked World ones, the camera zoom otherwise).
    pub fn screen_size(&self, transform: &Transform) -> Size<i32, Physical> {
        let size = self.size;
        let zoom = self.world_zoom(transform);
        if (zoom - 1.0).abs() < f64::EPSILON {
            return size;
        }
        Size::from(((size.w as f64 * zoom) as i32, (size.h as f64 * zoom) as i32))
    }

    /// Full on-screen rectangle (location + size, both transformed).
    pub fn screen_rect(
        &self,
        transform: &Transform,
        output_size: Size<f64, Physical>,
    ) -> Rectangle<i32, Physical> {
        Rectangle::new(
            self.screen_location(transform, output_size),
            self.screen_size(transform),
        )
    }

    /// True if the given screen point falls inside this item's current
    /// screen rect.
    pub fn contains_screen_point(
        &self,
        point: Point<f64, Physical>,
        transform: &Transform,
        output_size: Size<f64, Physical>,
    ) -> bool {
        let r = self.screen_rect(transform, output_size);
        let r = Rectangle::<f64, Physical>::new(
            (r.loc.x as f64, r.loc.y as f64).into(),
            (r.size.w as f64, r.size.h as f64).into(),
        );
        r.contains(point)
    }

    /// Translate a screen point to surface-local logical coords (texture
    /// natural pixel grid). None if outside the screen rect.
    pub fn local_coords(
        &self,
        screen_point: Point<f64, Physical>,
        transform: &Transform,
        output_size: Size<f64, Physical>,
    ) -> Option<(f32, f32)> {
        let r = self.screen_rect(transform, output_size);
        let r_f = Rectangle::<f64, Physical>::new(
            (r.loc.x as f64, r.loc.y as f64).into(),
            (r.size.w as f64, r.size.h as f64).into(),
        );
        if !r_f.contains(screen_point) {
            return None;
        }
        Some(self.local_coords_unclamped(screen_point, transform, output_size))
    }

    /// [`Self::local_coords`] for any screen point, inside the rect or not: a
    /// pointer grab follows the pointer off the surface.
    pub fn local_coords_unclamped(
        &self,
        screen_point: Point<f64, Physical>,
        transform: &Transform,
        output_size: Size<f64, Physical>,
    ) -> (f32, f32) {
        let r = self.screen_rect(transform, output_size);
        let local = (
            screen_point.x - r.loc.x as f64,
            screen_point.y - r.loc.y as f64,
        );
        // Map the on-screen offset into the surface's UI LOGICAL space — the
        // space UI lays out and hit-tests in (see `UiRuntime::tick`, which
        // builds the UI at `viewport.logical_size()`). The on-screen rect spans
        // `size × world_zoom` physical px while the logical extent is
        // `size / scale_factor`, so the offset scales by
        // `logical / screen = 1 / (world_zoom × scale_factor)`.
        //
        // Both factors matter. A zoom-locked overlay has world_zoom 1 and factor
        // 1 (1:1). A plain World surface scales by the camera zoom alone.
        // Dividing by the camera zoom ALONE mis-mapped a zoomed-out counter-
        // scaled surface — the selection toolbar took clicks at the wrong spot.
        let divisor = self.world_zoom(transform) * self.scale_factor as f64;
        ((local.0 / divisor) as f32, (local.1 / divisor) as f32)
    }
}
