// SPDX-License-Identifier: MIT OR Apache-2.0

#[tokio::test]
async fn failed_reverse_proofs_never_write_application_frames_under_enforce() {
    for mode in [
        AdmissionMode::Off,
        AdmissionMode::Observe,
        AdmissionMode::Enforce,
    ] {
        for failure in ["timeout", "invalid", "malformed"] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let (state, _incoming, peer) = policy_mesh_state(listener.local_addr().unwrap()).await;
            state
                .mesh
                .set_frame_enforce(mode == AdmissionMode::Enforce)
                .await;
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
                socket
                    .send(WsMessage::Ping(Vec::new().into()))
                    .await
                    .unwrap();
                let challenge = next_mesh_frame(&mut socket).await;
                assert_eq!(challenge.command_name(), Some("noded.identity.challenge"));
                if failure == "invalid" {
                    let bad = BusMessage::new()
                        .with_header("command", "noded.identity.response")
                        .with_header("id", challenge.get("id").unwrap())
                        .with_body("{}");
                    socket
                        .send(WsMessage::Text(bad.to_wire().into()))
                        .await
                        .unwrap();
                } else if failure == "malformed" {
                    socket
                        .send(WsMessage::Text("malformed".into()))
                        .await
                        .unwrap();
                }
                let mut frames = Vec::new();
                loop {
                    let wire =
                        match tokio::time::timeout(Duration::from_secs(2), socket.next()).await {
                            Ok(Some(Ok(WsMessage::Text(wire)))) => wire,
                            Ok(Some(Ok(WsMessage::Close(_))))
                            | Ok(None)
                            | Ok(Some(Err(_)))
                            | Err(_) => break,
                            _ => continue,
                        };
                    let frame = bus_mod::parse(&wire).unwrap();
                    let done = frame.command_name() == Some("probe.secret");
                    frames.push(frame);
                    if done {
                        break;
                    }
                }
                frames
            });
            let start = Instant::now();
            let sent = state
                .mesh
                .send(
                    peer,
                    BusMessage::new()
                        .with_header("command", "probe.secret")
                        .with_body("application payload"),
                )
                .await;
            let elapsed = start.elapsed();
            let frames = server.await.unwrap();
            if mode == AdmissionMode::Enforce {
                assert!(sent.is_err(), "{failure}");
                assert!(
                    frames.is_empty(),
                    "no registration or application write to an unproved peer"
                );
            } else {
                sent.unwrap();
                assert_eq!(frames.len(), 2);
                assert_eq!(frames[0].command_name(), Some("noded.register"));
                assert_eq!(frames[1].command_name(), Some("probe.secret"));
            }
            if failure == "timeout" {
                assert!(elapsed >= Duration::from_secs(1));
            }
            assert!(
                elapsed < Duration::from_secs(3),
                "bounded old-broker fallback: {elapsed:?}"
            );
        }
    }
}

#[tokio::test]
async fn reverse_answers_are_one_shot_and_bound_to_current_admitted_peer() {
    use base64::Engine as _;
    let mut state = mesh_delivery_test_state().await;
    state.d2_seed = Some(mesh::seed::D2Seed::new([8; 32]));
    let snapshot = state.authority.load_full();
    let crate::authority::Posture::Verified(a) = &snapshot.posture else {
        unreachable!()
    };
    let body = serde_json::json!({"mesh_fqdn":a.mesh,"inventory_epoch":a.epoch,
        "verifying_broker_node":"beta","session_id":base64::engine::general_purpose::STANDARD.encode([1;16]),
        "server_nonce":base64::engine::general_purpose::STANDARD.encode([2;32])});
    let message = BusMessage::new()
        .with_header("id", "reverse")
        .with_body(&body.to_string());
    for bad in [
        "verifier",
        "mesh",
        "unproven",
        "registered",
        "revoked",
    ] {
        let mut candidate = state.clone();
        let mut body = body.clone();
        match bad {
            "verifier" => body["verifying_broker_node"] = serde_json::json!("gamma"),
            "mesh" => body["mesh_fqdn"] = serde_json::json!("other.example"),
            "unproven" => candidate.peer_context.identity = Default::default(),
            "revoked" => apply_inventory_reload(&candidate, revoked_delivery_posture(&candidate)).await,
            _ => {}
        }
        let mut answered = false;
        assert!(
            super::reverse_identity_answer(
                &candidate,
                &message.clone().with_body(&body.to_string()),
                bad == "registered",
                &mut answered
            )
            .is_none(),
            "{bad}"
        );
        assert!(!answered);
    }
    // The revoked-case candidate shares authority; restore a fresh state.
    let mut state = mesh_delivery_test_state().await;
    state.d2_seed = Some(mesh::seed::D2Seed::new([8; 32]));
    // Sign the verifier-owned epoch, including either direction of skew.
    for epoch in [0, 1, 2] {
        let mut skewed = body.clone();
        skewed["inventory_epoch"] = serde_json::json!(epoch);
        let response = super::reverse_identity_answer(&state, &message.clone().with_body(&skewed.to_string()), false, &mut false).unwrap();
        let response = bus_mod::parse(&response).unwrap();
        assert_eq!(response.command_name(), Some("noded.identity.response"));
        let proof: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(proof["signed_epoch"], epoch);
    }
    let mut answered = false;
    assert!(super::reverse_identity_answer(&state, &message, false, &mut answered).is_some());
    assert!(super::reverse_identity_answer(&state, &message, false, &mut answered).is_none());
}

#[tokio::test]
async fn reverse_answer_waits_for_channel_capacity_instead_of_dropping() {
    let (tx, mut rx) = mpsc::channel(1);
    tx.send("occupied".into()).await.unwrap();
    let close = Arc::new(tokio::sync::Notify::new());
    let sender = tx.clone();
    let signal = close.clone();
    let task = tokio::spawn(async move {
        super::queue_identity_answer(&sender, "answer".into(), &signal).await
    });
    tokio::task::yield_now().await;
    assert!(!task.is_finished());
    assert_eq!(rx.recv().await.as_deref(), Some("occupied"));
    assert!(task.await.unwrap());
    assert_eq!(rx.recv().await.as_deref(), Some("answer"));
    tx.send("occupied".into()).await.unwrap();
    close.notify_one();
    assert!(!super::queue_identity_answer(&tx, "cancelled".into(), &close).await);
}

#[tokio::test]
async fn registration_insert_rechecks_after_waiting_for_live_session_lock() {
    let state = mesh_delivery_test_state().await;
    let live = state.live_sessions.write().await;
    let cloned = state.clone();
    let (tx, _rx) = mpsc::channel(1);
    let attempt = tokio::spawn(async move {
        super::track_mesh_session(
            &cloned,
            "racing",
            "192.0.2.2",
            &tx,
            &Arc::new(tokio::sync::Notify::new()),
        )
        .await
    });
    tokio::task::yield_now().await;
    assert!(!attempt.is_finished());
    // Publish the reload while the registration is waiting to insert.
    let next = crate::routing::RoutingAuthority::new(
        reload_posture(2, vec![]),
        "alpha",
        "127.0.0.1",
        4200,
        &[],
    );
    {
        let _fence = state.delivery_fence.write().unwrap();
        state.authority.store(Arc::new(next));
    }
    drop(live);
    assert!(!attempt.await.unwrap());
    assert!(state.live_sessions.read().await.is_empty());
}

#[tokio::test]
async fn mutual_proof_orders_listener_challenge_client_proof_then_reverse_answer() {
    use base64::Engine as _;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (state, _incoming, peer) = policy_mesh_state(listener.local_addr().unwrap()).await;
    let mut listener_state = state.clone();
    listener_state.node_name = "beta".into();
    listener_state.d2_seed = Some(mesh::seed::D2Seed::new([8; 32]));
    let mut posture = state.authority.load().posture.clone();
    if let crate::authority::Posture::Verified(a) = &mut posture {
        a.members_full.insert("alpha".into(),serde_json::json!({"name":"alpha","status":"active","bus":true,"credentials":[{
            "kind":"d2","pubkey":base64::engine::general_purpose::STANDARD.encode(ed25519_dalek::SigningKey::from_bytes(&[7;32]).verifying_key().to_bytes()),"from_epoch":1,"until_epoch":null}]}));
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
    let mesh_name = match &authority.posture {
        crate::authority::Posture::Verified(a) => a.mesh.clone(),
        _ => unreachable!(),
    };
    let (incoming, _rx) = mpsc::unbounded_channel();
    let dialler = MeshPeers::new(
        MeshConfig {
            node_name: "alpha".into(),
            peers: vec![peer.clone()],
            d2_seed: Some(mesh::seed::D2Seed::new([7; 32])),
        },
        incoming,
    );
    dialler.set_frame_enforce(true).await;
    dialler
        .reconcile_identity(
            authority.routes.desired_endpoints(),
            authority.revision(),
            authority.keys.clone(),
            mesh_name.clone(),
        )
        .await;
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
        let issued = listener_state
            .challenge_table
            .issue(1, &mesh_name, "beta")
            .await
            .unwrap();
        socket
            .send(WsMessage::Text(
                BusMessage::new()
                    .with_header("command", "noded.admit.challenge")
                    .with_header("id", &issued.id)
                    .with_body(&issued.body)
                    .to_wire()
                    .into(),
            ))
            .await
            .unwrap();
        let proof = next_mesh_frame(&mut socket).await;
        assert_eq!(proof.command_name(), Some("noded.admit.response"));
        let outcome = super::process_admit_response(
            &listener_state,
            &proof,
            "ordered",
            "192.0.2.1",
            Some(&issued.id),
        )
        .await;
        let Some(super::AdmitOutcome::Admit(node, identity)) = outcome else {
            panic!("client proof must precede reverse challenge");
        };
        assert_eq!(node, "alpha");
        listener_state.peer_context.identity = identity;
        let reverse = next_mesh_frame(&mut socket).await;
        assert_eq!(reverse.command_name(), Some("noded.identity.challenge"));
        let answer =
            super::reverse_identity_answer(&listener_state, &reverse, false, &mut false).unwrap();
        socket.send(WsMessage::Text(answer.into())).await.unwrap();
        assert_eq!(
            next_mesh_frame(&mut socket).await.command_name(),
            Some("noded.register")
        );
        assert_eq!(
            next_mesh_frame(&mut socket).await.command_name(),
            Some("probe.ordered")
        );
    });
    dialler
        .send(
            peer,
            BusMessage::new().with_header("command", "probe.ordered"),
        )
        .await
        .unwrap();
    server.await.unwrap();
}

#[tokio::test]
async fn unverified_authority_skips_reverse_challenge() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let peer = PeerConfig {
    tls_port: None,
        name: "beta".into(),
        mesh_ip: listener.local_addr().unwrap().ip(),
        noded_port: listener.local_addr().unwrap().port(),
    };
    let state = reload_test_state(
        crate::authority::Posture::Unverified {
            reason: "test".into(),
        },
        vec![peer.clone()],
    )
    .await;
    super::reconcile_transport_authority(&state.mesh, &state.authority.load_full()).await;
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
        socket
            .send(WsMessage::Ping(Vec::new().into()))
            .await
            .unwrap();
        assert_eq!(
            next_mesh_frame(&mut socket).await.command_name(),
            Some("noded.register")
        );
        assert_eq!(
            next_mesh_frame(&mut socket).await.command_name(),
            Some("probe.unverified")
        );
    });
    state
        .mesh
        .send(
            peer,
            BusMessage::new().with_header("command", "probe.unverified"),
        )
        .await
        .unwrap();
    server.await.unwrap();
}

#[tokio::test]
async fn unauthenticated_dialled_frame_cannot_supply_connection_stamp() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (state, incoming, peer) = policy_mesh_state(listener.local_addr().unwrap()).await;
    let (target, mut received) = mpsc::channel(8);
    state.registry.write().await.insert(
        "recipient".into(),
        super::ServiceEntry {
commands: Default::default(),
            tx: target,
            info: Default::default(),
            traffic_class: Default::default(),
            protected_responses: Default::default(),
        },
    );
    let (stop, stopped) = oneshot::channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
        socket
            .send(WsMessage::Ping(Vec::new().into()))
            .await
            .unwrap();
        let challenge = next_mesh_frame(&mut socket).await;
        let bad = BusMessage::new()
            .with_header("command", "noded.identity.response")
            .with_header("id", challenge.get("id").unwrap())
            .with_body("{}");
        socket
            .send(WsMessage::Text(bad.to_wire().into()))
            .await
            .unwrap();
        next_mesh_frame(&mut socket).await; // registration
        let forged = BusMessage::new()
            .with_header("type", "event")
            .with_header("to", "recipient.alpha.bus")
            .with_header("from", "hostd.beta.bus")
            .with_header("broker_connection", "beta/forged")
            .with_header("broker_peer", "beta");
        socket
            .send(WsMessage::Text(forged.to_wire().into()))
            .await
            .unwrap();
        let _ = stopped.await;
    });
    let receiver = tokio::spawn(super::mesh_incoming_loop(
        incoming,
        state.mesh.clone(),
        state.authority.clone(),
        state.registry.clone(),
        state.observe.clone(),
        "alpha".into(),
        AdmissionMode::Observe,
    ));
    state.mesh.send(peer, BusMessage::empty()).await.unwrap();
    let wire = tokio::time::timeout(Duration::from_secs(2), received.recv())
        .await
        .unwrap()
        .unwrap();
    let delivered = bus_mod::parse(&wire).unwrap();
    assert!(delivered.get("broker_connection").is_none());
    assert!(delivered.get("broker_peer").is_none());
    stop.send(()).unwrap();
    server.await.unwrap();
    receiver.abort();
}

#[tokio::test]
async fn real_unix_registration_survives_reload_and_tcp_loopback_is_refused() {
    let mut state = mesh_delivery_test_state().await;
    state.frame_policy = AdmissionMode::Observe;
    state.wg_bound = true;
    let root = std::env::temp_dir().join(format!("identity-local-{:032x}", rand::random::<u128>()));
    std::fs::create_dir(&root).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = root.join("bus.sock");
    let unix = tokio::net::UnixListener::bind(&path).unwrap();
    let unix_app = axum::Router::new()
        .route("/ws", axum::routing::get(super::unix_ws_handler))
        .with_state(state.clone());
    let unix_task = tokio::spawn(async move {
        axum::serve(
            unix,
            unix_app.into_make_service_with_connect_info::<crate::native_ingress::UnixPeer>(),
        )
        .await
        .unwrap();
    });
    let (mut sink, mut stream) = raw_connect(&format!("unix://{}", path.display())).await;
    raw_register(&mut sink, &mut stream, "local-survivor").await;
    assert!(state.live_sessions.read().await.is_empty());
    let mut posture = state.authority.load().posture.clone();
    if let crate::authority::Posture::Verified(a) = &mut posture {
        a.epoch += 1;
    }
    apply_inventory_reload(&state, posture).await;
    let ping = BusMessage::new()
        .with_header("command", "noded.ping")
        .with_header("to", "noded")
        .with_header("id", "alive");
    assert_eq!(
        raw_call(&mut sink, &mut stream, ping, "alive").await.rc(),
        Some(0)
    );
    let tcp = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", tcp.local_addr().unwrap());
    let app = axum::Router::new()
        .route("/ws", axum::routing::get(super::ws_handler))
        .with_state(state.clone());
    let tcp_task = tokio::spawn(async move {
        axum::serve(
            tcp,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let (mut tcp_sink, mut tcp_stream) = raw_connect(&url).await;
    let register = BusMessage::new()
        .with_header("command", "noded.register")
        .with_header("from", "local-impostor")
        .with_header("id", "refuse");
    tcp_sink
        .send(WsMessage::Text(register.to_wire().into()))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(Ok(frame)) = tcp_stream.next().await {
            match frame {
                WsMessage::Text(wire) => {
                    let message = bus_mod::parse(&wire).unwrap();
                    if message.get("id") == Some("refuse") {
                        assert_eq!(message.rc(), Some(10));
                        assert!(message.get("error").unwrap_or("").contains("no-proof"));
                    }
                }
                WsMessage::Close(_) => break,
                _ => {}
            }
        }
    })
    .await
    .unwrap();
    assert!(!state.registry.read().await.contains_key("local-impostor"));
    unix_task.abort();
    tcp_task.abort();
    let _ = unix_task.await;
    let _ = tcp_task.await;
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(root).unwrap();
}
#[tokio::test]
async fn reload_refuses_reopened_key_and_accepts_new_key_rotation() {
    use base64::Engine as _;
    fn snapshot(epoch: u64, key: u8, until: Option<u64>) -> crate::authority::Posture {
        let mut posture = reload_posture(epoch, vec![]);
        if let crate::authority::Posture::Verified(a) = &mut posture {
            a.members_full.insert("alpha".into(), serde_json::json!({"name":"alpha","status":"active","bus":true,"credentials":[{
                "kind":"bus-tls","pubkey":base64::engine::general_purpose::STANDARD.encode([key;32]),"from_epoch":1,"until_epoch":until
            }]}));
        }
        posture
    }
    let state = reload_test_state(snapshot(2, 7, Some(2)), vec![]).await;
    let before = state.authority.load_full();
    apply_inventory_reload(&state, snapshot(3, 7, None)).await;
    assert!(std::sync::Arc::ptr_eq(
        &before,
        &state.authority.load_full()
    ));
    let mut rotated = snapshot(3, 8, None);
    if let crate::authority::Posture::Verified(a) = &mut rotated {
        let crate::authority::Posture::Verified(old) = &before.posture else {
            unreachable!()
        };
        a.members_full.get_mut("alpha").unwrap()["credentials"]
            .as_array_mut()
            .unwrap()
            .push(old.members_full["alpha"]["credentials"][0].clone());
    }
    apply_inventory_reload(&state, rotated).await;
    assert_eq!(state.authority.load().revision().epoch, 3);
}
