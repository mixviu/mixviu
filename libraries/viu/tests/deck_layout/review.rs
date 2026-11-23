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
 b:{{widget:"column",grow:1,children:["bt"]}},bt:{{widget:"text",text:"Medium\n2\n3"}},
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
