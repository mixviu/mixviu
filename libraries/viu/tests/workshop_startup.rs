// SPDX-License-Identifier: MIT OR Apache-2.0
//! Fresh-process startup measurement: calls begin at registration, before ready.
#![cfg(feature = "test-support")]
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc, time::{Duration, Instant}};
use viu::{app_host::test_support::Harness, bundle::Bundle};

// Keep 250 ms below the unchanged one-second call deadline. A candidate must
// leave this margin for desk scheduling, rather than merely avoiding a timeout.
const STARTUP_CALL_BOUND_US: u64 = 750_000;

#[test]
fn workshop_calls_from_port_appearance_at_200_peers() {
    let whole = Instant::now();
    let broker = test_broker::Broker::start();
    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let mut peers = Vec::new();
    let mut tasks = Vec::new();
    for i in 0..200 {
        let peer = Arc::new(rt.block_on(bus::SupervisedClient::connect_options(&format!("peer-{i:03}"), &broker.url).connect()).unwrap());
        let mut incoming = peer.incoming().unwrap();
        let p = peer.clone();
        tasks.push(rt.spawn(async move {
            while let Some(command) = incoming.recv().await {
                if i == 0 { continue; }
                if i == 1 { p.close().await; break; }
                p.respond(&command, 0, "[]").await.unwrap();
            }
        }));
        peers.push(peer);
    }
    let caller = Arc::new(rt.block_on(bus::SupervisedClient::connect_options("startup-caller", &broker.url).connect()).unwrap());
    let sampler = caller.clone();
    // This independent client must run while construction/pumping blocks the
    // owner thread. Waiting for Harness::ready would conceal exactly that hold.
    let calls = rt.spawn(async move {
        let wait = Instant::now();
        loop {
            if sampler.list_services().await.unwrap().iter().any(|name| name == "workshop") { break; }
            assert!(wait.elapsed() < Duration::from_secs(5), "Workshop port did not appear");
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        let port = Instant::now();
        let mut samples = Vec::new();
        while port.elapsed() < Duration::from_millis(1500) {
            for command in ["workshop.ping", "workshop.info"] {
                if port.elapsed() >= Duration::from_millis(1500) { break; }
                let started = Instant::now();
                let answer = sampler.call_with_headers_raw_within("workshop", command, &BTreeMap::new(), "{}", Duration::from_secs(1)).await;
                samples.push(json!({"command":command,"start_us":started.duration_since(port).as_micros() as u64,"latency_us":started.elapsed().as_micros() as u64,"ok":answer.as_ref().is_ok_and(|(rc,_,_)| *rc == 0),"error":answer.err().map(|e|e.to_string())}));
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        samples
    });
    let mut bundle = Bundle::load(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop"), &["--headless".into()]).unwrap();
    bundle.manifest.bus_url = Some(broker.url.clone());
    bundle.launch = json!({"headless":true});
    bundle.manifest.saved_version = None;
    let mut host = Harness::new(bundle);
    host.probe_discovery();
    while !calls.is_finished() {
        host.pump();
        assert!(host.error().is_empty(), "{}", host.error());
        assert!(whole.elapsed() < Duration::from_secs(5), "startup fixture exceeded five seconds");
        std::thread::sleep(Duration::from_millis(2));
    }
    let samples = rt.block_on(calls).unwrap();
    let worst = samples.iter().map(|s|s["latency_us"].as_u64().unwrap()).max().unwrap();
    let failed = samples.iter().filter(|s| s["ok"] != true).count();
    eprintln!("WORKSHOP_STARTUP_PORT {}", json!({"peers":200,"window_us":1500000,"bound_us":STARTUP_CALL_BOUND_US,"worst_call_us":worst,"failed":failed,"whole_us":whole.elapsed().as_micros() as u64,"samples":samples,"host":host.discovery_measurements()}));
    assert!(failed == 0 && worst <= STARTUP_CALL_BOUND_US, "startup call regression: worst_us={worst}, bound_us={STARTUP_CALL_BOUND_US}, failed={failed}");
    for task in tasks { task.abort(); }
    rt.block_on(caller.close());
}
