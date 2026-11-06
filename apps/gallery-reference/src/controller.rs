// SPDX-License-Identifier: MIT OR Apache-2.0
//! Rust application composition and controller, independently expressed from Mix.
use serde_json::{Value, json};
use viu::widgets::Widget;

pub fn composition() -> Vec<Widget> {
    [
        ("label", "label", "Label", json!("Shared Viu typography")),
        ("button", "button", "Button", json!("Ready")),
        ("field", "field", "Text field", json!("Edit this text")),
        (
            "multiline",
            "multiline",
            "Multiline editor",
            json!("A small editable note"),
        ),
        ("checkbox", "checkbox", "Checkbox", json!(true)),
        ("toggle", "toggle", "Toggle", json!(false)),
        ("radio", "radio", "Radio group", json!("Alpha")),
        ("tabs", "tabs", "Tabs", json!("Beta")),
        ("slider", "slider", "Slider", json!(35)),
        ("number", "number", "Number / spin control", json!(42)),
        ("progress", "progress", "Progress indicator", json!(65)),
        ("combo", "combo", "Combo box", json!("Alpha")),
        ("list", "list", "Virtual list · 1,000 items", json!(1)),
        ("table", "table", "Virtual table · 1,000 rows", json!(0)),
        ("tree", "tree", "Tree rows", json!(0)),
        ("collapse", "collapse", "Disclosure", json!(false)),
        ("menu", "menu", "Menu", json!("Alpha")),
        ("dialog", "dialog", "Confirmation dialog", json!(false)),
        ("split", "split", "Resizable split", json!(50)),
        ("colour", "colour", "Colour picker", json!([70, 150, 210])),
        ("canvas", "canvas", "Canvas", json!("Ready")),
        ("link", "link", "Local link", json!("Ready")),
        ("tooltip", "tooltip", "Tooltip", json!("Hover for help")),
        (
            "separator",
            "separator",
            "Separator",
            json!("Section boundary"),
        ),
        (
            "icon",
            "icon",
            "Vector icon button",
            json!({"name":"copy","size":24,"colour":[80,180,230],"enabled":true,"selected":false}),
        ),
        ("toolbar", "toolbar", "Icon toolbar", json!("copy")),
    ]
    .into_iter()
    .map(|(id, kind, label, initial)| {
        Widget::new(id, kind, label, initial).with_properties(properties(kind))
    })
    .collect()
}
fn properties(kind: &str) -> Value {
    match kind {
        "button" => json!({"text":"Press me","icon":"play"}),
        "icon" => json!({"text":""}),
        "toolbar" => {
            json!({"items":[{"id":"copy","icon":"copy","text":"Copy","enabled":true},{"id":"save","icon":"save","text":"Save","enabled":true},{"id":"trash","icon":"trash","text":"Trash","enabled":false}]})
        }
        "checkbox" => json!({"text":"Enabled"}),
        "toggle" => json!({"text":"Power"}),
        "radio" | "combo" => json!({"choices":["Alpha","Beta","Gamma"]}),
        "tabs" => {
            json!({"choices":["Alpha","Beta","Gamma"],"pages":{"Alpha":"Alpha page content","Beta":"Beta page content","Gamma":"Gamma page content"}})
        }
        "menu" => {
            json!({"text":"Actions","choices":["Alpha","Beta","Gamma"],"choice_icons":{"Alpha":"folder","Beta":"file","Gamma":"star"}})
        }
        "list" | "table" | "tree" => {
            json!({"dataset":"examples","row_count":1000,"columns":["Name","Size"],"parent":"Example folder","context_action":"Choose item","row_icon":"file-text","parent_icon":"folder-open","context_icon":"check"})
        }
        "collapse" => json!({"text":"Details","body":"Reusable Viu content"}),
        "dialog" => {
            json!({"text":"Open confirmation…","title":"Gallery confirmation","body":"The same dialog in Rust and Mix."})
        }
        "link" => json!({"text":"Activate a safe local link"}),
        "tooltip" => json!({"body":"Shared Viu tooltip; identical in both languages."}),
        "canvas" => {
            json!({"shapes":[{"kind":"line","from":[0,1],"to":[1,0]},{"kind":"circle","at":[0.5,0.5],"radius":7}]})
        }
        _ => json!({}),
    }
}
pub fn initial() -> Value {
    let widgets = composition();
    let values: serde_json::Map<String, Value> = widgets
        .iter()
        .map(|w| (w.id.clone(), w.initial.clone()))
        .collect();
    let rows: Value = serde_json::from_str(include_str!("../../gallery/rows.json"))
        .expect("captured gallery dataset");
    json!({"widgets":widgets.iter().map(Widget::encode).collect::<Vec<_>>(),"datasets":{"examples":rows},"values":values,"expanded":{"tree":true},"sorted":{"table":false},"serial":0,"activations":0,"last_activation":null,"last":"Ready"})
}
pub fn apply(model: &mut Value, action: &Value) {
    if action["action"] == "reset" {
        *model = initial();
        return;
    }
    let id = action["id"].as_str().unwrap();
    match action["action"].as_str().unwrap() {
        "set" | "submit" => model["values"][id] = action["value"].clone(),
        "expand" => model["expanded"][id] = action["value"].clone(),
        "sort" => model["sorted"][id] = action["value"].clone(),
        "accept" | "cancel" => {
            model["values"][id] = json!(false);
            model["activations"] = json!(model["activations"].as_u64().unwrap() + 1);
        }
        "activate" => {
            model["activations"] = json!(model["activations"].as_u64().unwrap() + 1);
            model["last_activation"] = json!({"id":id,"value":action["value"]});
            if id == "toolbar" {
                model["values"][id] = action["value"].clone();
            }
        }
        _ => unreachable!("validated action"),
    }
    model["serial"] = json!(model["serial"].as_u64().unwrap() + 1);
    model["last"] = json!(format!("{}.{}", id, action["action"].as_str().unwrap()));
}

/// App action contract shared by UI and Bus. Rejected requests never mutate state.
pub fn validate(widgets: &[Widget], body: &Value) -> Result<(), String> {
    if body["action"] == "reset" {
        return Ok(());
    }
    let widget = widgets
        .iter()
        .find(|w| Some(w.id.as_str()) == body["id"].as_str())
        .ok_or("unknown widget")?;
    let action = body["action"].as_str().ok_or("action must be a string")?;
    widget.validate_action(action, &body["value"])
}
pub fn actions(w: &Widget) -> Vec<&'static str> {
    w.actions()
}

/// Validate publications before replacing the last good composition/model.
pub fn validate_model(model: &Value) -> Result<(), String> {
    let widgets = model["widgets"]
        .as_array()
        .filter(|w| !w.is_empty() && w.len() <= 128)
        .ok_or("expected 1..128 widgets")?;
    let mut ids = std::collections::BTreeSet::new();
    for value in widgets {
        let widget = Widget::decode(value)?;
        if !ids.insert(widget.id.clone()) {
            return Err("duplicate widget ID".into());
        }
        widget.validate_model(model)?;
    }
    Ok(())
}
