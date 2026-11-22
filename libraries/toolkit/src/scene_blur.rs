//! Bounded CPU Gaussian filter for font-atlas text and vector paint.
//! Image textures need an owner-provided CPU image; those ranges stay sharp.
use egui::{Color32, LayerId, Rect, Shape, TextureHandle, Vec2, layers::ShapeIdx};
use std::hash::{Hash, Hasher};

#[derive(Clone)]
struct Cached {
    signature: u64,
    texture: TextureHandle,
}

pub(super) fn apply(ui: &egui::Ui, layer: LayerId, start: ShapeIdx, sigma: f32) {
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
    let ppp = ui
        .pixels_per_point()
        .min(2047. / rect.width())
        .min(2047. / rect.height());
    let size = (rect.size() * ppp).ceil();
    if size.x > 4096. || size.y > 2048. || size.x * size.y > 4_194_304. {
        return;
    }
    let width = size.x as u32;
    let height = size.y as u32;
    let meshes = ui.ctx().tessellate(shapes, ppp);
    if meshes.iter().any(|p| !matches!(&p.primitive,egui::epaint::Primitive::Mesh(m) if m.texture_id==egui::TextureId::Managed(0))) {return}
    let atlas = ui.ctx().fonts(|f| f.image());
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    for v in [sigma, ppp, rect.min.x, rect.min.y, rect.max.x, rect.max.y] {
        v.to_bits().hash(&mut hash);
    }
    for p in &meshes {
        let egui::epaint::Primitive::Mesh(m) = &p.primitive else {
            unreachable!()
        };
        m.indices.hash(&mut hash);
        for v in &m.vertices {
            for f in [v.pos.x, v.pos.y, v.uv.x, v.uv.y] {
                f.to_bits().hash(&mut hash);
            }
            v.color.to_array().hash(&mut hash);
        }
    }
    let signature = hash.finish();
    let key = egui::Id::unique(("scene-gaussian", layer, start.0));
    let old = ui.ctx().data_mut(|d| d.get_temp::<Cached>(key));
    let cached = if let Some(c) = old.filter(|c| c.signature == signature) {
        c
    } else {
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
        let image = image::RgbaImage::from_raw(width, height, rgba).unwrap();
        let image = image::imageops::fast_blur(&image, sigma * ppp);
        let pixels = image
            .pixels()
            .map(|p| Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        let image = egui::ColorImage::new([width as usize, height as usize], pixels);
        Cached {
            signature,
            texture: ui
                .ctx()
                .load_texture("scene-gaussian", image, Default::default()),
        }
    };
    let shape = Shape::image(
        cached.texture.id(),
        rect,
        Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
        Color32::WHITE,
    );
    ui.ctx().data_mut(|d| d.insert_temp(key, cached));
    ui.ctx().graphics_mut(|g| {
        let list = g.entry(layer);
        let end = list.next_idx().0;
        for i in start.0..end {
            list.mutate_shape(ShapeIdx(i), |s| s.shape = Shape::Noop);
        }
        if start.0 < end {
            list.mutate_shape(start, |s| s.shape = shape);
        }
    });
}
