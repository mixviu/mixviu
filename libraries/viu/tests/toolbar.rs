// SPDX-License-Identifier: MIT OR Apache-2.0

#[test]
fn toolbar_reserves_end_actions_and_omits_whole_optional_groups() {
    let source = r#"---
scene: 1
name: toolbar
citizen: toolbar
---
```mix
{root:{widget:"row",packing:"toolbar",end:"save",fit:["width"],children:["take","width","save"]},
take:{widget:"control",definition:{id:"take",kind:"command-bar",label:"",initial:[{id:"take",label:"Take screenshot",enabled:true,primary:true}],properties:{style:"gold",hostable:false}}},
width:{widget:"control",definition:{id:"width",kind:"combo",label:"Width",initial:"medium",properties:{native:true,choices:["medium","extra"],choice_labels:{medium:"Medium",extra:"Extra wide choice"},measure_choices:true,width:96}}},
save:{widget:"control",definition:{id:"save",kind:"command-bar",label:"",initial:[{id:"save",label:"Save",enabled:true}],properties:{style:"gold",hostable:false,reverse:true}}}}
```
"#;
    let mut view = viu::view::SceneView::new(source).unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    for (width, visible) in [(800.0, true), (250.0, false), (800.0, true)] {
        for _ in 0..3 {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 80.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    view.show(ui);
                },
            )
            .drop_without_applying_deltas();
        }
        assert_eq!(view.geometry.contains_key("width"), visible);
        assert!(
            view.geometry["take.command.take"].right() < view.geometry["save.command.save"].left()
        );
        if visible {
            assert!(view.geometry["width"].right() < view.geometry["save.command.save"].left());
        }
    }
    assert!(
        viu::view::SceneView::new(&source.replace("fit:[\"width\"]", "fit:[\"take\"]")).is_err()
    );
}
