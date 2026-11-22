use super::*;

fn edited(model: &Value, edit: impl FnOnce(&mut Value)) -> Value {
    let mut model = model.clone();
    let source = model["presentation"]["scene"].as_str().unwrap();
    let start = source.find("```mix\n").unwrap() + 7;
    let end = source.rfind("\n```").unwrap();
    let mut nodes: Value = serde_json::from_str(&source[start..end]).unwrap();
    edit(&mut nodes);
    model["presentation"]["scene"] =
        json!(format!("{}{}{}", &source[..start], nodes, &source[end..]));
    model
}
fn remove(model: &Value, preset: &str) -> Value {
    edited(model, |nodes| {
        for p in nodes["root.0"]["primitives"].as_array_mut().unwrap() {
            p["effects"]
                .as_array_mut()
                .unwrap()
                .retain(|e| e["preset"] != preset);
        }
    })
}
fn eye_y(image: &image::RgbaImage) -> f64 {
    let mut sum = 0.;
    let mut n = 0.;
    for (x, y, p) in image.enumerate_pixels() {
        if x > 1400 && x < 2400 && y < 1100 && p[0] > 180 && p[1] < 40 && p[2] < 40 {
            sum += y as f64 + 0.5;
            n += 1.;
        }
    }
    sum / n
}
fn shadow_pixels(image: &image::RgbaImage) -> usize {
    image
        .enumerate_pixels()
        .filter(|(x, y, p)| {
            *x > 1400 && *x < 2400 && *y < 1200 && p[0] < 100 && p[0] > p[1].saturating_add(4)
        })
        .count()
}
pub(super) fn painted_centres(
    images: &[image::RgbaImage],
    geometry: &BTreeMap<String, egui::Rect>,
) {
    for id in ["root.0", "root.2", "root.4", "root.6", "root.8"] {
        let r = geometry[id];
        let mut left = u32::MAX;
        let mut right = 0;
        for (x, y, p) in images[8].enumerate_pixels() {
            if r.contains(egui::pos2(x as f32 / 2.4, y as f32 / 2.4))
                && p[0] > 50
                && p[1] > 100
                && p[2] > 100
            {
                left = left.min(x);
                right = right.max(x);
            }
        }
        let delta = ((left as f64 + right as f64 + 1.) / 2. - 1920.).abs();
        println!("STING_PAINT_CENTRE {id} delta_px={delta}");
        assert!(delta <= 4., "{id}: {left}..{right}");
    }
}
#[test]
fn removing_each_logo_primitive_is_detected() {
    let frames = samples();
    let (settled, geometry) = render(frames[8].clone());
    let logo = geometry["root.0"];
    let (early, _) = render(frames[0].clone());
    let (drawn, _) = render(frames[1].clone());
    let (blink, _) = render(frames[7].clone());
    let (floating, _) = render(frames[6].clone());
    let mut baseline = vec![settled.clone(); TIMES.len()];
    baseline[0] = early;
    baseline[1] = drawn;
    baseline[7] = blink;
    assert_eq!(animate_check(&baseline, logo), Ok(()));
    for preset in ["draw", "pop", "blink"] {
        let index = if preset == "blink" { 7 } else { 0 };
        let (mutant, _) = render(remove(&frames[index], preset));
        let mut images = baseline.clone();
        images[index] = mutant;
        assert!(
            animate_check(&images, logo).is_err(),
            "missing {preset} escaped"
        );
        println!("STING_MUTATION {preset}=FAIL (expected)");
    }
    let expected = (logo.top() as f64 + 130.) * 2.4;
    assert!((eye_y(&floating) - expected).abs() < 2.);
    let (no_float, _) = render(remove(&frames[6], "float"));
    assert!((eye_y(&no_float) - expected).abs() > 20.);
    assert!(shadow_pixels(&floating) > 1000);
    let (no_shadow, _) = render(edited(&frames[6], |n| {
        n["root.0"].as_object_mut().unwrap().remove("shadow");
    }));
    assert!(shadow_pixels(&no_shadow) < 500);
    println!("STING_SHADOW present={} removed={}",shadow_pixels(&floating),shadow_pixels(&no_shadow));
    println!("STING_MUTATION float=FAIL shadow=FAIL (expected)");
}
