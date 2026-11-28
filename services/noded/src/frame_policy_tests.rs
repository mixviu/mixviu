// SPDX-License-Identifier: MIT OR Apache-2.0

#[tokio::test]
async fn registration_rechecks_exact_key_after_rotation_before_dispatch() {
    let state = mesh_delivery_test_state().await;
    let mut registration = state.session_admission.clone();
    assert_eq!(registration.admitted_node.as_deref(), Some("beta"));
    let mut posture = state.authority.load().posture.clone();
    if let crate::authority::Posture::Verified(a) = &mut posture {
        a.epoch += 1;
        a.members_full.get_mut("beta").unwrap()["credentials"][0]["until_epoch"] = serde_json::json!(a.epoch);
    }
    apply_inventory_reload(&state, posture).await;
    registration.admitted_node =
        super::admitted_delivery_peer(&state, "192.0.2.2".parse().unwrap());
    assert!(matches!(
        super::register_gate_decision(
            false,
            "192.0.2.1:4200",
            AdmissionMode::Enforce,
            true,
            true,
            "bridge-beta",
            &registration
        ),
        super::RegisterGate::Refuse { .. }
    ));
}

#[tokio::test]
async fn inventory_rotation_during_inbound_proof_refuses_stale_handshake() {
    let state = mesh_delivery_test_state().await;
    let authority = state.authority.load_full();
    let crate::authority::Posture::Verified(a) = &authority.posture else {
        unreachable!()
    };
    let issued = state
        .challenge_table
        .issue(a.epoch, &a.mesh, &state.node_name)
        .await
        .unwrap();
    let challenge = BusMessage::new()
        .with_header("id", &issued.id)
        .with_body(&issued.body);
    let wire = mesh::build_admit_response(&challenge, "beta", &Some(mesh::seed::D2Seed::new([7; 32])), &state.node_name, &a.mesh, a.epoch).unwrap();
    let mut posture = authority.posture.clone();
    if let crate::authority::Posture::Verified(a) = &mut posture {
        a.epoch += 1;
    }
    apply_inventory_reload(&state, posture).await;
    let result = super::process_admit_response(
        &state,
        &bus_mod::parse(&wire).unwrap(),
        "in-flight",
        "192.0.2.2",
        Some(&issued.id),
    )
    .await;
    assert!(matches!(
        result,
        Some(super::AdmitOutcome::Refuse("epoch-mismatch"))
    ));
    assert!(state.challenge_table.take(&issued.id).await.is_none());
}

#[tokio::test]
async fn accepted_reload_cancels_dialled_proof_before_registration() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (state, _incoming, peer) = policy_mesh_state(listener.local_addr().unwrap()).await;
    let (arrived, waiting) = oneshot::channel();
    let (release, released) = oneshot::channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
        socket
            .send(WsMessage::Ping(Vec::new().into()))
            .await
            .unwrap();
        let challenge = next_mesh_frame(&mut socket).await;
        assert_eq!(challenge.command_name(), Some("noded.identity.challenge"));
        arrived.send(()).unwrap();
        released.await.unwrap();
        // Dropping this half also proves the cancelled attempt does not need a reply.
    });
    let mesh = state.mesh.clone();
    let call = tokio::spawn(async move { mesh.send(peer, BusMessage::empty()).await });
    waiting.await.unwrap();
    let mut posture = state.authority.load().posture.clone();
    if let crate::authority::Posture::Verified(a) = &mut posture {
        a.epoch += 1;
    }
    let started = Instant::now();
    apply_inventory_reload(&state, posture).await;
    assert!(
        tokio::time::timeout(Duration::from_secs(1), call)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    assert!(started.elapsed() < Duration::from_secs(1));
    release.send(()).unwrap();
    server.await.unwrap();
}

#[tokio::test]
async fn accepted_rotation_drops_exact_inbound_key_within_one_second() {
    let state = mesh_delivery_test_state().await;
    let (tx, _rx) = mpsc::channel(8);
    let close = Arc::new(tokio::sync::Notify::new());
    state.live_sessions.write().await.insert(
        "rotating".into(),
        super::GatedSession {
            transport: None,
            identity: state.peer_context.identity.clone(),
            claimed_source_node: "beta".into(),
            source_ip: "192.0.2.2".into(),
            tx,
            close: close.clone(),
        },
    );
    let mut posture = state.authority.load().posture.clone();
    if let crate::authority::Posture::Verified(a) = &mut posture {
        use base64::Engine as _;
        a.epoch += 1;
        let credentials = a.members_full.get_mut("beta").unwrap()["credentials"].as_array_mut().unwrap();
        credentials[0]["until_epoch"] = serde_json::json!(a.epoch);
        credentials.push(serde_json::json!({"kind":"d2", "pubkey":base64::engine::general_purpose::STANDARD.encode([4;32]),"from_epoch":a.epoch,"until_epoch":null}));
    }
    let start = Instant::now();
    apply_inventory_reload(&state, posture).await;
    tokio::time::timeout(Duration::from_secs(1), close.notified())
        .await
        .unwrap();
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(
        super::admitted_delivery_peer(&state, "192.0.2.2".parse().unwrap()).is_none(),
        "old session cannot reuse its admission after rotation"
    );
}

#[test]
fn frame_posture_defaults_observe_and_is_independent_of_admission() {
    let node: config::node::NodeConfig = config::from_conf_mix_str(
        "node: \"alpha\"\nnoded: { admission: \"enforce\", frame_policy: \"enforce\" }\n",
    )
    .unwrap();
    assert_eq!(
        super::frame_policy_config(&node.noded, None).unwrap(),
        AdmissionMode::Enforce
    );
    let mut config = config::node::NodedConfig {
        admission: AdmissionMode::Enforce,
        ..Default::default()
    };
    assert_eq!(
        super::frame_policy_config(&config, None).unwrap(),
        AdmissionMode::Observe
    );
    config
        .extra
        .insert("frame_policy".into(), serde_json::json!("enforce"));
    assert_eq!(
        super::frame_policy_config(&config, None).unwrap(),
        AdmissionMode::Enforce
    );
    assert_eq!(
        super::frame_policy_config(&config, Some("off")).unwrap(),
        AdmissionMode::Off
    );
    assert_eq!(
        super::frame_policy_config(&config, Some("observe")).unwrap(),
        AdmissionMode::Observe
    );
    assert!(super::frame_policy_config(&config, Some("invalid")).is_err());
    config
        .extra
        .insert("frame_policy".into(), serde_json::json!(true));
    assert!(super::frame_policy_config(&config, None).is_err());
}

async fn next_mesh_frame(
    socket: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) -> BusMessage {
    loop {
        if let WsMessage::Text(text) = socket
            .next()
            .await
            .expect("peer stayed connected")
            .expect("valid frame")
        {
            return bus_mod::parse(&text).unwrap();
        }
    }
}

async fn prove_policy_peer(socket: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
    let challenge = next_mesh_frame(socket).await;
    assert_eq!(challenge.command_name(), Some("noded.identity.challenge"));
    let wire = mesh::build_admit_response(&challenge, "beta", &Some(mesh::seed::D2Seed::new([8; 32])), "alpha", "bus", 1).unwrap();
    let mut response = bus_mod::parse(&wire).unwrap();
    response.set("command", "noded.identity.response");
    socket
        .send(WsMessage::Text(response.to_wire().into()))
        .await
        .unwrap();
}

async fn policy_mesh_state(
    endpoint: std::net::SocketAddr,
) -> (AppState, mpsc::UnboundedReceiver<MeshInbound>, PeerConfig) {
    let peer = PeerConfig {
    tls_port: None,
        name: "beta".into(),
        mesh_ip: endpoint.ip(),
        noded_port: endpoint.port(),
    };
    let mut state = reload_test_state(
        reload_posture(
            1,
            vec![
                active_route("alpha", "127.0.0.1".parse().unwrap(), DEFAULT_NODED_PORT),
                active_route("beta", endpoint.ip(), endpoint.port()),
            ],
        ),
        vec![],
    )
    .await;
    let (tx, incoming) = mpsc::unbounded_channel();
    state.mesh = Arc::new(MeshPeers::new(
        MeshConfig {
            node_name: "alpha".into(),
            peers: vec![peer.clone()],
            d2_seed: None,
        },
        tx,
    ));
    let current = state.authority.load_full();
    let mut posture = current.posture.clone();
    if let crate::authority::Posture::Verified(a) = &mut posture {
        use base64::Engine as _;
        a.members_full.insert("beta".into(), serde_json::json!({
            "status":"active", "bus":true, "credentials":[{"kind":"d2",
            "pubkey":base64::engine::general_purpose::STANDARD.encode(ed25519_dalek::SigningKey::from_bytes(&[8;32]).verifying_key().to_bytes()),
            "from_epoch":1, "until_epoch":null}]
        }));
    }
    state
        .authority
        .store(Arc::new(crate::routing::RoutingAuthority::new(
            posture,
            "alpha",
            "127.0.0.1",
            4200,
            &[],
        )));
    let authority = state.authority.load_full();
    state
        .mesh
        .reconcile_identity(
            authority.routes.desired_endpoints(),
            authority.revision(),
            authority.keys.clone(),
            match &authority.posture {
                crate::authority::Posture::Verified(a) => a.mesh.clone(),
                _ => unreachable!(),
            },
        )
        .await;
    (state, incoming, peer)
}

#[tokio::test]
async fn serve_mesh_receiver_policy_delivers_observe_and_drops_enforce() {
    for mode in [
        AdmissionMode::Off,
        AdmissionMode::Observe,
        AdmissionMode::Enforce,
    ] {
        for (from, to) in [
            ("hostd.beta.bus", "recipient.gamma.bus"),
            ("hostd", "recipient.alpha.bus"),
            ("absent", "recipient.alpha.bus"),
            ("hostd.gamma.bus", "recipient.alpha.bus"),
            ("", ""),
        ] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let (state, incoming, peer) = policy_mesh_state(listener.local_addr().unwrap()).await;
            let (target, mut received) = mpsc::channel(8);
            state.registry.write().await.insert(
                "recipient".into(),
                super::ServiceEntry {
                    origin: super::Origin::Client,
                    commands: Default::default(),
                    tx: target,
                    info: Default::default(),
                    traffic_class: Default::default(),
                    protected_responses: Default::default(),
                },
            );
            let (go, ready) = oneshot::channel();
            let (stop, stopped) = oneshot::channel();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
                socket
                    .send(WsMessage::Ping(Vec::new().into()))
                    .await
                    .unwrap();
                prove_policy_peer(&mut socket).await;
                assert_eq!(
                    next_mesh_frame(&mut socket).await.command_name(),
                    Some("noded.register")
                );
                ready.await.unwrap();
                let mut bad = if from.is_empty() {
                    BusMessage::empty()
                } else {
                    BusMessage::new()
                        .with_header("from", from)
                        .with_header("to", to)
                        .with_header("id", "bad")
                        .with_header("type", "event")
                };
                if from == "absent" {
                    bad.headers.remove("from");
                }
                let good = BusMessage::new()
                    .with_header("from", "hostd.beta.bus")
                    .with_header("to", "recipient.alpha.bus")
                    .with_header("id", "good")
                    .with_header("type", "event");
                for frame in [bad, good] {
                    socket
                        .send(WsMessage::Text(frame.to_wire().into()))
                        .await
                        .unwrap();
                }
                let _ = stopped.await;
            });
            let receiver = tokio::spawn(super::mesh_incoming_loop(
                incoming,
                state.mesh.clone(),
                state.authority.clone(),
                state.registry.clone(),
                state.observe.clone(),
                "alpha".into(),
                mode,
            ));
            state.mesh.send(peer, BusMessage::empty()).await.unwrap();
            go.send(()).unwrap();
            let first = tokio::time::timeout(Duration::from_secs(2), received.recv())
                .await
                .unwrap()
                .unwrap();
            let first = bus_mod::parse(&first).unwrap();
            if mode == AdmissionMode::Enforce || from.is_empty() {
                assert_eq!(first.get("id"), Some("good"));
            } else {
                assert_eq!(first.get("id"), Some("bad"));
                let next = tokio::time::timeout(Duration::from_secs(2), received.recv())
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(bus_mod::parse(&next).unwrap().get("id"), Some("good"));
            }
            assert!(received.try_recv().is_err());
            let _ = stop.send(());
            server.await.unwrap();
            receiver.abort();
        }
    }
}

#[tokio::test]
async fn solicited_mesh_responses_check_source_before_client_delivery() {
    for mode in [AdmissionMode::Observe, AdmissionMode::Enforce] {
        for from in ["noded", "hostd.gamma.bus", "hostd.beta.bus"] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let (mut state, _incoming, _peer) =
                policy_mesh_state(listener.local_addr().unwrap()).await;
            state.frame_policy = mode;
            let (stop, stopped) = oneshot::channel();
            let (answered, answer_sent) = oneshot::channel();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
                socket
                    .send(WsMessage::Ping(Vec::new().into()))
                    .await
                    .unwrap();
                prove_policy_peer(&mut socket).await;
                next_mesh_frame(&mut socket).await;
                let request = next_mesh_frame(&mut socket).await;
                assert_eq!(request.get("hop_limit"), Some("0"));
                let response = BusMessage::new()
                    .with_header("type", "response")
                    .with_header("from", from)
                    .with_header("id", request.get("id").unwrap())
                    .with_header("rc", "0");
                socket
                    .send(WsMessage::Text(response.to_wire().into()))
                    .await
                    .unwrap();
                let _ = answered.send(());
                let _ = stopped.await;
            });
            let root = std::env::temp_dir().join(format!(
                "bus-policy-{}-{}",
                std::process::id(),
                rand::random::<u64>()
            ));
            std::fs::create_dir(&root).unwrap();
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
            let path = root.join("bus.sock");
            let listener = tokio::net::UnixListener::bind(&path).unwrap();
            let app = axum::Router::new()
                .route("/ws", axum::routing::get(super::unix_ws_handler))
                .with_state(state);
            let local = tokio::spawn(async move {
                axum::serve(
                    listener,
                    app.into_make_service_with_connect_info::<crate::native_ingress::UnixPeer>(),
                )
                .await
                .unwrap();
            });
            let url = format!("unix://{}", path.display());
            let (mut sink, mut stream) = raw_connect(&url).await;
            let request = BusMessage::new()
                .with_header("to", "hostd.beta.bus")
                .with_header("command", "probe.echo")
                .with_header("id", "reply-probe")
                .with_header("request_id", "correlation-probe");
            if mode == AdmissionMode::Enforce && from != "hostd.beta.bus" {
                sink.send(WsMessage::Text(request.to_wire().into()))
                    .await
                    .unwrap();
                tokio::time::timeout(Duration::from_secs(3), answer_sent)
                    .await
                    .unwrap()
                    .unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(250), stream.next())
                        .await
                        .is_err()
                );
            } else {
                let reply = raw_call(&mut sink, &mut stream, request, "reply-probe").await;
                assert_eq!(reply.rc(), Some(0));
                assert_eq!(reply.get("request_id"), Some("correlation-probe"));
            }
            let _ = stop.send(());
            server.await.unwrap();
            local.abort();
            let _ = local.await;
            std::fs::remove_file(path).unwrap();
            std::fs::remove_dir(root).unwrap();
        }
    }
}
