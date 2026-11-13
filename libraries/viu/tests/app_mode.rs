// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Program, Runtime};

fn policy(app: &str, file: &std::path::Path, action: Option<&str>) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let module = root.join("mix/app-mode.mix");
    let change = action
        .map(|id| format!("$mode.write_mode($file,$mode.choose({}))\n", json!(id)))
        .unwrap_or_default();
    let source = format!(
        "$mode=require({})\n$file={}\n{}$choice={{mode:$mode.read_mode($file)}}\n$model=$mode.project({{}},{},$choice,true)\n$model.actions=$mode.actions($choice.mode)\nsend \"viu-local\" \"scenes.model\" body={{generation:viu_context().generation,value:$model}}",
        json!(module),
        json!(file),
        change,
        json!(app)
    );
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(Program {
            path: root.join("tests/mode-probe.mix"),
            source,
            scene: app.into(),
            citizen: app.into(),
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{app}: {:?}", done.error);
    done.model.unwrap()
}

#[test]
fn app_modes_follow_live_session_override_restart_and_return_to_session() {
    let temporary = tempfile::tempdir().unwrap();
    let file_theme = toolkit::Theme::embedded();
    for app in [
        "ced",
        "dopus",
        "email",
        "gallery",
        "monitor",
        "prefs",
        "screenshot",
        "term",
        "workshop",
    ] {
        let file = temporary.path().join(app).join("mode.json");
        let initial = policy(app, &file, None);
        assert!(initial["theme_choice"]["mode"].is_null());
        let source =
            std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
                "../../apps/{app}/{}",
                if app == "gallery" {
                    "gallery.mix"
                } else {
                    "app.mix"
                }
            )))
            .unwrap();
        assert!(
            source.contains(&format!("$app_mode.load(\"{app}\")")),
            "{app} restores shared choice"
        );
        let ctx = egui::Context::default();
        for action in [
            None,
            Some("view.mode.dark"),
            Some("view.mode.light"),
            Some("view.mode.session"),
        ] {
            let model = policy(app, &file, action);
            // A fresh evaluator is a restart: mode remains an explicit choice
            // or null, never the mode that happened to be resolved last time.
            let restarted = policy(app, &file, None);
            assert_eq!(restarted["theme_choice"], model["theme_choice"]);
            for session in ["light", "dark", "light"] {
                let snapshot = json!({"state":{"lost":false,"values":{"appearance.mode":session,"appearance.scheme":"mixos"}}});
                viu::appearance::install_session_choice(
                    &ctx,
                    &snapshot,
                    app,
                    &file_theme,
                    &Default::default(),
                    &model["theme_choice"],
                )
                .unwrap();
                let expected = model["theme_choice"]["mode"].as_str().unwrap_or(session);
                assert_eq!(
                    viu::appearance::session_theme(&ctx).mode().name(),
                    expected,
                    "{app} {action:?}"
                );
            }
            let actions = model["actions"].as_array().unwrap();
            assert_eq!(actions.iter().filter(|a| a["checked"] == true).count(), 1);
        }
    }
}

struct NativeTheme;
impl mix::BusHandler for NativeTheme {
    fn send<'a>(
        &'a self,
        _: &'a str,
        command: &'a str,
        args: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<(i32, mix::value::Value)>> {
        Box::pin(async move {
            let args = viu::runtime::to_json(args)?;
            let body = args.get("body").unwrap_or(&args);
            if command == "native.theme" {
                let choice = viu::appearance::theme_choice(&json!({"mode":body["mode"]})).unwrap();
                let theme = toolkit::Theme::embedded().with_choice(&choice).unwrap();
                return Ok((
                    0,
                    viu::runtime::from_json(
                        json!({"mode":theme.mode().name(),"scheme":theme.scheme().name()}),
                    )?,
                ));
            }
            if command == "native.saved" {
                return Ok((0, viu::runtime::from_json(json!({"schema_version":1,"writable":true,"window":{"theme":{}},"widgets":{"panels":{},"scrolls":{},"groups":{},"open":{}},"windows":{},"app":{"version":1,"value":{}}}))?));
            }
            Ok((
                0,
                viu::runtime::from_json(json!({"bindings":[],"ready":true}))?,
            ))
        })
    }
    fn emit<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
        _: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<()>> {
        Box::pin(async { Ok(()) })
    }
    fn port_exists<'a>(
        &'a self,
        _: &'a str,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<bool>> {
        Box::pin(async { Ok(false) })
    }
    fn next_incoming<'a>(
        &'a self,
    ) -> mix::evaluator::BusFuture<'a, Option<mix::evaluator::IncomingEvent>> {
        Box::pin(async { None })
    }
}

fn next(runtime: &Runtime, wakes: &mpsc::Receiver<()>, app: &str) -> viu::runtime::Completion {
    wakes.recv_timeout(Duration::from_secs(20)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(
        done.error.is_none(),
        "{app}: {:?}\n{}",
        done.error,
        done.stderr
    );
    done
}

#[test]
fn every_app_executes_the_mode_contract_and_restores_its_choice() {
    let home = tempfile::tempdir().unwrap();
    let prefix = home.path().join("prefix");
    std::fs::create_dir_all(prefix.join("bin")).unwrap();
    let mix = std::env::var("MIX_TEST_MIX_EXE").unwrap_or_else(|_| "/opt/mixos/bin/mix".into());
    std::os::unix::fs::symlink(&mix, prefix.join("bin/mix")).unwrap();
    std::os::unix::fs::symlink(
        std::path::Path::new(&mix).parent().unwrap().join("mixd"),
        prefix.join("bin/mixd"),
    )
    .unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "app_mode_child", "--nocapture"])
        .env("HOME", home.path())
        .env("MIX_APP_HOME", home.path())
        .env("MIX_APPS_HOME", home.path())
        .env("XDG_STATE_HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .env("XDG_RUNTIME_DIR", home.path())
        .env("MIX_RUN", home.path())
        .env("MIX_PREFIX", prefix)
        .env("MIX_BUS_SOCKET", home.path().join("absent.sock"))
        .env_remove("MIX_SESSION_FD")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "isolated child driven by the parent test"]
fn app_mode_child() {
    let temporary = tempfile::tempdir().unwrap();
    for app in [
        "ced",
        "dopus",
        "email",
        "gallery",
        "monitor",
        "prefs",
        "screenshot",
        "term",
        "workshop",
    ] {
        let app_home = temporary.path().join(app);
        std::fs::create_dir_all(&app_home).unwrap();
        for (restart, expected) in [(false, None), (true, Some("light"))] {
            let arguments = if app == "email" {
                vec!["--fixture".into()]
            } else if app == "screenshot" {
                vec![
                    "comp".into(),
                    app_home.to_string_lossy().into_owned(),
                    app_home.join("captures").to_string_lossy().into_owned(),
                ]
            } else if app == "dopus" {
                vec!["--profile".into(), app_home.to_string_lossy().into_owned()]
            } else {
                vec![]
            };
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../apps/{app}"));
            let bundle = if app == "dopus" {
                viu::launch::load(&root, &arguments)
            } else {
                viu::bundle::Bundle::load(&root, &arguments)
            }
            .unwrap();
            let handlers = bundle.program.source.clone();
            let (tx, wakes) = mpsc::channel();
            let mut runtime = Runtime::with_services(
                16,
                Arc::new(move || {
                    let _ = tx.send(());
                }),
                Some(Box::new(|| std::rc::Rc::new(NativeTheme))),
                viu::runtime::Services {
                    application_roots: Some(app_home.clone()),
                    calls: Some(Default::default()),
                    timers: Some(Default::default()),
                    documents: Some(Default::default()),
                    rasters: Some(Default::default()),
                    terminals: Some(Default::default()),
                    ui_state: Some(Default::default()),
                    options: json!({"state_home":temporary.path()}),
                    ..Default::default()
                },
            );
            runtime.load(bundle.program).unwrap();
            let initial = next(&runtime, &wakes, app).model.unwrap();
            assert_eq!(
                initial["theme_choice"]["mode"].as_str(),
                expected,
                "{app} restart={restart}"
            );
            if app == "workshop" {
                let legacy = json!({"app_loaded":false,"state":{"app":{"value":{}},"widgets":{"panels":{}},"window":{"theme":{"mode":"dark"}}}});
                runtime.notify("viu.saved.loaded".into(), legacy.clone()).unwrap();
                let restored = next(&runtime, &wakes, app).model.unwrap();
                assert_eq!(restored["theme_choice"]["mode"], if restart { "light" } else { "dark" });
                if !restart {
                    runtime.event("workshop.execute".into(), json!({"id":"view.mode.session"})).unwrap();
                    next(&runtime, &wakes, app);
                    runtime.notify("viu.saved.loaded".into(), legacy).unwrap();
                    assert!(next(&runtime, &wakes, app).model.unwrap()["theme_choice"]["mode"].is_null());
                }
            }
            runtime.event(format!("{app}.actions"), json!({})).unwrap();
            let actions = next(&runtime, &wakes, app).reply.unwrap();
            assert_eq!(actions.rc, 0, "{app}");
            for mode in ["light", "dark", "session"] {
                assert!(
                    actions
                        .body
                        .to_string()
                        .contains(&format!("view.mode.{mode}")),
                    "{app} discovers {mode}"
                );
            }
            for mode in if restart {
                vec!["session"]
            } else {
                vec!["dark", "session", "light"]
            } {
                runtime
                    .event(
                        format!("{app}.execute"),
                        json!({"id":format!("view.mode.{mode}")}),
                    )
                    .unwrap();
                let done = next(&runtime, &wakes, app);
                if let Some(reply) = done.reply {
                    assert_eq!(reply.rc, 0, "{app}: {}", reply.body);
                }
                let model = done.model.unwrap();
                let saved: Value = serde_json::from_slice(&std::fs::read(temporary.path().join("mixviu").join(app).join("mode.json")).unwrap()).unwrap();
                assert_eq!(saved["mode"], model["theme_choice"]["mode"], "{app} owns its mode file");
                assert_eq!(
                    model["theme_choice"]["mode"].as_str(),
                    if mode == "session" { None } else { Some(mode) },
                    "{app}"
                );
                let ctx = egui::Context::default();
                for session in ["light", "dark"] {
                    let snapshot = json!({"appearance":{"scheme":"mixos","mode":session,"style":"own"},"keymap":{},"state":{"lost":false,"error":null,"binding":{"instance":"example","profile":"default"},"incarnation":"mode-fixture","revision":"1","values":{"appearance.mode":session}}});
                    viu::appearance::install_session_choice(
                        &ctx,
                        &snapshot,
                        app,
                        &toolkit::Theme::embedded(),
                        &Default::default(),
                        &model["theme_choice"],
                    )
                    .unwrap();
                    assert_eq!(
                        viu::appearance::session_theme(&ctx).mode().name(),
                        if mode == "session" { session } else { mode },
                        "{app}"
                    );
                    let effective = json!({"scheme":"mixos","mode":if mode=="session" {session} else {mode},"style":"own"});
                    for (command, body) in [
                        ("viu.session.changed", snapshot.clone()),
                        ("viu.session", snapshot),
                        ("viu.theme.changed", effective),
                    ] {
                        if handlers.contains(&format!("on {command}\n")) {
                            runtime.notify(command.into(), body).unwrap();
                            if let Some(changed) = next(&runtime, &wakes, app).model {
                                assert_eq!(
                                    changed["theme_choice"]["mode"], model["theme_choice"]["mode"],
                                    "{app} {command} preserves choice"
                                );
                            }
                        }
                    }
                    let command = if app == "gallery" { "gallery.state".into() } else if app == "term" { "term.session".into() } else { format!("{app}.info") };
                    runtime.event(command, json!({})).unwrap();
                    let info = next(&runtime, &wakes, app).reply.unwrap();
                    assert_eq!(info.rc, 0, "{app}: {}", info.body);
                    assert_eq!(info.body["theme"]["effective"]["mode"], if mode=="session" {session} else {mode}, "{app} effective state readback");
                }
            }
        }
    }
}
