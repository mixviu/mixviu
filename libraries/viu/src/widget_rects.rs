// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scene identity and last-frame geometry, independent of input automation ids.
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub(crate) struct WidgetRect {
    pub id: String,
    pub kind: String,
    pub label: Option<String>,
    pub rect: egui::Rect,
    pub clip: egui::Rect,
    pub enabled: bool,
    pub focused: bool,
}

pub(crate) fn validate(args: &Value) -> Result<(), String> {
    if !args.is_object() {
        return Err("arguments must be an object".into());
    }
    if let Some(key) = args
        .as_object()
        .unwrap()
        .keys()
        .find(|k| !["ids", "kinds", "visible_only"].contains(&k.as_str()))
    {
        return Err(format!("unknown argument: {key}"));
    }
    for key in ["ids", "kinds"] {
        if let Some(value) = args.get(key)
            && !value
                .as_array()
                .is_some_and(|a| a.iter().all(Value::is_string))
        {
            return Err(format!("{key} must be an array of strings"));
        }
    }
    if args.get("visible_only").is_some_and(|v| !v.is_boolean()) {
        return Err("visible_only must be a boolean".into());
    }
    Ok(())
}

pub(crate) fn query(rows: &[WidgetRect], args: &Value) -> Result<Value, String> {
    validate(args)?;
    let matches = |key: &str, value: &str| {
        args.get(key)
            .is_none_or(|a| a.as_array().unwrap().iter().any(|v| v == value))
    };
    Ok(Value::Array(
        rows.iter()
            .filter(|row| {
                matches("ids", &row.id)
                    && matches("kinds", &row.kind)
                    && (args["visible_only"] != true || row.rect.intersect(row.clip).is_positive())
            })
            .map(|row| {
                let r = row.rect;
                let mut value = json!({"id":row.id,"kind":row.kind,
            "rect":{"x":r.left(),"y":r.top(),"w":r.width(),"h":r.height()},
            "window":null,"enabled":row.enabled,"focused":row.focused});
                if let Some(label) = &row.label {
                    value["label"] = json!(label);
                }
                value
            })
            .collect(),
    ))
}

pub(crate) fn descriptor(app: &str) -> Value {
    json!({"name":format!("{app}.widgets"),"args":{"ids":"array?","kinds":"array?","visible_only":"bool?"},
        "description":"Last-frame scene widget rectangles in surface-local logical coordinates", "read_only":true})
}

pub(crate) fn reply(
    bus: &crate::native_bus::Endpoint,
    request: crate::native_bus::Request,
    mut body: Value,
) {
    if body.as_array().unwrap().is_empty() {
        let _ = request.reply.send((0, body));
        return;
    }
    let unavailable = || {
        (
            10,
            json!({"error_code":"WINDOW_UNAVAILABLE","message":"No unique mapped window for the app host process"}),
        )
    };
    let (Some(client), Some(runtime)) = (bus.client(), bus.runtime_handle()) else {
        let _ = request.reply.send(unavailable());
        return;
    };
    runtime.spawn(async move {
        let result = client
            .call_with_headers_raw_within(
                "comp",
                "comp.windows.list",
                &Default::default(),
                "{}",
                std::time::Duration::from_secs(2),
            )
            .await;
        let window = result.ok().and_then(|(rc, body, _)| {
            if rc != 0 {
                return None;
            }
            let body: Value = serde_json::from_str(&body).ok()?;
            let mut windows = body["windows"]
                .as_array()?
                .iter()
                .filter(|w| w["pid"].as_u64() == Some(u64::from(std::process::id())));
            let window = windows.next()?;
            if windows.next().is_some() || !window["id"].is_u64() || !window["generation"].is_u64()
            {
                return None;
            }
            Some(json!({"id":window["id"],"generation":window["generation"]}))
        });
        let Some(window) = window else {
            let _ = request.reply.send(unavailable());
            return;
        };
        for row in body.as_array_mut().unwrap() {
            row["window"] = window.clone();
        }
        let _ = request.reply.send((0, body));
    });
}
