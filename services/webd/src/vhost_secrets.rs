// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Webd-only credential overlay; the shared node config reader stays unchanged.
use anyhow::{Context, Result, bail};
use config::node::WebdConfig;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Secrets {
    vhost: Vec<VhostSecrets>,
}

// Deliberately no Debug or Serialize implementation for the credential DTO.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VhostSecrets {
    name: String,
    dev_session_password: Option<String>,
    public_read_password: Option<String>,
    system_sender_password: Option<String>,
}

/// Return only field metadata, never credential values. Accept an explicit
/// directory so tests do not mutate the process environment.
fn merge(config: Option<&mut WebdConfig>, directory: Option<&Path>) -> Result<Option<String>> {
    let Some(directory) = directory else {
        return Ok(None);
    };
    let source = match std::fs::read_to_string(directory.join("webd-vhost-secrets")) {
        Ok(source) => source,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).context("reading webd-vhost-secrets credential"),
    };
    // Parser/serde diagnostics may contain source snippets or invalid values.
    // Keep those errors out of the startup log, including their error chain.
    let secrets: Secrets = config::from_conf_mix_str(&source)
        .map_err(|_| anyhow::anyhow!("invalid webd-vhost-secrets credential: expected strict-data {{vhost: [{{name, optional password fields}}]}}"))?;
    let mut rows = config.as_ref().map(|c| c.vhost.clone()).unwrap_or_default();
    let mut seen = std::collections::BTreeSet::new();
    let mut applied = Vec::new();
    for secret in secrets.vhost {
        if !seen.insert(secret.name.clone()) {
            bail!("webd-vhost-secrets credential contains a duplicate vhost name");
        }
        let Some(row) = rows.iter_mut().find(|r| r.host == secret.name) else {
            bail!("webd-vhost-secrets credential names a vhost absent from node config");
        };
        let mut fields = Vec::new();
        for (field, value, target) in [
            (
                "dev_session_password",
                secret.dev_session_password,
                &mut row.dev_session_password,
            ),
            (
                "public_read_password",
                secret.public_read_password,
                &mut row.public_read_password,
            ),
            (
                "system_sender_password",
                secret.system_sender_password,
                &mut row.system_sender_password,
            ),
        ] {
            if let Some(value) = value {
                *target = Some(value);
                fields.push(field);
            }
        }
        if !fields.is_empty() {
            applied.push(format!("{}: {}", row.host, fields.join(", ")));
        }
    }
    for row in &rows {
        for (field, email, password) in [
            (
                "dev_session",
                &row.dev_session_email,
                &row.dev_session_password,
            ),
            (
                "public_read",
                &row.public_read_email,
                &row.public_read_password,
            ),
            (
                "system_sender",
                &row.system_sender_email,
                &row.system_sender_password,
            ),
        ] {
            if email.is_some() != password.is_some() {
                bail!(
                    "webd-vhost-secrets: {}: set both {field}_email and {field}_password, or neither",
                    row.host
                );
            }
        }
    }
    if let Some(config) = config {
        config.vhost = rows;
    }
    Ok(Some(format!(
        "webd-vhost-secrets: {} vhosts took secrets from credential [{}]",
        applied.len(),
        applied.join("; ")
    )))
}

pub(crate) fn load(config: Option<&mut WebdConfig>, directory: Option<&Path>) -> Result<()> {
    if let Some(summary) = merge(config, directory)? {
        tracing::info!("{summary}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::node::WebdVhostConfig;

    const SECRET: &str = "credential-secret-marker";

    fn config() -> WebdConfig {
        WebdConfig {
            vhost: vec![WebdVhostConfig {
                host: "example.com".into(),
                dev_session_email: Some("dev@example.com".into()),
                public_read_email: Some("reader@example.com".into()),
                public_read_password: Some("old-marker".into()),
                system_sender_email: Some("sender@example.com".into()),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn credential(source: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("webd-vhost-secrets"), source).unwrap();
        dir
    }

    #[test]
    fn fills_overrides_redacts_and_reports_fields_only() {
        let mut cfg = config();
        let dir = credential(&format!(
            r#"{{vhost: [{{name: "example.com", dev_session_password: "{SECRET}", public_read_password: "{SECRET}", system_sender_password: "{SECRET}"}}]}}"#
        ));
        let summary = merge(Some(&mut cfg), Some(dir.path())).unwrap().unwrap();
        let row = &cfg.vhost[0];
        assert_eq!(row.dev_session_password.as_deref(), Some(SECRET));
        assert_eq!(row.public_read_password.as_deref(), Some(SECRET));
        assert_eq!(row.system_sender_password.as_deref(), Some(SECRET));
        assert_eq!(
            summary,
            "webd-vhost-secrets: 1 vhosts took secrets from credential [example.com: dev_session_password, public_read_password, system_sender_password]"
        );
        for output in [
            summary,
            format!("{cfg:?}"),
            serde_json::to_string(&cfg).unwrap(),
            config::to_conf_mix_string(&cfg).unwrap(),
        ] {
            assert!(!output.contains(SECRET));
            assert!(!output.contains("old-marker"));
        }
    }

    #[test]
    fn unknown_and_duplicate_names_refused_without_partial_update() {
        for names in [
            r#""example.com", "unknown.example.com""#,
            r#""example.com", "example.com""#,
        ] {
            let rows = names
                .split(", ")
                .map(|name| format!("{{name: {name}, public_read_password: \"{SECRET}\"}}"))
                .collect::<Vec<_>>()
                .join(",");
            let dir = credential(&format!("{{vhost: [{rows}]}}"));
            let mut cfg = config();
            assert!(merge(Some(&mut cfg), Some(dir.path())).is_err());
            assert_eq!(
                cfg.vhost[0].public_read_password.as_deref(),
                Some("old-marker")
            );
        }
    }

    #[test]
    fn pairing_checked_after_merge_for_each_identity() {
        for field in ["dev_session", "public_read", "system_sender"] {
            let dir = credential(&format!(
                r#"{{vhost: [{{name: "example.com", {field}_password: "{SECRET}"}}]}}"#
            ));
            let mut cfg = WebdConfig {
                vhost: vec![WebdVhostConfig {
                    host: "example.com".into(),
                    ..Default::default()
                }],
                ..Default::default()
            };
            let error = merge(Some(&mut cfg), Some(dir.path()))
                .unwrap_err()
                .to_string();
            assert!(error.contains(&format!("both {field}_email and {field}_password")));
            assert!(!error.contains(SECRET));
            cfg.vhost[0].public_read_email = Some("reader@example.com".into());
            let empty = credential("{vhost: []}");
            assert!(merge(Some(&mut cfg), Some(empty.path())).is_err());
        }
    }

    #[test]
    fn absent_credential_preserves_old_behaviour() {
        let mut cfg = config();
        let before = format!("{cfg:?}");
        let dir = tempfile::tempdir().unwrap();
        assert!(merge(Some(&mut cfg), None).unwrap().is_none());
        assert!(merge(Some(&mut cfg), Some(dir.path())).unwrap().is_none());
        assert!(
            merge(Some(&mut cfg), Some(&dir.path().join("missing")))
                .unwrap()
                .is_none()
        );
        assert_eq!(before, format!("{cfg:?}"));
        assert_eq!(
            cfg.vhost[0].public_read_password.as_deref(),
            Some("old-marker")
        );
    }

    #[test]
    fn malformed_credential_errors_do_not_echo_values() {
        for source in [
            format!("{{vhost: {SECRET}}}"),
            format!("{{vhost: [{{name: \"example.com\", public_read_password: [{SECRET}]}}]}}"),
            format!("{{vhost: [], {SECRET}: true}}"),
            "print(1)".into(),
        ] {
            let dir = credential(&source);
            let error = format!(
                "{:#}",
                merge(Some(&mut config()), Some(dir.path())).unwrap_err()
            );
            assert!(error.contains("invalid webd-vhost-secrets"));
            assert!(!error.contains(SECRET));
        }
    }

    #[test]
    fn startup_logs_once_without_values() {
        use std::io::Write;
        use std::sync::{Arc, Mutex};
        struct Writer(Arc<Mutex<Vec<u8>>>);
        impl Write for Writer {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let output = Arc::new(Mutex::new(Vec::new()));
        let capture = output.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(move || Writer(capture.clone()))
            .finish();
        let dir = credential(&format!(
            r#"{{vhost: [{{name: "example.com", public_read_password: "{SECRET}"}}]}}"#
        ));
        let mut cfg = WebdConfig {
            vhost: vec![WebdVhostConfig {
                host: "example.com".into(),
                public_read_email: Some("reader@example.com".into()),
                ..Default::default()
            }],
            ..Default::default()
        };
        tracing::subscriber::with_default(subscriber, || {
            load(Some(&mut cfg), Some(dir.path())).unwrap()
        });
        let log = String::from_utf8(output.lock().unwrap().clone()).unwrap();
        assert_eq!(log.lines().count(), 1);
        assert!(
            log.contains(
                "1 vhosts took secrets from credential [example.com: public_read_password]"
            )
        );
        assert!(!log.contains(SECRET));
    }
}
