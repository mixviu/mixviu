// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{bundle::Bundle, runtime::Runtime};

#[test]
fn prefs_mounts_the_shared_frame_and_owns_caption_close() {
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/prefs"),
        &[],
    )
    .unwrap();
    assert_eq!(bundle.manifest.frame.as_deref(), Some("gold"));
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    let model = done.model.unwrap();
    assert_eq!(model["values"]["gold-title"], "Preferences");
    let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
    let ctx = eframe::egui::Context::default();
    viu::icons::install(&ctx);
    ctx.run_ui(
        eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                eframe::egui::vec2(1040.0, 680.0),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
    assert!(view.geometry.contains_key("gold-header"));
    assert!(view.geometry.contains_key("browser"));
    runtime.event("viu.frame.close".into(), json!({})).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    assert_eq!(done.model.unwrap()["window_close"], true);
}

#[test]
fn prefs_search_is_read_only_and_every_gold_page_mounts() {
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/prefs"),
        &[],
    )
    .unwrap();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    for query in ["density", "BACKGROUND", "terminal.", ""] {
        runtime
            .event("prefs.search".into(), json!({"query":query}))
            .unwrap();
        rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let done = runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{:?}", done.error);
        assert!(
            done.model.is_none(),
            "read-only search published a window change"
        );
        let reply = done.reply.unwrap();
        assert_eq!(reply.rc, 0);
        assert_eq!(reply.body["query"], query);
        assert!(reply.body["hits"].is_array());
    }
    let pages: serde_json::Value =
        serde_json::from_str(include_str!("../../../apps/prefs/pages.json")).unwrap();
    assert_eq!(pages.as_array().unwrap().len(), 42);
    let ctx = eframe::egui::Context::default();
    viu::icons::install(&ctx);
    for page in pages.as_array().unwrap() {
        runtime
            .event("prefs.page.open".into(), json!({"page":page["id"]}))
            .unwrap();
        rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let done = runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{}: {:?}", page["id"], done.error);
        let reply = done.reply.unwrap();
        assert_eq!(reply.rc, 0, "page {}: {}", page["id"], reply.body);
        let mut view = viu::gold_frame::mount(&bundle.scene, done.model.unwrap()).unwrap();
        ctx.run_ui(
            eframe::egui::RawInput {
                screen_rect: Some(eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::ZERO,
                    eframe::egui::vec2(1040.0, 680.0),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        )
        .drop_without_applying_deltas();
        assert!(view.geometry.contains_key("browser"));
    }
}

#[test]
fn browser_refuses_hidden_disabled_removed_and_unknown_choice_rows() {
    let value = json!({"title":"Example","pages":[],"sections":[{"rows":[
        {"id":"disabled","title":"Disabled","kind":"toggle","blocked":"backend missing","value":false},
        {"id":"hidden","title":"Hidden","kind":"toggle","shown":false,"value":false},
        {"id":"choice","title":"Choice","kind":"choice","choices":[{"value":"a","label":"A"}],"value":"a"}
    ]}]});
    for payload in [
        json!({"row":"disabled","value":true}),
        json!({"row":"hidden","value":true}),
        json!({"row":"removed","value":true}),
        json!({"row":"choice","value":"b"}),
    ] {
        assert!(viu::settings_browser::admit(&value, "edit", &payload).is_err());
    }
    assert!(
        viu::settings_browser::admit(&value, "edit", &json!({"row":"choice","value":"a"})).is_ok()
    );
}

#[test]
fn search_controls_are_admitted_only_for_the_current_result_page() {
    let mut value = json!({"title":"Settings","pages":[],"sections":[],"searching":true,"search_groups":[
        {"page":"input","sections":[{"rows":[{"id":"enabled","title":"Enabled","kind":"toggle","value":false}]}]}
    ]});
    let payload = json!({"page":"input","row":"enabled","value":true});
    assert!(viu::settings_browser::admit(&value, "edit", &payload).is_ok());
    assert!(
        viu::settings_browser::admit(
            &value,
            "edit",
            &json!({"page":"other","row":"enabled","value":true})
        )
        .is_err()
    );
    value["searching"] = json!(false);
    assert!(viu::settings_browser::admit(&value, "edit", &payload).is_err());
}

#[test]
fn split_record_details_preserve_disabled_and_current_field_admission() {
    let value = json!({"title":"Accounts","pages":[],"sections":[{"rows":[
        {"id":"accounts","title":"Accounts","kind":"records","detail_rows":[
            {"id":"name","title":"Name","kind":"text","value":""},
            {"id":"password","title":"Password","kind":"secret","blocked":"busy"}
        ]}
    ]}]});
    assert!(
        viu::settings_browser::admit(&value, "edit", &json!({"row":"name","value":"Example"}))
            .is_ok()
    );
    assert!(
        viu::settings_browser::admit(&value, "edit", &json!({"row":"password","value":"secret"}))
            .is_err()
    );
    assert!(
        viu::settings_browser::admit(&value, "edit", &json!({"row":"name","value":false})).is_err()
    );
}

#[test]
fn email_uses_native_registry_admission_and_redacts_credentials() {
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(viu::runtime::Program {
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/prefs/email-test.mix"),
            scene: "prefs".into(),
            citizen: "prefs".into(),
            source: include_str!("../../../apps/prefs/email-test.mix").into(),
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(
        done.error.is_none(),
        "{:?}\n{}\n{}",
        done.error,
        done.stdout,
        done.stderr
    );
}

#[test]
fn settings_fences_appearance_and_navigation_use_the_embedded_native_path() {
    for (name, source) in [
        (
            "core-test.mix",
            include_str!("../../../apps/prefs/core-test.mix"),
        ),
        (
            "look-test.mix",
            include_str!("../../../apps/prefs/look-test.mix"),
        ),
        (
            "navigation-test.mix",
            include_str!("../../../apps/prefs/navigation-test.mix"),
        ),
    ] {
        let (tx, rx) = mpsc::channel();
        let mut runtime = Runtime::new(
            8,
            Arc::new(move || {
                let _ = tx.send(());
            }),
            None,
        );
        runtime
            .load(viu::runtime::Program {
                path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../apps/prefs")
                    .join(name),
                scene: "prefs".into(),
                citizen: "prefs".into(),
                source: source.into(),
            })
            .unwrap();
        rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let done = runtime.try_completion().unwrap();
        assert!(
            done.error.is_none(),
            "{name}: {:?}\n{}\n{}",
            done.error,
            done.stdout,
            done.stderr
        );
    }
}
