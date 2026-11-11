// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! MCP identity and provenance over the shared native agent client.

pub(crate) use bus::agent_client::{AGENT_SESSION_ENV, Client};
#[cfg(test)]
use bus::agent_client::{bridge_reply, broker_name, finish_connect, tcp_permission};
#[cfg(test)]
use bus::native_client::{ConnectError, NodedClient, UnixConnectOutcome};

pub(crate) struct BrokerConnection(bus::agent_client::BrokerConnection);

impl BrokerConnection {
    pub(crate) fn new() -> Self {
        let build = buildinfo::build_info!();
        let provenance = bus::RegisterProvenance {
            binary: Some("mix-mcp".into()), version: Some(build.version.into()),
            git_sha: Some(build.git_sha.into()), git_dirty: Some(build.git_dirty),
            build_time: Some(build.build_time.into()), pid: Some(std::process::id()),
            started_at: Some(buildinfo::now_rfc3339()), ..Default::default()
        };
        Self(bus::agent_client::BrokerConnection::for_current_agent(Some(provenance), Some(paths::env::MCP_ALLOW_TCP)))
    }
    pub(crate) fn is_connected(&self) -> bool { self.0.is_connected() }
    pub(crate) async fn get(&self) -> Result<std::sync::Arc<Client>, String> { self.0.get().await }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// noded's rule: 2-31 bytes, lowercase first, then lowercase/digit/'-'.
    fn noded_accepts(name: &str) -> bool {
        (2..=31).contains(&name.len())
            && name.as_bytes()[0].is_ascii_lowercase()
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    }

    #[test]
    fn agent_session_names_carry_the_session_and_fit_noded() {
        let name = broker_name(Some("k3x9q2ab7m1z"), 4_194_304, 0x1_beef).unwrap();
        assert_eq!(name, "mcp-s-k3x9q2ab7m1z-400000beef");
        assert!(noded_accepts(&name), "{name}");
        assert!(name.starts_with("mcp-s-k3x9q2ab7m1z-"));
        // The longest session with the largest Linux pid still fits.
        let name = broker_name(Some("zzzzzzzzzzzz"), 4_194_304, u128::MAX).unwrap();
        assert!(noded_accepts(&name), "{name}");
        // The same pid in another PID namespace still gets a different name.
        assert_ne!(
            broker_name(Some("abcd"), 1, 0x0001).unwrap(),
            broker_name(Some("abcd"), 1, 0x0002).unwrap()
        );
        let anonymous = broker_name(None, 1234, 0xabc).unwrap();
        assert_eq!(anonymous, "mcp-1234-abc");
        assert!(noded_accepts(&anonymous));
    }

    #[test]
    fn a_session_prefix_never_matches_a_longer_session() {
        let longer = broker_name(Some("abcd1"), 9, 0).unwrap();
        assert!(!longer.starts_with("mcp-s-abcd-"), "{longer}");
    }

    #[test]
    fn invalid_agent_sessions_refuse_the_bus_instead_of_going_anonymous() {
        for bad in [
            "abc",
            "toolongsession1",
            "Upper1",
            "has-dash",
            "a.b.c.d",
            "sp ace",
            "",
            "abcd\u{fffd}",
        ] {
            let error = broker_name(Some(bad), 77, 0x1f).unwrap_err();
            assert!(error.contains(AGENT_SESSION_ENV), "{bad}: {error}");
        }
    }

    #[tokio::test]
    async fn an_invalid_session_fails_every_bus_connection() {
        let connection = BrokerConnection(bus::agent_client::BrokerConnection::new(broker_name(Some("BAD"), 1, 0), None, None));
        let error = connection.get().await.err().expect("must refuse");
        assert!(error.contains("refusing to connect"), "{error}");
    }

    #[test]
    fn bridge_answers_protocol_commands_and_refuses_the_rest() {
        for command in ["HELP", "app.describe"] {
            let (rc, body) = bridge_reply(command);
            assert_eq!(rc, 0);
            let list: Vec<bus::CommandDescriptor> = serde_json::from_str(&body).unwrap();
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].name, "HELP");
            assert!(list[0].read_only);
        }
        let (rc, body) = bridge_reply("mcp.anything");
        assert_eq!(rc, 10);
        let refusal: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(refusal["error_code"], "UNKNOWN_COMMAND");
    }

    #[tokio::test]
    async fn missing_candidates_with_opt_in_connect_over_tcp() {
        let broker = test_broker::Broker::start();
        let mut options = broker.options();
        options.endpoint = None;
        options.discover_by_ping = false;
        let result = NodedClient::connect_unix("mcp-tcp-test", &broker.url, &options, None).await;
        assert!(matches!(result, Err(ConnectError::CandidatesMissing(_))));
        let client = finish_connect("mcp-tcp-test", &broker.url, None, Some("1"), result)
            .await
            .ok()
            .unwrap();
        assert!(matches!(client, Client::Tcp(_)));
        assert_eq!(
            client
                .call("noded", "noded.ping", serde_json::Value::Null)
                .await
                .unwrap()["pong"],
            true
        );
        client.close().await;
    }

    #[tokio::test]
    async fn existing_failed_socket_never_takes_tcp_even_with_opt_in() {
        let broker = test_broker::Broker::start();
        let directory = tempfile::tempdir().unwrap();
        let endpoint = directory.path().canonicalize().unwrap().join("stale.sock");
        drop(std::os::unix::net::UnixListener::bind(&endpoint).unwrap());
        let mut options = broker.options();
        options.endpoint = Some(endpoint.clone());
        options.allow_unverified_tcp_fallback = true;
        let result =
            NodedClient::connect_unix("mcp-failed-test", &broker.url, &options, None).await;
        // Only CandidatesMissing may take the TCP path; an existing socket
        // that fails must report its own Unix error. The OS text varies under
        // load (refused, or a timeout), so assert the path, not the wording
        // (2 of 91 release gates failed on "refused", 2026-11-02).
        assert!(
            !matches!(result, Err(ConnectError::CandidatesMissing(_))),
            "an existing socket was treated as missing"
        );
        let error = finish_connect("mcp-failed-test", &broker.url, None, Some("1"), result)
            .await
            .err()
            .unwrap();
        assert!(
            error.contains(&endpoint.display().to_string()),
            "error does not name the socket: {error}"
        );
        assert!(
            !error.to_lowercase().contains("tcp"),
            "a failed socket fell back to TCP: {error}"
        );
    }

    #[test]
    fn tcp_requires_explicit_opt_in_and_reports_candidates() {
        let paths = vec![
            "/tmp/example/first.sock".into(),
            "/tmp/example/second.sock".into(),
        ];
        for value in [None, Some(""), Some("0"), Some("true")] {
            let error = tcp_permission(value, paths.clone()).unwrap_err();
            assert!(error.contains("TCP opt-in setting =1"));
            for path in &paths {
                assert!(error.contains(&path.display().to_string()));
            }
        }
        assert!(tcp_permission(Some("1"), paths).is_ok());
    }
}
