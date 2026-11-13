// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool, mpsc},
    time::Duration,
};
use viu::{
    bundle::Bundle,
    runtime::{Runtime, Services},
};
mod fixture {
    include!("support/documents.rs");
}
struct Accepted;
struct Compositor(std::sync::Arc<AtomicBool>);
impl mix::BusHandler for Compositor {
    fn send<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<(i32, mix::value::Value)>> {
        Box::pin(async move {
            assert_eq!(target, "comp-status-test");
            let args = viu::runtime::to_json(args)?;
            let body = args.get("body").unwrap_or(&args);
            if command == "comp.window.wait" { assert_eq!(body["match"]["app_id"], "example.ced"); }
            assert!(
                self.0.load(std::sync::atomic::Ordering::SeqCst),
                "no request before registration"
            );
            let body = match command {
                "comp.window.wait" => json!({"mapped":true}),
                "comp.windows.list" => {
                    json!({"windows":[{"id":7,"generation":3,"app_id":"example.ced","pid":std::process::id()}]})
                }
                "comp.window.restore" => json!({"minimized":false}),
                "comp.window.focus" => json!({"focused":true}),
                _ => panic!("unexpected {command}"),
            };
            Ok((0, viu::runtime::from_json(body)?))
        })
    }
    fn emit<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
        _: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<()>> {
        Box::pin(async { Ok(()) })
    }
    fn port_exists<'a>(
        &'a self,
        target: &'a str,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<bool>> {
        Box::pin(async move { assert_eq!(target, "comp-status-test"); Ok(self.0.load(std::sync::atomic::Ordering::SeqCst)) })
    }
    fn next_incoming<'a>(
        &'a self,
    ) -> mix::evaluator::BusFuture<'a, Option<mix::evaluator::IncomingEvent>> {
        Box::pin(async { None })
    }
}

#[test]
fn ced_worker_recovers_when_compositor_registers_after_start() {
    let available = Arc::new(AtomicBool::new(false));
    let bus = available.clone();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        4,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        Some(Box::new(move || std::rc::Rc::new(Compositor(bus.clone())))),
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/ced/job.mix");
    runtime
        .load(viu::runtime::Program {
            source: std::fs::read_to_string(&path).unwrap() + r#"
$core=require("core.mix")
on test.queue
  $s=$core.queue_job({serial:0,queue:[]},"show",$event.args)
  reply(0,json_encode($s.queue[0]))
end
"#,
            path,
            scene: "worker-test".into(),
            citizen: "ced".into(),
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(runtime.try_completion().unwrap().error.is_none());
    for registered in [false, true] {
        available.store(registered, std::sync::atomic::Ordering::SeqCst);
        runtime.event("test.queue".into(), json!({"app_id":"example.ced","comp":"comp-status-test"})).unwrap();
        rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let queued = runtime.try_completion().unwrap().reply.unwrap().body;
        assert_eq!(queued, json!({"id":"ced-1","kind":"show","payload":{"app_id":"example.ced","comp":"comp-status-test"}}));
        runtime
            .event(
                "job.run".into(),
                queued,
            )
            .unwrap();
        rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let done = runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{:?}", done.error);
        let reply = done.reply.unwrap();
        assert_eq!(reply.rc, 0);
        let body = reply.body;
        assert_eq!(body["shown"], registered);
    }
}
#[test]
fn every_ced_worker_program_parses_with_its_real_includes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/ced/job.mix");
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        4,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(viu::runtime::Program {
            source: std::fs::read_to_string(&path).unwrap(),
            path,
            scene: "ced-worker".into(),
            citizen: "ced".into(),
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
}
impl mix::BusHandler for Accepted {
    fn send<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
        _: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<(i32, mix::value::Value)>> {
        Box::pin(async { Ok((0, mix::value::Value::Nil)) })
    }
    fn emit<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
        _: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<()>> {
        Box::pin(async { Ok(()) })
    }
    fn port_exists<'a>(
        &'a self,
        _: &'a str,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<bool>> {
        Box::pin(async { Ok(false) })
    }
    fn next_incoming<'a>(
        &'a self,
    ) -> mix::evaluator::BusFuture<'a, Option<mix::evaluator::IncomingEvent>> {
        Box::pin(async { None })
    }
}
struct App {
    runtime: Runtime,
    ready: mpsc::Receiver<()>,
    calls: viu::calls::Calls,
    timers: viu::timers::Timers,
    docs: viu::documents::Documents,
}
impl App {
    fn new() -> Self {
        let mut bundle = Bundle::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/ced"),
            &[],
        )
        .unwrap();
        bundle.program.source.push_str(
            "\n$state.tabs=[viu_document(\"state\",\"1\",{})]\n$state.next_tab=1
$state.classified={\"1\":true}\n$ced_core.publish_ced($state)\n",
        );
        bundle.program.source.push_str(
            r#"
on test.mail
  $state.mail["1"]=$event.args
  $state.session_loaded=true;$state.session_path="/tmp/ced-test-session.json"
  $state.session_seen=$ced_session.session_value($state);$state.session_dirty=false
end
on test.job
  $state.running["status-test"]={id:"status-test",kind:$event.args.kind ?? "show",payload:{}}
end
"#,
        );
        let docs = viu::documents::Documents::default();
        fixture::seed(&docs, "1", "/tmp/contract.rs", "λ one\ntwo\n");
        let calls = viu::calls::Calls::default();
        let timers = viu::timers::Timers::default();
        let (tx, ready) = mpsc::channel();
        let mut runtime = Runtime::with_services(
            16,
            Arc::new(move || {
                let _ = tx.send(());
            }),
            Some(Box::new(|| std::rc::Rc::new(Accepted))),
            Services {
                documents: Some(docs.clone()),
                calls: Some(calls.clone()),
                timers: Some(timers.clone()),
                ..Default::default()
            },
        );
        runtime.load(bundle.program).unwrap();
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        let first = runtime.try_completion().unwrap();
        assert!(first.error.is_none(), "{:?}", first.error);
        Self {
            runtime,
            ready,
            calls,
            timers,
            docs,
        }
    }
    fn command(&mut self, name: &str, body: Value) -> tokio::sync::oneshot::Receiver<(u8, Value)> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let token = self
            .calls
            .begin(tx, Arc::new(AtomicBool::new(false)))
            .unwrap();
        self.runtime
            .event_with_headers(
                name.into(),
                body,
                BTreeMap::from([
                    ("viu-call".into(), token.clone()),
                    ("from".into(), "contract-agent".into()),
                    ("broker_origin".into(), "local".into()),
                ]),
            )
            .unwrap();
        self.ready.recv_timeout(Duration::from_secs(5)).unwrap();
        let done = self.runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{name}: {:?}", done.error);
        let r = done.reply.unwrap();
        self.calls.complete(&token, r.rc, r.body);
        rx
    }
    fn answered(&mut self, name: &str, body: Value) -> (u8, Value) {
        self.command(name, body)
            .try_recv()
            .expect("synchronous reply")
    }
}

#[test]
fn job_notice_is_readable_retains_diagnostics_and_clears_on_success() {
    let mut app = App::new();
    app.answered("test.job", json!({}));
    let error = json!({"error_code":"ACTIVATION","message":"Cannot focus the window."});
    app.answered("viu.job.completed", json!({"tag":{"owner":{"id":"status-test","kind":"show","payload":{}}},"rc":10,"body":error}));
    let (_, state) = app.answered("ced.specimen", json!({}));
    assert_eq!(state["status"], "Failed: Cannot focus the window.");
    assert_eq!(state["last_job_error"]["body"], error);
    app.answered("test.job", json!({"kind":"notify"}));
    app.answered("viu.job.completed", json!({"tag":{"owner":{"id":"status-test","kind":"notify","payload":{}}},"rc":0,"body":{"fallback":false}}));
    let (_, state) = app.answered("ced.specimen", json!({}));
    assert_eq!(state["status"], "Failed: Cannot focus the window.");
    assert_eq!(state["last_job_error"]["body"], error);
    app.answered("test.job", json!({}));
    app.answered("viu.job.completed", json!({"tag":{"owner":{"id":"status-test","kind":"show","payload":{}}},"rc":0,"body":{"shown":true}}));
    let (_, state) = app.answered("ced.specimen", json!({}));
    assert_eq!(state["status"], "");
    assert_eq!(state["last_job_error"]["body"], error);
}

#[test]
fn non_map_job_failures_keep_diagnostics_and_use_plain_fallback() {
    let mut app = App::new();
    for body in [json!("transport stopped"), json!(17), Value::Null, json!({"message":17})] {
        app.answered("test.job", json!({}));
        app.answered("viu.job.completed", json!({"tag":{"owner":{"id":"status-test","kind":"show","payload":{}}},"rc":10,"body":body}));
        let (_, state) = app.answered("ced.specimen", json!({}));
        assert_eq!(state["status"], "Failed: The operation failed.");
        assert_eq!(state["last_job_error"]["body"], body);
    }
}

#[test]
fn status_without_right_fields_and_priority_error_stay_visible_at_desktop_scales() {
    for scale in [1.0, 2.5] {
        for right in [json!([]), json!(["INS", "Saved"])] {
            for message in [Value::Null, json!("Cannot save the document.")] {
                let ctx = egui::Context::default();
                viu::appearance::install(&ctx);
                ctx.set_pixels_per_point(scale);
                let widget = viu::widgets::Widget::new("status", "status-line", "", json!({"left":"Long notice 界 ".repeat(300),"right":right,"message":message,"error":false}));
                let mut widgets = viu::widgets::Widgets::default();
                let mut error_colour = egui::Color32::TRANSPARENT;
                let mut output = ctx.run_ui(egui::RawInput { screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(3840.0/scale,2160.0/scale))),..Default::default() },|ui| {
                    error_colour=ui.visuals().error_fg_color;
                    widgets.begin_frame();widgets.widget(ui,&widget,&json!({}));widgets.finish_frame().unwrap();
                });
                output.textures_delta.clear();
                let texts: Vec<_> = output.shapes.iter().filter_map(|s| if let egui::Shape::Text(t)=&s.shape {Some((s.clip_rect,t))} else {None}).collect();
                let (clip,left)=texts.iter().find(|(_,t)|t.galley.text().starts_with("Long notice")).expect("notice painted");
                let left_bounds=left.galley.rect.translate(left.pos.to_vec2()).intersect(*clip);
                assert!(left_bounds.width()>10.0,"scale {scale}: visible left text");
                assert!(left.galley.elided,"left text must ellipsise");
                if message.is_string() {
                    let (clip,error)=texts.iter().find(|(_,t)|t.galley.text()==message.as_str().unwrap()).expect("error has its own label");
                    assert!(!error.galley.elided,"full error message takes priority");
                    assert!(error.galley.job.sections.iter().all(|s|s.format.color==error_colour));
                    let bounds=error.galley.rect.translate(error.pos.to_vec2());
                    assert!(clip.contains_rect(bounds),"error is fully visible");
                    assert!(!left_bounds.intersects(bounds));
                }
                output.drop_without_applying_deltas();
            }
        }
    }
}

#[test]
fn long_status_paint_never_intersects_right_fields_at_desktop_scales() {
    for scale in [1.0, 2.5] {
        let ctx = egui::Context::default();
        viu::appearance::install(&ctx);
        ctx.set_pixels_per_point(scale);
        let widget = viu::widgets::Widget::new(
            "status",
            "status-line",
            "",
            json!({"left":"Long activation failure ".repeat(100),"right":["INS","Rust","UTF-8 · LF","rev 0","Saved"],"icon":"plug"}),
        );
        let mut widgets = viu::widgets::Widgets::default();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(3840.0 / scale, 2160.0 / scale),
                )),
                ..Default::default()
            },
            |ui| {
                widgets.begin_frame();
                widgets.widget(ui, &widget, &json!({}));
                widgets.finish_frame().unwrap();
            },
        );
        output.textures_delta.clear();
        let mut left = None;
        let mut right = Vec::new();
        for clipped in &output.shapes {
            if let egui::Shape::Text(text) = &clipped.shape {
                let bounds = text
                    .galley
                    .rect
                    .translate(text.pos.to_vec2())
                    .intersect(clipped.clip_rect);
                if text.galley.text().starts_with("Long activation") {
                    left = Some(bounds);
                }
                if ["INS", "Rust", "UTF-8 · LF", "rev 0", "Saved"].contains(&text.galley.text()) {
                    right.push(bounds);
                }
            }
        }
        assert_eq!(right.len(), 5);
        let left = left.expect("left notice painted");
        for field in right {
            assert!(
                !left.intersects(field),
                "scale {scale}: {left:?} intersects {field:?}"
            );
        }
        output.drop_without_applying_deltas();
    }
}
#[test]
fn gold_info_tabs_state_unicode_selection_and_refusal_shapes() {
    let mut app = App::new();
    let (rc, info) = app.answered("ced.info", json!({}));
    assert_eq!(rc, 0);
    assert_eq!(info["tabs"], 1);
    assert!(info["edit"].is_object());
    assert_eq!(info["headless"], false);
    let (_, tabs) = app.answered("ced.tabs", json!({}));
    assert_eq!(tabs["active"], 1);
    assert_eq!(tabs["tabs"][0]["preview"], "source");
    let (rc, state) = app.answered("ced.state", json!({"text":true}));
    assert_eq!(rc, 0);
    assert_eq!(state["text"], "λ one\ntwo\n");
    assert_eq!(
        state["text_hash"],
        blake3::hash("λ one\ntwo\n".as_bytes()).to_hex().to_string()
    );
    assert!(state["conflicts"].is_array());
    let (rc, selected) = app.answered(
        "ced.select",
        json!({"anchor":{"line":1,"col":2},"head":"end"}),
    );
    assert_eq!(rc, 0);
    assert_eq!(selected["selection"]["anchor"]["offset"], 2);
    let (rc, bad) = app.answered("ced.select", json!({"anchor":1,"head":0}));
    assert_eq!(rc, 10);
    assert_eq!(bad["error_code"], "INVALID_ARGUMENT");
    assert_eq!(bad["reason"], "bad_position");
    let (rc, missing) = app.answered("ced.focus", json!({"tab":999}));
    assert_eq!(rc, 10);
    assert_eq!(missing["error_code"], "NOT_FOUND");
    assert!(missing["reason"].is_null());
    let (rc, typed) = app.answered("ced.type", json!({"text":" replaced"}));
    assert_eq!(rc, 0);
    assert_eq!(typed["tab"], 1);
    assert!(typed["pending"].as_u64().unwrap() > 0);
    assert_eq!(
        app.docs.call("read", "1", json!({})).unwrap()["text"],
        "λ replaced"
    );
}
#[test]
fn gold_waits_are_deferred_and_event_driven_and_validate_exactly_one_condition() {
    let mut app = App::new();
    let (rc, ready) = app.answered("ced.wait", json!({"idle":true,"timeout_ms":1000}));
    assert_eq!(rc, 0);
    assert_eq!(ready["tab"], 1);
    assert_eq!(ready["phase"], "live");
    let (rc, bad) = app.answered("ced.wait", json!({"idle":true,"rev":1,"timeout_ms":1000}));
    assert_eq!(rc, 10);
    assert_eq!(bad["error_code"], "INVALID_ARGUMENT");
    let mut waiting = app.command("ced.wait", json!({"rev":100,"timeout_ms":0}));
    assert!(waiting.try_recv().is_err());
    let (due, _) = app.timers.poll();
    assert_eq!(due.len(), 1);
    app.runtime
        .event("viu.timer".into(), due[0].clone())
        .unwrap();
    app.ready.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(app.runtime.try_completion().unwrap().error.is_none());
    let (rc, body) = waiting.try_recv().unwrap();
    assert_eq!(rc, 10);
    assert_eq!(body["error_code"], "CONFLICT");
    assert_eq!(body["reason"], "timeout");
}

#[test]
fn action_inventory_and_execute_share_native_editor_semantics() {
    let mut app = App::new();
    let (rc, rows) = app.answered("ced.actions", json!({}));
    assert_eq!(rc, 0);
    let rows = rows["actions"].as_array().unwrap();
    assert_eq!(rows.len(), 61);
    assert_eq!(rows[0]["id"], "file.new");
    assert_eq!(rows[0]["keys"], json!(["Ctrl+N"]));
    let (rc, unknown) = app.answered("ced.execute", json!({"id":"edit.unknown"}));
    assert_eq!(rc, 10);
    assert_eq!(unknown["error_code"], "UNKNOWN_ACTION");
    let (rc, bad) = app.answered(
        "ced.execute",
        json!({"id":"edit.unknown","params":{"misspelt":1}}),
    );
    assert_eq!(rc, 10);
    assert_eq!(bad["reason"], "unknown_param");
    let (rc, result) = app.answered("ced.execute", json!({"id":"edit.select_all"}));
    assert_eq!(rc, 0);
    assert_eq!(
        result,
        json!({"id":"edit.select_all","ok":true,"result":null})
    );
    let (rc, result) = app.answered("ced.execute", json!({"id":"edit.delete_line"}));
    assert_eq!(rc, 0);
    assert_eq!(result["ok"], true);
    assert_eq!(app.docs.call("read", "1", json!({})).unwrap()["text"], "");
}

#[test]
fn window_actions_wait_for_dialog_acceptance_or_cancellation() {
    let mut app = App::new();
    for id in ["file.open", "file.save_as", "search.goto_line"] {
        let mut pending = app.command("ced.execute", json!({"id":id}));
        assert!(
            pending.try_recv().is_err(),
            "{id} answered before the window"
        );
        let control = match id {
            "file.open" => "open-dialog",
            "file.save_as" => "save-dialog",
            _ => "goto-dialog",
        };
        assert_eq!(
            app.answered(
                "ced.intent",
                json!({"id":control,"action":"answer","value":{"choice":"cancel","field":""}})
            )
            .0,
            0
        );
        let (rc, body) = pending.try_recv().unwrap();
        assert_eq!(rc, 10);
        assert_eq!(body["error_code"], "CONFLICT");
        assert_eq!(body["reason"], "cancelled");
    }
    let mut pending = app.command("ced.execute", json!({"id":"search.goto_line"}));
    app.answered(
        "ced.intent",
        json!({"id":"goto-dialog","action":"answer","value":{"choice":"ok","field":"2:2"}}),
    );
    let (rc, body) = pending.try_recv().unwrap();
    assert_eq!(rc, 0);
    assert_eq!(body["result"], json!({"line":2,"col":2}));
    assert_eq!(
        app.docs.call("state", "1", json!({})).unwrap()["selection"]["head"]["line"],
        2
    );
}

#[test]
fn gold_dialog_surface_reads_opens_refuses_and_answers_the_window() {
    let mut app = App::new();
    assert_eq!(
        app.answered("ced.dialog", json!({})),
        (0, json!({"dialog":null}))
    );
    assert_eq!(
        app.answered("ced.dialog", json!({"answer":"done"})).1["error_code"],
        "NO_DIALOG"
    );
    assert_eq!(
        app.answered("ced.dialog", json!({"unknown":true})).1["error_code"],
        "ARGUMENT"
    );
    let (rc, body) = app.answered("ced.dialog", json!({"open":"keys"}));
    assert_eq!(rc, 0);
    assert_eq!(body["dialog"]["kind"], "keys");
    assert_eq!(
        body["dialog"]["buttons"],
        json!([{"id":"done","label":"Close","role":"default"}])
    );
    assert_eq!(
        app.answered("ced.dialog", json!({"open":"about"})).1["error_code"],
        "DIALOG_OPEN"
    );
    assert_eq!(
        app.answered("ced.dialog", json!({"answer":"done"})),
        (0, json!({"dialog":null}))
    );
    let mut pending = app.command("ced.execute", json!({"id":"search.goto_line"}));
    assert!(pending.try_recv().is_err());
    assert_eq!(
        app.answered("ced.dialog", json!({})).1["dialog"]["kind"],
        "goto"
    );
    assert_eq!(
        app.answered("ced.dialog", json!({"answer":"go","text":"2:2"})),
        (0, json!({"dialog":null}))
    );
    assert_eq!(
        pending.try_recv().unwrap().1["result"],
        json!({"line":2,"col":2})
    );
}

#[test]
fn open_location_suffix_focuses_the_existing_document_and_ignores_invalid_positions() {
    let mut app = App::new();
    for (path, offset) in [
        ("/tmp/contract.rs:2:2", 8),
        ("/tmp/contract.rs:9999", 8),
        ("/tmp/contract.rs:1:2", 2),
    ] {
        let (rc, reply) = app.answered("ced.open", json!({"paths":[path]}));
        assert_eq!(rc, 0);
        assert_eq!(
            reply["tabs"],
            json!([{"tab":1,"buffer":"b1_12345678","path":"/tmp/contract.rs"}])
        );
        assert_eq!(
            app.docs.call("state", "1", json!({})).unwrap()["selection"]["head"]["offset"],
            offset
        );
        assert_eq!(
            app.answered("ced.tabs", json!({})).1["tabs"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
}

#[test]
fn header_form_rejects_hidden_read_only_and_unoffered_choices() {
    let value = json!({"enabled":true,"fields":[{"id":"to","label":"To","value":"","visible":true,"actions":[]},{"id":"thread","label":"Thread","value":"fixed","visible":true,"read_only":true,"actions":[]},{"id":"cc","label":"Cc","value":"","visible":false,"actions":[]}],"attachments":[],"attachment_label":"Attachments","attach_label":"Attach…","remove_label":"Remove","chooser":{"open":false,"title":"Attach…","directory":"/"}});
    let widget = viu::widgets::Widget::new("headers", "header-form", "Headers", value.clone());
    widget.validate(&value).unwrap();
    let model = json!({"values":{"headers":value}});
    assert!(
        widget
            .admit(&model, "edit", &json!({"to":"reader@example.org"}))
            .is_ok()
    );
    for payload in [
        json!({"thread":"forged"}),
        json!({"cc":"hidden"}),
        json!({"unknown":"forged"}),
    ] {
        assert!(widget.admit(&model, "edit", &payload).is_err());
    }
    assert!(widget.admit(&model, "remove", &json!(0)).is_err());
    assert!(
        widget
            .admit(&model, "picked", &json!(["/tmp/file"]))
            .is_err()
    );
    assert!(widget.admit(&model, "command", &json!("unknown")).is_err());
}

#[test]
fn recovery_notice_choices_refuse_disabled_and_unoffered_commands() {
    let value = json!([{"icon":"info","label":"Retained edit","choices":[{"id":"reinsert:1","label":"Re-insert at Caret","enabled":false},{"id":"dismiss:1","label":"Dismiss","enabled":true}]}]);
    let widget =
        viu::widgets::Widget::new("notice-actions", "notice-bar", "Recovery", value.clone());
    widget.validate(&value).unwrap();
    let model = json!({"values":{"notice-actions":value}});
    assert!(
        widget
            .admit(&model, "command", &json!({"command":"dismiss:1"}))
            .is_ok()
    );
    for command in ["reinsert:1", "dismiss:2", "keep-mine"] {
        assert!(
            widget
                .admit(&model, "command", &json!({"command":command}))
                .is_err()
        );
    }
}

#[test]
fn conflict_choices_reinsert_retained_text_or_dismiss_without_changing_it() {
    for reinsert in [false, true] {
        let mut app = App::new();
        app.docs.call("close", "1", json!({})).unwrap();
        let mut server = fixture::seed(&app.docs, "1", "/tmp/conflict.txt", "hello world\n");
        app.docs
            .call("select", "1", json!({"anchor":2,"head":4}))
            .unwrap();
        app.docs
            .call("command", "1", json!({"name":"insert","text":"XY"}))
            .unwrap();
        let effects = app.docs.call("effects", "1", json!({})).unwrap();
        let request = &effects["out"][0];
        let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
        let remote = server.handle(
            "agent",
            "edit.replace",
            &json!({"buffer":body["buffer"],"range":[3,5],"text":"Q","origin":"agent:witness"}),
        );
        assert_eq!(remote.rc, 0);
        let refused = server.handle("ced", request["command"].as_str().unwrap(), &body);
        assert_eq!(refused.rc, 10);
        app.docs
            .call(
                "reply",
                "1",
                json!({"op_id":request["op_id"],"rc":refused.rc,"body":refused.body}),
            )
            .unwrap();
        for event in server.take_events() {
            app.docs
                .call("event", "1", serde_json::to_value(event).unwrap())
                .unwrap();
        }
        fixture::settle(&app.docs, "1", &mut server);
        let state = app.docs.call("inspect", "1", json!({})).unwrap();
        assert_eq!(state["conflicts"].as_array().unwrap().len(), 1);
        assert_eq!(
            app.docs.call("read", "1", json!({})).unwrap()["text"],
            "helQ world\n"
        );
        app.docs
            .call("select", "1", json!({"anchor":0,"head":0}))
            .unwrap();
        let choice = format!(
            "{}:{}",
            if reinsert { "reinsert" } else { "dismiss" },
            state["conflicts"][0]["rev"]
        );
        assert_eq!(
            app.answered(
                "ced.intent",
                json!({"id":"notice-actions","action":"command","value":{"command":choice}})
            )
            .0,
            0
        );
        // Mix owns draining; the local choice has already updated its mirror.
        assert_eq!(
            app.docs.call("inspect", "1", json!({})).unwrap()["conflicts"],
            json!([])
        );
        assert_eq!(
            app.docs.call("read", "1", json!({})).unwrap()["text"],
            if reinsert {
                "XYhelQ world\n"
            } else {
                "helQ world\n"
            }
        );
    }
}

#[test]
fn bus_mail_header_edits_arm_session_persistence_and_keep_header_undo() {
    let mut app = App::new();
    let headers = json!({"from":"","to":"recipient@example.org","cc":"","bcc":"","reply_to":"","subject":"Before","in_reply_to":null,"references":[],"attachments":[]});
    app.runtime.event("test.mail".into(),json!({"path":"/tmp/message.md","disk":"---\nmail-subject: Before\n---\nBody","headers":headers,"saved_headers":headers,"body_hash":"same","saved_body_hash":"same","hash_gen":1,"saving":false,"sending":false,"parked":false,"notice":null,"undo":[],"redo":[]})).unwrap();
    app.ready.recv_timeout(Duration::from_secs(5)).unwrap();
    let setup = app.runtime.try_completion().unwrap();
    assert!(setup.error.is_none(), "{:?}", setup.error);
    let mut changed = headers.clone();
    changed["subject"] = json!("After");
    assert_eq!(
        app.answered(
            "ced.execute",
            json!({"id":"mail.headers","params":{"args":{"headers":changed}}})
        )
        .0,
        0
    );
    let (_, state) = app.answered("ced.specimen", json!({}));
    assert_eq!(state["session_dirty"], true);
    assert_eq!(
        state["session_seen"]["tabs"][0]["mail"]["headers"]["subject"],
        "After"
    );
    assert_eq!(app.answered("ced.execute", json!({"id":"edit.undo"})).0, 0);
    assert_eq!(
        app.answered("ced.state", json!({})).1["mail"]["headers"]["subject"],
        "Before"
    );
    assert_eq!(app.answered("ced.execute", json!({"id":"edit.redo"})).0, 0);
    assert_eq!(
        app.answered("ced.state", json!({})).1["mail"]["headers"]["subject"],
        "After"
    );
    assert_eq!(
        app.docs.call("read", "1", json!({})).unwrap()["text"],
        "λ one\ntwo\n"
    );
}

#[test]
fn reusable_header_form_keeps_all_rows_in_its_allocated_height() {
    let rows = vec![json!({"id":"field","label":"Header","value":"recipient@example.org"}); 6];
    let widget = viu::widgets::Widget::new("headers", "form-fields", "Mail headers", json!(rows));
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    let mut widgets = viu::widgets::Widgets::default();
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 720.0),
            )),
            ..Default::default()
        },
        |ui| {
            widgets.begin_frame();
            widgets.widget(ui, &widget, &json!({}));
            widgets.finish_frame().unwrap();
        },
    )
    .drop_without_applying_deltas();
    let height = widgets.geometry["headers"].height();
    assert!(
        height > 100.0 && height < 300.0,
        "six headers must not be clipped to a one-row options bar: {height}"
    );
}
