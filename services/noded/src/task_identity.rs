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
        // BLAKE3 Hash equality uses constant_time_eq for its fixed 32 bytes.
        blake3::Hash::from_bytes(self.0) == blake3::Hash::from_bytes(bytes)
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

    #[tokio::test]
    async fn real_task_connections_require_the_matching_enabled_capability() {
        let logger = Token::new();
        let monitor = Token::new();
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
            assert!(Token::new().connect(name, &url).await.is_err());
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
    }
}

#[derive(Clone, Default)]
pub(crate) struct TaskIdentities {
    pub(crate) logger: Option<Token>,
    pub(crate) monitor: Option<Token>,
}
