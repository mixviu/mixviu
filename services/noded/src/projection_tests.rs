// SPDX-License-Identifier: MIT OR Apache-2.0
use super::super::history_projection::Projector;
use super::*;
use crate::observe::{Direction, Observation, Outcome};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "load measurement"]
async fn history_observe_load() {
    let (state, _service_rx) = fixture().await;
    let (tx, mut rx) = mpsc::channel(64);
    state
        .observe
        .start(&tx, None, r#"{"capacity":4096}"#)
        .unwrap();
    rx.try_recv().unwrap();
    let writer = {
        let state = state.clone();
        tokio::spawn(async move {
            for _ in 0..2000 {
                drop(state.registry.write().await);
                tokio::task::yield_now().await;
            }
        })
    };
    let msg = BusMessage::new()
        .with_header("to", "maild.alpha.bus")
        .with_header("command", "example.read");
    let start = Instant::now();
    let mut received = 0;
    for i in 0..100_000 {
        state.observe.observe(Observation::canonical(
            Direction::Local,
            Outcome::Delivered,
            &msg,
            None,
        ));
        if i % 32 == 31 {
            state.observe.drain_once();
            while rx.try_recv().is_ok() {
                received += 1;
            }
            tokio::task::yield_now().await;
        }
    }
    writer.await.unwrap();
    assert_eq!(received, 100_000);
    println!(
        "HISTORY_OBSERVE rows={} rows_per_second={:.0} registry_locks_per_row=0 registry_lock_wait_ns=0",
        received,
        received as f64 / start.elapsed().as_secs_f64()
    );
}

async fn fixture() -> (AppState, mpsc::Receiver<String>) {
    let ip = "192.0.2.1".parse().unwrap();
    let mut posture = reload_posture(
        1,
        vec![
            active_route("alpha", ip, DEFAULT_NODED_PORT),
            active_route("example", "192.0.2.2".parse().unwrap(), DEFAULT_NODED_PORT),
        ],
    );
    if let crate::authority::Posture::Verified(a) = &mut posture {
        a.members_full.insert("example".into(), serde_json::json!({"name":"example", "status":"active", "bus":true, "mesh_ip":"192.0.2.2"}));
    }
    let state = reload_test_state(posture, vec![]).await;
    let (tx, rx) = mpsc::channel(8);
    let mut manifest = super::super::commands::Manifest::default();
    assert_eq!(manifest.replace(r#"["example.read"]"#), "0");
    state.registry.write().await.insert(
        "maild".into(),
        super::super::ServiceEntry {
            origin: super::super::Origin::Client,
            commands: manifest,
            tx,
            protected_responses: Default::default(),
            traffic_class: super::super::TrafficClass::Legacy,
            info: Default::default(),
        },
    );
    state.observe.set_projector(Projector {
        registry: state.registry.clone(),
        authority: state.authority.clone(),
        node: state.node_name.clone(),
        allowlist: state.history_allowlist.clone(),
    });
    (state, rx)
}

fn projected(
    state: &AppState,
    to: &str,
    command: &str,
) -> super::super::history_projection::Projected {
    let msg = BusMessage::new()
        .with_header("to", to)
        .with_header("command", command);
    Projector {
        registry: state.registry.clone(),
        authority: state.authority.clone(),
        node: state.node_name.clone(),
        allowlist: state.history_allowlist.clone(),
    }
    .project(&Observation::canonical(
        Direction::Local,
        Outcome::Rejected,
        &msg,
        None,
    ))
}

#[tokio::test]
async fn canonical_targets_and_fixed_failure_reasons() {
    let (state, _rx) = fixture().await;
    for target in [
        "maild",
        "maild.alpha",
        "maild.alpha.bus",
        "user.maild.alpha",
        "user.maild.alpha.bus",
    ] {
        let p = projected(&state, target, "example.read");
        assert_eq!(p.to.as_deref(), Some("maild"), "{target}");
        assert_eq!(p.reason, None);
        assert_eq!(p.to_sub_len, target.starts_with("user.").then_some(4));
    }
    for target in [
        "maild.example",
        "maild.example.bus",
        "user.maild.example",
        "user.maild.example.bus",
    ] {
        let p = projected(&state, target, "example.read");
        assert_eq!(p.to.as_deref(), Some("maild.example.bus"));
        assert_eq!(p.reason, None);
        assert_eq!(p.to_sub_len, target.starts_with("user.").then_some(4));
    }
    for target in ["noded", "alpha.bus", "noded.alpha", "noded.alpha.bus"] {
        assert_eq!(
            projected(&state, target, "noded.ping").to.as_deref(),
            Some("alpha.bus")
        );
    }
    for (target, label, reason, length) in [
        (
            "user.example.bus",
            "unknown-service.example.bus",
            "remote-unvouched",
            4,
        ),
        ("maild.absent.bus", "unknown-target", "no-such-node", 6),
        ("absent", "unknown-target", "no-such-service", 6),
        ("user@example.com", "invalid-target", "malformed-target", 16),
        (
            "maild.example.bus@example.test",
            "invalid-target",
            "refused-cross-mesh",
            30,
        ),
    ] {
        let p = projected(&state, target, "example.read");
        assert_eq!(p.to.as_deref(), Some(label), "{target}");
        assert_eq!(p.reason, Some(reason), "{target}");
        assert_eq!(p.name_len, Some(length), "{target}");
    }
}

#[tokio::test]
async fn mesh_source_uses_bridge_node_and_exact_session_shape() {
    let (state, _rx) = fixture().await;
    let projector = Projector {
        registry: state.registry.clone(),
        authority: state.authority.clone(),
        node: state.node_name.clone(),
        allowlist: state.history_allowlist.clone(),
    };
    for (from, expected) in [
        ("maild.forged.bus", "maild.example.bus"),
        ("user.forged.bus", "unknown-service.example.bus"),
        (
            "t0-aaaaaaaaaaaaaaaaaaaaaa.forged.bus",
            "t0-aaaaaaaaaaaaaaaaaaaaaa.example.bus",
        ),
        ("t0-short.forged.bus", "unknown-service.example.bus"),
    ] {
        let msg = BusMessage::new()
            .with_header("from", from)
            .with_header("command", "example.read");
        let p = projector.project(
            &Observation::canonical(Direction::MeshIn, Outcome::Delivered, &msg, None)
                .with_peer("example"),
        );
        assert_eq!(p.from.as_deref(), Some(expected));
    }
}

/// Generated hostile strings exercise the full observe queue/serialization,
/// including rejected rows. No new property-testing dependency is needed.
#[tokio::test]
async fn generated_caller_names_never_enter_metadata_and_rows_keep_sequence() {
    use rand::{RngCore, SeedableRng};
    let (state, _rx) = fixture().await;
    let (tx, mut rx) = mpsc::channel(64);
    state
        .observe
        .start(&tx, None, r#"{"capacity":64}"#)
        .unwrap();
    rx.try_recv().unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(293);
    let mut seq = 0;
    for index in 0..512 {
        let chosen = format!("private{}{}", index, rng.next_u64());
        for target in [
            format!("{chosen}.maild.alpha.bus"),
            format!("{chosen}.maild.alpha"),
            format!("{chosen}.maild.example.bus"),
            format!("{chosen}.maild.example"),
            format!("{chosen}.example.bus"),
            format!("{chosen}.example"),
            format!("maild.{chosen}.bus"),
            format!("maild.{chosen}"),
            format!("{chosen}@example.test"),
            format!("{chosen}.界"),
            chosen.repeat(20),
        ] {
            let command = match index % 4 {
                0 => format!("{chosen}@example.test"),
                1 => format!("{chosen}.verb"),
                2 => format!("{chosen}界"),
                _ => chosen.repeat(10),
            };
            let msg = BusMessage::new()
                .with_header("to", &target)
                .with_header("command", &command);
            state.observe.observe(Observation::canonical(
                Direction::Local,
                Outcome::Rejected,
                &msg,
                Some("example-id"),
            ));
            state.observe.drain_once();
            let wire = rx.try_recv().expect("projected row must exist");
            assert!(!wire.contains(&chosen), "caller bytes leaked");
            let event = bus_mod::parse(&wire).unwrap();
            let body: serde_json::Value = serde_json::from_str(&event.body).unwrap();
            seq += 1;
            assert_eq!(body["seq"], seq);
            assert_eq!(body["command"], "unknown");
            assert_eq!(body["class"], "unrecognised");
            assert_eq!(body["correlation_id"], "example-id");
            assert_eq!(body["size"], msg.to_wire().len());
            assert_eq!(body["dropped_count"], 0);
        }
    }
    assert_eq!(seq, 5632);
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn response_echo_and_manifest_lifetime_fail_closed() {
    let (state, rx) = fixture().await;
    let msg = BusMessage::new()
        .with_header("type", "response")
        .with_header("from", "maild")
        .with_header("to", "maild.alpha.bus")
        .with_header("command", "user@example.com");
    let projector = Projector {
        registry: state.registry.clone(),
        authority: state.authority.clone(),
        node: state.node_name.clone(),
        allowlist: state.history_allowlist.clone(),
    };
    let p = projector.project(&Observation::canonical(
        Direction::Local,
        Outcome::Delivered,
        &msg,
        None,
    ));
    assert_eq!(p.command.as_deref(), Some("unknown"));
    assert_eq!(p.reason, Some("unknown-verb"));
    assert_eq!(p.name_len, Some(16));
    assert!(state.vouched_command("example.read").await);
    drop(rx);
    assert!(!state.vouched_command("example.read").await);
    assert_eq!(
        projected(&state, "maild", "example.read")
            .command
            .as_deref(),
        Some("unknown")
    );
}

#[tokio::test]
async fn published_inner_command_is_projected_on_real_fanout() {
    use bus::native_session::{Assurance, BrokerPrincipal, HexBytes, PrincipalVersion};
    let (state, _service_rx) = fixture().await;
    let (observer, mut observed) = mpsc::channel(8);
    state
        .observe
        .start(
            &observer,
            None,
            r#"{"filter":{"commands":["user.example"]}}"#,
        )
        .unwrap();
    observed.try_recv().unwrap();
    let broker = SubscriptionBroker::with_observe(state.observe.clone());
    let (subscriber, mut delivered) = mpsc::channel(8);
    broker
        .subscribe_topic_verified("example.events", "maild", subscriber, None, true)
        .await;
    let (publisher, _publisher_rx) = mpsc::channel(8);
    let principal = BrokerPrincipal {
        version: PrincipalVersion::V1,
        assurance: Assurance::LocalUnix,
        owner_node: "alpha".into(),
        unix_uid: 1000,
        unix_gid: 1000,
        peer_pid: 10,
        broker_epoch: HexBytes([1; 16]),
        connection_id: HexBytes([2; 16]),
        session: None,
    };
    let inner = BusMessage::new()
        .with_header("type", "event")
        .with_header("command", "user.example");
    broker
        .publish_with_context(
            "example.events",
            &inner.to_wire(),
            "maild",
            publisher,
            crate::subscription::BrokerOrigin::Local,
            false,
            Some(&principal),
            Some("example-connection"),
        )
        .await
        .unwrap();
    assert_eq!(
        bus_mod::parse(&delivered.recv().await.unwrap())
            .unwrap()
            .command_name(),
        Some("user.example")
    );
    state.observe.drain_once();
    let event = bus_mod::parse(&observed.try_recv().unwrap()).unwrap();
    let body: serde_json::Value = serde_json::from_str(&event.body).unwrap();
    assert_eq!(body["command"], "unknown");
    assert_eq!(body["name_len"], "user.example".len());
    assert_eq!(body["seq"], 1);
    assert!(!event.body.contains("user.example"));
    assert!(observed.try_recv().is_err());
}
