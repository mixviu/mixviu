// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use viu::{view::SceneView, widgets::Widget};
fn board() -> serde_json::Value {
    json!({"selected":"a","items":[{"id":"a","title":"Example","values":[1]}],"blocks":[{"kind":"sparklines","height":44,"items":[{"values":[1,2],"peak":100}]}]})
}
#[test]
fn invalid_renderer_bounds_refuse_without_mutating_the_scene() {
    let source="---\nscene: 1\nname: example\ncitizen: example\n---\n```mix\n{root:{widget:\"control\",definition:\"= $model.definition\"}}\n```\n";
    let model=json!({"definition":{"id":"root","kind":"dashboard","label":"","initial":board(),"properties":{}}});
    let mut view=SceneView::with_model(source,model.clone()).unwrap();
    let revision=view.revision;
    let mut bad=model.clone();bad["definition"]["initial"]["blocks"][0]["height"]=json!(1e308);
    assert!(view.apply_model(bad).is_err());assert_eq!(view.model(),&model);assert_eq!(view.revision,revision);
    let mut bad=model.clone();bad["definition"]["initial"]["sidebar_width"]=json!(1e308);
    assert!(view.apply_model(bad).is_err());assert_eq!(view.model(),&model);
    let chart=json!({"series":[{"name":"Example","values":[1,null,3]}],"samples":60.0,"interval_s":1,"height":180,"scale":"percent","format":"percent"});
    viu::time_series::validate(&chart).unwrap();
    let mut bad=chart;bad["interval_s"]=json!(1e308);assert!(viu::time_series::validate(&bad).is_err());
}
#[test]
fn disabled_dashboard_and_read_only_table_never_admit_mutating_intents() {
    let widget=Widget::new("example","dashboard","",board()).with_properties(json!({"enabled":false}));
    assert_eq!(widget.admit(&json!({}),"choose",&json!("a")).unwrap_err(),"DISABLED");
    let table=Widget::new("table","tree-table","",json!({"key":"rows","columns":[{"id":"name","label":"Name"}],"rows":[]})).with_properties(json!({"style":"grid","read_only":true}));
    assert!(table.actions().is_empty());assert!(table.admit(&json!({}),"sort",&json!({"column":"name"})).is_err());
}
