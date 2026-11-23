// PR #255 review regressions: exercise natural measurements and painted bounds
// on every settled frame, including a fresh view sharing a warm Context.
use super::*;

fn source(mode: &str, leaf: &str, extra: &str) -> String {
    let packing = if mode == "grid" {
        "columns:3,column_gap:12,"
    } else {
        "fill:true,gap:12,"
    };
    format!(
        r##"---
scene: 1
name: review-height
citizen: test
---
```mix
{{root:{{widget:"{mode}",equal_height:true,{packing}{extra}children:["a","b","c"]}},
 a:{{widget:"column",grow:1,stretch_last:true,gap:12,padding_top:7,padding_bottom:9,children:["first","last"]}},
 first:{{widget:"column",children:["f"]}},f:{{widget:"text",text:"First"}},last:{leaf},
 l:{{widget:"text",text:"Last"}},
 b:{{widget:"column",grow:1,children:["bt"]}},bt:{{widget:"text",text:"Medium\n2\n3\n4\n5"}},
 c:{{widget:"column",grow:1,children:["ct"]}},ct:{{widget:"text",text:"Tall\n2\n3\n4\n5\n6\n7\n8"}}}}
```
"##
    )
}

fn frames(ctx: &egui::Context, view: &mut SceneView, check: impl Fn(&SceneView)) {
    for frame in 0..32 {
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000., 900.),
                )),
                time: Some(frame as f64),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        );
        out.textures_delta.clear();
        if frame >= 8 {
            assert!(
                !out.platform_output.requested_discard(),
                "frame {frame}: {:?}",
                out.platform_output.request_discard_reasons
            );
            assert!(
                out.viewport_output
                    .values()
                    .all(|v| v.repaint_delay == std::time::Duration::MAX),
                "frame {frame}: repaint"
            );
            check(view);
        }
    }
}

#[test]
fn stretch_last_grid_and_row_keep_equal_bottoms_on_every_settled_frame() {
    for mode in ["grid", "row"] {
        // Normal last card, non-column leaf, and a genuinely zero-height leaf.
        for leaf in [
            r#"{widget:"column",children:["l"]}"#,
            r#"{widget:"text",text:"Last"}"#,
            r#"{widget:"spacer",size:0}"#,
        ] {
            for extra in ["", "justify:\"between\","] {
                if mode == "grid" && !extra.is_empty() {
                    continue;
                }
                let src = source(mode, leaf, extra);
                let ctx = egui::Context::default();
                ctx.set_pixels_per_point(2.4);
                // Second view must not depend on the first views natural_heights.
                for _ in 0..2 {
                    let mut view = SceneView::new(&src).unwrap();
                    frames(&ctx, &mut view, |v| {
                        let g = &v.geometry;
                        assert!(
                            ((g["last"].bottom() + 9. - g["c"].bottom()) * 2.4).abs() <= 0.5,
                            "{mode} {leaf}: {g:?}"
                        );
                        assert!((g["last"].top() - g["first"].bottom() - 12.).abs() < 0.5);
                    });
                }
            }
        }
    }
}

#[test]
fn equal_cells_with_different_descendants_invalidate_on_content_and_visibility() {
    for mode in ["grid", "row"] {
        let src = source(mode, r#"{widget:"column",children:["l"]}"#, "")
            .replace(
                "text:\"Tall\\n2\\n3\\n4\\n5\\n6\\n7\\n8\"",
                "text:\"= $model.text\"",
            )
            .replace("text:\"Last\"", "text:\"Last\",hidden:\"= $model.hide\"");
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(2.4);
        let mut view = SceneView::with_model(
            &src,
            json!({"text":"Tall\n2\n3\n4\n5\n6\n7\n8","hide":false}),
        )
        .unwrap();
        let mut heights = Vec::new();
        for (text, hide) in [
            ("Tall\n2\n3\n4\n5\n6\n7\n8", false),
            ("Changed\n2\n3\n4\n5\n6", true),
            ("Even taller\n2\n3\n4\n5\n6\n7\n8\n9\n10", false),
        ] {
            view.apply_model(json!({"text":text,"hide":hide})).unwrap();
            frames(&ctx, &mut view, |v| {
                assert!(((v.geometry["a"].bottom() - v.geometry["c"].bottom()) * 2.4).abs() <= 0.5)
            });
            heights.push(view.geometry["c"].height());
        }
        assert!(
            heights[1] < heights[0] && heights[2] > heights[0],
            "{heights:?}"
        );
    }
}

#[test]
fn fixed_cells_are_not_stretched_and_authored_inner_height_wins() {
    for mode in ["grid", "row"] {
        let src = source(mode, r#"{widget:"column",children:["l"]}"#, "")
            .replace("text:\"Medium\\n2\\n3\\n4\\n5\"", "text:\"Fixed\"")
            .replace(
                "grow:1,stretch_last:true",
                "grow:1,height:160,stretch_last:true,justify:\"end\"",
            )
            .replace(
                "b:{widget:\"column\",grow:1,",
                "b:{widget:\"column\",grow:1,height:50,",
            );
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(2.4);
        let mut view = SceneView::new(&src).unwrap();
        frames(&ctx, &mut view, |v| {
            let g = &v.geometry;
            assert!((g["a"].height() - 176.).abs() < 0.5, "{g:?}");
            assert!(
                (g["b"].height() - 50.).abs() < 0.5,
                "fixed child stretched: {g:?}"
            );
            assert!(
                (g["c"].height() - 176.).abs() < 0.5,
                "fixed height ignored in row maximum: {g:?}"
            );
            assert!(
                (g["first"].top() - g["a"].top() - 7.).abs() < 0.5,
                "justify must yield to stretch_last"
            );
            assert!(
                ((g["last"].bottom() + 9. - g["a"].bottom()) * 2.4).abs() <= 0.5,
                "{g:?}"
            );
        });
    }
}

#[test]
fn hiding_last_card_rekeys_the_last_visible_child() {
    for mode in ["grid", "row"] {
        let src = source(
            mode,
            r#"{widget:"column",hidden:"= $model.hide",children:["l"]}"#,
            "",
        );
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(2.4);
        let mut view = SceneView::with_model(&src, json!({"hide":false})).unwrap();
        for hide in [false, true, false] {
            view.apply_model(json!({"hide":hide})).unwrap();
            frames(&ctx, &mut view, |v| {
                let id = if hide { "first" } else { "last" };
                assert!(
                    ((v.geometry[id].bottom() + 9. - v.geometry["c"].bottom()) * 2.4).abs() <= 0.5,
                    "{mode}: {:?}",
                    v.geometry
                );
            });
        }
    }
}

#[test]
fn assigned_justify_last_child_excludes_distributed_growth_for_24_settled_frames() {
    for justify in ["end","between"] {
        let (children,nodes)=if justify=="between" {
            (r#"["t","u"]"#,r#"t:{widget:"text",text:"T",size:10,line_height:1},u:{widget:"text",text:"U",size:10,line_height:1}"#)
        } else {
            (r#"["t"]"#,r#"t:{widget:"text",text:"T",size:20,line_height:1},u:{widget:"text",text:"unused"}"#)
        };
        let src=format!(r#"---
scene: 1
name: justify-last
citizen: test
---
```mix
{{root:{{widget:"column",height:200,stretch_last:true,gap:10,children:["first","last"]}},
first:{{widget:"column",height:20,children:[]}},
last:{{widget:"column",justify:"{justify}",gap:0,children:{children}}},{nodes}}}
```
"#);
        let ctx=egui::Context::default();ctx.set_pixels_per_point(2.4);
        for _ in 0..2 {
            let mut view=SceneView::new(&src).unwrap();
            frames(&ctx,&mut view,|v|{
                let g=&v.geometry;
                assert!((g["first"].height()-20.).abs()<0.01);
                assert!((g["last"].top()-30.).abs()<0.01,"{justify}: {g:?}");
                assert!((g["root"].height()-200.).abs()<0.01,"{justify}: {g:?}");
                assert!(((g["last"].bottom()-200.)*2.4).abs()<=0.5,"{justify}: {g:?}");
            });
        }
    }
}

#[test]
fn equal_justify_cells_shrink_when_the_tall_sibling_shrinks() {
    for mode in ["row","grid"] {
        for justify in ["end","between"] {
            let packing=if mode=="grid"{"columns:2,column_gap:10,"}else{"fill:true,gap:10,"};
            let src=format!(r#"---
scene: 1
name: justify-shrink
citizen: test
---
```mix
{{root:{{widget:"{mode}",equal_height:true,{packing}children:["a","b"]}},
a:{{widget:"column",grow:1,justify:"{justify}",gap:0,children:["t","u"]}},
t:{{widget:"text",text:"T",size:10,line_height:1}},u:{{widget:"text",text:"U",size:10,line_height:1}},
b:{{widget:"column",grow:1,height:"= $model.height",children:[]}}}}
```
"#);
            let ctx=egui::Context::default();ctx.set_pixels_per_point(2.4);
            let mut view=SceneView::with_model(&src,json!({"height":200})).unwrap();
            for height in [200,80,160,60] {
                view.apply_model(json!({"height":height})).unwrap();
                frames(&ctx,&mut view,|v|{
                    let g=&v.geometry;
                    assert!((g["root"].height()-height as f32).abs()<=0.5,"{mode} {justify}: {g:?}");
                    assert!(((g["a"].bottom()-g["b"].bottom())*2.4).abs()<=0.5,"{mode} {justify}: {g:?}");
                });
            }
        }
    }
}

#[test]
fn stretch_last_preserves_an_authored_last_child_height() {
    let src=r#"---
scene: 1
name: fixed-last
citizen: test
---
```mix
{root:{widget:"column",height:200,stretch_last:true,gap:10,children:["first","last"]},first:{widget:"column",height:20,children:[]},last:{widget:"column",height:20,justify:"end",children:["t"]},t:{widget:"text",text:"T",size:10,line_height:1}}
```
"#;
    let ctx=egui::Context::default();ctx.set_pixels_per_point(2.4);
    let mut view=SceneView::new(src).unwrap();
    frames(&ctx,&mut view,|v|assert!((v.geometry["last"].height()-20.).abs()<=0.5,"{:?}",v.geometry));
}
