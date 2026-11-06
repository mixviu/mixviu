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
$state.classified={\"1\":true}\npublish_ced($state)\n",
        );
        bundle.program.source.push_str(
            r#"
on test.mail
  $state.mail["1"]=$event.args
  $state.session_loaded=true;$state.session_path="/tmp/ced-test-session.json"
  $state.session_seen=session_value($state);$state.session_dirty=false
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
    let mut app=App::new();
    for id in ["file.open","file.save_as","search.goto_line"] {
        let mut pending=app.command("ced.execute",json!({"id":id}));
        assert!(pending.try_recv().is_err(),"{id} answered before the window");
        let control=match id {"file.open"=>"open-dialog","file.save_as"=>"save-dialog",_=>"goto-dialog"};
        assert_eq!(app.answered("ced.intent",json!({"id":control,"action":"answer","value":{"choice":"cancel","field":""}})).0,0);
        let (rc,body)=pending.try_recv().unwrap();
        assert_eq!(rc,10);assert_eq!(body["error_code"],"CONFLICT");assert_eq!(body["reason"],"cancelled");
    }
    let mut pending=app.command("ced.execute",json!({"id":"search.goto_line"}));
    app.answered("ced.intent",json!({"id":"goto-dialog","action":"answer","value":{"choice":"ok","field":"2:2"}}));
    let (rc,body)=pending.try_recv().unwrap();assert_eq!(rc,0);assert_eq!(body["result"],json!({"line":2,"col":2}));
    assert_eq!(app.docs.call("state","1",json!({})).unwrap()["selection"]["head"]["line"],2);
}

#[test]
fn header_form_rejects_hidden_read_only_and_unoffered_choices() {
    let value=json!({"enabled":true,"fields":[{"id":"to","label":"To","value":"","visible":true,"actions":[]},{"id":"thread","label":"Thread","value":"fixed","visible":true,"read_only":true,"actions":[]},{"id":"cc","label":"Cc","value":"","visible":false,"actions":[]}],"attachments":[],"attachment_label":"Attachments","attach_label":"Attach…","remove_label":"Remove","chooser":{"open":false,"title":"Attach…","directory":"/"}});
    let widget=viu::widgets::Widget::new("headers","header-form","Headers",value.clone());
    widget.validate(&value).unwrap();
    let model=json!({"values":{"headers":value}});
    assert!(widget.admit(&model,"edit",&json!({"to":"reader@example.org"})).is_ok());
    for payload in [json!({"thread":"forged"}),json!({"cc":"hidden"}),json!({"unknown":"forged"})] {assert!(widget.admit(&model,"edit",&payload).is_err());}
    assert!(widget.admit(&model,"remove",&json!(0)).is_err());
    assert!(widget.admit(&model,"picked",&json!(["/tmp/file"])).is_err());
    assert!(widget.admit(&model,"command",&json!("unknown")).is_err());
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
