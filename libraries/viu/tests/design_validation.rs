// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "test-support")]
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use viu::{app_host::test_support::Harness, bundle::Bundle};

static HOST_FIXTURE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn bad_unused() -> String {
    let good = design::EMBEDDED_DEFAULT_SOURCE;
    let needle = "when: { scheme: \"studio\", mode: \"light\" },";
    assert_eq!(good.matches(needle).count(), 1);
    let (before, after) = good.split_once(needle).unwrap();
    let pairs = "semantics: {\n          pairs: {";
    assert!(after.find(pairs).unwrap() < after.find("when:").unwrap());
    format!("{before}{needle}{}", after.replacen(pairs, &format!("{pairs} base: {{ surface: \"palette.background.1\", foreground: \"missing.late-control\" }},"), 1))
}

fn bad_requested() -> String {
    let good = design::EMBEDDED_DEFAULT_SOURCE;
    let needle =
        "when: { scheme: \"ocean\", mode: \"light\" },\n        semantics: {\n          pairs: {";
    assert_eq!(good.matches(needle).count(), 1);
    good.replace(needle, &format!("{needle} base: {{ surface: \"palette.background.1\", foreground: \"missing.startup-control\" }},"))
}

fn host(path: &Path, broker: &test_broker::Broker) -> Harness {
    let mut bundle = Bundle::load(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop"),
        &["--headless".into()],
    )
    .unwrap();
    bundle.manifest.bus_url = Some(broker.url.clone());
    bundle.launch = json!({"headless":true});
    bundle.manifest.saved_version = None;
    let host = Harness::with_theme_file(bundle, path);
    assert_eq!(
        host.state()["session"],
        json!({}),
        "ordering control must start before session values exist"
    );
    host
}

fn pump_until(host: &mut Harness, predicate: impl Fn(&Harness) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(60);
    while !predicate(host) {
        host.pump();
        if host.ready() {
            host.draw();
        }
        assert!(
            host.error().is_empty() || host.design_info()["notice"].as_str() == Some(host.error()),
            "{}",
            host.error()
        );
        assert!(
            Instant::now() < deadline,
            "design state did not settle: {}",
            host.design_info()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn call(
    host: &mut Harness,
    rt: &tokio::runtime::Runtime,
    url: &str,
    command: &str,
    args: Value,
) -> (u8, Value) {
    // This fixture asserts diagnostic contents, not cold-start latency (the
    // port-appearance regression owns that claim). Drain discovery first;
    // retain the real one-second Bus call deadline below.
    let until = Instant::now() + Duration::from_secs(30);
    while !host.callbacks_idle() {
        host.pump();
        assert!(Instant::now() < until, "fixture callbacks did not drain");
        std::thread::sleep(Duration::from_millis(2));
    }
    let client = rt
        .block_on(bus::SupervisedClient::connect_options("design-test", url).connect())
        .unwrap();
    eprintln!("DESIGN_TEST_CALL {command}");
    let command = command.to_owned();
    let task = rt.spawn(async move {
        let reply = client
            .call_with_headers_raw_within(
                "workshop",
                &command,
                &BTreeMap::new(),
                &args.to_string(),
                Duration::from_secs(1),
            )
            .await
            .unwrap();
        client.close().await;
        reply
    });
    // Bus admission does not require repeatedly painting the fixture while
    // its one-second call is pending. Paint explicitly at the notice oracle.
    while !task.is_finished() {
        host.pump();
        std::thread::sleep(Duration::from_millis(2));
    }
    let (rc, body, _) = rt.block_on(task).unwrap();
    (rc, serde_json::from_str(&body).unwrap())
}

fn info(host: &mut Harness, rt: &tokio::runtime::Runtime, url: &str) -> Value {
    let (rc, body) = call(host, rt, url, "workshop.info", json!({}));
    assert_eq!(rc, 0, "{body:?}");
    body
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn startup_fallback_surfaces_in_app_info_and_painted_notice() {
    let _fixture = HOST_FIXTURE.lock().unwrap_or_else(|e| e.into_inner());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("theme.mix");
    std::fs::write(&path, bad_requested()).unwrap();
    let broker = test_broker::Broker::start();
    let rt = runtime();
    let mut host = host(&path, &broker);
    pump_until(&mut host, |h| {
        h.ready() && h.state()["session"]["state"]["lost"] == true
    });
    assert_eq!(host.active_context()["scheme"], "studio");
    assert_eq!(host.active_context()["mode"], "dark");
    assert_eq!(host.design_info()["file_context"]["scheme"], "studio");
    assert_eq!(host.design_info()["file_context"]["mode"], "dark");
    let answer = info(&mut host, &rt, &broker.url);
    assert!(
        answer["design_validation"]["errors"][design::source_digest(&bad_requested())]
            .as_str()
            .unwrap()
            .contains("missing.startup-control")
    );
    host.draw();
    assert!(
        host.drawn_text().contains("missing.startup-control"),
        "fallback must be visible, not only logged"
    );
}

async fn generic_save(editd: &editd::Editd, path: &Path, text: &str) {
    async fn command(editd: &editd::Editd, command: &str, args: Value) -> Value {
        let request = bus::native_client::IncomingCommand {
            generation: 0,
            from: "design-test".into(),
            command: command.into(),
            id: Some("1".into()),
            args,
            body: String::new(),
            headers: BTreeMap::from([("broker_origin".into(), "local".into())]),
        };
        let (rc, body) = editd.handle(&request).await;
        assert_eq!(rc, 0, "{command}: {body}");
        serde_json::from_str(&body).unwrap()
    }
    let opened = command(editd, "edit.open", json!({"text":text})).await;
    command(
        editd,
        "edit.save",
        json!({"buffer":opened["buffer"],"path":path,"force":true}),
    )
    .await;
    command(editd, "edit.close", json!({"buffer":opened["buffer"]})).await;
    assert_eq!(std::fs::read_to_string(path).unwrap(), text);
}

#[test]
fn generic_save_gets_late_report_and_bad_context_switch_is_refused() {
    let _fixture = HOST_FIXTURE.lock().unwrap_or_else(|e| e.into_inner());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("theme.mix");
    std::fs::write(&path, design::EMBEDDED_DEFAULT_SOURCE).unwrap();
    let broker = test_broker::Broker::start();
    let rt = runtime();
    let mut host = host(&path, &broker);
    // No settingsd is registered in this fixture. Wait for its absence to
    // select the file theme, rather than the initial empty profile.
    pump_until(&mut host, |h| {
        h.ready()
            && h.design_info()["pending"] == 0
            && h.state()["session"]["state"]["lost"] == true
    });
    assert_eq!(
        host.design_info()["errors"],
        json!({}),
        "valid-source negative control"
    );
    let active_before = host.active_context();
    let (rc, error) = call(
        &mut host,
        &rt,
        &broker.url,
        "workshop.theme",
        json!({"scheme":"spoofed"}),
    );
    assert_ne!(rc, 0);
    assert_eq!(error["error_code"], "ARGUMENT");
    assert_eq!(host.design_info()["notice"], "");
    assert_eq!(host.design_info()["errors"], json!({}));
    let sink = Arc::new(editd::events::testing::RecordingSink::default());
    let editor = {
        let _entered = rt.enter();
        editd::Editd::start(
            editd::Config {
                epoch: "73ad912b".into(),
                budget_cap: editd::limits::MAX_TOTAL_BYTES,
            },
            sink,
        )
    };
    let bad = bad_unused();
    assert!(
        viu::design_admission::validate_source(&bad).is_err(),
        "write gate must reject this exact bypass source"
    );
    rt.block_on(generic_save(&editor, &path, &bad));
    host.draw();
    pump_until(&mut host, |h| {
        h.design_info()["errors"][design::source_digest(&bad)]
            .as_str()
            .unwrap_or("")
            .contains("missing.late-control")
    });
    assert_eq!(host.active_context(), active_before);
    assert_eq!(host.design_info()["file_context"]["scheme"], "ocean");
    assert_eq!(host.design_info()["file_context"]["mode"], "light");
    assert_eq!(
        host.design_info()["notice"],
        "",
        "background report cannot interrupt the app"
    );
    assert!(
        info(&mut host, &rt, &broker.url)["design_validation"]["errors"]
            .to_string()
            .contains("missing.late-control")
    );
    let (rc, error) = call(
        &mut host,
        &rt,
        &broker.url,
        "workshop.theme",
        json!({"scheme":"studio","mode":"light"}),
    );
    assert_ne!(rc, 0, "bad context switch must fail: {error}");
    assert_eq!(error["error_code"], "DESIGN_VALIDATION");
    let errors = host.design_info()["errors"].as_object().unwrap().clone();
    assert_eq!(
        errors.len(),
        1,
        "never attribute a refusal to the accepted source"
    );
    assert!(
        errors[&design::source_digest(&bad)]
            .as_str()
            .unwrap()
            .contains("missing.late-control")
    );
    assert_eq!(host.active_context(), active_before);
    assert_eq!(host.design_info()["file_context"]["scheme"], "ocean");
    assert_eq!(host.design_info()["file_context"]["mode"], "light");
    assert!(
        info(&mut host, &rt, &broker.url)["design_validation"]["notice"]
            .as_str()
            .unwrap()
            .contains("missing.late-control")
    );
    host.draw();
    assert!(host.drawn_text().contains("missing.late-control"));
    rt.block_on(generic_save(
        &editor,
        &path,
        design::EMBEDDED_DEFAULT_SOURCE,
    ));
    pump_until(&mut host, |h| h.design_info()["errors"] == json!({}));
    assert_eq!(
        host.active_context(),
        active_before,
        "repair report cannot restyle"
    );
    assert_eq!(
        info(&mut host, &rt, &broker.url)["design_validation"]["errors"],
        json!({})
    );
    assert_eq!(host.design_info()["notice"], "");
}
