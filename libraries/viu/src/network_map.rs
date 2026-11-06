// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reusable radial network canvas. Measured-label geometry is adapted from
//! MixOS 8779f332 Workshop; owners supply topology, traffic styling and actions.
use crate::widgets::{Intent, Widget};
use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use toolkit::chrome::Chrome;
#[derive(Clone)]
struct Curve {
    peer: String,
    out: bool,
    points: Vec<Pos2>,
}
#[derive(Clone, PartialEq)]
struct GeometryKey {
    rect: Rect,
    unit: f32,
    chrome: Chrome,
    ids: Vec<String>,
    nodes: Vec<(String, Option<String>, bool)>,
}
struct Geometry {
    key: GeometryKey,
    labels: Vec<NodeLabel>,
    positions: Vec<Placement>,
    dashes: Vec<Shape>,
    curves: Vec<Curve>,
    index: HashMap<String, [usize; 2]>,
}
impl Geometry {
    fn new(ui: &Ui, key: GeometryKey) -> Self {
        let padding = key.unit * 0.3;
        let labels: Vec<_> = key
            .nodes
            .iter()
            .map(|(name, ip, _)| {
                let text = |value: &str, style: egui::TextStyle| {
                    toolkit::tabs::label(
                        ui,
                        value,
                        style.resolve(ui.style()),
                        key.rect.width() * 0.25,
                        0.0,
                    )
                    .0
                };
                let name = text(name, egui::TextStyle::Body);
                let ip = ip.as_deref().map(|ip| text(ip, egui::TextStyle::Small));
                let size = Vec2::new(
                    name.size().x.max(ip.as_ref().map_or(0.0, |g| g.size().x)),
                    name.size().y + ip.as_ref().map_or(0.0, |g| g.size().y + padding),
                ) + Vec2::splat(padding * 2.0);
                NodeLabel { name, ip, size }
            })
            .collect();
        let positions = map_layout(
            key.rect,
            &labels.iter().map(|l| l.size).collect::<Vec<_>>(),
            key.unit,
        );
        let dashes = unobserved_links(&positions, key.unit, &key.chrome.palette);
        let mut curves = Vec::new();
        let mut index = HashMap::new();
        for (i, peer) in key.ids.iter().enumerate().skip(1) {
            index.insert(peer.clone(), [curves.len(), curves.len() + 1]);
            for out in [true, false] {
                curves.push(Curve {
                    peer: peer.clone(),
                    out,
                    points: spoke(positions[0], positions[i], out, key.unit),
                });
            }
        }
        Self {
            key,
            labels,
            positions,
            dashes,
            curves,
            index,
        }
    }
    fn curve(&self, peer: &str, out: bool) -> Option<&Curve> {
        self.index
            .get(peer)
            .map(|indices| &self.curves[indices[usize::from(!out)]])
    }
}

fn geometry_for(ui: &Ui, key: GeometryKey) -> Arc<Geometry> {
    let id = ui.scope_id().with("network-geometry");
    let cached = ui.ctx().data_mut(|d| d.get_temp::<Arc<Geometry>>(id));
    if let Some(cached) = cached.filter(|g| g.key == key) {
        cached
    } else {
        let geometry = Arc::new(Geometry::new(ui, key));
        ui.ctx().data_mut(|d| d.insert_temp(id, geometry.clone()));
        geometry
    }
}

struct NodeLabel {
    name: std::sync::Arc<egui::Galley>,
    ip: Option<std::sync::Arc<egui::Galley>>,
    size: Vec2,
}

#[derive(Clone, Copy)]
struct Placement {
    at: Pos2,
    radius: f32,
    label: Rect,
}

/// The lines x = +/- slope*y separate the centre plate (with clearance) from
/// the two sides of the map. Coordinates are relative to the centre node.
fn label_slope(width: f32, unit: f32) -> f32 {
    // The plate starts at local radius + label gap. Inflate its upper corners
    // horizontally and upwards so the separator also leaves stroke clearance.
    (width * 0.5 + unit * 0.5) / (unit * (1.3 + 0.4 - 0.25))
}

fn peer_arc_end(axes: Vec2, label_width: f32, unit: f32) -> f32 {
    let slope = label_slope(label_width, unit);
    let normal_length = Vec2::new(1.0, slope).length();
    // Leave circle clearance at either lower boundary. Solve
    // axes.x*cos(t) - slope*axes.y*sin(t) = clearance*normal_length.
    // If neither boundary fits below horizontal, reserve the entire lower half.
    let ellipse = Vec2::new(axes.x, slope * axes.y);
    let clearance = unit * (0.9 + 0.5) * normal_length;
    let phase = ellipse.y.atan2(ellipse.x);
    ((clearance / ellipse.length().max(f32::EPSILON))
        .clamp(0.0, 1.0)
        .acos()
        - phase)
        .max(0.0)
}

fn peer_angle(index: usize, count: usize, arc_end: f32) -> f32 {
    use std::f32::consts::PI;
    if count == 1 {
        return 0.0;
    }
    if count == 2 {
        return if index == 0 { PI } else { 0.0 };
    }
    // Fill the remaining arc through the top, including its two boundaries.
    // The omitted lower wedge is sized by the measured label and the ellipse.
    arc_end - index as f32 * (PI + 2.0 * arc_end) / (count - 1) as f32
}

fn map_layout(rect: Rect, labels: &[Vec2], unit: f32) -> Vec<Placement> {
    let centre = rect.center();
    let radius = unit * 0.9;
    let gap = unit * 0.4;
    let padding = unit * 0.5;
    // Bound the extent of a radially placed plate in either axis. Reserving
    // measured text margins lets the ellipse use the remaining map rectangle.
    let margin = labels.iter().skip(1).fold(Vec2::ZERO, |margin, size| {
        let half = *size * 0.5;
        let common = radius + gap + padding + half.length() * 0.5;
        margin.max(half * 1.5 + Vec2::splat(common))
    });
    let axes = (rect.size() * 0.5 - margin).max(Vec2::ZERO);
    let arc_end = peer_arc_end(axes, labels[0].x, unit);
    labels
        .iter()
        .enumerate()
        .map(|(i, size)| {
            let (at, radius, direction) = if i == 0 {
                (centre, unit * 1.3, Vec2::Y)
            } else {
                let angle = peer_angle(i - 1, labels.len() - 1, arc_end);
                let delta = Vec2::new(angle.cos() * axes.x, angle.sin() * axes.y);
                (
                    centre + delta,
                    radius,
                    delta / delta.length().max(f32::EPSILON),
                )
            };
            let half = *size * 0.5;
            let support = direction.x.abs() * half.x + direction.y.abs() * half.y;
            Placement {
                at,
                radius,
                label: Rect::from_center_size(at + direction * (radius + gap + support), *size),
            }
        })
        .collect()
}

/// Both directions use separate side ports on this node, leaving room for its
/// label below. The peer ports face inward, away from its outward label plate.
fn spoke_controls(local: Placement, peer: Placement, out: bool, unit: f32) -> [Pos2; 4] {
    let delta = peer.at - local.at;
    let radial = delta / delta.length().max(f32::EPSILON);
    let normal = Vec2::new(-radial.y, radial.x);
    let side = if delta.x < 0.0 { -1.0 } else { 1.0 };
    let lane = if out { -1.0 } else { 1.0 };
    let slope = label_slope(local.label.width(), unit);
    let port = 0.35_f32.min(0.5 / Vec2::new(1.0, slope).length());
    let along = (1.0 - port * port).sqrt();
    let a = local.at + Vec2::new(side * along, side * lane * port) * local.radius;
    let b = peer.at - radial * along * peer.radius + normal * lane * port * peer.radius;
    let lead = (local.label.width() * 0.5 + unit * 2.0)
        .min(delta.length() * 0.5)
        .min((delta.length() - local.radius - unit * 0.5).max(0.0));
    let c = a + Vec2::new(side * lead, 0.0);
    // All four controls stay on the peer's side of the centre-label wedge and
    // inward of the peer. A Bezier lies in their convex hull, so neither label
    // can be crossed, including between the sampled painter points.
    let d = c.lerp(b, 0.5);
    [a, c, d, b]
}

fn spoke(local: Placement, peer: Placement, out: bool, unit: f32) -> Vec<Pos2> {
    let [a, c, d, b] = spoke_controls(local, peer, out, unit);
    let mut points: Vec<_> = (0..=40)
        .map(|i| {
            let t = i as f32 / 40.0;
            let s = 1.0 - t;
            Pos2::new(
                s.powi(3) * a.x + 3.0 * s * s * t * c.x + 3.0 * s * t * t * d.x + t.powi(3) * b.x,
                s.powi(3) * a.y + 3.0 * s * s * t * c.y + 3.0 * s * t * t * d.y + t.powi(3) * b.y,
            )
        })
        .collect();
    if !out {
        points.reverse();
    }
    points
}

fn curve(a: Pos2, b: Pos2, bow: f32) -> Vec<Pos2> {
    let delta = b - a;
    let normal = Vec2::new(-delta.y, delta.x) / delta.length().max(1.0);
    let control = a + delta * 0.5 + normal * bow;
    (0..=40)
        .map(|i| {
            let t = i as f32 / 40.0;
            Pos2::new(
                (1.0 - t).powi(2) * a.x + 2.0 * (1.0 - t) * t * control.x + t * t * b.x,
                (1.0 - t).powi(2) * a.y + 2.0 * (1.0 - t) * t * control.y + t * t * b.y,
            )
        })
        .collect()
}

/// Peer pairs have no observed counters on this broker. Their painter accepts
/// geometry and theme roles only, so it cannot imply measured traffic.
fn unobserved_links(
    positions: &[Placement],
    unit: f32,
    palette: &toolkit::chrome::Palette,
) -> Vec<Shape> {
    let mut shapes = Vec::new();
    for i in 1..positions.len() {
        for j in i + 1..positions.len() {
            let a = positions[i];
            let b = positions[j];
            let delta = b.at - a.at;
            let normal = Vec2::new(-delta.y, delta.x) / delta.length().max(1.0);
            let control = a.at + delta * 0.5 - normal * unit * 2.0;
            let start = a.at + (control - a.at).normalized() * a.radius;
            let end = b.at + (control - b.at).normalized() * b.radius;
            let points = curve(start, end, -unit * 2.0);
            shapes.extend(Shape::dashed_line(
                &points,
                Stroke::new(unit * 0.06, palette.separator),
                unit * 0.2,
                unit * 0.5,
            ));
        }
    }
    shapes
}

/// Sample by arc length, matching SVG getPointAtLength rather than letting
/// a quadratic's varying derivative change the dot's speed along a spoke.
fn point_at(points: &[Pos2], fraction: f32) -> Pos2 {
    let length: f32 = points.windows(2).map(|s| s[0].distance(s[1])).sum();
    let mut remaining = length * fraction.clamp(0.0, 1.0);
    for segment in points.windows(2) {
        let distance = segment[0].distance(segment[1]);
        if remaining <= distance {
            return segment[0].lerp(segment[1], remaining / distance.max(f32::EPSILON));
        }
        remaining -= distance;
    }
    *points.last().unwrap()
}
fn width(count: u64, unit: f32) -> f32 {
    unit * (0.08 + ((1.0 + count as f32).log2() * 0.075).min(0.38))
}
fn u64_value(v: &Value) -> u64 {
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}
fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap_or("")
}
pub fn validate(value: &Value) -> Result<(), String> {
    if value["nodes"].as_array().is_none_or(|nodes| {
        nodes
            .iter()
            .any(|n| !n["id"].is_string() || !n["label"].is_string())
    }) || !value["events"].is_array()
        || !value["links"].is_object()
    {
        return Err("network map requires nodes, events and links".into());
    }
    Ok(())
}
pub fn admit(current: &Value, action: &str, value: &Value) -> Result<(), String> {
    if action == "set" {
        return validate(value);
    }
    let id = if value.is_string() {
        value.as_str().unwrap()
    } else {
        text(value, "id")
    };
    if ["node", "inspect", "edge"].contains(&action)
        && current["nodes"]
            .as_array()
            .is_some_and(|ns| ns.iter().any(|n| n["id"] == id))
    {
        return Ok(());
    }
    Err("unknown network node or action".into())
}
#[derive(Clone)]
struct Particle {
    event: Value,
    start: f64,
}
#[derive(Clone, Default)]
struct Motion {
    last_seq: u64,
    started_ms: u64,
    particles: Vec<Particle>,
    last_frame: f64,
}
impl Motion {
    fn feed(&mut self, value: &Value, frame: f64) {
        let now = u64_value(&value["now_ms"]);
        let started_ms = u64_value(&value["epoch_ms"]);
        let reset = self.started_ms != started_ms;
        if reset {
            *self = Self {
                started_ms,
                ..Default::default()
            };
        }
        let first = reset || self.last_frame + 1. < frame;
        let events = value["events"].as_array().unwrap();
        let mut fresh: Vec<_> = events
            .iter()
            .filter(|e| {
                u64_value(&e["seq"]) > self.last_seq
                    && now.saturating_sub(u64_value(&e["t"])) <= if first { 3000 } else { 15000 }
            })
            .cloned()
            .collect();
        fresh.sort_by_key(|e| u64_value(&e["t"]));
        if let Some(last) = events.last() {
            self.last_seq = u64_value(&last["seq"]);
        }
        if let (Some(first), Some(last)) = (fresh.first(), fresh.last()) {
            let t0 = u64_value(&first["t"]);
            let span = u64_value(&last["t"]).saturating_sub(t0).max(1) as f64;
            for event in fresh {
                let delay = u64_value(&event["t"]).saturating_sub(t0) as f64 / span
                    * span.min(1000.)
                    / 1000.;
                let extra = event["delay_s"].as_f64().unwrap_or(0.);
                self.particles.push(Particle {
                    event,
                    start: frame + delay + extra,
                });
            }
        }
        self.particles
            .retain(|p| frame < p.start + p.event["duration_s"].as_f64().unwrap_or(1.1));
        if self.particles.len() > 2000 {
            self.particles.drain(..self.particles.len() - 2000);
        }
        self.last_frame = frame;
    }
}
fn hue(role: &str, p: &toolkit::chrome::Palette) -> Color32 {
    match role {
        "accent" => p.accent,
        "warning" => p.warning,
        "danger" => p.danger,
        _ => p.text_dim,
    }
}
fn emit(events: &mut Vec<Intent>, id: &str, action: &str, value: Value) {
    events.push(Intent {
        id: id.into(),
        action: action.into(),
        value,
    });
}
pub fn draw(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry_out: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    let id = &widget.id;
    let nodes = value["nodes"].as_array().unwrap();
    let unit = ui.text_style_height(&egui::TextStyle::Body);
    let (_, rect) = ui.allocate_space(Vec2::new(
        ui.available_width(),
        (ui.available_height() * 0.62).max(unit * 16.),
    ));
    toolkit::canvas::surround(ui, rect);
    let response = ui.interact(rect, ui.scope_id().with(id), Sense::hover());
    if nodes.is_empty() {
        return response;
    }
    let chrome = Chrome::of(ui.ctx());
    let p = &chrome.palette;
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let padding = unit * 0.3;
    let geometry = geometry_for(
        ui,
        GeometryKey {
            rect,
            unit,
            chrome: chrome.clone(),
            ids: nodes.iter().map(|n| text(n, "id").to_owned()).collect(),
            nodes: nodes
                .iter()
                .map(|n| {
                    (
                        text(n, "label").into(),
                        n["detail"].as_str().map(str::to_owned),
                        n["primary"] == true,
                    )
                })
                .collect(),
        },
    );
    let curves = &geometry.curves;
    let positions = &geometry.positions;
    painter.extend(geometry.dashes.iter().cloned());
    for curve in curves {
        let node = nodes.iter().find(|n| n["id"] == curve.peer).unwrap();
        let link = &value["links"][text(node, "id")];
        let count = u64_value(&link[if curve.out { "out" } else { "in" }]);
        painter.add(Shape::line(
            curve.points.clone(),
            Stroke::new(
                width(count, unit),
                if curve.out { p.accent } else { p.warning }.gamma_multiply(if count > 0 {
                    0.55
                } else {
                    0.2
                }),
            ),
        ));
    }
    let mut activated = false;
    for curve in curves {
        let node = nodes.iter().find(|n| n["id"] == curve.peer).unwrap();
        let mid = point_at(&curve.points, 0.5);
        let r = ui.interact(
            Rect::from_center_size(mid, Vec2::splat(unit)),
            ui.scope_id().with((id, text(node, "id"), curve.out)),
            Sense::click(),
        );
        let label = text(node, if curve.out { "out_label" } else { "in_label" });
        r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), label));
        geometry_out.insert(
            format!(
                "{id}.edge.{}.{}",
                text(node, "id"),
                if curve.out { "out" } else { "in" }
            ),
            r.rect,
        );
        if r.has_focus() {
            painter.circle_stroke(mid, unit * 0.6, Stroke::new(unit * 0.08, p.accent));
        }
        if r.clicked() {
            activated = true;
            emit(events, id, "edge", json!({"id":node["id"],"out":curve.out}));
        }
    }
    if !activated && ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
        if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()).filter(|pointer| {
            !positions
                .iter()
                .any(|p| p.at.distance(*pointer) < p.radius * 1.25 || p.label.contains(*pointer))
        }) {
            let nearest = curves
                .iter()
                .map(|c| {
                    (
                        c,
                        c.points
                            .iter()
                            .map(|p| p.distance(pointer))
                            .fold(f32::INFINITY, f32::min),
                    )
                })
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((curve, distance)) = nearest.filter(|(_, d)| *d < unit * 0.55) {
                let _ = distance;
                let node = nodes.iter().find(|n| n["id"] == curve.peer).unwrap();
                emit(events, id, "edge", json!({"id":node["id"],"out":curve.out}));
            }
        }
    }
    let frame = ui.input(|i| i.time);
    let (shapes, moving, next) = ui.ctx().data_mut(|d| {
        let motion = d.get_temp_mut_or_default::<Motion>(ui.scope_id().with((id, "motion")));
        motion.feed(value, frame);
        motion.particles.retain(|pt| {
            pt.event["kind"] == "ripple" || nodes.iter().any(|n| n["id"] == pt.event["peer"])
        });
        let mut shapes = Vec::new();
        let mut moving = false;
        let mut next: Option<f64> = None;
        for pt in &motion.particles {
            let e = &pt.event;
            let duration = e["duration_s"].as_f64().unwrap_or(1.1);
            let k = (frame - pt.start) / duration;
            if k < 0. {
                next = Some(next.unwrap_or(f64::INFINITY).min(pt.start));
                continue;
            }
            if k >= 1. {
                continue;
            }
            moving = true;
            let colour = hue(text(e, "hue"), p);
            if e["kind"] == "ripple" {
                shapes.push(Shape::circle_stroke(
                    rect.center(),
                    unit * (1.3 + k as f32 * 1.3),
                    Stroke::new(unit * 0.1, colour.gamma_multiply((1. - k) as f32 * 0.8)),
                ));
            } else if let Some(node) = nodes.iter().find(|n| n["id"] == e["peer"]) {
                if let Some(curve) = geometry.curve(text(node, "id"), e["out"] == true) {
                    let eased = if k < 0.5 {
                        2. * k * k
                    } else {
                        1. - (-2. * k + 2.).powi(2) / 2.
                    };
                    let at = point_at(&curve.points, eased as f32);
                    let radius = unit
                        * (0.16 + ((1. + u64_value(&e["size"]) as f32).log2() / 50.).min(0.27));
                    let alpha = (k * 10.).min((1. - k) * 10.).min(1.) as f32;
                    shapes.push(Shape::circle_filled(
                        at,
                        radius,
                        colour.gamma_multiply(alpha),
                    ));
                    shapes.push(Shape::circle_filled(
                        at,
                        radius * 0.45,
                        p.text.gamma_multiply(alpha),
                    ));
                }
            }
        }
        (shapes, moving, next)
    });
    painter.extend(shapes);
    if moving && ui.is_visible() {
        ui.ctx().request_repaint();
    } else if ui.is_visible() {
        if let Some(next) = next {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64((next - frame).max(0.)));
        }
    }
    for (i, node) in nodes.iter().enumerate() {
        let Placement {
            at,
            radius,
            label: plate,
        } = positions[i];
        let hit = Rect::from_center_size(at, Vec2::splat(radius * 2.5)).union(plate);
        let r = ui.interact(hit, ui.scope_id().with((id, text(node, "id"))), Sense::click());
        r.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::Button,
                ui.is_enabled(),
                text(node, "accessible"),
            )
        });
        geometry_out.insert(format!("{id}.node.{}", text(node, "id")), hit);
        if (r.hovered() || r.has_focus()) && value["inspected"] != node["id"] {
            emit(events, id, "inspect", node["id"].clone());
        }
        let border = if node["failed"] == true {
            p.danger
        } else if r.has_focus() || r.hovered() {
            p.accent
        } else {
            p.field_border
        };
        painter.circle_filled(at, radius, p.card);
        painter.circle_stroke(
            at,
            radius,
            Stroke::new(unit * if r.has_focus() { 0.16 } else { 0.1 }, border),
        );
        painter.circle_filled(
            at,
            radius * 0.25,
            if node["primary"] == true {
                p.accent
            } else {
                p.text_dim
            },
        );
        if r.clicked() {
            emit(events, id, "node", node["id"].clone());
        }
    }
    for (placement, text) in positions.iter().zip(&geometry.labels) {
        let plate = placement.label;
        painter.rect_filled(plate, chrome.style.square_radius, p.card);
        let at = egui::pos2(
            plate.center().x - text.name.size().x * 0.5,
            plate.top() + padding,
        );
        let ip_y = at.y + text.name.size().y + padding;
        painter.galley(at, text.name.clone(), p.text);
        if let Some(ip) = &text.ip {
            painter.galley(
                egui::pos2(plate.center().x - ip.size().x * 0.5, ip_y),
                ip.clone(),
                p.text_dim,
            );
        }
    }
    response
}
