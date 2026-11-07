// SPDX-License-Identifier: MIT OR Apache-2.0
//! Read-only application metadata, published in-process by Mix. This document
//! carries no native session authority and implements no tab/pane policy.
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Default)]
pub(crate) struct Metadata {
    pub tabs: Vec<Tab>,
}
#[derive(Clone)]
pub(crate) struct Tab {
    pub id: u64,
    pub title: String,
    pub active: bool,
    pub selected: u64,
    pub panes: Vec<u64>,
}

impl Metadata {
    pub fn decode(value: &Value) -> Result<Self, String> {
        fn id(value: &Value) -> Result<u64, String> {
            crate::widgets::whole(value)
                .filter(|n| *n > 0)
                .ok_or_else(|| "positive application id required".into())
        }
        let tabs = value["tabs"]
            .as_array()
            .filter(|v| v.len() <= 32)
            .ok_or("bounded tab metadata required")?;
        let mut answer = Self::default();
        let mut tab_ids = BTreeSet::new();
        let mut pane_ids = BTreeSet::new();
        for tab in tabs {
            let tab_id = id(&tab["id"])?;
            let title = tab["title"]
                .as_str()
                .filter(|s| {
                    s.len() <= 256
                        && !s
                            .chars()
                            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
                })
                .ok_or("bounded sanitised title required")?;
            let active = tab["active"]
                .as_bool()
                .ok_or("explicit active tab required")?;
            let selected = id(&tab["selected"])?;
            let panes = tab["panes"]
                .as_array()
                .filter(|v| !v.is_empty() && v.len() <= 32)
                .ok_or("bounded pane metadata required")?;
            let panes = panes.iter().map(id).collect::<Result<Vec<_>, _>>()?;
            if !tab_ids.insert(tab_id)
                || !panes.contains(&selected)
                || panes.iter().any(|p| !pane_ids.insert(*p))
                || pane_ids.len() > 32
            {
                return Err("duplicate or inconsistent application metadata".into());
            }
            answer.tabs.push(Tab {
                id: tab_id,
                title: title.into(),
                active,
                selected,
                panes,
            });
        }
        if !answer.tabs.is_empty() && answer.tabs.iter().filter(|t| t.active).count() != 1 {
            return Err("one active tab required".into());
        }
        Ok(answer)
    }

    pub fn tab_of(&self, pane: u64) -> Option<u64> {
        self.tabs
            .iter()
            .find(|t| t.panes.contains(&pane))
            .map(|t| t.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn validates_application_metadata_without_deriving_layout_policy() {
        let good = json!({"tabs":[{"id":7,"title":"selected","active":true,"selected":3,"panes":[2,3]},{"id":1,"title":"earlier id","active":false,"selected":1,"panes":[1]}]});
        let value = Metadata::decode(&good).unwrap();
        assert_eq!(value.tabs[0].id, 7);
        assert_eq!(value.tabs[0].selected, 3);
        assert_eq!(value.tab_of(2), Some(7));
        assert_eq!(value.tab_of(1), Some(1));
        for (path, bad) in [
            ("selected", json!(8)),
            ("id", json!(0)),
            ("title", json!("bad\nname")),
            ("title", json!("x".repeat(257))),
            ("panes", json!([2, 2])),
            ("active", Value::Null),
        ] {
            let mut value = good.clone();
            value["tabs"][0][path] = bad;
            assert!(Metadata::decode(&value).is_err());
        }
        let mut duplicate = good.clone();
        duplicate["tabs"][1]["panes"] = json!([2]);
        duplicate["tabs"][1]["selected"] = json!(2);
        assert!(Metadata::decode(&duplicate).is_err());
        let mut inactive = good;
        inactive["tabs"][0]["active"] = json!(false);
        assert!(Metadata::decode(&inactive).is_err());
        assert!(
            Metadata::decode(&json!({"tabs":[]}))
                .unwrap()
                .tabs
                .is_empty()
        );
    }
}
