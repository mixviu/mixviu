// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Webd-only credential overlay; the shared node config reader stays unchanged.
use anyhow::{Context, Result, bail};
use config::node::WebdConfig;
use serde::Deserialize;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

const MAX_CREDENTIAL_BYTES: u64 = 64 * 1024;

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
    if !directory.is_absolute() || !directory.is_dir() {
        bail!("CREDENTIALS_DIRECTORY must be an absolute path to an existing directory");
    }
    let path = directory.join("webd.vhost-secrets");
    let metadata = match std::fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).context("inspecting webd.vhost-secrets credential"),
    };
    if !metadata.file_type().is_file() {
        bail!("webd.vhost-secrets credential must be a regular file (no symlinks)");
    }
    // Prevent a replacement between inspection and open from following a link
    // or blocking on a FIFO; recheck the opened descriptor before reading.
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .context("opening webd.vhost-secrets credential")?;
    if !file.metadata().context("inspecting opened webd.vhost-secrets credential")?.is_file() {
        bail!("webd.vhost-secrets credential must be a regular file");
    }
    let mut bytes = Vec::new();
    file.take(MAX_CREDENTIAL_BYTES + 1).read_to_end(&mut bytes)
        .context("reading webd.vhost-secrets credential")?;
    if bytes.len() as u64 > MAX_CREDENTIAL_BYTES {
        bail!("webd.vhost-secrets credential exceeds 64 KiB");
    }
    let source = std::str::from_utf8(&bytes)
        .map_err(|_| anyhow::anyhow!("webd.vhost-secrets credential must be UTF-8"))?;
    if source.trim().is_empty() {
        return Ok(None);
    }
    // Parser/serde diagnostics may contain source snippets or invalid values.
    // Keep those errors out of the startup log, including their error chain.
    let secrets: Secrets = config::from_conf_mix_str(source)
        .map_err(|_| anyhow::anyhow!("invalid webd.vhost-secrets credential: expected strict-data {{vhost: [{{name, optional password fields}}]}}"))?;
    if secrets.vhost.is_empty() {
        return Ok(None);
    }
    let mut rows = config.as_ref().map(|c| c.vhost.clone()).unwrap_or_default();
    let mut seen = std::collections::BTreeSet::new();
    let mut applied = Vec::new();
    for secret in secrets.vhost {
        if !seen.insert(secret.name.clone()) {
            bail!("webd.vhost-secrets credential contains a duplicate vhost name");
        }
        let Some(row) = rows.iter_mut().find(|r| r.host == secret.name) else {
            bail!("webd.vhost-secrets credential names a vhost absent from node config");
        };
        let mut fields = Vec::new();
        for (field, value, target, email) in [
            (
                "dev_session_password",
                secret.dev_session_password,
                &mut row.dev_session_password,
                &row.dev_session_email,
            ),
            (
                "public_read_password",
                secret.public_read_password,
                &mut row.public_read_password,
                &row.public_read_email,
            ),
            (
                "system_sender_password",
                secret.system_sender_password,
                &mut row.system_sender_password,
                &row.system_sender_email,
            ),
        ] {
            if let Some(value) = value {
                if email.is_none() {
                    bail!("webd.vhost-secrets: {}: set both {}_email and {field}, or neither", row.host, field.trim_end_matches("_password"));
                }
                *target = Some(value);
                fields.push(field);
            }
        }
        if !fields.is_empty() {
            applied.push(format!("{}: {}", row.host, fields.join(", ")));
        }
    }
    if applied.is_empty() {
        return Ok(None);
    }
    if let Some(config) = config {
        config.vhost = rows;
    }
    Ok(Some(format!(
        "webd.vhost-secrets: {} vhosts took secrets from credential [{}]",
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
        std::fs::write(dir.path().join("webd.vhost-secrets"), source).unwrap();
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
            "webd.vhost-secrets: 1 vhosts took secrets from credential [example.com: dev_session_password, public_read_password, system_sender_password]"
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
            assert!(merge(Some(&mut cfg), Some(empty.path())).unwrap().is_none());
        }
    }

    #[test]
    fn absent_credential_preserves_old_behaviour() {
        let mut cfg = config();
        let before = format!("{cfg:?}");
        let dir = tempfile::tempdir().unwrap();
        assert!(merge(Some(&mut cfg), None).unwrap().is_none());
        assert!(merge(Some(&mut cfg), Some(dir.path())).unwrap().is_none());
        assert_eq!(before, format!("{cfg:?}"));
        assert_eq!(
            cfg.vhost[0].public_read_password.as_deref(),
            Some("old-marker")
        );
    }

    #[test]
    fn empty_fallback_preserves_unmigrated_half_pairs() {
        for source in ["", " \n\t", "{vhost: []}"] {
            let mut cfg = config(); // dev and sender have an email but no password.
            let before = serde_json::to_value(&cfg).unwrap();
            assert!(merge(Some(&mut cfg), None).unwrap().is_none());
            let dir = credential(source);
            assert!(merge(Some(&mut cfg), Some(dir.path())).unwrap().is_none());
            assert_eq!(serde_json::to_value(&cfg).unwrap(), before);
            assert_eq!(cfg.vhost[0].public_read_password.as_deref(), Some("old-marker"));
            assert!(cfg.vhost[0].dev_session_password.is_none());
            assert!(cfg.vhost[0].system_sender_password.is_none());
        }
    }

    #[test]
    fn overlay_checks_only_supplied_fields_and_rows() {
        let mut cfg = config();
        cfg.vhost.push(WebdVhostConfig { host: "other.example.com".into(), public_read_email: Some("reader@other.example.com".into()), ..Default::default() });
        let dir = credential(&format!(r#"{{vhost: [{{name: "example.com", public_read_password: "{SECRET}"}}]}}"#));
        merge(Some(&mut cfg), Some(dir.path())).unwrap();
        assert_eq!(cfg.vhost[0].public_read_password.as_deref(), Some(SECRET));
        assert!(cfg.vhost[0].dev_session_password.is_none());
        assert!(cfg.vhost[1].public_read_password.is_none());
    }

    #[test]
    fn credential_directory_must_be_absolute_and_exist() {
        let dir = credential("{vhost: []}");
        for path in [Path::new(""), Path::new("relative"), &dir.path().join("missing"), &dir.path().join("webd.vhost-secrets")] {
            let error = merge(Some(&mut config()), Some(path)).unwrap_err().to_string();
            assert!(error.contains("CREDENTIALS_DIRECTORY must be an absolute path to an existing directory"));
        }
    }

    #[test]
    fn rejects_nonregular_files_without_reading_them() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("webd.vhost-secrets");
        symlink("/dev/zero", &path).unwrap();
        assert!(merge(None, Some(dir.path())).unwrap_err().to_string().contains("regular file"));
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(merge(None, Some(dir.path())).unwrap_err().to_string().contains("regular file"));
        std::fs::remove_dir(&path).unwrap();
        let cpath = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
        // SAFETY: cpath is NUL-terminated and remains valid for this call.
        assert_eq!(unsafe { libc::mkfifo(cpath.as_ptr(), 0o600) }, 0);
        assert!(merge(None, Some(dir.path())).unwrap_err().to_string().contains("regular file"));
    }

    #[test]
    fn bounded_read_accepts_limit_and_rejects_extra_byte_and_invalid_utf8() {
        let mut source = "{vhost: []}".to_string();
        source.push_str(&" ".repeat(MAX_CREDENTIAL_BYTES as usize - source.len()));
        let dir = credential(&source);
        assert!(merge(None, Some(dir.path())).unwrap().is_none());
        source.push(' ');
        std::fs::write(dir.path().join("webd.vhost-secrets"), &source).unwrap();
        let error = merge(None, Some(dir.path())).unwrap_err().to_string();
        assert_eq!(error, "webd.vhost-secrets credential exceeds 64 KiB");
        std::fs::write(dir.path().join("webd.vhost-secrets"), [0xff]).unwrap();
        assert_eq!(merge(None, Some(dir.path())).unwrap_err().to_string(), "webd.vhost-secrets credential must be UTF-8");
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
            assert!(error.contains("invalid webd.vhost-secrets"));
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
