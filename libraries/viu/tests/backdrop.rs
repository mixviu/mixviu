// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real Mix controller, native Jobs, retained pixels, timers and call receipts.
use mix::{
    BusHandler, MixResult,
    evaluator::{BusFuture, IncomingEvent},
    value::Value as MixValue,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use viu::{
    bundle::Bundle,
    runtime::{Runtime, Services},
};
struct Fixture;
impl BusHandler for Fixture {
    fn send<'a>(
        &'a self,
        _: &'a str,
        command: &'a str,
        _: &'a MixValue,
    ) -> BusFuture<'a, MixResult<(i32, MixValue)>> {
        Box::pin(async move {
            assert_eq!(command,"topic.publish","unexpected inter-process effect");
            Ok((0,viu::runtime::from_json(json!({}))?))
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
struct App {
    runtime: Runtime,
    jobs: viu::jobs::Jobs,
    rasters: viu::raster::Rasters,
    calls: viu::calls::Calls,
    timers: viu::timers::Timers,
    wakes: mpsc::Receiver<()>,
    model: Value,
    tokens: BTreeMap<u64, String>,
    directory: tempfile::TempDir,
}
impl App {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join("backdrop.mx");
        std::fs::write(&config, "schema: 1\ndefault: { colour: \"#1234ab\" }\n").unwrap();
        let bundle = Bundle::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/backdrop"),
            &["--config".into(), config.to_str().unwrap().into()],
        )
        .unwrap();
        let (tx, wakes) = mpsc::channel();
        let wake: viu::runtime::Wake = Arc::new(move || {
            let _ = tx.send(());
        });
        let rasters = viu::raster::Rasters::default();
        let calls = viu::calls::Calls::default();
        let timers = viu::timers::Timers::default();
        let services = Services {
            rasters: Some(rasters.clone()),
            calls: Some(calls.clone()),
            timers: Some(timers.clone()),
            ui_state: Some(Default::default()),
            ..Default::default()
        };
        let mut jobs = viu::jobs::Jobs::new(4, wake.clone());
        jobs.set_services(services.clone());
        let runtime = Runtime::with_services(
            32,
            wake,
            Some(jobs.backend(Some(Box::new(|| Rc::new(Fixture))))),
            services,
        );
        let mut app = Self {
            runtime,
            jobs,
            rasters,
            calls,
            timers,
            wakes,
            model: Value::Null,
            tokens: BTreeMap::new(),
            directory,
        };
        app.runtime.load(bundle.program).unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        while app.model.is_null() {
            app.pump();
            assert!(Instant::now() < end);
        }
        app
    }
    fn pump(&mut self) {
        for done in self.jobs.poll() {
            self.runtime
                .notify(
                    "viu.job.completed".into(),
                    json!({"tag":done.tag,"rc":done.rc,"body":done.body}),
                )
                .unwrap();
        }
        for timer in self.timers.poll().0 {
            self.runtime.notify("viu.timer".into(), timer).unwrap();
        }
        for done in self.runtime.drain() {
            assert!(
                done.error.is_none(),
                "{:?}\n{}\n{}",
                done.error,
                done.stdout,
                done.stderr
            );
            if let Some(model) = done.model {
                self.model = model;
            }
            if let Some(token) = self.tokens.remove(&done.sequence) {
                let (rc, body) = done
                    .reply
                    .map_or((0, json!({})), |reply| (reply.rc, reply.body));
                self.calls.complete(&token, rc, body);
            }
        }
        let _ = self.wakes.recv_timeout(Duration::from_millis(2));
    }
    fn request(
        &mut self,
        command: &str,
        args: Value,
    ) -> tokio::sync::oneshot::Receiver<(u8, Value)> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let token = self
            .calls
            .begin(tx, Arc::new(std::sync::atomic::AtomicBool::new(false)))
            .unwrap();
        let seq = self
            .runtime
            .event_with_headers(
                command.into(),
                args,
                BTreeMap::from([("viu-call".into(), token.clone())]),
            )
            .unwrap();
        self.tokens.insert(seq, token);
        rx
    }
    fn wait(&mut self, mut rx: tokio::sync::oneshot::Receiver<(u8, Value)>) -> (u8, Value) {
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            self.pump();
            if let Ok(reply) = rx.try_recv() {
                // Let the completed callback's model reach the native owner.
                for _ in 0..3 {
                    self.pump();
                }
                return reply;
            }
            assert!(Instant::now() < end, "deferred native reply timed out");
        }
    }
    fn call(&mut self, command: &str, args: Value) -> (u8, Value) {
        // Query/immediate refusal completions go through the same Calls owner
        // that Host uses. Its completion lane is driven here explicitly.
        let sequence = self.runtime.event(command.into(), args).unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            for done in self.jobs.poll() {
                self.runtime
                    .notify(
                        "viu.job.completed".into(),
                        json!({"tag":done.tag,"rc":done.rc,"body":done.body}),
                    )
                    .unwrap();
            }
            for done in self.runtime.drain() {
                assert!(done.error.is_none(), "{:?}", done.error);
                if let Some(model) = done.model {
                    self.model = model;
                }
                if done.sequence == sequence {
                    let reply = done.reply.unwrap();
                    return (reply.rc, reply.body);
                }
            }
            assert!(Instant::now() < end);
            let _ = self.wakes.recv_timeout(Duration::from_millis(2));
        }
    }
    fn png(&self, name: &str, colour: [u8; 4]) -> String {
        let path = self.directory.path().join(name);
        image::RgbaImage::from_pixel(7, 5, image::Rgba(colour))
            .save(&path)
            .unwrap();
        path.to_str().unwrap().into()
    }
}
#[test]
fn backdrop_colours_modes_outputs_and_refusals_use_mix() {
    let mut app = App::new();
    let image = app.png("fixture.png", [23, 45, 67, 255]);
    for mode in ["fill", "fit", "centre", "tile", "stretch"] {
        let rx=app.request("backdrop.set",json!({"schema":1,"default":{"colour":"#1234ab"},"outputs":{"HEADLESS-2":{"image":image,"fit":mode}}}));
        let (rc, body) = app.wait(rx);
        assert_eq!(rc, 0, "{body}");
        assert_eq!(app.model["values"]["root"]["colour"], json!([18, 52, 171]));
        assert_eq!(
            app.model["surfaces"]["HEADLESS-2"]["values"]["root"]["fit"],
            mode
        );
        assert!(app.model["surfaces"]["HEADLESS-2"]["values"]["root"]["handle"].is_string());
    }
    let before = app.call("backdrop.state", json!({})).1;
    let (rc, body) = app.call(
        "backdrop.set",
        json!({"schema":1,"default":{"slideshow":{}}}),
    );
    assert_eq!(rc, 10);
    assert_eq!(body["error_code"], "ARGUMENT");
    assert!(!body["message"].as_str().unwrap().is_empty());
    let after = app.call("backdrop.state", json!({})).1;
    assert_eq!(after["active"], before["active"]);
    assert_eq!(after["config"], before["config"]);
}
#[test]
fn decode_refusal_and_replaced_source_preserve_transactional_pixels_and_storage() {
    let mut app = App::new();
    let image = app.png("fixture.png", [12, 34, 56, 255]);
    let doc = json!({"schema":1,"default":{"image":image}});
    let rx = app.request("backdrop.set", doc.clone());
    assert_eq!(app.wait(rx).0, 0);
    let handle = app.model["values"]["root"]["handle"]
        .as_str()
        .unwrap()
        .to_owned();
    let pixels = app.rasters.get(&handle).unwrap();
    let before = app.call("backdrop.state", json!({})).1;
    let stored = std::fs::read(app.directory.path().join("backdrop.mx")).unwrap();
    let rx = app.request(
        "backdrop.set",
        json!({"schema":1,"default":{"image":app.directory.path().join("missing.png")}}),
    );
    let (rc, body) = app.wait(rx);
    assert_eq!(rc, 10);
    assert_eq!(body["error_code"], "IMAGE");
    let after = app.call("backdrop.state", json!({})).1;
    assert_eq!(after["active"], before["active"]);
    assert_eq!(after["config"], before["config"]);
    assert!(!after["error"].is_null());
    assert_eq!(
        std::fs::read(app.directory.path().join("backdrop.mx")).unwrap(),
        stored
    );
    app.png("fixture.png", [90, 80, 70, 255]);
    let rx = app.request("backdrop.set", doc);
    assert_eq!(app.wait(rx).0, 0);
    assert_ne!(app.model["values"]["root"]["handle"], handle);
    assert_eq!(pixels.decoded.rgba.get_pixel(0, 0).0, [12, 34, 56, 255]);
    assert!(app.call("backdrop.state", json!({})).1["error"].is_null());
}
#[test]
fn slideshow_next_skips_failed_entries_and_keeps_source_document() {
    let mut app = App::new();
    let a = app.png("a.png", [10, 20, 30, 255]);
    let c = app.png("c.png", [30, 20, 10, 255]);
    let bad = app.directory.path().join("bad.png");
    std::fs::write(&bad, b"bad").unwrap();
    let doc = json!({"schema":1,"default":{"slideshow":{"images":[a,bad,c],"interval_s":5}}});
    let rx = app.request("backdrop.set", doc);
    assert_eq!(app.wait(rx).0, 0);
    let saved = std::fs::read(app.directory.path().join("backdrop.mx")).unwrap();
    let rx = app.request("backdrop.next", json!({}));
    let (rc, body) = app.wait(rx);
    assert_eq!(rc, 10);
    assert_eq!(body["error_code"], "IMAGE");
    let rx = app.request("backdrop.next", json!({}));
    let (rc, body) = app.wait(rx);
    assert_eq!(rc, 0, "{body}");
    assert_eq!(body["slideshows"][0]["current_path"], c);
    assert_eq!(body["slideshows"][0]["index"], 2);
    assert_eq!(body["slideshows"][0]["fading"], true);
    assert_eq!(
        std::fs::read(app.directory.path().join("backdrop.mx")).unwrap(),
        saved
    );
    assert_eq!(body["config"]["default"]["slideshow"]["images"][0], a);
}
