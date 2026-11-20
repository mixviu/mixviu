// SPDX-License-Identifier: MIT OR Apache-2.0
//! Static, bounded radial colour fields. No clock or repaint requests.
use egui::{Color32, Rect, Shape, epaint::{Mesh,Vertex,WHITE_UV}};
use serde_json::Value;
pub(crate) fn shape(rect:Rect, base:Color32, radials:&Value)->Shape {
    let mut mesh=Mesh::default();
    const NX:u32=96; const NY:u32=64;
    for y in 0..=NY { for x in 0..=NX {
        let p=egui::pos2(rect.left()+rect.width()*x as f32/NX as f32,rect.top()+rect.height()*y as f32/NY as f32);
        let mut rgb=[base.r() as f64,base.g() as f64,base.b() as f64];
        for r in radials.as_array().into_iter().flatten().rev() {
            let dx=(p.x as f64-rect.left() as f64-rect.width() as f64*r["x"].as_f64().unwrap())/r["rx"].as_f64().unwrap();
            let dy=(p.y as f64-rect.top() as f64-rect.height() as f64*r["y"].as_f64().unwrap())/r["ry"].as_f64().unwrap();
            let a=(1.-dx.hypot(dy)/r["stop"].as_f64().unwrap()).clamp(0.,1.);
            let c=u32::from_str_radix(&r["color"].as_str().unwrap()[1..],16).unwrap();
            for (i,channel) in rgb.iter_mut().enumerate() {*channel=*channel*(1.-a)+((c>>(16-i*8))&255) as f64*a;}
        }
        mesh.vertices.push(Vertex{pos:p,uv:WHITE_UV,color:Color32::from_rgb(rgb[0].round() as u8,rgb[1].round() as u8,rgb[2].round() as u8)});
    }}
    for y in 0..NY {for x in 0..NX {let i=y*(NX+1)+x; mesh.indices.extend([i,i+1,i+NX+1,i+1,i+NX+2,i+NX+1]);}}
    Shape::mesh(mesh)
}
