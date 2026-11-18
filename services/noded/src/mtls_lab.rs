// SPDX-License-Identifier: MIT OR Apache-2.0
// Included in noded::tests: exercises the real WebSocket handler and router.
type LabWs =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

fn lab_inventory(epoch: u64, tombstone: bool) -> crate::authority::Posture {
    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use mesh_trust::inventory::*;
    let root = SigningKey::from_bytes(&[90; 32]);
    let b64 = base64::engine::general_purpose::STANDARD;
    let members = serde_json::Value::Array([("alpha", 11u8), ("beta", 22u8)].into_iter().map(|(name, seed)| {
        serde_json::json!({"name":name,"status":if tombstone && name == "beta" {"tombstoned"} else {"active"},"bus":true,
            "mesh_ip":if name == "alpha" {"192.0.2.1"} else {"192.0.2.2"},
            "credentials":[{"kind":"d2","pubkey":b64.encode(SigningKey::from_bytes(&[seed;32]).verifying_key().to_bytes()),"from_epoch":1,"until_epoch":null}]})
    }).collect());
    let payload = InventoryPayload {
        schema_version: 1,
        canonical_encoding: CANONICAL_ENCODING_V1.into(),
        mesh: "example.internal".into(),
        subnet: "192.0.2.0/24".into(),
        epoch,
        signed_at: "2026-11-18T00:00:00Z".into(),
        valid_until: "2027-11-18T00:00:00Z".into(),
        hub: vec!["alpha".into()],
        verify_keys: vec![VerifyKey {
            key_id: GENESIS_KEY_ID.into(),
            pubkey: b64.encode(root.verifying_key().to_bytes()),
            key_type: ALG_ED25519.into(),
            status: KeyStatus::Active,
        }],
        members,
        recovery: None,
        recovery_generation: None,
    };
    let sig = root.sign(&payload.canonical_bytes());
    let signed = SignedInventory {
        payload,
        signatures: vec![InvSignature {
            key_id: GENESIS_KEY_ID.into(),
            alg: ALG_ED25519.into(),
            sig: b64.encode(sig.to_bytes()),
        }],
    };
    let dir = std::env::temp_dir().join(format!("bus-tls-fixture-{:032x}", rand::random::<u128>()));
    std::fs::create_dir(&dir).unwrap();
    let paths = crate::authority::AuthorityPaths {
        genesis_pub: dir.join("genesis.pub"),
        signed: dir.join("inventory.signed"),
        baseline: dir.join("inventory.baseline"),
    };
    std::fs::write(
        &paths.genesis_pub,
        b64.encode(root.verifying_key().to_bytes()),
    )
    .unwrap();
    std::fs::write(&paths.signed, serde_json::to_vec(&signed).unwrap()).unwrap();
    let posture = crate::authority::load_and_verify(&paths);
    std::fs::remove_dir_all(dir).unwrap();
    assert!(
        matches!(posture, crate::authority::Posture::Verified(_)),
        "{posture:?}"
    );
    posture
}

fn lab_material(state: &AppState, seed: u8) -> mesh::tls::Material {
    let authority = state.authority.clone();
    mesh::tls::Material::new(
        [seed; 32],
        Arc::new(move |key| {
            let a = authority.load();
            match &a.posture {
                crate::authority::Posture::Verified(a) => super::tls_member(a, key),
                _ => None,
            }
        }),
    )
    .unwrap()
}

async fn lab_server(
    state: AppState,
    material: Option<mesh::tls::Material>,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let tcp = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = tcp.local_addr().unwrap();
    let task = tokio::spawn(async move {
        if let Some(m) = material {
            let app = axum::Router::new()
                .route("/ws", axum::routing::get(super::tls_ws_handler))
                .with_state(state);
            axum::serve(
                crate::tls_ingress::TlsListener::new(tcp, m).unwrap(),
                app.into_make_service_with_connect_info::<crate::tls_ingress::TlsPeer>(),
            )
            .await
            .unwrap();
        } else {
            let app = axum::Router::new()
                .route("/ws", axum::routing::get(super::ws_handler))
                .with_state(state);
            axum::serve(
                tcp,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap();
        }
    });
    (addr, task)
}

async fn lab_connect(
    addr: std::net::SocketAddr,
    m: Option<&mesh::tls::Material>,
    expected: &str,
) -> anyhow::Result<LabWs> {
    let url = format!("{}://{addr}/ws", if m.is_some() { "wss" } else { "ws" });
    let connector = m
        .map(|m| {
            m.client(expected.into())
                .map(tokio_tungstenite::Connector::Rustls)
        })
        .transpose()?;
    Ok(
        tokio_tungstenite::connect_async_tls_with_config(url, None, true, connector)
            .await?
            .0,
    )
}
async fn lab_recv(ws: &mut LabWs) -> Option<bus::BusMessage> {
    use futures_util::StreamExt;
    match tokio::time::timeout(Duration::from_secs(3), ws.next()).await {
        Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(t)))) => {
            Some(bus::parse(&t).unwrap())
        }
        _ => None,
    }
}
async fn lab_send(ws: &mut LabWs, msg: bus::BusMessage) {
    use futures_util::SinkExt;
    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        msg.to_wire().into(),
    ))
    .await
    .unwrap();
}
async fn lab_register(ws: &mut LabWs, name: &str) {
    lab_send(
        ws,
        bus::BusMessage::new()
            .with_header("to", "noded")
            .with_header("from", name)
            .with_header("command", "noded.register")
            .with_header("id", "register"),
    )
    .await;
    assert_eq!(lab_recv(ws).await.unwrap().get("rc"), Some("0"));
}
async fn lab_echo(state: AppState) -> tokio::task::JoinHandle<()> {
    let (addr, server) = lab_server(state, None).await;
    let mut ws = lab_connect(addr, None, "").await.unwrap();
    lab_register(&mut ws, "echo").await;
    tokio::spawn(async move {
        while let Some(msg) = lab_recv(&mut ws).await {
            let mut response = bus::BusMessage::new()
                .with_header("type", "response")
                .with_header("to", msg.from_addr().unwrap_or(""))
                .with_header("id", msg.get("id").unwrap_or(""))
                .with_header("rc", "0");
            response.body = msg.body;
            lab_send(&mut ws, response).await;
        }
        server.abort();
    })
}
fn lab_call(node: &str, size: usize) -> bus::BusMessage {
    let mut msg = bus::BusMessage::new()
        .with_header("to", &format!("echo.{node}.bus"))
        .with_header("from", "caller")
        .with_header("command", "echo.secret-method")
        .with_header("type", "request")
        .with_header("id", "call");
    msg.body = "secret-payload".repeat(size.div_ceil(14));
    msg
}

async fn lab_tap(
    target: std::net::SocketAddr,
) -> (
    std::net::SocketAddr,
    Arc<StdMutex<Vec<u8>>>,
    tokio::task::JoinHandle<()>,
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let bytes = Arc::new(StdMutex::new(Vec::new()));
    let captured = bytes.clone();
    let task = tokio::spawn(async move {
        let (a, _) = listener.accept().await.unwrap();
        let b = tokio::net::TcpStream::connect(target).await.unwrap();
        a.set_nodelay(true).unwrap();
        b.set_nodelay(true).unwrap();
        let (mut ar, mut aw) = a.into_split();
        let (mut br, mut bw) = b.into_split();
        let capture = captured.clone();
        let forward = async move {
            let mut buf = [0; 16384];
            loop {
                let n = ar.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    break;
                }
                capture.lock().unwrap().extend_from_slice(&buf[..n]);
                if bw.write_all(&buf[..n]).await.is_err() {
                    break;
                }
            }
        };
        let backward = async move {
            let mut buf = [0; 16384];
            loop {
                let n = br.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    break;
                }
                captured.lock().unwrap().extend_from_slice(&buf[..n]);
                if aw.write_all(&buf[..n]).await.is_err() {
                    break;
                }
            }
        };
        tokio::join!(forward, backward);
    });
    (addr, bytes, task)
}

#[tokio::test]
async fn mtls_lab_security() {
    let a = reload_test_state(lab_inventory(1, false), vec![]).await;
    let mut b = reload_test_state(lab_inventory(1, false), vec![]).await;
    b.node_name = "beta".into();
    let ma = lab_material(&a, 11);
    let mb = lab_material(&b, 22);
    let (aa, sa) = lab_server(a.clone(), Some(ma.clone())).await;
    let (ab, sb) = lab_server(b.clone(), Some(mb.clone())).await;
    let ea = lab_echo(a.clone()).await;
    let eb = lab_echo(b.clone()).await;
    let mut ba = lab_connect(aa, Some(&mb), "alpha").await.unwrap();
    let mut ab_ws = lab_connect(ab, Some(&ma), "beta").await.unwrap();
    lab_register(&mut ba, "bridge-beta").await;
    lab_register(&mut ab_ws, "bridge-alpha").await;
    for (ws, node) in [(&mut ba, "alpha"), (&mut ab_ws, "beta")] {
        let msg = lab_call(node, 16);
        let body = msg.body.clone();
        lab_send(ws, msg).await;
        assert_eq!(lab_recv(ws).await.unwrap().body, body);
    }
    let hostile = lab_material(&a, 33);
    assert!(lab_connect(aa, Some(&hostile), "alpha").await.is_err());
    assert!(
        lab_connect(aa, None, "").await.is_err(),
        "plaintext refused"
    );
    // Empty client certificate chain, but authenticate the server normally.
    let mut no_cert = (*mb.client("alpha".into()).unwrap()).clone();
    no_cert.client_auth_cert_resolver = Arc::new(rustls::client::FailResolveClientCert {});
    let no_cert = tokio_tungstenite::connect_async_tls_with_config(
        format!("wss://{aa}/ws"),
        None,
        true,
        Some(tokio_tungstenite::Connector::Rustls(Arc::new(no_cert))),
    )
    .await;
    assert!(no_cert.is_err(), "no certificate refused");
    // Copied public certificate + arbitrary recorded D2 proof cannot supply
    // CertificateVerify. rustls rejects the mismatched private key at config.
    let mut copied = hostile.clone();
    copied.cert = mb.cert.clone();
    assert!(lab_connect(aa, Some(&copied), "alpha").await.is_err());
    let mut unregistered = lab_connect(aa, Some(&mb), "alpha").await.unwrap();
    lab_send(&mut unregistered, lab_call("alpha", 16)).await;
    assert!(lab_recv(&mut unregistered).await.is_none());
    let mut relay = lab_connect(aa, Some(&mb), "alpha").await.unwrap();
    lab_register(&mut relay, "bridge-beta").await;
    lab_send(&mut relay, lab_call("beta", 16)).await;
    assert!(lab_recv(&mut relay).await.is_none());
    let (tap, bytes, tap_task) = lab_tap(aa).await;
    let mut sniffed = lab_connect(tap, Some(&mb), "alpha").await.unwrap();
    lab_register(&mut sniffed, "bridge-beta").await;
    lab_send(&mut sniffed, lab_call("alpha", 128)).await;
    assert!(lab_recv(&mut sniffed).await.is_some());
    {
        let bytes = bytes.lock().unwrap();
        assert!(bytes.len() > 128);
        for secret in [
            b"echo.secret-method".as_slice(),
            b"secret-payload".as_slice(),
        ] {
            assert!(
                !bytes.windows(secret.len()).any(|w| w == secret),
                "plaintext on TLS socket"
            );
        }
    }
    tap_task.abort();
    let start = Instant::now();
    let crate::authority::Posture::Verified(revoked) = lab_inventory(2, true) else {
        unreachable!()
    };
    let next = crate::routing::RoutingAuthority::new(
        crate::authority::Posture::Verified(revoked.clone()),
        "alpha",
        "127.0.0.1",
        4200,
        &[],
    );
    a.authority.store(Arc::new(next));
    super::reload_revocation_teardown(&a, &revoked).await;
    assert!(lab_recv(&mut ba).await.is_none());
    println!("MTLS_RELOAD_DROP_US={}", start.elapsed().as_micros());
    assert!(
        lab_connect(aa, Some(&mb), "alpha").await.is_err(),
        "tombstone handshake refused"
    );
    for task in [sa, sb, ea, eb] {
        task.abort();
    }
}

#[tokio::test]
#[ignore = "release CBC measurements"]
async fn mtls_lab_measure() {
    for tls in [false, true] {
        let state = reload_test_state(lab_inventory(1, false), vec![]).await;
        let server = lab_material(&state, 11);
        let client = lab_material(&state, 22);
        let (addr, task) = lab_server(state.clone(), tls.then_some(server)).await;
        let echo = lab_echo(state).await;
        let start = Instant::now();
        let mut ws = lab_connect(addr, tls.then_some(&client), "alpha")
            .await
            .unwrap();
        let handshake = start.elapsed().as_micros();
        lab_register(&mut ws, if tls { "bridge-beta" } else { "caller" }).await;
        let mut times = Vec::new();
        for _ in 0..1000 {
            let start = Instant::now();
            lab_send(&mut ws, lab_call("alpha", 16)).await;
            assert!(lab_recv(&mut ws).await.is_some());
            times.push(start.elapsed().as_micros());
        }
        times.sort_unstable();
        let start = Instant::now();
        for _ in 0..16 {
            lab_send(&mut ws, lab_call("alpha", 1024 * 1024)).await;
            assert!(lab_recv(&mut ws).await.is_some());
        }
        let mib_s = 32.0 / start.elapsed().as_secs_f64();
        let cpu_start = lab_cpu();
        let start = Instant::now();
        for _ in 0..10000 {
            lab_send(&mut ws, lab_call("alpha", 16)).await;
            assert!(lab_recv(&mut ws).await.is_some());
        }
        println!(
            "LAB_MEASURE tls={tls} handshake_us={handshake} p50_us={} p99_us={} duplex_mib_s={mib_s:.2} cpu_ms={:.2} wall_ms={:.2}",
            times[500],
            times[990],
            (lab_cpu() - cpu_start) * 1000.0,
            start.elapsed().as_secs_f64() * 1000.0
        );
        task.abort();
        echo.abort();
    }
}
fn lab_cpu() -> f64 {
    let mut time = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: clock_gettime writes the live timespec on success.
    assert_eq!(
        unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, time.as_mut_ptr()) },
        0
    );
    let time = unsafe { time.assume_init() };
    time.tv_sec as f64 + time.tv_nsec as f64 / 1e9
}
