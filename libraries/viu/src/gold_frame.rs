// SPDX-License-Identifier: MIT OR Apache-2.0
//! Independent frame/content scene mounting; application identities stay intact.
pub const SOURCE: &str = include_str!("../mix/gold-frame.scene.mix");
pub fn mount(source: &str, model: serde_json::Value) -> Result<crate::view::SceneView, String> {
    crate::view::SceneView::with_frame(source, model, SOURCE)
}
