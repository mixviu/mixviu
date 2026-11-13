// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Agent frame capture arguments and reply, shared by transport and render hosts.

use crate::reply::ControlReply;
use crate::request::invalid_argument;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureFormat {
    Png,
    Ppm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureFrameSpec {
    pub output: Option<String>,
    pub output_generation: Option<u64>,
    pub window: Option<CaptureWindow>,
    pub region: Option<CaptureRegion>,
    pub cursor: bool,
    pub path: String,
    pub format: CaptureFormat,
}

/// Render-time topology fence, maintained by the compositor's output observer.
pub struct OutputGeneration(
    pub std::sync::atomic::AtomicU64,
    pub std::sync::Mutex<OutputSignature>,
);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputSignature {
    pub mode: Option<(i32, i32, i32)>,
    pub scale_bits: u64,
    pub transform: String,
}

/// Output-local displayed logical pixels, matching `comp.region.select`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaptureWindow {
    pub id: u64,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RecordCommand {
    Start { request_id: String, targets: Vec<CaptureFrameSpec>, path: String, fps: u32, max_seconds: u64 },
    Marker { session: String, label: String, data: Value },
    Status { session: Option<String> },
    Stop { session: String, discard: bool },
}

pub fn parse_record(command: &str, args: &Value) -> Result<RecordCommand, ControlReply> {
    let fields = args.as_object().ok_or_else(|| invalid_argument("args", "object", "record arguments"))?;
    let allowed: &[&str] = match command {
        "comp.record.start" => &["request_id", "target", "path", "fps", "cursor", "max_seconds"],
        "comp.record.marker" => &["session", "label", "data"],
        "comp.record.status" => &["session"],
        _ => &["session", "discard"],
    };
    if let Some(key) = fields.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(invalid_argument(key, "record argument", "see comp.record contract"));
    }
    let text = |key: &str| -> Result<String, ControlReply> {
        fields.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
            .map(str::to_owned).ok_or_else(|| invalid_argument(key, "non-empty string", "string"))
    };
    match command {
        "comp.record.start" => {
            let request_id = text("request_id")?;
            let path = text("path")?;
            if !path.ends_with(".mkv") { return Err(invalid_argument("path", "Matroska path", "absolute .mkv path")); }
            let fps = fields.get("fps").unwrap_or(&Value::Null).as_u64().or_else(|| (!fields.contains_key("fps")).then_some(30))
                .filter(|fps| (1..=60).contains(fps)).ok_or_else(|| invalid_argument("fps", "integer", "1..=60"))? as u32;
            let max_seconds = fields.get("max_seconds").and_then(Value::as_u64).filter(|n| (1..=86400).contains(n))
                .ok_or_else(|| invalid_argument("max_seconds", "integer", "1..=86400"))?;
            let cursor = match fields.get("cursor") { None => false, Some(Value::Bool(b)) => *b, _ => return Err(invalid_argument("cursor", "boolean", "true|false")) };
            let target = fields.get("target").and_then(Value::as_object).ok_or_else(|| invalid_argument("target", "object", "{output?|window?|windows?|region?}"))?;
            let mut targets = Vec::new();
            if let Some(windows) = target.get("windows") {
                let windows = windows.as_array().filter(|w| !w.is_empty() && w.len() <= 16).ok_or_else(|| invalid_argument("windows", "array", "1..=16 window targets"))?;
                if target.len() != 1 { return Err(invalid_argument("target", "exclusive windows target", "{windows:[{id,generation}]}")); }
                for window in windows { targets.push(parse(&json!({"window":window,"path":path,"cursor":cursor}))?); }
            } else {
                let mut capture = Value::Object(target.clone());
                capture["path"] = json!(path);
                capture["cursor"] = json!(cursor);
                targets.push(parse(&capture)?);
            }
            Ok(RecordCommand::Start { request_id, targets, path, fps, max_seconds })
        }
        "comp.record.marker" => Ok(RecordCommand::Marker { session: text("session")?, label: text("label")?, data: fields.get("data").cloned().unwrap_or(Value::Null) }),
        "comp.record.status" => Ok(RecordCommand::Status { session: if fields.contains_key("session") { Some(text("session")?) } else { None } }),
        _ => Ok(RecordCommand::Stop { session: text("session")?, discard: match fields.get("discard") { None => false, Some(Value::Bool(b)) => *b, _ => return Err(invalid_argument("discard", "boolean", "true|false")) } }),
    }
}

pub fn parse(args: &Value) -> Result<CaptureFrameSpec, ControlReply> {
    let object = args.as_object().ok_or_else(|| {
        invalid_argument("args", "JSON object", "{output?|window?, path, format?}")
    })?;
    if let Some(field) = object.keys().find(|key| {
        ![
            "output",
            "output_generation",
            "window",
            "region",
            "cursor",
            "path",
            "format",
        ]
        .contains(&key.as_str())
    }) {
        return Err(ControlReply::InvalidArgs {
            field: field.clone(),
            allowed: &[
                "output",
                "output_generation",
                "window",
                "region",
                "cursor",
                "path",
                "format",
            ],
        });
    }
    if object.contains_key("output") && object.contains_key("window") {
        return Err(ControlReply::InvalidArgs {
            field: "window".into(),
            allowed: &["output", "path", "format"],
        });
    }
    let window = object
        .get("window")
        .map(|value| {
            let target = value
                .as_object()
                .ok_or_else(|| invalid_argument("window", "JSON object", "{id, generation}"))?;
            if let Some(field) = target
                .keys()
                .find(|key| !["id", "generation"].contains(&key.as_str()))
            {
                return Err(ControlReply::InvalidArgs {
                    field: format!("window.{field}"),
                    allowed: &["id", "generation"],
                });
            }
            let (id, generation) = crate::request::required_target(target)?;
            Ok(CaptureWindow { id, generation })
        })
        .transpose()?;
    let region = object
        .get("region")
        .map(|value| {
            let fields = value
                .as_object()
                .ok_or_else(|| invalid_argument("region", "object", "{x,y,width,height}"))?;
            if fields.len() != 4
                || fields
                    .keys()
                    .any(|key| !["x", "y", "width", "height"].contains(&key.as_str()))
            {
                return Err(invalid_argument(
                    "region",
                    "exact rectangle",
                    "{x,y,width,height}",
                ));
            }
            let number = |key: &str| {
                fields
                    .get(key)
                    .and_then(Value::as_i64)
                    .and_then(|n| i32::try_from(n).ok())
                    .ok_or_else(|| invalid_argument("region", "i32 integers", "{x,y,width,height}"))
            };
            let region = CaptureRegion {
                x: number("x")?,
                y: number("y")?,
                width: number("width")?,
                height: number("height")?,
            };
            if window.is_some()
                || region.x < 0
                || region.y < 0
                || region.width <= 0
                || region.height <= 0
                || region.x.checked_add(region.width).is_none()
                || region.y.checked_add(region.height).is_none()
            {
                return Err(invalid_argument(
                    "region",
                    "output rectangle with non-negative origin and positive extent",
                    "region excludes window",
                ));
            }
            Ok(region)
        })
        .transpose()?;
    let cursor = match object.get("cursor") {
        None => true,
        Some(Value::Bool(value)) => *value,
        _ => return Err(invalid_argument("cursor", "boolean", "true|false")),
    };
    let path = object
        .get("path")
        .and_then(Value::as_str)
        .filter(|path| std::path::Path::new(path).is_absolute() && !path.contains('\0'))
        .ok_or_else(|| invalid_argument("path", "absolute file path", "/path/to/frame.png"))?;
    let output = match object.get("output") {
        None => None,
        Some(value) => Some(
            value
                .as_str()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| invalid_argument("output", "non-empty output name", "output name"))?
                .to_owned(),
        ),
    };
    let format = match object.get("format") {
        None => CaptureFormat::Png,
        Some(value) if value.as_str() == Some("png") => CaptureFormat::Png,
        Some(value) if value.as_str() == Some("ppm") => CaptureFormat::Ppm,
        _ => return Err(invalid_argument("format", "image format", "png|ppm")),
    };
    let output_generation = object
        .get("output_generation")
        .map(|value| {
            if output.is_none() || window.is_some() {
                return Err(invalid_argument(
                    "output_generation",
                    "named output fence",
                    "requires output, excludes window",
                ));
            }
            value.as_u64().ok_or_else(|| {
                invalid_argument("output_generation", "u64", "selection output generation")
            })
        })
        .transpose()?;
    Ok(CaptureFrameSpec {
        output,
        output_generation,
        window,
        region,
        cursor,
        path: path.to_owned(),
        format,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureSource {
    Kms,
    Offscreen,
}

pub fn reply(
    path: &str,
    width: u32,
    height: u32,
    scale: f64,
    output: &str,
    window: Option<CaptureWindow>,
    source: CaptureSource,
) -> Value {
    let mut body = json!({"path": path, "width": width, "height": height, "scale": scale,
        "output": output, "source": match source { CaptureSource::Kms => "kms", CaptureSource::Offscreen => "offscreen" }});
    if let Some(window) = window {
        body["window"] = json!({"id": window.id, "generation": window.generation});
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_validation_and_read_only_classification() {
        let args=json!({"request_id":"take","target":{"windows":[{"id":1,"generation":2},{"id":3,"generation":4}]},"path":"/tmp/take.mkv","max_seconds":60});
        let RecordCommand::Start{targets,fps,..}=parse_record("comp.record.start",&args).unwrap() else {panic!()};
        assert_eq!(targets.len(),2);
        assert_eq!(fps,30);
        assert!(!targets[0].cursor);
        for (key,value) in [("path",json!("relative.mkv")),("fps",json!(0)),("fps",json!(61)),("max_seconds",json!(0)),("request_id",json!("")),("cursor",json!("false")),("unknown",json!(true))] {
            let mut bad=args.clone(); bad[key]=value;
            assert!(parse_record("comp.record.start",&bad).is_err(),"{bad}");
        }
        assert!(crate::catalogue::read_only("comp.record.status"));
        for command in ["comp.record.start","comp.record.marker","comp.record.stop"] { assert!(!crate::catalogue::read_only(command)); }
        assert!(matches!(crate::request::classify("comp.record.status",&json!({}),false),Ok(crate::request::Request::Long(crate::request::LongOp::Record(RecordCommand::Status{session:None})))));
    }

    #[test]
    fn defaults_and_explicit_arguments() {
        assert_eq!(
            parse(&json!({"path": "/tmp/frame.png"})).unwrap(),
            CaptureFrameSpec {
                output: None,
                output_generation: None,
                window: None,
                region: None,
                cursor: true,
                path: "/tmp/frame.png".into(),
                format: CaptureFormat::Png,
            }
        );
        let spec =
            parse(&json!({"path": "/tmp/frame.ppm", "output": "DP-1", "format": "ppm"})).unwrap();
        assert_eq!(spec.output.as_deref(), Some("DP-1"));
        assert_eq!(spec.format, CaptureFormat::Ppm);
    }

    #[test]
    fn window_arguments_and_exclusive_targets() {
        let spec = parse(&json!({
            "path": "/tmp/window.png", "window": {"id": 7, "generation": 3}
        }))
        .unwrap();
        assert_eq!(spec.output, None);
        assert_eq!(
            spec.window,
            Some(CaptureWindow {
                id: 7,
                generation: 3
            })
        );
        assert_eq!(spec.format, CaptureFormat::Png);
        let refusal = parse(&json!({
            "path": "/tmp/window.png", "output": "DP-1", "window": {"id": 7, "generation": 3}
        }))
        .unwrap_err()
        .into_wire();
        assert_eq!(refusal.0, 10);
        let body: Value = serde_json::from_str(&refusal.1).unwrap();
        assert_eq!(body["error"], "invalid_args");
        assert_eq!(body["field"], "window");
        for target in [
            Value::Null,
            json!(7),
            json!({"id": 7}),
            json!({"generation": 3}),
            json!({"id": -1, "generation": 3}),
            json!({"id": 7, "generation": "3"}),
            json!({"id": 7, "generation": 3, "extra": true}),
        ] {
            assert!(parse(&json!({"path": "/tmp/window.png", "window": target})).is_err());
        }
    }

    #[test]
    fn malformed_arguments_are_refused() {
        for args in [
            Value::Null,
            json!({}),
            json!({"path": "relative.png"}),
            json!({"path": 42}),
            json!({"path": "/tmp/a", "format": "jpg"}),
            json!({"path": "/tmp/a", "output": ""}),
            json!({"path": "/tmp/a", "extra": true}),
        ] {
            assert!(parse(&args).is_err(), "{args}");
        }
        assert!(
            crate::request::classify("comp.capture.frame", &json!({"path": "/tmp/a"}), true)
                .is_err()
        );
        assert!(matches!(
            crate::request::classify("comp.capture.frame", &json!({"path": "/tmp/a"}), false),
            Ok(crate::request::Request::Long(
                crate::request::LongOp::CaptureFrame(_)
            ))
        ));
    }

    #[test]
    fn regions_and_cursor_are_strict_and_additive() {
        let spec = parse(&json!({"path":"/tmp/a", "cursor":false,
            "region":{"x":10,"y":20,"width":30,"height":40}}))
        .unwrap();
        assert!(!spec.cursor);
        assert!(parse(&json!({"path":"/tmp/a","output_generation":1})).is_err());
        for generation in [Value::Null, json!(-1), json!(1.5), json!("1")] {
            assert!(
                parse(&json!({"path":"/tmp/a","output":"DP-1","output_generation":generation}))
                    .is_err()
            );
        }
        assert_eq!(
            parse(&json!({"path":"/tmp/a","output":"DP-1","output_generation":3}))
                .unwrap()
                .output_generation,
            Some(3)
        );
        assert_eq!(
            spec.region,
            Some(CaptureRegion {
                x: 10,
                y: 20,
                width: 30,
                height: 40
            })
        );
        for region in [
            Value::Null,
            json!({"x":0,"y":0,"width":0,"height":2}),
            json!({"x":-1,"y":0,"width":2,"height":2}),
            json!({"x":2147483647,"y":0,"width":2,"height":2}),
            json!({"x":0,"y":0,"width":2.5,"height":2}),
            json!({"x":0,"y":0,"width":2,"height":2,"extra":1}),
        ] {
            assert!(parse(&json!({"path":"/tmp/a","region":region})).is_err());
        }
        assert!(parse(&json!({"path":"/tmp/a","cursor":"false"})).is_err());
        assert!(
            parse(&json!({"path":"/tmp/a","window":{"id":1,"generation":1},
            "region":{"x":0,"y":0,"width":2,"height":2}}))
            .is_err()
        );
    }

    #[test]
    fn reply_shape_and_sources() {
        for (source, name) in [
            (CaptureSource::Kms, "kms"),
            (CaptureSource::Offscreen, "offscreen"),
        ] {
            assert_eq!(
                reply("/tmp/a", 1920, 1080, 1.5, "DP-1", None, source),
                json!({
                    "path": "/tmp/a", "width": 1920, "height": 1080, "scale": 1.5, "output": "DP-1", "source": name,
                })
            );
        }
        assert_eq!(
            reply(
                "/tmp/window.png",
                800,
                600,
                1.0,
                "DP-1",
                Some(CaptureWindow {
                    id: 7,
                    generation: 3
                }),
                CaptureSource::Offscreen
            ),
            json!({"path": "/tmp/window.png", "width": 800, "height": 600,
                "scale": 1.0, "output": "DP-1", "window": {"id": 7, "generation": 3}, "source": "offscreen"})
        );
    }
}
