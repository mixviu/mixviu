// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit acceptance against an isolated native Bus and headless compositor.
use bus::native_client::{NodedClient, UnixConnectOutcome};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
async fn call(client: &NodedClient, service: &str, command: &str, body: Value) -> (u8, Value) {
    let (rc, body, _) = client
        .call_with_headers_raw_within(
            service,
            command,
            &Default::default(),
            &body.to_string(),
            Duration::from_secs(40),
        )
        .await
        .unwrap();
    (rc, serde_json::from_str(&body).unwrap_or(json!(body)))
}
async fn ok(client: &NodedClient, service: &str, command: &str, body: Value) -> Value {
    let (rc, value) = call(client, service, command, body).await;
    assert_eq!(rc, 0, "{command}: {value}");
    value
}
#[test]
#[ignore = "requires CBC-built viud backend-headless, viu, mixd and render node"]
fn bus_sweep_record_reference_play_and_decode_build_states() {
    let bin = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    for name in ["viud", "viu", "mixd"] {
        assert!(bin.join(name).is_file(), "build {name} first");
    }
    let broker = test_broker::Broker::start();
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("node.conf.mix");
    let user = unsafe { std::ffi::CStr::from_ptr((*libc::getpwuid(libc::geteuid())).pw_name) }
        .to_str()
        .unwrap();
    std::fs::write(
        &config,
        format!(
            "noded: {{unix_socket:{},account:{}}}\n",
            json!(broker.endpoint),
            json!(user)
        ),
    )
    .unwrap();
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let log = std::fs::File::create(root.path().join("native.log")).unwrap();
    let command = |name: &str| {
        let mut c = Command::new(bin.join(name));
        c.env("HOME", root.path())
            .env("XDG_RUNTIME_DIR", root.path())
            .env("XDG_CONFIG_HOME", root.path().join("config"))
            .env("XDG_DATA_HOME", root.path().join("data"))
            .env("XDG_STATE_HOME", root.path().join("state"))
            .env("XDG_CACHE_HOME", root.path().join("cache"))
            .env("MIX_NODED_URL", &broker.url)
            .env("MIX_BUS_SOCKET", &broker.endpoint)
            .env("MIX_NODE_CONFIG", &config)
            .env("MIX_BROKER_ACCOUNT", user)
            .env("MIX_RUN", root.path().join("run"))
            .env("MIX_ETC", root.path().join("etc"))
            .env("MIX_VAR", root.path().join("var"))
            .env_remove("MIX_SESSION_FD")
            .env_remove("MIX_VT")
            .env_remove("MIX_TEST_VIUD_OSD_BUNDLE")
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log.try_clone().unwrap());
        c
    };
    let mut comp = command("viud");
    comp.args([
        "--headless",
        "--size",
        "1280x720",
        "--socket",
        "wayland-deck",
        "--bus-service",
        "comp-deck",
    ]);
    let _comp = Owned(comp.spawn().unwrap());
    let start = Instant::now();
    while !root.path().join("wayland-deck").exists() {
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "{}",
            std::fs::read_to_string(root.path().join("native.log")).unwrap()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut app = command("viu");
    app.args(["run"])
        .arg(workspace.join("apps/deck"))
        .arg("--noded-url")
        .arg(&broker.url)
        .env("WAYLAND_DISPLAY", "wayland-deck")
        .env("WINIT_UNIX_BACKEND", "wayland");
    let _app = Owned(app.spawn().unwrap());
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let evidence = std::env::temp_dir().join("mixviu-deck-app-native");
    std::fs::create_dir_all(&evidence).unwrap();
    let path = evidence.join("reference.vfl");
    assert!(
        !path.exists(),
        "remove only your previous evidence before rerunning: {}",
        path.display()
    );
    rt.block_on(async {
        let UnixConnectOutcome::VerifiedUnix(caller)=NodedClient::connect_unix("",&broker.url,&broker.options(),None).await.unwrap() else {panic!("verified native ingress required")};
        let c=caller.client();
        loop {
            let reply=call(c,"deck","deck.state",json!({})).await;
            if reply.0==0 {break;}
            assert!(start.elapsed()<Duration::from_secs(30),"deck startup: {}",std::fs::read_to_string(root.path().join("native.log")).unwrap());
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
        ok(c,"deck","deck.open",json!({"path":workspace.join("apps/deck/examples/vfl-self-recording.json")})).await;
        let actions=ok(c,"deck","deck.actions",json!({})).await;assert_eq!(actions["actions"].as_array().unwrap().len(),6);
        ok(c,"deck","deck.goto",json!({"slide":"before","step":1})).await;
        assert_eq!(ok(c,"deck","deck.prev",json!({})).await["step"],0);
        assert_eq!(ok(c,"deck","deck.next",json!({})).await["step"],1);
        ok(c,"deck","deck.execute",json!({"id":"deck.goto","params":{"slide":"before"}})).await;
        let bad=call(c,"deck","deck.goto",json!({"slide":"before","step":99})).await;assert_eq!(bad.0,10);assert_eq!(bad.1["error_code"],"DECK_RANGE");
        tokio::time::sleep(Duration::from_millis(800)).await;
        let recording=ok(c,"comp-deck","comp.record.start",json!({"request_id":"deck-reference","target":{"output":"headless-0"},"path":path,"fps":10,"cursor":false,"max_seconds":40})).await;
        let result=ok(c,"deck","deck.play",json!({"slides":["before","recorder"],"beats":[[0,3,11],[0,3,11]]})).await;
        assert_eq!(result["finished"],true);assert_eq!(result["playing"],false);
        tokio::time::sleep(Duration::from_millis(250)).await;
        ok(c,"comp-deck","comp.record.stop",json!({"session":recording["session"]})).await;
        let state=ok(c,"deck","deck.state",json!({})).await;assert_eq!(state["slide"],"recorder");
    });
    let verify = Command::new(bin.join("mixd"))
        .args(["vfl", "verify"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stdout)
    );
    println!("VFL_VERIFY={}", String::from_utf8_lossy(&verify.stdout));
    let mut reader = media_vfl::reader::Reader::open(&path).unwrap();
    let (width, height) = (reader.width, reader.height);
    let mut samples = Vec::new();
    while let Some(frame) = reader.read().unwrap() {
        for target in [2.0, 5.0, 13.0, 21.0] {
            if (frame.t_us as f64 / 1e6 - target).abs() < 0.045 {
                let rgb: Vec<u8> = frame
                    .pixels
                    .chunks_exact(4)
                    .flat_map(|p| [p[2], p[1], p[0]])
                    .collect();
                let image = image::RgbImage::from_raw(width, height, rgb).unwrap();
                let red = image
                    .pixels()
                    .filter(|p| p[0] > 170 && p[0] > p[1].saturating_add(50))
                    .count();
                let green = image
                    .pixels()
                    .filter(|p| {
                        p[1] > 130
                            && p[1] > p[0].saturating_add(40)
                            && p[1] > p[2].saturating_add(20)
                    })
                    .count();
                image
                    .save(evidence.join(format!("sample-{target}.png")))
                    .unwrap();
                samples.push((target, red, green));
            }
        }
    }
    assert_eq!(samples.len(), 4, "{samples:?}");
    assert!(
        samples[1].1 > samples[0].1 + 20,
        "before step 1 must reveal red before-card text: {samples:?}"
    );
    assert!(
        samples[3].2 > samples[2].2 + 20,
        "recorder step 1 must reveal green stats: {samples:?}"
    );
    std::fs::copy(root.path().join("native.log"), evidence.join("native.log")).unwrap();
    println!("DECK_NATIVE={} samples={samples:?}", evidence.display());
}
