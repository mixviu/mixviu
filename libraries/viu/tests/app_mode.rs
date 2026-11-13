// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value, json};
use std::{path::PathBuf, sync::{Arc, mpsc}, time::Duration};
use viu::runtime::{Program, Runtime};

fn policy(app: &str, file: &std::path::Path, action: Option<&str>) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let module = root.join("mix/app-mode.mix");
    let change = action.map(|id| format!("$mode.write_mode($file,$mode.choose({}))\n", json!(id))).unwrap_or_default();
    let source = format!("$mode=require({})\n$file={}\n{}$choice={{mode:$mode.read_mode($file)}}\n$model=$mode.project({{}},{},$choice,true)\n$model.actions=$mode.actions($choice.mode)\nsend \"viu-local\" \"scenes.model\" body={{generation:viu_context().generation,value:$model}}", json!(module), json!(file), change, json!(app));
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(8, Arc::new(move || { let _ = tx.send(()); }), None);
    runtime.load(Program { path: root.join("tests/mode-probe.mix"), source, scene: app.into(), citizen: app.into() }).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{app}: {:?}", done.error);
    done.model.unwrap()
}

#[test]
fn app_modes_follow_live_session_override_restart_and_return_to_session() {
    let temporary = tempfile::tempdir().unwrap();
    let file_theme = toolkit::Theme::embedded();
    for app in ["ced", "dopus", "email", "gallery", "monitor", "prefs", "screenshot", "term", "workshop"] {
        let file = temporary.path().join(app).join("mode.json");
        let initial = policy(app, &file, None);
        assert!(initial["theme_choice"]["mode"].is_null());
        let source = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../apps/{app}/{}", if app == "gallery" { "gallery.mix" } else { "app.mix" }))).unwrap();
        assert!(source.contains(&format!("$app_mode.load(\"{app}\")")), "{app} restores shared choice");
        let ctx = egui::Context::default();
        for action in [None, Some("view.mode.dark"), Some("view.mode.light"), Some("view.mode.session")] {
            let model = policy(app, &file, action);
            // A fresh evaluator is a restart: mode remains an explicit choice
            // or null, never the mode that happened to be resolved last time.
            let restarted = policy(app, &file, None);
            assert_eq!(restarted["theme_choice"], model["theme_choice"]);
            for session in ["light", "dark", "light"] {
                let snapshot = json!({"state":{"lost":false,"values":{"appearance.mode":session,"appearance.scheme":"mixos"}}});
                viu::appearance::install_session_choice(&ctx, &snapshot, app, &file_theme, &Default::default(), &model["theme_choice"]).unwrap();
                let expected = model["theme_choice"]["mode"].as_str().unwrap_or(session);
                assert_eq!(viu::appearance::session_theme(&ctx).mode().name(), expected, "{app} {action:?}");
            }
            let actions = model["actions"].as_array().unwrap();
            assert_eq!(actions.iter().filter(|a| a["checked"] == true).count(), 1);
        }
    }
}
