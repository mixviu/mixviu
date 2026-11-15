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

fn registration_identity(row: &Value) -> Option<String> {
    if let Some(id) = row["meta"]["noded.registration_id"].as_str() {
        return Some(id.to_owned());
    }
    let session = &row["native_session"];
    let identity: Vec<_> = [
        "broker_epoch",
        "record_id",
        "incarnation",
        "binding_generation",
    ]
    .iter()
    .map(|key| session[*key].clone())
    .collect();
    identity
        .iter()
        .all(|v| !v.is_null())
        .then(|| json!(identity).to_string())
}

pub struct Update {
    generation: u64,
    live: Arc<AtomicU64>,
    theme: Option<decor::appearance::Look>,
    agent_seat: Option<comp_model::agent_seat::Policy>,
    screen_off: Option<Option<u64>>,
    services: BTreeSet<String>,
    bars: BTreeMap<String, Value>,
}
impl Update {
    pub(crate) fn screen_off(&self) -> Option<Option<u64>> {
        (self.live.load(Ordering::Acquire) == self.generation)
            .then_some(self.screen_off)
            .flatten()
    }
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
            changed |= comp_model::agent_seat::install(policy);
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
    let mut registry_dirty = true;
    let mut registry_retry = None;
    let mut bars_dirty = BTreeSet::new();
    let mut presentation_retry = None;
    let mut generation = None;
    let mut work = None;
    let mut services = BTreeSet::new();
    let mut incarnations = BTreeMap::new();
    #[cfg(test)]
    let mut checkpoints: Vec<IncomingCommand> = Vec::new();
    loop {
        if let Some(now) = settings::native::live_generation(client) {
            if generation != Some(now) {
                generation = Some(now);
                work = consumer.connected(now);
                registry_dirty = true;
                registry_retry = None;
                // This is the Bus epoch, not a settingsd restart. The next
                // registry snapshot must refresh every live target on a new
                // connection, even if its registration identity stayed equal.
                incarnations.clear();
                bars_dirty.clear();
                presentation_retry = None;
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
        let mut screen_off = None;
        let pending = consumer.pending().cloned();
        if let Some(update) = &pending {
            screen_off = Some(
                update
                    .snapshot()
                    .values
                    .get("power.screen_off_s")
                    .and_then(Value::as_u64),
            );
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
            if registry_dirty && registry_retry.is_none_or(|at| std::time::Instant::now() >= at) {
                let result = tokio::time::timeout(std::time::Duration::from_secs(1), async {
                    client.subscribe_topic("noded.props.changed").await?;
                    let names = client
                        .call(
                            "noded",
                            "noded.props.get",
                            json!({"path":"services.registered"}),
                        )
                        .await?;
                    // Discovery is event-driven too. The broker's registration
                    // identity distinguishes unchanged targets from restarts
                    // hidden by the names-only property delivery.
                    let identities = if TARGETS.lock().unwrap().is_empty() {
                        Value::Null
                    } else {
                        client
                            .call("noded", "noded.list", json!({}))
                            .await
                            .unwrap_or(Value::Null)
                    };
                    Ok::<_, bus::SupervisedError>((names, identities))
                })
                .await;
                let snapshot = result.ok().and_then(Result::ok);
                let names = snapshot.as_ref().and_then(|(names, _)| {
                    names.as_array().and_then(|names| {
                        names
                            .iter()
                            .map(|name| name.as_str().map(str::to_owned))
                            .collect::<Option<BTreeSet<String>>>()
                    })
                });
                if let Some(names) = names {
                    services = names;
                    registry_dirty = false;
                    let identities = snapshot.as_ref().map(|(_, identities)| identities);
                    let targets = TARGETS.lock().unwrap().clone();
                    let mut next = BTreeMap::new();
                    for target in targets.iter().filter(|name| services.contains(*name)) {
                        let identity = identities
                            .and_then(Value::as_array)
                            .and_then(|rows| {
                                rows.iter()
                                    .find(|row| row["name"].as_str() == Some(target.as_str()))
                            })
                            .and_then(registration_identity);
                        // Native discovery rows currently lack a registration
                        // id too. Use their session identity when available;
                        // otherwise conservatively refresh per delivery, just
                        // as on older brokers. There is still no idle poll.
                        if identity.is_none() || incarnations.get(target) != identity.as_ref() {
                            bars_dirty.insert(target.clone());
                        }
                        if let Some(identity) = identity {
                            next.insert(target.clone(), identity);
                        }
                    }
                    incarnations = next;
                    bars_dirty.retain(|name| services.contains(name));
                }
                registry_retry = registry_dirty
                    .then(|| std::time::Instant::now() + std::time::Duration::from_secs(1));
            }
            let mut bars = BTreeMap::new();
            let targets = TARGETS.lock().unwrap().clone();
            let dirty: Vec<_> = targets
                .iter()
                .filter(|name| services.contains(*name) && bars_dirty.contains(*name))
                .cloned()
                .collect();
            for service in &dirty {
                let subscribed = tokio::time::timeout(
                    std::time::Duration::from_secs(1),
                    client.subscribe_topic(&format!("{service}.ui.titlebar.changed")),
                )
                .await;
                if !matches!(subscribed, Ok(Ok(()))) {
                    continue;
                }
                if let Ok(Ok(form)) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    client.call(service, &format!("{service}.ui.titlebar.form"), json!({})),
                )
                .await
                {
                    bars.insert(service.clone(), form);
                    bars_dirty.remove(service);
                }
            }
            let theme_ready = pending.is_none() || theme.is_some();
            let accepted = settings::native::live_generation(client) == Some(epoch)
                && ingress.chrome(Update {
                    generation: epoch,
                    live: live.clone(),
                    theme,
                    agent_seat,
                    screen_off,
                    services: services.clone(),
                    bars,
                })
                && theme_ready;
            if accepted && let Some(update) = &pending {
                consumer.acknowledge(update);
            }
            if !accepted {
                // A full engine queue must not discard a fetched titlebar.
                bars_dirty.extend(dirty);
            }
            presentation_retry = (!accepted || bars_dirty.iter().any(|s| services.contains(s)))
                .then(|| std::time::Instant::now() + std::time::Duration::from_secs(1));
        }
        let deadline = consumer
            .retry_deadline()
            .into_iter()
            .chain(registry_retry.filter(|_| generation.is_some()))
            .chain(presentation_retry.filter(|_| generation.is_some()))
            .min();
        #[cfg(test)]
        if !registry_dirty && bars_dirty.is_empty() && deadline.is_none() && consumer.pending().is_none() {
            for command in checkpoints.drain(..) {
                client.respond_parts(command.generation, &command.from, &command.command, command.id.as_deref(), 0, "{\"settled\":true}").await.unwrap();
            }
        }
        tokio::select! {
            changed = states.changed() => { if changed.is_err() { break; } states.borrow_and_update(); }
            event = events.recv() => {
                let Some(event) = event else { break; };
                #[cfg(test)]
                if event.command == "test.checkpoint" {
                    checkpoints.push(event);
                    continue;
                }
                if let Some(next) = consumer.native_delivery(&event) { work = Some(next); }
                if event.header("broker_origin") == Some("local") {
                    if event.topic() == Some("noded.props.changed") && event.header("broker_service") == Some("noded") {
                        if let Ok(body) = serde_json::from_str::<Value>(&event.body)
                            && (body["path"] == "services.registered" || body["gap"] == true || event.header("gap") == Some("true")) {
                            registry_dirty = true;
                            registry_retry = None;
                        }
                    } else if let Some(service) = event.header("broker_service")
                        && event.topic() == Some(format!("{service}.ui.titlebar.changed").as_str()) && services.contains(service) {
                        bars_dirty.insert(service.to_owned());
                    }
                }
            }
            _ = recovery_deadline(deadline) => {
                if consumer.retry_deadline().is_some_and(|at| std::time::Instant::now() >= at) { work = consumer.retry(); }
            }
        }
    }
}

async fn recovery_deadline(deadline: Option<std::time::Instant>) {
    match deadline {
        Some(at) => tokio::time::sleep_until(at.into()).await,
        None => std::future::pending().await,
    }
}

#[cfg(test)]
#[path = "../../../../../tests/lib/idle_bus.rs"]
mod idle_bus;

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn native_props_watch_echoes_the_requested_command() {
        let broker = test_broker::Broker::start();
        let reader = idle_bus::connect(&broker, "watch-reader").await;
        let reply = reader
            .call("noded", "noded.props.watch", json!({}))
            .await
            .unwrap();
        assert_eq!(reply["topic"], "noded.props.changed");
    }
    #[tokio::test]
    async fn registration_identity_is_stable_until_reregistered() {
        let broker = test_broker::Broker::start();
        let app = idle_bus::Service::start(&broker, "example-app").await;
        let reader = idle_bus::connect(&broker, "registry-reader").await;
        async fn identity(reader: &SupervisedClient) -> Value {
            let list = reader.call("noded", "noded.list", json!({})).await.unwrap();
            list.as_array()
                .unwrap()
                .iter()
                .find(|row| row["name"] == "example-app")
                .unwrap()["meta"]["noded.registration_id"]
                .clone()
        }
        let first = identity(&reader).await;
        assert!(
            first.as_str().is_some_and(|id| !id.is_empty()),
            "broker registration identity missing"
        );
        assert_eq!(identity(&reader).await, first);
        app.client
            .call(
                "noded",
                "noded.register",
                json!({"meta":{"noded.registration_id":"forged"}}),
            )
            .await
            .unwrap();
        let second = identity(&reader).await;
        assert_ne!(
            second,
            json!("forged"),
            "caller replaced the broker-owned identity"
        );
        assert_ne!(
            second, first,
            "explicit same-name re-registration kept the old identity"
        );
        app.client.shutdown().await;
        drop(app);
        let _app = idle_bus::Service::start(&broker, "example-app").await;
        assert_ne!(
            identity(&reader).await,
            second,
            "same-name registration reused an incarnation"
        );
    }
    #[tokio::test]
    async fn idle_chrome_reads_once_and_titlebar_changes_read_once() {
        let mut broker = test_broker::Broker::bounce_stable();
        let settings = idle_bus::Service::start(&broker, "settingsd").await;
        let mut app = idle_bus::Service::start(&broker, "example-app").await;
        targets(BTreeSet::from(["example-app".into()]));
        let (client, mut incoming, native_forward) = idle_bus::native(&broker, "comp-idle").await;
        let names = client
            .call(
                "noded",
                "noded.props.get",
                json!({"path":"services.registered"}),
            )
            .await
            .unwrap();
        assert!(
            names
                .as_array()
                .is_some_and(|names| names.contains(&json!("example-app"))),
            "registry: {names}"
        );
        let observer = idle_bus::Observer::start(&broker).await;
        let (send, events) = tokio::sync::mpsc::channel(64);
        let hints = send.clone();
        // Registrations are capped/coalesced independently by the broker.
        // Drive one explicit hint after each completed registration below,
        // so exact per-delivery counts do not depend on that timer's phase.
        let forward = tokio::spawn(async move {
            while let Some(event) = incoming.recv().await {
                if event.topic() != Some("noded.props.changed") && send.send(event).await.is_err() {
                    break;
                }
            }
        });
        let (wiring, starter) = crate::port::prepare(
            crate::port::PortIdentity {
                service: "comp-idle".into(),
                version: "test".into(),
                backend: "nested",
                engine: "test",
                noded_url: broker.url.clone(),
            },
            crate::channel::no_waker(),
        )
        .unwrap();
        let ingress = starter.ingress().clone();
        let live = Arc::new(AtomicU64::new(client.connection_generation()));
        let task = tokio::spawn(async move {
            run(&client, ingress, events, live).await;
        });
        settings.wait("settings.get", 1).await;
        app.wait("example-app.ui.titlebar.form", 1).await;
        settings.checkpoint("comp-idle").await;
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        assert_eq!(settings.count("settings.get"), 1);
        assert_eq!(
            observer.registry_reads("comp-idle"),
            1,
            "idle makes no registry read"
        );
        assert_eq!(
            app.count("example-app.ui.titlebar.form"),
            1,
            "idle does not re-read registrations/titlebars"
        );
        settings.settings_changed(2, "first").await;
        // A titlebar event below also orders the settings delivery on the
        // same connection; checkpoint waits for the resulting presentation.
        assert_eq!(settings.count("settings.get"), 1);
        assert_eq!(
            app.count("example-app.ui.titlebar.form"),
            1,
            "settings delivery is not a registry hint"
        );
        assert_eq!(observer.registry_reads("comp-idle"), 1);
        app.publish("example-app.ui.titlebar.changed", json!({}))
            .await;
        app.wait("example-app.ui.titlebar.form", 2).await;
        app.checkpoint("comp-idle").await;
        assert_eq!(app.count("example-app.ui.titlebar.form"), 2);
        let _newcomer = idle_bus::Service::start(&broker, "late-app").await;
        hints.send(idle_bus::Service::registration_hint(&["example-app", "late-app"])).await.unwrap();
        app.checkpoint("comp-idle").await;
        tokio::time::timeout(std::time::Duration::from_secs(8), async {
            while observer.registry_reads("comp-idle") < 2 {
                tokio::task::yield_now().await;
            }
        }).await.unwrap();
        assert_eq!(
            observer.registry_reads("comp-idle"),
            2,
            "one registration hint makes one registry read"
        );
        assert_eq!(
            app.count("example-app.ui.titlebar.form"),
            2,
            "unrelated registration re-read an unchanged target"
        );
        app.client.shutdown().await;
        drop(app);
        app = idle_bus::Service::start(&broker, "example-app").await;
        let _coalesced = idle_bus::Service::start(&broker, "coalesced-app").await;
        hints.send(idle_bus::Service::registration_hint(&["example-app", "late-app", "coalesced-app"])).await.unwrap();
        app.wait("example-app.ui.titlebar.form", 1).await;
        app.checkpoint("comp-idle").await;
        assert_eq!(
            app.count("example-app.ui.titlebar.form"),
            1,
            "coalesced restart fetched target more than once"
        );
        settings.fail_reads.store(1, Ordering::SeqCst);
        settings.settings_changed(3, "restarted").await;
        settings.wait("settings.get", 3).await;
        settings.checkpoint("comp-idle").await;
        settings.assert_read_retry_deadline();
        assert_eq!(
            settings.count("settings.get"),
            3,
            "authority recovery reads once after a fenced failed-read retry"
        );
        let forms = app.count("example-app.ui.titlebar.form");
        broker.bounce();
        settings.wait("settings.get", 4).await;
        app.wait("example-app.ui.titlebar.form", forms + 1).await;
        app.checkpoint("comp-idle").await;
        assert_eq!(app.count("example-app.ui.titlebar.form"), forms + 1, "recovery presentation completed");
        assert_eq!(
            settings.count("settings.get"),
            4,
            "reconnect reads once then stays idle"
        );
        let forms = app.count("example-app.ui.titlebar.form");
        app.publish("example-app.ui.titlebar.changed", json!({}))
            .await;
        app.wait("example-app.ui.titlebar.form", forms + 1).await;
        app.checkpoint("comp-idle").await;
        assert_eq!(
            app.count("example-app.ui.titlebar.form"),
            forms + 1,
            "post-bounce titlebar subscription did not resume exactly once"
        );
        task.abort();
        forward.abort();
        native_forward.abort();
        drop((wiring, starter));
        targets(BTreeSet::new());
    }
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
                screen_off: None,
                services: BTreeSet::from(["unexpected".into()]),
                bars: BTreeMap::new()
            }
            .apply()
        );
        assert_eq!(decor::window::live_menu_services(), services);
        assert_eq!(decor::window::generation(), before);
    }
}
