use super::*;

#[test]
fn drawing_primitives_and_bound_paint_are_validated() {
    let mut document = doc(
        "root: {widget: \"drawing\", w: 280, h: 280, sample_time: \"= $model.time\", primitives: [{kind: \"arc\", centre: [140,140], radius: [100,100], colour: \"#ffffff\", width: 30, start: 60, sweep: -300, dash_length: 770, effects: [{preset: \"draw\"}]}], paint_effect: \"= $model.effect\"}",
    );
    let good = serde_json::json!({"time":0.3,"effect":{"preset":"wipe","time":0.3}});
    document.model = Some(good);
    let resolved = resolve(&document);
    assert!(resolved.is_ok());
    for bad in [
        serde_json::json!({"preset":"bogus","time":0}),
        serde_json::json!({"preset":"zoom","time":99999}),
        serde_json::json!({"preset":"wipe","time":0,"extra":true}),
    ] {
        document.model = Some(serde_json::json!({"time":0.3,"effect":bad}));
        assert!(lint(&document).iter().any(|d| d.code == "binding-type"));
    }
    for bad in [
        "radius: [-1,100]",
        "effects: [{preset: \"unknown\"}]",
        "colour: \"red\"",
        "sweep: 9999",
    ] {
        let valid = "{kind: \"arc\", centre: [140,140], radius: [100,100], colour: \"#ffffff\", width: 30, start: 60, sweep: -300}";
        let invalid = match bad {
            "radius: [-1,100]" => valid.replace("radius: [100,100]", bad),
            "colour: \"red\"" => valid.replace("colour: \"#ffffff\"", bad),
            "sweep: 9999" => valid.replace("sweep: -300", bad),
            _ => valid.replace("width: 30", bad),
        };
        let result = lint(&doc(&format!(
            "root: {{widget: \"drawing\", primitives: [{invalid}]}}"
        )));
        assert!(!result.is_empty(), "{bad}");
    }
}

#[test]
fn hostile_drawing_ports_are_refused_at_the_boundary() {
    let p=serde_json::json!({"kind":"arc","centre":[50,50],"radius":[20,20],"colour":"#ffffff","width":5,"start":0,"sweep":180});
    for (field,bad) in [
        ("dash_length",serde_json::json!(0)),("dash_length",serde_json::json!(10001)),
        ("radius",serde_json::json!([1e-40,1e-40])),("sweep",serde_json::json!(1e-40)),
        ("centre",serde_json::json!([10001,0])),("width",serde_json::json!(-1)),
        ("width",serde_json::json!(10001)),("effects",serde_json::json!([{"preset":"float","delay":61}])),
        ("effects",serde_json::json!([{"preset":"float"},{"preset":"float"},{"preset":"float"},{"preset":"float"},{"preset":"float"}])),
    ] {
        let mut primitive=p.clone();primitive[field]=bad;
        assert!(crate::drawing::validate(&serde_json::json!([primitive])).is_err(),"{field}");
    }
    assert!(crate::drawing::validate(&serde_json::json!(vec![p;65])).is_err());
    let shadow=serde_json::json!({"colour":[242,70,70,77],"offset":[0,30],"blur":70});
    for (field,bad) in [("blur",serde_json::json!(-1)),("blur",serde_json::json!(201)),("blur",serde_json::Value::Null),("offset",serde_json::json!([1001,0])),("colour",serde_json::json!([256,0,0,0]))] {
        let mut s=shadow.clone();s[field]=bad;assert!(crate::drawing::validate_shadow(&s).is_err());
    }
    for bad in [serde_json::json!(1e308),serde_json::json!(-3601),serde_json::Value::Null] {
        assert!(crate::bindings::coerce_port(&bad,crate::schema::port_for("drawing","sample_time")).is_err());
    }
    assert!(crate::bindings::coerce_port(&serde_json::json!(101),crate::schema::port_for("text","letter_spacing")).is_err());
}
