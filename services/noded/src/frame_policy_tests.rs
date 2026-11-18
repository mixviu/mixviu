// SPDX-License-Identifier: MIT OR Apache-2.0

#[test]
fn frame_posture_defaults_observe_and_is_independent_of_admission() {
    let node: config::node::NodeConfig = config::from_conf_mix_str(
        "node: \"alpha\"\nnoded: { admission: \"enforce\", frame_policy: \"enforce\" }\n",
    ).unwrap();
    assert_eq!(super::frame_policy_config(&node.noded, None).unwrap(), AdmissionMode::Enforce);
    let mut config = config::node::NodedConfig::default();
    config.admission = AdmissionMode::Enforce;
    assert_eq!(super::frame_policy_config(&config, None).unwrap(), AdmissionMode::Observe);
    config.extra.insert("frame_policy".into(), serde_json::json!("enforce"));
    assert_eq!(super::frame_policy_config(&config, None).unwrap(), AdmissionMode::Enforce);
    assert_eq!(super::frame_policy_config(&config, Some("off")).unwrap(), AdmissionMode::Off);
    assert_eq!(super::frame_policy_config(&config, Some("observe")).unwrap(), AdmissionMode::Observe);
    assert!(super::frame_policy_config(&config, Some("invalid")).is_err());
    config.extra.insert("frame_policy".into(), serde_json::json!(true));
    assert!(super::frame_policy_config(&config, None).is_err());
}

async fn next_mesh_frame(socket: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) -> BusMessage {
    loop {
        if let WsMessage::Text(text) = socket.next().await.expect("peer stayed connected").expect("valid frame") {
            return bus_mod::parse(&text).unwrap();
        }
    }
}

async fn policy_mesh_state(
    endpoint: std::net::SocketAddr,
) -> (AppState, mpsc::UnboundedReceiver<MeshInbound>, PeerConfig) {
    let peer = PeerConfig { name: "beta".into(), mesh_ip: endpoint.ip(), noded_port: endpoint.port() };
    let mut state = reload_test_state(reload_posture(1, vec![
        active_route("alpha", "127.0.0.1".parse().unwrap(), DEFAULT_NODED_PORT),
        active_route("beta", endpoint.ip(), endpoint.port()),
    ]), vec![]).await;
    let (tx, incoming) = mpsc::unbounded_channel();
    state.mesh = Arc::new(MeshPeers::new(MeshConfig {
        node_name: "alpha".into(), peers: vec![peer.clone()], d2_seed: None,
    }, tx));
    let authority = state.authority.load_full();
    state.mesh.reconcile_endpoints(authority.routes.desired_endpoints(), authority.revision()).await;
    (state, incoming, peer)
}

#[tokio::test]
async fn serve_mesh_receiver_policy_delivers_observe_and_drops_enforce() {
    for mode in [AdmissionMode::Off, AdmissionMode::Observe, AdmissionMode::Enforce] {
        for (from, to) in [("hostd.beta.bus", "recipient.gamma.bus"), ("hostd", "recipient.alpha.bus"), ("absent", "recipient.alpha.bus"), ("hostd.gamma.bus", "recipient.alpha.bus"), ("", "")] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let (state, incoming, peer) = policy_mesh_state(listener.local_addr().unwrap()).await;
            let (target, mut received) = mpsc::channel(8);
            state.registry.write().await.insert("recipient".into(), super::ServiceEntry {
                tx: target, info: Default::default(), traffic_class: Default::default(), protected_responses: Default::default(),
            });
            let (go, ready) = oneshot::channel();
            let (stop, stopped) = oneshot::channel();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
                socket.send(WsMessage::Ping(Vec::new().into())).await.unwrap();
                assert_eq!(next_mesh_frame(&mut socket).await.command_name(), Some("noded.register"));
                ready.await.unwrap();
                let mut bad = if from.is_empty() { BusMessage::empty() } else {
                    BusMessage::new().with_header("from", from).with_header("to", to).with_header("id", "bad").with_header("type", "event")
                };
                if from == "absent" { bad.headers.remove("from"); }
                let good = BusMessage::new().with_header("from", "hostd.beta.bus").with_header("to", "recipient.alpha.bus").with_header("id", "good").with_header("type", "event");
                for frame in [bad, good] { socket.send(WsMessage::Text(frame.to_wire().into())).await.unwrap(); }
                let _ = stopped.await;
            });
            let receiver = tokio::spawn(super::mesh_incoming_loop(incoming, state.mesh.clone(), state.authority.clone(), state.registry.clone(), state.observe.clone(), "alpha".into(), mode));
            state.mesh.send(peer, BusMessage::empty()).await.unwrap();
            go.send(()).unwrap();
            let first = tokio::time::timeout(Duration::from_secs(2), received.recv()).await.unwrap().unwrap();
            let first = bus_mod::parse(&first).unwrap();
            if mode == AdmissionMode::Enforce || from.is_empty() {
                assert_eq!(first.get("id"), Some("good"));
            } else {
                assert_eq!(first.get("id"), Some("bad"));
                let next = tokio::time::timeout(Duration::from_secs(2), received.recv()).await.unwrap().unwrap();
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
            let (mut state, _incoming, _peer) = policy_mesh_state(listener.local_addr().unwrap()).await;
            state.frame_policy = mode;
            let (stop, stopped) = oneshot::channel();
            let (answered, answer_sent) = oneshot::channel();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
                socket.send(WsMessage::Ping(Vec::new().into())).await.unwrap();
                next_mesh_frame(&mut socket).await;
                let request = next_mesh_frame(&mut socket).await;
                assert_eq!(request.get("hop_limit"), Some("0"));
                let response = BusMessage::new().with_header("type", "response").with_header("from", from)
                    .with_header("id", request.get("id").unwrap()).with_header("rc", "0");
                socket.send(WsMessage::Text(response.to_wire().into())).await.unwrap();
                let _ = answered.send(());
                let _ = stopped.await;
            });
            let app = axum::Router::new().route("/ws", axum::routing::get(move |upgrade: axum::extract::ws::WebSocketUpgrade| {
                let state = state.clone();
                async move { upgrade.on_upgrade(move |socket| super::handle_socket(socket, state, bus::native_session::TransportIdentity::LegacyTcp { source_ip: "127.0.0.1".parse().unwrap() })) }
            }));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("ws://{}/ws", listener.local_addr().unwrap());
            let local = tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
            let (mut sink, mut stream) = raw_connect(&url).await;
            let request = BusMessage::new().with_header("to", "hostd.beta.bus").with_header("command", "probe.echo")
                .with_header("id", "reply-probe").with_header("request_id", "correlation-probe");
            if mode == AdmissionMode::Enforce && from != "hostd.beta.bus" {
                sink.send(WsMessage::Text(request.to_wire().into())).await.unwrap();
                tokio::time::timeout(Duration::from_secs(3), answer_sent).await.unwrap().unwrap();
                assert!(tokio::time::timeout(Duration::from_millis(250), stream.next()).await.is_err());
            } else {
                let reply = raw_call(&mut sink, &mut stream, request, "reply-probe").await;
                assert_eq!(reply.rc(), Some(0));
                assert_eq!(reply.get("request_id"), Some("correlation-probe"));
            }
            let _ = stop.send(());
            server.await.unwrap();
            local.abort();
        }
    }
}
