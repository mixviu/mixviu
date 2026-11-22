//! Scene vector paint: no brand geometry or application timing lives here.
use egui::{Color32, Pos2, Shape, Stroke, Vec2};
use motion::Property;
use scene::{Node, drawing::Primitive};

pub fn show(ui: &mut egui::Ui, node: &Node) {
    let n = |key: &str, default| node.ports.get(key).and_then(serde_json::Value::as_f64).unwrap_or(default) as f32;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(n("w", 280.), n("h", 280.)), egui::Sense::hover());
    let time = n("sample_time", 10.) as f64;
    let Ok(primitives) = serde_json::from_value::<Vec<Primitive>>(node.ports["primitives"].clone()) else { return };
    static TABLE: std::sync::OnceLock<Vec<motion::Preset>> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(motion::presets);
    for p in primitives {
        let mut scale = 1.;
        let mut sy = 1.;
        let mut sx = 1.;
        let mut dy = 0.;
        let mut opacity = 1.;
        let mut fraction = 1.;
        for e in &p.effects {
            let preset = table.iter().find(|a| a.name == e.preset).unwrap();
            let v = preset.sample(time-e.delay);
            scale *= v.get(Property::Scale) as f32;
            sx *= v.get(Property::ScaleX) as f32;
            sy *= v.get(Property::ScaleY) as f32;
            dy += v.get(Property::Ty) as f32;
            opacity *= v.get(Property::Opacity) as f32;
            if e.preset == "draw" { fraction *= 1.-v.get(Property::DashOffset) as f32/1000.; }
        }
        let centre = rect.min + Vec2::new(p.centre[0], p.centre[1]+dy);
        let radius = Vec2::new(p.radius[0]*scale*sx, p.radius[1]*scale*sy);
        let rgb = u32::from_str_radix(&p.colour[1..],16).unwrap();
        let colour = Color32::from_rgb((rgb>>16) as u8,(rgb>>8) as u8,rgb as u8).gamma_multiply(opacity.clamp(0.,1.));
        let shape = if p.kind == "arc" {
            if fraction <= 0. { continue; }
            let sweep = p.sweep.to_radians()*fraction;
            let points = (0..=256).map(|i| {
                let a=p.start.to_radians()+sweep*i as f32/256.;
                centre+Vec2::new(radius.x*a.cos(),radius.y*a.sin())
            }).collect::<Vec<Pos2>>();
            let stroke = Stroke::new(p.width*scale,colour);
            Shape::Vec(vec![Shape::circle_filled(points[0],stroke.width/2.,colour),Shape::circle_filled(points[256],stroke.width/2.,colour),Shape::line(points,stroke)])
        } else if p.kind == "bar" {
            let r=egui::Rect::from_center_size(centre,radius*2.);
            let end = p.end_colour.as_ref().map(|c| u32::from_str_radix(&c[1..],16).unwrap()).unwrap_or(rgb);
            let end=Color32::from_rgb((end>>16) as u8,(end>>8) as u8,end as u8).gamma_multiply(opacity);
            let mut mesh=egui::Mesh::default();
            // Rounded horizontal gradient, tessellated as narrow strips.
            for i in 0..260 {
                let x=r.left()+r.width()*i as f32/260.;
                let edge=(x-r.left()).min(r.right()-x).min(radius.y);
                let h=(radius.y*radius.y-(radius.y-edge).powi(2)).max(0.).sqrt();
                let c=egui::ecolor::tint_color_towards(colour,end);
                let t=i as f32/259.;
                let a=colour.to_array();let b=end.to_array();
                let c=Color32::from_rgba_premultiplied((a[0] as f32*(1.-t)+b[0] as f32*t) as u8,(a[1] as f32*(1.-t)+b[1] as f32*t) as u8,(a[2] as f32*(1.-t)+b[2] as f32*t) as u8,c.a());
                mesh.add_colored_rect(egui::Rect::from_min_max(egui::pos2(x,centre.y-h),egui::pos2(x+r.width()/260.,centre.y+h)),c);
            }
            Shape::mesh(mesh)
        } else {
            Shape::ellipse_filled(centre,radius,colour)
        };
        if let Some(shadow)=p.shadow {
            let mut shadow_shape=shape.clone();
            egui::epaint::shape_transform::adjust_colors(&mut shadow_shape,|c| *c=Color32::from_rgba_unmultiplied(shadow.colour[0],shadow.colour[1],shadow.colour[2],shadow.colour[3]).gamma_multiply(opacity));
            // Gaussian convolution of the alpha silhouette, sampled symmetrically.
            let mut total=0.;
            for y in -8..=8 { for x in -8..=8 { total+=(-((x*x+y*y) as f32)/18.).exp(); } }
            for y in -8..=8 { for x in -8..=8 {
                let weight=(-((x*x+y*y) as f32)/18.).exp()/total;
                let mut s=shadow_shape.clone();
                s.translate(Vec2::new(shadow.offset[0]+x as f32*shadow.blur/6.,shadow.offset[1]+y as f32*shadow.blur/6.));
                egui::epaint::shape_transform::adjust_colors(&mut s,|c| *c=c.gamma_multiply(weight));
                ui.painter().add(s);
            } }
        }
        ui.painter().add(shape);
    }
}
