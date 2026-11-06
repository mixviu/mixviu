// SPDX-License-Identifier: MIT OR Apache-2.0
//! Existing compd titlebar wire contract, derived from application-owned
//! command-bar data. No file-manager actions or availability policy live here.
use serde_json::{Value, json};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct Hosting {
    items: Vec<Value>,
    revision: u64,
    dirty: bool,
    pending: bool,
    retry: Option<Instant>,
    offered: bool,
    probes: usize,
    pub confirmed: bool,
}
impl Hosting {
    /// Topic deliveries carry broker_service; local RPCs carry the broker's
    /// registered caller in from. Both require the broker's local origin.
    pub fn local_owner(
        headers: &std::collections::BTreeMap<String, String>,
        expected: &str,
    ) -> bool {
        headers.get("broker_origin").map(String::as_str) == Some("local")
            && headers
                .get("broker_service")
                .or_else(|| headers.get("from"))
                .map(String::as_str)
                == Some(expected)
    }
    pub fn eligible(settings: &Value) -> bool {
        Self::host_available(settings) && settings["window.actions_place"] == "titlebar"
    }
    pub fn host_available(settings: &Value) -> bool {
        let requested = settings["window.decorations"]
            .as_str()
            .and_then(design::RequestedDecorations::from_name);
        design::decorations::resolve_with_actions(
            requested,
            design::RequestedDecorations::Auto,
            design::ActionsPlace::resolve(
                settings["window.actions_place"].as_str().unwrap_or("row"),
            ),
        )
        .mode
            == design::Decorations::Server
            && design::decorations::has_host()
    }
    pub fn update(&mut self, toolbar: &Value) -> Result<(), String> {
        crate::rich::validate("command-bar", toolbar)?;
        let mut items = Vec::new();
        for item in toolbar.as_array().ok_or("toolbar must be an array")? {
            if item["separator"] == true {
                continue;
            }
            let Some(icon) = item["icon"]
                .as_str()
                .and_then(crate::icons::Icon::from_name)
            else {
                return Err("hosted actions require a known icon".into());
            };
            let id = item["id"].as_str().ok_or("hosted action requires an ID")?;
            let command = if item["children"].is_array() {
                popup_command(id)?
            } else {
                id.into()
            };
            if command.is_empty()
                || command.len() > 128
                || !command
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b".-_".contains(&b))
                || items.iter().any(|v: &Value| v["command"] == command)
            {
                return Err("invalid or duplicate hosted command".into());
            }
            let tooltip = item["label"]
                .as_str()
                .ok_or("hosted action requires a label")?;
            if tooltip.len() > 2048 || items.len() >= 128 {
                return Err("hosted bar exceeds native budget".into());
            }
            items.push(json!({"command":command,"icon":icon.name(),"tooltip":tooltip,"enabled":item["enabled"]==true,"toggled":item["checked"]==true}));
        }
        if self.items != items {
            self.items = items;
            self.revision = self
                .revision
                .max(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_micros() as u64,
                )
                .saturating_add(1);
            self.dirty = true;
            self.probes = 0;
        }
        Ok(())
    }
    pub fn form(&self, mut form: Value, eligible: bool) -> Value {
        let hosted=eligible && self.confirmed;
        form["offered"]=json!(eligible);
        form["hosted"] = json!(hosted);
        form["effective"] = json!(if hosted { "titlebar" } else { "row" });
        form["bar"] = if hosted {
            json!({"revision":self.revision,"items":self.items})
        } else {
            json!({"revision":0,"items":[]})
        };
        form
    }
    pub fn reconnect(&mut self) {
        self.confirmed = false;
        self.dirty = true;
        self.pending = false;
        self.retry = None;
        self.offered = false;
        self.probes = 0;
    }
    pub fn completed(&mut self, ok: bool) {
        self.pending = false;
        if !ok || !self.confirmed {
            self.dirty = true;
        }
        let delays = [100, 250, 500, 1000, 2000, 5000];
        self.retry = Some(
            Instant::now() + Duration::from_millis(delays[self.probes.saturating_sub(1).min(5)]),
        );
    }
    pub fn poll(
        &mut self,
        ctx: &egui::Context,
        endpoint: &crate::native_bus::Endpoint,
        service: &str,
        host: &str,
        eligible: bool,
    ) {
        if eligible != self.offered {
            self.offered = eligible;
            self.probes = 0;
            self.dirty = true;
            self.retry = None;
        }
        if !eligible || !self.dirty || self.pending || (!self.confirmed && self.probes >= 6) {
            return;
        }
        if let Some(at) = self.retry {
            if Instant::now() < at {
                ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
                return;
            }
        }
        match endpoint.hint_with_probe(
            format!("{service}.ui.titlebar.changed"),
            json!({"revision":self.revision}),
            host.into(),
        ) {
            Ok(()) => {
                self.dirty = false;
                self.pending = true;
                self.probes += 1;
            }
            Err(_) => {
                self.retry = Some(Instant::now() + Duration::from_millis(100));
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
    }
}
fn popup_command(id: &str) -> Result<String, String> {
    if id.is_empty() || id.len() > 48 {
        return Err("hosted popup ID exceeds codec budget".into());
    }
    Ok(format!(
        "viu.popup.{}",
        id.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}
pub fn popup_target<'a>(toolbar: &'a Value, command: &str) -> Option<&'a str> {
    toolbar.as_array()?.iter().find_map(|item| {
        let id = item["id"].as_str()?;
        (item["enabled"] == true
            && item["children"].is_array()
            && popup_command(id).ok().as_deref() == Some(command))
        .then_some(id)
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hosting_acknowledgement_uses_only_the_native_broker_owner_stamp() {
        let mut headers = std::collections::BTreeMap::from([
            ("from".into(), "shell@example.invalid".into()),
            ("broker_service".into(), "shell".into()),
            ("broker_origin".into(), "local".into()),
        ]);
        assert!(Hosting::local_owner(&headers, "shell"));
        assert!(!Hosting::local_owner(&headers, "comp"));
        headers.insert("broker_origin".into(), "mesh".into());
        assert!(!Hosting::local_owner(&headers, "shell"));
        headers.remove("broker_origin");
        assert!(!Hosting::local_owner(&headers, "shell"));
        let rpc = std::collections::BTreeMap::from([
            ("from".into(), "shell".into()),
            ("broker_origin".into(), "local".into()),
        ]);
        assert!(Hosting::local_owner(&rpc, "shell"));
        let mut qualified = rpc.clone();
        qualified.insert("from".into(), "shell@example.invalid".into());
        assert!(!Hosting::local_owner(&qualified, "shell"));
    }
    #[test]
    fn native_bar_preserves_leaf_identity_and_bounds_popup_codec() {
        let mut host = Hosting::default();
        let toolbar = json!([{"id":"file.copy","label":"Copy","icon":"copy","enabled":false},{"id":"Look","label":"Look","icon":"sun","enabled":true,"children":[{"id":"theme.light","label":"Light","enabled":true}]}]);
        host.update(&toolbar).unwrap();
        let unacknowledged=host.form(json!({}),true);
        assert_eq!(unacknowledged["offered"],true);assert_eq!(unacknowledged["hosted"],false);assert_eq!(unacknowledged["effective"],"row");assert_eq!(unacknowledged["bar"]["items"],json!([]));
        host.confirmed=true;
        let form = host.form(json!({}), true);
        assert_eq!(form["bar"]["items"][0]["command"], "file.copy");
        assert_eq!(form["bar"]["items"][0]["enabled"], false);
        let command = form["bar"]["items"][1]["command"].as_str().unwrap();
        assert_eq!(popup_target(&toolbar, command), Some("Look"));
        assert_eq!(popup_target(&toolbar, "viu.popup.0000"), None);
        let first = host.revision;
        host.update(&toolbar).unwrap();
        assert_eq!(host.revision, first);
        let mut invalid = toolbar.clone();
        invalid[0]["id"] = json!("Bad COMMAND");
        assert!(host.update(&invalid).is_err());
        assert_eq!(host.revision, first);
        assert_eq!(host.form(json!({}), false)["bar"]["items"], json!([]));
    }
}
