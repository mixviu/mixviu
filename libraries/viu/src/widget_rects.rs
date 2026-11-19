// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scene identity and last-frame geometry, independent of input automation ids.
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub(crate) struct WidgetRect {
    pub id: String,
    pub kind: String,
    pub label: Option<String>,
    pub rect: egui::Rect,
    pub visible: bool,
    pub enabled: bool,
    pub focused: bool,
}

pub(crate) fn query(rows: &[WidgetRect], args: &Value) -> Result<Value, String> {
    if !args.is_object() {
        return Err("arguments must be an object".into());
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
    let matches = |key: &str, value: &str| {
        args.get(key)
            .is_none_or(|a| a.as_array().unwrap().iter().any(|v| v == value))
    };
    Ok(Value::Array(
        rows.iter()
            .filter(|row| {
                matches("ids", &row.id)
                    && matches("kinds", &row.kind)
                    && (args["visible_only"] != true || row.visible)
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
