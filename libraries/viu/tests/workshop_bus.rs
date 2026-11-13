// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real owner, evaluator, Jobs and private noded. No display or agent service.
#![cfg(feature = "test-support")]
use std::{collections::BTreeMap, sync::Arc, time::{Duration, Instant}};
use serde_json::{Value, json};
use viu::{bundle::Bundle, app_host::test_support::Harness};

struct RefuseJobs(Arc<std::sync::atomic::AtomicUsize>);
impl mix::BusHandler for RefuseJobs {
    fn send<'a>(&'a self, target: &'a str, command: &'a str, _: &'a mix::value::Value) -> mix::evaluator::BusFuture<'a,mix::MixResult<(i32,mix::value::Value)>> {
        Box::pin(async move {
            assert_eq!((target,command),("viu-local","jobs.start"));
            self.0.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
            Ok((10,viu::runtime::from_json(json!({"error_code":"BUSY"}))?))
        })
    }
    fn emit<'a>(&'a self,_: &'a str,_: &'a str,_: &'a mix::value::Value)->mix::evaluator::BusFuture<'a,mix::MixResult<()>> { Box::pin(async { Ok(()) }) }
    fn port_exists<'a>(&'a self,_: &'a str)->mix::evaluator::BusFuture<'a,mix::MixResult<bool>> { Box::pin(async { Ok(true) }) }
    fn next_incoming<'a>(&'a self)->mix::evaluator::BusFuture<'a,Option<mix::evaluator::IncomingEvent>> { Box::pin(async { None }) }
}

#[test]
fn job_admission_refusals_back_off_then_report_failure_once() {
    let calls=Arc::new(std::sync::atomic::AtomicUsize::new(0));let observed=calls.clone();
    let mut runtime=viu::runtime::Runtime::new(4,Arc::new(||{}),Some(Box::new(move||std::rc::Rc::new(RefuseJobs(observed)))));
    let path=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop/admission-probe.mix");
    runtime.load(viu::runtime::Program {path,scene:"admission".into(),citizen:"admission".into(),source:r#"
$m=require("model.mix");$w=require("work.mix");$c=require("controller.mix")
$s=$m.initial();$s=$w.enqueue($s,{owner:"subscription",op:"subscribe",topic:"test",lane:"subscription"})
for $attempt=1 to 3
  $s=$w.launch($s)
  if $attempt<3 then
    if $s.queue[0].attempts!=$attempt or $s.queue[0].retry_at<=monotonic() then raise("TEST","missing backoff") end
    $serial=$s.serial;$s=$w.launch($s)
    if $serial!=$s.serial then raise("TEST","retried before backoff") end
    $s.queue[0].retry_at=0
  end
end
if len($s.queue)!=0 or len($s.admission_failures)!=1 or $s.job_error==nil then raise("TEST","failure must be terminal and visible") end
$s=$c.follow_up($s)
if len($s.jobs)!=0 or len($s.admission_failures)!=0 then raise("TEST","failed owner must be released once") end
for $i=0 to 9 $s=$c.follow_up($s) end
print("PASS")
"#.into() }).unwrap();
    let end=Instant::now()+Duration::from_secs(10);
    loop { if let Some(done)=runtime.try_completion() {assert!(done.error.is_none(),"{:?}",done.error);assert!(done.stdout.contains("PASS"));break} assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(2)); }
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed),3);
}

fn fixture(url: &str) -> (tempfile::TempDir, Harness) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("scene.mix"), r#"---
scene: 1
name: ordering
citizen: workshop
---
```mix
{root:{widget:"column",children:["target"]},target:{widget:"control",definition:{id:"target",kind:"button",label:"Target",initial:"",properties:{text:"Target"}},on_intent:"workshop.clicked"}}
```
"#).unwrap();
    std::fs::write(dir.path().join("app.mix"), r#"
$clicks=0;$mutations=0
send "viu-local" "scenes.model" body={generation:viu_context().generation,value:{marker:0}}
on workshop.ping reply(0,json_encode({clicks:$clicks,mutations:$mutations})) end
on workshop.mutate
  $mutations=$mutations+1
  send "viu-local" "scenes.model" body={generation:viu_context().generation,value:{marker:$mutations}}
  reply(0,json_encode({clicks:$clicks,mutations:$mutations}))
end
on workshop.clicked $clicks=$clicks+1;reply(0,"{}") end
"#).unwrap();
    std::fs::write(dir.path().join("contract.json"), json!({"host":{
        "service":"workshop","native_drive":true,"commands":["workshop.ping","workshop.mutate"],
        "command_metadata":[{"name":"workshop.ping","description":"Read counters","read_only":true},{"name":"workshop.mutate","description":"Change counters","read_only":false}]
    }}).to_string()).unwrap();
    let mut bundle = Bundle::load(dir.path(), &[]).unwrap();
    bundle.manifest.bus_url = Some(url.into());
    bundle.launch = json!({"headless":true});
    let mut host = Harness::new(bundle);
    let end = Instant::now()+Duration::from_secs(5);
    while !host.ready() { host.pump(); assert!(Instant::now()<end); }
    host.windowed();
    (dir, host)
}

fn request(rt: &tokio::runtime::Runtime, caller: &Arc<bus::SupervisedClient>, command: &str, body: Value)
    -> tokio::task::JoinHandle<(u8, Value)> {
    let c=caller.clone();let command=command.to_owned();
    rt.spawn(async move {
        let (rc, body, _) = c.call_with_headers_raw_within("workshop", &command, &BTreeMap::new(), &body.to_string(), Duration::from_secs(10)).await.unwrap();
        (rc,serde_json::from_str(&body).unwrap())
    })
}
fn pump_until(host: &mut Harness, ready: impl Fn(&Harness)->bool) {
    let end=Instant::now()+Duration::from_secs(5);
    while !ready(host) { host.pump(); assert!(host.error().is_empty(),"{}",host.error());assert!(Instant::now()<end,"held {:?}",host.held());std::thread::sleep(Duration::from_millis(2)); }
}

#[test]
fn ordering_keeps_drive_fences_and_only_read_queries_bypass_undrawn_work() {
    let broker=test_broker::Broker::start();
    let rt=tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let caller=Arc::new(rt.block_on(bus::SupervisedClient::connect_options("ordering-caller",&broker.url).connect()).unwrap());
    let (_dir,mut host)=fixture(&broker.url);
    // A held, unplanned click must not stop a declared read-only ping, but
    // mutators, window commands and geometry queries keep the earlier order.
    let click=request(&rt,&caller,"workshop.ui.click",json!({"label":"Target"}));
    pump_until(&mut host,|h|h.held().contains(&"workshop.ui.click".into()));
    let mutation=request(&rt,&caller,"workshop.mutate",json!({}));
    let resize=request(&rt,&caller,"workshop.window",json!({"action":"resize","width":800,"height":600}));
    let window_state=request(&rt,&caller,"workshop.window.state",json!({}));
    let state=request(&rt,&caller,"viu.state",json!({}));
    assert_eq!(call(&mut host,&rt,&caller,"workshop.ping").1["mutations"],0);
    pump_until(&mut host,|h|h.held().len()>=5);
    assert!(!mutation.is_finished() && !resize.is_finished() && !state.is_finished() && !window_state.is_finished());
    // Draw enough to plan the click, then deliberately stop drawing during
    // its two-pass input: even ping must wait for that self-bounded fence.
    let end=Instant::now()+Duration::from_secs(5);
    while !toolkit::drive::pending(&host.ctx) { host.draw();host.pump();assert!(Instant::now()<end); }
    let ping=request(&rt,&caller,"workshop.ping",json!({}));
    pump_until(&mut host,|h|h.held().contains(&"workshop.ping".into()));
    assert!(!ping.is_finished() && !mutation.is_finished() && !resize.is_finished());
    let end=Instant::now()+Duration::from_secs(5);
    while !mutation.is_finished() || !state.is_finished() || !click.is_finished() || !resize.is_finished() || !window_state.is_finished() || !ping.is_finished() {
        host.draw();host.pump();assert!(Instant::now()<end,"held {:?}",host.held());
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(rt.block_on(click).unwrap().0,0);
    let changed=rt.block_on(mutation).unwrap();assert_eq!(changed.0,0);assert_eq!(changed.1["clicks"],1);
    let geometry=rt.block_on(state).unwrap();assert_eq!(geometry.0,0);
    assert_eq!(geometry.1["revision"],host.state()["revision"]);
    assert_eq!(geometry.1["geometry"],host.state()["geometry"]);
    assert_eq!(rt.block_on(window_state).unwrap().0,0);
    // Resize may be refused by the native command; admission must still wait.
    let _=rt.block_on(resize).unwrap();let _=rt.block_on(ping).unwrap();
    rt.block_on(caller.close());
}

#[test]
fn restoring_the_old_draw_barrier_times_out_ping() {
    let broker=test_broker::Broker::start();
    let rt=tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let caller=Arc::new(rt.block_on(bus::SupervisedClient::connect_options("negative-caller",&broker.url).connect()).unwrap());
    let (_dir,mut host)=fixture(&broker.url);host.legacy_draw_barrier();
    let c=caller.clone();
    let ping=rt.spawn(async move { c.call_with_headers_raw_within("workshop","workshop.ping",&BTreeMap::new(),"{}",Duration::from_millis(200)).await });
    let end=Instant::now()+Duration::from_secs(2);
    while !ping.is_finished() { host.pump();assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(2)); }
    assert!(rt.block_on(ping).unwrap().is_err(),"old barrier must break ping");
    rt.block_on(caller.close());
}

fn call(
    host: &mut Harness,
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
        host.pump();
        assert!(host.error().is_empty(), "{}", host.error());
        assert!(Instant::now() < end, "owner stalled: {:?}", host.held());
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
        let mut host = Harness::new(bundle);
        let start = Instant::now();
        while !host.ready() {
            host.pump();
            assert!(start.elapsed() < Duration::from_secs(5), "startup");
            std::thread::sleep(Duration::from_millis(2));
        }
        let mut overlapping = false;
        let mut streamed = false;
        loop {
            assert_eq!(call(&mut host, &rt, &caller, "workshop.ping").0, 0);
            let (rc, info) = call(&mut host, &rt, &caller, "workshop.info");
            assert_eq!(rc, 0);
            let services = &info["snapshot"]["services"];
            if services["peer-002"]["Ok"].is_array() && info["discovering"] == true {
                streamed = true;
            }
            if info["discovering"] == true && !overlapping {
                let (rc, body) = call(&mut host, &rt, &caller, "workshop.refresh");
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
        for task in tasks {
            task.abort();
        }
        for peer in peers {
            rt.block_on(peer.close());
        }
        rt.block_on(caller.close());
    }
}
