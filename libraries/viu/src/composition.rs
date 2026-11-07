// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded declarative native composition. Mix owns the tree, labels, source
//! interpretation, plots, geometry, motion vectors and animation leases.
use crate::raster::Rasters;
use egui::{Color32, Ui};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
#[derive(Clone, Deserialize)]
pub struct Block {
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub tone: usize,
    pub glow: f32,
    pub label: String,
    #[serde(default)]
    pub motion: [f32; 3],
    #[serde(default)]
    pub label_visible: bool,
}
pub fn validate(value: &Value) -> Result<(), String> {
    fn visit(nodes: &Value, depth: usize, remaining: &mut usize) -> Result<(), String> {
        let nodes = nodes
            .as_array()
            .ok_or("composition nodes must be an array")?;
        if depth > 16 || nodes.len() > *remaining {
            return Err("composition tree exceeds budget".into());
        }
        *remaining -= nodes.len();
        for n in nodes {
            let kind = n["kind"].as_str().ok_or("composition node kind required")?;
            if ![
                "clear", "frame", "panel", "centered", "heading", "label", "strong", "progress",
                "columns", "scroll", "space", "plot", "world",
            ]
            .contains(&kind)
            {
                return Err("unknown composition node".into());
            }
            if n["text"].as_str().is_some_and(|s| s.len() > 65536) {
                return Err("composition text exceeds budget".into());
            }
            if kind == "world" {
                let blocks: Vec<Block> =
                    serde_json::from_value(n["blocks"].clone()).map_err(|e| e.to_string())?;
                if blocks.len() > 256
                    || blocks.iter().any(|b| {
                        b.tone > 3
                            || !b.glow.is_finite()
                            || b.position
                                .iter()
                                .chain(&b.size)
                                .chain(&b.motion)
                                .any(|x| !x.is_finite())
                            || b.size.iter().any(|x| *x < 0.)
                            || b.label.len() > 65536
                    })
                {
                    return Err("world blocks exceed budget".into());
                }
            }
            if kind == "columns" {
                let columns = n["columns"]
                    .as_array()
                    .filter(|a| !a.is_empty() && a.len() <= 16)
                    .ok_or("invalid composition columns")?;
                for c in columns {
                    visit(c, depth + 1, remaining)?;
                }
            }
            if !n["children"].is_null() {
                visit(&n["children"], depth + 1, remaining)?;
            }
            if kind == "plot" {
                let series = n["series"]
                    .as_array()
                    .filter(|a| a.len() <= 256)
                    .ok_or("invalid plot series")?;
                for s in series {
                    if !["line", "points", "vline"].contains(&s["kind"].as_str().unwrap_or("")) {
                        return Err("unknown plot series".into());
                    }
                    if s["kind"] != "vline" {
                        let points: Vec<[f64; 2]> = serde_json::from_value(s["points"].clone())
                            .map_err(|e| e.to_string())?;
                        if points.len() > 4096 || points.iter().flatten().any(|v| !v.is_finite()) {
                            return Err("invalid plot points".into());
                        }
                    }
                }
            }
        }
        Ok(())
    }
    visit(&value["nodes"], 0, &mut 2048)
}
#[derive(Default)]
pub struct Composition {
    observer: Option<crate::raster::RepaintLease>,
    worlds: BTreeMap<String, crate::cuboids::Cache>,
    stamp: Value,
    started: Option<Instant>,
    fallback: bool,
}
fn text(n: &Value) -> &str {
    n["text"].as_str().unwrap_or("")
}
fn colour(ui: &Ui, role: &Value) -> Color32 {
    match role.as_str().unwrap_or("") {
        "warning" => ui.visuals().warn_fg_color,
        "error" => ui.visuals().error_fg_color,
        "accent" => ui.visuals().selection.bg_fill,
        "link" => ui.visuals().hyperlink_color,
        _ => ui.visuals().text_color(),
    }
}
impl Composition {
    pub fn draw(&mut self, ui: &mut Ui, value: &Value, rasters: &Rasters) -> egui::Response {
        if self.observer.is_none() {
            self.observer = Some(rasters.observe(ui.ctx()));
        }
        let transition = &value["transition"];
        if self.stamp != *transition {
            self.stamp = transition.clone();
            self.started = Some(Instant::now());
        }
        let duration = transition["duration_ms"]
            .as_f64()
            .unwrap_or(0.)
            .clamp(0., 2000.);
        let progress = if duration == 0. {
            1.
        } else {
            self.started.map_or(1., |t| {
                (t.elapsed().as_secs_f32() * 1000. / duration as f32).min(1.)
            })
        };
        let response = ui
            .scope(|ui| self.nodes(ui, &value["nodes"], rasters, progress))
            .response;
        if progress < 1. {
            ui.ctx().request_repaint_after(Duration::from_millis(
                transition["frame_ms"].as_u64().unwrap_or(100).max(100),
            ));
        }
        response
    }
    fn nodes(&mut self, ui: &mut Ui, nodes: &Value, rasters: &Rasters, progress: f32) {
        for (index, n) in nodes.as_array().into_iter().flatten().enumerate() {
            if n["when"] == "world-fallback" && !self.fallback {
                continue;
            }
            let id = n["id"]
                .as_str()
                .map_or_else(|| format!("node-{index}"), str::to_owned);
            match n["kind"].as_str().unwrap_or("") {
                "clear" => {
                    ui.painter()
                        .rect_filled(ui.max_rect(), 0., ui.visuals().panel_fill);
                }
                "world" => {
                    let blocks: Vec<Block> =
                        serde_json::from_value(n["blocks"].clone()).expect("validated blocks");
                    self.fallback = n["disabled"] == true
                        || !self
                            .worlds
                            .entry(id.clone())
                            .or_default()
                            .draw(ui, &blocks, progress, rasters);
                    if self.fallback {
                        self.worlds.remove(&id);
                    }
                }
                "frame" => {
                    let frame = if n["style"] == "popup" {
                        egui::Frame::popup(ui.style())
                    } else {
                        egui::Frame::new().fill(ui.visuals().error_fg_color.gamma_multiply(0.25))
                    };
                    frame.show(ui, |ui| self.nodes(ui, &n["children"], rasters, progress));
                }
                "panel" => {
                    egui::CentralPanel::default()
                        .show(ui, |ui| self.nodes(ui, &n["children"], rasters, progress));
                }
                "centered" => {
                    ui.vertical_centered(|ui| self.nodes(ui, &n["children"], rasters, progress));
                }
                "heading" => {
                    ui.heading(text(n));
                }
                "strong" => {
                    ui.strong(text(n));
                }
                "label" => {
                    let label = egui::RichText::new(text(n)).color(colour(ui, &n["colour"]));
                    let label = if n["strong"] == true {
                        label.strong()
                    } else {
                        label
                    };
                    let mut label = egui::Label::new(label);
                    if n["truncate"] == true {
                        label = label.truncate();
                    }
                    ui.add(label);
                }
                "progress" => {
                    ui.add(
                        egui::ProgressBar::new(
                            n["value"].as_f64().unwrap_or(0.).clamp(0., 1.) as f32
                        )
                        .fill(colour(ui, &n["colour"])),
                    );
                }
                "columns" => {
                    let columns = n["columns"].as_array().expect("validated columns");
                    ui.columns(columns.len(), |uis| {
                        for (ui, children) in uis.iter_mut().zip(columns) {
                            self.nodes(ui, children, rasters, progress);
                        }
                    });
                }
                "scroll" => {
                    egui::ScrollArea::vertical()
                        .id_salt(&id)
                        .max_height(n["max_height"].as_f64().unwrap_or(100.) as f32)
                        .show(ui, |ui| self.nodes(ui, &n["children"], rasters, progress));
                }
                "space" => {
                    if let Some(fraction) = n["fraction"].as_f64() {
                        ui.add_space(ui.available_height() * fraction as f32);
                        continue;
                    }
                    ui.add_space(if let Some(bottom) = n["bottom"].as_f64() {
                        (ui.available_height() - bottom as f32).max(0.)
                    } else {
                        n["height"].as_f64().unwrap_or(0.) as f32
                    });
                }
                "plot" => {
                    let height = n["height"].as_f64().map(|v| v as f32).unwrap_or_else(|| {
                        (ui.available_height()
                            - n["height_available_minus"].as_f64().unwrap_or(0.) as f32)
                            .max(80.)
                    });
                    let mut plot = egui_plot::Plot::new(&id)
                        .height(height)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false);
                    for x in n["include_x"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_f64)
                    {
                        plot = plot.include_x(x);
                    }
                    for y in n["include_y"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_f64)
                    {
                        plot = plot.include_y(y);
                    }
                    if n["legend"] == true {
                        plot = plot.legend(egui_plot::Legend::default());
                    }
                    if let Some(label) = n["x_axis"].as_str() {
                        plot = plot.x_axis_label(label);
                    }
                    if let Some(label) = n["y_axis"].as_str() {
                        plot = plot.y_axis_label(label);
                    }
                    let palette = [
                        ui.visuals().selection.bg_fill,
                        ui.visuals().warn_fg_color,
                        ui.visuals().hyperlink_color,
                        ui.visuals().error_fg_color,
                        ui.visuals().text_color(),
                    ];
                    plot.show(ui, |p| {
                        for s in n["series"].as_array().into_iter().flatten() {
                            let name = s["name"].as_str().unwrap_or("");
                            if s["kind"] == "vline" {
                                p.vline(egui_plot::VLine::new(name, s["x"].as_f64().unwrap_or(0.)));
                                continue;
                            }
                            let points: Vec<[f64; 2]> = serde_json::from_value(s["points"].clone())
                                .expect("validated points");
                            if s["kind"] == "line" {
                                p.line(egui_plot::Line::new(name, points));
                            } else {
                                let mut points = egui_plot::Points::new(name, points)
                                    .radius(s["radius"].as_f64().unwrap_or(3.) as f32);
                                if !s["colour"].is_null() {
                                    points = points.color(
                                        palette[match s["colour"].as_str().unwrap_or("") {
                                            "accent" => 0,
                                            "warning" => 1,
                                            "link" => 2,
                                            "error" => 3,
                                            _ => 4,
                                        }],
                                    );
                                }
                                p.points(points);
                                if let Some(label) = s["label"].as_str() {
                                    let at = &s["label_at"];
                                    p.text(egui_plot::Text::new(
                                        name,
                                        egui_plot::PlotPoint::new(
                                            at[0].as_f64().unwrap_or(0.),
                                            at[1].as_f64().unwrap_or(0.),
                                        ),
                                        label,
                                    ));
                                }
                            }
                        }
                    });
                }
                _ => {}
            }
        }
    }
}
