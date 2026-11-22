// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Private Bus helper. MIX_TEST_MIX_EXE overrides mix:prefix/bin/mix when
//! testing with another native Mix binary on the build worker.
#[path = "owned_bus.rs"]
mod owned_bus;
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Output},
};

pub fn binary(name: &str) -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(name)
}

pub struct Bus {
    _owner: owned_bus::OwnedBus,
    root: PathBuf,
    pub url: String,
}

impl Bus {
    #[allow(dead_code)] // Only the input integration target uses subscriptions.
    pub fn observe(&self) -> Observer {
        let child = Command::new(mix_binary())
            .arg("--serve")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/observer.mix"))
            .args(["--name", "headless-observer"])
            .env("MIX_NODED_URL", &self.url)
            .env("MIX_RUN", self.root.join("run"))
            .env("MIX_ETC", self.root.join("etc"))
            .env("MIX_VAR", self.root.join("var"))
            .env("MIX_NODE_CONFIG", self.root.join("etc/node.conf.mix"))
            .spawn()
            .expect("start topic observer with native Mix (MIX_TEST_MIX_EXE)");
        let observer = Observer(child);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            let reply = self.request("headless-observer", "observer.count", serde_json::json!({}));
            if reply["rc"] == 0 {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "observer did not register: {reply}"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        observer
    }

    pub fn start(label: &str) -> Self {
        let owner = owned_bus::OwnedBus::start(label);
        Self { root: owner.root.clone(), url: owner.url.clone(), _owner: owner }
    }

    pub fn ready(&self, service: &str) {
        check(&helper("ready", &self.root, &[service], Some(&self.url)));
    }

    pub fn request(&self, service: &str, command: &str, args: Value) -> Value {
        let output = helper(
            "call",
            &self.root,
            &[command, &args.to_string(), service],
            Some(&self.url),
        );
        check(&output);
        let text = String::from_utf8_lossy(&output.stdout);
        let mut reply: Value =
            serde_json::from_str(text.lines().last().expect("Bus reply")).unwrap();
        // A service still registering returns an error string. Keep it for
        // readiness callers, which inspect rc and retry; only success has a
        // JSON response body.
        if reply["rc"] == 0
            && let Some(body) = reply["body"].as_str()
        {
            reply["body"] = serde_json::from_str(body).unwrap();
        }
        reply
    }

    pub fn call(&self, service: &str, command: &str, args: Value) -> Value {
        let reply = self.request(service, command, args);
        assert_eq!(reply["rc"], 0, "{service} {command}: {reply}");
        reply["body"].clone()
    }
}

#[allow(dead_code)]
pub struct Observer(Child);

impl Drop for Observer {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn check(output: &Output) {
    assert!(
        output.status.success(),
        "Bus helper: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn helper(action: &str, root: &Path, rest: &[&str], url: Option<&str>) -> Output {
    Command::new(mix_binary())
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/bus.mix"))
        .arg(action)
        .arg(root)
        .args(rest)
        .env("MIX_NODED_URL", url.unwrap_or("ws://127.0.0.1:1"))
        .env("MIX_RUN", root.join("run"))
        .env("MIX_ETC", root.join("etc"))
        .env("MIX_VAR", root.join("var"))
        .env("MIX_NODE_CONFIG", root.join("etc/node.conf.mix"))
        .output()
        .expect("cannot start native Mix; set MIX_TEST_MIX_EXE to its binary path (default /opt/mix/bin/mix)")
}

fn mix_binary() -> std::ffi::OsString {
    std::env::var_os("MIX_TEST_MIX_EXE").unwrap_or_else(|| "/opt/mix/bin/mix".into())
}
