// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real owner, evaluator, Jobs and private noded. No display or agent service.
#![cfg(feature = "test-support")]
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};
use viu::{app_host::test_support::Harness, bundle::Bundle};

// The CBC diagnostic recipe enables this only for a measured control run.
const PROFILE_INFO_HELPERS: bool = false;
const PROFILE_TICK: bool = false;
const VERIFY_INCREMENTAL: bool = false;
const PROFILE_PUBLICATIONS: bool = false;
const CONTROL_SECOND_FULL: bool = false;
const MAX_PATCH_APPLY_CPU_US: u64 = 100_000;
// Fable ruling D: this binary runs alone. Post-startup round-5 standalone
// max pump was 387,676 us; 800,000 us gives just over 2x margin.
// Pump/apply bounds remain independent of the unchanged 1 s call and
// 2 s owner-stall assertions. Publication bytes are telemetry under Fable A.
const MAX_STANDALONE_PUMP_US: u64 = 800_000;
// Fable A (2026-11-22): bytes are telemetry; protocol kind and container rider guard publications.

// Diagnostic only: leave the installed app and every guard unchanged.
fn publication_probe(bundle: &mut Bundle) {
    bundle.program.source.insert_str(0, "$byte_probe={start:monotonic(),ticks:0,events:0,ack_ticks:0,ack_events:0,published_ticks:0,published_events:0}\n");
    bundle.program.source = bundle
        .program
        .source
        .replace("global $publication", "global $publication, $byte_probe");
    let source = bundle.program.source.clone();
    for line in source.lines().filter(|line| line.starts_with("on ")) {
        let increment = if line == "on viu.tick" {
            "$byte_probe.ticks=$byte_probe.ticks+1;"
        } else {
            ""
        };
        bundle.program.source = bundle.program.source.replace(
            &format!("{line}\n"),
            &format!("{line}\n  {increment}$byte_probe.events=$byte_probe.events+1\n"),
        );
    }
    let needle = "  if $full then send \"viu-local\" \"scenes.model\" body=";
    let probe = r#"  $op_sizes=[]
  for $op in $operations
    push($op_sizes,{op:$op.op,path:$op.path,bytes:byte_length(json_encode($op)),items:len($op.items ?? []),remove:$op.remove ?? 0,index:$op.index ?? nil})
  end
  print("WORKSHOP_BUILD "..json_encode({revision:$revision,base:$publication.ack,full:$full,bytes:byte_length(json_encode(if $full then {generation:$ctx.generation,revision:$revision,value:$value} else $body end)),model_bytes:byte_length(json_encode($value)),operations:len($operations),op_sizes:$op_sizes,ticks:$byte_probe.ticks-$byte_probe.ack_ticks,events:$byte_probe.events-$byte_probe.ack_events,coalesced_ticks:$byte_probe.ticks-$byte_probe.published_ticks,coalesced_events:$byte_probe.events-$byte_probe.published_events,phase:if $s.discovery!=nil then "discovery" else "steady" end,since_start_ms:(monotonic()-$byte_probe.start)*1000}))
  $byte_probe.published_ticks=$byte_probe.ticks;$byte_probe.published_events=$byte_probe.events
"#;
    assert!(bundle.program.source.contains(needle));
    bundle.program.source = bundle
        .program
        .source
        .replace(needle, &format!("{probe}{needle}"));
    bundle.program.source = bundle.program.source.replace(
        "    $publication.ack=$a.revision;",
        "    $byte_probe.ack_ticks=$byte_probe.ticks;$byte_probe.ack_events=$byte_probe.events\n    $publication.ack=$a.revision;",
    );
}

#[test]
fn publication_refusal_allows_one_recovery_full() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("scene.mix"), "---\nscene: 1\nname: recovery\ncitizen: recovery\n---\n```mix\n{root:{widget:\"column\",children:[]}}\n```\n").unwrap();
    std::fs::write(dir.path().join("app.mix"), r#"
$g=viu_context().generation
send "viu-local" "scenes.model" body={generation:$g,revision:1,value:{label:"initial"}}
on viu.model.applied
  if $event.args.revision==1 then
    send "viu-local" "scenes.model.patch" body={generation:$g,base:1,revision:2,operations:[{op:"set",path:["label"],value:"patch"}]}
  end
end
on viu.model.refused
  send "viu-local" "scenes.model" body={generation:$g,revision:3,value:{label:"recovered"}}
end
on viu.test.second_full
  send "viu-local" "scenes.model" body={generation:$g,revision:4,value:{label:"second"}}
end
"#).unwrap();
    let mut host = Harness::new(Bundle::load(dir.path(), &[]).unwrap());
    host.guard_workshop_publications();
    host.refuse_next_publication();
    let end = Instant::now() + Duration::from_secs(60);
    while host.state()["model"]["label"] != "recovered" {
        host.pump();
        assert!(
            host.error().is_empty() || host.error() == "injected publication refusal",
            "{}",
            host.error()
        );
        assert!(Instant::now() < end, "recovery full was not acknowledged");
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(host.publication_summary()["refusals"], 1);
    assert_eq!(host.publication_summary()["full"], 2);
    if CONTROL_SECOND_FULL {
        host.owner_event("viu.test.second_full", json!({}));
        pump_until(&mut host, |h| h.state()["model"]["label"] == "second");
        panic!("second full escaped the kind guard");
    }
}

#[test]
fn workshop_notice_fixture_obeys_publication_protocol() {
    let broker = test_broker::Broker::start();
    let mut bundle = Bundle::load(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop"),
        &["--headless".into()],
    )
    .unwrap();
    bundle.manifest.bus_url = Some(broker.url.clone());
    bundle.launch = json!({"headless":true});
    bundle.manifest.saved_version = None;
    let mut host = Harness::new(bundle);
    host.guard_workshop_publications();
    host.probe_discovery();
    pump_until(&mut host, |h| h.ready());
    for command in ["noded.props.changed", "agent.session.updated"]
        .into_iter()
        .cycle()
        .take(20)
    {
        host.owner_event("workshop.notice", json!({"command":command,"args":{}}));
    }
    pump_until(&mut host, |h| {
        h.discovery_measurements()["commands_us"]["workshop.notice"]["n"]
            .as_u64()
            .unwrap_or(0)
            >= 20
            && h.publication_summary()["patches"].as_u64().unwrap_or(0) > 0
    });
    assert_eq!(host.publication_summary()["refusals"], 0);
    eprintln!(
        "WORKSHOP_NOTICE_PUBLICATIONS {}",
        host.publication_summary()
    );
}

fn profile_tick(bundle: &mut Bundle) {
    for (needle, replacement) in [
        (
            "  $s=$saved.reconcile($s)",
            "  $pc=viu_test_cpu();$pw=monotonic()\n  $s=$saved.reconcile($s)",
        ),
        (
            "  $projection=$frame.project($s,$presentation.project($s));$persist=$saved.project($s)",
            r#"  $pr=viu_test_cpu()
  $presentation_value=$presentation.project($s);$presentation_cpu=(viu_test_cpu()-$pr)*1000000
  $pr=viu_test_cpu();$projection=$frame.project($s,$presentation_value);$frame_cpu=(viu_test_cpu()-$pr)*1000000
  $persist=$saved.project($s)"#,
        ),
        (
            "  $body={generation:$ctx.generation,base:$publication.ack,revision:$revision,operations:$operations}",
            r#"  $body={generation:$ctx.generation,base:$publication.ack,revision:$revision,operations:$operations}
  print("WORKSHOP_BUILD "..json_encode({cpu_us:(viu_test_cpu()-$pc)*1000000,wall_us:(monotonic()-$pw)*1000000,presentation_cpu_us:$presentation_cpu,frame_cpu_us:$frame_cpu,full:$full,model_bytes:byte_length(json_encode($value)),patch_bytes:byte_length(json_encode($body)),operations:len($operations)}))"#,
        ),
    ] {
        assert!(
            bundle.program.source.contains(needle),
            "publication probe source drift: {needle}"
        );
        bundle.program.source = bundle.program.source.replace(needle, replacement);
    }
}

fn profile_info_helpers(bundle: &mut Bundle) -> tempfile::NamedTempFile {
    let dir = bundle.program.path.parent().unwrap();
    let source = std::fs::read_to_string(dir.join("controller.mix")).unwrap();
    let before = r#"    send "viu-local" "native.saved" body=$saved.project($s)
    if $rc==0 then $s.saved=$reply end
    return result($s,$m.info($s,viu_context()))"#;
    let after = r#"    $pc=viu_test_cpu();$pw=monotonic()
    $projection=$saved.project($s)
    print("WORKSHOP_HELPER "..json_encode({name:"saved.project",cpu_us:(viu_test_cpu()-$pc)*1000000,wall_us:(monotonic()-$pw)*1000000}))
    $pc=viu_test_cpu();$pw=monotonic()
    send "viu-local" "native.saved" body=$projection
    $saved_rc=$rc;$saved_reply=$reply
    print("WORKSHOP_HELPER "..json_encode({name:"native.saved",cpu_us:(viu_test_cpu()-$pc)*1000000,wall_us:(monotonic()-$pw)*1000000}))
    if $saved_rc==0 then $s.saved=$saved_reply end
    $pc=viu_test_cpu();$pw=monotonic()
    $info=$m.info($s,viu_context())
    print("WORKSHOP_HELPER "..json_encode({name:"model.info",cpu_us:(viu_test_cpu()-$pc)*1000000,wall_us:(monotonic()-$pw)*1000000}))
    return result($s,$info)"#;
    assert!(
        source.contains(before),
        "info helper probe must be installed"
    );
    let file = tempfile::Builder::new()
        .prefix(".workshop-info-profile-")
        .suffix(".mix")
        .tempfile_in(dir)
        .unwrap();
    std::fs::write(file.path(), source.replacen(before, after, 1)).unwrap();
    let path = serde_json::to_string(file.path().to_str().unwrap()).unwrap();
    bundle.program.source = bundle
        .program
        .source
        .replace("require(\"controller.mix\")", &format!("require({path})"));
    let before = r#"on workshop.info
  $out=handle($state,$event);$state=$out.state
  if not $out.deferred then reply($out.rc,$counter.wire($out.body)) end
end"#;
    let after = r#"on workshop.info
  $pc=viu_test_cpu();$pw=monotonic()
  $out=handle($state,$event)
  print("WORKSHOP_HELPER "..json_encode({name:"handle.inclusive",cpu_us:(viu_test_cpu()-$pc)*1000000,wall_us:(monotonic()-$pw)*1000000}))
  $pc=viu_test_cpu();$pw=monotonic()
  $state=$out.state
  print("WORKSHOP_HELPER "..json_encode({name:"state.assign",cpu_us:(viu_test_cpu()-$pc)*1000000,wall_us:(monotonic()-$pw)*1000000}))
  if not $out.deferred then
    $pc=viu_test_cpu();$pw=monotonic()
    $wire=$counter.wire($out.body)
    print("WORKSHOP_HELPER "..json_encode({name:"counter.wire",cpu_us:(viu_test_cpu()-$pc)*1000000,wall_us:(monotonic()-$pw)*1000000}))
    reply($out.rc,$wire)
    $wire=nil
  end
end"#;
    assert!(
        bundle.program.source.contains(before),
        "info handler probe must be installed"
    );
    bundle.program.source = bundle.program.source.replacen(before, after, 1);
    file
}

struct RefuseJobs(Arc<std::sync::atomic::AtomicUsize>);
impl mix::BusHandler for RefuseJobs {
    fn send<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        _: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<(i32, mix::value::Value)>> {
        Box::pin(async move {
            assert_eq!((target, command), ("viu-local", "jobs.start"));
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok((10, viu::runtime::from_json(json!({"error_code":"BUSY"}))?))
        })
    }
    fn emit<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
        _: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<()>> {
        Box::pin(async { Ok(()) })
    }
    fn port_exists<'a>(
        &'a self,
        _: &'a str,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<bool>> {
        Box::pin(async { Ok(true) })
    }
    fn next_incoming<'a>(
        &'a self,
    ) -> mix::evaluator::BusFuture<'a, Option<mix::evaluator::IncomingEvent>> {
        Box::pin(async { None })
    }
}

#[test]
fn job_admission_refusals_back_off_then_report_failure_once() {
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = calls.clone();
    let mut runtime = viu::runtime::Runtime::new(
        4,
        Arc::new(|| {}),
        Some(Box::new(move || std::rc::Rc::new(RefuseJobs(observed)))),
    );
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/workshop/discovery-test.mix");
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
    let end = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(done) = runtime.try_completion() {
            assert!(done.error.is_none(), "{:?}", done.error);
            assert!(done.stdout.contains("PASS"));
            break;
        }
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 3);
}

#[test]
fn discovery_turns_bound_expansion_and_blocked_admission_scans() {
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = calls.clone();
    let mut runtime = viu::runtime::Runtime::new(
        4,
        Arc::new(|| {}),
        Some(Box::new(move || std::rc::Rc::new(RefuseJobs(observed)))),
    );
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/workshop/discovery-test.mix");
    runtime.load(viu::runtime::Program { path, scene:"budget".into(), citizen:"budget".into(), source:r#"
$m=require("model.mix");$w=require("work.mix");$d=require("discovery.mix")
$s=$m.initial();$s=$d.begin($s,nil);$s.queue=[];$s.jobs={}
$items=[]
for $i=0 to 199 push($items,{name:"peer-"..$i}) end
$task={generation:$s.discovery.generation,op:"list",node:"@local",service:"noded"}
$s=$d.received($s,$task,{rc:0,body:{rc:0,body:json_encode($items)}})
if len($s.queue)!=0 or $s.discovery.nodes["@local"].list_done then raise("TEST","list expanded in its receive turn") end
$turns=0
while len($s.discovery.lists)>0
  $before=len($s.queue);$s=$d.advance($s);$delta=len($s.queue)-$before
  if $delta<1 or $delta>8 then raise("TEST","expansion must make bounded progress") end
  $turns=$turns+1
end
if $turns!=26 or len($s.queue)!=201 or not $s.discovery.nodes["@local"].list_done then raise("TEST","lost a service across turns") end
if len($s.discovery.data.services)!=201 then raise("TEST","missing streamed placeholders") end
$s=$m.initial()
for $i=1 to 100 $s.queue=push($s.queue,{owner:"discovery",retry_at:monotonic()+60}) end
$s.queue=push($s.queue,{owner:"discovery"})
$s=$w.launch($s)
if $s.serial!=0 then raise("TEST","scanned the entire blocked queue") end
for $i=1 to 6 $s=$w.launch($s) end
if $s.serial!=1 then raise("TEST","rotation starved the ready tail") end
print("PASS")
"#.into() }).unwrap();
    let end = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(done) = runtime.try_completion() {
            assert!(done.error.is_none(), "{:?}", done.error);
            assert!(done.stdout.contains("PASS"));
            break;
        }
        assert!(Instant::now() < end, "budget regression stalled");
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 3);
}

#[test]
fn discovery_expiry_flushes_pending_lists_without_describes() {
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = calls.clone();
    let mut runtime = viu::runtime::Runtime::new(
        4,
        Arc::new(|| {}),
        Some(Box::new(move || std::rc::Rc::new(RefuseJobs(observed)))),
    );
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/workshop/discovery-expiry-test.mix");
    runtime
        .load(viu::runtime::Program {
            source: std::fs::read_to_string(&path).unwrap(),
            path,
            scene: "expiry".into(),
            citizen: "expiry".into(),
        })
        .unwrap();
    let end = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(done) = runtime.try_completion() {
            assert!(done.error.is_none(), "{:?}", done.error);
            assert!(done.stdout.contains("Workshop expiry contracts PASS"));
            break;
        }
        assert!(Instant::now() < end, "expiry regression stalled");
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 0);
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
    let end = Instant::now() + Duration::from_secs(60);
    while !host.ready() {
        host.pump();
        assert!(Instant::now() < end);
    }
    host.windowed();
    (dir, host)
}

fn request(
    rt: &tokio::runtime::Runtime,
    caller: &Arc<bus::SupervisedClient>,
    command: &str,
    body: Value,
) -> tokio::task::JoinHandle<(u8, Value)> {
    let c = caller.clone();
    let command = command.to_owned();
    rt.spawn(async move {
        let (rc, body, _) = c
            .call_with_headers_raw_within(
                "workshop",
                &command,
                &BTreeMap::new(),
                &body.to_string(),
                Duration::from_secs(10),
            )
            .await
            .unwrap();
        (rc, serde_json::from_str(&body).unwrap())
    })
}
fn pump_until(host: &mut Harness, ready: impl Fn(&Harness) -> bool) {
    let end = Instant::now() + Duration::from_secs(60);
    while !ready(host) {
        host.pump();
        assert!(host.error().is_empty(), "{}", host.error());
        assert!(Instant::now() < end, "held {:?}", host.held());
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn ordering_keeps_drive_fences_and_only_read_queries_bypass_undrawn_work() {
    let broker = test_broker::Broker::start();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let caller = Arc::new(
        rt.block_on(
            bus::SupervisedClient::connect_options("ordering-caller", &broker.url).connect(),
        )
        .unwrap(),
    );
    let (_dir, mut host) = fixture(&broker.url);
    // A held, unplanned click must not stop a declared read-only ping, but
    // mutators, window commands and geometry queries keep the earlier order.
    let click = request(
        &rt,
        &caller,
        "workshop.ui.click",
        json!({"label":"Target","trace":true}),
    );
    pump_until(&mut host, |h| {
        h.held().contains(&"workshop.ui.click".into())
    });
    let mutation = request(&rt, &caller, "workshop.mutate", json!({}));
    pump_until(&mut host, |h| h.held().contains(&"workshop.mutate".into()));
    let resize = request(
        &rt,
        &caller,
        "workshop.window",
        json!({"action":"resize","width":800,"height":600}),
    );
    pump_until(&mut host, |h| h.held().contains(&"workshop.window".into()));
    let window_state = request(&rt, &caller, "workshop.window.state", json!({}));
    pump_until(&mut host, |h| {
        h.held().contains(&"workshop.window.state".into())
    });
    let state = request(&rt, &caller, "viu.state", json!({}));
    assert_eq!(
        call(&mut host, &rt, &caller, "workshop.ping").1["mutations"],
        0
    );
    pump_until(&mut host, |h| h.held().len() >= 5);
    assert!(
        !mutation.is_finished()
            && !resize.is_finished()
            && !state.is_finished()
            && !window_state.is_finished()
    );
    // Draw enough to plan the click, then deliberately stop drawing during
    // its two-pass input: even ping must wait for that self-bounded fence.
    let end = Instant::now() + Duration::from_secs(60);
    while !toolkit::drive::pending(&host.ctx) {
        host.draw();
        host.pump();
        assert!(Instant::now() < end);
    }
    let ping = request(&rt, &caller, "workshop.ping", json!({}));
    pump_until(&mut host, |h| h.held().contains(&"workshop.ping".into()));
    assert!(!ping.is_finished() && !mutation.is_finished() && !resize.is_finished());
    let end = Instant::now() + Duration::from_secs(60);
    while !mutation.is_finished()
        || !state.is_finished()
        || !click.is_finished()
        || !resize.is_finished()
        || !window_state.is_finished()
        || !ping.is_finished()
    {
        host.draw();
        host.pump();
        assert!(Instant::now() < end, "held {:?}", host.held());
        std::thread::sleep(Duration::from_millis(2));
    }
    let clicked = rt.block_on(click).unwrap();
    assert_eq!(clicked.0, 0);
    let changed = rt.block_on(mutation).unwrap();
    assert_eq!(changed.0, 0);
    assert_eq!(
        changed.1["clicks"], 1,
        "click reply {clicked:?}; mutation {changed:?}"
    );
    let geometry = rt.block_on(state).unwrap();
    assert_eq!(geometry.0, 0);
    assert_eq!(geometry.1["revision"], host.state()["revision"]);
    assert_eq!(geometry.1["geometry"], host.state()["geometry"]);
    assert_eq!(rt.block_on(window_state).unwrap().0, 0);
    // Resize may be refused by the native command; admission must still wait.
    let _ = rt.block_on(resize).unwrap();
    let _ = rt.block_on(ping).unwrap();
    rt.block_on(caller.close());
}

#[test]
fn restoring_the_old_draw_barrier_times_out_ping() {
    let broker = test_broker::Broker::start();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let caller = Arc::new(
        rt.block_on(
            bus::SupervisedClient::connect_options("negative-caller", &broker.url).connect(),
        )
        .unwrap(),
    );
    let (_dir, mut host) = fixture(&broker.url);
    host.legacy_draw_barrier();
    let c = caller.clone();
    let ping = rt.spawn(async move {
        c.call_with_headers_raw_within(
            "workshop",
            "workshop.ping",
            &BTreeMap::new(),
            "{}",
            Duration::from_millis(200),
        )
        .await
    });
    let end = Instant::now() + Duration::from_secs(60);
    while !ping.is_finished() {
        host.pump();
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(
        rt.block_on(ping).unwrap().is_err(),
        "old barrier must break ping"
    );
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
    let started = Instant::now();
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
    let end = Instant::now() + Duration::from_secs(60);
    while !task.is_finished() {
        host.pump();
        assert!(host.error().is_empty(), "{}", host.error());
        assert!(Instant::now() < end, "owner stalled: {:?}", host.held());
        std::thread::sleep(Duration::from_millis(2));
    }
    eprintln!("WORKSHOP_CALL {label} us={}", started.elapsed().as_micros());
    host.record_call(&label, started.elapsed());
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
        eprintln!("WORKSHOP_PEERS {count}");
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
        if count == 200 {
            bundle
                .program
                .source
                .insert_str(0, "$fixture_traffic=require(\"traffic.mix\")\n");
            // A deterministic large retained model, authored in Mix and
            // injected only through this test owner's native event queue.
            bundle.program.source.push_str(r#"
on viu.test.retained_model
  if $event.args.offset==0 then $state.traffic=[];$state.traffic_bytes=0 end
  for $i=$event.args.offset to $event.args.offset+7
    $state=$fixture_traffic.retain($state,{ts:to_string($i),direction:"local",command:"fixture.retained",payload:{text:$event.args.padding}})
  end
  $state=changed_workshop($state);$dirty=true
end
"#);
        }
        if PROFILE_PUBLICATIONS {
            publication_probe(&mut bundle);
        }
        let _profile_module = PROFILE_INFO_HELPERS.then(|| profile_info_helpers(&mut bundle));
        if PROFILE_TICK {
            profile_tick(&mut bundle);
        }
        // Probe only this fixture's discovery callbacks, without adding an
        // owner round-trip or changing the installed Workshop program.
        bundle.program.source = bundle.program.source.replace(
            "on viu.job.completed",
            "on viu.job.completed\n  if ($state.jobs[$event.args.tag.id].owner ?? nil)==\"discovery\" then print(\"WORKSHOP_DISCOVERY_STEP\") end",
        );
        let mut host = Harness::new(bundle);
        host.guard_workshop_publications();
        host.probe_discovery();
        if PROFILE_TICK {
            host.profile_scene_apply();
        }
        let start = Instant::now();
        while !host.ready() {
            host.pump();
            assert!(
                start.elapsed() < Duration::from_secs(60),
                "startup: {}",
                host.error()
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        eprintln!("WORKSHOP_STARTUP {}", host.discovery_measurements());
        host.probe_discovery();
        if count == 200 {
            host.discovery_numeric_bounds(MAX_STANDALONE_PUMP_US);
            host.patch_cpu_bound(MAX_PATCH_APPLY_CPU_US);
        }
        if PROFILE_TICK {
            host.profile_scene_apply();
        }
        if VERIFY_INCREMENTAL {
            host.verify_incremental_publications();
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
        for command in ["workshop.ping", "workshop.info"] {
            let invalid = request(&rt, &caller, command, json!([]));
            pump_until(&mut host, |_| invalid.is_finished());
            let answer = rt.block_on(invalid).unwrap();
            assert_eq!(answer.0, 10, "{command} must refuse non-object arguments");
            assert_eq!(answer.1["error_code"], "ARGUMENT");
        }
        let info = call(&mut host, &rt, &caller, "workshop.info").1;
        assert_eq!(info["discovering"], false);
        assert!(info["snapshot"]["services"].as_object().unwrap().len() >= count);
        if count == 200 {
            eprintln!("WORKSHOP_DISCOVERY_PHASE {}", host.discovery_measurements());
            host.probe_discovery();
            let padding="{\"name\":\"example.command\",\"description\":\"Read state\",\"read_only\":true,\"args\":{}}\n".repeat(600);
            let until = Instant::now() + Duration::from_secs(60);
            for offset in (0..48).step_by(8) {
                let marker = format!("\"ts\": \"{}\"", offset + 7);
                host.owner_event(
                    "viu.test.retained_model",
                    json!({"padding":padding,"offset":offset}),
                );
                loop {
                    host.pump();
                    assert!(host.error().is_empty(), "{}", host.error());
                    let state = host.state();
                    // Ordinary observation rows can interleave with the seed.
                    // Wait for this batch's actual final fixture row, not a
                    // row count that unrelated observations could satisfy.
                    let seeded = state["model"]["traffic_view"]["initial"]
                        .as_array()
                        .is_some_and(|rows| {
                            rows.iter().any(|row| {
                                row[4]["collapse"].as_str().is_some_and(|text| {
                                    text.contains("fixture.retained") && text.contains(&marker)
                                })
                            })
                        });
                    if seeded {
                        break;
                    }
                    assert!(
                        Instant::now() < until,
                        "retained Traffic seed was not published"
                    );
                    std::thread::sleep(Duration::from_millis(2));
                }
            }
            // Retained rows were seeded as ordinary acknowledged patches.
            // Initialisation is outside the per-tick pump budget,
            // exactly as startup is. Every subsequent ordinary tick is guarded.
            host.probe_discovery();
            host.discovery_numeric_bounds(MAX_STANDALONE_PUMP_US);
            host.patch_cpu_bound(MAX_PATCH_APPLY_CPU_US);
            if PROFILE_TICK {
                host.profile_scene_apply();
            }
            if VERIFY_INCREMENTAL {
                host.verify_incremental_publications();
            }
            for _ in 0..8 {
                assert_eq!(call(&mut host, &rt, &caller, "workshop.info").0, 0);
                let until = Instant::now() + Duration::from_millis(250);
                while Instant::now() < until {
                    host.pump();
                    std::thread::sleep(Duration::from_millis(2));
                }
            }
            let until = Instant::now() + Duration::from_millis(1200);
            while Instant::now() < until {
                host.pump();
                std::thread::sleep(Duration::from_millis(2));
            }
            assert!(
                host.state()["model"]["traffic_view"]["initial"].is_array(),
                "steady Traffic must be rendered"
            );
            assert!(
                serde_json::to_vec(&host.state()["model"]).unwrap().len() >= 2 * 1024 * 1024,
                "steady guard must exercise a retained model of at least 2 MiB"
            );
            assert!(
                host.discovery_measurements()["tick_model_bytes"]["n"]
                    .as_u64()
                    .unwrap()
                    > 0,
                "numeric model guard must observe a published tick"
            );
        }
        assert_eq!(
            host.publication_summary()["refusals"],
            0,
            "200-peer run must not require recovery"
        );
        eprintln!(
            "WORKSHOP_PUBLICATIONS peers={count} {}",
            host.publication_summary()
        );
        eprintln!(
            "WORKSHOP_PROBE peers={count} {}",
            host.discovery_measurements()
        );
        for task in tasks {
            task.abort();
        }
        for peer in peers {
            rt.block_on(peer.close());
        }
        rt.block_on(caller.close());
    }
}

// Keep 250 ms below the unchanged one-second call deadline. A candidate must
// leave this margin for desk scheduling, rather than merely avoiding a timeout.
const STARTUP_CALL_BOUND_US: u64 = 750_000;

#[test]
fn workshop_calls_from_port_appearance_at_200_peers() {
    let whole = Instant::now();
    let broker = test_broker::Broker::start();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let mut peers = Vec::new();
    let mut tasks = Vec::new();
    for i in 0..200 {
        let peer = Arc::new(
            rt.block_on(
                bus::SupervisedClient::connect_options(&format!("peer-{i:03}"), &broker.url)
                    .connect(),
            )
            .unwrap(),
        );
        let mut incoming = peer.incoming().unwrap();
        let p = peer.clone();
        tasks.push(rt.spawn(async move {
            while let Some(command) = incoming.recv().await {
                if i == 0 {
                    continue;
                }
                if i == 1 {
                    p.close().await;
                    break;
                }
                p.respond(&command, 0, "[]").await.unwrap();
            }
        }));
        peers.push(peer);
    }
    let caller = Arc::new(
        rt.block_on(
            bus::SupervisedClient::connect_options("startup-caller", &broker.url).connect(),
        )
        .unwrap(),
    );
    let sampler = caller.clone();
    // This independent client must run while construction/pumping blocks the
    // owner thread. Waiting for Harness::ready would conceal exactly that hold.
    let calls = rt.spawn(async move {
        let wait = Instant::now();
        loop {
            if sampler.list_services().await.unwrap().iter().any(|name| name == "workshop") { break; }
            assert!(wait.elapsed() < Duration::from_secs(60), "Workshop port did not appear");
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        let port = Instant::now();
        let mut samples = Vec::new();
        let mut answered = false;
        while port.elapsed() < Duration::from_millis(1500) {
            for command in ["workshop.ping", "workshop.info"] {
                if port.elapsed() >= Duration::from_millis(1500) { break; }
                let started = Instant::now();
                let answer = sampler.call_with_headers_raw_within("workshop", command, &BTreeMap::new(), "{}", Duration::from_secs(1)).await;
                // NOT_READY is the existing bounded pre-load reply, not a
                // completed app query. Keep sampling without waiting for ready.
                let ok = answer.as_ref().is_ok_and(|(rc, body, _)| {
                    *rc == 0 || (!answered && *rc == 10 && serde_json::from_str::<serde_json::Value>(body).ok().is_some_and(|value| value["error_code"] == "NOT_READY"))
                });
                answered |= answer.as_ref().is_ok_and(|(rc, _, _)| *rc == 0);
                samples.push(json!({"command":command,"start_us":started.duration_since(port).as_micros() as u64,"latency_us":started.elapsed().as_micros() as u64,"ok":ok,"reply":answer.as_ref().ok().filter(|(rc,_,_)| *rc != 0).map(|(rc,body,_)|json!({"rc":rc,"body":body})),"error":answer.err().map(|e|e.to_string())}));
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        (samples, answered)
    });
    let mut bundle = Bundle::load(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop"),
        &["--headless".into()],
    )
    .unwrap();
    bundle.manifest.bus_url = Some(broker.url.clone());
    bundle.launch = json!({"headless":true});
    bundle.manifest.saved_version = None;
    let mut host = Harness::new(bundle);
    host.probe_discovery();
    let mut painted = false;
    let mut first_pump = serde_json::Value::Null;
    while !calls.is_finished() {
        host.pump();
        if first_pump.is_null() {
            first_pump = host.discovery_measurements();
        }
        if host.ready() && !painted {
            host.draw();
            painted = true;
        }
        assert!(host.error().is_empty(), "{}", host.error());
        // The sampler bounds port discovery and every call. Report whole
        // fixture time below: the five-second budget is measured standalone,
        // rather than treating eight-wide CPU contention as a fixture fault.
        std::thread::sleep(Duration::from_millis(2));
    }
    let (samples, answered) = rt.block_on(calls).unwrap();
    let worst = samples
        .iter()
        .map(|s| s["latency_us"].as_u64().unwrap())
        .max()
        .unwrap();
    let failed = samples.iter().filter(|s| s["ok"] != true).count();
    eprintln!(
        "WORKSHOP_STARTUP_PORT {}",
        json!({"peers":200,"window_us":1500000,"bound_us":STARTUP_CALL_BOUND_US,"worst_call_us":worst,"failed":failed,"whole_us":whole.elapsed().as_micros() as u64,"samples":samples,"first_pump":first_pump,"host":host.discovery_measurements()})
    );
    assert!(
        failed == 0 && answered && worst <= STARTUP_CALL_BOUND_US,
        "startup call regression: worst_us={worst}, bound_us={STARTUP_CALL_BOUND_US}, failed={failed}"
    );
    for task in tasks {
        task.abort();
    }
    rt.block_on(caller.close());
}
