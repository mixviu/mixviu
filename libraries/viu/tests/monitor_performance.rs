// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Program, Runtime};
#[test]
fn mix_authors_every_resource_page_and_native_widgets_draw_the_projected_blocks() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    let fixture = root.join("tests/fixtures/monitor-snapshot.json");
    runtime.load(Program {path:root.join("../../apps/monitor/performance-probe.mix"),scene:"probe".into(),citizen:"probe".into(),source:format!(r#"
$p=require("performance.mix");$h=require("history.mix")
$v=json_parse(read_file({}))
$s={{snapshot:$v,section:"cpu",node:"",rings:$h.record({{}},$v),connections:nil,connections_error:nil}}
on probe
  $s.section=$event.args.section
  $widget=$p.project($s)
  send "viu-local" "scenes.model" body={{generation:viu_context().generation,value:{{definition:$widget}}}}
end
"#,serde_json::to_string(&fixture).unwrap())}).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(runtime.try_completion().unwrap().error.is_none());
    for section in [
        "cpu",
        "memory",
        "disk:nvme1n1",
        "disk:nvme0n1",
        "disk:nvme2n1",
        "net:eth0",
        "tcp",
    ] {
        runtime
            .event("probe".into(), json!({"section":section}))
            .unwrap();
        rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let done = runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{section}: {:?}", done.error);
        let definition = done.model.unwrap()["definition"].clone();
        assert_eq!(definition["initial"]["selected"], section);
        assert_eq!(definition["initial"]["items"].as_array().unwrap().len(), 7);
        let widget = viu::widgets::Widget::decode(&definition).unwrap();
        let ctx = egui::Context::default();
        toolkit::install(&ctx, &toolkit::Theme::embedded());
        let mut widgets = viu::widgets::Widgets::default();
        for _ in 0..2 {
            widgets.begin_frame();
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1040., 680.),
                    )),
                    ..Default::default()
                },
                |ui| {
                    widgets.widget(ui, &widget, &json!({}));
                },
            )
            .drop_without_applying_deltas();
            widgets.finish_frame().unwrap();
        }
        assert!(widgets.geometry.contains_key("performance"));
        assert!(widget.admit(&json!({}), "choose", &json!("memory")).is_ok());
        assert!(
            widget
                .admit(&json!({}), "choose", &json!("not-a-resource"))
                .is_err()
        );
    }
}
