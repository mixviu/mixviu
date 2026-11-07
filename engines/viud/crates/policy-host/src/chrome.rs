// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Plain action-button hover state. The client owns tooltip timing/drawing.
use serde_json::{Value, json};
use world::state::Loop;

pub fn hover(lp: &Loop) -> Value {
    if world::comp::session_lock::active(lp) {
        return Value::Null;
    }
    let Some((window, rect, text, key, pressed)) = decor::seat::action_tip() else {
        return Value::Null;
    };
    let output = lp.inner.current_output();
    let space = &lp.inner.space_state().state;
    let Some(output_rect) = space.output_geometry(output) else {
        return Value::Null;
    };
    let Some(loc) = space.element_location(&window) else {
        return Value::Null;
    };
    let Some(ext) = decor::window::extents(&window) else {
        return Value::Null;
    };
    payload(
        lp.inner.active_output_key(),
        &key,
        &text,
        pressed,
        [
            loc.x as f32 - ext.left + rect.x - output_rect.loc.x as f32,
            loc.y as f32 - ext.top + rect.y - output_rect.loc.y as f32,
            rect.w,
            rect.h,
        ],
        [output_rect.size.w as f32, output_rect.size.h as f32],
    )
}
fn payload(
    output: String,
    key: &str,
    text: &str,
    pressed: bool,
    rect: [f32; 4],
    size: [f32; 2],
) -> Value {
    let [x, y, w, h] = rect;
    if !rect.iter().all(|v| v.is_finite())
        || w <= 0.0
        || h <= 0.0
        || x + w / 2.0 < 0.0
        || y + h / 2.0 < 0.0
        || x + w / 2.0 > size[0]
        || y + h / 2.0 > size[1]
    {
        return Value::Null;
    }
    // The renderer's identity is local process state; publish an opaque key.
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hash);
    json!({"output":output,"key":format!("{:016x}",hash.finish()),"title":text,
        "pressed":pressed,"anchor":rect,"output_size":size})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hover_preserves_anchor_and_press_state_but_clips_off_output_buttons() {
        let value = payload(
            "o_test".into(),
            "private-identity",
            "Save",
            true,
            [4.0, 8.0, 24.0, 24.0],
            [800.0, 600.0],
        );
        assert_eq!(value["anchor"], json!([4.0, 8.0, 24.0, 24.0]));
        assert_eq!(value["pressed"], true);
        assert_ne!(value["key"], "private-identity");
        assert!(
            payload(
                "o_test".into(),
                "k",
                "Save",
                false,
                [-100.0, 8.0, 24.0, 24.0],
                [800.0, 600.0]
            )
            .is_null()
        );
    }
}
