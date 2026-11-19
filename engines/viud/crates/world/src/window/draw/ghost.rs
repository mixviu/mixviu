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

pub type Ghost<T> = ElementWindowSurface<TextureRenderElement<T>>;

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
}
type Cache<T> = HashMap<(Uuid, String), Vec<Snapshot<T>>>;
thread_local! { static CACHES: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new()); }

pub fn retain<R>(
    renderer: &R,
    uuid: Uuid,
    output: &str,
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
        });
    }
    CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let cache = caches
            .entry(TypeId::of::<R::TextureId>())
            .or_insert_with(|| Box::new(Cache::<R::TextureId>::new()))
            .downcast_mut::<Cache<R::TextureId>>()
            .unwrap();
        cache.insert((uuid, output.to_owned()), snapshots);
    });
}

pub fn elements<R>(output: &str) -> (Vec<Ghost<R::TextureId>>, Vec<SolidColorRenderElement>)
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
            return (Vec::new(), Vec::new());
        };
        if !crate::comp::lifecycle_motion::retain_snapshots() {
            cache.clear();
            return (Vec::new(), Vec::new());
        }
        cache.retain(|(uuid, _), _| keep.contains(uuid));
        let mut elements = Vec::new();
        let mut solids = Vec::new();
        for ((uuid, key), snapshots) in cache.iter() {
            if key != output {
                continue;
            }
            let Some(values) = exits.get(uuid) else {
                continue;
            };
            for snapshot in snapshots {
                if snapshot.texture.is_some() {
                    elements.push(draw(snapshot, values));
                }
                if let Some(color) = snapshot.color {
                    let alpha = values.get(Property::Opacity).clamp(0.0, 1.0) as f32;
                    let color = Color32F::from([
                        color.r() * alpha,
                        color.g() * alpha,
                        color.b() * alpha,
                        color.a() * alpha,
                    ]);
                    solids.push(SolidColorRenderElement::new(
                        snapshot.id.clone(),
                        transformed(snapshot, values),
                        CommitCounter::default(),
                        color,
                        Kind::Unspecified,
                    ));
                }
            }
        }
        (elements, solids)
    })
}

fn transformed<T: Texture>(snapshot: &Snapshot<T>, values: &Values) -> Rectangle<i32, Physical> {
    let scale = values.get(Property::Scale).clamp(0.0, 64.0);
    let geometry = snapshot.geometry;
    let x = snapshot.anchor.x as f64
        + (geometry.loc.x - snapshot.anchor.x) as f64 * scale
        + values.get(Property::Tx) * snapshot.logical_scale;
    let y = snapshot.anchor.y as f64
        + (geometry.loc.y - snapshot.anchor.y) as f64 * scale
        + values.get(Property::Ty) * snapshot.logical_scale;
    let size = (
        (geometry.size.w as f64 * scale).round() as i32,
        (geometry.size.h as f64 * scale).round() as i32,
    );
    Rectangle::new(((x.round() as i32), (y.round() as i32)).into(), size.into())
}

fn draw<T: Texture + Clone>(snapshot: &Snapshot<T>, values: &Values) -> Ghost<T> {
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
