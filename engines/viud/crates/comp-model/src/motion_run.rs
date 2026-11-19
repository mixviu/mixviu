// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! `comp.motion.run`: one Bus call carries a whole move (plan rule: one call,
//! one complex move). Phase 3 accepts exactly one target, a pointer or a
//! window.
//!
//! ```text
//! comp.motion.run {targets: [{pointer: {
//!     output?: "name",
//!     path: [{x, y}, ...],        // 1..=64 output-local points
//!     duration_ms?: 0..=10000,    // default 400
//!     curve?: "linear" | "ease" | "ease-in" | "ease-out" | "ease-in-out"
//!           | "min-jerk" | [x1, y1, x2, y2]   // default "ease-in-out"
//! }}]}
//! comp.motion.run {targets: [{window: {
//!     id, generation,
//!     to?: {x?, y?, w?, h?},      // global logical; absent fields stay put
//!     tracks?: [{property, keyframes}], // opacity, scale, tx, ty
//!     preset?: "pop",            // shared table; exclusive with tracks
//!     duration_ms?, curve?        // curve also accepts shared easing objects
//! }}]}
//! ```
//!
//! A pointer glide starts at `path[0]` (the pointer is placed there at once)
//! and visits the rest in order. A window travels from where it is to `to`.
//! The reply waits until the motion ends.

use std::time::Duration;

use serde_json::Value;

use motion::{CubicBezier, Easing};

use crate::reply::ControlReply;
use crate::request::{args_object, invalid_argument, present};

/// Longest motion a call may ask for (see the plan's VT decision).
pub const MAX_DURATION_MS: u64 = 10_000;
/// Default motion length when `duration_ms` is absent.
pub const DEFAULT_DURATION_MS: u64 = 400;
/// Most points one pointer glide may visit.
pub const MAX_PATH_POINTS: usize = 64;
/// Slack the reply gets beyond the motion itself.
const REPLY_MARGIN: Duration = Duration::from_secs(1);

/// One target of `comp.motion.run`.
#[derive(Clone, Debug, PartialEq)]
pub enum MotionTarget {
    Pointer {
        output: Option<String>,
        path: Vec<(f64, f64)>,
        duration: Duration,
        easing: Easing,
    },
    Window {
        id: u64,
        generation: u64,
        /// Target `x, y, w, h` in global logical coordinates; `None` keeps
        /// the window's current value for that field.
        to: [Option<f64>; 4],
        duration: Duration,
        easing: Easing,
    },
    /// Additive visual-track form; the original geometry variant stays valid.
    WindowTracks {
        id: u64,
        generation: u64,
        to: [Option<f64>; 4],
        duration: Duration,
        easing: Easing,
        visual: motion::Preset,
    },
}

/// A parsed `comp.motion.run`.
#[derive(Clone, Debug, PartialEq)]
pub struct MotionSpec {
    pub targets: Vec<MotionTarget>,
}

impl MotionSpec {
    /// The longest any target runs, plus the reply margin.
    pub fn budget(&self) -> Duration {
        self.targets
            .iter()
            .map(|target| match target {
                MotionTarget::Pointer { duration, .. }
                | MotionTarget::Window { duration, .. }
                | MotionTarget::WindowTracks { duration, .. } => *duration,
            })
            .max()
            .unwrap_or_default()
            + REPLY_MARGIN
    }
}

/// Parse the command's arguments.
pub fn parse(args: &Value) -> Result<MotionSpec, ControlReply> {
    let empty = serde_json::Map::new();
    let object = args_object(args, &empty, &["targets"])?;
    let targets = match present(object, "targets") {
        Some(Value::Array(items)) if items.len() == 1 => items,
        _ => {
            return Err(invalid_argument(
                "targets",
                "array of exactly one target",
                "one target per call in this phase",
            ));
        }
    };
    let target = parse_target(&targets[0])?;
    Ok(MotionSpec {
        targets: vec![target],
    })
}

const POINTER_FIELDS: &[&str] = &["output", "path", "duration_ms", "curve"];
const WINDOW_FIELDS: &[&str] = &[
    "id",
    "generation",
    "to",
    "duration_ms",
    "curve",
    "tracks",
    "preset",
];
const TO_FIELDS: &[&str] = &["x", "y", "w", "h"];

fn parse_target(value: &Value) -> Result<MotionTarget, ControlReply> {
    let object = match value {
        Value::Object(object) if object.len() == 1 => object,
        _ => {
            return Err(invalid_argument(
                "targets[0]",
                "object with one key: pointer or window",
                "{pointer: {...}} or {window: {...}}",
            ));
        }
    };
    match object.get("pointer") {
        Some(Value::Object(pointer)) => parse_pointer(pointer),
        Some(_) => Err(invalid_argument(
            "targets[0].pointer",
            "object",
            "{path, duration_ms?, curve?, output?}",
        )),
        None => match object.get("window") {
            Some(Value::Object(window)) => parse_window(window),
            _ => Err(invalid_argument(
                "targets[0]",
                "object with one key: pointer or window",
                "{pointer: {...}} or {window: {...}}",
            )),
        },
    }
}

fn parse_pointer(pointer: &serde_json::Map<String, Value>) -> Result<MotionTarget, ControlReply> {
    if pointer
        .keys()
        .any(|key| !POINTER_FIELDS.contains(&key.as_str()))
    {
        return Err(invalid_argument(
            "pointer",
            "only output, path, duration_ms and curve",
            "unknown field refused",
        ));
    }
    let output = match present(pointer, "output") {
        None => None,
        Some(Value::String(name)) if !name.is_empty() => Some(name.clone()),
        Some(_) => {
            return Err(invalid_argument(
                "pointer.output",
                "non-empty string",
                "output name",
            ));
        }
    };
    let path = match present(pointer, "path") {
        Some(Value::Array(points)) if (1..=MAX_PATH_POINTS).contains(&points.len()) => points
            .iter()
            .map(parse_point)
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(invalid_argument(
                "pointer.path",
                "array of 1..=64 {x, y} points",
                "output-local logical coordinates",
            ));
        }
    };
    let (duration, easing) = parse_timing(pointer)?;
    Ok(MotionTarget::Pointer {
        output,
        path,
        duration,
        easing,
    })
}

fn parse_window(window: &serde_json::Map<String, Value>) -> Result<MotionTarget, ControlReply> {
    if window
        .keys()
        .any(|key| !WINDOW_FIELDS.contains(&key.as_str()))
    {
        return Err(invalid_argument(
            "window",
            "only id, generation, to, duration_ms, curve, tracks and preset",
            "unknown field refused",
        ));
    }
    let id = window
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_argument("window.id", "integer", "window id"))?;
    let generation = window
        .get("generation")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_argument("window.generation", "integer", "window generation"))?;
    let visual = parse_visual(window)?;
    let empty_to = serde_json::Map::new();
    let to_object = match present(window, "to") {
        Some(Value::Object(to)) => to,
        None if visual.is_some() => &empty_to,
        _ => {
            return Err(invalid_argument(
                "window.to",
                "object with any of x, y, w, h",
                "global logical coordinates",
            ));
        }
    };
    if to_object
        .keys()
        .any(|key| !TO_FIELDS.contains(&key.as_str()))
    {
        return Err(invalid_argument(
            "window.to",
            "only x, y, w and h",
            "unknown field refused",
        ));
    }
    let mut to = [None; 4];
    for (slot, key) in to.iter_mut().zip(TO_FIELDS) {
        *slot = match to_object.get(*key) {
            None => None,
            Some(value) => Some(
                value
                    .as_f64()
                    .filter(|v| v.is_finite())
                    .ok_or_else(|| invalid_argument("window.to", "finite number", "logical px"))?,
            ),
        };
    }
    if to.iter().all(Option::is_none) && visual.is_none() {
        return Err(invalid_argument(
            "window.to",
            "at least one of x, y, w, h",
            "global logical coordinates",
        ));
    }
    if to[2].is_some_and(|w| w <= 0.0) || to[3].is_some_and(|h| h <= 0.0) {
        return Err(invalid_argument(
            "window.to",
            "w and h above zero",
            "logical px",
        ));
    }
    let (duration, easing) = parse_timing(window)?;
    if let Some(mut visual) = visual {
        visual.timing.duration = duration.as_secs_f64();
        return Ok(MotionTarget::WindowTracks {
            id,
            generation,
            to,
            duration,
            easing,
            visual,
        });
    }
    Ok(MotionTarget::Window {
        id,
        generation,
        to,
        duration,
        easing,
    })
}

/// `duration_ms` and `curve`, shared by both target kinds.
fn parse_timing(
    fields: &serde_json::Map<String, Value>,
) -> Result<(Duration, Easing), ControlReply> {
    let duration_ms = match present(fields, "duration_ms") {
        None => DEFAULT_DURATION_MS,
        Some(value) => value
            .as_u64()
            .filter(|ms| *ms <= MAX_DURATION_MS)
            .ok_or_else(|| invalid_argument("duration_ms", "integer 0..=10000", "milliseconds"))?,
    };
    let easing = match present(fields, "curve") {
        None => Easing::Bezier(CubicBezier::EASE_IN_OUT),
        Some(value) => parse_curve(value)?,
    };
    Ok((Duration::from_millis(duration_ms), easing))
}

fn parse_point(value: &Value) -> Result<(f64, f64), ControlReply> {
    let bad = || {
        invalid_argument(
            "pointer.path",
            "{x, y} with finite numbers",
            "output-local logical coordinates",
        )
    };
    let Value::Object(point) = value else {
        return Err(bad());
    };
    let x = point
        .get("x")
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite());
    let y = point
        .get("y")
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite());
    match (x, y) {
        (Some(x), Some(y)) => Ok((x, y)),
        _ => Err(bad()),
    }
}

fn parse_curve(value: &Value) -> Result<Easing, ControlReply> {
    let bad = || {
        invalid_argument(
            "curve",
            "a curve name, shared easing object or [x1, y1, x2, y2]",
            "linear | ease | ease-in | ease-out | ease-in-out | min-jerk",
        )
    };
    match value {
        Value::Object(_) => {
            let curve: motion::Curve = serde_json::from_value(value.clone()).map_err(|_| bad())?;
            curve.validate().map_err(|_| bad())?;
            Ok(Easing::Track(curve))
        }
        Value::String(name) => match name.as_str() {
            "linear" => Ok(Easing::Linear),
            "min-jerk" => Ok(Easing::MinJerk),
            "ease" => Ok(Easing::Bezier(CubicBezier::EASE)),
            "ease-in" => Ok(Easing::Bezier(CubicBezier::EASE_IN)),
            "ease-out" => Ok(Easing::Bezier(CubicBezier::EASE_OUT)),
            "ease-in-out" => Ok(Easing::Bezier(CubicBezier::EASE_IN_OUT)),
            _ => Err(bad()),
        },
        Value::Array(numbers) if numbers.len() == 4 => {
            let n: Vec<f64> = numbers
                .iter()
                .map(|v| v.as_f64().ok_or_else(bad))
                .collect::<Result<_, _>>()?;
            CubicBezier::new(n[0], n[1], n[2], n[3])
                .map(Easing::Bezier)
                .map_err(|_| bad())
        }
        _ => Err(bad()),
    }
}

/// The compositor supports this projection of the shared preset vocabulary.
pub fn window_property(property: motion::Property) -> bool {
    matches!(
        property,
        motion::Property::Opacity
            | motion::Property::Scale
            | motion::Property::Tx
            | motion::Property::Ty
    )
}

fn parse_visual(
    window: &serde_json::Map<String, Value>,
) -> Result<Option<motion::Preset>, ControlReply> {
    let bad = || {
        invalid_argument(
            "window.tracks",
            "1..=4 unique opacity, scale, tx, ty tracks or a preset name",
            "shared motion keyframes; no rotation",
        )
    };
    let mut visual = match (window.get("tracks"), window.get("preset")) {
        (None, None) => return Ok(None),
        (Some(tracks), None) => {
            let tracks: Vec<motion::Track> =
                serde_json::from_value(tracks.clone()).map_err(|_| bad())?;
            if tracks.is_empty()
                || tracks.len() > 4
                || tracks
                    .iter()
                    .any(|t| !window_property(t.property) || t.keyframes.len() > 64)
            {
                return Err(bad());
            }
            motion::Preset {
                name: "window".into(),
                timing: motion::Timing::default(),
                tracks,
            }
        }
        (None, Some(Value::String(name))) => {
            let mut preset = motion::presets()
                .into_iter()
                .find(|p| &p.name == name)
                .ok_or_else(bad)?;
            preset.tracks.retain(|t| window_property(t.property));
            if preset.tracks.is_empty() {
                return Err(bad());
            }
            preset
        }
        _ => return Err(bad()),
    };
    visual.timing = motion::Timing {
        fill: motion::Fill::Both,
        ..motion::Timing::default()
    };
    if let Some(value) = window.get("curve") {
        let easing = match parse_curve(value)? {
            Easing::Linear => motion::Curve::Linear,
            Easing::Track(curve) => curve,
            Easing::Bezier(_) | Easing::MinJerk => {
                // Named CSS curves and bezier arrays also have a shared Curve representation.
                curve_for_tracks(value)?
            }
        };
        for track in &mut visual.tracks {
            for frame in &mut track.keyframes {
                frame.easing = easing;
            }
        }
    }
    visual.validate().map_err(|_| bad())?;
    for track in &visual.tracks {
        if track.keyframes.iter().any(|f| match track.property {
            motion::Property::Opacity => !(0.0..=1.0).contains(&f.value),
            motion::Property::Scale => !(0.0..=64.0).contains(&f.value),
            _ => !(i32::MIN as f64..=i32::MAX as f64).contains(&f.value),
        }) {
            return Err(bad());
        }
    }
    Ok(Some(visual))
}

fn curve_for_tracks(value: &Value) -> Result<motion::Curve, ControlReply> {
    Ok(match value.as_str() {
        Some("ease") => motion::Curve::Ease,
        Some("ease-in") => motion::Curve::EaseIn,
        Some("ease-out") => motion::Curve::EaseOut,
        Some("ease-in-out") => motion::Curve::EaseInOut,
        _ => {
            let a = value.as_array().ok_or_else(|| {
                invalid_argument("curve", "shared easing", "min-jerk is geometry-only")
            })?;
            motion::Curve::Bezier {
                x1: a[0].as_f64().unwrap(),
                y1: a[1].as_f64().unwrap(),
                x2: a[2].as_f64().unwrap(),
                y2: a[3].as_f64().unwrap(),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ok(args: Value) -> MotionSpec {
        parse(&args).unwrap_or_else(|_| panic!("should parse: {args}"))
    }

    #[test]
    fn parses_a_full_pointer_glide() {
        let spec = ok(json!({"targets": [{"pointer": {
            "output": "o1",
            "path": [{"x": 10, "y": 20}, {"x": 300, "y": 40}],
            "duration_ms": 750,
            "curve": "min-jerk"
        }}]}));
        assert_eq!(
            spec.targets,
            vec![MotionTarget::Pointer {
                output: Some("o1".into()),
                path: vec![(10.0, 20.0), (300.0, 40.0)],
                duration: Duration::from_millis(750),
                easing: Easing::MinJerk,
            }]
        );
    }

    #[test]
    fn parses_a_window_move_and_resize() {
        let spec = ok(json!({"targets": [{"window": {
            "id": 7, "generation": 3,
            "to": {"x": 100, "w": 640.5, "h": 480},
            "duration_ms": 900, "curve": "ease-out"
        }}]}));
        assert_eq!(
            spec.targets,
            vec![MotionTarget::Window {
                id: 7,
                generation: 3,
                to: [Some(100.0), None, Some(640.5), Some(480.0)],
                duration: Duration::from_millis(900),
                easing: Easing::Bezier(CubicBezier::EASE_OUT),
            }]
        );
    }

    #[test]
    fn defaults_are_400ms_and_ease_in_out() {
        let spec = ok(json!({"targets": [{"pointer": {"path": [{"x": 1, "y": 2}]}}]}));
        let MotionTarget::Pointer {
            duration, easing, ..
        } = &spec.targets[0]
        else {
            panic!("pointer target");
        };
        assert_eq!(*duration, Duration::from_millis(DEFAULT_DURATION_MS));
        assert_eq!(*easing, Easing::Bezier(CubicBezier::EASE_IN_OUT));
    }

    #[test]
    fn custom_bezier_curve_is_accepted() {
        let spec = ok(json!({"targets": [{"pointer": {
            "path": [{"x": 0, "y": 0}], "curve": [0.05, 0.9, 0.1, 1.1]
        }}]}));
        let MotionTarget::Pointer { easing, .. } = &spec.targets[0] else {
            panic!("pointer target");
        };
        assert!(matches!(easing, Easing::Bezier(_)));
    }

    #[test]
    fn budget_is_the_motion_plus_margin() {
        let spec = ok(json!({"targets": [{"pointer": {
            "path": [{"x": 0, "y": 0}], "duration_ms": 2000
        }}]}));
        assert_eq!(spec.budget(), Duration::from_millis(3000));
    }

    #[test]
    fn window_presets_project_shared_properties_and_timing() {
        for name in ["pop", "fade", "zoom"] {
            let spec = ok(
                json!({"targets":[{"window":{"id":7,"generation":3,"preset":name,"duration_ms":300}}]}),
            );
            let MotionTarget::WindowTracks {
                visual,
                duration,
                to,
                ..
            } = &spec.targets[0]
            else {
                panic!("tracks")
            };
            assert_eq!(*duration, Duration::from_millis(300));
            assert_eq!(visual.timing.duration, 0.3);
            assert!(to.iter().all(Option::is_none));
            assert!(visual.tracks.iter().all(|t| window_property(t.property)));
        }
    }

    #[test]
    fn validates_explicit_tracks_and_shared_easing() {
        let track = json!({"property":"opacity","keyframes":[{"offset":0,"value":0,"easing":{"kind":"steps","n":4,"start":false}},{"offset":1,"value":1}]});
        let spec = ok(
            json!({"targets":[{"window":{"id":7,"generation":3,"tracks":[track],"to":{"x":50},"curve":{"kind":"spring","stiffness":100,"damping":10,"mass":1,"velocity":0}}}]}),
        );
        assert!(matches!(
            spec.targets[0],
            MotionTarget::WindowTracks {
                easing: Easing::Track(motion::Curve::Spring { .. }),
                ..
            }
        ));
        for tracks in [
            json!([]),
            json!([track, track]),
            json!([{"property":"rotate","keyframes":[{"offset":0,"value":0},{"offset":1,"value":30}]}]),
            json!([{"property":"opacity","keyframes":[{"offset":0,"value":-1},{"offset":1,"value":1}]}]),
        ] {
            assert!(
                parse(&json!({"targets":[{"window":{"id":7,"generation":3,"tracks":tracks}}]}))
                    .is_err()
            );
        }
        assert!(parse(&json!({"targets":[{"window":{"id":7,"generation":3,"preset":"pop","tracks":[track]}}]})).is_err());
    }

    #[test]
    fn refuses_bad_input() {
        let refused = |args: Value| parse(&args).is_err();
        // Two targets (phase 3 takes one).
        assert!(refused(json!({"targets": [
            {"pointer": {"path": [{"x": 0, "y": 0}]}},
            {"pointer": {"path": [{"x": 0, "y": 0}]}}
        ]})));
        // Empty path.
        assert!(refused(json!({"targets": [{"pointer": {"path": []}}]})));
        // Over the duration cap.
        assert!(refused(json!({"targets": [{"pointer": {
            "path": [{"x": 0, "y": 0}], "duration_ms": 10001
        }}]})));
        // Unknown curve name.
        assert!(refused(json!({"targets": [{"pointer": {
            "path": [{"x": 0, "y": 0}], "curve": "bouncy"
        }}]})));
        // Bezier x out of range.
        assert!(refused(json!({"targets": [{"pointer": {
            "path": [{"x": 0, "y": 0}], "curve": [-0.1, 0, 0.5, 1]
        }}]})));
        // Non-finite point.
        assert!(refused(
            json!({"targets": [{"pointer": {"path": [{"x": "a", "y": 0}]}}]})
        ));
        // Unknown pointer field.
        assert!(refused(json!({"targets": [{"pointer": {
            "path": [{"x": 0, "y": 0}], "speed": 3
        }}]})));
        // Window: missing id, empty `to`, zero width, unknown `to` field.
        assert!(refused(json!({"targets": [{"window": {
            "generation": 1, "to": {"x": 1}
        }}]})));
        assert!(refused(json!({"targets": [{"window": {
            "id": 1, "generation": 1, "to": {}
        }}]})));
        assert!(refused(json!({"targets": [{"window": {
            "id": 1, "generation": 1, "to": {"w": 0}
        }}]})));
        assert!(refused(json!({"targets": [{"window": {
            "id": 1, "generation": 1, "to": {"z": 1}
        }}]})));
    }
}
