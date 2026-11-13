// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real owner, evaluator, Jobs and private noded. No display or agent service.
use super::*;

fn call(
    host: &mut Host,
    ctx: &egui::Context,
    rt: &tokio::runtime::Runtime,
    caller: &Arc<bus::SupervisedClient>,
    command: &str,
) -> (u8, Value) {
    let caller = caller.clone();
    let command = command.to_owned();
    let label = command.clone();
    let task = rt.spawn(async move {
        tokio::time::timeout(
            Duration::from_secs(1),
            caller.call_with_headers_raw_within(
                "workshop",
                &command,
                &BTreeMap::new(),
                "{}",
                Duration::from_secs(1),
            ),
        )
        .await
    });
    let end = Instant::now() + Duration::from_secs(2);
    while !task.is_finished() {
        ctx.run_ui(egui::RawInput::default(), |ui| host.pump(ui.ctx()))
            .drop_without_applying_deltas();
        assert!(host.error.is_empty(), "{}", host.error);
        assert!(Instant::now() < end, "owner stalled: {:?}", host.inflight);
        std::thread::sleep(Duration::from_millis(2));
    }
    let (rc, body, _) = rt
        .block_on(task)
        .unwrap()
        .unwrap_or_else(|e| panic!("{label}: {e}"))
        .unwrap_or_else(|e| panic!("{label}: {e}"));
    (rc, serde_json::from_str(&body).unwrap())
}

#[test]
fn workshop_answers_during_discovery() {
    for count in [4, 200] {
        let broker = test_broker::Broker::start();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let disconnected = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut peers = Vec::new();
        let mut tasks = Vec::new();
        for i in 0..count {
            let peer = Arc::new(
                rt.block_on(
                    bus::SupervisedClient::connect_options(&format!("peer-{i:03}"), &broker.url)
                        .connect(),
                )
                .unwrap(),
            );
            let mut incoming = peer.incoming().unwrap();
            {
                let p = peer.clone();
                let gone = disconnected.clone();
                tasks.push(rt.spawn(async move {
                    while let Some(command) = incoming.recv().await {
                        if i == 0 {
                            continue;
                        }
                        if i == 1 {
                            gone.store(true, std::sync::atomic::Ordering::Release);
                            p.close().await;
                            break;
                        }
                        p.respond(&command, 0, "[]").await.unwrap();
                    }
                }));
            }
            peers.push(peer);
        }
        let caller = Arc::new(
            rt.block_on(
                bus::SupervisedClient::connect_options("test-caller", &broker.url).connect(),
            )
            .unwrap(),
        );
        let mut bundle = Bundle::load(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop"),
            &["--headless".into()],
        )
        .unwrap();
        bundle.manifest.bus_url = Some(broker.url.clone());
        bundle.launch = json!({"headless":true});
        bundle.manifest.saved_version = None;
        let mut host = create(bundle, Arc::new(|| {}));
        let ctx = egui::Context::default();
        crate::appearance::install(&ctx);
        let start = Instant::now();
        while !host.ready || host.bus.client().is_none() {
            ctx.run_ui(egui::RawInput::default(), |ui| host.pump(ui.ctx()))
                .drop_without_applying_deltas();
            assert!(start.elapsed() < Duration::from_secs(5), "startup");
            std::thread::sleep(Duration::from_millis(2));
        }
        // Keep a windowed owner's model deliberately undrawn. Its ordinary
        // Bus surface must work even while UI automation waits for geometry.
        if count == 4 {
            host.headless = false;
        }
        let blocked_ui = if count == 4 {
            let c = caller.clone();
            Some(rt.spawn(async move {
                c.call_with_headers_raw_within(
                    "workshop",
                    "workshop.ui.tree",
                    &BTreeMap::new(),
                    "{}",
                    Duration::from_secs(30),
                )
                .await
            }))
        } else {
            None
        };
        let mut overlapping = false;
        let mut streamed = false;
        loop {
            assert_eq!(call(&mut host, &ctx, &rt, &caller, "workshop.ping").0, 0);
            let (rc, info) = call(&mut host, &ctx, &rt, &caller, "workshop.info");
            assert_eq!(rc, 0);
            let services = &info["snapshot"]["services"];
            if services["peer-002"]["Ok"].is_array() && info["discovering"] == true {
                streamed = true;
            }
            if info["discovering"] == true && !overlapping {
                let (rc, body) = call(&mut host, &ctx, &rt, &caller, "workshop.refresh");
                assert_eq!(rc, 10);
                assert_eq!(body["error_code"], "BUSY");
                overlapping = true;
            }
            if overlapping && info["discovering"] == false {
                assert!(
                    services["peer-000"]["Err"].is_string(),
                    "silent peer: {services}"
                );
                assert!(
                    services["peer-001"]["Err"].is_string(),
                    "departed peer: {services}"
                );
                assert!(services.as_object().unwrap().len() >= count);
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(20),
                "refresh never finished"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(
            streamed,
            "healthy results must appear before the silent peer finishes"
        );
        assert!(disconnected.load(std::sync::atomic::Ordering::Acquire));
        if let Some(task) = blocked_ui {
            assert!(!task.is_finished(), "UI must still wait for an actual draw");
            task.abort();
        }
        for task in tasks {
            task.abort();
        }
        for peer in peers {
            rt.block_on(peer.close());
        }
        rt.block_on(caller.close());
    }
}
