// SPDX-License-Identifier: MIT OR Apache-2.0
//! Workshop test policy, observed at the actual local protocol endpoint.
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Audit {
    acknowledged: bool,
    recovery: bool,
    pending_full: Option<u64>,
    pub full: usize,
    pub patches: usize,
    pub refusals: usize,
    pub refuse_next: bool,
    largest: Value,
}
impl Audit {
    pub fn observe(&mut self, command: &str, body: &Value, before: &Value) -> Result<(), String> {
        let operations = body["operations"].as_array();
        let bytes = encoded(body);
        let mut sections = BTreeMap::<String, usize>::new();
        if let Some(ops) = operations {
            for op in ops {
                let key = op["path"][0].as_str().unwrap_or("<root>").to_owned();
                *sections.entry(key).or_default() += encoded(op);
            }
        } else if let Some(model) = body["value"].as_object() {
            for (key, value) in model {
                sections.insert(key.clone(), encoded(value));
            }
        }
        if bytes >= self.largest["bytes"].as_u64().unwrap_or(0) as usize {
            self.largest = json!({"kind":command,"operations":operations.map_or(0, Vec::len),"bytes":bytes,"sections":sections,"revision":body["revision"]});
        }
        match command {
            "scenes.model" => {
                if (self.acknowledged || self.full > 0) && !self.recovery {
                    return Err(format!(
                        "Workshop publication kind: expected=scenes.model.patch actual={command} revision={} refusal_credit=0",
                        body["revision"]
                    ));
                }
                self.recovery = false;
                self.full += 1;
                self.pending_full = body["revision"].as_u64();
            }
            "scenes.model.patch" => {
                self.recovery = false;
                self.patches += 1;
                for op in operations.into_iter().flatten() {
                    if op["op"] != "set" {
                        continue;
                    }
                    let path = op["path"].as_array().expect("protocol path");
                    let value = &op["value"];
                    if path.is_empty()
                        || (path.len() == 1 && (value.is_object() || value.is_array()))
                    {
                        let mut old = before;
                        for part in path {
                            old = if let Some(key) = part.as_str() {
                                &old[key]
                            } else {
                                &old[part.as_u64().unwrap() as usize]
                            };
                        }
                        return Err(format!(
                            "Workshop container rider: path={} value_type={} encoded_bytes={} fallback={} (depth-8 collapse unreachable at path length {})",
                            op["path"],
                            kind(value),
                            encoded(value),
                            fallback(old, value, path.len()),
                            path.len()
                        ));
                    }
                }
            }
            _ => unreachable!("publication endpoint recorder"),
        }
        Ok(())
    }
    pub fn receipt(&mut self, revision: u64, result: &Result<(), String>) {
        if result.is_err() {
            self.refusals += 1;
            self.recovery = true;
        } else if self.pending_full == Some(revision) {
            self.acknowledged = true;
            self.pending_full = None;
        }
    }
    pub fn summary(&self) -> Value {
        json!({"full":self.full,"patches":self.patches,"refusals":self.refusals,"largest_publication":self.largest})
    }
}
fn encoded(value: &Value) -> usize {
    serde_json::to_vec(value).unwrap().len()
}
fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "nil",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "list",
        Value::Object(_) => "map",
    }
}
fn long_changed_child(a: &Value, b: &Value) -> bool {
    let (Some(a), Some(b)) = (a.as_object(), b.as_object()) else {
        return false;
    };
    a.keys()
        .chain(b.keys())
        .any(|key| key.len() > 256 && a.get(key) != b.get(key))
}
fn fallback(a: &Value, b: &Value, depth: usize) -> &'static str {
    if kind(a) != kind(b) {
        "type change"
    } else if depth >= 8 {
        "depth-8 collapse"
    } else if long_changed_child(a, b) {
        "long-key parent"
    } else if a.is_array() && b.is_array() && diff_count(a, b, depth, true) > 256 {
        "over-256-ops list fallback"
    } else {
        "unknown"
    }
}
// Recompute the Mix differ's operation count, uncapping only this list.
// Recursive child lists retain their own 256-operation fallback. This is
// diagnostic work only after a rider violation, never normal tick work.
fn diff_count(a: &Value, b: &Value, depth: usize, uncapped: bool) -> usize {
    if a == b {
        return 0;
    }
    if depth >= 8 || long_changed_child(a, b) {
        return 1;
    }
    if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
        return a.keys().filter(|k| !b.contains_key(*k)).count()
            + b.iter()
                .map(|(k, v)| {
                    a.get(k)
                        .map_or(1, |old| diff_count(old, v, depth + 1, false))
                })
                .sum::<usize>();
    }
    if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
        let mut current = a.clone();
        let mut i = 0;
        let mut count = 0;
        while i < b.len() {
            if current.get(i) == Some(&b[i]) {
                i += 1;
                continue;
            }
            if let Some(anchor) = ((i + 1)..current.len()).find(|&j| current[j] == b[i]) {
                current.drain(i..anchor);
                count += 1;
            } else if current.get(i).is_some_and(|old| {
                old.is_object()
                    && b[i].is_object()
                    && !old["id"].is_null()
                    && old["id"] == b[i]["id"]
            }) {
                count += diff_count(&current[i], &b[i], depth + 1, false);
                current[i] = b[i].clone();
                i += 1;
            } else {
                current.insert(i, b[i].clone());
                count += 1;
                i += 1;
            }
            if count > 256 {
                return if uncapped { count } else { 1 };
            }
        }
        return count + usize::from(current.len() > b.len());
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnoses_the_actual_fallback_and_allows_scalars() {
        assert_eq!(fallback(&json!(null), &json!({}), 1), "type change");
        assert_eq!(
            fallback(&json!({}), &json!({("x".repeat(257)):1}), 1),
            "long-key parent"
        );
        assert_eq!(
            fallback(&json!([]), &json!(vec![0; 257]), 1),
            "over-256-ops list fallback"
        );
        assert_eq!(fallback(&json!([]), &json!([1]), 8), "depth-8 collapse");
        let mut audit = Audit::default();
        audit
            .observe(
                "scenes.model.patch",
                &json!({"operations":[{"op":"set","path":["scalar"],"value":"allowed"}]}),
                &json!({}),
            )
            .unwrap();
        let error = audit
            .observe(
                "scenes.model.patch",
                &json!({"operations":[{"op":"set","path":["section"],"value":{}}]}),
                &json!({"section":{}}),
            )
            .unwrap_err();
        assert!(
            error.contains("path=[\"section\"] value_type=map encoded_bytes=2 fallback=unknown"),
            "{error}"
        );
    }
}
