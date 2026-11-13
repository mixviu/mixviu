// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real host/Bus/shell smoke; run explicitly after building viu, mix and mixd.
use bus::native_client::{NodedClient, UnixConnectOutcome};
use serde_json::json;
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

async fn call(
    client: &NodedClient,
    service: &str,
    command: &str,
    body: serde_json::Value,
) -> Result<(u8, serde_json::Value), String> {
    let (rc, body, _) = client
        .call_with_headers_raw_within(
            service,
            command,
            &Default::default(),
            &body.to_string(),
            Duration::from_secs(2),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok((rc, serde_json::from_str(&body).unwrap_or(json!(body))))
}

struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
#[ignore = "requires CBC-built viu, mix and mixd beside the test binaries"]
fn headless_term_default_and_override_attach_and_exit_after_last_tab_reply() {
    let broker = test_broker::Broker::start();
    let root = tempfile::tempdir().unwrap();
    let bin = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let prefix = root.path().join("prefix");
    std::fs::create_dir_all(prefix.join("bin")).unwrap();
    for name in ["mix", "mixd"] {
        std::os::unix::fs::symlink(bin.join(name), prefix.join("bin").join(name)).unwrap();
    }
    let config = root.path().join("node.conf.mix");
    let user = unsafe { std::ffi::CStr::from_ptr((*libc::getpwuid(libc::geteuid())).pw_name) }
        .to_str()
        .unwrap();
    std::fs::write(
        &config,
        format!(
            "noded: {{unix_socket: {}, account: {}}}\n",
            json!(broker.endpoint),
            json!(user)
        ),
    )
    .unwrap();
    let bundle = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/term");
    let launch = |service: Option<&str>, home: &str| {
        let home = root.path().join(home);
        std::fs::create_dir_all(&home).unwrap();
        let mut cmd = Command::new(bin.join("viu"));
        cmd.arg("run")
            .arg(&bundle)
            .arg("--noded-url")
            .arg(&broker.url);
        if let Some(service) = service {
            cmd.args([
                "--service",
                service,
                "--title",
                "Term",
                "--app-id",
                "mixviu.term-test",
            ]);
        }
        Owned(
            cmd.arg("--headless")
                .env("HOME", &home)
                .env("XDG_RUNTIME_DIR", &home)
                .env("XDG_CONFIG_HOME", &home)
                .env("XDG_DATA_HOME", &home)
                .env("MIX_PREFIX", &prefix)
                .env("MIX_NODE_CONFIG", &config)
                .env("MIX_BUS_SOCKET", &broker.endpoint)
                .env("MIX_BROKER_ACCOUNT", user)
                .env("MIX_RUN", &home)
                .env("MIX_ETC", &home)
                .env("MIX_VAR", &home)
                .env_remove("MIX_SESSION_FD")
                .env_remove("MIX_VIU_SERVICE")
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        )
    };
    let mut mix = launch(Some("term-test"), "mix-home");
    let mut default = launch(None, "default-home");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let UnixConnectOutcome::VerifiedUnix(caller) =
            NodedClient::connect_unix("", &broker.url, &broker.options(), None)
                .await
                .unwrap()
        else {
            panic!("verified ingress required")
        };
        for service in ["term-test", "term"] {
            let start = Instant::now();
            let help = loop {
                if let Ok((0, value)) = call(caller.client(), service, "HELP", json!({})).await {
                    break value;
                }
                assert!(
                    start.elapsed() < Duration::from_secs(30),
                    "{service} never answered HELP"
                );
                tokio::time::sleep(Duration::from_millis(50)).await;
            };
            assert!(
                help.as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v["name"] == format!("{service}.tabs")),
                "{help}"
            );
            let (rc, tabs) = call(
                caller.client(),
                service,
                &format!("{service}.tabs"),
                json!({}),
            )
            .await
            .unwrap();
            assert_eq!(rc, 0);
            assert!(tabs.as_str().unwrap().contains("child_pid="), "{tabs}");
            let (rc, _) = call(
                caller.client(),
                service,
                &format!("{service}.type"),
                json!({"pane":1,"instance":0,"text":"stale"}),
            )
            .await
            .unwrap();
            assert_ne!(rc, 0, "stale process token must reject input");
            let (rc, _) = call(
                caller.client(),
                service,
                &format!("{service}.type"),
                json!({"pane":1,"text":"print(731942)\n"}),
            )
            .await
            .unwrap();
            assert_eq!(rc, 0);
            let start = Instant::now();
            loop {
                let (rc, value) = call(
                    caller.client(),
                    service,
                    &format!("{service}.snapshot"),
                    json!({"pane":1}),
                )
                .await
                .unwrap();
                assert_eq!(rc, 0);
                // Two occurrences prove output, rather than just PTY echo.
                if value.to_string().matches("731942").count() >= 2 {
                    break;
                }
                assert!(
                    start.elapsed() < Duration::from_secs(15),
                    "shell output missing: {value}"
                );
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
        let (rc, _) = call(
            caller.client(),
            "term-test",
            "term-test.tab.close",
            json!({"id":1}),
        )
        .await
        .unwrap();
        assert_eq!(rc, 0, "last-tab reply must cross transport before exit");
        let start = Instant::now();
        loop {
            if let Some(status) = mix.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(15),
                "last-tab host did not exit"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(
            default.0.try_wait().unwrap().is_none(),
            "default instance was disturbed"
        );
        assert_eq!(
            call(caller.client(), "term", "term.tab.close", json!({"id":1}))
                .await
                .unwrap()
                .0,
            0
        );
        let start = Instant::now();
        loop {
            if let Some(status) = default.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(15),
                "default host did not exit"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        caller.client().close().await;
    });
}
