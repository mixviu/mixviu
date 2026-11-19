// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native presentation of Mix-authored, literal OSD scene data.
//! Policy, lifetimes and Bus actions remain in the app's reviewed Mix behaviour.
use super::*;
use crate::layer::{LayerOptions, LayerPresentation, LayerProgram, SurfaceInfo};

struct Item {
    epoch: u64,
    origin: Instant,
    initial_elapsed: f64,
    scene: Value,
    view: SceneView,
}

struct Program {
    bundle: Option<Bundle>,
    host: Option<Host>,
    logic: egui::Context,
    items: BTreeMap<(String, String), Item>,
    presets: BTreeMap<String, motion::Preset>,
    scene_source: String,
}

fn output_matches(name: &str, target: &str) -> bool {
    // Window observations use the public output slug; explicit anchors may
    // use the connector name reported by Wayland.
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    name == target || format!("o_{slug}") == target
}

fn pointer_region(rect: egui::Rect) -> (i32, i32, i32, i32) {
    let x = rect.min.x.floor() as i32;
    let y = rect.min.y.floor() as i32;
    (
        x,
        y,
        rect.max.x.ceil() as i32 - x,
        rect.max.y.ceil() as i32 - y,
    )
}

/// Resolve against the same projected window observations the Bus reports.
fn geometry(item: &Value, model: &Value, info: &SurfaceInfo) -> Option<egui::Rect> {
    let number = |value: &Value, default: f32| value.as_f64().map(|v| v as f32).unwrap_or(default);
    let scene = &item["scene"];
    let width = number(&scene["width"], 480.);
    let height = number(&scene["height"], 80.);
    let anchor = &item["anchor"];
    let output = info.output.as_deref().unwrap_or_default();
    let (x, y) = if let Some(id) = anchor["window"]["id"].as_u64() {
        let window = &model["windows"][id.to_string()];
        if window["generation"] != anchor["window"]["generation"]
            || window["visible"] != true
            || window["minimized"] == true
            || !output_matches(output, window["output"].as_str()?)
        {
            return None;
        }
        (
            number(&window["x"], 0.) + number(&anchor["offset"]["x"], 0.),
            number(&window["y"], 0.) + number(&anchor["offset"]["y"], 0.),
        )
    } else {
        if !output_matches(output, anchor["output"].as_str()?) {
            return None;
        }
        let edge = anchor["edge"].as_str()?;
        let margin = &anchor["margin"];
        let x = if edge.contains("left") {
            number(&margin["left"], 16.)
        } else if edge.contains("right") {
            info.width as f32 - width - number(&margin["right"], 16.)
        } else {
            (info.width as f32 - width) / 2.
        };
        let y = if edge.contains("top") {
            number(&margin["top"], 16.)
        } else if edge.contains("bottom") {
            info.height as f32 - height - number(&margin["bottom"], 16.)
        } else {
            (info.height as f32 - height) / 2.
        };
        (x, y)
    };
    Some(egui::Rect::from_min_size(
        egui::pos2(x, y),
        egui::vec2(width, height),
    ))
}

impl Program {
    fn snapshots(&self) -> Vec<Value> {
        self.host
            .as_ref()
            .and_then(|host| host.model["items"].as_array())
            .cloned()
            .unwrap_or_default()
    }
    fn motion(&self, item: &Value, elapsed: f64) -> (motion::Values, bool) {
        let Some(preset) = item["motion"]
            .as_str()
            .and_then(|name| self.presets.get(name))
        else {
            return (motion::Values::default(), false);
        };
        let duration = preset.timing.duration;
        let t = if item["phase"] == "exit" {
            (duration - elapsed).max(0.)
        } else {
            elapsed.min(duration)
        };
        (preset.sample(t), elapsed < duration)
    }
}

impl LayerProgram for Program {
    fn set_waker(&mut self, wake: crate::runtime::Wake) {
        if let Some(bundle) = self.bundle.take() {
            // Bus work must wake the event loop even with no output or while
            // every surface is detached. egui coalesces repaint requests
            // until its next pass, which an empty OSD never draws.
            let mut host = create(bundle, wake);
            // Per-item views present the model, not Host's root view.
            host.headless = true;
            self.host = Some(host);
        }
    }

    fn wake_deadline(&self) -> Option<Instant> {
        self.host.as_ref().and_then(|host| host.timers.deadline())
    }

    fn service(&mut self) {
        if let Some(host) = &mut self.host {
            host.pump(&self.logic);
        }
    }

    fn setup(&mut self, ctx: &egui::Context) {
        crate::icons::install(ctx);
        toolkit::install(ctx, &toolkit::Theme::embedded());
    }

    fn presentation(&mut self, _ctx: &egui::Context, info: &SurfaceInfo) -> LayerPresentation {
        let Some(host) = &self.host else {
            return LayerPresentation {
                visible: false,
                ..Default::default()
            };
        };
        let mut visible = false;
        let mut regions = Vec::new();
        for item in self.snapshots() {
            if let Some(rect) = geometry(&item, &host.model, info) {
                visible = true;
                if item["pointer_input"] == true {
                    let key = (
                        info.output.clone().unwrap_or_default(),
                        item["id"].as_str().unwrap_or_default().to_owned(),
                    );
                    let elapsed = self
                        .items
                        .get(&key)
                        .map(|entry| entry.initial_elapsed + entry.origin.elapsed().as_secs_f64())
                        .unwrap_or_else(|| item["elapsed_ms"].as_f64().unwrap_or(0.) / 1000.);
                    let (values, _) = self.motion(&item, elapsed);
                    let rect = egui::Rect::from_center_size(
                        rect.center()
                            + egui::vec2(
                                values.get(motion::Property::Tx) as f32,
                                values.get(motion::Property::Ty) as f32,
                            ),
                        rect.size() * values.get(motion::Property::Scale) as f32,
                    );
                    regions.push(pointer_region(rect));
                }
            }
        }
        LayerPresentation {
            visible,
            input_regions: Some(regions),
            exclusive_zone: Some(-1),
            ..Default::default()
        }
    }

    fn strict_frame_callbacks(&self) -> bool {
        true
    }

    fn show(&mut self, ui: &mut egui::Ui, info: &SurfaceInfo) {
        let model = self
            .host
            .as_ref()
            .map(|h| h.model.clone())
            .unwrap_or(Value::Null);
        let output = info.output.clone().unwrap_or_default();
        let snapshots = self.snapshots();
        let keep: BTreeSet<String> = snapshots
            .iter()
            .filter_map(|i| i["id"].as_str().map(str::to_owned))
            .collect();
        self.items
            .retain(|(out, id), _| out != &output || keep.contains(id));
        for snapshot in snapshots {
            let Some(rect) = geometry(&snapshot, &model, info) else {
                continue;
            };
            let id = snapshot["id"].as_str().unwrap_or_default().to_owned();
            let key = (output.clone(), id.clone());
            let epoch = snapshot["epoch"].as_u64().unwrap_or(0);
            if !self.items.contains_key(&key) {
                let view = SceneView::with_model(
                    &self.scene_source,
                    json!({"text":"","title":"","kind":"caption","text_size":22}),
                )
                .expect("reviewed OSD scene");
                self.items.insert(
                    key.clone(),
                    Item {
                        epoch,
                        origin: Instant::now(),
                        initial_elapsed: snapshot["elapsed_ms"].as_f64().unwrap_or(0.) / 1000.,
                        scene: Value::Null,
                        view,
                    },
                );
            }
            let entry = self.items.get_mut(&key).unwrap();
            if entry.epoch != epoch {
                entry.epoch = epoch;
                entry.origin = Instant::now();
                entry.initial_elapsed = snapshot["elapsed_ms"].as_f64().unwrap_or(0.) / 1000.;
            }
            let elapsed = entry.initial_elapsed + entry.origin.elapsed().as_secs_f64();
            if entry.scene != snapshot["scene"] {
                entry.scene = snapshot["scene"].clone();
                entry
                    .view
                    .apply_model(entry.scene.clone())
                    .expect("validated literal scene model");
            }
            let (values, moving) = self.motion(&snapshot, elapsed);
            let translate = egui::vec2(
                values.get(motion::Property::Tx) as f32,
                values.get(motion::Property::Ty) as f32,
            );
            let scale = values.get(motion::Property::Scale) as f32;
            let draw_rect = rect;
            let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::unique(("osd", &id)));
            let mut child =
                ui.new_child(egui::UiBuilder::new().max_rect(draw_rect).layer_id(layer));
            child.set_opacity(values.get(motion::Property::Opacity) as f32);
            child.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
            child.painter().rect_filled(
                draw_rect,
                8.,
                egui::Color32::from_rgba_unmultiplied(20, 24, 32, 240),
            );
            let content = draw_rect.shrink(12.);
            let mut content_ui = child.new_child(egui::UiBuilder::new().max_rect(content));
            let entry = self.items.get_mut(&key).unwrap();
            let _ = entry.view.show(&mut content_ui);
            #[allow(deprecated)]
            ui.ctx().transform_layer_shapes(
                layer,
                egui::emath::TSTransform {
                    scaling: scale,
                    translation: rect.center().to_vec2() * (1. - scale) + translate,
                },
            );
            if moving {
                ui.ctx().request_repaint_after(Duration::from_millis(16));
            }
        }
    }

    fn should_close(&self) -> bool {
        if self.host.as_ref().is_some_and(|h| {
            h.require_bus
                && h.bus
                    .registration_failed
                    .load(std::sync::atomic::Ordering::Acquire)
        }) {
            std::process::exit(1);
        }
        self.host.as_ref().is_some_and(|h| {
            h.bus
                .close_requested
                .load(std::sync::atomic::Ordering::Acquire)
        })
    }

    fn removed(&mut self, _ctx: &egui::Context, info: &SurfaceInfo) {
        let output = info.output.clone().unwrap_or_default();
        self.items.retain(|(out, _), _| out != &output);
    }
}

pub(super) fn run(bundle: Bundle) {
    let mut options = LayerOptions::background("viu-osd");
    options.layer = crate::layer::Layer::Overlay;
    let scene_source = bundle.scene.clone();
    let program = Program {
        bundle: Some(bundle),
        host: None,
        logic: egui::Context::default(),
        items: BTreeMap::new(),
        presets: motion::presets()
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect(),
        scene_source,
    };
    if let Err(error) = crate::layer::run(options, program) {
        eprintln!("viu OSD: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behaviour_and_expiry_work_without_any_output_or_egui_pass() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/osd");
        let mut bundle = Bundle::load(&root, &[]).unwrap();
        bundle.manifest.bus = false;
        bundle.manifest.require_bus = false;
        let mut program = Program {
            scene_source: bundle.scene.clone(),
            bundle: Some(bundle),
            host: None,
            logic: egui::Context::default(),
            items: BTreeMap::new(),
            presets: BTreeMap::new(),
        };
        program.set_waker(Arc::new(|| {}));
        let until = Instant::now() + Duration::from_secs(10);
        while !program.host.as_ref().unwrap().ready {
            program.service();
            assert!(Instant::now() < until, "OSD logic startup timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        program.host.as_mut().unwrap().dispatch(
            "osd.show".into(),
            json!({"id":"no-output","scene":{"kind":"caption","text":"Expires without a surface"},
                "anchor":{"output":"absent-output","edge":"bottom"},"duration_ms":100}),
            BTreeMap::new(),
            Some(tx),
        );
        loop {
            program.service();
            if let Ok((rc, body)) = rx.try_recv() {
                assert_eq!(rc, 0, "{body}");
                break;
            }
            assert!(Instant::now() < until, "OSD show timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(program.wake_deadline().is_some());
        loop {
            program.service();
            let host = program.host.as_ref().unwrap();
            assert!(host.error.is_empty(), "{}", host.error);
            if host.model["items"].as_array().unwrap().is_empty() {
                break;
            }
            assert!(Instant::now() < until, "OSD expiry timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(program.wake_deadline().is_none(), "empty OSD must disarm");
    }

    #[test]
    fn shipped_bundle_loads_and_renders_each_literal_scene_kind() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/osd");
        let bundle = Bundle::load(&root, &[]).expect("shipped OSD bundle and Bus surface validate");
        assert_eq!(bundle.manifest.surface, "osd");
        for (kind, title, size) in [
            ("title", "", 30),
            ("caption", "", 22),
            ("card", "Card title", 22),
        ] {
            let mut view = SceneView::with_model(
                &bundle.scene,
                json!({"kind":kind,"title":title,"text":"Literal OSD text","text_size":size}),
            )
            .expect("shipped scene accepts item model");
            let context = egui::Context::default();
            toolkit::install(&context, &toolkit::Theme::embedded());
            let output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(480., 160.),
                    )),
                    ..Default::default()
                },
                |ui| {
                    let _ = view.show(ui);
                },
            );
            assert!(!output.shapes.is_empty(), "{kind} scene draws content");
            output.drop_without_applying_deltas();
        }
    }

    #[test]
    fn pointer_region_covers_fractional_far_edges() {
        let rect = egui::Rect::from_min_max(egui::pos2(10.8, -3.2), egui::pos2(20.2, 7.1));
        assert_eq!(pointer_region(rect), (10, -4, 11, 12));
    }

    #[test]
    fn window_anchor_hides_on_minimise_or_generation_change() {
        let item = json!({"scene":{"width":300,"height":80},"anchor":{"window":{"id":1,"generation":7},"offset":{"x":12,"y":24}}});
        let info = SurfaceInfo {
            output: Some("HEADLESS-1".into()),
            width: 1920,
            height: 1080,
            scale: 1.,
        };
        let mut model = json!({"windows":{"1":{"generation":7,"visible":true,"minimized":false,"output":"HEADLESS-1","x":100,"y":200}}});
        assert_eq!(
            geometry(&item, &model, &info).unwrap().min,
            egui::pos2(112., 224.)
        );
        model["windows"]["1"]["minimized"] = json!(true);
        assert!(geometry(&item, &model, &info).is_none());
        model["windows"]["1"]["minimized"] = json!(false);
        model["windows"]["1"]["generation"] = json!(8);
        assert!(geometry(&item, &model, &info).is_none());
    }
}
