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
struct OwnedChild(std::process::Child);
impl std::ops::Deref for OwnedChild { type Target=std::process::Child;fn deref(&self)->&Self::Target {&self.0} }
impl std::ops::DerefMut for OwnedChild { fn deref_mut(&mut self)->&mut Self::Target {&mut self.0} }
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {let _=self.0.kill();let _=self.0.wait();}
    }
}
struct Processes {
    pid: u32,
    reads: Arc<Mutex<usize>>,
}
impl BusHandler for Processes {
    fn send<'a>(
        &'a self,
        _: &'a str,
        command: &'a str,
        _: &'a MixValue,
    ) -> BusFuture<'a, MixResult<(i32, MixValue)>> {
        Box::pin(async move {
            *self.reads.lock().unwrap() += 1;
            let value = match command {
                "monitord.snapshot" => {
                    json!({"host":"fixture","cpu":{"busy":0,"cores":1},"memory":{"used":0,"total":1},"swap":{"used":0,"total":0}})
                }
                "monitord.processes" => {
                    json!({"total":1,"rows":[{"pid":self.pid,"name":"disposable child","user":"test","cpu":0,"memory":0,"cmdline":"sleep 60"}]})
                }
                "monitord.history" => json!({"series":{}}),
                _ => panic!("unexpected effect {command}"),
            };
            Ok((0, from_json(value)?))
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
    let done = runtime.try_completion().unwrap();
    assert!(
        done.error.is_none(),
        "{:?} {} {}",
        done.error,
        done.stdout,
        done.stderr
    );
    done
}
fn event(runtime: &mut Runtime, rx: &mpsc::Receiver<()>, name: &str, args: Value) -> Completion {
    runtime.event(name.into(), args).unwrap();
    done(runtime, rx)
}
#[test]
fn mix_signal_api_uses_snapshot_and_refuses_reuse_then_refreshes_without_prompt() {
    let mut child = OwnedChild(std::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .unwrap());
    let pid = child.id();
    let reads = Arc::new(Mutex::new(0));
    let keep = reads.clone();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        Some(Box::new(move || Rc::new(Processes { pid, reads }))),
    );
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/monitor"),
        &[],
    )
    .unwrap();
    let scene = bundle.scene.clone();
    runtime.load(bundle.program).unwrap();
    done(&runtime, &rx);
    let refused = event(
        &mut runtime,
        &rx,
        "monitor.signal",
        json!({"pid":pid,"signal":"SIGTERM"}),
    )
    .reply
    .unwrap();
    assert_eq!(refused.body["error_code"], "DISABLED");
    let model = event(&mut runtime, &rx, "monitor.refresh", json!({}))
        .model
        .unwrap();
    assert_eq!(
        model["table"]["initial"]["columns"]
            .as_array()
            .unwrap()
            .len(),
        7
    );
    assert_eq!(model["table"]["initial"]["rows"][0]["cells"][6], "sleep 60");
    let mut view = viu::gold_frame::mount(&scene, model).unwrap();
    let ctx = egui::Context::default();
    viu::icons::install(&ctx);
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1040., 680.),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
    assert!(view.geometry.contains_key(&format!("processes.key.{pid}")));
    assert!(view.geometry.contains_key("processes.column.cmdline"));
    event(&mut runtime, &rx, "monitor.select", json!({"pid":pid}));
    let before = *keep.lock().unwrap();
    let refused = event(
        &mut runtime,
        &rx,
        "monitor.signal",
        json!({"pid":pid,"signal":"SIGTERM","start_time":"0"}),
    )
    .reply
    .unwrap();
    assert_eq!(refused.rc, 10);
    assert_eq!(refused.body["message"], "pid reused");
    assert_eq!(
        *keep.lock().unwrap(),
        before + 2,
        "refusal must refresh both list inputs"
    );
    assert!(child.try_wait().unwrap().is_none());
    let info = event(&mut runtime, &rx, "monitor.info", json!({}))
        .reply
        .unwrap();
    assert!(info.body["dialog"].is_null());
    assert!(info.body["start_times"][pid.to_string()].is_string());
    let allowed = event(
        &mut runtime,
        &rx,
        "monitor.signal",
        json!({"pid":pid,"signal":"SIGTERM"}),
    )
    .reply
    .unwrap();
    assert_eq!(allowed.rc, 0, "{}", allowed.body);
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(child.wait().unwrap().signal(), Some(libc::SIGTERM));
}
