// SPDX-License-Identifier: MIT OR Apache-2.0
use egui::{Color32, FontDefinitions, FontFamily, FontId};
use serde_json::json;
use toolkit::fonts::optional::{FILES, register};
use viu::view::SceneView;

fn identity(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut defs = FontDefinitions::default();
    register(&mut defs, dir)?;
    let ctx = egui::Context::default();
    ctx.set_fonts(defs);
    let roles = [
        ("sans heading", "Heading", "Liberation Sans", 700, FILES[1]),
        ("sans body", "Body", "Liberation Sans", 400, FILES[0]),
        ("sans medium", "Medium", "Liberation Sans", 500, FILES[0]),
        ("sans semibold", "Semibold", "Liberation Sans", 600, FILES[1]),
        ("mono date", "2026-11-22", "Noto Sans Mono", 400, FILES[2]),
        ("mono PR tag", "#215", "Noto Sans Mono", 700, FILES[2]),
    ];
    let mut nodes = json!({"root":{"widget":"column","children":[]}});
    for (i, (_, text, family, weight, _)) in roles.iter().enumerate() {
        let id = format!("r{i}");
        nodes["root"]["children"].as_array_mut().unwrap().push(json!(id));
        nodes[&id] = json!({"widget":"text","text":text,"font":family,"weight":weight,"size":24});
    }
    let source = format!("---\nscene: 1\nname: deck\ncitizen: deck\n---\n```mix\n{nodes}\n```\n");
    let mut view = SceneView::new(&source)?;
    let output = ctx.run_ui(egui::RawInput::default(), |ui| { view.show(ui); });
    for (role, text, _, _, file) in roles {
        let actual = output.shapes.iter().find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.job.text == text => Some(&t.galley),
            _ => None,
        }).expect("role painted by SceneView");
        let expected = ctx.fonts_mut(|fonts| fonts.layout_no_wrap(text.into(),
            FontId::new(24., FontFamily::Name(file.into())), Color32::WHITE));
        let glyphs = |galley: &egui::Galley| galley.rows.iter().flat_map(|r| r.glyphs.iter())
            .map(|g| (g.chr, g.uv_rect, g.advance_width)).collect::<Vec<_>>();
        assert_eq!(glyphs(actual), glyphs(&expected), "{role}: actual raster face differs");
        let mut db = fontdb::Database::new();
        db.load_font_file(dir.join(file))?;
        let face = db.faces().next().expect("one face");
        println!("FONT_IDENTITY {role}: {}", face.post_script_name);
    }
    Ok(())
}

#[test]
fn deck_font_identity_and_missing_bundle_control() {
    let dir = toolkit::fonts::optional::directory();
    identity(&dir).expect("bundled font identity");
    let temp = tempfile::tempdir().unwrap();
    for file in FILES { std::fs::copy(dir.join(file), temp.path().join(file)).unwrap(); }
    std::fs::rename(temp.path().join(FILES[0]), temp.path().join("absent.ttf")).unwrap();
    let error = identity(temp.path()).expect_err("missing bundled file must fail");
    println!("FONT_IDENTITY without fonts: FAIL (expected): {error}");
}
