// SPDX-License-Identifier: MIT OR Apache-2.0
//! Last imported client textures, independent of protocol/window lifetimes.
use super::element::{Element, ElementWindowSurface, RawElement};
use dispatcher::frame::SceneDispatch;
use motion::{Property, Values};
use smithay::backend::renderer::element::solid::SolidColorRenderElement;
use smithay::backend::renderer::element::surface::WaylandSurfaceTexture;
use smithay::backend::renderer::element::texture::TextureRenderElement;
use smithay::backend::renderer::element::{Element as _, Id, Kind};
use smithay::backend::renderer::utils::CommitCounter;
use smithay::backend::renderer::{Color32F, ContextId, ImportAll, ImportMem, Renderer, Texture};
use smithay::utils::{Buffer, Physical, Point, Rectangle, Scale, Transform};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use uuid::Uuid;

smithay::render_elements! {
    pub Ghost<R> where R: ImportAll + ImportMem + SceneDispatch;
    Texture = ElementWindowSurface<TextureRenderElement<R::TextureId>>,
    ClippedTexture = decor::clip::Clipped<ElementWindowSurface<TextureRenderElement<R::TextureId>>>,
    Solid = SolidColorRenderElement,
    ClippedSolid = decor::clip::Clipped<SolidColorRenderElement>,
}

struct Snapshot<T: Texture> {
    id: Id,
    texture: Option<T>,
    color: Option<Color32F>,
    context: ContextId<T>,
    geometry: Rectangle<i32, Physical>,
    src: Rectangle<f64, Buffer>,
    transform: Transform,
    alpha: f32,
    anchor: Point<i32, Physical>,
    logical_scale: f64,
    clip: Option<(Option<dispatcher::frame::TexProgram>, decor::clip::Clip)>,
    // Retaining the protocol buffer delays only its release/reuse, never client
    // destruction. A texture clone alone could outlive wl_buffer.release.
    _buffer: Option<smithay::backend::renderer::utils::Buffer>,
}
type Cache<T> = HashMap<(Uuid, String, Option<u64>), Vec<Snapshot<T>>>;
thread_local! { static CACHES: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new()); }
thread_local! { static ORDERS: RefCell<HashMap<(String, Option<u64>), Vec<Uuid>>> = RefCell::new(HashMap::new()); }

/// Preserve an outgoing drawable's last slot relative to surviving neighbours,
/// including world UI stacking barriers. The order remains pane-local.
pub fn order(output: &str, pane: Option<u64>, current: &[Uuid]) -> Vec<Uuid> {
    let exits = crate::comp::lifecycle_motion::exits(crate::comp::animation::frame_time());
    ORDERS.with(|orders| {
        let mut orders = orders.borrow_mut();
        let previous = orders.entry((output.to_owned(), pane)).or_default();
        let mut merged = current.to_vec();
        for (index, uuid) in previous.iter().enumerate().rev() {
            if exits.contains_key(uuid) && !merged.contains(uuid) {
                let next = previous[index + 1..].iter().find_map(|next| merged.iter().position(|id| id == next));
                merged.insert(next.unwrap_or(merged.len()), *uuid);
            }
        }
        *previous = merged.clone();
        merged
    })
}

pub fn exiting(uuid: Uuid) -> bool {
    crate::comp::lifecycle_motion::exits(crate::comp::animation::frame_time()).contains_key(&uuid)
}

pub fn prune<R: Renderer>() where R::TextureId: 'static {
    let keep = crate::comp::lifecycle_motion::retained_windows();
    let enabled = crate::comp::lifecycle_motion::retain_snapshots();
    CACHES.with(|caches| {
        if let Some(cache) = caches.borrow_mut().get_mut(&TypeId::of::<R::TextureId>())
            .and_then(|cache| cache.downcast_mut::<Cache<R::TextureId>>()) {
            cache.retain(|(uuid, _, _), _| enabled && keep.contains(uuid));
        }
    });
}

pub fn retain<R>(
    renderer: &R,
    uuid: Uuid,
    output: &str,
    pane: Option<u64>,
    anchor: Point<i32, Physical>,
    logical_scale: f64,
    elements: &mut [Element<R>],
) where
    R: Renderer + ImportAll + ImportMem + SceneDispatch,
    R::TextureId: Texture + Clone + Send + 'static,
{
    if !crate::comp::lifecycle_motion::retain_snapshots() {
        return;
    }
    let mut snapshots = Vec::new();
    for element in elements {
        let geometry = element.geometry(Scale::from(1.0));
        let src = element.src();
        let transform = element.transform();
        let alpha = element.alpha();
        let raw = element.raw_mut();
        let clip = match &*raw {
            RawElement::WindowFitClipped(clipped) => Some(clipped.snapshot_clip()),
            _ => None,
        };
        let buffer = match &mut *raw {
            RawElement::Window(e) => Some(e.inner.inner.buffer().clone()),
            RawElement::WindowFit(e) => Some(e.inner.element_mut().element_mut().element_mut().inner.buffer().clone()),
            RawElement::WindowFitClipped(e) => Some(e.inner_mut().inner.element_mut().element_mut().element_mut().inner.buffer().clone()),
            _ => None,
        };
        let surface_solid = matches!(raw, RawElement::Window(_) | RawElement::WindowFit(_) | RawElement::WindowFitClipped(_));
        let (texture, color) = match raw {
            RawElement::Chrome(decor::render::ChromeElement::Piece(piece)) => {
                (Some(piece.texture().clone()), None)
            }
            RawElement::Chrome(decor::render::ChromeElement::Solid(solid))
            | RawElement::SolidBox(solid) => (None, Some(solid.color())),
            RawElement::Window(e) => match e.inner.inner.texture() {
                WaylandSurfaceTexture::Texture(texture) => (Some(texture.clone()), None),
                WaylandSurfaceTexture::SolidColor(color) => (None, Some(*color)),
            },
            RawElement::WindowFit(e) => match e
                .inner
                .element_mut()
                .element_mut()
                .element_mut()
                .inner
                .texture()
            {
                WaylandSurfaceTexture::Texture(texture) => (Some(texture.clone()), None),
                WaylandSurfaceTexture::SolidColor(color) => (None, Some(*color)),
            },
            RawElement::WindowFitClipped(e) => match e
                .inner_mut()
                .inner
                .element_mut()
                .element_mut()
                .element_mut()
                .inner
                .texture()
            {
                WaylandSurfaceTexture::Texture(texture) => (Some(texture.clone()), None),
                WaylandSurfaceTexture::SolidColor(color) => (None, Some(*color)),
            },
            _ => continue,
        };
        let color = color.map(|color| {
            if surface_solid {
                Color32F::from([color.r() * alpha, color.g() * alpha,
                    color.b() * alpha, color.a() * alpha])
            } else { color }
        });
        snapshots.push(Snapshot {
            id: Id::new(),
            texture,
            color,
            context: renderer.context_id(),
            geometry,
            src,
            transform,
            alpha,
            anchor,
            logical_scale,
            clip,
            _buffer: buffer,
        });
    }
    CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let cache = caches
            .entry(TypeId::of::<R::TextureId>())
            .or_insert_with(|| Box::new(Cache::<R::TextureId>::new()))
            .downcast_mut::<Cache<R::TextureId>>()
            .unwrap();
        cache.insert((uuid, output.to_owned(), pane), snapshots);
    });
}

pub fn elements<R>(output: &str, pane: Option<u64>, uuid: Uuid) -> Vec<Ghost<R>>
where
    R: Renderer + ImportAll + ImportMem + SceneDispatch,
    R::TextureId: Texture + Clone + Send + 'static,
{
    let exits = crate::comp::lifecycle_motion::exits(crate::comp::animation::frame_time());
    let keep = crate::comp::lifecycle_motion::retained_windows();
    CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let Some(cache) = caches
            .get_mut(&TypeId::of::<R::TextureId>())
            .and_then(|cache| cache.downcast_mut::<Cache<R::TextureId>>())
        else {
            return Vec::new();
        };
        if !crate::comp::lifecycle_motion::retain_snapshots() {
            cache.clear();
            return Vec::new();
        }
        cache.retain(|(uuid, _, _), _| keep.contains(uuid));
        let mut elements = Vec::new();
        if let (Some(snapshots), Some(values)) = (cache.get(&(uuid, output.to_owned(), pane)), exits.get(&uuid)) {
            for snapshot in snapshots {
                if snapshot.texture.is_some() {
                    let texture = draw(snapshot, values);
                    elements.push(if let Some((program, clip)) = &snapshot.clip {
                        let (scale, offset) = transform_parts(snapshot, values);
                        Ghost::ClippedTexture(decor::clip::Clipped::new(texture, program.clone(), *clip).rescaled(scale, offset))
                    } else { Ghost::Texture(texture) });
                }
                if let Some(color) = snapshot.color {
                    let alpha = values.get(Property::Opacity).clamp(0.0, 1.0) as f32;
                    let color = Color32F::from([
                        color.r() * alpha,
                        color.g() * alpha,
                        color.b() * alpha,
                        color.a() * alpha,
                    ]);
                    let solid = SolidColorRenderElement::new(
                        snapshot.id.clone(),
                        transformed(snapshot, values),
                        CommitCounter::default(),
                        color,
                        Kind::Unspecified,
                    );
                    elements.push(if let Some((program, clip)) = &snapshot.clip {
                        let (scale, offset) = transform_parts(snapshot, values);
                        Ghost::ClippedSolid(decor::clip::Clipped::new(solid, program.clone(), *clip).rescaled(scale, offset))
                    } else { Ghost::Solid(solid) });
                }
            }
        }
        elements
    })
}

fn transformed<T: Texture>(snapshot: &Snapshot<T>, values: &Values) -> Rectangle<i32, Physical> {
    let (scale, offset) = transform_parts(snapshot, values);
    let geometry = snapshot.geometry;
    let x0 = (geometry.loc.x as f64 * scale).round() as i32 + offset.x;
    let y0 = (geometry.loc.y as f64 * scale).round() as i32 + offset.y;
    let x1 = ((geometry.loc.x as f64 + geometry.size.w as f64) * scale).round() as i32 + offset.x;
    let y1 = ((geometry.loc.y as f64 + geometry.size.h as f64) * scale).round() as i32 + offset.y;
    Rectangle::new((x0,y0).into(), ((x1-x0).max(0),(y1-y0).max(0)).into())
}

fn transform_parts<T: Texture>(snapshot: &Snapshot<T>, values: &Values) -> (f64, Point<i32, Physical>) {
    let scale = values.get(Property::Scale).clamp(0.0, 64.0);
    let offset = Point::from((
        ((1.0-scale) * snapshot.anchor.x as f64 + values.get(Property::Tx)*snapshot.logical_scale).round() as i32,
        ((1.0-scale) * snapshot.anchor.y as f64 + values.get(Property::Ty)*snapshot.logical_scale).round() as i32,
    ));
    (scale, offset)
}

fn draw<T: Texture + Clone>(snapshot: &Snapshot<T>, values: &Values) -> ElementWindowSurface<TextureRenderElement<T>> {
    let rect = transformed(snapshot, values);
    let texture = snapshot.texture.as_ref().unwrap();
    let inner = TextureRenderElement::from_static_texture(
        snapshot.id.clone(),
        snapshot.context.clone(),
        rect.loc.to_f64(),
        texture.clone(),
        1,
        snapshot.transform,
        Some(snapshot.alpha * values.get(Property::Opacity).clamp(0.0, 1.0) as f32),
        Some(
            snapshot
                .src
                .to_logical(1.0, snapshot.transform, &texture.size().to_f64()),
        ),
        Some((rect.size.w, rect.size.h).into()),
        None,
        Kind::Unspecified,
    );
    ElementWindowSurface {
        inner,
        zoom: 1.0,
        bundle_owned: true,
    }
}
