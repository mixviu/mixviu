//! Scene vector paint: no brand geometry or application timing lives here.
use egui::{Color32, Pos2, Shape, Stroke, Vec2};
use motion::Property;
use scene::{Node, drawing::Primitive};

pub fn show(ui: &mut egui::Ui, id: &str, node: &Node) {
    let n = |key: &str, default| {
        node.ports
            .get(key)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(default) as f32
    };
    let size = Vec2::new(n("w", 280.), n("h", 280.));
    if !size.is_finite() || size.x <= 0. || size.y <= 0. || size.x > 10000. || size.y > 10000. {
        return;
    }
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let time = node
        .ports
        .get("sample_time")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(10.);
    let Some(raw) = node.ports.get("primitives") else {
        return;
    };
    if scene::drawing::validate(raw).is_err() || !time.is_finite() || time.abs() > 3600. {
        return;
    }
    let Ok(primitives) = serde_json::from_value::<Vec<Primitive>>(raw.clone()) else {
        return;
    };
    let legacy = node
        .ports
        .get("motion_profile")
        .and_then(serde_json::Value::as_str)
        == Some("legacy-sting");
    let mut shapes = Vec::new();
    let mut shadow_shapes = Vec::new();
    let mut float_offset = None;
    let mut sampled = Vec::new();
    for p in primitives {
        let mut scale = 1.;
        let mut sy = 1.;
        let mut sx = 1.;
        let mut dy = 0.;
        let mut opacity = 1.;
        let mut fraction = 1.;
        for e in &p.effects {
            // The host selects a profile; authored primitive names never select
            // the internal legacy namespace, even on a sting drawing.
            if e.preset.starts_with("legacy-") {
                return;
            }
            let name = if legacy {
                format!("legacy-{}", e.preset)
            } else {
                e.preset.clone()
            };
            let Ok(preset) = motion::scene::paint_preset(&name) else {
                return;
            };
            let v = preset.sample(time - e.delay);
            scale *= v.get(Property::Scale) as f32;
            sx *= v.get(Property::ScaleX) as f32;
            sy *= v.get(Property::ScaleY) as f32;
            dy += v.get(Property::Ty) as f32;
            opacity *= v.get(Property::Opacity) as f32;
            if e.preset == "draw" {
                let length = p.radius[0] * p.sweep.to_radians().abs();
                if !length.is_finite() || length <= 0. {
                    return;
                }
                fraction *= (p.dash_length.unwrap_or(length)
                    * (1. - v.get(Property::DashOffset) as f32 / 1000.)
                    / length)
                    .clamp(0., 1.);
            }
        }
        let centre = rect.min + Vec2::new(p.centre[0], p.centre[1] + dy);
        let radius = Vec2::new(p.radius[0] * scale * sx, p.radius[1] * scale * sy);
        let Some(rgb) = parse_colour(&p.colour) else {
            return;
        };
        let opacity = opacity.clamp(0., 1.);
        sampled.push([scale, sx, sy, opacity, fraction, dy]);
        let colour = Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
            .gamma_multiply(opacity.clamp(0., 1.));
        let shape = if p.kind == "arc" {
            if fraction <= 0. {
                continue;
            }
            let sweep = p.sweep.to_radians() * fraction;
            let points = (0..=256)
                .map(|i| {
                    let a = p.start.to_radians() + sweep * i as f32 / 256.;
                    centre + Vec2::new(radius.x * a.cos(), radius.y * a.sin())
                })
                .collect::<Vec<Pos2>>();
            let stroke = Stroke::new(p.width * scale, colour);
            Shape::Vec(vec![
                Shape::circle_filled(points[0], stroke.width / 2., colour),
                Shape::circle_filled(points[256], stroke.width / 2., colour),
                Shape::line(points, stroke),
            ])
        } else if p.kind == "bar" {
            let r = egui::Rect::from_center_size(centre, radius * 2.);
            let end = p
                .end_colour
                .as_ref()
                .and_then(|c| parse_colour(c))
                .unwrap_or(rgb);
            let end = Color32::from_rgb((end >> 16) as u8, (end >> 8) as u8, end as u8)
                .gamma_multiply(opacity);
            let mut mesh = egui::Mesh::default();
            // Rounded horizontal gradient, tessellated as narrow strips.
            let strips = r.width().ceil().clamp(1., 2048.) as usize;
            for i in 0..strips {
                let x = r.left() + r.width() * i as f32 / strips as f32;
                let mid = x + r.width() / strips as f32 / 2.;
                let edge = (mid - r.left()).min(r.right() - mid).min(radius.y);
                let h = (radius.y * radius.y - (radius.y - edge).powi(2))
                    .max(0.)
                    .sqrt();
                let t = (i as f32 + 0.5) / strips as f32;
                let a = colour.to_array();
                let b = end.to_array();
                let c = Color32::from_rgba_premultiplied(
                    (a[0] as f32 * (1. - t) + b[0] as f32 * t) as u8,
                    (a[1] as f32 * (1. - t) + b[1] as f32 * t) as u8,
                    (a[2] as f32 * (1. - t) + b[2] as f32 * t) as u8,
                    colour.a(),
                );
                mesh.add_colored_rect(
                    egui::Rect::from_min_max(
                        egui::pos2(x, centre.y - h),
                        egui::pos2(x + r.width() / strips as f32, centre.y + h),
                    ),
                    c,
                );
            }
            Shape::mesh(mesh)
        } else {
            Shape::ellipse_filled(centre, radius, colour)
        };
        // A common float is a translation of the whole silhouette. Cache
        // its unshifted alpha and move the shadow quad, not the raster.
        let common = dy;
        if float_offset.is_none() {
            float_offset = Some(common)
        }
        if float_offset.is_some_and(|v| v != common) {
            float_offset = Some(f32::NAN)
        }
        if node.ports.contains_key("shadow") {
            shadow_shapes.push(shape.clone());
        }
        shapes.push(shape);
    }
    if node.ports.contains_key("shadow") {
        let dy = float_offset.filter(|n| n.is_finite()).unwrap_or(0.);
        for shape in &mut shadow_shapes {
            shape.translate(Vec2::new(0., -dy));
        }
        if float_offset.is_some_and(|n| n.is_finite()) {
            for sample in &mut sampled {
                sample[5] = 0.;
            }
        }
        #[cfg(test)]
        let shadow_start = std::time::Instant::now();
        paint_shadow(
            ui,
            id,
            rect,
            node.ports.get("shadow"),
            &shadow_shapes,
            format!(
                "{:?}:{raw}:{sampled:?}:{}",
                rect.size(),
                node.ports.get("shadow").unwrap_or(&serde_json::Value::Null)
            ),
            Vec2::new(0., dy),
        );
        #[cfg(test)]
        {
            shadows(ui.ctx()).lock().unwrap().last_shadow_ms =
                shadow_start.elapsed().as_secs_f64() * 1000.;
        }
    }
    ui.painter().extend(shapes);
}
fn parse_colour(text: &str) -> Option<u32> {
    let hex = text.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    u32::from_str_radix(hex, 16).ok()
}

#[derive(Clone)]
struct ShadowCache {
    signature: String,
    texture: egui::TextureHandle,
    pixels: usize,
}
#[derive(Default)]
struct Shadows {
    entries: std::collections::VecDeque<(egui::Id, ShadowCache)>,
    pass: u64,
    spent: usize,
    hits: usize,
    rasters: usize,
    blurs: usize,
    worker_rasters: usize,
    worker_blurs: usize,
    pending: Option<PendingShadow>,
    waiting: std::collections::VecDeque<(egui::Id, u64)>,
    spawn_backoff: std::time::Duration,
    spawn_retry_at: Option<std::time::Instant>,
    #[cfg(test)]
    synchronous_updates: bool,
    #[cfg(test)]
    last_shadow_ms: f64,
    #[cfg(test)]
    fail_spawn: bool,
}
struct PendingShadow {
    key: egui::Id,
    base_signature: String,
    texture: egui::TextureId,
    signature: String,
    image: std::sync::mpsc::Receiver<Option<egui::ColorImage>>,
    ready: Option<egui::ColorImage>,
    last_seen: u64,
}
const SHADOW_PIXELS: usize = 8 * 1024 * 1024;

fn receive_shadow(c: &mut Shadows, key: egui::Id, signature: &str, pass: u64, ctx: &egui::Context) {
    let Some(mut pending) = c.pending.take() else {
        return;
    };
    if pending.key == key {
        pending.last_seen = pass;
    }
    if pending.ready.is_none() {
        match pending.image.try_recv() {
            Ok(Some(image)) => pending.ready = Some(image),
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                c.pending = Some(pending);
                return;
            }
            _ => return,
        }
    }
    let Some(index) = c.entries.iter().position(|(id, cache)| {
        *id == pending.key
            && cache.signature == pending.base_signature
            && cache.texture.id() == pending.texture
    }) else {
        return;
    };
    // Only the source's own paint knows its current request. Do not install a
    // sibling's result speculatively, even when it completes mid-pass.
    if pending.key != key {
        if pending.last_seen.saturating_add(1) >= pass {
            c.pending = Some(pending);
            ctx.request_repaint();
        }
        return;
    }
    if c.entries[index].1.signature == signature {
        return;
    }
    let image = pending.ready.take().unwrap();
    let Some(pixels) = image.size[0].checked_mul(image.size[1]) else {
        return;
    };
    if pixels > SHADOW_PIXELS {
        return;
    }
    let (_, mut cache) = c.entries.remove(index).unwrap();
    cache.pixels = pixels;
    cache.texture.set(image, Default::default());
    cache.signature = pending.signature;
    while c.entries.iter().map(|(_, v)| v.pixels).sum::<usize>() + pixels > SHADOW_PIXELS {
        c.entries.pop_front();
    }
    c.entries.push_back((key, cache));
}
fn shadows(ctx: &egui::Context) -> std::sync::Arc<std::sync::Mutex<Shadows>> {
    let key = egui::Id::unique("scene-shadow-budget-v1");
    ctx.data_mut(|d| {
        if let Some(c) = d.get_temp::<std::sync::Arc<std::sync::Mutex<Shadows>>>(key) {
            return c;
        }
        let c = std::sync::Arc::new(std::sync::Mutex::new(Shadows::default()));
        d.insert_temp(key, c.clone());
        c
    })
}
fn paint_shadow(
    ui: &egui::Ui,
    id: &str,
    rect: egui::Rect,
    spec: Option<&serde_json::Value>,
    shapes: &[Shape],
    signature: String,
    translation: Vec2,
) {
    let Some(spec) = spec else { return };
    if scene::drawing::validate_shadow(spec).is_err() {
        return;
    }
    let Ok(shadow) = serde_json::from_value::<scene::drawing::Shadow>(spec.clone()) else {
        return;
    };
    let pad = (shadow.blur * 3.).ceil();
    let width = (rect.width() + pad * 2.).ceil() as u32;
    let height = (rect.height() + pad * 2.).ceil() as u32;
    if width == 0 || height == 0 || width > 2048 || height > 2048 {
        return;
    }
    let origin = rect.min - Vec2::splat(pad);
    let key = ui.scope_id().with(("drawing-shadow", id));
    let draw = |texture: &egui::TextureHandle| {
        let pos = origin + Vec2::new(shadow.offset[0], shadow.offset[1]) + translation;
        ui.painter().image(
            texture.id(),
            egui::Rect::from_min_size(pos, Vec2::new(width as f32, height as f32)),
            egui::Rect::from_min_max(Pos2::ZERO, egui::pos2(1., 1.)),
            Color32::WHITE,
        );
    };
    let shared = shadows(ui.ctx());
    let old = {
        let mut c = shared.lock().unwrap_or_else(|e| e.into_inner());
        let pass = ui.ctx().cumulative_pass_nr();
        if c.pass != pass {
            c.pass = pass;
            c.spent = 0;
        }
        let pixels = width as usize * height as usize;
        if c.spent + pixels > SHADOW_PIXELS {
            return;
        }
        c.spent += pixels;
        // At most one raster/blur is in flight per context. Receive without
        // waiting; the previous texture remains usable throughout animation.
        receive_shadow(&mut c, key, &signature, pass, ui.ctx());
        let found = c.entries.iter().position(|(id, _)| *id == key);
        found.and_then(|i| c.entries.remove(i)).map(|(_, v)| {
            c.hits += 1;
            v
        })
    };
    #[cfg(test)]
    let old = old.filter(|cache| {
        !(cache.signature != signature && shared.lock().unwrap().synchronous_updates)
    });
    if let Some(cache) = old {
        if cache.signature != signature {
            let mut c = shared.lock().unwrap_or_else(|e| e.into_inner());
            let pass = ui.ctx().cumulative_pass_nr();
            // Round-robin dirty sources, without storing obsolete frame meshes.
            // A source that disappears cannot hold the worker indefinitely.
            c.waiting.retain(|(_, seen)| seen.saturating_add(1) >= pass);
            if !c.pending.as_ref().is_some_and(|job| job.key == key) {
                if let Some((_, seen)) = c.waiting.iter_mut().find(|(id, _)| *id == key) {
                    *seen = pass;
                } else {
                    if c.waiting.len() >= 8 {
                        c.waiting.pop_front();
                    }
                    c.waiting.push_back((key, pass));
                }
            }
            let retry_delay = c.spawn_retry_at.map_or(std::time::Duration::ZERO, |at| {
                at.saturating_duration_since(std::time::Instant::now())
            });
            if c.pending.is_none() && !retry_delay.is_zero() {
                ui.ctx().request_repaint_after(retry_delay);
            } else if c.pending.is_none() && c.waiting.front().is_some_and(|(id, _)| *id == key) {
                c.waiting.pop_front();
                let (input_tx, input_rx) = std::sync::mpsc::sync_channel(1);
                let (tx, rx) = std::sync::mpsc::sync_channel(1);
                let ctx = ui.ctx().clone();
                let shadow = shadow.clone();
                let blurred = shadow.blur > 0.;
                let task = move || {
                    if let Ok(input) = input_rx.recv() {
                        let image = raster_shadow(width, height, origin, &shadow, input);
                        // Send without taking the shadow cache lock: completion
                        // may be awaited while the test owns that lock.
                        let _ = tx.send(image);
                        ctx.request_repaint();
                    }
                };
                #[cfg(test)]
                let fail_spawn = c.fail_spawn;
                #[cfg(not(test))]
                let fail_spawn = false;
                let spawned = if fail_spawn {
                    Err(std::io::Error::other("injected spawn failure"))
                } else {
                    std::thread::Builder::new()
                        .name("scene-shadow".into())
                        .spawn(task)
                };
                if spawned.is_ok() {
                    c.spawn_backoff = std::time::Duration::ZERO;
                    c.spawn_retry_at = None;
                    let _ = input_tx.send(shadow_input(ui.ctx(), shapes));
                    c.worker_rasters += 1;
                    c.worker_blurs += usize::from(blurred);
                    c.pending = Some(PendingShadow {
                        key,
                        base_signature: cache.signature.clone(),
                        texture: cache.texture.id(),
                        signature,
                        image: rx,
                        ready: None,
                        last_seen: pass,
                    });
                } else {
                    c.waiting.push_back((key, pass));
                    c.spawn_backoff = if c.spawn_backoff.is_zero() {
                        std::time::Duration::from_millis(100)
                    } else {
                        (c.spawn_backoff * 2).min(std::time::Duration::from_secs(2))
                    };
                    c.spawn_retry_at = Some(std::time::Instant::now() + c.spawn_backoff);
                    ui.ctx().request_repaint_after(c.spawn_backoff);
                }
            } else if c.pending.is_none() {
                ui.ctx().request_repaint();
            }
        } else {
            shared
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .waiting
                .retain(|(id, _)| *id != key);
        }
        draw(&cache.texture);
        shared
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entries
            .push_back((key, cache));
        return;
    }
    let input = shadow_input(ui.ctx(), shapes);
    let Some(image) = raster_shadow(width, height, origin, &shadow, input) else {
        return;
    };
    {
        let mut c = shared.lock().unwrap_or_else(|e| e.into_inner());
        c.rasters += 1;
        c.blurs += usize::from(shadow.blur > 0.);
    }
    let texture =
        ui.ctx()
            .load_texture(format!("drawing-shadow-{key:?}"), image, Default::default());
    draw(&texture);
    let mut c = shared.lock().unwrap_or_else(|e| e.into_inner());
    c.entries.retain(|(id, _)| *id != key);
    let pixels = width as usize * height as usize;
    while c.entries.len() >= 8
        || c.entries.iter().map(|(_, v)| v.pixels).sum::<usize>() + pixels > SHADOW_PIXELS
    {
        c.entries.pop_front();
    }
    c.entries.push_back((
        key,
        ShadowCache {
            signature,
            texture,
            pixels,
        },
    ));
}

fn shadow_input(ctx: &egui::Context, shapes: &[Shape]) -> Vec<egui::ClippedPrimitive> {
    let input = shapes
        .iter()
        .cloned()
        .map(|shape| egui::epaint::ClippedShape {
            clip_rect: egui::Rect::EVERYTHING,
            shape,
            transform_after_tessellation: Default::default(),
        })
        .collect();
    ctx.tessellate(input, 1.)
}

// The exact same raster and blur serve cold/static shadows and worker updates.
// No Context lock, texture upload or UI access occurs on the worker.
fn raster_shadow(
    width: u32,
    height: u32,
    origin: Pos2,
    shadow: &scene::drawing::Shadow,
    input: Vec<egui::ClippedPrimitive>,
) -> Option<egui::ColorImage> {
    let mut mask = tiny_skia::Pixmap::new(width, height)?;
    for primitive in input {
        let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive else {
            continue;
        };
        for triangle in mesh.indices.chunks_exact(3) {
            let mut path = tiny_skia::PathBuilder::new();
            let mut alpha = 0.;
            for (i, index) in triangle.iter().enumerate() {
                let v = mesh.vertices[*index as usize];
                let p = v.pos - origin;
                if i == 0 {
                    path.move_to(p.x, p.y)
                } else {
                    path.line_to(p.x, p.y)
                }
                alpha += v.color.a() as f32 / 255. / 3.;
            }
            path.close();
            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(255, 255, 255, (alpha * 255.).round() as u8);
            if let Some(path) = path.finish() {
                mask.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    tiny_skia::Transform::identity(),
                    None,
                );
            }
        }
    }
    let image = image::RgbaImage::from_raw(width, height, mask.data().to_vec())?;
    let mut blurred = if shadow.blur > 0. {
        image::imageops::fast_blur(&image, shadow.blur)
    } else {
        image
    };
    for pixel in blurred.pixels_mut() {
        *pixel = image::Rgba([
            shadow.colour[0] as u8,
            shadow.colour[1] as u8,
            shadow.colour[2] as u8,
            (pixel[3] as f32 * shadow.colour[3] / 255.).round() as u8,
        ]);
    }
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        blurred.as_raw(),
    ))
}

#[cfg(test)]
mod tests {
    fn pending_for(
        store: &Shadows,
        index: usize,
        signature: &str,
        image: std::sync::mpsc::Receiver<Option<egui::ColorImage>>,
    ) -> PendingShadow {
        let (key, cache) = &store.entries[index];
        PendingShadow {
            key: *key,
            base_signature: cache.signature.clone(),
            texture: cache.texture.id(),
            signature: signature.into(),
            image,
            ready: None,
            last_seen: store.pass,
        }
    }
    fn sting_node() -> scene::Node {
        let ports: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/sting-shadow.json")).unwrap();
        serde_json::from_value(serde_json::json!({
            "family":"drawing", "line":1, "is_template":false, "ports":ports
        }))
        .unwrap()
    }

    fn shadow_frame(
        ctx: &egui::Context,
        id: &str,
        signature: &str,
        scale: f32,
    ) -> egui::FullOutput {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            paint_shadow(
                ui,
                id,
                egui::Rect::from_min_size(Pos2::ZERO, Vec2::splat(32. * scale)),
                Some(&serde_json::json!({"colour":[0,0,0,100],"offset":[0,0],"blur":2})),
                &[Shape::circle_filled(
                    egui::pos2(16., 16.),
                    8. * scale,
                    Color32::WHITE,
                )],
                signature.into(),
                Vec2::ZERO,
            );
        })
    }

    fn fake_update(
        ctx: &egui::Context,
        signature: &str,
    ) -> std::sync::mpsc::SyncSender<Option<egui::ColorImage>> {
        let shared = shadows(ctx);
        let mut store = shared.lock().unwrap();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        store.pending = Some(pending_for(&store, 0, signature, receiver));
        sender
    }

    fn tiny_image() -> egui::ColorImage {
        egui::ColorImage::from_rgba_unmultiplied([1, 1], &[255, 0, 0, 255])
    }

    #[test]
    fn evicted_and_recreated_shadow_rejects_the_old_worker_result() {
        let ctx = egui::Context::default();
        shadow_frame(&ctx, "root", "a", 1.).drop_without_applying_deltas();
        let sender = fake_update(&ctx, "b");
        let old_texture = shadows(&ctx).lock().unwrap().entries[0].1.texture.id();
        for i in 0..8 {
            shadow_frame(&ctx, &format!("sibling-{i}"), "sibling", 1.)
                .drop_without_applying_deltas();
        }
        // Recreate even the same signature: the texture identity guards ABA.
        shadow_frame(&ctx, "root", "a", 1.).drop_without_applying_deltas();
        let shared = shadows(&ctx);
        let texture = shared
            .lock()
            .unwrap()
            .entries
            .back()
            .unwrap()
            .1
            .texture
            .id();
        assert_ne!(texture, old_texture);
        sender.send(Some(tiny_image())).unwrap();
        let out = shadow_frame(&ctx, "root", "a", 1.);
        assert!(!out.textures_delta.set.contains_key(&texture));
        out.drop_without_applying_deltas();
        let store = shared.lock().unwrap();
        assert_eq!(store.entries.back().unwrap().1.signature, "a");
        assert!(store.pending.is_none());
        assert_eq!(store.worker_rasters, 0);
    }

    #[test]
    fn reverting_to_the_cached_signature_rejects_a_ready_sibling_result() {
        let ctx = egui::Context::default();
        shadow_frame(&ctx, "root", "a", 1.).drop_without_applying_deltas();
        let sender = fake_update(&ctx, "b");
        shadow_frame(&ctx, "root", "b", 1.).drop_without_applying_deltas();
        sender.send(Some(tiny_image())).unwrap();
        shadow_frame(&ctx, "sibling", "c", 1.).drop_without_applying_deltas();
        let shared = shadows(&ctx);
        assert!(
            shared
                .lock()
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .ready
                .is_some()
        );
        let texture = shared.lock().unwrap().entries[0].1.texture.id();
        let out = shadow_frame(&ctx, "root", "a", 1.);
        assert!(!out.textures_delta.set.contains_key(&texture));
        out.drop_without_applying_deltas();
        let store = shared.lock().unwrap();
        assert_eq!(store.entries.back().unwrap().1.signature, "a");
        assert_eq!(store.worker_rasters, 0);
        assert!(store.pending.is_none());
    }

    #[test]
    fn scaled_worker_image_replaces_the_full_texture_at_its_new_size() {
        let ctx = egui::Context::default();
        shadow_frame(&ctx, "root", "base", 1.).drop_without_applying_deltas();
        let shared = shadows(&ctx);
        let texture = shared.lock().unwrap().entries[0].1.texture.id();
        assert_eq!(shared.lock().unwrap().entries[0].1.texture.size(), [44, 44]);
        let sender = fake_update(&ctx, "scaled");
        let image = egui::ColorImage::from_rgba_unmultiplied([76, 76], &vec![255; 76 * 76 * 4]);
        sender.send(Some(image)).unwrap();
        let out = shadow_frame(&ctx, "root", "scaled", 2.);
        let delta = out.textures_delta.set[&texture].last().unwrap();
        assert!(delta.pos.is_none());
        assert_eq!(delta.image.size(), [76, 76]);
        out.drop_without_applying_deltas();
        let store = shared.lock().unwrap();
        assert_eq!(store.entries[0].1.texture.id(), texture);
        assert_eq!(store.entries[0].1.texture.size(), [76, 76]);
        assert_eq!(store.entries[0].1.pixels, 76 * 76);
        assert!(store.pending.is_none());
    }

    #[test]
    fn oversized_worker_result_is_dropped_before_eviction() {
        let ctx = egui::Context::default();
        shadow_frame(&ctx, "root", "a", 1.).drop_without_applying_deltas();
        let sender = fake_update(&ctx, "b");
        let mut image = tiny_image();
        image.size = [SHADOW_PIXELS + 1, 1];
        sender.send(Some(image)).unwrap();
        let shared = shadows(&ctx);
        let mut store = shared.lock().unwrap();
        let key = store.entries[0].0;
        let pass = store.pass;
        receive_shadow(&mut store, key, "b", pass, &ctx);
        assert!(store.pending.is_none());
        assert_eq!(store.entries.len(), 1);
        assert_eq!(store.entries[0].1.signature, "a");
        assert_eq!(store.entries[0].1.texture.size(), [44, 44]);
    }

    #[test]
    fn failed_spawn_backs_off_without_immediate_repaint_and_recovers() {
        let ctx = egui::Context::default();
        for _ in 0..4 {
            shadow_frame(&ctx, "root", "a", 1.).drop_without_applying_deltas();
        }
        let shared = shadows(&ctx);
        shared.lock().unwrap().fail_spawn = true;
        for milliseconds in [100, 200, 400, 800, 1600, 2000, 2000] {
            // Advance the retry deadline without a wall-clock sleep.
            shared.lock().unwrap().spawn_retry_at = None;
            let out = shadow_frame(&ctx, "root", "b", 1.);
            let delay = out.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
            out.drop_without_applying_deltas();
            assert!(!delay.is_zero());
            assert!(delay <= std::time::Duration::from_millis(milliseconds));
            {
                let store = shared.lock().unwrap();
                assert_eq!(
                    store.spawn_backoff,
                    std::time::Duration::from_millis(milliseconds)
                );
                assert!(store.pending.is_none());
                assert_eq!(store.waiting.front().unwrap().0, store.entries[0].0);
                assert_eq!(store.worker_rasters, 0);
            }
            // An unrelated frame during cooldown must neither retry nor ask
            // for an immediate repaint. Hold the deadline in the future so
            // preemption cannot accidentally turn this into an expired retry.
            shared.lock().unwrap().spawn_retry_at =
                Some(std::time::Instant::now() + std::time::Duration::from_secs(60));
            let out = shadow_frame(&ctx, "root", "b", 1.);
            assert!(
                !out.viewport_output[&egui::ViewportId::ROOT]
                    .repaint_delay
                    .is_zero()
            );
            out.drop_without_applying_deltas();
            assert_eq!(
                shared.lock().unwrap().spawn_backoff,
                std::time::Duration::from_millis(milliseconds)
            );
        }
        {
            let mut store = shared.lock().unwrap();
            store.fail_spawn = false;
            store.spawn_retry_at = None;
        }
        shadow_frame(&ctx, "root", "b", 1.).drop_without_applying_deltas();
        let store = shared.lock().unwrap();
        assert_eq!(store.spawn_backoff, std::time::Duration::ZERO);
        assert!(store.spawn_retry_at.is_none());
        assert!(store.pending.is_some());
        assert_eq!(store.worker_rasters, 1);
    }

    #[test]
    fn animated_silhouettes_never_raster_or_blur_again_on_the_frame_path() {
        let ctx = egui::Context::default();
        let mut node = sting_node();
        node.ports
            .insert("motion_profile".into(), serde_json::json!("legacy-sting"));
        for frame in 0..30 {
            node.ports
                .insert("sample_time".into(), serde_json::json!(frame as f64 / 30.));
            ctx.run_ui(egui::RawInput::default(), |ui| show(ui, "sting", &node))
                .drop_without_applying_deltas();
        }
        let store = shadows(&ctx);
        let store = store.lock().unwrap();
        assert_eq!(store.rasters, 1);
        assert_eq!(store.blurs, 1);
        assert!(store.worker_rasters > 0 && store.worker_rasters <= 29);
        assert_eq!(store.worker_blurs, store.worker_rasters);
        assert!(store.entries.iter().map(|(_, v)| v.pixels).sum::<usize>() <= SHADOW_PIXELS);
    }

    #[test]
    fn busy_shadow_worker_coalesces_changed_frames_and_keeps_the_last_texture() {
        let ctx = egui::Context::default();
        let mut node = sting_node();
        ctx.run_ui(egui::RawInput::default(), |ui| show(ui, "sting", &node))
            .drop_without_applying_deltas();
        let shared = shadows(&ctx);
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let texture = {
            let mut store = shared.lock().unwrap();
            let texture = store.entries[0].1.texture.id();
            // A deliberately stalled result, with no sleep or timing race.
            store.pending = Some(pending_for(&store, 0, "busy", receiver));
            texture
        };
        for frame in 0..30 {
            node.ports
                .insert("sample_time".into(), serde_json::json!(frame as f64 / 30.));
            let out = ctx.run_ui(egui::RawInput::default(), |ui| show(ui, "sting", &node));
            assert!(out.shapes.iter().any(|s| matches!(&s.shape,
                Shape::Mesh(mesh) if mesh.texture_id == texture)));
            out.drop_without_applying_deltas();
        }
        let store = shared.lock().unwrap();
        assert_eq!((store.rasters, store.blurs), (1, 1));
        assert_eq!((store.worker_rasters, store.worker_blurs), (0, 0));
        assert!(store.pending.is_some());
        drop(sender);
    }

    #[test]
    fn completed_worker_shadow_is_pixel_identical_to_a_cold_static_shadow() {
        let mut node = sting_node();
        let ctx = egui::Context::default();
        node.ports
            .insert("sample_time".into(), serde_json::json!(0.));
        ctx.run_ui(egui::RawInput::default(), |ui| show(ui, "sting", &node))
            .drop_without_applying_deltas();
        node.ports
            .insert("sample_time".into(), serde_json::json!(0.3));
        let reference_ctx = egui::Context::default();
        let reference =
            reference_ctx.run_ui(egui::RawInput::default(), |ui| show(ui, "sting", &node));
        let reference_id = shadows(&reference_ctx).lock().unwrap().entries[0]
            .1
            .texture
            .id();
        let expected = reference.textures_delta.set[&reference_id]
            .last()
            .unwrap()
            .image
            .clone();
        reference.drop_without_applying_deltas();
        let texture = shadows(&ctx).lock().unwrap().entries[0].1.texture.id();
        ctx.run_ui(egui::RawInput::default(), |ui| show(ui, "sting", &node))
            .drop_without_applying_deltas();
        {
            let shared = shadows(&ctx);
            let mut store = shared.lock().unwrap();
            let pending = store.pending.as_mut().expect("shadow worker started");
            // Wait for completion, allowing pool contention rather than timing
            // frame polling. The worker sends without taking the cache lock.
            // Keep a safety bound so a hung worker cannot wedge the test pool.
            pending.ready = Some(
                pending
                    .image
                    .recv_timeout(std::time::Duration::from_secs(60))
                    .expect("shadow worker did not complete")
                    .expect("shadow worker failed to raster"),
            );
        }
        let out = ctx.run_ui(egui::RawInput::default(), |ui| show(ui, "sting", &node));
        let egui::ImageData::Color(actual) = &out
            .textures_delta
            .set
            .get(&texture)
            .expect("completed shadow texture delta")
            .last()
            .expect("completed shadow uploaded")
            .image;
        let egui::ImageData::Color(expected) = &expected;
        assert_eq!(actual.size, expected.size);
        assert_eq!(actual.pixels, expected.pixels);
        out.drop_without_applying_deltas();
        assert_eq!(shadows(&ctx).lock().unwrap().rasters, 1);
    }

    #[test]
    fn sibling_frame_promotes_a_real_worker_image_from_channel_to_ready() {
        let ctx = egui::Context::default();
        shadow_frame(&ctx, "root", "a", 1.).drop_without_applying_deltas();
        shadow_frame(&ctx, "root", "b", 1.).drop_without_applying_deltas();
        let shared = shadows(&ctx);
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let expected = {
            let mut store = shared.lock().unwrap();
            let pending = store.pending.as_mut().expect("real shadow worker started");
            let image = pending
                .image
                .recv_timeout(std::time::Duration::from_secs(60))
                .expect("real shadow worker completed")
                .expect("real shadow worker rasterised");
            // Existing PendingShadow test seam: queue the real worker's image
            // deterministically without filling ready or racing frame polling.
            pending.image = receiver;
            assert!(pending.ready.is_none());
            sender.send(Some(image.clone())).unwrap();
            image
        };
        shadow_frame(&ctx, "sibling", "c", 1.).drop_without_applying_deltas();
        let texture = {
            let store = shared.lock().unwrap();
            let pending = store.pending.as_ref().expect("sibling retains root result");
            let ready = pending
                .ready
                .as_ref()
                .expect("frame promoted channel result");
            assert_eq!(ready.size, expected.size);
            assert_eq!(ready.pixels, expected.pixels);
            assert!(matches!(
                pending.image.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Empty)
            ));
            pending.texture
        };
        let out = shadow_frame(&ctx, "root", "b", 1.);
        assert!(out.textures_delta.set.contains_key(&texture));
        out.drop_without_applying_deltas();
        assert!(shared.lock().unwrap().pending.is_none());
        drop(sender);
    }

    #[test]
    fn animated_siblings_take_turns_on_the_shadow_worker() {
        let ctx = egui::Context::default();
        let mut node = sting_node();
        node.ports
            .insert("sample_time".into(), serde_json::json!(0.));
        let paint = |node: &scene::Node| {
            ctx.run_ui(egui::RawInput::default(), |ui| {
                show(ui, "a", node);
                show(ui, "b", node);
            })
            .drop_without_applying_deltas();
        };
        paint(&node);
        let shared = shadows(&ctx);
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let second = {
            let mut store = shared.lock().unwrap();
            let second = store.entries[1].0;
            store.pending = Some(pending_for(&store, 0, "busy", receiver));
            second
        };
        node.ports
            .insert("sample_time".into(), serde_json::json!(0.3));
        paint(&node);
        sender.send(None).unwrap();
        paint(&node);
        assert_eq!(shared.lock().unwrap().pending.as_ref().unwrap().key, second);
    }

    #[test]
    #[ignore = "explicit CBC headless GPU sting benchmark"]
    fn sting_shadow_frame_benchmark() {
        use std::sync::{Arc, Mutex};
        for synchronous in [true, false] {
            let mut node = sting_node();
            node.ports
                .insert("motion_profile".into(), serde_json::json!("legacy-sting"));
            let state = Arc::new(Mutex::new((node, Vec::<f64>::new())));
            let captured = state.clone();
            let mut harness = egui_kittest::Harness::builder()
                .with_size(egui::vec2(1600., 900.))
                .with_pixels_per_point(2.4)
                .wgpu()
                .build_ui(move |ui| {
                    shadows(ui.ctx()).lock().unwrap().synchronous_updates = synchronous;
                    let mut state = captured.lock().unwrap();
                    show(ui, "sting", &state.0);
                    state
                        .1
                        .push(shadows(ui.ctx()).lock().unwrap().last_shadow_ms);
                });
            let mut hashes = std::collections::HashSet::new();
            state.lock().unwrap().1.clear();
            for frame in 0..30 {
                state.lock().unwrap().0.ports.insert(
                    "sample_time".into(),
                    serde_json::json!(3. + frame as f64 / 30.),
                );
                harness.step();
                let image = harness.render().unwrap();
                assert_eq!(image.dimensions(), (3840, 2160));
                hashes.insert(blake3::hash(image.as_raw()));
            }
            let state = state.lock().unwrap();
            println!(
                "STING_SHADOW_FRAME_MS synchronous={synchronous} {:?}",
                state.1
            );
            let mut times = state.1.clone();
            times.sort_by(f64::total_cmp);
            println!(
                "STING_SHADOW synchronous={synchronous} distinct={}/30 shadow_ms_median={:.3} shadow_ms_max={:.3} samples={}",
                hashes.len(),
                times[times.len() / 2],
                times.last().unwrap(),
                times.len()
            );
            assert_eq!(hashes.len(), 30);
            if !synchronous {
                assert!(
                    *times.last().unwrap() < 1000. / 30.,
                    "shadow exceeded a 30 Hz frame budget"
                );
            }
        }
    }

    #[test]
    fn authored_legacy_primitive_names_never_paint_in_either_profile() {
        for profile in ["current", "legacy-sting"] {
            let node: scene::Node = serde_json::from_value(serde_json::json!({
                "family":"drawing","line":1,"is_template":false,
                "ports":{"w":100,"h":100,"sample_time":0.3,"motion_profile":profile,
                    "primitives":[{"kind":"ellipse","centre":[50,50],"radius":[20,20],"colour":"#ff0000","effects":[{"preset":"legacy-pop"}]}]}
            })).unwrap();
            let ctx = egui::Context::default();
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                super::show(ui, "untrusted", &node);
            });
            out.textures_delta.clear();
            assert!(
                !out.shapes
                    .iter()
                    .any(|s| matches!(s.shape, egui::Shape::Ellipse(_)))
            );
        }
    }
    #[test]
    fn ordinary_drawings_use_current_pop_and_sting_profile_uses_frozen_pop() {
        for profile in ["current", "legacy-sting"] {
            let source = format!(
                r##"---
scene: 1
name: drawing-profile
citizen: test
---
```mix
{{root:{{widget:"drawing",w:100,h:100,sample_time:0.3,motion_profile:"{profile}",primitives:[{{kind:"ellipse",centre:[50,50],radius:[20,20],colour:"#ff0000",effects:[{{preset:"pop"}}]}}]}}}}
```
"##
            );
            let mut view = crate::view::SceneView::new(&source).unwrap();
            let ctx = egui::Context::default();
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                view.show(ui);
            });
            out.textures_delta.clear();
            let ellipse = out
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Ellipse(e) => Some(e),
                    _ => None,
                })
                .expect("painted ellipse");
            let name = if profile == "current" {
                "pop"
            } else {
                "legacy-pop"
            };
            let expected = motion::scene::paint_preset(name).unwrap().sample(0.3);
            assert!(
                (ellipse.radius.x - 20. * expected.get(motion::Property::Scale) as f32).abs()
                    < 0.01
            );
            let other = motion::scene::paint_preset(if profile == "current" {
                "legacy-pop"
            } else {
                "pop"
            })
            .unwrap()
            .sample(0.3);
            assert!(
                (ellipse.radius.x - 20. * other.get(motion::Property::Scale) as f32).abs() > 0.1
            );
        }
    }
    use super::*;
    #[test]
    fn sibling_shadows_cache_and_float_translates_the_cached_raster() {
        let source = "---\nscene: 1\nname: shadows\ncitizen: test\n---\n```mix\n{root:{widget:\"drawing\",w:100,h:100,primitives:[{kind:\"ellipse\",centre:[50,50],radius:[20,20],colour:\"#ff0000\",effects:[{preset:\"float\"}]}],shadow:{colour:[242,70,70,77],offset:[0,5],blur:10}}}\n```\n";
        let node = scene::resolve(&scene::parse(source).unwrap())
            .unwrap()
            .nodes["root"]
            .clone();
        let ctx = egui::Context::default();
        for time in [2., 2.1] {
            ctx.run_ui(egui::RawInput::default(), |ui| {
                let mut n = node.clone();
                n.ports
                    .insert("sample_time".into(), serde_json::json!(time));
                show(ui, "a", &n);
                show(ui, "b", &n);
            })
            .drop_without_applying_deltas();
        }
        let store = shadows(&ctx);
        let store = store.lock().unwrap();
        assert_eq!(store.rasters, 2);
        assert_eq!(store.entries.len(), 2);
        assert!(store.hits >= 2);
        assert!(store.entries.iter().map(|(_, v)| v.pixels).sum::<usize>() <= SHADOW_PIXELS);
    }
}
