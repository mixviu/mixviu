// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

#[test]
fn workshop_answers_during_discovery() {
    let broker = test_broker::Broker::start();
    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let mut peers = Vec::new();
    for i in 0..200 {
        peers.push(rt.block_on(bus::SupervisedClient::connect_options(&format!("peer-{i:03}"), &broker.url).connect()).unwrap());
    }
    let caller = Arc::new(rt.block_on(bus::SupervisedClient::connect_options("test-caller", &broker.url).connect()).unwrap());
    let mut bundle = Bundle::load(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop"), &["--headless".into()]).unwrap();
    bundle.manifest.bus_url = Some(broker.url.clone());
    bundle.manifest.saved_version = None;
    let mut host = create(bundle, Arc::new(|| {}));
    let ctx = egui::Context::default();
    crate::appearance::install(&ctx);
    let start = Instant::now();
    let mut gone = false;
    while start.elapsed() < Duration::from_secs(18) {
        let c = caller.clone();
        let ping = rt.spawn(async move { c.call_with_headers_raw_within("workshop", "workshop.ping", &BTreeMap::new(), "{}", Duration::from_secs(1)).await });
        let end = Instant::now() + Duration::from_secs(2);
        while !ping.is_finished() {
            ctx.run_ui(egui::RawInput::default(), |ui| host.pump(ui.ctx())).drop_without_applying_deltas();
            assert!(host.error.is_empty(), "{}", host.error);
            assert!(Instant::now() < end, "owner stalled");
            std::thread::sleep(Duration::from_millis(2));
        }
        let result = rt.block_on(ping).unwrap();
        if start.elapsed() > Duration::from_secs(2) {
            assert_eq!(result.unwrap().0, 0, "ping must remain available");
        }
        if !gone && start.elapsed() > Duration::from_secs(1) {
            rt.block_on(peers[0].close());
            gone = true;
        }
    }
    for peer in peers { rt.block_on(peer.close()); }
    rt.block_on(caller.close());
}
