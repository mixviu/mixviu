use super::*;

#[test]
fn ordinary_scene_widget_ids_match_mains_direct_node_path() {
    let nodes = json!({"root":{"widget":"column","children":["entry","other"]},"entry":{"widget":"control","definition":{"id":"entry","kind":"field","label":"Entry","initial":""}},"other":{"widget":"control","definition":{"id":"other","kind":"field","label":"Other","initial":""}}});
    let source =
        format!("---\nscene: 1\nname: identity\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    let draw = |direct: bool| {
        let ctx = egui::Context::default();
        let mut view = SceneView::new(&source).unwrap();
        let nodes = view.tree.nodes.clone().into_iter().collect();
        let mut ids = Vec::new();
        for target in ["entry", "other", "entry"] {
            view.widgets.request_focus(target, None).unwrap();
            ctx.run_ui(egui::RawInput::default(), |ui| {
                // Main's node entry calls the contents directly; the motion
                // wrapper must take exactly that path for ordinary scenes.
                if direct {
                    view.node_contents(ui, "root", &nodes, None, 0, &mut Vec::new());
                } else {
                    view.node(ui, "root", &nodes, None, 0, &mut Vec::new());
                }
            })
            .drop_without_applying_deltas();
            let focused = ctx.memory(|m| m.focused()).expect("field focus");
            ids.push(focused);
        }
        assert_ne!(ids[0], ids[1]);
        assert_eq!(ids[0], ids[2]);
        ids
    };
    assert_eq!(draw(false), draw(true));
}
