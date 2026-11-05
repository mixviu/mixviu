// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit in-process safe backend fixture. Never reaches audio or a broker.
use std::{cell::RefCell, rc::Rc};
use mix::{BusHandler, MixResult, value::Value};
use mix::evaluator::{BusFuture, IncomingEvent};
use serde_json::{Value as Json, json};
use viu::runtime::{from_json, to_json};

pub struct AudioFixture { snapshot: RefCell<Json> }
impl AudioFixture {
    pub fn new() -> Rc<Self> { Rc::new(Self { snapshot: RefCell::new(json!({
        "ok":true,"default_sink_found":true,"default_source_found":true,"raw_sources":1,
        "sinks":[{"index":1,"name":"example.output","description":"Example output","level":50,"muted":false,"default":true}],
        "sources":[{"index":2,"name":"example.input","description":"Example input","level":75,"muted":false,"default":true}],
        "streams":[{"index":3,"name":"Example player","media":"Fixture","level":40,"muted":false,"sink":1}]
    })) }) }
}
impl BusHandler for AudioFixture {
    fn send<'a>(&'a self, target: &'a str, command: &'a str, args: &'a Value) -> BusFuture<'a, MixResult<(i32,Value)>> {
        Box::pin(async move {
            if target != "audio-fixture" { return Ok((10, Value::String("unknown fixture target".into()))); }
            match command {
                "snapshot" => Ok((0, Value::String(self.snapshot.borrow().to_string()))),
                "write" => {
                    let args = to_json(args)?;
                    let body: Json = serde_json::from_str(args["body"].as_str().unwrap_or("{}")).unwrap_or(Json::Null);
                    let argv = body["argv"].as_array();
                    let mut snap = self.snapshot.borrow_mut();
                    let mut accepted = false;
                    if let Some(argv) = argv.filter(|a| a.len() == 3) {
                        let command = argv[0].as_str().unwrap_or("");
                        let collection = if command.contains("sink-input") {"streams"} else if command.contains("source") {"sources"} else {"sinks"};
                        let key = argv[1].as_str().unwrap_or("");
                        for device in snap[collection].as_array_mut().unwrap() {
                            if device["name"].as_str() == Some(key) || device["index"].to_string() == key {
                                if command.ends_with("-volume") {
                                    if let Some(level) = argv[2].as_str().and_then(|s| s.strip_suffix('%')).and_then(|s| s.parse::<f64>().ok()).filter(|n| n.is_finite() && *n >= 0.0 && *n <= 150.0) {
                                        device["level"] = json!(level); accepted = true;
                                    }
                                } else if command.ends_with("-mute") && argv[2] == "toggle" {
                                    device["muted"] = json!(!device["muted"].as_bool().unwrap_or(false)); accepted = true;
                                }
                            }
                        }
                    }
                    Ok((0, Value::String(json!({"ok":accepted,"message":if accepted {""} else {"fixture refused"}}).to_string())))
                }
                _ => Ok((10, from_json(json!({"error":"unknown fixture command"}))?)),
            }
        })
    }
    fn emit<'a>(&'a self, _: &'a str, _: &'a str, _: &'a Value) -> BusFuture<'a, MixResult<()>> { Box::pin(async { Ok(()) }) }
    fn port_exists<'a>(&'a self, target: &'a str) -> BusFuture<'a, MixResult<bool>> { Box::pin(async move { Ok(target == "audio-fixture") }) }
    fn next_incoming<'a>(&'a self) -> BusFuture<'a, Option<IncomingEvent>> { Box::pin(async {None}) }
}
