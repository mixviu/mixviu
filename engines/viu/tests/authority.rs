// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Real broker admission and attestation for compositor-owned furniture events.
use std::{collections::BTreeMap, time::Duration};

#[tokio::test]
async fn furniture_events_refuse_foreign_publishers_and_attest_the_compositor() {
    let broker = test_broker::Broker::start();
    let owner = bus::SupervisedClient::connect_options("comp-fixture", &broker.url).connect().await.unwrap();
    let attacker = bus::SupervisedClient::connect_options("foreign", &broker.url).connect().await.unwrap();
    let client = bus::SupervisedClient::connect_options("furniture", &broker.url).connect().await.unwrap();
    let mut incoming = client.incoming().unwrap();
    for suffix in ["corner.input","corner.entered","corner.left","corner.clicked","corner.clicked.v2","panel.command","chrome.action.hover"] {
        let topic = format!("comp-fixture.{suffix}");
        client.subscribe_topic(&topic).await.unwrap();
        let headers = BTreeMap::from([("name".into(),topic.clone()),("retain".into(),"false".into())]);
        let (rc, _) = attacker.call_with_headers_raw("noded","topic.publish",&headers,"{\"phase\":\"forged\"}").await.unwrap();
        assert_ne!(rc, 0, "foreign publisher admitted on {topic}");
        let (rc, _) = owner.call_with_headers_raw("noded","topic.publish",&headers,"{\"phase\":\"consumed\"}").await.unwrap();
        assert_eq!(rc,0,"owner refused on {topic}");
        let delivered = tokio::time::timeout(Duration::from_secs(2),incoming.recv()).await.unwrap().unwrap();
        assert_eq!(delivered.topic(),Some(topic.as_str()));
        assert_eq!(delivered.header("broker_service"),Some("comp-fixture"));
        assert_eq!(delivered.header("broker_origin"),Some("local"));
        assert_eq!(serde_json::from_str::<serde_json::Value>(&delivered.body).unwrap()["phase"],"consumed");
    }
    client.close().await;
    attacker.close().await;
    owner.close().await;
}
