// SPDX-License-Identifier: MIT OR Apache-2.0
//! App-owned shortcut policy over shared Viu native input.
use eframe::egui;
use serde_json::Value;
pub use viu::input::Input;
pub fn shortcuts(ctx: &egui::Context, contract: &Value, text_input_active: bool) -> Vec<Value> {
    if text_input_active {
        return Vec::new();
    }
    ctx.input(|input|{
            let mut found=Vec::new();for event in &input.events {if let egui::Event::Key{key,pressed:true,modifiers,repeat:false,..}=event{
                let name=viu::input::key_name(*key);
                for action in contract["actions"].as_array().into_iter().flatten(){for binding in action["keys"].as_array().into_iter().flatten().filter_map(Value::as_str){let parts=binding.split('+').collect::<Vec<_>>();if parts.last()==Some(&name)&&((action["id"].as_str().is_some_and(|id|id.starts_with("selection."))&&!modifiers.alt)||(parts.contains(&"Ctrl")==modifiers.ctrl&&parts.contains(&"Shift")==modifiers.shift&&parts.contains(&"Alt")==modifiers.alt)){found.push(serde_json::json!({"id":action["id"],"params":{"ctrl":modifiers.ctrl,"shift":modifiers.shift}}));}}}
                if ["ArrowLeft","ArrowRight"].contains(&name)&&!modifiers.alt&&!modifiers.ctrl{found.push(serde_json::json!({"tree_navigation":name,"ctrl":modifiers.ctrl,"shift":modifiers.shift}));}
            }}found
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn native_arrow_aliases_reach_selection_history_and_tree_policy() {
        let ctx = egui::Context::default();
        let contract = json!({"actions":[{"id":"selection.next","keys":["ArrowDown"]},{"id":"nav.back","keys":["Alt+ArrowLeft"]}]});
        for (key, modifiers, expected) in [
            (
                egui::Key::ArrowDown,
                egui::Modifiers::SHIFT,
                json!({"id":"selection.next","params":{"ctrl":false,"shift":true}}),
            ),
            (
                egui::Key::ArrowLeft,
                egui::Modifiers::ALT,
                json!({"id":"nav.back","params":{"ctrl":false,"shift":false}}),
            ),
            (
                egui::Key::ArrowLeft,
                egui::Modifiers::NONE,
                json!({"tree_navigation":"ArrowLeft","ctrl":false,"shift":false}),
            ),
        ] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events: vec![
                        egui::Event::ModifiersChanged(modifiers),
                        egui::Event::Key {
                            key,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers,
                        },
                    ],
                    ..Default::default()
                },
                |_| {
                    assert_eq!(shortcuts(&ctx, &contract, false), vec![expected.clone()]);
                    assert!(shortcuts(&ctx, &contract, true).is_empty());
                },
            );
            output.textures_delta.clear();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events: vec![egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers,
                    }],
                    ..Default::default()
                },
                |_| {},
            );
            output.textures_delta.clear();
        }
    }
}
