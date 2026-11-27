// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Where the broker is.
//!
//! Resolution order: `MIX_NODED_URL`; `MIX_BUS_SOCKET`; the node configuration file
//! (the shared `config::node::NodeConfigSearch`, also used for Unix socket
//! and account settings); and finally the local Unix broker. Explicit TCP
//! URLs are legacy and grant no node-local authority.

use std::path::PathBuf;

use strict::Value;

/// Legacy TCP endpoint for callers that explicitly request it.
/// Local discovery uses the runtime Unix socket instead.
pub const DEFAULT_NODED_URL: &str = "ws://127.0.0.1:4200/ws";

const URL_VAR: &str = "MIX_NODED_URL";
const DEFAULT_PORT: u16 = 4200;

/// The broker WebSocket URL for this node.
pub fn noded_url() -> String {
    noded_url_with_config(None)
}

/// Local discovery with an optional explicit node-config path. Environment
/// endpoint overrides retain precedence; no process environment is mutated.
pub fn noded_url_with_config(node_config: Option<&std::path::Path>) -> String {
    if let Some(url) = std::env::var(URL_VAR)
        .ok()
        .filter(|url| !url.trim().is_empty())
    {
        return url;
    }
    if let Ok(path) = std::env::var(paths::env::BUS_SOCKET)
        && std::path::Path::new(&path).is_absolute()
    {
        return format!("unix://{path}");
    }
    let source = match node_config {
        Some(path) => std::fs::read_to_string(path)
            .map(|source| Some((path.to_owned(), source)))
            .map_err(anyhow::Error::from),
        None => config::node::NodeConfigSearch::capture().read(),
    };
    let (path, source) = match source {
        Ok(Some(source)) => source,
        Ok(None) => return default_local_url(),
        Err(e) => {
            tracing::warn!(
                error = %e,
                "failed to read node config; using local Unix endpoint"
            );
            return default_local_url();
        }
    };
    match strict::parse(&source) {
        Ok(config) => match config
            .get("noded")
            .and_then(|n| n.get("unix_socket"))
            .and_then(Value::as_str)
            .filter(|path| std::path::Path::new(path).is_absolute())
            .map(|path| format!("unix://{path}"))
        {
            Some(url) => {
                tracing::debug!(source = %path.display(), url = %url, "broker URL from node config");
                url
            }
            None => {
                tracing::warn!(
                    source = %path.display(),
                    "node config has no valid absolute Unix socket; using local runtime endpoint"
                );
                default_local_url()
            }
        },
        Err(e) => {
            tracing::warn!(
                source = %path.display(),
                error = %e,
                "node config is not valid strict data; using local Unix endpoint"
            );
            default_local_url()
        }
    }
}

fn default_local_url() -> String {
    format!(
        "unix://{}",
        config::path(config::Dir::Run)
            .join("noded/bus.sock")
            .display()
    )
}

/// The node configuration file to read, or `None` when no candidate exists.
/// `MIX_NODE_CONFIG` names it outright (whether or not it exists, so a
/// misconfiguration is reported rather than silently skipped).
pub fn node_config_path() -> Option<PathBuf> {
    config::node::NodeConfigSearch::capture()
        .path()
        .unwrap_or_else(|error| {
            tracing::warn!(%error, "failed to locate node config");
            None
        })
}

/// The search order, the user directory ahead of the system one. When
/// `MIX_ETC` is set it is the only candidate: the caller asked for an
/// isolated tree (a test, a chroot, an alternate install) and must not fall
/// through to the host's `<system etc>`.
#[cfg(test)]
fn candidate_paths(
    etc: Option<PathBuf>,
    xdg_config_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Vec<PathBuf> {
    let environment = config::Environment {
        etc,
        xdg_config_home,
        home,
        uid: 1000,
        ..Default::default()
    };
    let dirs = config::Dirs::resolve(&environment);
    config::node::NodeConfigSearch::from_environment(None, environment)
        .candidate_paths_with_dirs(&dirs)
        .unwrap()
}

/// An endpoint explicitly described by a node configuration. A configured
/// `noded.unix_socket` takes precedence. Older configurations describe legacy
/// TCP as `ws://{mesh_addr}:{port}/ws`, with loopback for a missing or empty
/// `mesh_addr` and 4200 for a missing `noded.port`. Such TCP URLs grant no local
/// trust. [`noded_url`] uses only Unix settings for implicit local discovery.
/// `None` when the document or explicit socket is invalid, or neither endpoint
/// has readable settings.
pub fn url_from_node_config(source: &str) -> Option<String> {
    url_from_node_value(&strict::parse(source).ok()?)
}

/// [`url_from_node_config`] on an already parsed document.
fn url_from_node_value(config: &Value) -> Option<String> {
    if let Some(socket) = config
        .get("noded")
        .and_then(|noded| noded.get("unix_socket"))
    {
        return socket
            .as_str()
            .filter(|path| std::path::Path::new(path).is_absolute())
            .map(|path| format!("unix://{path}"));
    }
    // Raw config readers need the same one-release alias as typed config.
    if config.get("mesh_addr").is_some() && config.get("wg_ip").is_some() {
        return None;
    }
    let mesh_addr = config
        .get("mesh_addr")
        .or_else(|| config.get("wg_ip"))
        .and_then(Value::as_str);
    let port = config
        .get("noded")
        .and_then(|noded| noded.get("port"))
        .and_then(port_of);
    if mesh_addr.is_none() && port.is_none() {
        return None;
    }
    let host = mesh_addr.filter(|ip| !ip.is_empty()).unwrap_or("127.0.0.1");
    Some(format!("ws://{host}:{}/ws", port.unwrap_or(DEFAULT_PORT)))
}

/// A port: a whole number in range, or a string holding one.
fn port_of(value: &Value) -> Option<u16> {
    match value {
        Value::Number(n) if n.fract() == 0.0 && (0.0..=f64::from(u16::MAX)).contains(n) => {
            Some(*n as u16)
        }
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_unix_socket_precedes_legacy_tcp_and_must_be_absolute() {
        assert_eq!(
            url_from_node_config(
                "mesh_addr: '192.0.2.1'\nnoded: { port: 4300, unix_socket: '/run/example/bus.sock' }"
            )
            .as_deref(),
            Some("unix:///run/example/bus.sock")
        );
        for socket in ["''", "'relative.sock'", "7"] {
            assert_eq!(
                url_from_node_config(&format!("noded: {{ port: 4300, unix_socket: {socket} }}")),
                None
            );
        }
    }

    #[test]
    fn reads_the_example_shape() {
        let source = r#"
-- A node's identity and services.
node: "alpha"
mesh_addr: "192.0.2.10"
mesh: "example.com"

noded: {
  port: 4201,
  admission: "off",
}
"#;
        assert_eq!(
            url_from_node_config(source).as_deref(),
            Some("ws://192.0.2.10:4201/ws")
        );
    }

    #[test]
    fn reads_the_quoted_key_shape_with_commas() {
        let source = r#"{
  "maild": { "port": 25, "enabled": true },
  "node": "beta",
  "noded": {
    "admission": "enforce",
    "port": 4200
  },
  "mesh_addr": "192.0.2.9"
}"#;
        assert_eq!(
            url_from_node_config(source).as_deref(),
            Some("ws://192.0.2.9:4200/ws")
        );
    }

    #[test]
    fn empty_mesh_addr_falls_back_to_loopback_and_missing_port_to_default() {
        assert_eq!(
            url_from_node_config("mesh_addr: \"\"\nnoded: { port: 4300 }\n").as_deref(),
            Some("ws://127.0.0.1:4300/ws")
        );
        assert_eq!(
            url_from_node_config("node: \"solo\"\nmesh_addr: \"10.0.0.2\"\n").as_deref(),
            Some("ws://10.0.0.2:4200/ws")
        );
        assert_eq!(url_from_node_config("node: \"solo\"\n"), None);
        assert_eq!(url_from_node_config(""), None);
    }

    #[test]
    fn nested_port_keys_do_not_leak_into_noded() {
        // `port` inside maild, and a `mesh_addr` inside a nested map, are not the
        // top-level keys.
        let source = "maild: { port: 25, mesh_addr: \"9.9.9.9\" }\nnoded: { port: 4444 }\nmesh_addr: \"192.0.2.1\"\n";
        assert_eq!(
            url_from_node_config(source).as_deref(),
            Some("ws://192.0.2.1:4444/ws")
        );
        // A list between keys does not disturb the top level.
        let source = "names: [\"a\", \"b\"]\nnoded: { port: 4445 }\n";
        assert_eq!(
            url_from_node_config(source).as_deref(),
            Some("ws://127.0.0.1:4445/ws")
        );
    }

    #[test]
    fn comments_and_strings_with_dashes_are_handled() {
        let source = "mesh_addr: \"10.0.0.1\" -- the mesh address\n# hash comment\nnoded: { port: 4200, note: \"a--b\" }\n";
        assert_eq!(
            url_from_node_config(source).as_deref(),
            Some("ws://10.0.0.1:4200/ws")
        );
    }

    #[test]
    fn ports_read_as_whole_numbers_in_range_or_numeric_strings() {
        assert_eq!(
            url_from_node_config("noded: { port: \"4301\" }\n").as_deref(),
            Some("ws://127.0.0.1:4301/ws")
        );
        // An unreadable port is missing: the default applies when mesh_addr is
        // there, and nothing is readable when it is not.
        assert_eq!(
            url_from_node_config("mesh_addr: \"10.0.0.3\"\nnoded: { port: 70000 }\n").as_deref(),
            Some("ws://10.0.0.3:4200/ws")
        );
        assert_eq!(url_from_node_config("noded: { port: 42.5 }\n"), None);
        assert_eq!(url_from_node_config("noded: { port: true }\n"), None);
    }

    #[test]
    fn a_document_that_is_not_strict_data_reads_as_nothing() {
        // Executable constructs and broken syntax are refused, not scanned.
        assert_eq!(url_from_node_config("mesh_addr: $addr\n"), None);
        assert_eq!(
            url_from_node_config("mesh_addr: \"10.0.0.4\"\nnoded: {\n"),
            None
        );
        // Entries inside braces need commas; a newline alone is a refusal.
        assert_eq!(
            url_from_node_config(
                "mesh_addr: \"10.0.0.5\"\nnoded: {\n  port: 4200\n  admission: \"off\"\n}\n"
            ),
            None
        );
    }

    #[test]
    fn candidate_order_is_user_then_system_unless_etc_is_set() {
        let paths = candidate_paths(None, None, Some(PathBuf::from("/home/user")));
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/home/user/.config/mix/node.conf.mix"),
                PathBuf::from("/etc/mix/node.conf.mix"),
            ]
        );
        let paths = candidate_paths(
            None,
            Some(PathBuf::from("/home/user/cfg")),
            Some(PathBuf::from("/home/user")),
        );
        assert_eq!(paths[0], PathBuf::from("/home/user/cfg/mix/node.conf.mix"));
        // MIX_ETC suppresses the system fallback.
        let paths = candidate_paths(Some(PathBuf::from("/sandbox/etc/mix")), None, None);
        assert_eq!(paths, vec![PathBuf::from("/sandbox/etc/mix/node.conf.mix")]);
        // No home at all still finds the system file.
        assert_eq!(
            candidate_paths(None, None, None),
            vec![PathBuf::from("/etc/mix/node.conf.mix")]
        );
    }
}
