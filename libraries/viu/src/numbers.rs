// SPDX-License-Identifier: MIT OR Apache-2.0
//! Presentation of a scalar track. The motion library owns interpolation.
use motion::{Curve, Fill, Keyframe, Preset, Property, Timing, Track};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct Numbers(BTreeMap<String, Number>);
struct Number {
    stamp: Value,
    preset: Preset,
    seen: bool,
    changing: bool,
}
impl Numbers {
    pub fn begin(&mut self) {
        for n in self.0.values_mut() {
            n.seen = false;
        }
    }
    pub fn end(&mut self) {
        self.0.retain(|_, n| n.seen);
    }
    pub fn text(&mut self, ui: &egui::Ui, key: &str, spec: &Value, enabled: bool) -> String {
        let now = ui.input(|i| i.time);
        let n = self
            .0
            .entry(key.into())
            .or_insert_with(|| number(spec, now));
        if n.stamp != *spec {
            *n = number(spec, now);
        }
        n.seen = true;
        let finish = n.preset.timing.start() + n.preset.timing.duration;
        let animated = enabled && ui.style().animation_time != 0.0 && n.changing;
        let at = if animated { now } else { finish };
        let value = n.preset.sample(at).get(Property::Scalar);
        if animated && now < finish {
            if now < n.preset.timing.start() {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_secs_f64(
                        n.preset.timing.start() - now,
                    ));
            } else {
                ui.ctx().request_repaint();
            }
        }
        let raw = format!("{:.0}", value.round());
        if spec["group"] == false {
            return raw;
        }
        let (sign, digits) = raw
            .strip_prefix('-')
            .map_or(("", raw.as_str()), |n| ("-", n));
        let mut text = String::from(sign);
        for (i, digit) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                text.push(',');
            }
            text.push(digit);
        }
        text
    }
}
fn number(spec: &Value, now: f64) -> Number {
    let from = spec["from"].as_f64().unwrap_or(0.);
    let to = spec["to"].as_f64().expect("validated number target");
    let preset = Preset {
        name: "number".into(),
        timing: Timing {
            beat_start: now,
            delay: spec["delay_ms"].as_f64().unwrap_or(200.) / 1000.,
            duration: spec["ms"].as_f64().unwrap_or(1400.) / 1000.,
            fill: Fill::Both,
            ..Default::default()
        },
        tracks: vec![Track {
            property: Property::Scalar,
            keyframes: vec![
                Keyframe {
                    offset: 0.,
                    value: from,
                    easing: Curve::Bezier {
                        x1: 1. / 3.,
                        y1: 1.,
                        x2: 2. / 3.,
                        y2: 1.,
                    },
                },
                Keyframe {
                    offset: 1.,
                    value: to,
                    easing: Curve::Linear,
                },
            ],
        }],
    };
    Number {
        stamp: spec.clone(),
        preset,
        seen: true,
        changing: from != to,
    }
}
