// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Two production brokers, separate roots and verified Unix clients. The hop
//! between brokers is native ABP; the fixture never relays application data.
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use bus::native_client::{BrokerAccount, NodedClient, UnixConnectOptions, UnixConnectOutcome};
use ed25519_dalek::{Signer as _, SigningKey};
use mesh_trust::inventory::{
    ALG_ED25519, CANONICAL_ENCODING_V1, InvSignature, InventoryPayload, KeyStatus, SignedInventory,
    VerifyKey,
};
use serde_json::{Value, json};
use std::{
    fs,
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

struct Fixture {
    root: PathBuf,
    children: Vec<Child>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for child in &mut self.children {
            let _ = child.kill();
            let _ = child.wait();
        }
        if std::thread::panicking() {
            eprintln!("mesh process evidence retained at {}", self.root.display());
        } else {
            fs::remove_dir_all(&self.root).expect("remove owned fixture root");
        }
    }
}

fn write(path: &Path, value: &impl serde::Serialize) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

#[test]
fn frame_enforcement_refuses_startup_without_identity_source() {
    let root = std::env::temp_dir().join(format!(
        "noded-no-authority-{:032x}",
        rand::random::<u128>()
    ));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o711)).unwrap();
    let fixture = Fixture {
        root,
        children: Vec::new(),
    };
    for dir in ["etc", "var", "run", "home"] {
        fs::create_dir(fixture.root.join(dir)).unwrap();
    }
    let config = fixture.root.join("etc/node.conf.mix");
    write(
        &config,
        &json!({"node":"alpha","wg_ip":"127.0.0.1","noded":{"port":0,"frame_policy":"enforce","admission":"off","unix_socket":fixture.root.join("run/bus.sock")}}),
    );
    let result = Command::new(env!("CARGO_BIN_EXE_noded"))
        .args(["serve", "--no-monitor", "--no-log"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", fixture.root.join("home"))
        .env("MIX_ETC", fixture.root.join("etc"))
        .env("MIX_VAR", fixture.root.join("var"))
        .env("MIX_RUN", fixture.root.join("run"))
        .env("MIX_NODE_CONFIG", config)
        .output()
        .unwrap();
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("no identity source"), "{stderr}");
}

async fn connect(
    root: &Path,
    name: &str,
    url: &str,
    child: &mut Child,
) -> bus::native_client::VerifiedConnection {
    let mut options = UnixConnectOptions::new(BrokerAccount {
        // SAFETY: process credentials, never an application-supplied identity.
        uid: unsafe { libc::geteuid() },
        gid: unsafe { libc::getegid() },
    });
    options.endpoint = Some(root.join("run/bus.sock"));
    options.default_endpoints = Vec::new();
    options.require_native_session = true;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            child.try_wait().unwrap().is_none(),
            "broker exited; see {}",
            root.display()
        );
        let error = match NodedClient::connect_unix(name, url, &options, None).await {
            Ok(UnixConnectOutcome::VerifiedUnix(connection)) => return connection,
            Ok(_) => panic!("Unix verification cannot fall back to TCP"),
            Err(error) => error,
        };
        assert!(
            tokio::time::Instant::now() < deadline,
            "broker readiness deadline: {}: {error}",
            root.display(),
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_production_brokers_route_verified_native_clients() {
    production_mesh(1, 1, "enforce", false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admission_enforce_routes_to_newer_inventory_with_frame_observe() {
    production_mesh(7, 8, "observe", false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admission_enforce_routes_to_older_inventory_with_frame_observe() {
    production_mesh(7, 6, "observe", false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_session_caller_crosses_an_admitted_bridge() {
    production_mesh(1, 1, "enforce", true).await;
}

async fn production_mesh(dialler_epoch: u64, listener_epoch: u64, frame_policy: &str, session_caller: bool) {
    tokio::time::timeout(Duration::from_secs(30), async {
        let root = std::env::temp_dir().join(format!(
            "noded-mesh-{}-{:032x}", std::process::id(), rand::random::<u128>()
        ));
        fs::create_dir(&root).unwrap();
        // Native ingress serves separate UIDs and refuses non-traversable
        // existing ancestors. No directory listing or write access is granted.
        fs::set_permissions(&root, fs::Permissions::from_mode(0o711)).unwrap();
        let mut fixture = Fixture { root, children: Vec::new() };
        let names = ["alpha", "beta"];
        let ips = ["127.0.0.2", "127.0.0.3"];
        let reservations: Vec<_> = ips.iter().map(|ip| TcpListener::bind((*ip, 0)).unwrap()).collect();
        let ports: Vec<_> = reservations.iter().map(|listener| listener.local_addr().unwrap().port()).collect();
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let pubkey = B64.encode(signing_key.verifying_key().to_bytes());
        let payload = InventoryPayload {
            schema_version: 1,
            canonical_encoding: CANONICAL_ENCODING_V1.into(),
            mesh: "example.test".into(),
            subnet: "127.0.0.0/24".into(),
            epoch: 1,
            signed_at: datetime::Utc::now().to_rfc3339(),
            valid_until: (datetime::Utc::now() + datetime::Duration::minutes(10)).to_rfc3339(),
            hub: vec!["alpha".into()],
            verify_keys: vec![VerifyKey {
                key_id: "genesis".into(), pubkey: pubkey.clone(),
                key_type: ALG_ED25519.into(), status: KeyStatus::Active,
            }],
            members: json!([
                {"name":"alpha", "mesh_ip":ips[0], "bus":true, "status":"active", "noded_port":ports[0],
                 "credentials":[{"kind":"d2","pubkey":B64.encode(SigningKey::from_bytes(&[7;32]).verifying_key().to_bytes()),"from_epoch":1,"until_epoch":null}]},
                {"name":"beta", "mesh_ip":ips[1], "bus":true, "status":"active", "noded_port":ports[1],
                 "credentials":[{"kind":"d2","pubkey":B64.encode(SigningKey::from_bytes(&[8;32]).verifying_key().to_bytes()),"from_epoch":1,"until_epoch":null}]}
            ]),
            recovery: None,
            recovery_generation: None,
        };
        let mut signed = SignedInventory {
            signatures: vec![InvSignature {
                key_id: "genesis".into(), alg: ALG_ED25519.into(),
                sig: B64.encode(signing_key.sign(&payload.canonical_bytes()).to_bytes()),
            }],
            payload,
        };
        let roots: Vec<_> = names.iter().map(|name| fixture.root.join(name)).collect();
        let urls: Vec<_> = ips.iter().zip(&ports).map(|(ip, port)| format!("ws://{ip}:{port}/ws")).collect();
        drop(reservations);
        for i in 0..2 {
            signed.payload.epoch = if i == 0 { dialler_epoch } else { listener_epoch };
            signed.signatures[0].sig = B64.encode(signing_key.sign(&signed.payload.canonical_bytes()).to_bytes());
            let root = &roots[i];
            for dir in ["etc/noded", "var/noded", "run", "home"] {
                fs::create_dir_all(root.join(dir)).unwrap();
            }
            fs::write(root.join("etc/noded/genesis.pub"), &pubkey).unwrap();
            fs::write(root.join("etc/noded/d2.seed"), B64.encode([7 + i as u8;32])).unwrap();
            fs::set_permissions(root.join("etc/noded/d2.seed"), fs::Permissions::from_mode(0o600)).unwrap();
            write(&root.join("var/noded/inventory.signed"), &signed);
            write(&root.join("etc/node.conf.mix"), &json!({
                "node":names[i], "wg_ip":ips[i],
                "noded":{"port":ports[i],"unix_socket":root.join("run/bus.sock"),"admission":"enforce","frame_policy":frame_policy}
            }));
            write(&root.join("etc/mesh.conf.mix"), &json!({"node_name":names[i],"peers":[]}));
            let log = fs::File::create(root.join("noded.log")).unwrap();
            fixture.children.push(Command::new(env!("CARGO_BIN_EXE_noded"))
                .args(["serve", "--no-monitor", "--no-log"])
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", root.join("home"))
                .env("MIX_ETC", root.join("etc"))
                .env("MIX_VAR", root.join("var"))
                .env("MIX_RUN", root.join("run"))
                .env("MIX_NODE_CONFIG", root.join("etc/node.conf.mix"))
                .env("RUST_LOG", "info")
                .stdout(Stdio::from(log.try_clone().unwrap()))
                .stderr(Stdio::from(log)).spawn().unwrap());
        }
        let first = connect(&roots[0], "caller-one", &urls[0], &mut fixture.children[0]).await;
        let second = connect(&roots[0], "caller-two", &urls[0], &mut fixture.children[0]).await;
        let service = connect(&roots[1], "echo", &urls[1], &mut fixture.children[1]).await;
        let session_name = if session_caller {
            let key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let record = first.session_allocate(&key, bus::native_session::Policy::DefaultOpen)
                .await.unwrap().record;
            assert!(record.name.starts_with(&format!("t{}-", record.owner_uid)));
            Some(record.name)
        } else {
            None
        };
        for client in [first.client(), second.client(), service.client()] {
            let inventory = client.call("noded", "noded.inventory", json!({})).await.unwrap();
            assert_eq!(inventory["posture"], "verified", "{inventory}");
            let peers = client.call("noded", "noded.peers", json!({})).await.unwrap();
            assert_eq!(peers["source"], "signed-inventory", "{peers}");
            assert!(peers["peers"].as_array().unwrap().iter().any(|peer| peer["name"] == "beta" || peer["name"] == "alpha"));
        }
        let responder = async {
            for _ in 0..2 {
                let delivery = service.recv_shared().await.unwrap();
                let command = delivery.command();
                assert_eq!(command.command, "echo.tag");
                let value: Value = serde_json::from_str(&command.body).unwrap();
                if value["tag"] == "one" && let Some(name) = &session_name {
                    // These are produced by the real alpha sender and beta
                    // receiver, not supplied by this fixture's request.
                    assert_eq!(command.header("broker_service"), Some(name.as_str()));
                    assert_eq!(command.header("broker_peer"), Some("alpha"));
                    assert_eq!(command.header("broker_origin"), Some("mesh"));
                    assert!(delivery.trusted_context().is_none(), "remote session is not a local principal");
                }
                service.client().respond(command, 0, &value.to_string()).await.unwrap();
            }
        };
        // Both fresh connections use the same call counter. The mesh must
        // restore each caller's correlation ID without exchanging their replies.
        let (one, two, ()) = tokio::join!(
            first.client().call("echo.beta.bus", "echo.tag", json!({"tag":"one"})),
            second.client().call("echo.beta.bus", "echo.tag", json!({"tag":"two"})),
            responder
        );
        assert_eq!(one.unwrap(), json!({"tag":"one"}));
        assert_eq!(two.unwrap(), json!({"tag":"two"}));
        // Alpha's outgoing transport is registered on beta as bridge-alpha.
        // It is visible in discovery but is not a command-serving endpoint.
        for command in ["HELP", "x.ping"] {
            let reply = tokio::time::timeout(Duration::from_millis(250),
                service.client().call_typed("bridge-alpha", command, json!({})))
                .await.expect("link refusal must be immediate").unwrap();
            match reply {
                bus::PortReply::AppError { rc, message } => {
                    assert_eq!(rc, 10);
                    assert!(message.contains("UNKNOWN_COMMAND"), "{message}");
                }
                reply => panic!("expected link refusal, got {reply:?}"),
            }
        }
        let drain = async {
            let delivery = service.recv_shared().await.unwrap();
            let command = delivery.command();
            service.client().deregister().await.unwrap();
            service.client().respond(command, 16,
                r#"{"error_code":"HANDLER_CANCELLED"}"#).await.unwrap();
        };
        let (cancelled, ()) = tokio::join!(
            first.client().call_typed("echo.beta.bus", "echo.tag", json!({"tag":"shutdown"})),
            drain,
        );
        match cancelled.unwrap() {
            bus::PortReply::AppError { rc, message } => {
                assert_eq!(rc, 16);
                assert!(message.contains("HANDLER_CANCELLED"), "{message}");
            }
            reply => panic!("expected shutdown refusal, got {reply:?}"),
        }
        assert!(first.client().call("echo.unknown.bus", "echo.tag", json!({})).await.is_err());
        assert_eq!(first.client().call("noded", "noded.ping", json!({})).await.unwrap()["pong"], true);
        if session_caller {
            first.client().close().await;
        } else {
            first.client().deregister().await.unwrap();
        }
        second.client().deregister().await.unwrap();
        service.client().close().await;
    }).await.expect("native two-process mesh acceptance deadline");
}
