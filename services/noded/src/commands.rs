// SPDX-License-Identifier: MIT OR Apache-2.0
//! Broker-held manifests. Validation never retains a partially accepted set.
use std::collections::HashSet;

pub(super) const DEFAULT_ALLOWLIST: &[&str] = &[
    "maild",
    "webd",
    "dnsd",
    "blobd",
    "toolsd",
    "filesd",
    "provisiond",
    "sshm",
    "mon",
];

#[derive(Clone, Default)]
pub(super) struct Manifest {
    pub names: HashSet<String>,
    sets: u8,
}

impl Manifest {
    pub fn replace(&mut self, body: &str) -> &'static str {
        self.names.clear();
        if self.sets >= 16 {
            return "25";
        }
        self.sets += 1;
        let Ok(names) = serde_json::from_str::<Vec<String>>(body) else {
            return "20";
        };
        if names.len() > 512 {
            return "21";
        }
        if names.iter().any(|name| name.len() > 64) {
            return "22";
        }
        if names.iter().any(|name| name.split('.').count() > 6) {
            return "23";
        }
        if names.iter().any(|name| {
            name.split('.').any(|segment| {
                segment.is_empty()
                    || !segment
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            })
        }) {
            return "24";
        }
        self.names.extend(names);
        "0"
    }
}

pub(super) fn allowlist(
    config: Option<&config::node::NodedConfig>,
) -> anyhow::Result<HashSet<String>> {
    match config.and_then(|c| c.extra.get("history_allowlist")) {
        Some(value) => Ok(serde_json::from_value::<Vec<String>>(value.clone())?
            .into_iter()
            .collect()),
        None => Ok(DEFAULT_ALLOWLIST.iter().map(|s| (*s).to_owned()).collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_refuse_whole_set_with_distinct_rc() {
        for (names, rc) in [
            (serde_json::json!(vec!["example.read"; 513]), "21"),
            (serde_json::json!(["x".repeat(65)]), "22"),
            (serde_json::json!(["a.b.c.d.e.f.g"]), "23"),
            (serde_json::json!(["example@invalid"]), "24"),
            (serde_json::json!(["example..read"]), "24"),
            (serde_json::json!(["example-rpc"]), "24"),
            (serde_json::json!([""]), "24"),
            (serde_json::json!(["é"]), "24"),
            (serde_json::json!({}), "20"),
        ] {
            let mut manifest = Manifest::default();
            assert_eq!(manifest.replace(r#"["example.read"]"#), "0");
            assert_eq!(manifest.replace(&names.to_string()), rc);
            assert!(manifest.names.is_empty());
        }
        let mut manifest = Manifest::default();
        for _ in 0..16 {
            assert_eq!(manifest.replace(r#"["example.read"]"#), "0");
        }
        assert_eq!(manifest.replace("[]"), "25");
        assert!(manifest.names.is_empty());
        assert_eq!(manifest.replace("[]"), "25");
    }
}
