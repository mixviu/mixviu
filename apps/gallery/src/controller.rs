// SPDX-License-Identifier: MIT OR Apache-2.0
//! Rust application composition and controller, independently expressed from Mix.
use serde_json::{Value, json};
use viu::widgets::Widget;

pub fn composition() -> Vec<Widget> {
    [
        ("label","label","Label",json!("Shared Viu typography")),
        ("button","button","Button",json!("Ready")),
        ("field","field","Text field",json!("Edit this text")),
        ("multiline","multiline","Multiline editor",json!("A small editable note")),
        ("checkbox","checkbox","Checkbox",json!(true)),
        ("toggle","toggle","Toggle",json!(false)),
        ("radio","radio","Radio group",json!("Alpha")),
        ("tabs","tabs","Tabs",json!("Beta")),
        ("slider","slider","Slider",json!(35)),
        ("number","number","Number / spin control",json!(42)),
        ("progress","progress","Progress indicator",json!(65)),
        ("combo","combo","Combo box",json!("Alpha")),
        ("list","list","Virtual list · 1,000 items",json!(1)),
        ("table","table","Virtual table · 1,000 rows",json!(0)),
        ("tree","tree","Tree rows",json!(0)),
        ("collapse","collapse","Disclosure",json!(false)),
        ("menu","menu","Menu",json!("Alpha")),
        ("dialog","dialog","Confirmation dialog",json!(false)),
        ("split","split","Resizable split",json!(50)),
        ("colour","colour","Colour picker",json!([70,150,210])),
        ("canvas","canvas","Canvas",json!("Ready")),
        ("link","link","Local link",json!("Ready")),
        ("tooltip","tooltip","Tooltip",json!("Hover for help")),
        ("separator","separator","Separator",json!("Section boundary")),
    ].into_iter().map(|(id,kind,label,initial)|Widget::new(id,kind,label,initial)).collect()
}
pub fn initial() -> Value {
    let widgets=composition();
    let values:serde_json::Map<String,Value>=widgets.iter().map(|w|(w.id.clone(),w.initial.clone())).collect();
    json!({"widgets":widgets.iter().map(Widget::encode).collect::<Vec<_>>(),"values":values,"serial":0,"activations":0,"last":"Ready"})
}
pub fn apply(model:&mut Value, action:&Value) {
    if action["action"]=="reset" { *model=initial(); return; }
    let id=action["id"].as_str().unwrap();
    match action["action"].as_str().unwrap() {
        "set"|"submit"=>model["values"][id]=action["value"].clone(),
        "accept"|"cancel"=>{model["values"][id]=json!(false);model["activations"]=json!(model["activations"].as_u64().unwrap()+1);},
        "activate"=>model["activations"]=json!(model["activations"].as_u64().unwrap()+1),
        _=>unreachable!("validated action"),
    }
    model["serial"]=json!(model["serial"].as_u64().unwrap()+1);
    model["last"]=json!(format!("{}.{}",id,action["action"].as_str().unwrap()));
}

/// App action contract shared by UI and Bus. Rejected requests never mutate state.
pub fn validate(widgets:&[Widget], body:&Value) -> Result<(),String> {
    if body["action"]=="reset" {return Ok(());}
    let widget=widgets.iter().find(|w|Some(w.id.as_str())==body["id"].as_str()).ok_or("unknown widget")?;
    let action=body["action"].as_str().ok_or("action must be a string")?;
    if !actions(widget).contains(&action) {return Err("action not supported by widget".into());}
    if action=="set" || action=="submit" {widget.validate(&body["value"])?;}
    Ok(())
}
pub fn actions(w:&Widget) -> Vec<&'static str> {
    let mut actions=vec!["set"];
    if ["button","list","table","tree","canvas","link"].contains(&w.kind.as_str()){actions.push("activate");}
    if ["field","multiline"].contains(&w.kind.as_str()){actions.push("submit");}
    if w.kind=="dialog" {actions.extend(["accept","cancel"]);}
    actions
}
