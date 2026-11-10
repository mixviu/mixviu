// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
// Gold 8779f33 native-session fault tests, included in session_actor::tests.
// Actor-only mechanisms: no application or tab policy is imported.
    fn descriptor(fd: &LaunchFd) -> GrantResult {
        // Read only public descriptor bytes, never copy the seed into a test log.
        let mut header = [0u8; 5];
        assert_eq!(
            unsafe { libc::pread(fd.mapping().0, header.as_mut_ptr().cast(), 5, 0) },
            5
        );
        assert_eq!(header[0], 1);
        let n = u32::from_be_bytes(header[1..].try_into().unwrap()) as usize;
        assert!(n <= 16384);
        let mut json = vec![0; n];
        assert_eq!(
            unsafe { libc::pread(fd.mapping().0, json.as_mut_ptr().cast(), n, 5) },
            n as isize
        );
        serde_json::from_slice(&json).unwrap()
    }

    async fn expire(actor: &mut Actor, id: u64, now: Instant) {
        let record = actor.children[&id].record.clone().unwrap();
        if record.state == BindingState::Revoked {
            return;
        }
        actor
            .connection
            .as_ref()
            .unwrap()
            .session_revoke(record.reference())
            .await
            .unwrap();
        actor.notice_at(&serde_json::json!({"target":record.reference(), "state":"revoked", "broker_epoch":record.broker_epoch}).to_string(), now).await;
        actor
            .children
            .get_mut(&id)
            .unwrap()
            .pending
            .as_mut()
            .unwrap()
            .deadline = 0;
        if let Some(ready) = &mut actor.shared.lock().unwrap().ready {
            ready.deadline = 0;
        }
    }

    #[test]
    fn pool_failure_latches_distinguish_free_retry_from_uncertain_mint() {
        let broker = Broker::start();
        runtime().block_on(async {
            let mut actor = test_actor(&broker);
            actor.connect().await;
            actor.faults.memfd = true;
            actor.provision().await;
            assert_eq!(actor.provisioned, None);
            assert_eq!(actor.grant_creates, 1);
            actor.provision().await;
            assert!(actor.shared.lock().unwrap().ready.is_some());
            assert_eq!(actor.grant_creates, 1, "retry delivery of the known grant");
            let (_pane, fd) = test_handle(&actor).prepare(1).unwrap();
            drop(fd);
            // Failure before create sends nothing, so no slot latch may remain.
            actor.faults.fetch = true;
            actor.provision().await;
            assert_eq!(actor.provisioned, None);
            assert_eq!(actor.grant_creates, 1);
            actor.faults.create_ack = true;
            actor.provision().await;
            assert_eq!(actor.provisioned, Some(2));
            let count = actor.grant_creates;
            actor.provision().await;
            assert_eq!(
                actor.grant_creates, count,
                "uncertain ACK must retain hold/floor"
            );
            actor.shutdown().await;
        });
    }

    #[test]
    fn gap_storms_cannot_reset_live_child_mint_floor() {
        let broker = Broker::start();
        runtime().block_on(async {
            let mut actor = test_actor(&broker);
            actor.connect().await;
            let start = Instant::now();
            actor.provision_at(start, clock_ms().unwrap()).await;
            let (_pane, fd) = test_handle(&actor).prepare(1).unwrap();
            drop(fd);
            for round in 1..=3 {
                let after = actor.children[&1].retry.mint_after.unwrap();
                expire(&mut actor, 1, after - GRANT_LIFETIME).await;
                for _ in 0..32 {
                    actor.reconcile().await;
                }
                assert_eq!(actor.grant_creates, round as usize);
                actor.retry_due(after - Duration::from_millis(1)).await;
                assert_eq!(actor.grant_creates, round as usize);
                actor.retry_due(after).await;
                assert_eq!(actor.grant_creates, round as usize + 1);
            }
            actor.shutdown().await;
        });
    }

    #[test]
    fn quota_refusal_clears_pool_latch_and_retries_without_a_new_event() {
        let broker = Broker::with_grant_limit(1);
        runtime().block_on(async {
            let mut actor = test_actor(&broker);
            actor.connect().await;
            actor.provision().await;
            let (_pane, fd) = test_handle(&actor).prepare(1).unwrap();
            drop(fd);
            actor.provision().await;
            assert_eq!(actor.provisioned, None);
            assert!(actor.shared.lock().unwrap().ready.is_none());
            let first = actor.children[&1].record.as_ref().unwrap().reference();
            actor
                .connection
                .as_ref()
                .unwrap()
                .session_revoke(first)
                .await
                .unwrap();
            actor.provision().await; // next renew tick, no reconnect / input
            assert_eq!(
                actor.shared.lock().unwrap().ready.as_ref().unwrap().pane.id,
                2
            );
            actor.shutdown().await;
        });
    }

    #[test]
    fn replacement_withdraws_generation_two_bundle_before_await() {
        for consume_during_reconnect in [false, true] {
            let mut broker = Broker::start();
            runtime().block_on(async {
                let mut actor = test_actor(&broker);
                actor.connect().await;
                let start = Instant::now();
                let base = clock_ms().unwrap();
                actor.provision_at(start, base).await;
                expire(&mut actor, 1, start + GRANT_LIFETIME).await;
                let after = actor.children[&1].retry.mint_after.unwrap();
                actor.provision_at(after, base + 60_000).await;
                let after_replacement = actor.children[&1].retry.mint_after.unwrap();
                let (old, old_epoch, key) = {
                    let shared = actor.shared.lock().unwrap();
                    let ready = shared.ready.as_ref().unwrap();
                    let grant = descriptor(&ready.fd);
                    assert_eq!(grant.record.pane_generation, Some(DecimalU64(2)));
                    (
                        ready.pane.clone(),
                        grant.record.broker_epoch,
                        grant.grant.public_key,
                    )
                };
                let handle = test_handle(&actor);
                broker.bounce();
                let paused = broker.pause();
                {
                    let connecting = actor.connect();
                    tokio::pin!(connecting);
                    tokio::select! {
                        _ = &mut connecting => panic!("paused broker should delay reconnect"),
                        _ = tokio::time::sleep(Duration::from_millis(20)) => {},
                    }
                    assert!(handle.1.lock().unwrap().ready.is_none());
                    assert!(!old.live.load(Ordering::Acquire));
                    assert!(!old.launched.load(Ordering::Acquire));
                    if consume_during_reconnect {
                        assert!(handle.prepare(1).is_none());
                    }
                    drop(paused);
                    connecting.await;
                }
                actor.provision_at(after_replacement, base + 120_000).await;
                {
                    let shared = actor.shared.lock().unwrap();
                    let grant = descriptor(&shared.ready.as_ref().unwrap().fd);
                    assert_ne!(grant.record.broker_epoch, old_epoch);
                    assert_eq!(grant.record.pane_generation, Some(DecimalU64(1)));
                    if !consume_during_reconnect {
                        assert_eq!(grant.grant.public_key, key);
                    } else {
                        assert_eq!(grant.record.pane_id, Some(DecimalU64(2)));
                    }
                }
                actor.shutdown().await;
            });
        }
    }

    #[test]
    fn real_uds_resume_retains_parent_and_reconciles_lost_grant_ack() {
        let broker = Broker::start();
        runtime().block_on(async {
            let mut actor = Actor {
                grant_creates: 0,
                faults: TestFaults::default(),
                options: broker.options(),
                url: broker.url.clone(),
                key: fresh_key().unwrap(),
                connection: None,
                parent: None,
                own_lease: None,
                children: HashMap::new(),
                shared: Arc::new(std::sync::Mutex::new(Shared::default())),
                provisioned: None,
                pool_key: None,
                activity: Arc::new(AtomicU64::new(clock_ms().unwrap())),
                startup: None,
            };
            actor.connect().await;
            let parent = actor.parent.clone().unwrap();
            let key = fresh_key().unwrap();
            actor.children.insert(
                1,
                Child {
                    pane: Arc::new(PaneState {
                        id: 1,
                        generation: AtomicU64::new(1),
                        live: AtomicBool::new(true),
                        launched: AtomicBool::new(true),
                        control_ready: AtomicBool::new(false),
                        public_key: HexBytes(key.verifying_key().to_bytes()),
                        binding: std::sync::Mutex::new(None),
                    }),
                    record: None,
                    retry: Retry {
                        external: true,
                        ..Retry::default()
                    },
                    pending: None,
                },
            );
            let grant = actor.grant(1).await.unwrap();
            // Model a committed create whose reply did not reach the supervisor.
            actor.children.get_mut(&1).unwrap().record = None;
            actor.connection.as_ref().unwrap().client().close().await;
            actor.connect().await;
            let resumed = actor.parent.as_ref().unwrap();
            assert_eq!(resumed.record_id, parent.record_id);
            assert_eq!(resumed.incarnation, parent.incarnation);
            assert!(resumed.binding_generation.0 > parent.binding_generation.0);
            let reconciled = actor.grant(1).await.unwrap();
            assert_eq!(reconciled.grant.grant_id, grant.grant.grant_id);
            assert_eq!(reconciled.record.record_id, grant.record.record_id);
            actor.close_child(1).await;
            actor.connection.as_ref().unwrap().client().close().await;
        });
    }

