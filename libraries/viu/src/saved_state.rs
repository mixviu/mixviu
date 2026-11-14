// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native persistence envelope. Mix owns its versioned application value;
//! toolkit owns portable frames, secondary windows and named widget state.
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
pub struct Saved {
    pub store: toolkit::saved::Store<Value>,
    pub announced: bool,
    accepted: bool,
    native: Option<(String, String)>,
    restoring: bool,
    captured_exit: bool,
    changed: Arc<tokio::sync::Notify>,
    pending_panels: std::collections::BTreeMap<String, f32>,
}
impl Saved {
    pub fn load(app: &str, version: u32, headless: bool) -> Self {
        Self {
            store: toolkit::saved::Store::load(
                if headless {
                    None
                } else {
                    config::AppDirs::resolve(app).map(|d| d.state())
                },
                version,
            ),
            announced: false,
            accepted: false,
            native: None,
            restoring: false,
            captured_exit: false,
            changed: Arc::new(tokio::sync::Notify::new()),
            pending_panels: Default::default(),
        }
    }
    pub fn configure(&mut self, app_id: String, comp: String) {
        self.native = Some((app_id, comp));
    }
    pub fn notice_topic(&self, topic: &str) {
        if self
            .native
            .as_ref()
            .is_some_and(|(_, comp)| topic == format!("{comp}.props.changed"))
        {
            self.changed.notify_one();
        }
    }
    pub fn restore(&mut self, client: Option<Arc<bus::SupervisedClient>>) {
        if self.restoring {
            return;
        }
        let (Some((app_id, comp)), Some(client)) = (self.native.clone(), client) else {
            return;
        };
        self.restoring = true;
        let frame = self.store.window.clone();
        let changed = self.changed.clone();
        std::thread::spawn(move || {
            let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            rt.block_on(async {
                let _ = client
                    .subscribe_topic(&format!("{comp}.props.changed"))
                    .await;
                frame
                    .restore_with_events(
                        &app_id,
                        |command, args| {
                            let client = client.clone();
                            let comp = comp.clone();
                            async move { native_call(client, &comp, command, args).await }
                        },
                        || async {
                            changed.notified().await;
                            true
                        },
                    )
                    .await;
            });
        });
    }
    pub fn capture_exit(&mut self, client: Option<Arc<bus::SupervisedClient>>) {
        if self.captured_exit {
            return;
        }
        let (Some((app_id, comp)), Some(client)) = (self.native.clone(), client) else {
            return;
        };
        self.captured_exit = true;
        let Ok(rt) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        rt.block_on(async {
            let _ = tokio::time::timeout(
                Duration::from_millis(700),
                self.store.window.capture_exit(&app_id, |command, args| {
                    let client = client.clone();
                    let comp = comp.clone();
                    async move { native_call(client, &comp, command, args).await }
                }),
            )
            .await;
        });
    }
    pub fn notice(&self) -> Value {
        json!({"state":self.store.describe(),"app_loaded":self.store.app_loaded()})
    }
    pub fn apply(&mut self, model: &Value) {
        self.accepted |= model.get("saved_app").is_some() || model["saved_panels"].is_object();
        if let Some(app) = model.get("saved_app") {
            self.store.app = app.clone();
        }
        if let Some(panels) = model["saved_panels"].as_object() {
            for (name, value) in panels {
                if let Some(share) = value
                    .as_f64()
                    .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
                {
                    self.pending_panels.insert(name.clone(), share as f32);
                }
            }
        }
        if let Some(appearance) = model["theme_choice"]
            .as_object()
            .or_else(|| model["appearance"].as_object())
        {
            let mut choice = Value::Object(appearance.clone());
            if !choice["caption_side"].is_null() && choice["captions"].is_null() {
                choice["captions"] = choice["caption_side"].clone();
            }
            if let Ok(theme) = serde_json::from_value(choice) {
                self.store.window.theme = theme;
            }
        }
    }
    pub fn tick(&mut self, ctx: &egui::Context, app: &str, flush: bool) {
        if !self.accepted {
            return;
        }
        self.store.window.capture(ctx);
        for (name, share) in std::mem::take(&mut self.pending_panels) {
            if self
                .store
                .widgets
                .panels
                .get(&name)
                .is_none_or(|old| (*old - share).abs() > f32::EPSILON)
            {
                toolkit::saved::widgets::set_panel(ctx, &name, share);
            }
        }
        self.store.widgets.collect(ctx);
        self.store.collect_windows(ctx, app);
        let result = if flush {
            self.store.flush()
        } else {
            self.store.tick(
                std::time::Instant::now(),
                ctx.input(|i| i.pointer.any_down()),
            )
        };
        if let Err(e) = result {
            eprintln!("saved window state: {e}");
        }
    }
}

async fn native_call(
    client: Arc<bus::SupervisedClient>,
    comp: &str,
    command: &str,
    args: Value,
) -> Option<Value> {
    let (rc, body, _) = client
        .call_with_headers_raw_within(
            comp,
            command,
            &Default::default(),
            &args.to_string(),
            Duration::from_secs(10),
        )
        .await
        .ok()?;
    if rc != 0 {
        return None;
    }
    serde_json::from_str(&body).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_captions_survive_both_choice_spellings() {
        let mut saved = Saved::load("mode-test", 1, true);
        for choice in [
            json!({"captions":"left"}),
            json!({"caption_side":"left"}),
            json!({"captions":"left","caption_side":null}),
        ] {
            saved.apply(&json!({"theme_choice":choice}));
            assert_eq!(
                serde_json::to_value(&saved.store.window.theme).unwrap()["captions"],
                "left"
            );
        }
    }
}
