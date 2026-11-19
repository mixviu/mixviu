// SPDX-License-Identifier: MIT OR Apache-2.0
//! Last imported client textures, independent of protocol/window lifetimes.
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use motion::{Property, Values};
use smithay::backend::renderer::element::{Element as _, Id, Kind};
use smithay::backend::renderer::element::surface::WaylandSurfaceTexture;
use smithay::backend::renderer::element::texture::TextureRenderElement;
use smithay::backend::renderer::{ContextId, ImportAll, ImportMem, Renderer, Texture};
use smithay::utils::{Buffer, Logical, Physical, Rectangle, Scale, Transform};
use dispatcher::frame::SceneDispatch;
use uuid::Uuid;
use super::element::{Element, ElementWindowSurface, RawElement};

pub type Ghost<T> = ElementWindowSurface<TextureRenderElement<T>>;

struct Snapshot<T: Texture> {
    id: Id, texture: T, context: ContextId<T>,
    geometry: Rectangle<i32, Physical>, src: Rectangle<f64, Buffer>,
    transform: Transform, alpha: f32,
}
type Cache<T> = HashMap<(Uuid, String), Vec<Snapshot<T>>>;
thread_local! { static CACHES: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new()); }

pub fn retain<R>(renderer: &R, uuid: Uuid, output: &str, elements: &mut [Element<R>])
where R: Renderer + ImportAll + ImportMem + SceneDispatch,
      R::TextureId: Texture + Clone + Send + 'static {
    if !crate::comp::lifecycle_motion::retain_snapshots() { return; }
    let mut snapshots = Vec::new();
    for element in elements {
        let geometry = element.geometry(Scale::from(1.0));
        let src = element.src();
        let transform = element.transform();
        let alpha = element.alpha();
        let surface = match element.raw_mut() {
            RawElement::Window(e) => &mut e.inner.inner,
            RawElement::WindowFit(e) => &mut e.inner.element_mut().element_mut().element_mut().inner,
            RawElement::WindowFitClipped(e) => &mut e.inner_mut().inner.element_mut().element_mut().element_mut().inner,
            _ => continue,
        };
        let WaylandSurfaceTexture::Texture(texture) = surface.texture() else { continue };
        snapshots.push(Snapshot { id: Id::new(), texture: texture.clone(), context: renderer.context_id(),
            geometry, src, transform, alpha });
    }
    CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let cache = caches.entry(TypeId::of::<R::TextureId>()).or_insert_with(|| Box::new(Cache::<R::TextureId>::new()))
            .downcast_mut::<Cache<R::TextureId>>().unwrap();
        cache.insert((uuid, output.to_owned()), snapshots);
    });
}

pub fn elements<R>(output: &str) -> Vec<Ghost<R::TextureId>>
where R: Renderer + ImportAll + ImportMem + SceneDispatch,
      R::TextureId: Texture + Clone + Send + 'static {
    let exits = crate::comp::lifecycle_motion::exits(crate::comp::animation::frame_time());
    CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let Some(cache) = caches.get_mut(&TypeId::of::<R::TextureId>())
            .and_then(|cache| cache.downcast_mut::<Cache<R::TextureId>>()) else { return Vec::new() };
        let mut elements = Vec::new();
        for ((uuid, key), snapshots) in cache.iter() {
            if key != output { continue; }
            let Some(values) = exits.get(uuid) else { continue; };
            for snapshot in snapshots {
                elements.push(draw(snapshot, values));
            }
        }
        elements
    })
}

fn draw<T: Texture + Clone>(snapshot: &Snapshot<T>, values: &Values) -> Ghost<T> {
    let scale = values.get(Property::Scale).clamp(0.0, 64.0);
    let geometry = snapshot.geometry;
    let x = geometry.loc.x as f64 + geometry.size.w as f64 * (1.0 - scale) / 2.0 + values.get(Property::Tx);
    let y = geometry.loc.y as f64 + geometry.size.h as f64 * (1.0 - scale) / 2.0 + values.get(Property::Ty);
    let size = ((geometry.size.w as f64 * scale).round() as i32,
                (geometry.size.h as f64 * scale).round() as i32);
    let inner = TextureRenderElement::from_static_texture(snapshot.id.clone(), snapshot.context.clone(),
        (x, y), snapshot.texture.clone(), 1, snapshot.transform,
        Some(snapshot.alpha * values.get(Property::Opacity).clamp(0.0, 1.0) as f32),
        Some(snapshot.src.to_logical(1.0, snapshot.transform, &snapshot.texture.size().to_f64())),
        Some(size.into()), None, Kind::Unspecified);
    ElementWindowSurface { inner, zoom: 1.0, bundle_owned: true }
}
