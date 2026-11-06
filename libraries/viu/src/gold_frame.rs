// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lab composition adapter. Regions and controls are authored in the shared
//! Mix scene; the application retains its content, model and intent policy.
pub fn compose(source: &str) -> Result<String, String> {
    let mut app = scene::parse(source).map_err(|e| format!("{e:?}"))?;
    let frame =
        scene::parse(include_str!("../mix/gold-frame.scene.mix")).map_err(|e| format!("{e:?}"))?;
    for id in frame.nodes.keys().filter(|id| id.as_str() != "root") {
        if app.nodes.contains_key(id) {
            return Err(format!("shared frame node collision: {id}"));
        }
    }
    let content = app.nodes.shift_remove("root").ok_or("missing app root")?;
    // Mounting relocates the layout root. Refuse roots that own controls or
    // emit intents rather than silently changing the application's protocol.
    // A future generic mount must retain that identity before Dopus adopts it.
    if !matches!(
        content.widget.as_str(),
        "column" | "row" | "grid" | "dock" | "split"
    ) || content.ports.keys().any(|key| key.starts_with("on_"))
    {
        return Err("gold frame requires a layout-only root without intent handlers".into());
    }
    app.nodes.insert("gold-content".into(), content);
    for (id, node) in frame.nodes {
        if id != "gold-content" {
            app.nodes.insert(id, node);
        }
    }
    Ok(scene::to_source(&app))
}
