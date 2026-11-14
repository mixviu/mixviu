// SPDX-License-Identifier: MIT OR Apache-2.0
use mix::{
    BusHandler, MixResult,
    evaluator::{BusFuture, IncomingEvent},
    value::Value as MixValue,
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};
use viu::{
    bundle::Bundle,
    runtime::{Completion, Runtime, from_json},
};

#[derive(Default)]
struct BrokerState {
    calls: Vec<(String, Value)>,
    refused: bool,
    serial: usize,
}
struct Broker(Arc<Mutex<BrokerState>>);
impl BusHandler for Broker {
    fn send<'a>(
        &'a self,
        _: &'a str,
        command: &'a str,
        body: &'a MixValue,
    ) -> BusFuture<'a, MixResult<(i32, MixValue)>> {
        Box::pin(async move {
            let mut s = self.0.lock().unwrap();
            s.calls.push((command.into(), viu::runtime::to_json(body)?));
            let (rc, value) = match command {
                "noded.observe.start" if s.refused => (10, json!({"error":"refused"})),
                "noded.observe.start" => {
                    s.serial += 1;
                    (
                        0,
                        json!({"subscription_id":format!("observe-{}", s.serial)}),
                    )
                }
                "noded.observe.stop" => (0, json!({"stopped":true})),
                _ => panic!("unexpected broker command {command}"),
            };
            Ok((rc, from_json(value)?))
        })
    }
    fn emit<'a>(&'a self, _: &'a str, _: &'a str, _: &'a MixValue) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async { Ok(()) })
    }
    fn port_exists<'a>(&'a self, _: &'a str) -> BusFuture<'a, MixResult<bool>> {
        Box::pin(async { Ok(true) })
    }
    fn next_incoming<'a>(&'a self) -> BusFuture<'a, Option<IncomingEvent>> {
        Box::pin(async { None })
    }
}
fn done(runtime: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let result = runtime.try_completion().unwrap();
    assert!(
        result.error.is_none(),
        "{:?} {} {}",
        result.error,
        result.stdout,
        result.stderr
    );
    result
}
fn event(runtime: &mut Runtime, rx: &mpsc::Receiver<()>, name: &str, args: Value) -> Completion {
    runtime.event(name.into(), args).unwrap();
    done(runtime, rx)
}
#[test]
fn pure_mix_policy_contracts() {
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/bustap/core-test.mix");
    runtime
        .load(viu::runtime::Program {
            source: std::fs::read_to_string(&path).unwrap(),
            path,
            scene: "probe".into(),
            citizen: "probe".into(),
        })
        .unwrap();
    done(&runtime, &rx);
}
#[test]
fn hosted_bustap_subscribes_switches_filters_recovers_and_exposes_rows() {
    let broker = Arc::new(Mutex::new(BrokerState::default()));
    let keep = broker.clone();
    let (tx, rx) = mpsc::channel();
    let backend: viu::runtime::BackendFactory = Box::new(move || Rc::new(Broker(broker)));
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        Some(backend),
    );
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/bustap"),
        &[],
    )
    .unwrap();
    runtime.load(bundle.program).unwrap();
    assert!(done(&runtime, &rx).model.is_some());
    event(&mut runtime, &rx, "bus.connected", json!({}));
    {
        let s = keep.lock().unwrap();
        assert_eq!(s.calls[0].0, "noded.observe.start");
        assert_eq!(s.calls[0].1["body"], "none");
    }
    event(
        &mut runtime,
        &rx,
        "bustap.filter",
        json!({"services":["example"]}),
    );
    {
        let s = keep.lock().unwrap();
        assert_eq!(s.calls[1].0, "noded.observe.start");
        assert_eq!(s.calls[2].0, "noded.observe.stop");
        assert_eq!(s.calls[2].1["subscription_id"], "observe-1");
    }
    keep.lock().unwrap().refused = true;
    event(
        &mut runtime,
        &rx,
        "bustap.filter",
        json!({"services":["other"]}),
    );
    let stats = event(&mut runtime, &rx, "bustap.stats", json!({}));
    assert_eq!(stats.reply.as_ref().unwrap().rc, 0);
    let stats = stats.reply.unwrap().body;
    assert_eq!(stats["filter"]["services"], json!(["example"]));
    assert_eq!(stats["subscription_id"], "observe-2");
    assert!(!stats["error"].is_null());
    event(&mut runtime, &rx, "bus.disconnected", json!({}));
    keep.lock().unwrap().refused = false;
    event(&mut runtime, &rx, "bus.connected", json!({}));
    // Notification path is important: replying to this event is a handler fault.
    runtime.notify("noded.observe.event".into(), json!({"seq":1,"ts":"2026-11-15T01:02:03.000Z","message_type":"request","from":null,"to":"example","command":"example.ping","size":42,"direction":"local","outcome":"rejected","rc":10,"dropped_count":3})).unwrap();
    assert!(done(&runtime, &rx).reply.is_none());
    let rows = event(&mut runtime, &rx, "bustap.rows", json!({"limit":1}));
    let rows = rows.reply.unwrap().body;
    assert_eq!(rows["rows"][0]["from"], "(anon)");
    assert_eq!(rows["rows"][0]["error"], true);
    let model = event(&mut runtime, &rx, "viu.tick", json!({}))
        .model
        .unwrap();
    assert_eq!(model["table"]["initial"]["rows"][0]["tone"], "danger");
    event(
        &mut runtime,
        &rx,
        "bustap.execute",
        json!({"id":"view.pause"}),
    );
    event(&mut runtime, &rx, "bustap.clear", json!({}));
    let stats = event(&mut runtime, &rx, "bustap.stats", json!({}));
    let stats = stats.reply.unwrap().body;
    assert_eq!(stats["paused"], true);
    assert_eq!(stats["retained"], 0);
    assert_eq!(stats["dropped"], 3);
    assert_eq!(stats["subscription_id"], "observe-3");
}
