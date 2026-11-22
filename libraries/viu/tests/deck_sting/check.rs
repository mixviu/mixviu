use super::*;
use std::collections::BTreeMap;

const TIMES: [f64; 9] = [0.1, 0.3, 0.6, 0.9, 1.2, 1.5, 2., 92. / 30., 172. / 30.];
fn render(model: Value) -> (image::RgbaImage, BTreeMap<String, egui::Rect>) {
    let mut view = SceneView::with_model(
        model["presentation"]["scene"].as_str().unwrap(),
        model.clone(),
    )
    .unwrap();
    let geometry = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
    let captured = geometry.clone();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1600., 900.))
        .with_pixels_per_point(2.4)
        .with_context_setup(|ctx| viu::fonts::install(ctx, 400, 600))
        .wgpu()
        .build_ui(move |ui| {
            // Match app_host's unframed logical presentation canvas, rather
            // than the harness central panel's default eight-pixel margin.
            let mut canvas=ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(1600.,900.))));
            canvas.set_clip_rect(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(1600.,900.)));
            view.show(&mut canvas);
            *captured.lock().unwrap() = view.geometry.clone();
        });
    harness.run();
    let image = harness.render().unwrap();
    let rects = geometry.lock().unwrap().clone();
    (image, rects)
}
fn samples() -> Vec<Value> {
    samples_for(None)
}
fn samples_for(copy:Option<(&str,&str)>) -> Vec<Value> {
    let mut deck = Deck::new();
    let dir=tempfile::tempdir().unwrap();
    if let Some((title,subtitle))=copy {
        let mut document:Value=serde_json::from_str(include_str!("../../../../apps/deck/examples/vfl-self-recording.json")).unwrap();
        document["slides"][0]["scene"]["root.4"]["text"]=json!(title);
        document["slides"][0]["scene"]["root.6"]["text"]=json!(subtitle);
        let path=dir.path().join("sting.json");std::fs::write(&path,document.to_string()).unwrap();
        assert_eq!(deck.call("deck.open",json!({"path":path})).reply.unwrap().rc,0);
    } else { deck.open(); }
    deck.call("deck.play", json!({"slides":["sting"],"beats":[[0,10]]}));
    deck.call("viu.presentation", json!({"time":100}));
    TIMES
        .iter()
        .map(|t| {
            deck.call("viu.presentation", json!({"time":100.+t}))
                .model
                .unwrap()
        })
        .collect()
}
fn coloured(image: &image::RgbaImage, rect: egui::Rect, red: bool) -> usize {
    image
        .enumerate_pixels()
        .filter(|(x, y, p)| {
            rect.contains(egui::pos2(*x as f32 / 2.4, *y as f32 / 2.4))
                && if red {
                    p[0] > 180 && p[1] < 40 && p[2] < 40
                } else {
                    p[0] > 220 && p[1] > 220 && p[2] > 220
                }
        })
        .count()
}
fn animate_check(images: &[image::RgbaImage], logo: egui::Rect) -> Result<(), String> {
    let ring = |i| coloured(&images[i], logo, false) as f64;
    let eye = |i| coloured(&images[i], logo, true) as f64;
    let fraction = ring(1) / ring(8);
    if !(0.01..0.5).contains(&fraction) {
        return Err(format!("ring at 0.3 must be partial: {fraction}"));
    }
    if eye(1) < 500. {
        return Err("eye has not popped at 0.3".into());
    }
    if eye(7) / eye(8) > 0.18 {
        return Err("eye did not close at blink".into());
    }
    Ok(())
}

fn element_pixels(image: &image::RgbaImage, rect: egui::Rect, cyan: bool) -> usize {
    image
        .enumerate_pixels()
        .filter(|(x, y, p)| {
            rect.contains(egui::pos2(*x as f32 / 2.4, *y as f32 / 2.4))
                && if cyan {
                    p[1] > 100 && p[2] > 120 && p[0] < 140
                } else {
                    p[0] > 100 && p[1] > 100 && p[2] > 100
                }
        })
        .count()
}
fn entrances(
    images: &[image::RgbaImage],
    geometry: &BTreeMap<String, egui::Rect>,
) -> Result<(), String> {
    for (id, delay, cyan) in [
        ("root.2", 0.5, false),
        ("root.4", 0.8, false),
        ("root.6", 1.3, true),
        ("root.8", 1.6, true),
    ] {
        let rect = geometry[id];
        let settled = element_pixels(&images[8], rect, cyan);
        if settled < 100 {
            return Err(format!("{id} never appeared: {settled}"));
        }
        for (i, t) in TIMES.iter().enumerate() {
            let pixels = element_pixels(&images[i], rect, cyan);
            if *t < delay && pixels > 5 {
                return Err(format!("{id} visible before delay at {t}: {pixels}"));
            }
            if *t > delay + 0.9 && pixels < settled / 2 {
                return Err(format!(
                    "{id} absent after delay at {t}: {pixels}/{settled}"
                ));
            }
        }
    }
    Ok(())
}
#[test]
fn sting_entrance_paints_and_rejects_a_static_control() {
    let frames = samples();
    for (i, t) in TIMES.iter().enumerate() {
        for (id, delay) in [
            ("root.2", 0.5),
            ("root.4", 0.8),
            ("root.6", 1.3),
            ("root.8", 1.6),
        ] {
            let effect = &frames[i]["build"][id]["paint_effect"];
            assert!((effect["time"].as_f64().unwrap() - (t - delay)).abs() < 1e-6);
        }
    }
    let mut images = Vec::new();
    let mut final_geometry = BTreeMap::new();
    for model in frames {
        let (image, rects) = render(model);
        images.push(image);
        final_geometry = rects;
    }
    let logo = final_geometry["root.0"];
    assert_eq!(animate_check(&images, logo), Ok(()));
    assert_eq!(entrances(&images, &final_geometry), Ok(()));
    println!(
        "STING_ENTRANCE animated=PASS ring_fraction_0.3={:.6} blink_ratio={:.6}",
        coloured(&images[1], logo, false) as f64 / coloured(&images[8], logo, false) as f64,
        coloured(&images[7], logo, true) as f64 / coloured(&images[8], logo, true) as f64
    );
    let control = vec![images[8].clone(); TIMES.len()];
    let rejected = animate_check(&control, logo).unwrap_err();
    assert!(entrances(&control, &final_geometry).is_err());
    println!("STING_ENTRANCE static=FAIL (expected): {rejected}");
    for id in ["root.0", "root.2", "root.4", "root.6", "root.8"] {
        let delta = (final_geometry[id].center().x * 2.4 - 1920.).abs();
        println!("STING_CENTRE {id} delta_px={delta}");
        assert!(delta <= 4.);
    }
}

#[test]
#[ignore = "explicit CBC evidence: set STING_ORACLE to Chrome frame directory"]
fn chrome_entrance_contact_sheet() {
    let oracle = PathBuf::from(std::env::var("STING_ORACLE").expect("STING_ORACLE"));
    let out = std::env::temp_dir().join("mixviu-deck-sting");
    std::fs::create_dir_all(&out).unwrap();
    let title=std::env::var("STING_TITLE").expect("STING_TITLE");
    let subtitle=std::env::var("STING_SUBTITLE").expect("STING_SUBTITLE");
    let frames = samples_for(Some((&title,&subtitle)));
    let mut sheet = image::RgbaImage::new(1920, 540 * TIMES.len() as u32);
    let mut native = Vec::new();
    let mut chrome = Vec::new();
    let mut logo = egui::Rect::NOTHING;
    let mut final_geometry = BTreeMap::new();
    for (i, (model, t)) in frames.into_iter().zip(TIMES).enumerate() {
        let (image, geometry) = render(model);
        logo = geometry["root.0"];
        final_geometry = geometry;
        image.save(out.join(format!("native-{t}.png"))).unwrap();
        let reference = image::open(oracle.join(format!("f{:05}.png",(t*30.).round() as u32)))
            .unwrap()
            .to_rgba8();
        assert_eq!(image.dimensions(), (3840, 2160));
        assert_eq!(reference.dimensions(), image.dimensions());
        for (column, img) in [&reference, &image].into_iter().enumerate() {
            let small =
                image::imageops::resize(img, 960, 540, image::imageops::FilterType::Lanczos3);
            image::imageops::replace(&mut sheet, &small, column as i64 * 960, i as i64 * 540);
        }
        native.push(image);
        chrome.push(reference);
    }
    let path = out.join("chrome-native-contact-sheet.png");
    sheet.save(&path).unwrap();
    println!("STING_CONTACT_SHEET={}", path.display());
    assert_eq!(animate_check(&native, logo), Ok(()));
    assert_eq!(entrances(&native, &final_geometry), Ok(()));
    // Oracle entrance regions use the same logical column, allowing for font
    // metrics by splitting the whitespace between its five element bands.
    let oracle_geometry = BTreeMap::from([
        (
            "root.2".into(),
            egui::Rect::from_min_max(egui::pos2(0., 440.), egui::pos2(1600., 490.)),
        ),
        (
            "root.4".into(),
            egui::Rect::from_min_max(egui::pos2(0., 490.), egui::pos2(1600., 630.)),
        ),
        (
            "root.6".into(),
            egui::Rect::from_min_max(egui::pos2(0., 630.), egui::pos2(1600., 700.)),
        ),
        (
            "root.8".into(),
            egui::Rect::from_min_max(egui::pos2(650., 700.), egui::pos2(950., 750.)),
        ),
    ]);
    assert_eq!(entrances(&chrome, &oracle_geometry), Ok(()));
    assert_eq!(
        animate_check(
            &chrome,
            egui::Rect::from_min_max(egui::pos2(660., 120.), egui::pos2(940., 420.))
        ),
        Ok(())
    );
    for (i, t) in TIMES.iter().enumerate() {
        let n = coloured(&native[i], logo, true) as f64 / coloured(&native[8], logo, true) as f64;
        let c = coloured(
            &chrome[i],
            egui::Rect::from_min_max(egui::pos2(660., 120.), egui::pos2(940., 420.)),
            true,
        ) as f64
            / coloured(
                &chrome[8],
                egui::Rect::from_min_max(egui::pos2(660., 120.), egui::pos2(940., 420.)),
                true,
            ) as f64;
        println!("STING_ORACLE time={t} eye_ratio chrome={c:.6} native={n:.6}");
        assert!((n - c).abs() < 0.18, "eye at {t}");
    }
    // The oracle uses the same centred logo box. Compare area fractions, so
    // antialiasing and the independently supplied font roles cannot dominate.
    let f = |images: &[image::RgbaImage]| {
        coloured(&images[1], logo, false) as f64 / coloured(&images[8], logo, false) as f64
    };
    println!(
        "STING_ORACLE ring_fraction chrome={} native={}",
        f(&chrome),
        f(&native)
    );
    assert!((f(&chrome) - f(&native)).abs() < 0.04);
    let path = out.join("chrome-native-contact-sheet.png");
    sheet.save(&path).unwrap();
    println!("STING_CONTACT_SHEET={}", path.display());
}
