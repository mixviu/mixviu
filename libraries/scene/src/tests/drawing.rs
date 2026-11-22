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
