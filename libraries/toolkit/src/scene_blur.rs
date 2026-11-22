//! Bounded CPU Gaussian filter for font-atlas text and vector paint.
//! Image textures need an owner-provided CPU image; those ranges stay sharp.
use egui::{Color32, LayerId, Rect, Shape, TextureHandle, Vec2, layers::ShapeIdx};
use std::hash::{Hash, Hasher};

#[derive(Clone)]
struct Cached {
    signature: u64,
    texture: TextureHandle,
    pixels: usize,
    // Retain the layout jobs used as font identities: their pointers cannot
    // be reused while an entry refers to them.
    _sources: Vec<egui::epaint::ClippedShape>,
    _fonts: Vec<std::sync::Arc<egui::FontData>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zoom_requires_scale_and_blur_and_wipe_requires_partial_clip() {
        let ctx = egui::Context::default();
        ctx.run_ui(egui::RawInput::default(), |ui| {
            let layer = ui.layer_id();
            let start = ui.ctx().graphics_mut(|g| g.entry(layer).next_idx());
            ui.label("A zoomed title");
            let before = ui.ctx().graphics_mut(|g| {
                g.entry(layer)
                    .all_entries()
                    .skip(start.0)
                    .fold(Rect::NOTHING, |r, s| {
                        r.union(s.shape.visual_bounding_rect())
                    })
            });
            super::super::paint_effect(
                ui,
                "zoom",
                layer,
                start,
                before,
                &serde_json::json!({"preset":"zoom","time":0.1}),
            );
            let shared = cache(ui.ctx());
            let c = shared.lock().unwrap();
            assert_eq!(c.rasterizations, 1, "zoom blur must rasterize");
            let painted = c
                .entries
                .back()
                .unwrap()
                .1
                ._sources
                .iter()
                .fold(Rect::NOTHING, |r, s| {
                    r.union(s.shape.visual_bounding_rect())
                });
            let expected = motion::scene::paint_preset("zoom")
                .unwrap()
                .sample(0.1)
                .get(motion::Property::Scale) as f32;
            assert!(
                (painted.width() / before.width() - expected).abs() < 0.001,
                "zoom must scale"
            );
        })
        .drop_without_applying_deltas();
        ctx.run_ui(egui::RawInput::default(), |ui| {
            let layer = ui.layer_id();
            let start = ui.ctx().graphics_mut(|g| g.entry(layer).next_idx());
            ui.label("A progressively wiped subtitle");
            let bounds = ui.ctx().graphics_mut(|g| {
                g.entry(layer)
                    .all_entries()
                    .skip(start.0)
                    .fold(Rect::NOTHING, |r, s| {
                        r.union(s.shape.visual_bounding_rect())
                    })
            });
            super::super::paint_effect(
                ui,
                "wipe",
                layer,
                start,
                bounds,
                &serde_json::json!({"preset":"wipe","time":0.4}),
            );
            let right = ui.ctx().graphics_mut(|g| {
                g.entry(layer)
                    .all_entries()
                    .skip(start.0)
                    .next()
                    .unwrap()
                    .clip_rect
                    .right()
            });
            assert!(right > bounds.left() && right < bounds.right());
        })
        .drop_without_applying_deltas();
    }
    #[test]
    fn repeated_frame_hits_before_atlas_or_raster_and_budget_is_bounded() {
        let ctx = egui::Context::default();
        ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label("A paused blurred sample");
        })
        .drop_without_applying_deltas();
        for i in 0..3 {
            ctx.run_ui(
                egui::RawInput {
                    time: Some(i as f64),
                    ..Default::default()
                },
                |ui| {
                    let layer = ui.layer_id();
                    let start = ui.ctx().graphics_mut(|g| g.entry(layer).next_idx());
                    ui.label("A paused blurred sample");
                    apply(ui, "label", layer, start, 2.);
                },
            )
            .drop_without_applying_deltas();
        }
        let shared = cache(&ctx);
        let c = shared.lock().unwrap();
        assert!(c.hits >= 1, "hits={} rasters={}", c.hits, c.rasterizations);
        assert_eq!(c.atlas_reads, c.rasterizations);
        assert_eq!(c.rasterizations, 1);
        assert_eq!(c.atlas_reads, 1);
        println!(
            "BLUR_CACHE hits={} rasterizations={} atlas_reads={}",
            c.hits, c.rasterizations, c.atlas_reads
        );
        assert!(c.entries.iter().map(|(_, c)| c.pixels).sum::<usize>() <= PIXELS);
    }
    #[test]
    fn stable_ids_and_texture_labels_are_not_allocators() {
        assert_eq!(egui::Id::unique("sample"), egui::Id::unique("sample"));
        let ctx = egui::Context::default();
        let a = ctx.load_texture(
            "same-label",
            egui::ColorImage::filled([1, 1], Color32::RED),
            Default::default(),
        );
        let b = ctx.load_texture(
            "same-label",
            egui::ColorImage::filled([1, 1], Color32::BLUE),
            Default::default(),
        );
        assert_ne!(a.id(), b.id());
    }
    #[test]
    fn hostile_filter_bounds_return_without_allocating() {
        let ctx = egui::Context::default();
        ctx.run_ui(egui::RawInput::default(), |ui| {
            for sigma in [f32::NAN, f32::INFINITY, -1., 0., 201.] {
                apply(ui, "hostile", ui.layer_id(), ShapeIdx(0), sigma);
            }
        })
        .drop_without_applying_deltas();
        assert_eq!(cache(&ctx).lock().unwrap().rasterizations, 0);
    }
}
#[derive(Default)]
struct Cache {
    entries: std::collections::VecDeque<(egui::Id, Cached)>,
    pass: u64,
    spent: usize,
    hits: usize,
    rasterizations: usize,
    atlas_reads: usize,
}
const PIXELS: usize = 8 * 1024 * 1024;
fn cache(ctx: &egui::Context) -> std::sync::Arc<std::sync::Mutex<Cache>> {
    let key = egui::Id::unique("scene-blur-budget-v1");
    ctx.data_mut(|d| {
        if let Some(c) = d.get_temp::<std::sync::Arc<std::sync::Mutex<Cache>>>(key) {
            return c;
        }
        let c = std::sync::Arc::new(std::sync::Mutex::new(Cache::default()));
        d.insert_temp(key, c.clone());
        c
    })
}
fn hash_shape(shape: &Shape, hash: &mut impl Hasher) {
    match shape {
        Shape::Text(t) => {
            (std::sync::Arc::as_ptr(&t.galley.job) as usize).hash(hash);
            for n in [
                t.pos.x,
                t.pos.y,
                t.angle,
                t.opacity_factor,
                t.underline.width,
            ] {
                n.to_bits().hash(hash);
            }
            t.fallback_color.hash(hash);
            t.override_text_color.hash(hash);
            t.underline.color.hash(hash);
        }
        Shape::Vec(shapes) => {
            for s in shapes {
                hash_shape(s, hash)
            }
        }
        other => format!("{other:?}").hash(hash),
    }
}

pub(super) fn apply(ui: &egui::Ui, node: &str, layer: LayerId, start: ShapeIdx, sigma: f32) {
    if !sigma.is_finite() || !(0.3..=200.).contains(&sigma) {
        return;
    }
    let shapes = ui.ctx().graphics_mut(|g| {
        g.entry(layer)
            .all_entries()
            .skip(start.0)
            .cloned()
            .collect::<Vec<_>>()
    });
    let mut rect = Rect::NOTHING;
    for s in &shapes {
        rect = rect.union(s.shape.visual_bounding_rect());
    }
    if !rect.is_finite() || !rect.is_positive() {
        return;
    }
    rect = rect.expand(sigma * 3.);
    if !rect.is_finite() || !rect.is_positive() {
        return;
    }
    let ppp = ui
        .pixels_per_point()
        .min(2047. / rect.width())
        .min(2047. / rect.height());
    let size = (rect.size() * ppp).ceil();
    if !ppp.is_finite()
        || ppp <= 0.
        || !size.is_finite()
        || size.x < 1.
        || size.y < 1.
        || size.x > 4096.
        || size.y > 2048.
        || size.x * size.y > 4_194_304.
    {
        return;
    }
    let width = size.x as u32;
    let height = size.y as u32;
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    // Family mappings, font-data identity and text options are cheap inputs.
    // The atlas pixels are consulted only on a miss, never hashed or cloned
    // on an identical-frame hit.
    let fonts = ui.ctx().fonts(|f| {
        f.definitions().families.hash(&mut hash);
        format!("{:?}", f.options()).hash(&mut hash);
        f.definitions()
            .font_data
            .iter()
            .map(|(name, font)| {
                name.hash(&mut hash);
                (std::sync::Arc::as_ptr(font) as usize).hash(&mut hash);
                font.clone()
            })
            .collect()
    });
    for v in [sigma, ppp, rect.min.x, rect.min.y, rect.max.x, rect.max.y] {
        v.to_bits().hash(&mut hash);
    }
    let mut clip = Rect::NOTHING;
    for p in &shapes {
        clip = clip.union(p.clip_rect);
        for v in [
            p.clip_rect.min.x,
            p.clip_rect.min.y,
            p.clip_rect.max.x,
            p.clip_rect.max.y,
        ] {
            v.to_bits().hash(&mut hash);
        }
        hash_shape(&p.shape, &mut hash);
        format!("{:?}", p.transform_after_tessellation).hash(&mut hash);
    }
    let signature = hash.finish();
    let key = ui.scope_id().with(("scene-gaussian", node));
    let shared = cache(ui.ctx());
    let old = {
        let mut cache = shared.lock().unwrap_or_else(|e| e.into_inner());
        let pass = ui.ctx().cumulative_pass_nr();
        if cache.pass != pass {
            cache.pass = pass;
            cache.spent = 0;
        }
        let pixels = width as usize * height as usize;
        if cache.spent + pixels > PIXELS {
            return;
        }
        cache.spent += pixels;
        let found = cache
            .entries
            .iter()
            .position(|(id, c)| *id == key && c.signature == signature);
        found
            .and_then(|i| cache.entries.remove(i))
            .map(|(_, entry)| {
                cache.hits += 1;
                entry
            })
    };
    let cached = if let Some(c) = old {
        c
    } else {
        let meshes = ui.ctx().tessellate(shapes.clone(), ppp);
        if meshes.iter().any(|p| !matches!(&p.primitive,egui::epaint::Primitive::Mesh(m) if m.texture_id==egui::TextureId::Managed(0))) {return}
        let atlas = ui.ctx().fonts(|f| f.image());
        {
            let mut c = shared.lock().unwrap_or_else(|e| e.into_inner());
            c.atlas_reads += 1;
            c.rasterizations += 1;
        }
        let mut pixels = vec![[0f32; 4]; (width * height) as usize];
        let edge = |a: egui::Pos2, b: egui::Pos2, p: egui::Pos2| {
            (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
        };
        let top_left = |a: egui::Pos2, b: egui::Pos2| b.y < a.y || (b.y == a.y && b.x > a.x);
        let texel = |x: i32, y: i32| {
            atlas.pixels[(y.clamp(0, atlas.size[1] as i32 - 1) as usize) * atlas.size[0]
                + x.clamp(0, atlas.size[0] as i32 - 1) as usize]
                .to_array()
        };
        for p in &meshes {
            let egui::epaint::Primitive::Mesh(mesh) = &p.primitive else {
                unreachable!()
            };
            for triangle in mesh.indices.chunks_exact(3) {
                let mut v = [
                    mesh.vertices[triangle[0] as usize],
                    mesh.vertices[triangle[1] as usize],
                    mesh.vertices[triangle[2] as usize],
                ];
                let mut pos = v.map(|v| egui::Pos2::ZERO + (v.pos - rect.min) * ppp);
                if edge(pos[0], pos[1], pos[2]) < 0. {
                    v.swap(1, 2);
                    pos.swap(1, 2);
                }
                let area = edge(pos[0], pos[1], pos[2]);
                if area <= 0. {
                    continue;
                }
                let bounds =
                    Rect::from_points(&pos).intersect(Rect::from_min_size(egui::Pos2::ZERO, size));
                for y in bounds.top().floor() as u32..(bounds.bottom().ceil() as u32).min(height) {
                    for x in bounds.left().floor() as u32..(bounds.right().ceil() as u32).min(width)
                    {
                        let point = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
                        if !p.clip_rect.contains(rect.min + point.to_vec2() / ppp) {
                            continue;
                        }
                        let e = [
                            edge(pos[1], pos[2], point),
                            edge(pos[2], pos[0], point),
                            edge(pos[0], pos[1], point),
                        ];
                        let edges = [(1, 2), (2, 0), (0, 1)];
                        if (0..3).any(|i| {
                            e[i] < 0. || (e[i] == 0. && !top_left(pos[edges[i].0], pos[edges[i].1]))
                        }) {
                            continue;
                        }
                        let w = e.map(|n| n / area);
                        let uv = v[0].uv.to_vec2() * w[0]
                            + v[1].uv.to_vec2() * w[1]
                            + v[2].uv.to_vec2() * w[2];
                        let uv = uv * Vec2::new(atlas.size[0] as f32, atlas.size[1] as f32)
                            - Vec2::splat(0.5);
                        let tx = uv.x.floor() as i32;
                        let ty = uv.y.floor() as i32;
                        let fx = uv.x - uv.x.floor();
                        let fy = uv.y - uv.y.floor();
                        let t = [
                            texel(tx, ty),
                            texel(tx + 1, ty),
                            texel(tx, ty + 1),
                            texel(tx + 1, ty + 1),
                        ];
                        let c = v.map(|v| v.color.to_array());
                        let mut src = [0.; 4];
                        for channel in 0..4 {
                            let texture = (t[0][channel] as f32 * (1. - fx) * (1. - fy)
                                + t[1][channel] as f32 * fx * (1. - fy)
                                + t[2][channel] as f32 * (1. - fx) * fy
                                + t[3][channel] as f32 * fx * fy)
                                / 255.;
                            src[channel] = (c[0][channel] as f32 * w[0]
                                + c[1][channel] as f32 * w[1]
                                + c[2][channel] as f32 * w[2])
                                / 255.
                                * texture;
                        }
                        let dst = &mut pixels[(y * width + x) as usize];
                        for channel in 0..4 {
                            dst[channel] = src[channel] + dst[channel] * (1. - src[3]);
                        }
                    }
                }
            }
        }
        let rgba = pixels
            .into_iter()
            .flat_map(|p| p.map(|n| (n.clamp(0., 1.) * 255.).round() as u8))
            .collect();
        let Some(image) = image::RgbaImage::from_raw(width, height, rgba) else {
            return;
        };
        let image = image::imageops::fast_blur(&image, sigma * ppp);
        let pixels = image
            .pixels()
            .map(|p| Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        let image = egui::ColorImage::new([width as usize, height as usize], pixels);
        Cached {
            signature,
            pixels: width as usize * height as usize,
            _sources: shapes,
            _fonts: fonts,
            texture: ui.ctx().load_texture(
                format!("scene-gaussian-{key:?}"),
                image,
                Default::default(),
            ),
        }
    };
    let shape = Shape::image(
        cached.texture.id(),
        Rect::from_min_size(rect.min, size / ppp),
        Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
        Color32::WHITE,
    );
    {
        let mut cache = shared.lock().unwrap_or_else(|e| e.into_inner());
        cache.entries.retain(|(id, _)| *id != key);
        while cache.entries.len() >= 8
            || cache.entries.iter().map(|(_, c)| c.pixels).sum::<usize>() + cached.pixels > PIXELS
        {
            cache.entries.pop_front();
        }
        cache.entries.push_back((key, cached));
    }
    ui.ctx().graphics_mut(|g| {
        let list = g.entry(layer);
        let end = list.next_idx().0;
        for i in start.0..end {
            list.mutate_shape(ShapeIdx(i), |s| s.shape = Shape::Noop);
        }
        if start.0 < end {
            list.mutate_shape(start, |s| {
                s.shape = shape;
                s.clip_rect = clip;
            });
        }
    });
}
