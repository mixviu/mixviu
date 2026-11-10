// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Native chrome ownership on the existing compositor Bus connection.
use crate::port::{FireSender, PortIngress};
use bus::{IncomingCommand, SupervisedClient};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

static ENABLED: AtomicBool = AtomicBool::new(false);
static TARGETS: std::sync::Mutex<BTreeSet<String>> = std::sync::Mutex::new(BTreeSet::new());
pub fn targets(targets: BTreeSet<String>) {
    *TARGETS.lock().unwrap() = targets;
}
pub fn enable() {
    ENABLED.store(true, Ordering::Release);
}
pub(crate) fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

pub fn install_emitters(fires: FireSender) {
    let menu = fires.clone();
    decor::window::menu_emitter(move |service, viewport, side, x| {
        menu.fire(
            service,
            &format!("{service}.ui.menu.popup"),
            menu_args(viewport, side.name(), x),
        )
    });
    decor::window::action_emitter(move |service, viewport, command| {
        fires.fire(
            service,
            &format!("{service}.execute"),
            action_args(viewport, command),
        )
    });
}
fn menu_args(viewport: Option<&str>, side: &str, x: f32) -> Value {
    let mut args = json!({"side": side, "x": x});
    if let Some(viewport) = viewport {
        args["viewport"] = json!(viewport);
    }
    args
}
fn action_args(viewport: Option<&str>, command: &str) -> Value {
    let mut args = json!({"id": command});
    if let Some(viewport) = viewport {
        args["window"] = json!(viewport);
    }
    args
}

pub struct Update {
    generation: u64,
    live: Arc<AtomicU64>,
    theme: Option<decor::appearance::Look>,
    agent_seat: Option<comp_model::agent_seat::Policy>,
    services: BTreeSet<String>,
    bars: BTreeMap<String, Value>,
}
impl Update {
    pub(crate) fn look(&self) -> Option<&decor::appearance::Look> {
        (self.live.load(Ordering::Acquire) == self.generation).then_some(())?;
        self.theme.as_ref()
    }
    /// Runs only on the engine thread. Reconnect fences queued presentations.
    pub(crate) fn apply(self) -> bool {
        if self.live.load(Ordering::Acquire) != self.generation {
            return false;
        }
        let mut changed = self.theme.is_some();
        if let Some(policy) = self.agent_seat {
            comp_model::agent_seat::install(policy);
        }
        if let Some(look) = self.theme {
            decor::window::install(look.chrome(look.window_style));
        }
        changed |= decor::window::menu_services(self.services);
        for (service, form) in self.bars {
            let bar = if form["hosted"] == true && form["service"].as_str() == Some(&service) {
                serde_json::from_value(form["bar"].clone()).unwrap_or_default()
            } else {
                Default::default()
            };
            changed |= decor::window::update_actions(&service, bar);
        }
        changed
    }
}

pub(crate) async fn run(
    client: &SupervisedClient,
    ingress: PortIngress,
    mut events: tokio::sync::mpsc::Receiver<IncomingCommand>,
    live: Arc<AtomicU64>,
) {
    let Ok(binding) = settings::session::binding() else {
        return;
    };
    let Ok(mut consumer) = settings::consumer::Consumer::for_shell(binding) else {
        return;
    };
    let mut states = client.subscribe_state();
    let mut retry = tokio::time::interval(std::time::Duration::from_secs(1));
    retry.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut generation = None;
    let mut work = None;
    let mut services = BTreeSet::new();
    loop {
        if let Some(now) = settings::native::live_generation(client) {
            if generation != Some(now) {
                generation = Some(now);
                work = consumer.connected(now);
            }
        } else {
            generation = None;
            consumer.disconnected();
        }
        while let Some(action) = work.take() {
            let result = settings::native::execute(client, &action).await;
            work = consumer.complete(&action, result);
        }
        let mut theme = None;
        let mut agent_seat = None;
        let pending = consumer.pending().cloned();
        if let Some(update) = &pending {
            use settings::view::View;
            let policy = settings::view::AgentSeat::of(&update.snapshot().values);
            agent_seat = Some(comp_model::agent_seat::Policy {
                default: policy.default,
                allow: policy.allow,
                deny: policy.deny,
            });
            match decor::appearance::build(update.snapshot()) {
                Ok(look) => theme = Some(look),
                Err(error) => tracing::warn!(?error, "native chrome settings refused"),
            }
        }
        // Read only on recovery/event/retry. Calls stay off the compositor loop.
        if let Some(epoch) = generation {
            if let Ok(Ok(names)) = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                client.call(
                    "noded",
                    "noded.props.get",
                    json!({"path":"services.registered"}),
                ),
            )
            .await
            {
                if let Some(names) = names.as_array().and_then(|names| {
                    names
                        .iter()
                        .map(|name| name.as_str().map(str::to_owned))
                        .collect::<Option<BTreeSet<String>>>()
                }) {
                    services = names;
                }
            }
            let mut bars = BTreeMap::new();
            let targets = TARGETS.lock().unwrap().clone();
            for service in targets.iter().filter(|name| services.contains(*name)) {
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(1),
                    client.subscribe_topic(&format!("{service}.ui.titlebar.changed")),
                )
                .await;
                if let Ok(Ok(form)) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    client.call(service, &format!("{service}.ui.titlebar.form"), json!({})),
                )
                .await
                {
                    bars.insert(service.clone(), form);
                }
            }
            let theme_ready = pending.is_none() || theme.is_some();
            if settings::native::live_generation(client) == Some(epoch)
                && ingress.chrome(Update {
                    generation: epoch,
                    live: live.clone(),
                    theme,
                    agent_seat,
                    services: services.clone(),
                    bars,
                })
                && theme_ready
            {
                if let Some(update) = &pending {
                    consumer.acknowledge(update);
                }
            }
        }
        tokio::select! {
            changed = states.changed() => { if changed.is_err() { break; } states.borrow_and_update(); }
            event = events.recv() => {
                let Some(event) = event else { break; };
                work = consumer.native_delivery(&event);
            }
            _ = retry.tick() => { work = consumer.refresh(); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn titlebar_routes_keep_application_viewport_contracts() {
        assert_eq!(
            action_args(Some("main"), "file.save"),
            json!({"id":"file.save","window":"main"})
        );
        assert_eq!(
            menu_args(Some("main"), "right", 24.0),
            json!({"viewport":"main","side":"right","x":24.0})
        );
        assert!(action_args(None, "file.save").get("window").is_none());
    }
    #[test]
    fn queued_chrome_from_an_old_connection_cannot_repaint() {
        let before = decor::window::generation();
        let services = decor::window::live_menu_services();
        assert!(
            !Update {
                generation: 1,
                live: Arc::new(AtomicU64::new(2)),
                theme: None,
                agent_seat: None,
                services: BTreeSet::from(["unexpected".into()]),
                bars: BTreeMap::new()
            }
            .apply()
        );
        assert_eq!(decor::window::live_menu_services(), services);
        assert_eq!(decor::window::generation(), before);
    }
}
