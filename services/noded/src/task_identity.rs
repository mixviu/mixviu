// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-process capabilities for the two broker tasks. Never implement Debug.
use rand::RngCore;

#[derive(Clone)]
pub(crate) struct Token([u8; 32]);

impl Token {
    pub(crate) fn new() -> Self {
        let mut bytes = [0; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    fn encoded(&self) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(self.0)
    }

    pub(super) fn registration_body(&self) -> String {
        serde_json::json!({"broker_task_token": self.encoded()}).to_string()
    }

    pub(crate) fn matches(&self, value: &serde_json::Value) -> bool {
        use base64::Engine;
        let Some(value) = value.as_str() else {
            return false;
        };
        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(value) else {
            return false;
        };
        let Ok(bytes) = <[u8; 32]>::try_from(bytes) else {
            return false;
        };
        constant_time_eq::constant_time_eq_32(&self.0, &bytes)
    }

    pub(crate) async fn connect(
        &self,
        name: &str,
        url: &str,
    ) -> anyhow::Result<bus::native_client::NodedClient> {
        let client = bus::native_client::NodedClient::connect_anonymous(url).await?;
        if let Err(error) = client
            .register_as_with_body(name, &self.registration_body())
            .await
        {
            client.close().await;
            return Err(error);
        }
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn real_task_connections_require_the_matching_enabled_capability() {
        #[derive(Clone)]
        struct Capture(Arc<Mutex<Vec<u8>>>);
        impl std::io::Write for Capture {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        // This current-thread runtime keeps broker, client and transport tasks
        // under the same capturing subscriber, including log's trace records.
        let _ = tracing_log::LogTracer::init();
        log::set_max_level(log::LevelFilter::Trace);
        let output = Capture(Arc::new(Mutex::new(Vec::new())));
        let writer = output.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_ansi(false)
            .without_time()
            .with_writer(move || writer.clone())
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);
        let logger = Token::new();
        let monitor = Token::new();
        let wrong = Token::new();
        let url = crate::noded::tests::spawn_task_broker(TaskIdentities {
            logger: Some(logger.clone()),
            monitor: Some(monitor.clone()),
        })
        .await
        .unwrap();
        for (name, token) in [
            ("log", &logger),
            ("log-observe-123", &logger),
            ("mon", &monitor),
        ] {
            assert!(wrong.connect(name, &url).await.is_err());
            assert!(
                bus::native_client::NodedClient::connect(name, &url)
                    .await
                    .is_err()
            );
            let client = token.connect(name, &url).await.unwrap();
            assert!(
                client
                    .call("noded", "noded.ping", serde_json::json!({}))
                    .await
                    .is_ok()
            );
            client.close().await;
        }
        let disabled = crate::noded::tests::spawn_task_broker(TaskIdentities::default())
            .await
            .unwrap();
        assert!(logger.connect("log", &disabled).await.is_err());
        assert!(monitor.connect("mon", &disabled).await.is_err());
        let client = bus::native_client::NodedClient::connect_anonymous(&url)
            .await
            .unwrap();
        let previous = client.name();
        assert!(client.register_as_with_body("log", "{}").await.is_err());
        assert_eq!(client.name(), previous);
        let marker = format!("registration-marker-{:016x}", rand::random::<u64>());
        let mut body: serde_json::Value = serde_json::from_str(&logger.registration_body()).unwrap();
        body["binary"] = serde_json::json!(marker);
        client.register_as_with_body("trace-probe", &body.to_string()).await.unwrap();
        client.close().await;
        tokio::task::yield_now().await;
        let logs = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
        assert!(
            logs.contains("Service 'mon' registered"),
            "capture did not cover broker registration"
        );
        assert!(
            logs.contains("TRACE"),
            "trace transport logging was not captured"
        );
        assert!(logs.lines().any(|line| line.contains("TRACE") && line.contains(&marker)), "registration body marker was not traced");
        for token in [&logger, &monitor, &wrong] {
            assert!(
                !logs.contains(&token.encoded()),
                "capability reached a log record"
            );
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct TaskIdentities {
    pub(crate) logger: Option<Token>,
    pub(crate) monitor: Option<Token>,
}
