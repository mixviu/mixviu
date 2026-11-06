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
impl std::ops::Deref for OwnedChild {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for OwnedChild {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
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
fn event(
    runtime: &mut Runtime,
    rx: &mpsc::Receiver<()>,
    jobs: &mut viu::jobs::Jobs,
    name: &str,
    args: Value,
) -> Completion {
    let sequence = runtime.event(name.into(), args).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut answer = None;
    let mut model = None;
    let mut barrier = None;
    loop {
        for completed in jobs.poll() {
            runtime
                .notify(
                    "viu.job.completed".into(),
                    json!({"tag":completed.tag,"rc":completed.rc,"body":completed.body}),
                )
                .unwrap();
        }
        for mut completed in runtime.drain() {
            assert!(completed.error.is_none(), "{:?}", completed.error);
            if completed.model.is_some() {
                model = completed.model.take();
            }
            if completed.sequence == sequence {
                answer = Some(completed);
            } else if Some(completed.sequence) == barrier {
                let mut answer = answer.unwrap();
                answer.model = model;
                return answer;
            }
        }
        if answer.is_some() && jobs.pending() == 0 && barrier.is_none() {
            barrier = Some(
                runtime
                    .event("monitor.debug.state".into(), json!({}))
                    .unwrap(),
            );
        }
        assert!(
            std::time::Instant::now() < deadline,
            "native Jobs/reducer timeout"
        );
        let _ = rx.recv_timeout(Duration::from_millis(2));
    }
}
#[test]
fn mix_signal_api_uses_snapshot_and_refuses_reuse_then_refreshes_without_prompt() {
    let mut child = OwnedChild(
        std::process::Command::new("sleep")
            .arg("60")
            .spawn()
            .unwrap(),
    );
    let pid = child.id();
    let reads = Arc::new(Mutex::new(0));
    let keep = reads.clone();
    let (tx, rx) = mpsc::channel();
    let wake: viu::runtime::Wake = Arc::new(move || {
        let _ = tx.send(());
    });
    let maker: viu::runtime::BackendMaker = Arc::new(move || {
        let reads = reads.clone();
        Box::new(move || Rc::new(Processes { pid, reads }))
    });
    let mut jobs = viu::jobs::Jobs::new(4, wake.clone());
    jobs.set_native_backend(maker.clone());
    let mut runtime = Runtime::new(8, wake, Some(jobs.backend(Some(maker()))));
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
        &mut jobs,
        "monitor.signal",
        json!({"pid":pid,"signal":"SIGTERM"}),
    )
    .reply
    .unwrap();
    assert_eq!(refused.body["error_code"], "DISABLED");
    let model = event(&mut runtime, &rx, &mut jobs, "monitor.refresh", json!({}))
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
    let before_navigation = *keep.lock().unwrap();
    event(
        &mut runtime,
        &rx,
        &mut jobs,
        "monitor.pause",
        json!({"on":true}),
    );
    event(
        &mut runtime,
        &rx,
        &mut jobs,
        "monitor.tab",
        json!({"tab":"performance"}),
    );
    event(
        &mut runtime,
        &rx,
        &mut jobs,
        "monitor.section",
        json!({"section":"tcp"}),
    );
    assert_eq!(
        *keep.lock().unwrap(),
        before_navigation,
        "paused navigation must not fetch TCP or process readings"
    );
    event(
        &mut runtime,
        &rx,
        &mut jobs,
        "monitor.tab",
        json!({"tab":"processes"}),
    );
    assert_eq!(
        *keep.lock().unwrap(),
        before_navigation,
        "paused return must preserve the held table"
    );
    event(
        &mut runtime,
        &rx,
        &mut jobs,
        "monitor.select",
        json!({"pid":pid}),
    );
    let before = *keep.lock().unwrap();
    let refused = event(
        &mut runtime,
        &rx,
        &mut jobs,
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
    let info = event(
        &mut runtime,
        &rx,
        &mut jobs,
        "monitor.debug.state",
        json!({}),
    )
    .reply
    .unwrap();
    assert!(info.body["dialog"].is_null());
    assert!(info.body["start_times"][pid.to_string()].is_string());
    let allowed = event(
        &mut runtime,
        &rx,
        &mut jobs,
        "monitor.signal",
        json!({"pid":pid,"signal":"SIGTERM"}),
    )
    .reply
    .unwrap();
    assert_eq!(allowed.rc, 0, "{}", allowed.body);
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(child.wait().unwrap().signal(), Some(libc::SIGTERM));
}
