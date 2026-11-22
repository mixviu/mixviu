use super::*;

#[test]
fn sting_font_identity_rows() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../share/fonts");
    let mut defs = egui::FontDefinitions::default();
    for file in [
        "LiberationSans-Regular.ttf",
        "LiberationSans-Bold.ttf",
        "NotoSansMono-Regular.ttf",
        "NotoSansMono-Medium.ttf",
        "NotoSansMono-Bold.ttf",
        "NotoSans-Regular.ttf",
    ] {
        defs.font_data.insert(
            file.into(),
            egui::FontData::from_owned(std::fs::read(dir.join(file)).expect("PR #235 font bundle"))
                .into(),
        );
        defs.families
            .insert(egui::FontFamily::Name(file.into()), vec![file.into()]);
    }
    for (family, weight, file) in [
        ("Liberation Sans", 400, "LiberationSans-Regular.ttf"),
        ("Liberation Sans", 700, "LiberationSans-Bold.ttf"),
        ("Noto Sans Mono", 400, "NotoSansMono-Regular.ttf"),
        ("Noto Sans Mono", 500, "NotoSansMono-Medium.ttf"),
        ("Noto Sans Mono", 700, "NotoSansMono-Bold.ttf"),
    ] {
        defs.families.insert(
            egui::FontFamily::Name(format!("{family}/{weight}").into()),
            vec![file.into()],
        );
        if weight == 400 {
            defs.families
                .insert(egui::FontFamily::Name(family.into()), vec![file.into()]);
        }
    }
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 700);
    ctx.set_fonts(defs);
    let model = samples().pop().unwrap();
    let mut view = SceneView::with_model(
        model["presentation"]["scene"].as_str().unwrap(),
        model.clone(),
    )
    .unwrap();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600., 900.),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    );
    for (role, text, skip, take, size, file) in [
        (
            "title 700",
            "The desk records itself",
            0,
            usize::MAX,
            104.,
            "LiberationSans-Bold.ttf",
        ),
        ("Mix 600", "MIXVIU", 0, 3, 34., "LiberationSans-Bold.ttf"),
        ("Viu 400", "MIXVIU", 3, 3, 34., "LiberationSans-Regular.ttf"),
        (
            "subtitle mono 400",
            "MixViu Adventures",
            0,
            usize::MAX,
            36.,
            "NotoSansMono-Regular.ttf",
        ),
    ] {
        let actual = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == text => Some(t.galley.clone()),
                _ => None,
            })
            .expect("sting role painted");
        let run = text.chars().skip(skip).take(take).collect::<String>();
        let expected = ctx.fonts_mut(|f| {
            f.layout_no_wrap(
                run,
                egui::FontId::new(size, egui::FontFamily::Name(file.into())),
                egui::Color32::WHITE,
            )
        });
        let glyphs = |g: &egui::Galley, skip, take| {
            g.rows
                .iter()
                .flat_map(|r| r.glyphs.iter())
                .skip(skip)
                .take(take)
                .map(|g| (g.chr, g.uv_rect))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            glyphs(&actual, skip, take),
            glyphs(&expected, 0, usize::MAX),
            "{role}: wrong raster face"
        );
        println!("STING_FONT_IDENTITY {role}={file}");
    }
    output.textures_delta.clear();
}
