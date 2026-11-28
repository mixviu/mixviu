// SPDX-License-Identifier: MIT OR Apache-2.0
//! Broker-held manifests. Validation never retains a partially accepted set.
use std::collections::HashSet;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};

// 512 strings: 64 ASCII bytes each, worst-case six-byte JSON Unicode escapes,
// two quotes and one comma per string, plus the brackets (last comma omitted).
// Whitespace also consumes this wire budget.
pub(super) const MAX_BODY_BYTES: usize = 512 * (64 * 6 + 3) + 1;

pub(super) fn decode(body: &str) -> Result<Vec<String>, &'static str> {
    if body.len() > MAX_BODY_BYTES { return Err("21"); }
    struct Names;
    impl<'de> Visitor<'de> for Names {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("command name array") }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut names = Vec::new();
            while names.len() < 512 {
                match seq.next_element::<String>()? {
                    Some(name) => names.push(name),
                    None => return Ok(names),
                }
            }
            let _ = seq.next_element_seed(Overflow)?;
            Ok(names)
        }
    }
    struct Overflow;
    impl<'de> DeserializeSeed<'de> for Overflow {
        type Value = ();
        fn deserialize<D: serde::Deserializer<'de>>(self, _d: D) -> Result<(), D::Error> {
            Err(serde::de::Error::custom("manifest count limit"))
        }
    }
    let mut deserializer = serde_json::Deserializer::from_str(body);
    let names = serde::Deserializer::deserialize_seq(&mut deserializer, Names)
        .map_err(|e| if e.to_string().starts_with("manifest count limit") { "21" } else { "20" })?;
    deserializer.end().map_err(|_| "20")?;
    Ok(names)
}

/// Static infrastructure verbs, independent of service manifests. PR A consumes this table.
#[allow(dead_code)]
pub(super) const BROKER_CONTROL_VERBS: &[&str] = &[
    "noded.register",
    "noded.deregister",
    "noded.commands.set",
    "noded.ping",
    "noded.list",
    "noded.info",
    "noded.peers",
    "noded.inventory",
    "noded.observe.start",
    "noded.observe.stop",
    "noded.tap",
    "topic.publish",
    "topic.subscribe",
    "topic.unsubscribe",
    "topic.subscriber_count",
    "topic.list",
    "topic.clear",
    "ui.subscribe",
    "ui.unsubscribe",
    "noded.props.subscribe_grant",
    "noded.props.get",
    "noded.props.list",
    "noded.props.describe",
    "noded.props.watch",
    "props.watch",
    "spec.get",
    "spec.v2.get",
    "log.props.get",
    "log.props.list",
    "log.props.describe",
];

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
    #[cfg(test)]
    pub fn attempts(&self) -> u8 { self.sets }
    #[cfg(test)]
    pub fn replace(&mut self, body: &str) -> &'static str {
        self.replace_decoded(decode(body))
    }
    pub fn replace_decoded(&mut self, decoded: Result<Vec<String>, &'static str>) -> &'static str {
        self.names.clear();
        if self.sets >= 16 {
            return "25";
        }
        self.sets += 1;
        let names = match decoded {
            Ok(names) => names,
            Err(rc) => return rc,
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
    let names: HashSet<String> = match config.and_then(|c| c.extra.get("history_allowlist")) {
        Some(value) => serde_json::from_value::<Vec<String>>(value.clone())?
            .into_iter()
            .collect(),
        None => DEFAULT_ALLOWLIST.iter().map(|s| (*s).to_owned()).collect(),
    };
    anyhow::ensure!(names.iter().all(|name| name == "mon" || !super::is_reserved_service_name(name)), "history_allowlist contains reserved identity");
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_boundaries_and_allowlist_types() {
        assert_eq!(decode(&" ".repeat(MAX_BODY_BYTES + 1)), Err("21"));
        // The 513th item is never decoded, even when it is invalid JSON.
        let prefix = format!("[{}", "\"a\",".repeat(512));
        assert_eq!(decode(&(prefix + "{invalid")), Err("21"));
        for names in [
            vec!["example.read".to_owned(); 512],
            vec!["x".repeat(64)],
            vec!["a.b.c.d.e.f".to_owned()],
        ] {
            assert_eq!(
                Manifest::default().replace(&serde_json::to_string(&names).unwrap()),
                "0"
            );
        }
        for value in [
            serde_json::json!("maild"),
            serde_json::json!(42),
            serde_json::json!(["maild", 42]),
        ] {
            let mut config = config::node::NodedConfig::default();
            config.extra.insert("history_allowlist".into(), value);
            assert!(allowlist(Some(&config)).is_err());
        }
        assert!(!allowlist(None).unwrap().contains("log"));
        for verb in ["log.props.get", "log.props.list", "log.props.describe"] {
            assert!(BROKER_CONTROL_VERBS.contains(&verb));
        }
    }

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
