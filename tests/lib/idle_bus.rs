// SPDX-License-Identifier: MIT OR Apache-2.0
// Real broker fixtures shared by the shell and compositor idle-traffic tests.
use bus::SupervisedClient;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}},
    time::{Duration, Instant},
};

pub async fn connect(broker: &test_broker::Broker, name: &str) -> SupervisedClient {
    SupervisedClient::connect_options(name, &broker.url)
        .connect()
        .await
        .unwrap()
}

pub fn snapshot(revision: u64, incarnation: &str) -> settings::Snapshot {
    let values = settings::registry::Values::from([("window.gaps".into(), json!(revision * 4))]);
    settings::Snapshot {
        schema: settings::SCHEMA,
        binding: settings::session::binding().unwrap(),
        incarnation: incarnation.into(),
        revision: settings::Revision(revision),
        design_revision: settings::Revision(1),
        source_digest: settings::source_digest(settings::EMBEDDED_DEFAULT_SOURCE),
        effective: settings::resolve::values::resolve(&values, settings::EMBEDDED_DEFAULT_SOURCE)
            .unwrap(),
        desktop: settings::Desktop::default(),
        values,
    }
}

pub struct Service {
    pub client: Arc<SupervisedClient>,
    pub calls: Arc<Mutex<Vec<(String, Instant)>>>,
    pub fail_reads: Arc<AtomicUsize>,
    current: Arc<Mutex<settings::Snapshot>>,
    task: tokio::task::JoinHandle<()>,
}
impl Service {
    pub async fn start(broker: &test_broker::Broker, name: &str) -> Self {
        let client = Arc::new(connect(broker, name).await);
        let mut incoming = client.incoming().unwrap();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let captured = calls.clone();
        let current = Arc::new(Mutex::new(snapshot(1, "first")));
        let answer = current.clone();
        let fail_reads = Arc::new(AtomicUsize::new(0));
        let failures = fail_reads.clone();
        let service = client.clone();
        let task = tokio::spawn(async move {
            while let Some(command) = incoming.recv().await {
                if command.is_topic_delivery() {
                    continue;
                }
                captured.lock().unwrap().push((command.command.clone(), Instant::now()));
                let refused = command.command == "settings.get" && failures.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1)).is_ok();
                let value = if refused { json!({"status":"busy"}) } else if command.command == "settings.get" {
                    let snapshot = answer.lock().unwrap().clone();
                    json!({"status":"current", "snapshot":settings::compact::encode_values(&snapshot, &snapshot.values).unwrap()})
                } else {
                    json!({})
                };
                service
                    .respond_parts(
                        command.generation,
                        &command.from,
                        &command.command,
                        command.id.as_deref(),
                        if refused { 10 } else { 0 },
                        &value.to_string(),
                    )
                    .await
                    .unwrap();
            }
        });
        Self {
            client,
            calls,
            current,
            fail_reads,
            task,
        }
    }
    pub fn count(&self, command: &str) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|(c, _)| c.as_str() == command)
            .count()
    }
    pub fn assert_read_retry_deadline(&self) {
        let calls = self.calls.lock().unwrap();
        let reads: Vec<_> = calls.iter().filter(|(c,_)| c == "settings.get").map(|(_,at)| *at).collect();
        assert!(reads[2].duration_since(reads[1]) >= Duration::from_millis(240), "failed read retried before its deadline");
    }
    pub async fn wait(&self, command: &str, count: usize) {
        tokio::time::timeout(Duration::from_secs(8), async {
            while self.count(command) < count {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
    }
    pub async fn publish(&self, topic: &str, body: Value) {
        let headers = BTreeMap::from([
            ("name".into(), topic.into()),
            ("retain".into(), "false".into()),
        ]);
        self.client
            .call_with_headers_raw("noded", "topic.publish", &headers, &body.to_string())
            .await
            .unwrap();
    }
    pub async fn settings_changed(&self, revision: u64, incarnation: &str) {
        let snapshot = snapshot(revision, incarnation);
        *self.current.lock().unwrap() = snapshot.clone();
        self.publish(
            &settings::topics::snapshot_topic(&snapshot.binding.profile),
            serde_json::to_value(
                settings::compact::encode_values(&snapshot, &snapshot.values).unwrap(),
            )
            .unwrap(),
        )
        .await;
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        self.task.abort();
    }
}
