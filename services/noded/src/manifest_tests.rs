// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

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
async fn raw_clients_cannot_register_broker_fixed_or_uninventoried_names() {
    let state = reload_test_state(reload_posture(1, vec![]), vec![]).await;
    let (url, task) = broker(state.clone()).await;
    let mut accepted = Vec::new();
    for identity in ["anon-x", "log-observe-123", "t0-x", "c0-x", "bridge-example"] {
        let (mut sink, mut stream) = raw_connect(&url).await;
        let request = BusMessage::new()
            .with_header("command", "noded.register")
            .with_header("from", identity)
            .with_header("to", "noded")
            .with_header("type", "request")
            .with_header("id", "reserved-register");
        let response = raw_call(&mut sink, &mut stream, request, "reserved-register").await;
        if response.get("rc") == Some("0") {
            let request = BusMessage::new()
                .with_header("command", "noded.commands.set")
                .with_header("to", "noded")
                .with_header("type", "request")
                .with_header("id", "reserved-set")
                .with_body(r#"["example.vouch"]"#);
            let response = raw_call(&mut sink, &mut stream, request, "reserved-set").await;
            accepted.push((
                identity,
                response.get("rc").map(str::to_owned),
                state.vouched_command("example.vouch").await,
            ));
            let request = BusMessage::new()
                .with_header("command", "noded.deregister")
                .with_header("to", "noded")
                .with_header("type", "request")
                .with_header("id", "reserved-deregister");
            assert_eq!(
                raw_call(&mut sink, &mut stream, request, "reserved-deregister")
                    .await
                    .get("rc"),
                Some("0")
            );
        }
        sink.close().await.unwrap();
    }
    task.abort();
    assert!(
        accepted.is_empty(),
        "round-3 stop rule: raw registrations accepted (identity, set rc, honoured): {accepted:?}"
    );
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
            "on example.read desc \"Example authored description\"\nend\nquit()\n",
        )
        .await
        .unwrap();
    evaluator.run_event_pump().await.unwrap();
    wait_name(&state, "example.read", true).await;
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
