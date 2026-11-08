// SPDX-License-Identifier: MIT OR Apache-2.0
//! Same mailbox/model/fresh-adapter/egui method as ports_round5's Email probe.
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use viu::runtime::{Completion, Runtime};
fn done(r: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(30)).unwrap();
    let c = r.try_completion().unwrap();
    assert!(c.error.is_none(), "{:?}", c.error);
    assert!(c.reply.as_ref().is_none_or(|r| r.rc == 0), "{:?}", c.reply);
    if !c.stdout.is_empty() {
        print!("{}", c.stdout);
    }
    c
}
fn stats(mut a: Vec<f64>) -> Value {
    a.sort_by(f64::total_cmp);
    json!({"median":a[a.len()/2],"p95":a[(a.len()*95).div_ceil(100)-1],"p99":a[(a.len()*99).div_ceil(100)-1],"raw_sorted":a})
}
#[test]
fn dopus_2000_sort_refresh_stage_profile() {
    let tmp = tempfile::tempdir().unwrap();
    let folder = tmp.path().join("rows");
    std::fs::create_dir(&folder).unwrap();
    let empty = tmp.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    for i in 0..2000 {
        std::fs::write(folder.join(format!("file-{i:04}.txt")), "fixture").unwrap();
    }
    let config = json!({"schema_version":2,"left":{"path":folder,"previews":false},"right":{"path":empty,"previews":false}});
    std::fs::write(tmp.path().join("config.conf.mix"), config.to_string()).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    let bundle = viu::launch::load(
        &path,
        &[
            "--profile".into(),
            tmp.path().to_string_lossy().into_owned(),
        ],
    )
    .unwrap();
    let (tx, rx) = mpsc::channel();
    let mut r = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    r.load(bundle.program).unwrap();
    done(&r, &rx);
    for action in ["view.sort-name", "view.refresh"] {
        let mut callback = vec![];
        let mut ack = vec![];
        let mut adapter = vec![];
        let mut draw = vec![];
        let mut clone_cost = vec![];
        let mut publish_cost = vec![];
        for n in 0..24 {
            let t = Instant::now();
            r.event("dopus.execute".into(), json!({"id":action}))
                .unwrap();
            let c = done(&r, &rx);
            let ack_ms = t.elapsed().as_secs_f64() * 1000.;
            let model = c.model.unwrap();
            assert_eq!(
                model["values"]["left-tree"]["rows"]
                    .as_array()
                    .unwrap()
                    .len(),
                2000
            );
            assert_eq!(model["panes"][0]["error"], Value::Null);
            let mv = viu::runtime::from_json(model.clone()).unwrap();
            let t = Instant::now();
            let copied = mv.clone();
            let clone_ms = t.elapsed().as_secs_f64() * 1000.;
            let t = Instant::now();
            let json_copy = viu::runtime::to_json(&copied).unwrap();
            let publish_ms = t.elapsed().as_secs_f64() * 1000.;
            assert_eq!(json_copy, model);
            let t = Instant::now();
            let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
            let adapter_ms = t.elapsed().as_secs_f64() * 1000.;
            let ctx = egui::Context::default();
            viu::appearance::install(&ctx);
            let t = Instant::now();
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1100., 900.),
                    )),
                    ..Default::default()
                },
                |ui| {
                    view.show(ui);
                },
            )
            .drop_without_applying_deltas();
            let draw_ms = t.elapsed().as_secs_f64() * 1000.;
            if n >= 3 {
                callback.push(c.elapsed.as_secs_f64() * 1000.);
                ack.push(ack_ms);
                adapter.push(adapter_ms);
                draw.push(draw_ms);
                clone_cost.push(clone_ms);
                publish_cost.push(publish_ms);
            }
        }
        println!(
            "DOPUS_2000 {}",
            json!({"action":action,"samples":21,"callback_ms":stats(callback),"mailbox_ack_ms":stats(ack),"fresh_adapter_ms":stats(adapter),"cold_context_draw_ms":stats(draw),"mix_value_clone_ms":stats(clone_cost),"standalone_to_json_ms":stats(publish_cost)})
        );
    }
}
