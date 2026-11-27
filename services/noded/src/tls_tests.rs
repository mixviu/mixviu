// SPDX-License-Identifier: MIT OR Apache-2.0
use bus::native_session::TransportIdentity;

fn tls_credential(seed: u8) -> mesh::tls::Credential {
    use std::os::unix::fs::PermissionsExt;
    let path = std::env::temp_dir().join(format!("tls-seed-{:032x}", rand::random::<u128>()));
    std::fs::write(&path, [seed; 32]).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let credential = mesh::tls::Credential::new(mesh::tls::load_seed(&path).unwrap()).unwrap();
    std::fs::remove_file(path).unwrap();
    credential
}

async fn tls_brokers() -> (Vec<AppState>, Vec<PeerConfig>, Vec<tokio::task::JoinHandle<()>>) {
    use base64::Engine as _;
    let mut listeners = Vec::new();
    let mut peers = Vec::new();
    let credentials: Vec<_> = (1..=3).map(tls_credential).collect();
    for name in ["alpha", "beta", "gamma"] {
        let tcp = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = tcp.local_addr().unwrap();
        peers.push(PeerConfig { name: name.into(), mesh_ip: address.ip(), noded_port: address.port(), tls_port: Some(address.port()) });
        listeners.push(tcp);
    }
    let mut posture = reload_posture(1, peers.iter().map(|p| active_route(&p.name, p.mesh_ip, p.noded_port)).collect());
    if let crate::authority::Posture::Verified(a) = &mut posture {
        a.mesh = "example.test".into();
        for (i, peer) in peers.iter().enumerate() {
            let b64 = base64::engine::general_purpose::STANDARD;
            let d2 = ed25519_dalek::SigningKey::from_bytes(&[11 + i as u8; 32]);
            a.members_full.insert(peer.name.clone(), serde_json::json!({"name":peer.name,"status":"active","bus":true,
                "credentials":[{"kind":"d2","pubkey":b64.encode(d2.verifying_key().to_bytes()),"from_epoch":1},
                {"kind":"bus-tls","pubkey":b64.encode(mesh::tls::public_key(&credentials[i].certificate).unwrap()),"from_epoch":1}]}));
        }
    }
    let mut states = Vec::new();
    let mut tasks = Vec::new();
    for (i, (credential, tcp)) in credentials.into_iter().zip(listeners).enumerate() {
        let name = &peers[i].name;
        let mut state = reload_test_state(posture.clone(), peers.clone()).await;
        state.node_name = name.clone();
        state.d2_seed = Some(mesh::seed::D2Seed::new([11 + i as u8; 32]));
        state.frame_policy = AdmissionMode::Enforce;
        let authority = crate::routing::RoutingAuthority::new(posture.clone(), name, "127.0.0.1", peers[i].noded_port, &peers);
        let tls = Arc::new(mesh::TlsRuntime { credential,
            authority: Arc::new(std::sync::RwLock::new(authority.tls_keys.clone())) });
        let (incoming, _rx) = mpsc::unbounded_channel();
        state.mesh = Arc::new(MeshPeers::new(MeshConfig { node_name: name.clone(), peers: peers.clone(), d2_seed: state.d2_seed.clone() }, incoming));
        state.mesh.set_tls(tls.clone()).await;
        state.mesh.set_frame_enforce(true).await;
        super::reconcile_transport_authority(&state.mesh, &authority).await;
        state.authority.store(Arc::new(authority));
        state.tls = Some(tls.clone());
        let listener = crate::tls_ingress::TlsListener::new(tcp, &tls.credential, tls.authority.clone(),
            &Default::default(), state.tls_refusals.clone()).unwrap();
        let app = axum::Router::new().route("/ws", axum::routing::get(super::tls_ws_handler)).with_state(state.clone());
        tasks.push(tokio::spawn(async move {
            axum::serve(listener, app.into_make_service_with_connect_info::<crate::tls_ingress::TlsPeer>()).await.unwrap();
        }));
        states.push(state);
    }
    (states, peers, tasks)
}

async fn tls_ping(state: &AppState, peer: &PeerConfig) {
    let reply = tokio::time::timeout(Duration::from_secs(3), state.mesh.call(peer.clone(),
        BusMessage::new().with_header("command", "noded.ping").with_header("to", "noded")
            .with_header("from", &format!("probe.{}.bus", state.node_name)))).await.unwrap().unwrap();
    assert_eq!(reply.rc(), Some(0));
}

#[tokio::test]
async fn tls_brokers_prove_both_directions_and_revoke_either_exact_key_only() {
    for revoked in [None, Some("d2"), Some("bus-tls")] {
        let (states, peers, tasks) = tls_brokers().await;
        for other in 1..3 {
            tls_ping(&states[0], &peers[other]).await;
            tls_ping(&states[other], &peers[0]).await;
            let context = states[0].mesh.peer_context(&peers[other].name).await.unwrap();
            let Some(TransportIdentity::MeshTls { member, tls_key, d2_key, .. }) = &context.transport else { panic!("typed TLS context") };
            assert_eq!(member, &peers[other].name);
            assert_ne!(tls_key, d2_key);
            assert_eq!(context.mesh_peer().unwrap(), (member.as_str(), d2_key));
        }
        assert_eq!(states[0].live_sessions.read().await.len(), 2);
        let mut next = states[0].authority.load().posture.clone();
        if let crate::authority::Posture::Verified(a) = &mut next {
            a.epoch += 1;
            if let Some(kind) = revoked {
                for key in a.members_full.get_mut("beta").unwrap()["credentials"].as_array_mut().unwrap() {
                    if key["kind"] == kind { key["until_epoch"] = serde_json::json!(a.epoch); }
                }
            }
        }
        super::apply_inventory_reload(&states[0], next).await;
        assert_eq!(states[0].mesh.peer_context("beta").await.is_some(), revoked.is_none());
        assert!(states[0].mesh.peer_context("gamma").await.is_some());
        let live = states[0].live_sessions.read().await;
        assert_eq!(live.len(), if revoked.is_some() { 1 } else { 2 });
        assert!(live.values().all(|s| matches!(s.transport, Some(TransportIdentity::MeshTls { .. }))));
        let snapshot = states[0].authority.load_full();
        for s in live.values() {
            if let Some(TransportIdentity::MeshTls { tls_key, .. }) = &s.transport {
                let member = states[0].tls.as_ref().unwrap().authority.read().unwrap().member_for(tls_key).unwrap();
                assert_eq!(member.epoch, snapshot.keys.epoch());
                assert_eq!(member.revision, snapshot.tls_keys.member_for(tls_key).unwrap().revision);
            }
        }
        drop(live);
        tls_ping(&states[0], &peers[2]).await;
        for task in tasks { task.abort(); }
    }
}

#[tokio::test]
async fn tls_pin_failure_never_touches_the_legacy_port() {
    let (states, peers, tasks) = tls_brokers().await;
    let legacy = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut wrong = peers[1].clone();
    wrong.name = "gamma".into();
    wrong.noded_port = legacy.local_addr().unwrap().port();
    let snapshot = states[0].authority.load_full();
    states[0].mesh.reconcile_security(
        std::collections::HashMap::from([("gamma".into(), wrong.noded_addr())]), snapshot.revision(),
        snapshot.keys.clone(), "example.test".into(), snapshot.tls_keys.clone(),
    ).await;
    let before = mesh::tls::DIAL_REFUSALS.total();
    assert!(states[0].mesh.send(wrong, BusMessage::empty()).await.is_err());
    assert!(mesh::tls::DIAL_REFUSALS.total() > before);
    assert!(tokio::time::timeout(Duration::from_millis(100), legacy.accept()).await.is_err());
    for task in tasks { task.abort(); }
}

#[tokio::test]
async fn tls_startup_requires_inventory_seed_mode_and_current_own_key() {
    use std::os::unix::fs::PermissionsExt;
    let (states, _, tasks) = tls_brokers().await;
    let path = std::env::temp_dir().join(format!("tls-startup-{:032x}", rand::random::<u128>()));
    let settings = config::node::NodedTls { key: path.to_str().unwrap().into(), ..Default::default() };
    let posture = states[0].authority.load().posture.clone();
    assert!(crate::tls_ingress::prepare(&posture, "alpha", &settings).is_err());
    std::fs::write(&path, [1; 32]).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(crate::tls_ingress::prepare(&posture, "alpha", &settings).is_err());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(crate::tls_ingress::prepare(&posture, "alpha", &settings).is_ok());
    assert!(crate::tls_ingress::prepare(&posture, "beta", &settings).is_err());
    assert!(crate::tls_ingress::prepare(&crate::authority::Posture::Unverified { reason: "fixture".into() }, "alpha", &settings).is_err());
    std::fs::remove_file(path).unwrap();
    for task in tasks { task.abort(); }
}

#[tokio::test]
async fn tls_d2_binding_refuses_zero_relay_and_another_member() {
    let (states, peers, tasks) = tls_brokers().await;
    let mut verifier = states[0].clone();
    let member = verifier.tls.as_ref().unwrap().authority.read().unwrap()
        .member_for(&mesh::tls::public_key(&states[1].tls.as_ref().unwrap().credential.certificate).unwrap()).unwrap();
    verifier.tls_peer = Some(crate::tls_ingress::TlsPeer { address: peers[1].noded_addr(), member, binding: [7; 32] });
    for (binding, node, seed, accepted) in [([0; 32], "beta", 12, false), ([8; 32], "beta", 12, false),
        ([7; 32], "gamma", 13, false), ([7; 32], "beta", 12, true)] {
        let issued = verifier.challenge_table.issue(1, "example.test", "alpha").await.unwrap();
        let challenge = BusMessage::new().with_header("id", &issued.id).with_body(&issued.body);
        let wire = mesh::build_bound_admit_response(&challenge, node,
            &Some(mesh::seed::D2Seed::new([seed; 32])), "alpha", "example.test", 1, &binding).unwrap();
        let outcome = super::process_admit_response(&verifier, &bus_mod::parse(&wire).unwrap(), "tls", "127.0.0.1", Some(&issued.id)).await;
        assert_eq!(matches!(outcome, Some(super::AdmitOutcome::Admit(_, _))), accepted);
        let snapshot = verifier.authority.load_full();
        let response: serde_json::Value = serde_json::from_str(&bus_mod::parse(&wire).unwrap().body).unwrap();
        let challenge: serde_json::Value = serde_json::from_str(&issued.body).unwrap();
        assert_eq!(snapshot.keys.authenticate_bound_response(&challenge, &response, "beta", Some(&[7; 32])).is_some(), accepted);
    }
    for task in tasks { task.abort(); }
}

#[tokio::test]
async fn tls_handshake_timeout_queue_and_refusals() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let (states, peers, tasks) = tls_brokers().await;
    let tcp = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = tcp.local_addr().unwrap();
    let runtime = states[0].tls.as_ref().unwrap();
    let refusals = Arc::new(mesh::tls::Refusals::default());
    let listener = crate::tls_ingress::TlsListener::new(tcp, &runtime.credential, runtime.authority.clone(),
        &config::node::NodedTls { handshake_timeout_s: 1, max_handshakes: 1, max_queue: 1, ..Default::default() }, refusals.clone()).unwrap();
    let app = axum::Router::new().route("/ws", axum::routing::get(super::tls_ws_handler)).with_state(states[0].clone());
    let driver = tokio::spawn(async move { axum::serve(listener,
        app.into_make_service_with_connect_info::<crate::tls_ingress::TlsPeer>()).await.unwrap(); });
    let mut first = tokio::net::TcpStream::connect(address).await.unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;
    let mut second = tokio::net::TcpStream::connect(address).await.unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;
    let mut excess = tokio::net::TcpStream::connect(address).await.unwrap();
    assert!(matches!(tokio::time::timeout(Duration::from_millis(250), excess.read(&mut [0; 1])).await, Ok(Ok(0)) | Ok(Err(_))));
    assert!(tokio::time::timeout(Duration::from_millis(100), second.read(&mut [0; 1])).await.is_err());
    let started = Instant::now();
    let _ = tokio::time::timeout(Duration::from_secs(2), first.read(&mut [0; 1])).await.unwrap();
    assert!(started.elapsed() >= Duration::from_millis(500));
    let _ = tokio::time::timeout(Duration::from_secs(2), second.read(&mut [0; 1])).await.unwrap();
    assert!(started.elapsed() >= Duration::from_millis(1500));
    assert_eq!(refusals.total(), 3);
    let mut plaintext = tokio::net::TcpStream::connect(address).await.unwrap();
    plaintext.write_all(b"GET /ws HTTP/1.1\r\n\r\n").await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(2), plaintext.read(&mut [0; 32])).await.unwrap();
    assert_eq!(refusals.total(), 4);
    for seed in [99, 2] {
        let credential = tls_credential(seed);
        let mut keys = states[0].authority.load().posture.clone();
        if seed == 2 {
            if let crate::authority::Posture::Verified(a) = &mut keys { a.members_full.remove("beta"); }
            let keys = crate::routing::RoutingAuthority::new(keys, "alpha", "127.0.0.1", 4200, &peers);
            *runtime.authority.write().unwrap() = keys.tls_keys;
        }
        let config = credential.client_config(runtime.authority.clone(), "alpha".into()).unwrap();
        let stream = tokio::net::TcpStream::connect(address).await.unwrap();
        let result = tokio_rustls::TlsConnector::from(config).connect(
            tokio_rustls::rustls::pki_types::ServerName::try_from("bus.example.test").unwrap(), stream).await;
        if let Ok(mut stream) = result { let _ = stream.read(&mut [0; 1]).await; }
    }
    tokio::time::timeout(Duration::from_secs(2), async { while refusals.total() < 6 { tokio::task::yield_now().await; } }).await.unwrap();
    assert_eq!(refusals.total(), 6);
    driver.abort();
    for task in tasks { task.abort(); }
}
