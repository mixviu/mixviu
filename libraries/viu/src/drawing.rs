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
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(n("w", 280.), n("h", 280.)), egui::Sense::hover());
    let time = node
        .ports
        .get("sample_time")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(10.);
    let Some(raw) = node.ports.get("primitives") else { return };
    if scene::drawing::validate(raw).is_err() || !time.is_finite() || time.abs()>3600. {return}
    let Ok(primitives) = serde_json::from_value::<Vec<Primitive>>(raw.clone())
    else {
        return;
    };
    static TABLE: std::sync::OnceLock<Vec<motion::Preset>> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(motion::presets);
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
            let Some(preset) = table.iter().find(|a| a.name == e.preset) else { return };
            let v = preset.sample(time - e.delay);
            scale *= v.get(Property::Scale) as f32;
            sx *= v.get(Property::ScaleX) as f32;
            sy *= v.get(Property::ScaleY) as f32;
            dy += v.get(Property::Ty) as f32;
            opacity *= v.get(Property::Opacity) as f32;
            if e.preset == "draw" {
                let length = p.radius[0] * p.sweep.to_radians().abs();
                if !length.is_finite() || length<=0. {return}
                fraction *= (p.dash_length.unwrap_or(length)
                    * (1. - v.get(Property::DashOffset) as f32 / 1000.) / length)
                    .clamp(0., 1.);
            }
        }
        let centre = rect.min + Vec2::new(p.centre[0], p.centre[1] + dy);
        let radius = Vec2::new(p.radius[0] * scale * sx, p.radius[1] * scale * sy);
        let Some(rgb) = parse_colour(&p.colour) else {return};
        let opacity=opacity.clamp(0.,1.);
        sampled.push([scale,sx,sy,opacity,fraction,dy]);
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
            let strips=r.width().ceil().clamp(1.,2048.) as usize;
            for i in 0..strips {
                let x = r.left() + r.width() * i as f32 / strips as f32;
                let mid=x+r.width()/strips as f32/2.;
                let edge = (mid - r.left()).min(r.right() - mid).min(radius.y);
                let h = (radius.y * radius.y - (radius.y - edge).powi(2))
                    .max(0.)
                    .sqrt();
                let t = (i as f32+0.5)/strips as f32;
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
        let common=dy;
        if float_offset.is_none() {float_offset=Some(common)}
        if float_offset.is_some_and(|v|v!=common) {float_offset=Some(f32::NAN)}
        if node.ports.contains_key("shadow") {shadow_shapes.push(shape.clone());}
        shapes.push(shape);
    }
    if node.ports.contains_key("shadow") {
    let dy=float_offset.filter(|n|n.is_finite()).unwrap_or(0.);
    for shape in &mut shadow_shapes {shape.translate(Vec2::new(0.,-dy));}
    if float_offset.is_some_and(|n|n.is_finite()) {for sample in &mut sampled {sample[5]=0.;}}
    paint_shadow(
        ui,
        id,
        rect,
        node.ports.get("shadow"),
        &shadow_shapes,
        format!("{rect:?}:{raw}:{sampled:?}:{}", node.ports.get("shadow").unwrap_or(&serde_json::Value::Null)),
        Vec2::new(0.,dy),
    );
    }
    ui.painter().extend(shapes);
}
fn parse_colour(text:&str)->Option<u32> {
    let hex=text.strip_prefix('#')?;
    if hex.len()!=6 {return None}
    u32::from_str_radix(hex,16).ok()
}

#[derive(Clone)]
struct ShadowCache {
    signature: String,
    texture: egui::TextureHandle,
}
fn paint_shadow(
    ui: &egui::Ui,
    id: &str,
    rect: egui::Rect,
    spec: Option<&serde_json::Value>,
    shapes: &[Shape],
    signature: String,
    translation:Vec2,
) {
    let Some(spec) = spec else { return };
    if scene::drawing::validate_shadow(spec).is_err() {return}
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
    let key = ui.scope_id().with(("drawing-shadow",id));
    let draw = |texture: &egui::TextureHandle| {
        let pos = origin + Vec2::new(shadow.offset[0], shadow.offset[1])+translation;
        ui.painter().image(
            texture.id(),
            egui::Rect::from_min_size(pos, Vec2::new(width as f32, height as f32)),
            egui::Rect::from_min_max(Pos2::ZERO, egui::pos2(1., 1.)),
            Color32::WHITE,
        );
    };
    if let Some(cache) = ui
        .ctx()
        .data_mut(|d| d.get_temp::<ShadowCache>(key))
        .filter(|c| c.signature == signature)
    {
        draw(&cache.texture);
        return;
    }
    let Some(mut mask) = tiny_skia::Pixmap::new(width, height) else {return};
    let input = shapes
        .iter()
        .cloned()
        .map(|shape| egui::epaint::ClippedShape {
            clip_rect: egui::Rect::EVERYTHING,
            shape,
            transform_after_tessellation: Default::default(),
        })
        .collect();
    for primitive in ui.ctx().tessellate(input, 1.) {
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
    let Some(image) = image::RgbaImage::from_raw(width, height, mask.data().to_vec()) else {return};
    let mut blurred = if shadow.blur>0. {image::imageops::fast_blur(&image, shadow.blur)} else {image};
    for pixel in blurred.pixels_mut() {
        *pixel = image::Rgba([
            shadow.colour[0] as u8,
            shadow.colour[1] as u8,
            shadow.colour[2] as u8,
            (pixel[3] as f32 * shadow.colour[3] / 255.).round() as u8,
        ]);
    }
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        blurred.as_raw(),
    );
    let texture = ui
        .ctx()
        .load_texture(format!("drawing-shadow-{key:?}"), image, Default::default());
    draw(&texture);
    ui.ctx()
        .data_mut(|d| d.insert_temp(key, ShadowCache { signature, texture }));
}
