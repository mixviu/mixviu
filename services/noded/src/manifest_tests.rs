// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

/// Run explicitly on buildd; reports rather than enforcing a machine speed.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "load measurement"]
async fn history_vouched_load() {
    let state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let (url, task) = broker(state.clone()).await;
    let client = bus::native_client::NodedClient::connect("maild", &url)
        .await
        .unwrap();
    client.set_commands(vec![bus::CommandDescriptor::new(
        "example.read",
        &[],
        "Example",
        true,
    )]);
    wait_name(&state, "example.read", true).await;
    let started = Instant::now();
    let mut jobs = Vec::new();
    for _ in 0..8 {
        let state = state.clone();
        jobs.push(tokio::spawn(async move {
            let mut wait = Duration::ZERO;
            let mut max_wait = Duration::ZERO;
            for _ in 0..20_000 {
                let start = Instant::now();
                let guard = state.registry.read().await;
                let elapsed = start.elapsed();
                wait += elapsed;
                max_wait = max_wait.max(elapsed);
                drop(guard);
                assert!(state.vouched_command("example.read").await);
            }
            (wait, max_wait)
        }));
    }
    let writer = {
        let state = state.clone();
        tokio::spawn(async move {
            for _ in 0..2000 {
                let guard = state.registry.write().await;
                drop(guard);
                tokio::task::yield_now().await;
            }
        })
    };
    let mut wait = Duration::ZERO;
    let mut max_wait = Duration::ZERO;
    for job in jobs {
        let (sum, max) = job.await.unwrap();
        wait += sum;
        max_wait = max_wait.max(max);
    }
    writer.await.unwrap();
    println!(
        "HISTORY_LOAD queries=160000 queries_per_second={:.0} registry_read_wait_mean_ns={} registry_read_wait_max_ns={}",
        160000.0 / started.elapsed().as_secs_f64(),
        wait.as_nanos() / 160000,
        max_wait.as_nanos()
    );
    client.close().await;
    task.abort();
}

async fn broker(state: AppState) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let app = axum::Router::new()
        .route("/ws", axum::routing::get(super::super::ws_handler))
        .with_state(state);
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    (url, task)
}

async fn wait_name(state: &AppState, name: &str, present: bool) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if state.vouched_command(name).await == present {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("manifest convergence");
}

#[tokio::test]
async fn empty_allowlist_honours_nothing_and_bursts_coalesce() {
    let mut state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let mut config = config::node::NodedConfig::default();
    config
        .extra
        .insert("history_allowlist".into(), serde_json::json!([]));
    state.registry = Arc::new(super::super::history_projection::HistoryRegistry::new(
        HashMap::new(), super::super::commands::allowlist(Some(&config)).unwrap(),
    ));
    let (url, task) = broker(state.clone()).await;
    let client = bus::native_client::NodedClient::connect("maild", &url)
        .await
        .unwrap();
    for i in 0..20 {
        client.set_commands(vec![bus::CommandDescriptor::new(
            &format!("example.cmd{i}"),
            &[],
            "Example",
            true,
        )]);
    }
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if state.registry.read().await["maild"]
                .commands
                .names
                .contains("example.cmd19")
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(state.registry.read().await["maild"].commands.attempts() <= 3);
    assert!(!state.vouched_command("example.cmd19").await);
    client.close().await;
    task.abort();
}

#[tokio::test]
async fn oversized_manifest_keeps_registry_responsive() {
    let state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let (url, task) = broker(state.clone()).await;
    let (mut a, mut ar) = raw_connect(&url).await;
    let (mut b, mut br) = raw_connect(&url).await;
    raw_register(&mut a, &mut ar, "maild").await;
    // Fits the byte cap, but must stop at element 513 instead of allocating
    // all 40,000 strings. The other registration must not wait on JSON parsing.
    let body = serde_json::to_string(&vec!["a"; 40_000]).unwrap();
    assert!(body.len() < super::super::commands::MAX_BODY_BYTES);
    let request = BusMessage::new()
        .with_header("command", "noded.commands.set")
        .with_header("to", "noded")
        .with_header("id", "large")
        .with_body(&body);
    tokio::time::timeout(Duration::from_secs(1), async {
        let (reply, ()) = tokio::join!(
            raw_call(&mut a, &mut ar, request, "large"),
            raw_register(&mut b, &mut br, "webd")
        );
        assert_eq!(reply.get("rc"), Some("21"));
    })
    .await
    .unwrap();
    assert!(state.registry.read().await.contains_key("webd"));
    a.close().await.unwrap();
    b.close().await.unwrap();
    task.abort();
}

#[tokio::test]
async fn default_allowlist_starts() {
    let (listener, listen) = pick_listener().unwrap();
    let (ready, readiness) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(super::super::run_on(
        super::super::RunConfig {
            network: Some(config::node::NodedConfig::default()),
            session_probe: None,
            unix_socket: None,
            pending_grants_per_parent: 32,
            listen,
            node: "example".into(),
            mesh_addr: "127.0.0.1".into(),
            mesh_config_path: None,
            spec_dir: None,
            admission_mode: config::node::AdmissionMode::Off,
            frame_policy: config::node::AdmissionMode::Off,
        },
        listener,
        ready,
    ));
    tokio::time::timeout(Duration::from_secs(3), readiness)
        .await
        .unwrap()
        .unwrap();
    task.abort();
}

#[tokio::test]
async fn reserved_names_follow_broker_024_and_exact_allowlist() {
    let mut state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let token = super::super::task_identity::Token::new();
    state.task_identities.logger = Some(token.clone());
    let (url, task) = broker(state.clone()).await;
    for (identity, allowed, honoured) in [
        ("anon-x", false, false),
        ("log-observe-123", false, false),
        ("log", false, false),
        ("bridge-example", false, false),
        ("t0-x", true, false),
        ("c0-x", true, false),
        ("maild", true, true),
    ] {
        let (mut sink, mut stream) = raw_connect(&url).await;
        let request = BusMessage::new()
            .with_header("command", "noded.register")
            .with_header("from", identity)
            .with_header("to", "noded")
            .with_header("type", "request")
            .with_header("id", "register");
        let reply = raw_call(&mut sink, &mut stream, request, "register").await;
        assert_eq!(
            reply.get("rc"),
            Some(if allowed { "0" } else { "10" }),
            "{identity}"
        );
        if allowed {
            let request = BusMessage::new()
                .with_header("command", "noded.commands.set")
                .with_header("to", "noded")
                .with_header("type", "request")
                .with_header("id", "set")
                .with_body(r#"["example.vouch"]"#);
            assert_eq!(
                raw_call(&mut sink, &mut stream, request, "set")
                    .await
                    .get("rc"),
                Some("0")
            );
            assert_eq!(
                state.vouched_command("example.vouch").await,
                honoured,
                "{identity}"
            );
        }
        sink.close().await.unwrap();
        tokio::time::timeout(Duration::from_secs(3), async {
            while state.registry.read().await.contains_key(identity) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }
    let logger = token.connect("log", &url).await.unwrap();
    assert_eq!(
        state.registry.read().await["log"].origin,
        super::super::Origin::Logger
    );
    assert!(!state.honours_manifest("log"));
    for verb in ["log.props.get", "log.props.list", "log.props.describe"] {
        assert!(super::super::commands::BROKER_CONTROL_VERBS.contains(&verb));
    }
    logger.close().await;
    task.abort();
}

#[tokio::test]
async fn manifest_ownership_refuses_unregistered_and_stale_channels() {
    let state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let (url, task) = broker(state.clone()).await;
    let (mut a, mut ar) = raw_connect(&url).await;
    let request = || {
        BusMessage::new()
            .with_header("command", "noded.commands.set")
            .with_header("to", "noded")
            .with_header("id", "set")
            .with_header("type", "request")
            .with_body(r#"["example.current"]"#)
    };
    assert_eq!(
        raw_call(&mut a, &mut ar, request(), "set").await.get("rc"),
        Some("26")
    );
    raw_register(&mut a, &mut ar, "maild").await;
    // Simulate registry pruning before the stale socket's service_name cleanup.
    // A live owner still rejects collisions; this does not bypass that policy.
    state.registry.write().await.remove("maild");
    let (mut b, mut br) = raw_connect(&url).await;
    raw_register(&mut b, &mut br, "maild").await;
    assert_eq!(
        raw_call(&mut b, &mut br, request(), "set").await.get("rc"),
        Some("0")
    );
    let mut stale = request();
    stale.body = r#"["example.stale"]"#.into();
    assert_eq!(
        raw_call(&mut a, &mut ar, stale, "set").await.get("rc"),
        Some("26")
    );
    assert!(state.vouched_command("example.current").await);
    assert!(!state.vouched_command("example.stale").await);
    a.close().await.unwrap();
    b.close().await.unwrap();
    task.abort();
}

#[tokio::test]
async fn malformed_allowlists_refuse_startup() {
    for value in [
        serde_json::json!("maild"),
        serde_json::json!(42),
        serde_json::json!(["maild", 42]),
        serde_json::json!(["t0-aaaaaaaaaaaaaaaaaaaaaa"]),
        serde_json::json!(["c0-aaaaaaaaaaaaaaaaaaaaaa"]),
        serde_json::json!(["bridge-example"]),
        serde_json::json!(["anon-example"]),
        serde_json::json!(["log"]),
        serde_json::json!(["log-observe-123"]),
        serde_json::json!(["noded"]),
        serde_json::json!(["noded-example"]),
    ] {
        let (listener, listen) = pick_listener().unwrap();
        let mut network = config::node::NodedConfig::default();
        network.extra.insert("history_allowlist".into(), value);
        let (ready, mut readiness) = tokio::sync::oneshot::channel();
        let result = super::super::run_on(
            super::super::RunConfig {
                network: Some(network),
                session_probe: None,
                unix_socket: None,
                pending_grants_per_parent: 32,
                listen,
                node: "example".into(),
                mesh_addr: "127.0.0.1".into(),
                mesh_config_path: None,
                spec_dir: None,
                admission_mode: config::node::AdmissionMode::Off,
                frame_policy: config::node::AdmissionMode::Off,
            },
            listener,
            ready,
        )
        .await;
        assert!(result.is_err());
        assert!(readiness.try_recv().is_err());
    }
}

#[tokio::test]
async fn manifest_refusals_warn_and_budget_recovers() {
    if log_capture_subprocess("manifest_tests::manifest_refusals_warn_and_budget_recovers") {
        return;
    }
    #[derive(Clone)]
    struct Capture(Arc<std::sync::Mutex<Vec<u8>>>);
    impl std::io::Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let capture = Capture(Arc::new(std::sync::Mutex::new(Vec::new())));
    let writer = capture.clone();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(tracing::Level::WARN)
        .with_writer(move || writer.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);
    let state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let (url, task) = broker(state.clone()).await;
    let client = bus::native_client::NodedClient::connect("maild", &url)
        .await
        .unwrap()
        .with_commands(vec![bus::CommandDescriptor::new(
            "example.initial",
            &[],
            "Example",
            true,
        )]);
    wait_name(&state, "example.initial", true).await;
    let incarnation =
        state.registry.read().await["maild"].info.meta["noded.registration_id"].clone();
    for index in 0..17 {
        let name = format!("example.cmd{index}");
        client.set_commands(vec![bus::CommandDescriptor::new(
            &name,
            &[],
            "Example",
            true,
        )]);
        wait_name(&state, &name, true).await;
    }
    assert_ne!(
        state.registry.read().await["maild"].info.meta["noded.registration_id"],
        incarnation
    );
    client.set_commands(vec![bus::CommandDescriptor::new(
        "example@invalid",
        &[],
        "Example",
        true,
    )]);
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if String::from_utf8_lossy(&capture.0.lock().unwrap()).contains("rc=\"24\"") {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let incarnation =
        state.registry.read().await["maild"].info.meta["noded.registration_id"].clone();
    client
        .call("noded", "noded.ping", serde_json::Value::Null)
        .await
        .unwrap();
    assert_eq!(
        state.registry.read().await["maild"].info.meta["noded.registration_id"],
        incarnation
    );
    client.set_commands(vec![bus::CommandDescriptor::new(
        "example.recovered",
        &[],
        "Example",
        true,
    )]);
    wait_name(&state, "example.recovered", true).await;
    let logs = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
    assert!(logs.contains("rc=\"25\""), "{logs}");
    assert_eq!(logs.matches("rc=\"24\"").count(), 1, "{logs}");
    assert_eq!(logs.matches("rc=\"25\"").count(), 1, "{logs}");
    client.close().await;
    task.abort();
}

#[tokio::test]
async fn broker_limits_clear_the_previous_set_and_registration_resets_budget() {
    let state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let (url, task) = broker(state.clone()).await;
    let (mut sink, mut stream) = raw_connect(&url).await;
    raw_register(&mut sink, &mut stream, "maild").await;
    for (body, rc) in [
        (serde_json::json!(vec!["example.read"; 513]), "21"),
        (serde_json::json!(["x".repeat(65)]), "22"),
        (serde_json::json!(["a.b.c.d.e.f.g"]), "23"),
        (serde_json::json!(["example@invalid"]), "24"),
        (serde_json::json!(["example..read"]), "24"),
        (serde_json::json!({}), "20"),
    ] {
        for (value, expected) in [(serde_json::json!(["example.read"]), "0"), (body, rc)] {
            let request = BusMessage::new()
                .with_header("command", "noded.commands.set")
                .with_header("to", "noded")
                .with_header("type", "request")
                .with_header("id", "set")
                .with_body(&value.to_string());
            assert_eq!(
                raw_call(&mut sink, &mut stream, request, "set")
                    .await
                    .get("rc"),
                Some(expected)
            );
            assert_eq!(state.vouched_command("example.read").await, expected == "0");
        }
    }
    for index in 12..18 {
        let request = BusMessage::new()
            .with_header("command", "noded.commands.set")
            .with_header("to", "noded")
            .with_header("type", "request")
            .with_header("id", "budget")
            .with_body(r#"["example.read"]"#);
        let rc = if index < 16 { "0" } else { "25" };
        assert_eq!(
            raw_call(&mut sink, &mut stream, request, "budget")
                .await
                .get("rc"),
            Some(rc)
        );
        assert_eq!(state.vouched_command("example.read").await, rc == "0");
    }
    raw_register(&mut sink, &mut stream, "maild").await;
    let request = BusMessage::new()
        .with_header("command", "noded.commands.set")
        .with_header("to", "noded")
        .with_header("id", "reset")
        .with_header("type", "request")
        .with_body("[]");
    assert_eq!(
        raw_call(&mut sink, &mut stream, request, "reset")
            .await
            .get("rc"),
        Some("0")
    );
    sink.close().await.unwrap();
    task.abort();
}

#[tokio::test]
async fn real_disconnect_removes_names_and_supervised_reconnect_restores_them() {
    let state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let (url, task) = broker(state.clone()).await;
    let proxy = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_url = format!("ws://{}/ws", proxy.local_addr().unwrap());
    let address = url
        .strip_prefix("ws://")
        .unwrap()
        .strip_suffix("/ws")
        .unwrap()
        .to_owned();
    let (cut, cut_rx) = tokio::sync::oneshot::channel::<()>();
    let (resume, resume_rx) = tokio::sync::oneshot::channel::<()>();
    let proxy_task = tokio::spawn(async move {
        let (mut downstream, _) = proxy.accept().await.unwrap();
        let mut upstream = tokio::net::TcpStream::connect(&address).await.unwrap();
        tokio::select! {
            _ = tokio::io::copy_bidirectional(&mut downstream, &mut upstream) => {},
            _ = cut_rx => {},
        }
        drop(downstream);
        drop(upstream);
        resume_rx.await.unwrap();
        let (mut downstream, _) = proxy.accept().await.unwrap();
        let mut upstream = tokio::net::TcpStream::connect(address).await.unwrap();
        let _ = tokio::io::copy_bidirectional(&mut downstream, &mut upstream).await;
    });
    let client = bus::native_client::SupervisedClient::connect("maild", &proxy_url)
        .await
        .unwrap();
    client.set_command_names(vec!["example.read".into()]).await;
    wait_name(&state, "example.read", true).await;
    // Close the actual socket from the broker side. The client must register
    // again and replay its names, with a new registration incarnation.
    let old = state.registry.read().await["maild"].info.meta["noded.registration_id"].clone();
    cut.send(()).unwrap();
    wait_name(&state, "example.read", false).await;
    tokio::time::timeout(Duration::from_secs(3), async {
        while state.registry.read().await.contains_key("maild") {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(!state.registry.read().await.contains_key("maild"));
    resume.send(()).unwrap();
    wait_name(&state, "example.read", true).await;
    assert_ne!(
        state.registry.read().await["maild"].info.meta["noded.registration_id"],
        old
    );
    client.close().await;
    wait_name(&state, "example.read", false).await;
    tokio::time::timeout(Duration::from_secs(3), async {
        while state.registry.read().await.contains_key("maild") {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    task.abort();
    proxy_task.abort();
}

#[tokio::test]
async fn allowlisted_mix_citizen_publishes_handlers_without_changing_help() {
    use mix::evaluator::{Evaluator, ServeRuntime};
    use std::rc::Rc;
    let state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let (url, task) = broker(state.clone()).await;
    let client = Arc::new(
        bus::native_client::SupervisedClient::connect("maild", &url)
            .await
            .unwrap(),
    );
    let runtime = Rc::new(mix_shell::MixServeRuntime::new("maild").with_bus(client.clone()));
    let mut evaluator = Evaluator::new();
    evaluator.set_serve_runtime(runtime.clone());
    evaluator
        .execute_script_source(
            "on example.read desc \"Example authored description\"\nend\non a.b.c.d.e.f.g\nend\nquit()\n",
        )
        .await
        .unwrap();
    evaluator.run_event_pump().await.unwrap();
    wait_name(&state, "example.read", true).await;
    assert!(!state.vouched_command("a.b.c.d.e.f.g").await);
    let incarnation =
        state.registry.read().await["maild"].info.meta["noded.registration_id"].clone();
    for _ in 0..20 {
        runtime
            .publish_commands(&["example.read".into(), "a.b.c.d.e.f.g".into()])
            .await;
    }
    assert_eq!(
        state.registry.read().await["maild"].info.meta["noded.registration_id"],
        incarnation
    );
    let help = runtime
        .handle_reserved(
            "HELP",
            None,
            "",
            &[("example.read", Some("Example authored description"))],
            true,
        )
        .unwrap();
    assert!(help.body.contains("Example authored description"));
    // Native HELP must still reach the owning Mix runtime.
    let caller = bus::native_client::NodedClient::connect_anonymous(&url)
        .await
        .unwrap();
    let mut incoming = client.incoming().unwrap();
    let help_call = caller.call("maild", "HELP", serde_json::Value::Null);
    let receive = async {
        // The signed-inventory fixture sends its bootstrap challenge before
        // registration. As the serve driver does, ignore this non-app command.
        let request = tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let request = incoming.recv().await.unwrap();
                if request.command != "noded.admit.challenge" {
                    break request;
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(request.command, "HELP");
        client.respond(&request, 0, "[]").await.unwrap();
    };
    let (reply, ()) = tokio::join!(help_call, receive);
    assert_eq!(reply.unwrap(), serde_json::json!([]));
    caller.close().await;
    client.close().await;
    task.abort();
}
