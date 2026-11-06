// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Completion, Program, Runtime, Services};
fn next(r: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(5)).unwrap();
    r.try_completion().unwrap()
}
fn worker(services: Services) -> (Runtime, mpsc::Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    let mut r = Runtime::with_services(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
        services,
    );
    r.load(Program {
        path: PathBuf::from("/tmp/raster-test.mix"),
        scene: "raster".into(),
        citizen: "raster".into(),
        source: r#"
on document
  try
    $r=viu_document($event.args.op,$event.args.body)
    reply(0,json_encode($r))
  catch $m reply(10,json_encode({message:$m})) end
end
on cancelled
  reply(0,json_encode({cancelled:viu_cancelled()}))
end
on crypto
  reply(0,json_encode({run:uuid(),digest:hash_sha256("abc")}))
end
"#
        .into(),
    })
    .unwrap();
    assert!(next(&r, &rx).error.is_none());
    (r, rx)
}
#[test]
fn reusable_documents_keep_history_and_worker_exports_the_same_annotated_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.png");
    let target = dir.path().join("export.png");
    image::RgbaImage::from_pixel(24, 18, image::Rgba([255, 255, 255, 255]))
        .save(&source)
        .unwrap();
    let services = Services::default();
    let (mut root, wakes) = worker(services.clone());
    root.event("crypto".into(), json!({})).unwrap();
    let crypto = next(&root, &wakes);
    assert!(crypto.error.is_none(), "{:?}", crypto.error);
    let crypto = crypto.reply.unwrap().body;
    assert_eq!(crypto["run"].as_str().unwrap().len(), 36);
    assert_eq!(
        crypto["digest"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    root.event(
        "document".into(),
        json!({"op":"open","body":{"path":source}}),
    )
    .unwrap();
    let opened = next(&root, &wakes).reply.unwrap();
    assert_eq!(opened.rc, 0);
    let handle = opened.body["handle"].clone();
    root.event("document".into(),json!({"op":"annotate","body":{"handle":handle,"shape":{"kind":"rectangle","points":[{"x":2,"y":2},{"x":20,"y":15}],"colour":[255,0,0,255],"width":4}}})).unwrap();
    let added = next(&root, &wakes).reply.unwrap();
    assert_eq!(added.rc, 0);
    assert_eq!(added.body["document"]["undo"], 1);
    let (mut job, job_wakes) = worker(services);
    job.event(
        "document".into(),
        json!({"op":"export","body":{"handle":handle,"path":target}}),
    )
    .unwrap();
    assert_eq!(next(&job, &job_wakes).reply.unwrap().rc, 0);
    let bytes = std::fs::read(&target).unwrap();
    let image = image::open(&target).unwrap().to_rgba8();
    assert!(image.pixels().any(|p| p.0[0] > 200 && p.0[1] < 50));
    job.event(
        "document".into(),
        json!({"op":"export","body":{"handle":handle,"path":target}}),
    )
    .unwrap();
    assert_eq!(next(&job, &job_wakes).reply.unwrap().rc, 10);
    assert_eq!(bytes, std::fs::read(&target).unwrap());
    for (op, dirty, objects) in [("undo", false, 0), ("redo", true, 1), ("saved", false, 1)] {
        root.event("document".into(), json!({"op":op,"body":{"handle":handle}}))
            .unwrap();
        let result = next(&root, &wakes).reply.unwrap();
        assert_eq!(result.rc, 0);
        assert_eq!(result.body["document"]["dirty"], dirty);
        assert_eq!(
            result.body["document"]["objects"].as_array().unwrap().len(),
            objects
        );
    }
    root.signal_cancel();
    root.event("cancelled".into(), json!({})).unwrap();
    assert_eq!(
        next(&root, &wakes).reply.unwrap().body,
        json!({"cancelled":true})
    );
    root.event(
        "document".into(),
        json!({"op":"close","body":{"handle":handle}}),
    )
    .unwrap();
    assert_eq!(next(&root, &wakes).reply.unwrap().rc, 0);
    job.event(
        "document".into(),
        json!({"op":"info","body":{"handle":handle}}),
    )
    .unwrap();
    assert_eq!(next(&job, &job_wakes).reply.unwrap().rc, 10);
}

#[test]
fn image_canvas_admission_rejects_invalid_gestures_and_tracks_live_enabled_state() {
    let value = json!({"canvas":true,"phase":"ready","path":"/tmp/frame.png","colour":"design.canvas","fit":"fit","enabled":true,"zoom":1,"pan":{"x":0,"y":0},"width":32,"height":24});
    let widget = viu::widgets::Widget::new("image", "image-canvas", "", value.clone());
    widget.validate(&value).unwrap();
    let model = json!({"values":{"image":value}});
    widget
        .admit(
            &model,
            "gesture",
            &json!({"points":[{"x":1,"y":2},{"x":5,"y":7}],"scale":2}),
        )
        .unwrap();
    assert!(
        widget
            .admit(&model, "gesture", &json!({"points":[],"scale":2}))
            .is_err()
    );
    assert!(widget.admit(&model, "zoom", &json!({"factor":0})).is_err());
    let mut disabled = model;
    disabled["values"]["image"]["enabled"] = json!(false);
    assert!(
        widget
            .admit(&disabled, "pan", &json!({"x":1,"y":2}))
            .is_err()
    );
}

#[test]
fn scene_normalized_integral_control_data_keeps_its_native_types() {
    let widget = viu::widgets::Widget::decode(&json!({"id":"inks","kind":"swatches","label":"Colour","initial":[194.0,23.0,37.0,255.0],"properties":{"items":[{"id":"red","label":"Red","colour":[194.0,23.0,37.0,255.0]}]}})).unwrap();
    assert_eq!(widget.initial, json!([194, 23, 37, 255]));
    assert!(viu::widgets::Widget::decode(&json!({"id":"inks","kind":"swatches","label":"Colour","initial":[194.5,23.0,37.0,255.0],"properties":{"items":[]}})).is_err());
}
