// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{
    bundle::Bundle,
    runtime::{Completion, Runtime},
};

fn complete(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    done
}

#[test]
fn modern_mail_row_projection_measures_2000_distinct_dates() {
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/email"),
        &[],
    )
    .unwrap();
    let mut program = bundle.program;
    program.source.push_str(r#"
on test.rows
  $began=monotonic();$projected=gold_rows($event.args.emails)
  reply(0,json_encode({rows:len($projected),first:$projected[0],milliseconds:(monotonic()-$began)*1000}))
end
"#);
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(program).unwrap();
    complete(&runtime, &rx);
    let emails=(0..2000).map(|n|json!({"id":format!("email-{n}"),"subject":format!("A realistic message {n}"),"from":[{"name":"Example Sender","email":"sender@example.test"}],"receivedAt":chrono::DateTime::from_timestamp(1794000000+n*120,0).unwrap().to_rfc3339(),"keywords":{},"hasAttachment":n%3==0})).collect::<Vec<_>>();
    runtime
        .event("test.rows".into(), json!({"emails":emails}))
        .unwrap();
    let reply = complete(&runtime, &rx).reply.unwrap();
    assert_eq!(reply.rc, 0, "{}", reply.body);
    assert_eq!(reply.body["rows"], 2000);
    assert_eq!(reply.body["first"]["strong"], true);
    assert_eq!(reply.body["first"]["cells"][1], "Example Sender");
    eprintln!(
        "Email modern 2000-row projection: {} ms",
        reply.body["milliseconds"]
    );
}

#[test]
fn connected_email_model_mounts_all_panes_and_draws_native_rows() {
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/email"),
        &[],
    )
    .unwrap();
    let mut program = bundle.program;
    program.source.push_str(r#"
on test.ready
  $entries=[{id:"a",name:"Fixture",label:"Fixture",user:"user@example.test",email:"user@example.test",enabled:true}];$active="a"
  $wire_views.a={state:"ready",unread:1};$connected=true
  $boxes=[{id:"inbox",name:"Inbox",role:"inbox",parentId:nil,totalEmails:1,unreadEmails:1}]
  $mail_emails=[{id:"e1",subject:"Hello",from:[{name:"Example",email:"sender@example.test"}],receivedAt:"2026-11-07T08:00:00Z",keywords:{}}]
  $wire_data.a={boxes:$boxes,emails:$mail_emails};$box="inbox";$rows=gold_rows($mail_emails);$total=1
  publish_email();reply(0,"{}")
end
on test.pim
  $wire_data.a.calendars=[{id:"calendar-1",name:"Personal"}]
  $wire_data.a.calendar="calendar-1"
  $wire_data.a.events=[{id:"event-1",title:"A real event",start:"2026-11-07T08:00:00Z",end:"2026-11-07T09:00:00Z",calendarId:"calendar-1",data:{description:"Details"}}]
  $wire_data.a.open_event="event-1"
  surface_mode("calendar");publish_email();reply(0,"{}")
end
"#);
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(program).unwrap();
    complete(&runtime, &rx);
    runtime.event("test.ready".into(), json!({})).unwrap();
    let model = complete(&runtime, &rx).model.unwrap();
    assert_eq!(model["native_table"]["initial"]["rows"][0]["strong"], true);
    let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    for _ in 0..3 {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1040.0, 680.0),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        )
        .drop_without_applying_deltas();
    }
    assert!(view.geometry["places-pane"].width() > 150.0);
    assert!(view.geometry["message-list"].width() > 400.0);
    assert!(view.geometry["reader-pane"].width() > 300.0);
    assert!(view.geometry.contains_key("emails"));
    assert!(
        view.geometry
            .contains_key("account-tree.key.a/mailbox/inbox")
    );
    runtime.event("email.info".into(), json!({})).unwrap();
    let reply = complete(&runtime, &rx).reply.unwrap();
    assert_eq!(reply.rc, 0, "{}", reply.body);
    assert_eq!(reply.body["schema"], "email.v1");
    assert_eq!(
        reply.body["accounts"][0]["mail"]["emails"][0]["from"],
        "Example"
    );
    assert_eq!(
        reply.body["accounts"][0]["mail"]["emails"][0]["read"],
        false
    );
    runtime.event("test.pim".into(), json!({})).unwrap();
    let model = complete(&runtime, &rx).model.unwrap();
    assert_eq!(
        model["pim_table"]["initial"]["rows"][0]["cells"][0],
        "A real event"
    );
    assert_eq!(model["values"]["pim-body"], "Details");
    let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1040.0, 680.0),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
    assert!(view.geometry.contains_key("pim-list.key.event-1"));
    runtime
        .event(
            "email.compose".into(),
            json!({"to":"user@example.test","subject":"Composer witness","body":"Native body"}),
        )
        .unwrap();
    let done = complete(&runtime, &rx);
    assert_eq!(done.reply.unwrap().rc, 0);
    let model = done.model.unwrap();
    let child = &model["native_windows"][0];
    assert_eq!(child["key"], "compose-1");
    let mut view =
        viu::gold_frame::mount(child["scene"].as_str().unwrap(), child["model"].clone()).unwrap();
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 700.0),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
    assert!(view.geometry.contains_key("compose-body"));
    assert!(view.geometry.contains_key("compose-headers"));
}

#[test]
fn email_empty_frame_draws_and_primary_button_keeps_native_intent() {
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/email"),
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
    let model = complete(&runtime, &rx).model.unwrap();
    assert_eq!(model["no_accounts"], true);
    let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    let mut input = viu::input::Input::default();
    let mut intents = Vec::new();
    for step in 0..5 {
        if step == 2 {
            input
                .queue("click", &json!({"id":"open-settings"}), &mut view)
                .unwrap();
        }
        let events = if !input.now.is_empty() {
            std::mem::take(&mut input.now)
        } else {
            input.later.pop_front().unwrap_or_default()
        };
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1040.0, 680.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                intents.extend(view.show(ui));
            },
        )
        .drop_without_applying_deltas();
    }
    assert!(view.geometry["no-accounts"].top() >= view.geometry["gold-header"].bottom());
    assert!(view.geometry["open-settings"].bottom() < view.geometry["gold-status"].top());
    assert!(
        intents
            .iter()
            .any(|e| e.command == "email.intent" && e.body["id"] == "open-settings")
    );
    runtime
        .event("email.execute".into(), json!({"id":"mail.compose"}))
        .unwrap();
    let reply = complete(&runtime, &rx).reply.unwrap();
    assert_eq!(reply.rc, 10);
    assert_eq!(reply.body["error_code"], "DISABLED");
    runtime
        .event("email.dialog".into(), json!({"open":"about"}))
        .unwrap();
    let model = complete(&runtime, &rx).model.unwrap();
    assert_eq!(
        model["native_overlays"][0]["definition"]["initial"]["title"],
        "About Email"
    );
    runtime
        .event("email.dialog".into(), json!({"open":"shortcuts"}))
        .unwrap();
    let model = complete(&runtime, &rx).model.unwrap();
    let widget = viu::widgets::Widget::decode(&model["native_overlays"][0]["definition"]).unwrap();
    assert!(widget.initial["grid"].as_array().unwrap().len() > 12);
    let mut overlays = viu::widgets::Widgets::default();
    for _ in 0..3 {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1040.0, 680.0),
                )),
                ..Default::default()
            },
            |ui| {
                overlays.widget(ui, &widget, &model);
            },
        )
        .drop_without_applying_deltas();
    }
    assert!(
        overlays.geometry["email-help"].height() > 400.0,
        "shortcut rows were not drawn"
    );
}
